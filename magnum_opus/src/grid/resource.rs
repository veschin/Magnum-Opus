use bevy::prelude::{Entity, Resource};
use std::collections::BTreeMap;

#[derive(Resource, Default, Debug)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub occupancy: BTreeMap<(u32, u32), Entity>,
    pub dims_set: bool,
}
