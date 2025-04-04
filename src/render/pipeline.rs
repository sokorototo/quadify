use super::{geometry::Vertex, rgba::Rgba};
use bevy_reflect::Reflect;
use glam::{vec2, vec3};
use miniquad::*;
use std::collections::BTreeMap;

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct GlPipeline(usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
	Triangles,
	Lines,
}

#[derive(Clone, Debug)]
pub struct Uniform {
	name: String,
	uniform_type: UniformType,
	byte_offset: usize,
}

pub struct DrawCall {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,

	pub vertices_count: usize,
	pub indices_count: usize,

	pub clip: Option<(i32, i32, i32, i32)>,
	pub viewport: Option<(i32, i32, i32, i32)>,
	pub texture: Option<miniquad::TextureId>,

	pub model: glam::Mat4,

	pub draw_mode: DrawMode,
	pub pipeline: GlPipeline,
	pub uniforms: Option<Vec<u8>>,
	pub render_pass: Option<RenderPass>,
	pub capture: bool,
}

impl DrawCall {
	pub fn new(
		texture: Option<miniquad::TextureId>,
		model: glam::Mat4,
		draw_mode: DrawMode,
		pipeline: GlPipeline,
		uniforms: Option<Vec<u8>>,
		render_pass: Option<RenderPass>,
		max_vertices: usize,
		max_indices: usize,
	) -> DrawCall {
		DrawCall {
			vertices: vec![Vertex::new(vec3(0.0, 0.0, 0.0), vec2(0.0, 0.0), Rgba::default()); max_vertices],
			indices: vec![0; max_indices],
			vertices_count: 0,
			indices_count: 0,
			viewport: None,
			clip: None,
			texture,
			model,
			draw_mode,
			pipeline,
			uniforms,
			render_pass,
			capture: false,
		}
	}

	pub fn vertices(&self) -> &[Vertex] {
		&self.vertices[0..self.vertices_count]
	}

	pub fn indices(&self) -> &[u16] {
		&self.indices[0..self.indices_count]
	}
}

pub struct GlState {
	pub texture: Option<miniquad::TextureId>,
	pub draw_mode: DrawMode,
	pub clip: Option<(i32, i32, i32, i32)>,
	pub viewport: Option<(i32, i32, i32, i32)>,
	pub model_stack: Vec<glam::Mat4>,
	pub pipeline: Option<GlPipeline>,
	pub depth_test_enable: bool,

	pub break_batching: bool,

	pub render_pass: Option<RenderPass>,
}

impl Default for GlState {
	fn default() -> Self {
		Self {
			clip: None,
			viewport: None,
			texture: None,
			model_stack: vec![glam::Mat4::IDENTITY],
			draw_mode: DrawMode::Triangles,
			pipeline: None,
			break_batching: false,
			depth_test_enable: false,
			render_pass: None,
		}
	}
}

impl GlState {
	pub fn model(&self) -> glam::Mat4 {
		*self.model_stack.last().unwrap()
	}
}

#[derive(Clone)]
pub struct PipelineExt {
	pub pipeline: miniquad::Pipeline,
	pub uniforms: Vec<Uniform>,
	pub uniforms_data: Vec<u8>,
	pub textures: Vec<String>,
	pub textures_data: BTreeMap<String, TextureId>,
}

impl PipelineExt {
	pub fn set_uniform<T>(&mut self, name: &str, uniform: T) {
		let uniform_meta = self.uniforms.iter().find(|Uniform { name: uniform_name, .. }| uniform_name == name);
		if uniform_meta.is_none() {
			#[cfg(feature = "log")]
			bevy_log::warn!("Trying to set non-existing uniform: {}", name);
			return;
		}
		let uniform_meta = uniform_meta.unwrap();
		let uniform_format = uniform_meta.uniform_type;
		let uniform_byte_size = uniform_format.size();
		let uniform_byte_offset = uniform_meta.byte_offset;

		if std::mem::size_of::<T>() != uniform_byte_size {
			#[cfg(feature = "log")]
			bevy_log::warn!("Trying to set uniform {} sized {} bytes value of {} bytes", name, uniform_byte_size, std::mem::size_of::<T>());
			return;
		}

		// ? This part could be questionable
		macro_rules! transmute_uniform {
			($uniform_size:expr, $byte_offset:expr, $n:expr) => {
				if $uniform_size == $n {
					let data: [u8; $n] = unsafe { std::mem::transmute_copy(&uniform) };

					for i in 0..$uniform_size {
						self.uniforms_data[$byte_offset + i] = data[i];
					}
				}
			};
		}
		transmute_uniform!(uniform_byte_size, uniform_byte_offset, 4);
		transmute_uniform!(uniform_byte_size, uniform_byte_offset, 8);
		transmute_uniform!(uniform_byte_size, uniform_byte_offset, 12);
		transmute_uniform!(uniform_byte_size, uniform_byte_offset, 16);
		transmute_uniform!(uniform_byte_size, uniform_byte_offset, 64);
	}
}

#[repr(transparent)]
pub struct PipelineStorage {
	pub pipelines: [Option<PipelineExt>; Self::MAX_PIPELINES],
}

impl PipelineStorage {
	const MAX_PIPELINES: usize = 32;
	const TRIANGLES_PIPELINE: GlPipeline = GlPipeline(0);
	const LINES_PIPELINE: GlPipeline = GlPipeline(1);
	const TRIANGLES_DEPTH_PIPELINE: GlPipeline = GlPipeline(2);
	const LINES_DEPTH_PIPELINE: GlPipeline = GlPipeline(3);

