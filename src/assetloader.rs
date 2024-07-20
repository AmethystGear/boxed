use crate::server::ServerState;
use bevy::{prelude::*, utils::HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ImageKey {
    Server(ServerState),
    Power,
    Tile
}

impl AssetKey for ImageKey {
    type Asset = Image;
}

impl FromWorld for HandleMap<ImageKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (ImageKey::Server(ServerState::Healthy), asset_server.load("images/server/healthy.png")),
            (ImageKey::Server(ServerState::Corrupted), asset_server.load("images/server/corrupted.png")),
            (ImageKey::Server(ServerState::Infected), asset_server.load("images/server/infected.png")),
            (ImageKey::Power, asset_server.load("images/power.png")),
            (ImageKey::Tile, asset_server.load("images/tile.png"))
        ]
        .into()
    }
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);


impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}