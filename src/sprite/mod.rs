/// This plugin is an abstraction over macroquad's textures
/// Don't be scared, because it's my first prototype. In the future I'm planning to change it.

use bevy::prelude::*;
use macroquad::math;
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
// use macroquad::prelude::*;
use crate::{asset::{AssetStatus, AssetStorage, Handle, TextureKey}, color::*, render::RenderSprites};

#[derive(Component)]
pub struct SpriteTexture(pub Handle<TextureKey>);

/// Sprite color. Use [`WHITE`] if you don't want any changes.
#[derive(Component)]
pub struct SpriteColor(Color);

/// Sprite flip parameter. To flip by `x` axis, by `y` or by both?
#[derive(Component)]
pub struct SpriteFlip {
    x: bool,
    y: bool
}

#[derive(Component)]
pub struct SpriteSize(Vec2);

/// Source rectangular area from the original texture
#[derive(Component)]
pub struct SpriteSrc {
    x: f32, y: f32,
    w: f32, h: f32
}

/// Around what point to rotate this texture? (Screen space)
#[derive(Component)]
pub struct SpritePivot(Vec2);

/// An optional offset from target
#[derive(Component)]
pub struct SpriteOffset(Vec2);

/// Simple sprite bundle, similar to bevy's `Sprite`
#[derive(Bundle)]
pub struct SpriteBundle {
    pub transform: TransformBundle,
    pub texture: SpriteTexture,
    pub color: SpriteColor,
    pub flip: SpriteFlip
    // * Additional components can be added manually
}

impl Default for SpriteBundle {
    fn default() -> Self {
        Self {
            transform: TransformBundle::default(),
            texture: SpriteTexture(Handle::null()),
            color: SpriteColor(WHITE),
            flip: SpriteFlip { x: false, y: false }
        }
    }
}

pub struct QuadSpritePlugin;
impl Plugin for QuadSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RenderSprites, batch_draw);
    }
}

type BatchComponents<'a> = (
    &'a GlobalTransform, &'a SpriteTexture, &'a SpriteColor, &'a SpriteFlip,
    Option<&'a SpriteSize>, Option<&'a SpriteSrc>, Option<&'a SpritePivot>, Option<&'a SpriteOffset>
);

/// Just a massive draw call on all sprites. 
/// 
/// Honestly, should be replaced with its own sprite batching and pipeline.
fn batch_draw(
    world: &mut World,
    // render_targets: Query<BatchComponents>,
) {
    // Checking if the sprite has the loaded 
    let mut sprites: Vec<BatchComponents> = world.query::<BatchComponents>()
        .iter(world)
        .collect();
    sprites.sort_by(|a, b| 
        a.0.translation().z.partial_cmp(&b.0.translation().z
    ).unwrap());
    let assets = world.get_resource::<AssetStorage>().unwrap();

    for (trns, texture, color, flip, size, src, pivot, offset) in sprites.iter() {
        let asset = assets.textures.get(texture.0.key);
        if let Some(AssetStatus::Done(txtr)) = asset {
            let offset = if let Some(o) = offset { o.0 } else { Vec2::ZERO };
            let trns = trns.compute_transform();

            let dest_size = if let Some(s) = size { 
                math::Vec2 {
                    x: s.0.x*trns.scale.x, 
                    y: s.0.y*trns.scale.y
                }
            } else { 
                math::Vec2 {
                    x: txtr.width()*trns.scale.x, 
                    y: txtr.height()*trns.scale.y
                }
            };

            let source = if let Some(r) = src { Some(math::Rect::new(r.x, r.y, r.w, r.h)) } else { None };
            let pivot =  if let Some(p) = pivot { Some(math::Vec2::new(p.0.x, p.0.y)) } else { None };

            draw_texture_ex(
                &txtr,
                trns.translation.x+offset.x,
                trns.translation.y+offset.y,
                color.0, 
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source,
                    rotation: trns.rotation.x,
                    flip_x: flip.x,
                    flip_y: flip.y,
                    pivot
                }
            )
        }
    }
}