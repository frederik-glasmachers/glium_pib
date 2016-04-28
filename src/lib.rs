/*!
Backend for the glium library which allows it to be used on the raspberry pi without X.

Note:
This library does not provide any glutin functionality.
So there is no event mechanism to get mouse or keyboard input from.

# Example
```no_run
#[macro_use] extern crate glium;
extern crate glium_pi_backend;

#[derive(Copy, Clone)]
struct Vertex {
	position: [f32; 2],
}
implement_vertex!(Vertex, position);

fn main() {
	use std::default::Default;
	use std::sync::Arc;
	use std::rc::Rc;
	
	use glium::DisplayBuild;
	use glium::Surface;
	use glium::backend::Facade;

	// Create a glium facade (window to draw on).
	let facade = glium::glutin::WindowBuilder::new()
		.with_dimensions(1024, 768)
		.with_title(format!("Hello world"))
		.build_glium();
	let facade: Rc<glium::backend::Context> = match facade {
		Ok(f) => f.get_context().clone(),
		Err(_) => {
			println!("Failed to create X window.");
			println!("Trying to use broadcom libraries for the raspberry pi.");
			let system = glium_pi_backend::System::new(Default::default());
			let system = match system {
				Ok(s) => s,
				Err(_) => {
					println!("Failed to use broadcom libraries.");
					return;
				}
			};
			let system = Arc::new(system);
			let facade = glium_pi_backend::create_window_facade(
				&system,
				&std::default::Default::default()
			);
			match facade {
				Ok(f) => f,
				Err(_) => {
					println!("Failed to use broadcom libraries.");
					return;
				},
			}
		},
	};

	// Create vertex buffer and index buffer as normal.
	let vertex1 = Vertex { position: [-0.5, -0.5] };
	let vertex2 = Vertex { position: [ 0.0,  0.5] };
	let vertex3 = Vertex { position: [ 0.5, -0.25] };
	let shape = vec![vertex1, vertex2, vertex3];
	let vertex_buffer = glium::VertexBuffer::new(&facade, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	// The raspberry pi has only basic GLSL support.
	let vertex_shader_src = r#"
		attribute vec2 position;
		void main() {
			gl_Position = vec4(position, 0.0, 1.0);
		}
	"#;
	let fragment_shader_src = r#"
		void main() {
			gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
		}
	"#;
	let program = glium::Program::from_source(
		&facade,
		vertex_shader_src,
		fragment_shader_src,
		None
	).unwrap();

	// Instead of using window.draw we create the frame on our own.
	let mut target = glium::Frame::new(
		facade.clone(),
		facade.get_framebuffer_dimensions()
	);
	
	// Draw as usual.
	target.clear_color(0.0, 0.0, 1.0, 1.0);
	target.draw(
		&vertex_buffer,
		&indices,
		&program,
		&glium::uniforms::EmptyUniforms,
		&Default::default()
	).unwrap();
	target.finish().unwrap();
	
	std::thread::sleep(std::time::Duration::new(5, 0));
}
```
*/

#[macro_use] extern crate shared_library;
#[macro_use] extern crate glium;
extern crate libc;

mod ffi;
mod error;
mod config;

use std::sync::atomic::{Ordering, AtomicBool, ATOMIC_BOOL_INIT};
use std::sync::Mutex;
use std::rc::Rc;
use std::sync::Arc;
use std::ops::Deref;
use std::default::Default;
use std::path::Path;

pub use error::Error;
use error::gl_error;
pub use config::{LibDir, Display, ColorBits, DepthBits, WindowConfig};

use shared_library::dynamic_library::DynamicLibrary;


