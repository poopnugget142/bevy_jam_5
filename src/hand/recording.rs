use std::time::Duration;
use super::*;

#[derive(Component)]
pub struct Recording{
    pub timer: Timer,
    record: Vec<(Duration, Vec2)>,
    pub grabs: Vec<Duration>
}

#[derive(Component)]
pub struct Playback{
    timer: Timer,
    record: Vec<(Duration, Vec2)>,
    stored_record: Vec<(Duration, Vec2)>,
    grabs: Vec<Duration>,
    stored_grabs: Vec<Duration>,
}

const RECORDING_TIME: u64 = 3;

pub(super) fn register(app: &mut App) {
    app
        .add_systems(Update, (record, playback))
        .add_systems(FixedUpdate, recording);
}

fn record (
    mut commands: Commands,
    hands: Query<(&ActionState<HandActions>, Entity), With<CurrentHand>>,
) {
    let (action, entity) = hands.single();
    
    //TODO! disallow for more than one recording at once
    if !action.just_pressed(&HandActions::Record) {
        return;
    }

    //TODO! add some sort of indincator that you are recording
    println!("AH");

    let mut entity_commands = commands.entity(entity);

    entity_commands.insert(Recording {
        timer: Timer::new(Duration::from_secs(RECORDING_TIME), TimerMode::Once),
        record: Vec::new(),
        grabs: Vec::new(),
    });
}

fn recording (
    mut commands: Commands,
    mut hands: Query<(&mut Recording, Entity, &Goal)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    for (mut recording, entity, goal) in hands.iter_mut() {
        recording.timer.tick(time.delta());

        let elasped = recording.timer.elapsed();
        let goal_position = goal.0.clone();

        recording.record.push((elasped, goal_position));

        if recording.timer.finished() {
            let mut entity_commands = commands.entity(entity);
            entity_commands.remove::<Recording>();

            let mut record = recording.record.clone();

            record.reverse();

            let mut grabs = recording.grabs.clone();

            grabs.reverse();

            let texture = asset_server.load("hand.png");

            // Spawns the hand!
            commands.spawn((
                HandBundle::default(),
                SpriteBundle {
                    texture,
                    transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)),
                    ..default()
                },
                ActionState::<HandActions>::default(),
                Playback {
                    timer: Timer::new(Duration::from_secs(RECORDING_TIME), TimerMode::Repeating),
                    record: record.clone(),
                    stored_record: record.clone(),
                    grabs: grabs.clone(),
                    stored_grabs: grabs.clone(),
                },
            )).with_children(|parent| {
                parent.spawn((
                    TransformBundle::from_transform(Transform::from_xyz(0.0, -HAND_OFFSET, 0.0)),
                    Collider::rectangle(400.0, 600.0),
                    Sensor,
                ));
            });
        }
    }
}

fn playback (
    mut hands: Query<(&mut Playback, &mut Goal, &mut ActionState<HandActions>)>,
    time: Res<Time>,
) {
    for (mut playback, mut goal, mut action) in hands.iter_mut() {
        playback.timer.tick(time.delta());

        if playback.timer.finished() {
            playback.record = playback.stored_record.clone();
            playback.grabs = playback.stored_grabs.clone();
        }

        let (next_time, goal_position) = *playback.record.last().unwrap();

        if playback.timer.elapsed() >= next_time {
            *goal = Goal(goal_position);
            playback.record.pop();
        }

        if action.pressed(&HandActions::Grab) {
            action.release(&HandActions::Grab);
        }

        if playback.grabs.is_empty() {
            continue;
        }

        let next_grab= *playback.grabs.last().unwrap();

        if playback.timer.elapsed() >= next_grab {
            action.press(&HandActions::Grab);
            playback.grabs.pop();
            continue;
        }
    }
}