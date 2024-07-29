use super::*;

pub(super) fn register(app: &mut App) {
    app
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
