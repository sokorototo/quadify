use bevy_app::*;
use bevy_ecs::{event::EventReader, system::ResMut};
use quadify::prelude::*;

#[test]
fn main() {
	App::new()
		.add_plugins(QuadifyPlugins.set(WindowPlugin {
			title: "Read Keyboard Events Test".to_string(),
			width: 600,
			height: 600,
			high_dpi: false,
			resizeable: true,
			..Default::default()
		}))
		.add_systems(Startup, || println!("TIP: press ESC to quit the test!"))
		.add_systems(Update, (keycode_events, exit_on_esc, char_events))
		.run();
}

fn keycode_events(mut events: EventReader<KeyCodeEvent>, mut window: ResMut<WindowProperties>, mut cursor: ResMut<CursorProperties>) {
	for event in events.read().filter(|ev| !ev.released) {
		match event.keycode {
			miniquad::KeyCode::F => {
				window.fullscreen = !window.fullscreen;

				if !window.fullscreen {
					window.width = 600;
					window.height = 600;
				}
			}
			miniquad::KeyCode::R => cursor.grabbed = !cursor.grabbed,
			_ => {}
		}
	}
}

fn char_events(mut events: EventReader<KeyCharEvent>, mut window: ResMut<WindowProperties>) {
	for event in events.read() {
		if let Some(x) = event.character.to_digit(10) {
			window.position = Some(glam::u32::UVec2::new(x * 100, 80));
		}
	}
}
