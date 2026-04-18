//! Static building-definition database.
//!
//! Read-only after `BuildingDbModule::install`. BTreeMap for deterministic
//! iteration order (matches Grid.occupancy rule).

use super::types::{BuildingDef, BuildingType};
use bevy::prelude::Resource;
use std::collections::BTreeMap;

#[derive(Resource, Debug, Default)]
pub struct BuildingDB {
    pub defs: BTreeMap<BuildingType, BuildingDef>,
}
