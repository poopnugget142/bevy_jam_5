use super::*;

use avian2d::prelude::*;
use hand::Grabbing;
use level::{ColliderInfo, CurrentLevel, LoadLevel};
use serde::Deserialize;
use submit::Deliveries;

#[derive(Deserialize, Debug, Clone)]
enum CollectorInteraction {
    FinishLevel,
    Count,
}

#[derive(Component, Deserialize, Debug, Clone)]
pub struct Collector {
    collecting: String,
    interaction: CollectorInteraction,
}

#[derive(Deserialize, Debug, Clone)]
pub enum GrabInteractions {
    Grab,
    Spawn,
}

#[derive(Component)]
pub struct ObjectInfo {
    pub grab: GrabInteractions,
    // TODO! this is stupid but I need this to spawn new things with same image
    pub texture_name: String,
}

#[derive(Component)]
pub struct Object;

#[derive(Component)]
pub struct Hitbox;

#[derive(Component)]
pub struct Grabbable;

#[derive(Component)]
pub struct Grabbed(pub Entity);

pub(super) fn register(app: &mut App) {
    app
        .add_systems(Update, (add_image_size, collector_collide));
}

//TODO! Only sets image size once
fn add_image_size(
    mut commands: Commands,
    mut sprites: Query<(&Handle<Image>, &ColliderInfo, Entity), (With<Object>, Without<Hitbox>)>,
    has_rigid_body: Query<&RigidBody>,
    assets: Res<Assets<Image>>,
) {
    for (image_handle, collider_info, entity) in sprites.iter_mut() {
        let image = match assets.get(image_handle) {
            Some(image) => image,
            None => {
                return;
            }
        };

        let image_dimensions = image.size().as_vec2();

        let mut e = commands.get_entity(entity).unwrap();

        e.insert(Hitbox);

        if has_rigid_body.get(entity).is_ok() {
            match collider_info.name.as_str() {
                "rectangle" => {
                    e.insert(Collider::rectangle(image_dimensions.x, image_dimensions.y));
                }
                "circle" => {
                    e.insert(Collider::circle(image_dimensions.x/2.0));
                }
                _ => {
                    e.insert(Collider::rectangle(image_dimensions.x, image_dimensions.y));
                }
            }
        }
    }
}

fn collector_collide(
    mut commands: Commands,
    query: Query<(&CollidingEntities, &Collector)>,
    objects: Query<&ObjectInfo>,
    is_grabbed: Query<&mut Grabbed>,
    mut ev_level: EventWriter<LoadLevel>,
    current_level: Res<CurrentLevel>,
    mut deliveries: ResMut<Deliveries>,
) {
    for (colliding_entities, collector) in &query {
        for other_entity in colliding_entities.0.clone() {
            let object = objects.get(other_entity);

            if object.is_err() {
                continue;
            }

            let object = object.unwrap();

            if object.texture_name != collector.collecting {
                continue;
            }

            let grabbed = is_grabbed.get(other_entity);
            if grabbed.is_ok() {
                let hand = grabbed.unwrap().0;
                
                let mut hand_commands = commands.entity(hand);
    
                hand_commands.remove::<Grabbing>();
            }

            match collector.interaction {
                CollectorInteraction::FinishLevel => {
                    ev_level.send(LoadLevel(current_level.0+1));
                }
                CollectorInteraction::Count => {
                    deliveries.0 += 1;
                }
            }

            let mut e = commands.entity(other_entity);

            e.despawn();
        }
    }
}