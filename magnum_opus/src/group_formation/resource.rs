//! Group index Resource - map of every Building entity to its current group.

use bevy::prelude::{Entity, Resource};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Resource, Debug, Default)]
pub struct GroupIndex {
    pub groups: BTreeSet<Entity>,
    pub member_to_group: BTreeMap<Entity, Entity>,
}
