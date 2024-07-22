// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod hand;

#[derive(Component)]
pub struct ImageSize(Vec2);

#[derive(Component)]
pub struct BoundingBox(Rect);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        // Wasm builds will check for meta files (that don't exist) if this isn't set.
        // This causes errors and even panics in web builds on itch.
        // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_systems(Startup, setup)
    .add_systems(Update, (add_image_size, update_bounding_box));

    hand::register(&mut app);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

//TODO! Only sets image size once
fn add_image_size(
    mut commands: Commands,
    mut sprites: Query<
        (&Transform, &Handle<Image>, Entity),
        (With<Sprite>, Without<ImageSize>, Changed<Handle<Image>>),
    >,
    assets: Res<Assets<Image>>,
) {
    for (transform, image_handle, entity) in sprites.iter_mut() {
        //TODO! possible crash on load with unwrap here! account for none!
        let image_dimensions = assets.get(image_handle).unwrap().size().as_vec2();
        let scaled_image_dimension = image_dimensions * transform.scale.truncate();
        let bounding_box =
            Rect::from_center_size(transform.translation.truncate(), scaled_image_dimension);

        let mut e = commands.get_entity(entity).unwrap();

        e.insert(ImageSize(image_dimensions));

        e.insert(BoundingBox(bounding_box));
    }
}

fn update_bounding_box(mut sprites: Query<(&Transform, &ImageSize, &mut BoundingBox)>) {
    for (transform, image_size, mut bounding_box) in sprites.iter_mut() {
        let scaled_image_dimension = image_size.0 * transform.scale.truncate();
        *bounding_box = BoundingBox(Rect::from_center_size(
            transform.translation.truncate(),
            scaled_image_dimension,
        ));
    }
}
