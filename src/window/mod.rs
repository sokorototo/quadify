use bevy_app::{App, Last, Plugin};
use bevy_ecs::schedule::ExecutorKind;
use miniquad::conf::{Conf, PlatformSettings};

mod conversions;
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
		let window_entity = app.world.spawn(()).id();
		let window_properties = events::WindowProperties {
			window: window_entity,

			position: None,
			width: self.width,
			height: self.height,
			fullscreen: self.fullscreen,

			cursor_grabbed: false,
			cursor: miniquad::CursorIcon::Default,
			cursor_position: glam::Vec2::ZERO,
		};

		// Init Resources, Events, and Systems
		app.add_event::<events::WindowEvent>()
			.add_event::<events::DroppedFileEvent>()
			.insert_resource(window_properties)
			.insert_resource(state::AcceptQuitRequest(true))
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
			.init_schedule(state::MiniquadKeyDownSchedule)
			.edit_schedule(state::MiniquadKeyDownSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadMouseDownSchedule)
			.edit_schedule(state::MiniquadMouseDownSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadMouseMotionSchedule)
			.edit_schedule(state::MiniquadMouseMotionSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.init_schedule(state::MiniquadQuitRequestedSchedule)
			.edit_schedule(state::MiniquadQuitRequestedSchedule, |s| {
				s.set_executor_kind(ExecutorKind::SingleThreaded);
			})
			.add_systems(Last, (events::apply_window_properties, events::quit_on_app_exit));

		// Init Runner
		app.set_runner(move |app| {
			miniquad::start(conf, move || Box::new(state::QuadifyState::new(app, window_entity)));
		});
	}
}
