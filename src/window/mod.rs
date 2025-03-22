use bevy_app::{App, AppExit, Last, Plugin};
use bevy_ecs::schedule::ExecutorKind;
use miniquad::conf::{Conf, PlatformSettings};

pub(crate) mod events;
pub(crate) mod icon;
pub(crate) mod state;

/// Initializes main window and starts the `miniquad` event loop.
pub struct WindowPlugin {
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub fullscreen: bool,
	pub high_dpi: bool,
	pub resizeable: bool,
	pub icon: Option<icon::WindowIcon>,
	pub default_cursor: Option<miniquad::CursorIcon>,
	/// Platform specific settings. See [`miniquad::conf::Platform`]
	pub platform: Option<PlatformSettings>,
}

impl Default for WindowPlugin {
	fn default() -> Self {
		let conf = Conf::default();

		Self {
			title: conf.window_title,
			width: conf.window_width,
			height: conf.window_height,
			fullscreen: conf.fullscreen,
			high_dpi: conf.high_dpi,
			resizeable: conf.window_resizable,
			default_cursor: None,
			icon: None,
			platform: None,
		}
	}
}

impl Plugin for WindowPlugin {
	fn build(&self, app: &mut App) {
		let mut conf = Conf {
			window_title: self.title.clone(),
			window_width: self.width,
			window_height: self.height,
			fullscreen: self.fullscreen,
			high_dpi: self.high_dpi,
			window_resizable: self.resizeable,
			..Default::default()
		};

		if let Some(icon) = &self.icon {
			conf.icon = icon.try_into().ok();
		}

		if let Some(platform) = &self.platform {
			conf.platform = platform.clone();
		}

		// Empty entity to identify the main window, for compatibility with Bevy's multiwindow support
		let window_properties = events::WindowProperties {
			position: None,
			width: self.width,
			height: self.height,
			fullscreen: self.fullscreen,
		};
		let cursor_properties = events::CursorProperties {
			position: glam::vec2(0.0, 0.0),
			grabbed: false,
			icon: miniquad::CursorIcon::Default,
		};

		// Init Resources, Events, and Systems
		app.add_event::<events::WindowEvent>()
			.add_event::<events::DroppedFileEvent>()
			.add_event::<events::KeyCharEvent>()
			.add_event::<events::KeyCodeEvent>()
			.add_event::<events::MouseButtonEvent>()
			.add_event::<events::MouseMotionEvent>()
			.add_event::<events::MouseWheelEvent>()
			.add_event::<events::TouchEvent>()
			.insert_resource(window_properties)
			.insert_resource(cursor_properties)
			.insert_resource(state::QuitRequested { accept: true, status: 0 })
			.init_schedule(state::MiniquadPrepareDraw)
			.edit_schedule(state::MiniquadPrepareDraw, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadDraw)
			.edit_schedule(state::MiniquadDraw, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadEndDraw)
			.edit_schedule(state::MiniquadEndDraw, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadPrivilegedSchedule)
			.edit_schedule(state::MiniquadPrivilegedSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadQuitRequestedSchedule)
			.edit_schedule(state::MiniquadQuitRequestedSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.add_systems(Last, (events::apply_window_properties, events::apply_cursor_properties, events::quit_on_app_exit));

		// Init Runner
		app.set_runner(move |app| {
			let (sender, receiver) = oneshot::channel();

			miniquad::start(conf, move || Box::new(state::QuadifyState::new(app, sender)));
			match receiver.recv().unwrap() {
				0 => AppExit::Success,
				n => AppExit::Error(std::num::NonZeroU8::new(n).unwrap()),
			}
		});
	}
}
