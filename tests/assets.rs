//! This test tests for main functionality.
//! The app should start, wait 2 seconds and then gracefully quit.
//! The screen should be blue.

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::utils::hashbrown::hash_map::DefaultHashBuilder;
use macroquad::rand::rand;
use quadify::asset::{Assets, Handle, QuadAssetPlugin, TextureKey};
use quadify::sprite::{SpriteBundle, SpriteTexture};
use quadify::{prelude::*, render};
use quadify::render::ClearColor;

#[derive(Resource)]
struct ExitTimer(Timer);

#[test]
fn main() {
    App::new()
        .add_plugins(QuadifyPlugins.set(QuadAssetPlugin {
            asset_path: "assets",
            ..Default::default()
        }))
        .insert_resource(ExitTimer(Timer::from_seconds(5.0, TimerMode::Once)))
        .insert_resource(ClearColor(quadify::color::BLACK))
        .add_systems(Startup, load_assets)
        .add_systems(Update, run_timer)
        .run();
}

fn load_assets(
    mut commands: Commands,
    mut asset: Assets,
) {
    let handle = asset.load_texture_fs("texture.png");

    commands.spawn(
        SpriteBundle{
            texture: SpriteTexture(handle.clone()),
            transform: TransformBundle::from_transform(Transform {
                translation: Vec3::new(640.0, 360.0, 0.0),
                rotation: Quat::from_rotation_x(127.0),
                scale: Vec3::new(5.0, 5.0, 1.0)
            }),
            ..Default::default()
        },
    );
}

fn run_timer(time: Res<Time>, mut timer: ResMut<ExitTimer>, mut exit_events: EventWriter<AppExit>) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        exit_events.send(AppExit);
    }
}
