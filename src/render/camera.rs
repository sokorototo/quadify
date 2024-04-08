use bevy::prelude::*;
use macroquad::camera::Camera2D as MQCamera2D;
use macroquad::texture::RenderTarget as MQRenderTarget;

#[derive(Component)]
pub struct Camera2D(MQCamera2D);

impl Camera2D {
    pub fn new() -> Self {
        Self(MQCamera2D::default())
    }
}


#[derive(Component)]
pub struct CameraOffset(Vec2);

#[derive(Component)]
pub struct CameraZoom(Vec2);

#[derive(Component)]
pub struct RenderTarget(MQRenderTarget);

// * I think the most important component here, allowing for post-processing and all sorts of cool effects.
// * I guess it could be combined with material, thus giving the way to write post-processing shaders.

