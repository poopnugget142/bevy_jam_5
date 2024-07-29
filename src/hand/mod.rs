use super::*;

use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;
use level::ColliderInfo;
use object::{GrabInteractions, Grabbable, Grabbed, Object, ObjectInfo};
use recording::Recording;

mod recording;

pub use recording::Playback;

#[derive(Component)]
pub struct Hand;

#[derive(Component)]
pub struct CurrentHand;

#[derive(Component)]
pub struct Grabbing(Entity);

#[derive(Component)]
pub struct Goal(Vec2);

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum HandActions {
    Grab,
    Record,
    Reload,
}

#[derive(Bundle)]
struct HandBundle {
    name: Name,
    goal: Goal,
    rigid_body: RigidBody,
    velocity: LinearVelocity,
    locked: LockedAxes,
    hand: Hand,
}

impl Default for HandBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Hand"),
            goal: Goal(Vec2::new(0.0, 0.0)),
            rigid_body: RigidBody::Dynamic,
            velocity: LinearVelocity::ZERO,
            locked: LockedAxes::ROTATION_LOCKED,
            hand: Hand,
        }
    }
}

const HAND_OFFSET: f32 = -200.0;

pub(super) fn register(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<HandActions>::default())
        .add_systems(Startup, spawn_hand)
        .add_systems(Update, (move_hand, grab, drop))
        .add_systems(FixedUpdate, update_goal);

    recording::register(app);
}

fn spawn_hand(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    //Input map
    let mut input_map = InputMap::new([
        (HandActions::Grab, MouseButton::Left),
        (HandActions::Record, MouseButton::Right),
    ]);

    input_map.insert(HandActions::Record, KeyCode::Space);
    input_map.insert(HandActions::Reload, KeyCode::KeyR);

    let texture = asset_server.load("hand.png");

    // Spawns the hand!
    commands.spawn((
        HandBundle::default(),
        SpriteBundle {
            texture,
            transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)),
            ..default()
        },
        InputManagerBundle::with_map(input_map),
        CurrentHand,
    )).with_children(|parent| {
        parent.spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.0, -HAND_OFFSET, 0.0)),
            Collider::rectangle(400.0, 600.0),
            Sensor,
        ));
    });
}

fn drop(
    mut commands: Commands,
    hands: Query<(&ActionState<HandActions>, Entity, &Grabbing), With<Hand>>,
    joints: Query<&FixedJoint>,
    mut is_recording: Query<&mut Recording, With<CurrentHand>>,
) {
    for (action, hand, grabbing) in hands.iter() {
        if !action.just_pressed(&HandActions::Grab) {
            continue;
        }

        if let Ok(mut recording) = is_recording.get_mut(hand) {
            let elapsed = recording.timer.elapsed();
    
            recording.grabs.push(elapsed);
        }
    
        let mut hand_commands = commands.entity(hand);
    
        hand_commands.remove::<Grabbing>();
        let joint = grabbing.0;
    
        let object = joints.get(joint).unwrap().entity1;
    
        let mut object_commands = commands.entity(object);
        object_commands.insert(Grabbable);
        object_commands.remove::<Grabbed>();

        commands.entity(joint).despawn();
    }
}

fn grab(
    mut commands: Commands,
    hands: Query<
        (&ActionState<HandActions>, Entity, &Children),
        (With<Hand>, Without<Grabbing>),
    >,
    collisions: Query<&CollidingEntities>,
    objects: Query<(&Transform, &ObjectInfo, &ColliderInfo), With<Grabbable>>,
    //TODO! only one hand can record current hand isn't required
    mut is_recording: Query<&mut Recording, With<CurrentHand>>,
    asset_server: Res<AssetServer>,
) {
    for (action, hand, children) in hands.iter() {
        if !action.just_pressed(&HandActions::Grab) {
            continue;
        }
    
        if let Ok(mut recording) = is_recording.get_mut(hand) {
            let elapsed = recording.timer.elapsed();
    
            recording.grabs.push(elapsed);
        }
    
        for child in children.iter() {
            let colliding_entities = collisions.get(*child).unwrap();
    
            if colliding_entities.is_empty() {
                continue;
            }
    
            let mut object = None;
            //TODO! stupidest thing I've ever seen HOW DO YOU CODE IN RUST
            for x in colliding_entities.0.iter() {
                if objects.get(*x).is_ok() {
                    object = Some(x);
                    break;
                }
            }
    
            if object.is_none() {
                continue;
            }

            let mut object = *object.unwrap();

            let (transform, interaction, collider_info) = objects.get(object).unwrap();

            match interaction.grab {
                GrabInteractions::Grab => {
                    let mut object_commands = commands.entity(object);
                    object_commands.remove::<Grabbable>();
                    object_commands.insert(Grabbed(hand));
    
                    let mut joint = FixedJoint::new(object, hand);
                    joint.local_anchor2 = Vec2::new(0.0, -HAND_OFFSET);
            
                    let joint_commands = commands.spawn(joint);
                    let joint_entity = joint_commands.id();
            
                    let mut hand_commands = commands.entity(hand);
                    hand_commands.insert(Grabbing(joint_entity));
                },
                GrabInteractions::Spawn => {
                    let texture = asset_server.load(interaction.texture_name.clone());
                    object = commands.spawn((
                        SpriteBundle {
                            texture,
                            transform: *transform,
                            ..default()
                        },
                        RigidBody::Dynamic,
                        LinearDamping(1.0),
                        Object,
                        collider_info.clone(),
                        ObjectInfo {
                            grab: GrabInteractions::Grab,
                            texture_name: interaction.texture_name.clone(),
                        }
                    )).id();

                    let mut object_commands = commands.entity(object);
                    object_commands.insert(Grabbed(hand));
    
                    let mut joint = FixedJoint::new(object, hand);
                    joint.local_anchor2 = Vec2::new(0.0, -HAND_OFFSET);
            
                    let joint_commands = commands.spawn(joint);
                    let joint_entity = joint_commands.id();
            
                    let mut hand_commands = commands.entity(hand);
                    hand_commands.insert(Grabbing(joint_entity));
                },
            }
        }
    }
}

fn move_hand(
    mut hands: Query<(&Transform, &mut LinearVelocity, &Goal), With<Hand>>,
) {
    for (transform, mut velocity, goal) in hands.iter_mut() {
        let hand_position = transform.translation.truncate();
        let cursor_dir = goal.0 - hand_position;

        velocity.x = cursor_dir.x * 5.0;
        velocity.y = cursor_dir.y * 5.0;
    }
}

fn update_goal(
    mut hands: Query<&mut Goal, With<CurrentHand>>,
    windows: Query<&mut Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    let mut goal = hands.single_mut();

    if let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let goal_position = Vec2::new(cursor_position.x, cursor_position.y + HAND_OFFSET);
        *goal = Goal(goal_position);
    }
}
