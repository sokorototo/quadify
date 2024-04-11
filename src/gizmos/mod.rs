use bevy::{ecs::system::SystemParam, prelude::*};
use macroquad::{camera::set_default_camera, 
    shapes::{
        draw_circle_lines, draw_ellipse_lines, draw_line, draw_rectangle_lines
    }
};
use crate::{color, render::RenderGizmos};

/// # Gizmos
/// This is a lightweight version of bevy gizmos.
/// It uses really basic 2D shapes instead.
/// *(Note that gizmos is rendered after sprites, but before UI)*
/// 
/// 
/// I'll probably redo-it with custom pipeline in the future, because it currently lacks actual polygons 
/// and ability to rotate rectangles

#[derive(Resource, Clone, Copy)]
pub struct GizmosConfig {
    pub enabled: bool,
    pub line_width: f32,
}

impl Default for GizmosConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            line_width: 2.0,
        }
    }
}

enum GizmosCommand {
    Line {
        start: Vec2,
        end: Vec2,
        color: color::Color
    },
    Circle {
        pos: Vec2, 
        r: f32, 
        color: color::Color
    },
    Ellipse {
        pos: Vec2,
        size: Vec2,
        angle: f32,
        color: color::Color
    },
    Rect {
        pos: Vec2,
        size: Vec2,
        color: color::Color
    },
}

#[derive(Resource)]
struct GizmosCommands(Vec<GizmosCommand>);

impl Default for GizmosCommands {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(SystemParam)]
pub struct Gizmos<'w> {
    commands: ResMut<'w, GizmosCommands>,
}

impl<'w> Gizmos<'w> {
    pub fn line(&mut self, start: Vec2, end: Vec2, color: color::Color) {
        self.commands.0.push(GizmosCommand::Line { start, end, color });
    }

    pub fn circle(&mut self, pos: Vec2, r: f32, color: color::Color) {
        self.commands.0.push(GizmosCommand::Circle { pos, r, color });
    }

    pub fn ellipse(&mut self, pos: Vec2, angle: f32, half_size: Vec2, color: color::Color) {
        self.commands.0.push(GizmosCommand::Ellipse { pos, size: half_size, angle, color });
    }

    pub fn rect(&mut self, pos: Vec2, size: Vec2, color: color::Color) {
        self.commands.0.push(GizmosCommand::Rect { pos, size, color });
    }
}

pub struct MQGizmosPlugin;
impl Plugin for MQGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GizmosCommands>()
            .init_resource::<GizmosConfig>()
            .add_systems(RenderGizmos, draw_gizmos)
        ;
    }
}

/// Draws all gizmos in the separate [`RenderGizmos`] schedule label (after sprites)
/// It checks for GizmosConfig, and if enabled in GizmosConfig - draws all shapes.
/// In the future I want to use a different pipeline for rotations, gradients and shapes like polygons
fn draw_gizmos(world: &mut World) {
    let (enabled, thickness) = {
        match world.get_resource::<GizmosConfig>() {
            Some(conf) =>  (conf.enabled, conf.line_width),
            None => (true, 2.0) // In case someone removes the resource
        }
    };
    if enabled {
        let mut commands = world.get_resource_mut::<GizmosCommands>().unwrap();
    
        set_default_camera();
        for command in commands.0.drain(..) {
            match command {
                GizmosCommand::Line { start, end, color } => {
                    draw_line(start.x, start.y, end.x, end.y, thickness, color);
                },
                GizmosCommand::Circle { pos, r, color } => {
                    draw_circle_lines(pos.x, pos.y, r, thickness, color);
                },
                GizmosCommand::Ellipse { pos, size, angle, color } => {
                    draw_ellipse_lines(pos.x, pos.y, size.x, size.y, angle, thickness, color);
                },
                GizmosCommand::Rect { pos, size, color } => {
                    draw_rectangle_lines(pos.x, pos.y, size.x, size.y, thickness, color);
                }
            };
        }
    }
}