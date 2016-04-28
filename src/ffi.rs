use libc;

pub type DispmanxDisplayHandle = libc::uint32_t;
pub type DispmanxUpdateHandle = libc::uint32_t;
pub type DispmanxResourceHandle = libc::uint32_t;
pub type DispmanxElementHandle = libc::uint32_t;
pub type DispmanxProtection = libc::uint32_t;
pub type DispmanxTransform = libc::c_int;
pub type DispmanxClamp = libc::c_void;
pub type VcDispmanxAlpha = libc::c_void;

#[repr(C)]
pub struct VcRect {
	pub x: libc::int32_t,
	pub y: libc::int32_t,
	pub width: libc::int32_t,
	pub height: libc::int32_t,
}

pub const DISPMANX_PROTECTION_NONE: DispmanxProtection = 0 as DispmanxProtection;
pub const DISPMANX_SUCCESS: libc::c_int = 0 as libc::c_int;
pub const DISPMANX_NO_HANDLE: libc::uint32_t = 0 as libc::uint32_t;

#[repr(C)]
pub struct EGLDispmanxWindow {
	pub element: DispmanxElementHandle,
	pub width: libc::c_int,
	pub height: libc::c_int,
}
pub type EGLNativeDisplayType = *const libc::c_void;
pub type EGLNativeWindowType = *const EGLDispmanxWindow;
pub type EGLConfig = *const libc::c_void;
pub type EGLSurface = *const libc::c_void;
pub type EGLContext = *const libc::c_void;
pub type EGLDisplay = *const libc::c_void;
pub type EGLenum = libc::c_uint;
pub type EGLint = libc::int32_t;
pub type EGLBoolean = libc::c_uint;

pub const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = 0 as EGLNativeDisplayType;
pub const EGL_NO_DISPLAY: EGLDisplay = 0 as EGLDisplay; 
pub const EGL_ALPHA_SIZE: GLenum = 0x3021; 
pub const EGL_BLUE_SIZE: GLenum = 0x3022; 
pub const EGL_GREEN_SIZE: GLenum = 0x3023; 
pub const EGL_RED_SIZE: GLenum = 0x3024; 
pub const EGL_DEPTH_SIZE: GLenum = 0x3025;
pub const EGL_SURFACE_TYPE: GLenum = 0x3033;
pub const EGL_WINDOW_BIT: GLenum = 0x0004;
pub const EGL_NONE: GLenum = 0x3038;
pub const EGL_OPENGL_ES_API: GLenum = 0x30A0; 
pub const EGL_NO_CONTEXT: EGLContext = 0 as EGLContext; 
pub const EGL_CONTEXT_CLIENT_VERSION: GLenum = 0x3098;
pub const EGL_NO_SURFACE: EGLSurface = 0 as EGLSurface;

pub type GLenum = libc::c_uint;

shared_library!(LibBcmHost,
	pub fn bcm_host_init(),
	pub fn bcm_host_deinit(),
	pub fn graphics_get_display_size(
		display_number: libc::uint16_t,
		width: *mut libc::uint32_t, height: *mut libc::uint32_t
	) -> libc::int32_t,
	pub fn vc_dispmanx_display_open(device: libc::uint32_t) -> DispmanxDisplayHandle,
	pub fn vc_dispmanx_display_close(handle: DispmanxDisplayHandle) -> libc::c_int,
	pub fn vc_dispmanx_update_start(priority: libc::int32_t) -> DispmanxUpdateHandle,
	pub fn vc_dispmanx_update_submit_sync(update: DispmanxUpdateHandle) -> libc::c_int,
	pub fn vc_dispmanx_element_add(
		update: DispmanxUpdateHandle, display: DispmanxDisplayHandle,
		layer: libc::int32_t, dest_rect: *const VcRect, src: DispmanxResourceHandle,
		src_rect: *const VcRect, protection: DispmanxProtection,
		alpha: *mut VcDispmanxAlpha,
		clamp: *mut DispmanxClamp, transform: DispmanxTransform
	) -> DispmanxElementHandle,
	pub fn vc_dispmanx_element_remove(update: DispmanxUpdateHandle, element: DispmanxElementHandle) -> libc::c_int,
);

shared_library!(LibGLESv2,
	pub fn glGetError() -> GLenum,
);

shared_library!(LibEGL,
	pub fn eglGetDisplay(native_display: EGLNativeDisplayType) -> EGLDisplay,
	pub fn eglInitialize(display: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean,
	pub fn eglTerminate(display: EGLDisplay) -> EGLBoolean,
	pub fn eglChooseConfig(display: EGLDisplay, attrib_list: *const EGLint, configs: *mut EGLConfig, config_size: EGLint, num_config: *mut EGLint) -> EGLBoolean,
	pub fn eglBindAPI(api: EGLenum) -> EGLBoolean,
	pub fn eglCreateContext(display: EGLDisplay, config: EGLConfig, share_context: EGLContext, attrib_list: *const EGLint) -> EGLContext,
	pub fn eglDestroyContext(display: EGLDisplay, context: EGLContext) -> EGLBoolean,
	pub fn eglCreateWindowSurface(display: EGLDisplay, config: EGLConfig, win: EGLNativeWindowType, attrib_list: *const EGLint) -> EGLSurface,
	pub fn eglDestroySurface(display: EGLDisplay, surface: EGLSurface) -> EGLBoolean,
	pub fn eglMakeCurrent(display: EGLDisplay, draw: EGLSurface, read: EGLSurface, context: EGLContext) -> EGLBoolean,
	pub fn eglSwapBuffers(display: EGLDisplay, draw: EGLSurface) -> EGLBoolean,
	pub fn eglGetCurrentContext() -> EGLContext,
);