	pub(crate) fn new(ctx: &mut dyn RenderingBackend) -> PipelineStorage {
		let source = ShaderSource::new(shader::VERTEX, shader::FRAGMENT);

		let shader = ctx.new_shader(source, shader::meta()).unwrap();
		let params = PipelineParams {
			color_blend: Some(BlendState::new(
				Equation::Add,
				BlendFactor::Value(BlendValue::SourceAlpha),
				BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
			)),
			..Default::default()
		};

		let mut storage = PipelineStorage { pipelines: Default::default() };

		let triangles_pipeline = storage.make_pipeline(
			ctx,
			shader,
			PipelineParams {
				primitive_type: PrimitiveType::Triangles,
				..params
			},
			vec![],
			vec![],
		);
		assert_eq!(triangles_pipeline, Self::TRIANGLES_PIPELINE);

		let lines_pipeline = storage.make_pipeline(
			ctx,
			shader,
			PipelineParams {
				primitive_type: PrimitiveType::Lines,
				..params
			},
			vec![],
			vec![],
		);
		assert_eq!(lines_pipeline, Self::LINES_PIPELINE);

		let triangles_depth_pipeline = storage.make_pipeline(
			ctx,
			shader,
			PipelineParams {
				depth_write: true,
				depth_test: Comparison::LessOrEqual,
				primitive_type: PrimitiveType::Triangles,
				..params
			},
			vec![],
			vec![],
		);
		assert_eq!(triangles_depth_pipeline, Self::TRIANGLES_DEPTH_PIPELINE);

		let lines_depth_pipeline = storage.make_pipeline(
			ctx,
			shader,
			PipelineParams {
				depth_write: true,
				depth_test: Comparison::LessOrEqual,
				primitive_type: PrimitiveType::Lines,
				..params
			},
			vec![],
			vec![],
		);
		assert_eq!(lines_depth_pipeline, Self::LINES_DEPTH_PIPELINE);

		storage
	}

	pub fn make_pipeline(&mut self, ctx: &mut dyn RenderingBackend, shader: ShaderId, params: PipelineParams, mut uniforms: Vec<(String, UniformType)>, textures: Vec<String>) -> GlPipeline {
		// TODO: Is it possible to create custom pipelines, with custom vertex attributes? Or is it batch-bound?
		let pipeline = ctx.new_pipeline(&[BufferLayout::default()], &Vertex::attributes(), shader, params);

		let id = self.pipelines.iter().position(|p| p.is_none()).expect("Pipelines amount exceeded");
		let mut max_offset = 0;

		for (name, kind) in shader::uniforms().into_iter().rev() {
			uniforms.insert(0, (name.to_owned(), kind));
		}

		let uniforms = uniforms
			.iter()
			.scan(0, |offset, uniform| {
				let uniform_byte_size = uniform.1.size();
				let uniform = Uniform {
					name: uniform.0.clone(),
					uniform_type: uniform.1,
					byte_offset: *offset,
				};
				*offset += uniform_byte_size;
				max_offset = *offset;

				Some(uniform)
			})
			.collect();

		self.pipelines[id] = Some(PipelineExt {
			pipeline,
			uniforms,
			uniforms_data: vec![0; max_offset],
			textures,
			textures_data: BTreeMap::new(),
		});

		GlPipeline(id)
	}

	pub fn get_default_by(&self, draw_mode: DrawMode, depth_enabled: bool) -> GlPipeline {
		match (draw_mode, depth_enabled) {
			(DrawMode::Triangles, false) => Self::TRIANGLES_PIPELINE,
			(DrawMode::Triangles, true) => Self::TRIANGLES_DEPTH_PIPELINE,
			(DrawMode::Lines, false) => Self::LINES_PIPELINE,
			(DrawMode::Lines, true) => Self::LINES_DEPTH_PIPELINE,
		}
	}

	pub fn get_pipeline_mut(&mut self, pip: GlPipeline) -> &mut PipelineExt {
		self.pipelines[pip.0].as_mut().unwrap()
	}

	pub fn delete_pipeline(&mut self, pip: GlPipeline) {
		self.pipelines[pip.0] = None;
	}
}

// TODO: Make Color part of uniform in shaders
pub(crate) mod shader {
	use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

	pub const VERTEX: &str = r#"#version 100
	attribute vec3 position;
	attribute vec2 texcoord;
	attribute vec4 color0;

	varying lowp vec2 uv;
	varying lowp vec4 color;

	uniform mat4 Model;
	uniform mat4 Projection;

	void main() {
		gl_Position = Projection * Model * vec4(position, 1);
		color = color0 / 255.0;
		uv = texcoord;
	}"#;

	pub const FRAGMENT: &str = r#"#version 100
	varying lowp vec4 color;
	varying lowp vec2 uv;

	uniform sampler2D Texture;

	void main() {
		gl_FragColor = color * texture2D(Texture, uv);
	}"#;

	pub fn uniforms() -> Vec<(&'static str, UniformType)> {
		vec![("Projection", UniformType::Mat4), ("Model", UniformType::Mat4), ("_Time", UniformType::Float4)]
	}

	pub fn meta() -> ShaderMeta {
		ShaderMeta {
			images: vec!["Texture".to_string()],
			uniforms: UniformBlockLayout {
				uniforms: uniforms().into_iter().map(|(name, kind)| UniformDesc::new(name, kind)).collect(),
			},
		}
	}
}
