use bevy_ecs::system::{NonSendMut, Query, Res, Resource};
use glam::{vec2, vec3};
use miniquad::*;
use miniquad::{window, PassAction, RenderingBackend as MqdRenderingBackend};

use self::geometry::Vertex;
use self::material::Material;
use self::rgba::Rgba;
use crate::window::state;

use super::render::{material::*, pipeline::*};

pub mod camera;
pub mod geometry;
pub mod material;
pub mod pipeline;
pub mod rgba;

/// Miniquad rendering backend object.
pub struct RenderingBackend {
	backend: Box<dyn MqdRenderingBackend>,
	start_time: f64,

	white_texture: miniquad::TextureId,

	pipelines: pipeline::PipelineStorage,
	max_vertices: usize,
	max_indices: usize,

	state: GlState,
	draw_calls: Vec<DrawCall>,
	draw_calls_count: usize,
	draw_call_bindings: Vec<miniquad::Bindings>,
}

// For ease of use
impl std::ops::Deref for RenderingBackend {
	type Target = dyn MqdRenderingBackend;

	fn deref(&self) -> &Self::Target {
		&*self.backend
	}
}

impl std::ops::DerefMut for RenderingBackend {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.backend
	}
}

impl Default for RenderingBackend {
	fn default() -> Self {
		Self::new()
	}
}

impl RenderingBackend {
	pub fn new() -> Self {
		let mut backend = window::new_rendering_backend();

		let white_texture = backend.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);
		let pipelines = pipeline::PipelineStorage::new(&mut *backend);

