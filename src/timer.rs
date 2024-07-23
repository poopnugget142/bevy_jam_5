use std::time::Duration;

use super::*;

#[derive(Component)]
struct ResetText;

#[derive(Resource)]
struct ResetConfig {
    /// How often to spawn a new bomb? (repeating timer)
    timer: Timer,
}


pub(super) fn register(app: &mut App) {
    app
        .insert_resource(ResetConfig {
            // create the repeating timer
            timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
        })
        .add_systems(Startup, setup_text)
        .add_systems(Update, update_reset);
}

fn setup_text(
    mut commands: Commands,
) {
    commands.spawn((
        TextBundle::from_section(
            "10.0",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ResetText,
    ));
}

fn update_reset(
    time: Res<Time>,
    mut config: ResMut<ResetConfig>,
    mut texts: Query<&mut Text, With<ResetText>>,
) {
    // tick the timer
    config.timer.tick(time.delta());

    let mut text = texts.single_mut();
    text.sections[0].value = config.timer.remaining().as_secs().to_string();

    if config.timer.finished() {

    }
}