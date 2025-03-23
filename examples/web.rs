use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use miniquad::*;
use quadify::prelude::*;

fn main() {
	App::new()
		.add_plugins(QuadifyPlugins.set(WindowPlugin {
			title: "Comprehensive Web Platform Test".to_string(),
			width: 600,
			height: 600,
			high_dpi: true,
			resizeable: false,
			..Default::default()
		}))
		.add_systems(Startup, || {
			#[cfg(target_arch = "wasm32")]
			wasm_bindgen_futures::spawn_local(async move {
				let string = load_string("index.html").await;
				bevy_log::info!("Loaded index.html: {:?}", string.map(|_| ()));
			});
		})
		.add_systems(Update, (read_keyboard, exit_on_esc, file_drop_events, mouse_events))
		.run();
}

fn read_keyboard(mut keyboard_events: EventReader<KeyCodeEvent>, mut window_properties: ResMut<WindowProperties>) {
	for event in keyboard_events.read() {
		if event.released {
			let width = match event.keycode {
				KeyCode::Key0 => Some(0),
				KeyCode::Key1 => Some(1),
				KeyCode::Key2 => Some(2),
				KeyCode::Key3 => Some(3),
				KeyCode::Key4 => Some(4),
				KeyCode::Key5 => Some(5),
				KeyCode::Key6 => Some(6),
				KeyCode::Key7 => Some(7),
				KeyCode::Key8 => Some(8),
				KeyCode::Key9 => Some(9),
				_ => None,
			};

			width.map(|w| window_properties.width = (w + 1) * 100);
		}
	}
}

fn file_drop_events(mut events: EventReader<DroppedFileEvent>) {
	for event in events.read() {
		let string = event.bytes.as_ref().map(|d| String::from_utf8_lossy(d));
		bevy_log::info!("File {:?} Dropped into Application: {:?}", event.path, string);
	}
}

fn mouse_events(mut idx: Local<usize>, mut events: EventReader<MouseButtonEvent>, mut clear_colour: ResMut<ClearColor>, window: Res<WindowProperties>, mut cursor: ResMut<CursorProperties>) {
	static CURSORS: [CursorIcon; 8] = [
		CursorIcon::Default,
		CursorIcon::Crosshair,
		CursorIcon::Text,
		CursorIcon::Move,
		CursorIcon::NotAllowed,
		CursorIcon::Pointer,
		CursorIcon::Wait,
		CursorIcon::Help,
	];

	for event in events.read() {
		if !event.released {
			continue;
		}

		match event.button {
			MouseButton::Right => {
				let glam::Vec2 { x, y } = cursor.position;

				let r = (x / window.height as f32) * 255.0;
				let g = (y / window.width as f32) * 255.0;

				clear_colour.0 = rgba::rgba(r as u8, g as u8, 128, 255);
			}
			MouseButton::Left => {
				cursor.grabbed = !cursor.grabbed;
			}
			MouseButton::Middle => {
				*idx = (*idx + 1) % CURSORS.len();
				cursor.icon = CURSORS[*idx % CURSORS.len()];
			}
			_ => {}
		}
	}
}
