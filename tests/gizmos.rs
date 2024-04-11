/// This test tests for main functionality.
/// The app should start, wait 2 seconds and then gracefully quit.
/// The screen should be blue.
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::PrimaryWindow;
use quadify::gizmos::Gizmos;
use quadify::{prelude::*, render};
use quadify::render::ClearColor;

#[derive(Resource)]
struct ExitTimer(Timer);

#[test]
fn main() {
    App::new()
        .add_plugins(QuadifyPlugins)
        .insert_resource(ExitTimer(Timer::from_seconds(10.0, TimerMode::Once)))
        .insert_resource(ClearColor(quadify::color::BLUE))
        .add_systems(Update, (run_timer, draw_gizmos))
        .run();
}

fn run_timer(time: Res<Time>, mut timer: ResMut<ExitTimer>, mut exit_events: EventWriter<AppExit>) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        exit_events.send(AppExit);
    }
}

fn draw_gizmos(
    qwin: Query<&Window, With<PrimaryWindow>>,
    mut gizmos: Gizmos
) {
    if let Some(pos) = qwin.single().cursor_position() {
        gizmos.circle(pos, 45.0, quadify::color::RED);
        gizmos.rect(pos, Vec2::new(45.0, 45.0), quadify::color::YELLOW);
    }
}