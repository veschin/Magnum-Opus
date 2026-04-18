//! Cache mapping each live Building entity to its render-layer sprite entity.

use bevy::prelude::{Entity, Resource};
use std::collections::BTreeMap;

#[derive(Resource, Debug, Default)]
pub struct BuildingSceneCache {
    pub entities: BTreeMap<Entity, Entity>,
}
