use bevy_app::*;
use bevy_ecs::{
	change_detection::{DetectChanges, DetectChangesMut},
	schedule::ScheduleLabel,
	system::Resource,
};
use glam::vec2;

use super::events;
use crate::render::RenderingBackend;

/// General `miniquad` state handler for the entire app. It stores bevy's [`App`], manages its event loop and so on
pub(crate) struct QuadifyState {
	app: App,
	sender: Option<oneshot::Sender<u8>>,
}

impl QuadifyState {
	/// Creates a new `QuadifyState` object
	pub(crate) fn new(mut app: App, sender: oneshot::Sender<u8>) -> Self {
		app.insert_non_send_resource(RenderingBackend::new());
		Self { app, sender: Some(sender) }
	}
}

/// Systems add to the [`MiniquadDraw`] schedule will be called from within the [`EventHandler::draw`] method
///
/// On Android and Web, this schedule will be called conditionally. If the App is currently in focus.
/// Systems on this schedule are expected to be using [`RenderingBackend`] non-send resources, thus are run on the main thread. Without any form of multithreading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct MiniquadDraw;

/// Almost the same as [`MiniquadDraw`], but is for general preparation like screen and depth clearing, runs before [`MiniquadDraw`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub(crate) struct MiniquadPrepareDraw;

/// Almost the same as [`MiniquadDraw`], but is only used to commit the framebuffer to the screen. Don't use it
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub(crate) struct MiniquadEndDraw;

/// Runs in the Event Listeners Context on the Web, use this schedule to call `requestFullScreen` and other protected Web APIs.
/// Runs very early in the schedule, so Resources and Events will be from the previous frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct MiniquadPrivilegedSchedule;

/// Run when the user requests to quit the application, use this to set [`AcceptQuitRequest`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct MiniquadQuitRequestedSchedule;

/// The user is requesting to quit the application. `accept` can cancel the request, and status is the exit code
#[derive(Debug, Resource)]
pub struct QuitRequested {
	pub accept: bool,
	pub status: u8,
}

impl miniquad::EventHandler for QuadifyState {
	// Called every frame
	fn update(&mut self) {
		self.app.update();
	}

	// Called on every frame if App has an active surface
	fn draw(&mut self) {
		self.app.world_mut().run_schedule(MiniquadPrepareDraw);
		self.app.world_mut().run_schedule(MiniquadDraw);
	}

	// WM Events
	fn window_minimized_event(&mut self) {
		self.app.world_mut().send_event(events::WindowEvent::Minimized);
	}

	fn window_restored_event(&mut self) {
		self.app.world_mut().send_event(events::WindowEvent::Restored);
	}

	fn resize_event(&mut self, width: f32, height: f32) {
		if let Some(mut props) = self.app.world_mut().get_resource_mut::<events::WindowProperties>() {
			// to avoid infinite looping once WindowProperties is applied to the miniquad::window
			let props = props.bypass_change_detection();

			props.width = width as u32;
			props.height = height as u32;
		}

		self.app.world_mut().send_event(events::WindowEvent::Resized { width, height });
	}

	// Mouse Events
	fn mouse_button_down_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
		self.app.world_mut().send_event(events::MouseButtonEvent {
			position: glam::vec2(x, y),
			button,
			released: false,
		});
		self.app.world_mut().run_schedule(MiniquadPrivilegedSchedule);
	}

	fn mouse_button_up_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
		self.app.world_mut().send_event(events::MouseButtonEvent {
			position: glam::vec2(x, y),
			button,
			released: true,
		});
		self.app.world_mut().run_schedule(MiniquadPrivilegedSchedule);
	}

	fn mouse_motion_event(&mut self, x: f32, y: f32) {
		let world_mut = self.app.world_mut();

		// x and y are the absolute mouse position, not the delta
		let (first_run, previous) = world_mut.get_resource_ref::<events::CursorProperties>().map(|r| (r.is_added(), r.position)).unwrap();
		let current = vec2(x, y);

		// only send mouse motion events if the mouse has moved and not start of application
		if current != previous && !first_run {
			world_mut.send_event(events::MouseMotionEvent { delta: current - previous });
		}

		// update MousePosition Resource
		let mut cursor = world_mut.get_resource_mut::<events::CursorProperties>().unwrap();
		cursor.position = current;

		world_mut.run_schedule(MiniquadPrivilegedSchedule);
	}

	fn mouse_wheel_event(&mut self, x: f32, y: f32) {
		self.app.world_mut().send_event(events::MouseWheelEvent { delta: glam::vec2(x, y) });
	}

	// Touch Events
	fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
		self.app.world_mut().send_event(events::TouchEvent {
			phase,
			id,
			position: glam::vec2(x, y),
		});
	}

	// Keyboard Events
	fn char_event(&mut self, character: char, mods: miniquad::KeyMods, repeat: bool) {
		self.app.world_mut().send_event(events::KeyCharEvent { character, repeat, mods });
	}

	fn key_down_event(&mut self, keycode: miniquad::KeyCode, mods: miniquad::KeyMods, repeat: bool) {
		self.app.world_mut().send_event(events::KeyCodeEvent {
			keycode,
			mods,
			repeat,
			released: false,
		});
		self.app.world_mut().run_schedule(MiniquadPrivilegedSchedule);
	}

	fn key_up_event(&mut self, keycode: miniquad::KeyCode, mods: miniquad::KeyMods) {
		self.app.world_mut().send_event(events::KeyCodeEvent {
			keycode,
			mods,
			repeat: false,
			released: true,
		});
	}

	// File Drag n' Drop
	fn files_dropped_event(&mut self, paths: Vec<std::path::PathBuf>, bytes: Option<Vec<Vec<u8>>>) {
		let world = self.app.world_mut();
		match bytes {
			Some(s) => {
				debug_assert!(paths.len() == s.len());
				for (path, bytes) in paths.into_iter().zip(s.into_iter()) {
					world.send_event(events::DroppedFileEvent { path, bytes: Some(bytes) });
				}
			}
			None => {
				for path in paths {
					world.send_event(events::DroppedFileEvent { path, bytes: None });
				}
			}
		}

		world.run_schedule(MiniquadPrivilegedSchedule);
	}

	// App Quit
	fn quit_requested_event(&mut self) -> bool {
		self.app.world_mut().run_schedule(MiniquadQuitRequestedSchedule);

		// extract results from schedule
		let quit = self.app.world_mut().resource::<QuitRequested>();
		if quit.accept {
			if let Some(s) = self.sender.take() {
				s.send(quit.status).unwrap();
			}
		}

		quit.accept
	}
}
