// ! The current problem is that all bevy systems can be run in parallel, if one
// ! uses bevy's parallel processing plugin. Macroquad is designed to work on a single thread, thus
// ! there needs to be some sort of isolation for ALL of its functionality.

use asset::QuadAssetPlugin;
use bevy::{
    a11y::AccessibilityPlugin, app::PluginGroupBuilder, diagnostic::DiagnosticsPlugin,
    input::InputPlugin, log::LogPlugin, prelude::*, time::TimePlugin,
};
use gizmos::QuadGizmosPlugin;
pub use macroquad;

pub use macroquad::color;
use render::QuadRenderPlugin;
use window::QuadWindowPlugin;

// pub mod sprite;
pub mod prelude;
pub mod window;
pub mod gizmos;
pub mod render;
pub mod asset;
pub mod sprite;

/// This collection of plugins is a custom made [`DefaultPlugins`] bundle.
pub struct QuadifyPlugins;
impl PluginGroup for QuadifyPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LogPlugin::default())
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin)
            .add(FrameCountPlugin)
            .add(TimePlugin)
            .add(TransformPlugin)
            .add(HierarchyPlugin)
            .add(DiagnosticsPlugin)
            .add(InputPlugin)
            .add(WindowPlugin::default())
            .add(AccessibilityPlugin)
            // ? Custom Quadify Plugins. Planning to limit them by features
            .add(QuadWindowPlugin::default())
            .add(QuadRenderPlugin)
            .add(QuadGizmosPlugin)
            .add(QuadAssetPlugin::default())
    }
}
