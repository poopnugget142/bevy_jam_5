use avian2d::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use hand::{CurrentHand, HandActions, Playback};
use leafwing_input_manager::prelude::ActionState;
use object::{Collector, GrabInteractions, Grabbable, Object, ObjectInfo};
use serde::Deserialize;
use bevy::reflect::TypePath;
use bevy_common_assets::toml::TomlAssetPlugin;

use super::*;

//TODO! make load next level to avoid annoying
#[derive(Event)]
pub struct LoadLevel(pub i16);

#[derive(Resource)]
pub struct CurrentLevel(pub i16);

#[derive(Component, Deserialize, Debug, Clone)]
pub struct ColliderInfo {
    pub name: String,
    pub size: Option<Vec<f32>>,
}

#[derive(Deserialize, Debug, Clone)]
struct LoadObject {
    texture_name: Option<String>,
    position: Vec<f32>,
    scale: Vec<f32>,
    collector: Option<Collector>,
    body_static: Option<bool>,
    grabbable: Option<bool>,
    collider_info: Option<ColliderInfo>,
    grab: Option<GrabInteractions>,
    sensor: Option<bool>,
    anchored: Option<bool>,
}

#[derive(Resource)]
struct LevelHandle(Option<Handle<Level>>);

#[derive(Deserialize, Debug, Asset, TypePath, Clone)]
struct Level {
    objects: Vec<LoadObject>,
    background_color: Vec<f32>,
}

pub(super) fn register(app: &mut App) {
    app
        .add_plugins(TomlAssetPlugin::<Level>::new(&["level.toml"]))
        .insert_resource(CurrentLevel(0))
        .add_event::<LoadLevel>()
        .add_systems(Startup, setup)
        .add_systems(Update, (load_level, reload_level, load_event));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
    let level = LevelHandle(Some(asset_server.load("levels/0.level.toml")));
    commands.insert_resource(level);
}

fn reload_level(
    mut ev_level: EventWriter<LoadLevel>,
    current_level: Res<CurrentLevel>,
    hands: Query<&ActionState<HandActions>, With<CurrentHand>>,
){
    for action in hands.iter() {
        if !action.just_pressed(&HandActions::Reload) {
            continue;
        }

        ev_level.send(LoadLevel(current_level.0));
    }
}

fn load_event(
    mut commands: Commands,
    mut ev_level: EventReader<LoadLevel>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_level.read() {
        dbg!(ev.0);
        commands.insert_resource(CurrentLevel(ev.0));
        let level = LevelHandle( Some(asset_server.load(format!("levels/{}.level.toml", ev.0))) );
        commands.insert_resource(level);
    }
}

// for now just load main.toml
fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    objects: Query<Entity, With<Object>>,
    hands: Query<Entity, With<Playback>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut levels: ResMut<Assets<Level>>,
    mut the_level: ResMut<LevelHandle>,
) {
    if the_level.0.is_none() {
        return;
    }

    if let Some(level) = levels.get(the_level.0.clone().unwrap().id()) {

        let level = level.clone();

        *the_level = LevelHandle(None);

        for object in objects.iter() {
            commands.entity(object).despawn();
        }

        for hand in hands.iter() {
            commands.entity(hand).despawn_descendants();
            commands.entity(hand).despawn();
        }

        let background_color = level.background_color;

        //background
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(1280.0, 720.0))),
                material: materials.add(Color::hsl(background_color[0], background_color[1], background_color[2])),
                transform: Transform::from_xyz(0.0, 0.0, -3.0),
                ..default()
            },
            Object
        ));

        for object in level.objects {
            let mut e = commands.spawn(Object);

            if object.texture_name.is_some() {
                let texture = asset_server.load(object.texture_name.clone().unwrap());
                let mut grab = GrabInteractions::Grab;

                if object.grab.is_some() {
                    grab = object.grab.unwrap();
                }

                e.insert((
                    SpriteBundle {
                        texture,
                        transform: Transform::from_xyz(object.position[0], object.position[1], object.position[2])
                            .with_scale(Vec3::new(object.scale[0], object.scale[1], 1.0)),
                        ..default()
                    },
                    ObjectInfo {
                        grab,
                        texture_name: object.texture_name.clone().unwrap(),
                    },
                ));
            } else {
                e.insert(TransformBundle::from_transform(Transform::from_xyz(object.position[0], object.position[1], object.position[2])));
            }

            //TODO! wow all of this code sucks ass
            // I just learned how cool the toml crate is and how better you can make this lol!
            if object.collector.is_some() {
                e.insert(object.collector.unwrap());
            }

            if object.anchored.is_some() {
                e.insert(LockedAxes::ALL_LOCKED);
            }

            if object.sensor.is_some() {
                e.insert(Sensor);
            }

            if object.body_static.is_some() {
                e.insert(RigidBody::Static);
            } else {
                e.insert((
                    RigidBody::Dynamic,
                    LinearDamping(1.0),
                ));
            }

            if object.grabbable.is_some() {
                e.insert(Grabbable);
            }

            if object.collider_info.is_some() {
                let collider_info = object.collider_info.unwrap();

                match collider_info.clone().name.as_str() {
                    "segment" => {
                        let size = collider_info.size.unwrap().clone();
                        e.insert(Collider::segment(Vec2::new(0.0, 0.0), Vec2::new(size[0], size[1])));
                    }
                    _ => {
                        e.insert(collider_info);
                    }
                }
            } else {
                e.insert(ColliderInfo {
                    name: "rectangle".into(),
                    size: None,
                });
            }
        }
    }
}