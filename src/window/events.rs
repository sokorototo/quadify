use std::path::PathBuf;

use bevy_app::AppExit;
use bevy_ecs::{
	change_detection::DetectChanges,
	event::{Event, EventReader, EventWriter},
	system::{Local, Res, Resource},
};

#[derive(Debug, Clone, Event)]
pub enum WindowEvent {
	/// The window was minimized, `blur` event on Web
	Minimized,
	/// The window was restored, `focus` event on Web
	Restored,
	/// The window was resized, uses `ResizeObserver` on Web
	Resized {
		/// New width of the window
		width: f32,
		/// New height of the window
		height: f32,
	},
}

#[derive(Debug, Clone, Resource)]
pub struct WindowProperties {
	pub position: Option<glam::u32::UVec2>,
	pub width: u32,
	pub height: u32,
	pub fullscreen: bool,
}

pub(crate) fn apply_window_properties(mut previous: Local<Option<WindowProperties>>, properties: Res<WindowProperties>) {
	// skip first run
	if properties.is_changed() && !properties.is_added() {
		if let Some(p) = properties.position {
			miniquad::window::set_window_position(p.x, p.y);
		}

		if previous.as_ref().map_or(true, |p| p.fullscreen != properties.fullscreen) {
			miniquad::window::set_fullscreen(properties.fullscreen);
		}

		if previous.as_ref().map_or(true, |p| p.width != properties.width || p.height != properties.height) {
			miniquad::window::set_window_size(properties.width, properties.height);
		}
	}

	*previous = Some(properties.clone());
}

/// Exits the application when the escape key is pressed
pub fn exit_on_esc(mut keyboard_input: EventReader<KeyCodeEvent>, mut app_exit: EventWriter<AppExit>) {
	for event in keyboard_input.read() {
		if !event.released && event.keycode == miniquad::KeyCode::Escape {
			app_exit.send(AppExit::Success);
		}
	}
}

/// Closes the application on an [`AppExit`] event
pub fn quit_on_app_exit(app_exit: EventReader<AppExit>) {
	if !app_exit.is_empty() {
		miniquad::window::quit();
	}
}

#[derive(Debug, Clone, Resource)]
pub struct CursorProperties {
	pub position: glam::Vec2,
	pub grabbed: bool,
	pub icon: miniquad::CursorIcon,
}

pub(crate) fn apply_cursor_properties(mut previous: Local<Option<CursorProperties>>, properties: Res<CursorProperties>) {
	if properties.is_changed() && !properties.is_added() {
		if previous.as_ref().map_or(true, |p| p.icon != properties.icon) {
			miniquad::window::set_mouse_cursor(properties.icon);
		}
		if previous.as_ref().map_or(true, |p| p.grabbed != properties.grabbed) {
			miniquad::window::set_cursor_grab(properties.grabbed);
		}

		// save previous state
		*previous = Some(properties.clone());
	}
}

#[derive(Debug, Event)]
pub struct MouseMotionEvent {
	pub delta: glam::Vec2,
}

#[derive(Debug, Event)]
pub struct MouseWheelEvent {
	pub delta: glam::Vec2,
}

#[derive(Debug, Event)]
pub struct MouseButtonEvent {
	pub position: glam::Vec2,
	pub button: miniquad::MouseButton,
	pub released: bool,
}

impl std::ops::Deref for MouseButtonEvent {
	type Target = miniquad::MouseButton;

	fn deref(&self) -> &Self::Target {
		&self.button
	}
}

#[derive(Debug, Event)]
pub struct TouchEvent {
	pub phase: miniquad::TouchPhase,
	pub id: u64,
	pub position: glam::Vec2,
}

impl std::ops::Deref for TouchEvent {
	type Target = miniquad::TouchPhase;

	fn deref(&self) -> &Self::Target {
		&self.phase
	}
}

#[derive(Debug, Event)]
pub struct KeyCodeEvent {
	pub keycode: miniquad::KeyCode,
	pub mods: miniquad::KeyMods,
	pub repeat: bool,
	pub released: bool,
}

impl std::ops::Deref for KeyCodeEvent {
	type Target = miniquad::KeyCode;

	fn deref(&self) -> &Self::Target {
		&self.keycode
	}
}

#[derive(Debug, Event)]
pub struct KeyCharEvent {
	pub character: char,
	pub mods: miniquad::KeyMods,
	pub repeat: bool,
}

impl std::ops::Deref for KeyCharEvent {
	type Target = char;

	fn deref(&self) -> &Self::Target {
		&self.character
	}
}

/// A file dropped into the application window. Bytes is `Some(---)` on the Web, None on Desktop
#[derive(Debug, Clone, Event)]
pub struct DroppedFileEvent {
	pub path: PathBuf,
	pub bytes: Option<Vec<u8>>,
}
