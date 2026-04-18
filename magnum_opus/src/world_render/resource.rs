use bevy::prelude::{Entity, Resource};
use std::collections::BTreeMap;

#[derive(Resource, Default, Debug)]
pub struct WorldSceneCache {
    pub tiles: BTreeMap<(u32, u32), Entity>,
    pub veins: BTreeMap<(u32, u32), Entity>,
    pub synced: bool,
}
