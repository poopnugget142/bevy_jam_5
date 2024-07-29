// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::window::WindowResolution;

mod hand;
mod object;
mod physics;
mod camera;
mod level;
mod submit;

pub const SCREEN_W : f32 = 1280.0;
pub const SCREEN_H : f32 = 720.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_W, SCREEN_H),
                resizable: false,
                ..default()
            }),
            ..default()
        }));

    hand::register(&mut app);
    object::register(&mut app);
    physics::register(&mut app);
    camera::register(&mut app);
    level::register(&mut app);
    submit::register(&mut app);

    app.run();
}