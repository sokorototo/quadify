/// I'm not sure how rendering should be implemented exactly, but could be that it's an overall final operation on draw primitives.
/// Say, there's a texture to draw, or a rectangle. It could also be interpeted as a draw operation, that can be extracted and
/// rendered at the end of the frame. That's for now how I see it, but it could change in the future.

use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::app::MainScheduleOrder;
use macroquad::color;
use macroquad::window::clear_background; // ? Re-exporting it, since it's way simpler than the one bevy uses.

mod camera;
mod texture;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPrepare;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderSprites;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderGizmos;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderUI;
// ? Actually perform drawing operations and wait for the next frame

#[derive(Resource)]
pub struct ClearColor(pub color::Color);
impl Default for ClearColor {
    fn default() -> Self {
        Self(color::BLACK)
    }
}

pub struct QuadRenderPlugin;
impl Plugin for QuadRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_schedule(RenderPrepare)
            .init_schedule(RenderSprites)
            .init_schedule(RenderGizmos)
            .init_schedule(RenderUI)
            .init_resource::<ClearColor>();
        {
            let mut sched_order = app.world.resource_mut::<MainScheduleOrder>();
            sched_order.insert_after(Last, RenderPrepare);
            sched_order.insert_after(RenderPrepare, RenderSprites);
            sched_order.insert_after(RenderSprites, RenderGizmos);
            sched_order.insert_after(RenderGizmos, RenderUI);
        }

        app.add_systems(RenderPrepare, draw_prepare);
    }
}

/// A global, exclusive draw system that draws everything (textures, gizmos, ui).
/// Custom shaders can be made in the future.
fn draw_prepare(w: &mut World) {
    let col = if let Some(col) = w.get_resource::<ClearColor>() { col.0 } else { color::BLACK };
    clear_background(col);
}