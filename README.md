# Quadify

## Quadify is a bevy plugin with a minimal set of bevy's features. It uses macroquad for windowing/graphics/sound

If an enormous bevy's dependency tree is too much for your game - you may want to try a simpler solution. This
plugin provides you with macroquad windowing/rendering/sound, while also trying to use existing, known to you bevy's API.
If you're doing simple web-games with 2D graphics - this might be suitable for you.

*(Note: I'm not macroquad nor bevy developer, so please check their respected projects first; I'm just combining these two in a simple plugin.)*

## Planned features:

| feature name | description                   | reference (bevy)               |is required |status|
| ---          | ---                           | ---                            | ---        | ---  |
| quad_window  | Window management and events  | bevy_window                    | ❗        | ⚒️   |
| quad_input   | Input types                   | bevy_input                     | ❗        | ⚒️   |
| parallelism  | Support for parallelism       | None                           | ❗        | ❌   |
| quad_render  | Basic rendering abstractions  | bevy_render/bevy_core_pipeline | ❔        | ❌   |
| quad_asset   | Really basic asset management | bevy_asset                     | ❔        | ❌   |
| quad_sprite  | Sprite rendering              | bevy_sprite                    | ❔        | ❌   |
| quad_text    | Text rendering                | bevy_text                      | ❔        | ❌   |
| quad_ui      | GUI from macroquad            | macroquad megaui               | ❔        | ❌   |
| quad_audio   | Audio functionality           | bevy_audio                     | ❔        | ❌   |

*This list was composed on my personal needs, if the project gains attention I'll maybe try to add other functionality as well (But with no bloat)*

*Note: I'm changing my mind about `quad_ui` in favor of using macroquad's `megaui`. The reason being the simplicity of immediate gui in general.
megaui is a really lightweight version of egui, which is also very customizable and is already baked in. Though, what's likely to happen is the
combination of both.*

## Licensing

I'm leaving the same MIT and APACHE licenses from both projects for you to choose.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