		Self {
			backend,
			start_time: miniquad::date::now(),

			white_texture,

			pipelines,
			max_vertices: 10000,
			max_indices: 5000,

			state: GlState::default(),
			draw_calls: Vec::with_capacity(200),
			draw_call_bindings: Vec::with_capacity(64),
			draw_calls_count: 0,
		}
	}

	pub fn make_pipeline(&mut self, shader: miniquad::ShaderSource, params: PipelineParams, uniforms: Vec<(String, UniformType)>, textures: Vec<String>) -> Result<GlPipeline, ShaderError> {
		let mut shader_meta: ShaderMeta = pipeline::shader::meta();

		for uniform in &uniforms {
			shader_meta.uniforms.uniforms.push(UniformDesc::new(&uniform.0, uniform.1));
		}

		for texture in &textures {
			if texture == "Texture" {
				panic!("you can't use name `Texture` for your texture. This name is reserved for the texture that will be drawn with that material");
			}
			shader_meta.images.push(texture.clone());
		}
		let shader = self.backend.new_shader(shader, shader_meta)?;

		Ok(self.pipelines.make_pipeline(&mut *self.backend, shader, params, uniforms, textures))
	}

	/// Tries to compile shaders and create a pipeline, and on success will return a new [`Material`]
	pub fn request_material(&mut self, shader: ShaderSource, params: MaterialParams) -> Result<Material, ShaderError> {
		match self.make_pipeline(shader, params.pipeline_params, params.uniforms, params.textures) {
			Ok(pipeline) => Ok(Material { pipeline }),
			Err(err) => Err(err),
		}
	}

	pub fn remove_material(&mut self, material: Material) {
		self.delete_pipeline(material.pipeline);
	}

	/// Clears all draw calls and clears the screen with specified color
	pub fn clear(&mut self, color: Rgba) {
		let col = color.to_float();
		let clear = PassAction::clear_color(col.x, col.y, col.z, col.w);

		if let Some(current_pass) = self.state.render_pass {
			self.backend.begin_pass(Some(current_pass), clear);
		} else {
			self.backend.begin_default_pass(clear);
		}
		self.backend.end_render_pass();

		self.clear_draw_calls();
	}

	/// Reset only draw calls state
	pub fn clear_draw_calls(&mut self) {
		self.draw_calls_count = 0;
	}

	/// Reset internal state to known default
	pub fn reset(&mut self) {
		self.state.clip = None;
		self.state.texture = None;
		self.state.model_stack = vec![glam::Mat4::IDENTITY];

		self.draw_calls_count = 0;
	}

	/// Flushes all the draw calls, applying the specified projection as uniform camera.
	pub fn draw(&mut self, projection: glam::Mat4) {
		let white_texture = self.white_texture;

		for _ in 0..self.draw_calls.len() - self.draw_call_bindings.len() {
			let vertex_buffer = self.backend.new_buffer(BufferType::VertexBuffer, BufferUsage::Stream, BufferSource::empty::<Vertex>(self.max_vertices));
			let index_buffer = self.backend.new_buffer(BufferType::IndexBuffer, BufferUsage::Stream, BufferSource::empty::<u16>(self.max_indices));
			let bindings = Bindings {
				vertex_buffers: vec![vertex_buffer],
				index_buffer,
				images: vec![white_texture],
			};

			self.draw_call_bindings.push(bindings);
		}
		assert_eq!(self.draw_call_bindings.len(), self.draw_calls.len());

		let (screen_width, screen_height) = miniquad::window::screen_size();
		let time = (miniquad::date::now() - self.start_time) as f32;
		let time = glam::vec4(time, time.sin(), time.cos(), 0.);

		for (dc, bindings) in self.draw_calls[0..self.draw_calls_count].iter_mut().zip(self.draw_call_bindings.iter_mut()) {
			let pipeline = self.pipelines.get_pipeline_mut(dc.pipeline);

			let (width, height) = if let Some(render_pass) = dc.render_pass {
				let render_texture = self.backend.render_pass_texture(render_pass);
				let (width, height) = self.backend.texture_size(render_texture);
				(width, height)
			} else {
				(screen_width, screen_height)
			};

			if let Some(render_pass) = dc.render_pass {
				self.backend.begin_pass(Some(render_pass), PassAction::Nothing);
			} else {
				self.backend.begin_default_pass(PassAction::Nothing);
			}

			self.backend.buffer_update(bindings.vertex_buffers[0], BufferSource::slice(dc.vertices()));
			self.backend.buffer_update(bindings.index_buffer, BufferSource::slice(dc.indices()));

			bindings.images[0] = dc.texture.unwrap_or(white_texture);
			bindings.images.resize(1 + pipeline.textures.len(), white_texture);

			for (pos, name) in pipeline.textures.iter().enumerate() {
				if let Some(texture) = pipeline.textures_data.get(name).copied() {
					bindings.images[1 + pos] = texture;
				}
			}

			self.backend.apply_pipeline(&pipeline.pipeline);
			if let Some((x, y, w, h)) = dc.viewport {
				self.backend.apply_viewport(x, y, w, h);
			} else {
				self.backend.apply_viewport(0, 0, width as i32, height as i32);
			}
			if let Some(clip) = dc.clip {
				self.backend.apply_scissor_rect(clip.0, height as i32 - (clip.1 + clip.3), clip.2, clip.3);
			} else {
				self.backend.apply_scissor_rect(0, 0, width as i32, height as i32);
			}
			self.backend.apply_bindings(bindings);

			if let Some(ref uniforms) = dc.uniforms {
				for i in 0..uniforms.len() {
					pipeline.uniforms_data[i] = uniforms[i];
				}
			}
			pipeline.set_uniform("Projection", projection);
			pipeline.set_uniform("Model", dc.model);
			pipeline.set_uniform("_Time", time);
			self.backend.apply_uniforms_from_bytes(pipeline.uniforms_data.as_ptr(), pipeline.uniforms_data.len());
			self.backend.draw(0, dc.indices_count as i32, 1);
			self.backend.end_render_pass();

			dc.vertices_count = 0;
			dc.indices_count = 0;
		}

		self.draw_calls_count = 0;
	}

	pub fn get_active_render_pass(&self) -> Option<RenderPass> {
		self.state.render_pass
	}

	pub fn is_depth_test_enabled(&self) -> bool {
		self.state.depth_test_enable
	}

	pub fn render_pass(&mut self, render_pass: Option<RenderPass>) {
		self.state.render_pass = render_pass;
	}

	pub fn depth_test(&mut self, enable: bool) {
		self.state.depth_test_enable = enable;
	}

	/// Set the draw call texture. It's recommended to set this for every draw call, so the geometry won't merge with the previous draw call.
	/// (Check the [`RenderingBackend::geometry`] method documentation)
	pub fn texture(&mut self, texture: Option<&TextureId>) {
		self.state.texture = texture.copied();
		// ! I'm cloning here because from the macroquad code, it converts Texture2D's id to an owned type (thus cloning it anyway)
	}

	pub fn scissor(&mut self, clip: Option<(i32, i32, i32, i32)>) {
		self.state.clip = clip;
	}

	pub fn viewport(&mut self, viewport: Option<(i32, i32, i32, i32)>) {
		self.state.viewport = viewport;
	}

	pub fn get_viewport(&self) -> (i32, i32, i32, i32) {
		let (w, h) = miniquad::window::screen_size();
		self.state.viewport.unwrap_or((0, 0, w as _, h as _))
	}

	pub fn push_model_matrix(&mut self, matrix: glam::Mat4) {
		self.state.model_stack.push(self.state.model() * matrix);
	}

	pub fn pop_model_matrix(&mut self) {
		if self.state.model_stack.len() > 1 {
			self.state.model_stack.pop();
		}
	}

	/// Set the draw call pipeline. This will create a new draw call, if previous pipeline is different to the new one.
	pub fn pipeline(&mut self, pipeline: Option<GlPipeline>) {
		if self.state.pipeline == pipeline {
			return;
		}

		self.state.break_batching = true;
		self.state.pipeline = pipeline;
	}

	/// Set the draw call draw mode (i.e. triangles or lines)
	pub fn draw_mode(&mut self, mode: DrawMode) {
		self.state.draw_mode = mode;
	}

	/// Put verticies and indicies into the draw call geometry. This will **append** geometry to the same draw call
	/// if some of the parameters didn't change:
	/// - Clip region
	/// - Viewport region
	/// - Model matrix
	/// - Pipeline
	/// - Render pass
	/// - Texture
	/// - Draw mode
	///
	/// The new draw call will be allocated, if previous + new geometry exceeds the vertex or indices limit (`10000` and `5000`)
	///
	/// You can manually allocate a new draw call by calling [`RenderingBackend::break_batching`]
	pub fn geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
		if vertices.len() >= self.max_vertices || indices.len() >= self.max_indices {
			#[cfg(feature = "log")]
			bevy_log::warn!("geometry() exceeded max drawcall size, clamping");
		}

		let vertices = &vertices[0..self.max_vertices.min(vertices.len())];
		let indices = &indices[0..self.max_indices.min(indices.len())];

		let pip = self.state.pipeline.unwrap_or(self.pipelines.get_default_by(self.state.draw_mode, self.state.depth_test_enable));

		let previous_dc_ix = if self.draw_calls_count == 0 { None } else { Some(self.draw_calls_count - 1) };
		let previous_dc = previous_dc_ix.and_then(|ix| self.draw_calls.get(ix));

		if previous_dc.map_or(true, |draw_call| {
			draw_call.texture != self.state.texture
				|| draw_call.clip != self.state.clip
				|| draw_call.viewport != self.state.viewport
				|| draw_call.model != self.state.model()
				|| draw_call.pipeline != pip
				|| draw_call.render_pass != self.state.render_pass
				|| draw_call.draw_mode != self.state.draw_mode
				|| draw_call.vertices_count >= self.max_vertices - vertices.len()
				|| draw_call.indices_count >= self.max_indices - indices.len()
				|| self.state.break_batching
		}) {
			let uniforms = self.state.pipeline.map(|pipeline| self.pipelines.get_pipeline_mut(pipeline).uniforms_data.clone());

			if self.draw_calls_count >= self.draw_calls.len() {
				self.draw_calls.push(DrawCall::new(
					self.state.texture,
					self.state.model(),
					self.state.draw_mode,
					pip,
					uniforms.clone(),
					self.state.render_pass,
					self.max_vertices,
					self.max_indices,
				));
			}
			self.draw_calls[self.draw_calls_count].texture = self.state.texture;
			self.draw_calls[self.draw_calls_count].uniforms = uniforms;
			self.draw_calls[self.draw_calls_count].vertices_count = 0;
			self.draw_calls[self.draw_calls_count].indices_count = 0;
			self.draw_calls[self.draw_calls_count].clip = self.state.clip;
			self.draw_calls[self.draw_calls_count].viewport = self.state.viewport;
			self.draw_calls[self.draw_calls_count].model = self.state.model();
			self.draw_calls[self.draw_calls_count].pipeline = pip;
			self.draw_calls[self.draw_calls_count].render_pass = self.state.render_pass;

			self.draw_calls_count += 1;
			self.state.break_batching = false;
		}

		let dc = &mut self.draw_calls[self.draw_calls_count - 1];

		for i in 0..vertices.len() {
			dc.vertices[dc.vertices_count + i] = vertices[i];
		}

		for i in 0..indices.len() {
			dc.indices[dc.indices_count + i] = indices[i] + dc.vertices_count as u16;
		}

		dc.vertices_count += vertices.len();
		dc.indices_count += indices.len();
		dc.texture = self.state.texture;
	}

	/// Deletes the pipeline from the inner pipeline storage.
	///
	/// *Attention: using the same pipeline again will panic, or give unexpected results*
	pub fn delete_pipeline(&mut self, pipeline: GlPipeline) {
		self.pipelines.delete_pipeline(pipeline);
	}

	/// Update the uniform of a loaded pipeline
	pub fn set_uniform<T>(&mut self, pipeline: GlPipeline, name: &str, uniform: T) {
		self.state.break_batching = true;

		self.pipelines.get_pipeline_mut(pipeline).set_uniform(name, uniform);
	}

	/// Prepare material for a draw call. Basically the same as [`RenderingBackend::pipeline`]
	pub fn set_material(&mut self, material: &Material) {
		self.pipeline(Some(material.pipeline));
	}

	/// Update the uniform of an already loaded material. The same as [`RenderingBackend::set_uniform`], but for materials
	pub fn material_set_uniform<T>(&mut self, material: &Material, name: &str, uniform: T) {
		self.set_uniform(material.pipeline, name, uniform);
	}

	/// Update the texture under specific name, in a specific pipeline. Useful in materials
	pub fn set_texture(&mut self, pipeline: GlPipeline, name: &str, texture: TextureId) {
		let pipeline = self.pipelines.get_pipeline_mut(pipeline);
		pipeline
			.textures
			.iter()
			.find(|x| *x == name)
			.unwrap_or_else(|| panic!("can't find texture with name '{}', there is only this names: {:?}", name, pipeline.textures));
		*pipeline.textures_data.entry(name.to_owned()).or_insert(texture) = texture;
	}

	/// Update the texture under specific name, in a specific material. The same as [`RenderingBackend::set_texture`]
	pub fn material_set_texture(&mut self, material: &Material, name: &str, texture: TextureId) {
		self.set_texture(material.pipeline, name, texture);
	}

	/// Update the vertex/index limits of draw calls
	///
	/// *Note: It will resize all existing draw calls as well*
	pub fn update_drawcall_capacity(&mut self, max_vertices: usize, max_indices: usize) {
		self.max_vertices = max_vertices;
		self.max_indices = max_indices;

		for draw_call in &mut self.draw_calls {
			draw_call.vertices = vec![Vertex::new(vec3(0.0, 0.0, 0.0), vec2(0.0, 0.0), Rgba::default()); max_vertices];
			draw_call.indices = vec![0; max_indices];
		}
		for binding in &mut self.draw_call_bindings {
			let vertex_buffer = self.backend.new_buffer(BufferType::VertexBuffer, BufferUsage::Stream, BufferSource::empty::<Vertex>(self.max_vertices));
			let index_buffer = self.backend.new_buffer(BufferType::IndexBuffer, BufferUsage::Stream, BufferSource::empty::<u16>(self.max_indices));
			*binding = Bindings {
				vertex_buffers: vec![vertex_buffer],
				index_buffer,
				images: vec![self.white_texture],
			};
		}
	}
}

