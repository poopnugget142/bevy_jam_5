use std::time::Duration;

use super::*;

#[derive(Component)]
pub struct Recording{
    timer: Timer,
    record: Vec<(Duration, Vec2)>
}

pub(super) fn register(app: &mut App) {
    app
        .add_systems(FixedUpdate, (record, recording));
}

fn record (
    mut commands: Commands,
    hands: Query<(&ActionState<HandActions>, Entity), With<CurrentHand>>,
) {
    let (action, entity) = hands.single();
    
    if !action.just_pressed(&HandActions::Record) {
        return;
    }

    println!("AH");

    let mut entity_commands = commands.entity(entity);

    entity_commands.insert(Recording {
        timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
        record: Vec::new(),
    });
}

fn recording (
    mut commands: Commands,
    mut hands: Query<(&mut Recording, Entity, &Goal)>,
    time: Res<Time>,
) {
    for (mut recording, entity, goal) in hands.iter_mut() {
        recording.timer.tick(time.delta());

        let elasped = recording.timer.elapsed();
        let goal_position = goal.0.clone();

        recording.record.push((elasped, goal_position));

        if recording.timer.finished() {
            dbg!(recording.record.clone());

            let mut entity_commands = commands.entity(entity);
            entity_commands.remove::<Recording>();
        }
    }
}