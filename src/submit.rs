use level::{CurrentLevel, LoadLevel};

use super::*;

#[derive(Resource)]
pub struct Deliveries(pub i16);

pub(super) fn register(app: &mut App) {
    app
        .insert_resource(Deliveries(0))
        .add_systems(Update, submitting);
}

fn submitting (
    windows: Query<&mut Window>,
    mut deliveries: ResMut<Deliveries>,
    mut ev_level: EventWriter<LoadLevel>,
    current_level: Res<CurrentLevel>,
) {
    let window = windows.single();

    if let None = window.cursor_position() {
        // submitting
        if deliveries.0 >= 3 {
            ev_level.send(LoadLevel(current_level.0+1));
            deliveries.0 = 0;
        }
    } else {
        // not submitting
        deliveries.0 = 0;
    }
}