// System singleton guard (The system may be created just once during the whole lifetime of the process).
static SYSTEM_SINGLETON_GUARD: AtomicBool = ATOMIC_BOOL_INIT;
/// Process wide shared data. Only one instance may be created per process.
pub struct System {
	/// The library directory.
	lib_dir: LibDir,
	// Bcm-Host library.
	lib_bcm_host: ffi::LibBcmHost,
	// GLES library.
	lib_glesv2: ffi::LibGLESv2,
	// Dynamic library used to enable glium to load any symbol.
	dlib_glesv2: DynamicLibrary,
	// EGL library.
	lib_egl: ffi::LibEGL,
	/// Egl display.
	egl_display: ffi::EGLDisplay,
	/// Mutex used to protect potential unsynchronized functionality of the ffi.
	mutex: Mutex<()>,
}
impl System {
	/// Create a new system using the libraries from the library directory specified. This should only be called once per process.
	pub fn new(lib_dir: LibDir) -> Result<Self, Error> {

		// Create the mutex.
		let mutex: Mutex<()> = Mutex::new(());
		
		// Load the libraries needed.
		let lib_bcm_host = try!(
			ffi::LibBcmHost::open(&lib_dir.join("libbcm_host.so")).map_err(|e| { Error::Sl(e) })
		);
		let lib_glesv2 = try!(
			ffi::LibGLESv2::open(&lib_dir.join("libGLESv2.so")).map_err(|e| { Error::Sl(e) })
		);
		let dlib_glesv2 = try!(
			DynamicLibrary::open(Some(&lib_dir.join("libGLESv2.so"))).map_err(|e| { Error::Dl(e) })
		);
		let lib_egl = try!(
			ffi::LibEGL::open(&lib_dir.join("libEGL.so")).map_err(|e| { Error::Sl(e) })
		);

		assert!(SYSTEM_SINGLETON_GUARD.swap(true, Ordering::AcqRel) == false);

		let egl_display = unsafe {
		
			// Lock the mutex.
			let _ = mutex.lock();

			(lib_bcm_host.bcm_host_init)();

			// Get the default egl display.
			let egl_display = (lib_egl.eglGetDisplay)(ffi::EGL_DEFAULT_DISPLAY);
			if egl_display == ffi::EGL_NO_DISPLAY { return Err(Error::Fn("eglGetDisplay")); }
			try!{gl_error(&lib_glesv2, "eglGetDisplay")};

			// Initialize EGL.
			if (lib_egl.eglInitialize)(egl_display, 0 as *mut ffi::EGLint, 0 as *mut ffi::EGLint) == 0 { return Err(Error::Fn("eglInitialize")); }
			try!{gl_error(&lib_glesv2, "eglInitialize")};

			egl_display
		};

		// Create and return system.
		Ok(System {
			lib_dir: lib_dir,
			lib_bcm_host: lib_bcm_host,
			lib_glesv2: lib_glesv2,
			dlib_glesv2: dlib_glesv2,
			lib_egl: lib_egl,
			egl_display: egl_display,
			mutex: mutex,
		})
	}
	/// Get the size of a display.
	unsafe fn display_size_no_lock(&self, display: Display) -> Result<(u32, u32), Error> {
		// Get display size.
		let mut width: libc::uint32_t = 0;
		let mut height: libc::uint32_t = 0;
		let res = (self.lib_bcm_host.graphics_get_display_size)(
			display.index(),
			&mut width as *mut libc::uint32_t,
			&mut height as *mut libc::uint32_t
		);
		if res < 0 { return Err(Error::Fn("graphics_get_display_size")); }
		Ok((width as u32, height as u32))
	}
	/// Get the size of a display.
	pub fn display_size(&self, display: Display) -> Result<(u32, u32), Error> {
		let _ = self.mutex.lock();
		unsafe { self.display_size_no_lock(display) }
	}
	/// The library directory in use.
	pub fn lib_dir(&self) -> &Path {
		self.lib_dir.deref()
	}
}
impl Drop for System {
	fn drop(&mut self) {
		unsafe {
			// Finelize EGL.
			if self.egl_display != ffi::EGL_NO_DISPLAY {
				assert!((self.lib_egl.eglTerminate)(self.egl_display) != 0);
				self.egl_display = ffi::EGL_NO_DISPLAY;
			}
			// Finalize 
			(self.lib_bcm_host.bcm_host_deinit)();
		}
		//SYSTEM_SINGLETON_GUARD.store(false, Ordering::Release);
	}
}
unsafe impl Sync for System {}


