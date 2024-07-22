use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use leafwing_input_manager::prelude::*;

use crate::BoundingBox;

#[derive(Component)]
pub struct Hand;

#[derive(Component)]
pub struct CurrentHand;

#[derive(Component)]
pub struct Grabbing(Entity);

#[derive(Component)]
pub struct Grabbable;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum HandActions {
    Grab,
    Move,
}

const HAND_OFFSET: f32 = -500.0;

pub(super) fn register(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<HandActions>::default())
        .add_systems(Startup, spawn_hand)
        .add_systems(Update, (move_hand, grab, drop));
}

fn spawn_hand(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("ace_hearts.png");

    //Spawns card
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_scale(Vec3::new(0.25, 0.25, -1.0)),
            ..default()
        },
        Grabbable,
    ));

    //Input map
    let input_map = InputMap::new([(HandActions::Grab, MouseButton::Left)]);

    // Spawns the hand!
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 1000.0))),
            material: materials.add(Color::hsl(18.0, 0.57, 0.79)),
            ..default()
        },
        InputManagerBundle::with_map(input_map),
        Hand,
        CurrentHand,
    ));
}

fn drop(
    mut commands: Commands,
    hands: Query<(&ActionState<HandActions>, Entity, &Grabbing), With<CurrentHand>>,
    mut objects: Query<(&GlobalTransform, &mut Transform)>,
) {
    if hands.is_empty() {
        return;
    }

    let (action, hand, grabbing) = hands.single();

    if !action.just_pressed(&HandActions::Grab) {
        return;
    }

    let mut hand_commands = commands.entity(hand);

    let (global_transform, mut transform) = objects.get_mut(grabbing.0).unwrap();

    let stored_transform = global_transform;
    hand_commands.remove_children(&[grabbing.0]);
    hand_commands.remove::<Grabbing>();

    let mut position = stored_transform.translation();

    position.z = -1.0;

    transform.translation = position;
}

fn grab(
    mut commands: Commands,
    //TODO! see if possible work around for this because it's annoying
    mut hands_and_objects: ParamSet<(
        Query<
            (&ActionState<HandActions>, &Transform, Entity),
            (With<CurrentHand>, Without<Grabbing>),
        >,
        Query<(&BoundingBox, Entity, &mut Transform), With<Grabbable>>,
    )>,
) {
    let binding = hands_and_objects.p0();

    if binding.is_empty() {
        return;
    }

    let (action, transform, hand) = binding.single();

    if !action.just_pressed(&HandActions::Grab) {
        return;
    }

    let hand_spot = transform.translation + Vec3::new(0.0, -HAND_OFFSET, 0.0);

    for (bounding_box, object, mut object_transform) in hands_and_objects.p1().iter_mut() {
        // Clicked on grabble thing attach to hand
        if bounding_box.0.contains(hand_spot.truncate()) {
            let mut hand_commands = commands.entity(hand);
            hand_commands.insert(Grabbing(object));
            hand_commands.add_child(object);

            object_transform.translation = Vec3::new(0.0, -HAND_OFFSET, 1.0);
        }
    }
}

fn move_hand(
    mut hands: Query<&mut Transform, With<CurrentHand>>,
    windows: Query<&mut Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let mut transform = hands.single_mut();
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        transform.translation = Vec3::new(cursor_position.x, cursor_position.y + HAND_OFFSET, 0.0);
    }
}
