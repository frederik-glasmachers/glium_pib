# glium_pib
Backend for the [glium library](https://github.com/tomaka/glium) which allows it to be used on the raspberry pi without X.

Note:
This library does not provide any [glutin](https://github.com/tomaka/glutin) functionality.
So there is no event mechanism to get mouse or keyboard input from.

[![Build Status](https://travis-ci.org/pentagolo/glium_pib.svg?branch=master)](https://travis-ci.org/pentagolo/glium_pib)

[Online documentation](https://pentagolo.github.io/glium_pib/glium_pib/)

# Example
```rust
#[macro_use] extern crate glium;
extern crate glium_pib;

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
			let system = glium_pib::System::new(Default::default());
			let system = match system {
				Ok(s) => s,
				Err(_) => {
					println!("Failed to use broadcom libraries.");
					return;
				}
			};
			let system = Arc::new(system);
			let facade = glium_pib::create_window_facade(
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