/// Sets the Clear Color of the window
#[repr(transparent)]
#[derive(Resource, Default)]
pub struct ClearColor(pub rgba::Rgba);

/// Plugin responsible for initializing the [`RenderBackend`](MqdRenderingBackend)
pub struct RenderBackendPlugin {
	/// Controls whether to turn on Quadify's default rendering like Mesh, Materials etc.
	///
	/// Some games would like some freedom, so you're free to use the NonSend [`RenderingBackend`] instead.
	pub default_pipeline: bool,
}

impl Default for RenderBackendPlugin {
	fn default() -> Self {
		Self { default_pipeline: true }
	}
}

impl bevy_app::Plugin for RenderBackendPlugin {
	fn build(&self, app: &mut bevy_app::App) {
		if self.default_pipeline {
			// Setup default camera
			let camera = camera::Camera2D::default();
			let id = app.world_mut().spawn((camera, camera::RenderTarget::Window)).id();
			// Setup the rendering backend
			app.insert_resource(camera::CurrentCameraTag(id))
				.init_resource::<ClearColor>()
				.add_systems(state::MiniquadPrepareDraw, apply_clear_color)
				.add_systems(state::MiniquadEndDraw, commit_frame);
		}
	}
}

fn apply_clear_color(
	mut render_ctx: NonSendMut<RenderingBackend>,
	clear_color: Res<ClearColor>,
	current_camera: Res<camera::CurrentCameraTag>,
	render_target: Query<&camera::RenderTarget>,
	// renderables: Query<&Handle<Mesh>>
) {
	let color = clear_color.as_ref().0.to_float();
	let clear = PassAction::clear_color(color.x, color.y, color.z, color.w);
	let entity = current_camera.as_ref().0;

	match render_target.get(entity) {
		Ok(rt) => match rt {
			camera::RenderTarget::Window => render_ctx.begin_default_pass(clear),
			camera::RenderTarget::Texture { render_pass, .. } => render_ctx.begin_pass(Some(*render_pass), clear),
		},
		Err(_e) => {
			#[cfg(feature = "log")]
			bevy_log::error!("Failed to get render target: {:?} on current Camera: {:?}", _e, entity);
			return;
		}
	};

	// End the render pass
	// TODO: Fill the render pass with some basic materials
	render_ctx.end_render_pass();
}

/// Commit the rendered frame
fn commit_frame(mut render_ctx: NonSendMut<RenderingBackend>) {
	render_ctx.commit_frame();
}
