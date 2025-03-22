use bevy_app::{prelude::*, AppExit};
use bevy_ecs::prelude::*;
use quadify::prelude::*;

const INPUT_NEEDED: u32 = 3;

#[derive(Resource)]
struct InputReceived(u32);

#[test]
fn main() {
	App::new()
		.add_plugins(QuadifyPlugins.set(WindowPlugin {
			title: "Read Mouse Events Test".to_string(),
			width: 600,
			height: 600,
			high_dpi: false,
			resizeable: true,
			..Default::default()
		}))
		.insert_resource(InputReceived(0))
		.add_systems(Startup, || {
			println!("Hi, this is an interactive mouse input test. Please move, click and scroll your mouse!");
		})
		.add_systems(Update, (mouse_btn_events, mouse_motion_events, mouse_scroll_events, close_when_received_all))
		.run();
}

fn close_when_received_all(received: Res<InputReceived>, mut quit_events: EventWriter<AppExit>) {
	if received.0 == INPUT_NEEDED {
		quit_events.send(AppExit::Success);
		println!("Received all events!");
	}
}

fn mouse_btn_events(btn_events: EventReader<MouseButtonEvent>, mut received: ResMut<InputReceived>, mut is_done: Local<bool>) {
	if !*is_done && !btn_events.is_empty() {
		*is_done = true;
		received.0 += 1;
		println!("{}: Mouse button event received!", received.0);
	}
}

fn mouse_motion_events(motion_events: EventReader<MouseMotionEvent>, mut received: ResMut<InputReceived>, mut is_done: Local<bool>) {
	if !*is_done && !motion_events.is_empty() {
		*is_done = true;
		received.0 += 1;
		println!("{}: Mouse motion event received!", received.0);
	}
}

fn mouse_scroll_events(scroll_events: EventReader<MouseWheelEvent>, mut received: ResMut<InputReceived>, mut is_done: Local<bool>) {
	if !*is_done && !scroll_events.is_empty() {
		*is_done = true;
		received.0 += 1;
		println!("{}: Mouse wheel event received!", received.0);
	}
}
