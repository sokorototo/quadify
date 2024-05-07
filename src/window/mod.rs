use bevy_app::{App, Plugin};
use miniquad::conf::Conf;

use crate::state::QuadifyState;

mod icon;

/// Initializes main window and starts the `miniquad` event loop
pub struct WindowPlugin {
	pub title: String,
	pub width: i32,
	pub height: i32,
	pub fullscreen: bool,
	pub high_dpi: bool,
	pub window_resizable: bool,
	pub icon: Option<icon::WindowIcon>,
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
			window_resizable: conf.window_resizable,
			icon: None,
		}
	}
}

impl Plugin for WindowPlugin {
	fn build(&self, app: &mut App) {
		let mut conf = Conf::default();

		conf.window_title = self.title.clone();
		conf.window_width = self.width;
		conf.window_height = self.height;
		conf.fullscreen = self.fullscreen;
		conf.high_dpi = self.high_dpi;
		conf.window_resizable = self.window_resizable;

		if let Some(icon) = &self.icon {
			// TODO: Log when Icon conversion fails
			conf.icon = icon.try_into().ok();
		}

		// Init Runner
		app.set_runner(move |app| {
			miniquad::start(conf, move || Box::new(QuadifyState::new(app)));
		});
	}
}
