
use bevy::{
    asset::Handle, prelude::{Component, Entity, Query, Res}, reflect::Reflect, render::texture::Image
};

use crate::assetloader::{HandleMap, ImageKey};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ServerState {
    Healthy,
    Hacked,
    Infected,
    Broken,
    Off
}

pub enum ServerProgram {
    Heat,
    Reboot(usize),
    Cycle,
    Hack(Entity),
    Wait
}

#[derive(Component)]
pub struct Server {
    pub state: ServerState,
    pub program: ServerProgram,
    pub temp: f32,
    pub overheat: f32,
    pub name: String,
}

#[derive(Component)]
pub struct Generator {
    pub load: f32,
    pub overload: f32,
    pub heat: f32,
    pub overheat: f32
}



pub fn update_server_visuals(handle_map: Res<HandleMap<ImageKey>>, mut servers: Query<(&mut Handle<Image>, &Server)>,) {
    for (mut texture, server) in &mut servers {
        *texture = handle_map[&ImageKey::Server(server.state)].clone();
    }
}