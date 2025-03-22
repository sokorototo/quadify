use bevy_app::*;
use bevy_ecs::{
	event::EventReader,
	system::{Local, Res, ResMut},
};
use quadify::prelude::*;

#[test]
fn main() {
	App::new()
		.add_plugins(QuadifyPlugins.set(WindowPlugin {
			title: "Cancel Exit Test".to_string(),
			width: 600,
			height: 600,
			high_dpi: true,
			resizeable: false,
			..Default::default()
		}))
		.add_plugins(bevy_time::TimePlugin)
		.add_systems(Startup, || {
			println!("Press [X] twice to exit, demonstrates blocking exit");
		})
		.add_systems(MiniquadQuitRequestedSchedule, toggle_exit)
		// Run a System just before the application Quits
		.add_systems(Last, |mut quit: EventReader<AppExit>, tick: Res<bevy_time::Time>| {
			for _ in quit.read() {
				println!("[{}] Quit Permitted, Bye Folks!", tick.elapsed_secs());
			}
		})
		.run();
}

fn toggle_exit(mut first_run: Local<bool>, mut exit_request: ResMut<QuitRequested>, tick: Res<bevy_time::Time>) {
	let f_run = !*first_run; // defaults to false

	if f_run {
		println!("[{}] Cancelling Exit", tick.elapsed_secs());
		exit_request.accept = false;
		*first_run = true; // Inverted
	} else {
		println!("[{}] Permitting Exit", tick.elapsed_secs());
		exit_request.accept = true;
		exit_request.status = 1;
	}
}