/// A (fullscreen) window.
pub struct Window<S> where S: Deref<Target=System> {
	/// The system.
	pub system: S,
	/// EGL context.
	egl_context: ffi::EGLContext,
	/// Dispmanx display.
	dispmanx_display: ffi::DispmanxDisplayHandle,
	/// Egl-Dispmanx window.
	egl_dispmanx_window: Box<ffi::EGLDispmanxWindow>,
	/// EGL surface.
	egl_surface: ffi::EGLSurface,
}
impl<S> Window<S> where S: Deref<Target=System> {
	/// Create a window.
	pub fn new(system: S, config: &WindowConfig) -> Result<Self, Error> {
		unsafe {
			let mut window = Window {
				system: system,
				egl_context: 0 as ffi::EGLContext,
				dispmanx_display: ffi::DISPMANX_NO_HANDLE,
				egl_dispmanx_window: Box::new(ffi::EGLDispmanxWindow {
					element: ffi::DISPMANX_NO_HANDLE,
					width: 0,
					height: 0,
				}),
				egl_surface: 0 as ffi::EGLSurface,
			};

			{
				// Lock the mutex
				let _ = window.system.mutex.lock();
		
				// Choose a EGL-config
				let egl_config = {
					let mut attribute_list: [ffi::EGLint; 13] = [
						ffi::EGL_SURFACE_TYPE as ffi::EGLint, ffi::EGL_WINDOW_BIT as ffi::EGLint,
						ffi::EGL_RED_SIZE as ffi::EGLint, config.red.0 as ffi::EGLint,
						ffi::EGL_GREEN_SIZE as ffi::EGLint, config.green.0 as ffi::EGLint,
						ffi::EGL_BLUE_SIZE as ffi::EGLint, config.blue.0 as ffi::EGLint,
						ffi::EGL_NONE as ffi::EGLint, ffi::EGL_NONE as ffi::EGLint,
						ffi::EGL_NONE as ffi::EGLint, ffi::EGL_NONE as ffi::EGLint,
						ffi::EGL_NONE as ffi::EGLint,
					];
					let mut attribute_list_size = 9;
					match config.alpha.as_ref() {
						Some(alpha) => {
							attribute_list[attribute_list_size + 0] = ffi::EGL_ALPHA_SIZE as ffi::EGLint;
							attribute_list[attribute_list_size + 1] = alpha.0 as ffi::EGLint;
							attribute_list_size += 2;
						},
						None => {},
					}
					match config.depth.as_ref() {
						Some(depth) => {
							attribute_list[attribute_list_size + 0] = ffi::EGL_DEPTH_SIZE as ffi::EGLint;
							attribute_list[attribute_list_size + 1] = depth.0 as ffi::EGLint;
						},
						None => {},
					}
					let mut egl_config: ffi::EGLConfig = 0 as ffi::EGLConfig;
					let mut egl_num_config: ffi::EGLint = 1;
					if (window.system.lib_egl.eglChooseConfig)(window.system.egl_display, &attribute_list as *const ffi::EGLint, &mut egl_config as *mut ffi::EGLConfig, 1, &mut egl_num_config as *mut ffi::EGLint) == 0 { return Err(Error::Fn("eglChooseConfig")); }
					try!{gl_error(&window.system.lib_glesv2, "eglInitialize")};
			
					egl_config
				};

				// Bind GLES api.
				if (window.system.lib_egl.eglBindAPI)(ffi::EGL_OPENGL_ES_API) == 0 { return Err(Error::Fn("eglBindAPI")); }
				try!{gl_error(&window.system.lib_glesv2, "eglBindAPI")};

				// Create a GLES context with client version 2. 
				let context_attributes: [ffi::EGLint; 3] = [
					ffi::EGL_CONTEXT_CLIENT_VERSION as ffi::EGLint, 2,
					ffi::EGL_NONE as ffi::EGLint
				];
				window.egl_context = (window.system.lib_egl.eglCreateContext)(window.system.egl_display, egl_config, ffi::EGL_NO_CONTEXT, &context_attributes as *const ffi::EGLint);
				if window.egl_context == ffi::EGL_NO_CONTEXT { return Err(Error::Fn("eglCreateContext")); }
				try!{gl_error(&window.system.lib_glesv2, "eglCreateContext")};

				// Get the size of the display.
				let (dest_width, dest_height) = try!(window.system.display_size_no_lock(config.display));
				// The selected surface size.
				let (src_width, src_height) = config.surface_size.unwrap_or((dest_width, dest_height));
		
				window.dispmanx_display = (window.system.lib_bcm_host.vc_dispmanx_display_open)(0);
				if window.dispmanx_display == ffi::DISPMANX_NO_HANDLE { return Err(Error::Fn("vc_dispmanx_display_open")); } 
				let dispmanx_update = (window.system.lib_bcm_host.vc_dispmanx_update_start)(0);
				if dispmanx_update == ffi::DISPMANX_NO_HANDLE { return Err(Error::Fn("vc_dispmanx_update_start")); } 
				let dispmanx_element = {
					let src_rect = ffi::VcRect {
						x: 0,
						y: 0,
						width: src_width as libc::int32_t,
						height: src_height as libc::int32_t,
					};
					let dest_rect = ffi::VcRect {
						x: 0,
						y: 0,
						width: (dest_width << 16) as libc::int32_t,
						height: (dest_height << 16) as libc::int32_t,
					};
					(window.system.lib_bcm_host.vc_dispmanx_element_add)(
						dispmanx_update,
						window.dispmanx_display,
						0, &dest_rect as *const ffi::VcRect,
						0, &src_rect as *const ffi::VcRect,
						ffi::DISPMANX_PROTECTION_NONE,
						0 as *mut ffi::VcDispmanxAlpha,
						0 as *mut ffi::DispmanxClamp,
						0
					)
				};
				if dispmanx_element == ffi::DISPMANX_NO_HANDLE { return Err(Error::Fn("vc_dispmanx_element_add")); }
				if (window.system.lib_bcm_host.vc_dispmanx_update_submit_sync)(dispmanx_update) != ffi::DISPMANX_SUCCESS { return Err(Error::Fn("vc_dispmanx_update_submit_sync")); }
				try!{gl_error(&window.system.lib_glesv2, "vc_dispmanx_update_submit_sync")};

				window.egl_dispmanx_window.element = dispmanx_element;
				window.egl_dispmanx_window.width = src_width as libc::c_int;
				window.egl_dispmanx_window.height = src_height as libc::c_int;

				window.egl_surface = (window.system.lib_egl.eglCreateWindowSurface)(window.system.egl_display, egl_config, window.egl_dispmanx_window.as_ref() as ffi::EGLNativeWindowType, 0 as *const ffi::EGLint);
				if window.egl_surface == ffi::EGL_NO_SURFACE { return Err(Error::Fn("eglCreateWindowSurface")); }
				try!{gl_error(&window.system.lib_glesv2, "eglCreateWindowSurface")};
		
				if (window.system.lib_egl.eglMakeCurrent)(window.system.egl_display, window.egl_surface, window.egl_surface, window.egl_context) == 0 { return Err(Error::Fn("eglMakeCurrent")); }
				try!{gl_error(&window.system.lib_glesv2, "eglMakeCurrent")};
			}
			
			Ok(window)
		}
	}
}
impl<S> Drop for Window<S> where S: Deref<Target=System> {
	fn drop(&mut self) {
		let _ = self.system.mutex.lock();
		unsafe {
			if self.egl_surface != ffi::EGL_NO_SURFACE {
				assert!((self.system.lib_egl.eglMakeCurrent)(self.system.egl_display, ffi::EGL_NO_SURFACE, ffi::EGL_NO_SURFACE, ffi::EGL_NO_CONTEXT) != 0);
				assert!((self.system.lib_egl.eglDestroySurface)(self.system.egl_display, self.egl_surface) != 0);
				self.egl_surface = ffi::EGL_NO_SURFACE;
			}
			if self.egl_dispmanx_window.element != ffi::DISPMANX_NO_HANDLE {
				let update = (self.system.lib_bcm_host.vc_dispmanx_update_start)(0);
				assert!(update != ffi::DISPMANX_NO_HANDLE);
				assert!((self.system.lib_bcm_host.vc_dispmanx_element_remove)(update, self.egl_dispmanx_window.element) == ffi::DISPMANX_SUCCESS);
				assert!((self.system.lib_bcm_host.vc_dispmanx_update_submit_sync)(update) == ffi::DISPMANX_SUCCESS);
				self.egl_dispmanx_window.element = ffi::DISPMANX_NO_HANDLE;
			}
			if self.dispmanx_display != ffi::DISPMANX_NO_HANDLE {
				assert!((self.system.lib_bcm_host.vc_dispmanx_display_close)(self.dispmanx_display) == ffi::DISPMANX_SUCCESS);
				self.dispmanx_display = ffi::DISPMANX_NO_HANDLE;
			}
			if self.egl_context != ffi::EGL_NO_CONTEXT {
				assert!((self.system.lib_egl.eglDestroyContext)(self.system.egl_display, self.egl_context) != 0);
				self.egl_context = ffi::EGL_NO_CONTEXT;
			}
		}
	}
}
unsafe impl<S> glium::backend::Backend for Window<S> where S: Deref<Target=System> {
	fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
		if unsafe { (self.system.lib_egl.eglSwapBuffers)(self.system.egl_display, self.egl_surface) } == 0 { panic!("eglSwapBuffers failed"); }
		Ok(())
	}
	unsafe fn get_proc_address(&self, symbol: &str) -> *const std::os::raw::c_void {
		//println!("get_proc_address({})", symbol);
		match self.system.dlib_glesv2.symbol::<std::os::raw::c_void>(symbol) {
			Err(_) => std::ptr::null(),
			Ok(a) => a
		}
	}
	fn get_framebuffer_dimensions(&self) -> (u32, u32) {
		let win = self.egl_dispmanx_window.as_ref();
		(win.width as u32, win.height as u32)
	}
	fn is_current(&self) -> bool {
		unsafe { (self.system.lib_egl.eglGetCurrentContext)() == self.egl_context }
	}
	/// Makes the OpenGL context the current context in the current thread.
	unsafe fn make_current(&self) {
		if (self.system.lib_egl.eglMakeCurrent)(self.system.egl_display, self.egl_surface, self.egl_surface, self.egl_context) == 0 { panic!("eglMakeCurrent failed"); }
	}
}
/// Creates a new glium facade.
pub fn create_window_facade(system: &Arc<System>, config: &WindowConfig) -> Result<Rc<glium::backend::Context>, glium::GliumCreationError<Error>> {
	let window = Rc::new(try!(Window::new(system.clone(), config).map_err(|e| { glium::GliumCreationError::BackendCreationError(e) })));
	unsafe { glium::backend::Context::new::<Rc<Window<Arc<System>>>, Error>(window, true, Default::default()) }
}

