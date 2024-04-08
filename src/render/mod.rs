/// I'm not sure how rendering should be implemented exactly, but could be that it's an overall final operation on draw primitives.
/// Say, there's a texture to draw, or a rectangle. It could also be interpeted as a draw operation, that can be extracted and
/// rendered at the end of the frame. That's for now how I see it, but it could change in the future.

use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::app::MainScheduleOrder;
pub use macroquad::color;
use macroquad::window::clear_background; // ? Re-exporting it, since it's way simpler than the one bevy uses.

mod camera;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Render;
// ? Actually perform drawing operations and wait for the next frame

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Extract;

#[derive(Resource)]
pub struct ClearColor(pub color::Color);
impl Default for ClearColor {
    fn default() -> Self {
        Self(color::BLACK)
    }
}

pub struct MQRenderPlugin;
impl Plugin for MQRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_schedule(Extract)
            .init_schedule(Render)
            .init_resource::<ClearColor>();
        {
            let mut sched_order = app.world.resource_mut::<MainScheduleOrder>();
            sched_order.insert_after(Last, Extract);
            sched_order.insert_after(Extract, Render);
        }

        app.add_systems(Render, draw);
    }
}

fn draw(w: &mut World) {
    // ! I'm probably going to replace this with a global render system. 
    // ! Clearing and drawing in separate threads is a bit useless.
    let col = if let Some(col) = w.get_resource::<ClearColor>() { col.0 } else { color::BLACK };
    clear_background(col);

    // * More operations later
}