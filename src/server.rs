use std::collections::HashMap;

use bevy::{
    asset::Handle, color::palettes::css::{RED, SKY_BLUE}, math::Vec3, prelude::{Component, CubicCurve, Entity, Gizmos, Query, Res, Resource}, reflect::Reflect, render::texture::Image, sprite::{Sprite, SpriteBundle}
};

use crate::assetloader::{HandleMap, ImageKey};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ServerState {
    Healthy,
    Corrupted,
    Infected,
}

#[derive(Component)]
pub struct Server {
    pub state: ServerState,
    pub health: f32,
    pub temp: f32,
    pub name: String
}

pub type WireId = usize;

#[derive(Resource, Default)]
pub struct Connections {
    pub graph: HashMap<Entity, Vec<(usize, Entity)>>,
    pub wires: Vec<Entity>,
}

pub struct Wire {
    pub path: CubicCurve<Vec3>,
    pub kind: WireKind,
}

pub enum WireKind {
    Power,
    Communication,
}

pub fn update_server_visuals(handle_map: Res<HandleMap<ImageKey>>, mut servers: Query<(&mut Handle<Image>, &Server)>,) {
    for (mut texture, server) in &mut servers {
        *texture = handle_map[&ImageKey::Server(server.state)].clone();
    }
}