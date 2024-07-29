use super::*;
use avian2d::prelude::*;

pub(super) fn register(app: &mut App) {
    app
        .insert_resource(Gravity(Vec2::default()))
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ));
}