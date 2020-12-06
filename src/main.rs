// use glfw::{Action, Context as _, Key, WindowEvent};
use glfw::{Context, WindowEvent};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use cgmath::{perspective, EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use luminance_derive::{Semantics, Vertex, UniformInterface};
use luminance_front::pipeline::PipelineState;
use luminance_front::context::GraphicsContext;
use luminance_front::render_state::RenderState;
use luminance_front::tess::{Interleaved, Mode, Tess, TessError};
use luminance_front::Backend;
use luminance_front::shader::Uniform;
use std::process::exit;
use std::fs::File;
use std::io::Read as _;
use std::{collections::HashMap, path::Path, time::Instant};
use try_guard::verify;
use wavefront_obj::obj;




#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
	#[sem(name="position", repr = "[f32; 3]", wrapper = "VertexPosition")]
	Position,
	#[sem(name="normal", repr="[f32; 3]", wrapper = "VertexNormal")]
	Normal,
}

#[derive(Copy, Clone, Debug, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
	#[allow(dead_code)]
	position: VertexPosition,
	#[allow(dead_code)]
	normal: VertexNormal,
}

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
	#[uniform(unbound)]
	projection: Uniform<[[f32; 4]; 4]>,
	#[uniform(unbound)]
	view: Uniform<[[f32; 4]; 4]>,
}
type VertexIndex = u32;

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

const VS_STR: &str = include_str!("vs.glsl");
const FS_STR: &str = include_str!("fs.glsl");
const BLURFS_STR: &str = include_str!("blurfs.glsl");

#[derive(Debug)]
struct Obj {
  vertices: Vec<Vertex>,
  indices: Vec<VertexIndex>,
}

impl Obj {
  fn to_tess<C>(
    self,
    surface: &mut C,
  ) -> Result<Tess<Vertex, VertexIndex, (), Interleaved>, TessError>
  where
    C: GraphicsContext<Backend = Backend>,
  {
    surface
      .new_tess()
      .set_mode(Mode::Triangle)
      .set_vertices(self.vertices)
      .set_indices(self.indices)
      .build()
  }

  fn load<P>(path: P) -> Result<Self, String>
  where
    P: AsRef<Path>,
  {
    let file_content = {
      let mut file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
      let mut content = String::new();
      file.read_to_string(&mut content).unwrap();
      content
    };
    let obj_set = obj::parse(file_content).map_err(|e| format!("cannot parse: {:?}", e))?;
    let objects = obj_set.objects;

    verify!(objects.len() == 1).ok_or("expecting a single object".to_owned())?;

    let object = objects.into_iter().next().unwrap();

    verify!(object.geometry.len() == 1).ok_or("expecting a single geometry".to_owned())?;

    let geometry = object.geometry.into_iter().next().unwrap();

    println!("loading {}", object.name);
    println!("{} vertices", object.vertices.len());
    println!("{} shapes", geometry.shapes.len());

    // build up vertices; for this to work, we remove duplicated vertices by putting them in a
    // map associating the vertex with its ID
    let mut vertex_cache: HashMap<obj::VTNIndex, VertexIndex> = HashMap::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<VertexIndex> = Vec::new();

    for shape in geometry.shapes {
      if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
        for key in &[a, b, c] {
          if let Some(vertex_index) = vertex_cache.get(key) {
            indices.push(*vertex_index);
          } else {
            let p = object.vertices[key.0];
            let n = object.normals[key.2.ok_or("missing normal for a vertex".to_owned())?];
            let position = VertexPosition::new([p.x as f32, p.y as f32, p.z as f32]);
            let normal = VertexNormal::new([n.x as f32, n.y as f32, n.z as f32]);
            let vertex = Vertex { position, normal };
            let vertex_index = vertices.len() as VertexIndex;

            vertex_cache.insert(*key, vertex_index);
            vertices.push(vertex);
            indices.push(vertex_index);
          }
        }
      } else {
        return Err("unsupported non-triangle shape".to_owned());
      }
    }

    Ok(Obj { vertices, indices })
  }
}


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
	let [width, height] = back_buffer.size();
	let projection = perspective(FOVY, width as f32 / height as f32, Z_NEAR, Z_FAR);
	let view = Matrix4::<f32>::look_at(Point3::new(2., 2., 2.), Point3::origin(), Vector3::unit_y());
	let mesh = Obj::load("suzanne.obj").unwrap().to_tess(&mut surface).unwrap();
	let time = Instant::now();

	let mut program = surface
		.new_shader_program::<VertexSemantics, (), ShaderInterface>()
		.from_strings(VS_STR, None, None, FS_STR)
		.unwrap()
		.ignore_warnings();

	let mut blurprogram = surface
		.new_shader_program::<VertexSemantics, (), ()>()
		.from_strings(None, None, None, BLURFS_STR)
		.unwrap()
		.ignore_warnings()

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
				shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
					iface.set(&uni.projection, projection.into());
					iface.set(&uni.view, view.into());

					rdr_gate.render(&RenderState::default(), |mut tess_gate| {
						tess_gate.render(&mesh)
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