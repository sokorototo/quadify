use std::f32::consts::PI;

use glam::{vec2, vec3, Quat, Vec2, Vec3};
use miniquad::{VertexAttribute, VertexFormat};

use bevy_asset::Asset;
use bevy_reflect::Reflect;

use super::rgba::Rgba;

#[repr(C)]
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Vertex {
	pub position: Vec3,
	pub uv: Vec2,
	pub color: Rgba,
}

impl Vertex {
	pub fn new(position: Vec3, uv: Vec2, color: Rgba) -> Self {
		Self { position, uv, color }
	}

	/// Default's vertex attributes constant
	pub const fn attributes() -> [VertexAttribute; 3] {
		[
			VertexAttribute::new("position", VertexFormat::Float3),
			VertexAttribute::new("texcoord", VertexFormat::Float2),
			VertexAttribute::new("color0", VertexFormat::Byte4),
		]
	}
}

#[derive(Clone, PartialEq)]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
}

impl Mesh {
	/// Makes a simple quad mesh
	fn quad(pos: Vec3, size: Vec2, color: Rgba) -> Self {
		let indices = vec![0, 1, 2, 1, 2, 3];
		let (hw, hh) = (size.x / 2.0, size.y / 2.0);
		let vertices = vec![
			Vertex::new(vec3(pos.x - hw, pos.y + hh, pos.z), vec2(0.0, 0.0), color), // top-left
			Vertex::new(vec3(pos.x + hw, pos.y + hh, pos.z), vec2(1.0, 0.0), color), // top-right
			Vertex::new(vec3(pos.x - hw, pos.y - hh, pos.z), vec2(0.0, 1.0), color), // bottom-left
			Vertex::new(vec3(pos.x + hw, pos.y - hh, pos.z), vec2(1.0, 1.0), color), // bottom-right
		];
		Self { vertices, indices }
	}

	/// Makes a circle mesh, with a specified amount of points
	fn circle(pos: Vec3, r: f32, npoints: u32, color: Rgba) -> Self {
		debug_assert!(npoints >= 3, "Not enough points to represent a circle mesh. Minimum is 3");
		let mut indices: Vec<u16> = vec![];
		let mut vertices: Vec<Vertex> = vec![];

		let circle_piece = 2.0 * PI / (npoints as f32);
		for i in 0..npoints {
			let degrees = (i as f32) * circle_piece;
			let (x, y) = (degrees.cos(), degrees.sin());
			vertices.push(Vertex::new(vec3(pos.x + x * r, pos.y + y * r, pos.z), vec2(x, y), color));

			if i < npoints - 2 {
				let i = i as u16;
				indices.append(&mut vec![0, i + 1, i + 2]);
			}
		}
		Self { vertices, indices }
	}

	/// Colors the verticies of the mesh by the given [`Rgba`].
	pub fn colored_as(mut self, color: Rgba) -> Self {
		self.color_as(color);
		self
	}

	/// Colors the verticies of the mesh in place by the given [`Rgba`].
	pub fn color_as(&mut self, color: Rgba) {
		for vert in self.vertices.iter_mut() {
			vert.color = color;
		}
	}

	/// Translates the vertex positions of the mesh by the given [`Vec3`].
	pub fn translated_by(mut self, translation: Vec3) -> Self {
		self.translate_by(translation);
		self
	}

	/// Translates the vertex positions of the mesh in place by the given [`Vec3`].
	pub fn translate_by(&mut self, translation: Vec3) {
		if translation == Vec3::ZERO {
			return;
		}

		for vert in self.vertices.iter_mut() {
			vert.position += translation;
		}
	}

	/// Rotates the vertex positions of the mesh by the given [`Quat`].
	pub fn rotated_by(mut self, rotation: Quat) -> Self {
		self.rotate_by(rotation);
		self
	}

	/// Rotates the vertex positions of the mesh in place by the given [`Quat`].
	pub fn rotate_by(&mut self, rotation: Quat) {
		for vert in self.vertices.iter_mut() {
			vert.position = rotation * vert.position;
		}
	}

	/// Scales the vertex positions, normals, and tangents of the mesh by the given [`Vec3`].
	pub fn scaled_by(mut self, scale: Vec3) -> Self {
		self.scale_by(scale);
		self
	}

	/// Scales the vertex positions, normals, and tangents of the mesh in place by the given [`Vec3`].
	pub fn scale_by(&mut self, scale: Vec3) {
		for vert in self.vertices.iter_mut() {
			vert.position = scale * vert.position;
		}
	}
}

/// A private struct that only stores meshes size.
enum MeshShape {
	Quad(Vec2),
	Circle(f32),
}

/// A Mesh constructor for generating/loading meshes. Meshes in `quadify` also contain color information
pub struct MeshBuilder {
	shape: Option<MeshShape>,
	color: Option<Rgba>,
	position: Option<Vec3>,
	circle_points: u32,
}

impl Default for MeshBuilder {
	fn default() -> Self {
		Self {
			shape: None,
			color: None,
			position: None,
			circle_points: 20,
		}
	}
}

impl MeshBuilder {
	/// Generates a quad mesh, with a specified size
	pub fn as_quad(&mut self, size: Vec2) -> &mut Self {
		self.shape = Some(MeshShape::Quad(size));
		self
	}

	/// Generates a circle mesh, with a specified radius
	///
	/// *Note: there's also [`circle_points`](MeshBuilder::circle_points) that controls the amount of points your circle has
	/// (more points look better)*
	pub fn as_circle(&mut self, radius: f32) -> &mut Self {
		self.shape = Some(MeshShape::Circle(radius));
		self
	}

	/// Sets circle's point amount. The default value is `20`, but you can increase/reduce this number to a desired result.
	pub fn circle_points(&mut self, n_points: u32) -> &mut Self {
		debug_assert!(n_points >= 3, "Not enough points to represent a circle mesh. Minimum is 3");
		self.circle_points = n_points;
		self
	}

	/// Sets the center position of the mesh (for both circles and quads). This method cannot be ignored.
	pub fn at_position(&mut self, position: Vec3) -> &mut Self {
		self.position = Some(position);
		self
	}

	/// Sets the color of the mesh. If not set up - will use the default black color.
	pub fn with_color(&mut self, color: Rgba) -> &mut Self {
		self.color = Some(color);
		self
	}

	/// Constructs and returns the desired mesh back.
	///
	/// *Note: panics if the shape wasn't provided*
	pub fn build(&mut self) -> Mesh {
		debug_assert!(self.shape.is_some(), "Can't build a Mesh without shape parameter provided.");
		debug_assert!(self.position.is_some(), "Can't build a Mesh without position parameter provided.");

		let color = self.color.unwrap_or_default();
		let pos = self.position.unwrap();
		match self.shape.take().unwrap() {
			MeshShape::Quad(size) => Mesh::quad(pos, size, color),
			MeshShape::Circle(r) => Mesh::circle(pos, r, self.circle_points, color),
		}
	}
}
