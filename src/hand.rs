use super::*;

use avian2d::math::Vector;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use leafwing_input_manager::prelude::*;

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
        RigidBody::Dynamic,
        Grabbable,
        LinearDamping(1.0),
    ));

    //background
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(1280.0, 720.0))),
            material: materials.add(Color::hsl(0.0, 0.50, 0.50)),
            transform: Transform::from_xyz(0.0, 0.0, -2.0),
            ..default()
        },
    ));

    let left_corner = Transform::from_xyz(-SCREEN_W/2.0, -SCREEN_H/2.0, 0.0);
    let right_corner = Transform::from_xyz(SCREEN_W/2.0, SCREEN_H/2.0, 0.0);

    //WALLS
    commands.spawn((
        RigidBody::Static,
        Collider::segment(Vector::new(0.0, 0.0), Vector::new(0.0, SCREEN_H)),
        TransformBundle::from_transform(left_corner),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::segment(Vector::new(0.0, 0.0), Vector::new(SCREEN_W, 0.0)),
        TransformBundle::from_transform(left_corner),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::segment(Vector::new(0.0, 0.0), Vector::new(0.0, -SCREEN_H)),
        TransformBundle::from_transform(right_corner),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::segment(Vector::new(0.0, 0.0), Vector::new(-SCREEN_W, 0.0)),
        TransformBundle::from_transform(right_corner),
    ));

    //Input map
    let input_map = InputMap::new([(HandActions::Grab, MouseButton::Left)]);

    // Spawns the hand!
    commands.spawn((
        Name::new("Hand"),
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 1000.0))),
            material: materials.add(Color::hsl(18.0, 0.57, 0.79)),
            ..default()
        },
        InputManagerBundle::with_map(input_map),
        RigidBody::Dynamic,
        LinearVelocity::ZERO,
        LockedAxes::ROTATION_LOCKED,
        Hand,
        CurrentHand,
    ));
}

fn drop(
    mut commands: Commands,
    hands: Query<(&ActionState<HandActions>, Entity, &Grabbing), With<CurrentHand>>,
    joints: Query<&FixedJoint>,
    mut objects: Query<&mut Transform>,
) {
    if hands.is_empty() {
        return;
    }

    let (action, hand, grabbing) = hands.single();

    if !action.just_pressed(&HandActions::Grab) {
        return;
    }

    let mut hand_commands = commands.entity(hand);

    hand_commands.remove::<Grabbing>();
    let joint = grabbing.0;

    let object = joints.get(joint).unwrap().entity1;

    commands.entity(joint).despawn();

    let mut transform = objects.get_mut(object).unwrap();

    transform.translation.z = -1.0;
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
            let mut joint = FixedJoint::new(object, hand);
            joint.local_anchor2 = Vec2::new(0.0, -HAND_OFFSET);

            let joint_commands = commands.spawn(joint);
            let joint_entity = joint_commands.id();

            let mut hand_commands = commands.entity(hand);
            hand_commands.insert(Grabbing(joint_entity));

            object_transform.translation.z = 1.0;
        }
    }
}

fn move_hand(
    mut hands: Query<(&Transform, &mut LinearVelocity), With<CurrentHand>>,
    windows: Query<&mut Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let (transform, mut velocity) = hands.single_mut();
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let hand_position = transform.translation.truncate();
        let goal_position = Vec2::new(cursor_position.x, cursor_position.y + HAND_OFFSET);
        let cursor_dir = goal_position - hand_position;

        velocity.x = cursor_dir.x * 5.0;
        velocity.y = cursor_dir.y * 5.0;
    }
}
