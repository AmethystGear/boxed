// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use std::collections::hash_map::Entry;

use assetloader::{HandleMap, ImageKey};
use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};
use bevy_prototype_lyon::path::PathBuilder;
use bevy_prototype_lyon::prelude::*;
use bevy_simple_text_input::{TextInputPlugin, TextInputSystem};
use mouse::{mouse_world_coords, MouseWorldCoords};
use rand::Rng;
use rendering::{fit_canvas, setup_camera};
use server::{update_server_visuals, Server, ServerState, WireKind};
use text_input::{ focus, setup_textbox};
use wire::{setup_env, LineMaterial};

mod assetloader;
mod mouse;
mod rendering;
mod server;
mod text_input;
mod wire;

const TILE_SIZE: f32 = 16.0;
/* 
fn setup_env(
    mut commands: Commands,
    handle_map: Res<HandleMap<ImageKey>>,
    mut connections: ResMut<Connections>,
) {
    // setup tiles
    for x in -50..50 {
        for y in -50..50 {
            commands.spawn(SpriteBundle {
                texture: handle_map[&ImageKey::Tile].clone(),
                transform: get_transform((x, y), 1.0, -1.0),
                ..default()
            });
        }
    }

    let mut rng = rand::thread_rng();
    let powers: Vec<_> = (0..2)
        .into_iter()
        .map(|_| (rng.gen_range(-10..10), rng.gen_range(-10..10)))
        .map(|(x, y)| {
            let transform = get_transform((x, y), 1.0, 0.0);
            (
                commands
                    .spawn(SpriteBundle {
                        texture: handle_map[&ImageKey::Power].clone(),
                        transform,
                        ..default()
                    })
                    .id(),
                transform,
            )
        })
        .collect();

    let mut i = 0;
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    let servers: Vec<_> = (0..8)
        .into_iter()
        .map(|_| (rng.gen_range(-10..10), rng.gen_range(-10..10)))
        .map(|(x, y)| {
            let mut rng = rand::thread_rng();
            let transform = get_transform((x, y), 2.0, 0.0);
            let state = if i == 0 {
                ServerState::Infected
            } else {
                if rng.gen_bool(0.3) {
                    ServerState::Healthy
                } else {
                    ServerState::Hacked
                }
            };
            let name = alphabet
                .chars()
                .nth(i)
                .expect("too many servers")
                .to_string();
            i += 1;

            commands.spawn(Text2dBundle {
                text: Text::from_section(name.clone(), TextStyle::default()),
                transform: transform
                    .with_scale(Vec3::new(0.5, 0.5, 1.0))
                    .with_translation(transform.translation + Vec3::new(-12., -12., 0.0)),
                ..default()
            });
            (
                commands
                    .spawn((
                        SpriteBundle {
                            texture: handle_map[&ImageKey::Server(state)].clone(),
                            transform,
                            ..default()
                        },
                        Server {
                            state,

                            name,
                        },
                    ))
                    .id(),
                transform,
            )
        })
        .collect();

    // connect every server to the closest power source
    for &server in &servers {
        let &power = powers
            .iter()
            .min_by(|(_, a), (_, b)| {
                let a = a.translation.distance_squared(server.1.translation);
                let b = b.translation.distance_squared(server.1.translation);
                a.partial_cmp(&b).expect("tried to compare NaN")
            })
            .expect("no power sources?");
        connect(
            &mut commands,
            server,
            power,
            WireKind::Power,
            &mut connections,
        );
    }

    // randomly connect some servers together
    for i in 0..servers.len() {
        for j in (i + 1)..servers.len() {
            if rng.gen_bool(0.25) {
                connect(
                    &mut commands,
                    servers[i],
                    servers[j],
                    WireKind::Communication,
                    &mut connections,
                );
            }
        }
    }

    // TODO: ensure every server is reachable from every other server. Simplest way I can think of is to do some kind of
    // flood fill to find seperated networks, then connect the two closest nodes from two seperated graphs
    // and then repeat.
    //
    // probably there is some fancy smart mincut maxcut graph algo but we will not have enough nodes
    // for it to matter.

    // TODO: throw out generations where things overlap, or a wire goes underneath and through a server, because those
    // make things unclear/confusing.
}

fn random_vec3_2d() -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(rng.gen(), rng.gen(), 0.0) + Vec3::new(-0.5, -0.5, 0.0)
}

fn connect(
    commands: &mut Commands,
    a: (Entity, Transform),
    b: (Entity, Transform),
    wire_kind: WireKind,
    connections: &mut Connections,
) {
    for (a, b) in [(a, b), (b, a)] {
        let val = (connections.wires.len(), b.0);
        match connections.graph.entry(a.0) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(val);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![val]);
            }
        }
    }
    // TODO: rn i'm using bevy_prototype_lyon to generate these bezier curves,
    // but that won't work for animation or cutting lines (unless we just slap on a particle effect for the broken wire).
    // either i'll need to generate the curves myself with something like https://bevyengine.org/examples-webgpu/3d-rendering/lines/
    // or do shader stuff.
    let mut path_builder = PathBuilder::new();
    let diff = b.1.translation - a.1.translation;
    path_builder.move_to(Vec2::new(0., 0.));
    path_builder.cubic_bezier_to(
        (diff * 0.2 + random_vec3_2d() * 128.0).truncate(),
        (diff * 0.8 + random_vec3_2d() * 128.0).truncate(),
        diff.truncate(),
    );
    let path = path_builder.build();

    connections.wires.push(
        commands
            .spawn((
                ShapeBundle {
                    path,
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(
                            a.1.translation.x,
                            a.1.translation.y,
                            match wire_kind {
                                WireKind::Power => -0.5,
                                WireKind::Communication => -0.4,
                            },
                        ),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(
                    match wire_kind {
                        WireKind::Power => RED,
                        WireKind::Communication => BLUE,
                    },
                    1.0,
                ),
            ))
            .id(),
    );
}

fn get_transform(loc: (i32, i32), size: f32, z: f32) -> Transform {
    Transform::from_xyz(
        loc.0 as f32 * TILE_SIZE - TILE_SIZE * size * 0.5,
        loc.1 as f32 * TILE_SIZE - TILE_SIZE * size * 0.5,
        z,
    )
}
*/
fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(MouseWorldCoords::default())
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<LineMaterial>::default(),
            ShapePlugin,
            TextInputPlugin,
        ))
        .register_type::<HandleMap<ImageKey>>()
        .init_resource::<HandleMap<ImageKey>>()
        .add_systems(Startup, (setup_camera, setup_env, setup_textbox))
        
        .add_systems(
            Update,
            (
                //mouse_world_coords,
                fit_canvas,
                //update_server_visuals,
                //command,
                //focus.before(TextInputSystem),
            
            ),
        )   
        .run();
}
