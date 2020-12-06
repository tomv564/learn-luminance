// use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::{pipeline::PipelineState, context::GraphicsContext as _};
use glfw::{Context, WindowEvent};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::process::exit;
use std::time::Instant;

use luminance_derive::{Semantics, Vertex};
use luminance::tess::Mode;
use luminance::shader::Program;
use luminance::render_state::RenderState;

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
	#[sem(name="position", repr = "[f32; 2]", wrapper = "VertexPosition")]
	Position,
	#[sem(name="color", repr = "[u8; 3]", wrapper="VertexRGB")]
	Color,
}

#[derive(Copy, Clone, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
	#[allow(dead_code)]
	position: VertexPosition,

	#[allow(dead_code)]
	#[vertex(normalized = "true")]
	color: VertexRGB,
}

const VERTICES: [Vertex; 3] = [
  Vertex::new(
    VertexPosition::new([-0.5, -0.5]),
    VertexRGB::new([255, 0, 0]),
  ),
  Vertex::new(
    VertexPosition::new([0.5, -0.5]),
    VertexRGB::new([0, 255, 0]),
  ),
  Vertex::new(
    VertexPosition::new([0., 0.5]),
    VertexRGB::new([0, 0, 255])
  ),
];

const VS_STR: &str = include_str!("vs.glsl");
const FS_STR: &str = include_str!("fs.glsl");

fn main() {
    println!("Hello, world!");
    let dim = WindowDim::Windowed {
    	width: 960,
    	height: 540
    };
    let surface = GlfwSurface::new_gl33("Hello World", WindowOpt::default().set_dim(dim));
    match surface {
    	Ok(surface) => {
    		eprintln!("Surface created");
    		main_loop(surface);
    	}
    	Err(e) => {
    		eprintln!("Cannot create surface:\n{}", e);
    		exit(1);
    	}
    }
}

fn main_loop(mut surface: GlfwSurface) {
	let back_buffer = surface.back_buffer().unwrap();
	let time = Instant::now();
	let triangle = surface
		.new_tess()
		.set_vertices(&VERTICES[..]) // TODO: reference only via a slice?
		.set_mode(Mode::Triangle)
		.build()
		.unwrap();

	let mut program = surface
		.new_shader_program::<VertexSemantics, (), ()>()
		.from_strings(VS_STR, None, None, FS_STR)
		.unwrap()
		.ignore_warnings();

	'app: loop {
		surface.window.glfw.poll_events();
		for (_, event) in surface.events_rx.try_iter() {
			match event {
				WindowEvent::Close => break 'app,
				_ => ()
			}
			match event {
				WindowEvent::Close => break 'app,
				// WindowEvent::Key(Key::Escape, _, Action::Release, _)  => break 'app,
				_ => ()
			}
		}
		let t = time.elapsed().as_millis() as f32 * 1e-3;
		let color = [t.cos(), t.sin(), 0.5, 1.];

		let render = surface.new_pipeline_gate().pipeline(
			&back_buffer,
			&PipelineState::default().set_clear_color(color),
			|_, mut shd_gate| {
				shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
					rdr_gate.render(&RenderState::default(), |mut tess_gate| {
						tess_gate.render(&triangle)
					})
				})
			},
		).assume();

		if render.is_ok() {
			surface.window.swap_buffers();
		} else {
			break 'app;
		}
	}
}