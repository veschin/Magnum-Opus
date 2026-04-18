//! BuildingDB StaticData module.
//!
//! Populates BuildingDB with the MVP building set at install time. No
//! systems; the database is read-only at runtime.

use super::resource::BuildingDB;
use super::types::{BuildingDef, BuildingType};
use crate::core::*;
use crate::names;
use std::collections::BTreeMap;

pub struct BuildingDbModule;

impl StaticData for BuildingDbModule {
    const ID: &'static str = "building_db";

    fn writes() -> &'static [TypeKey] {
        names![BuildingDB]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut DataInstaller) {
        let mut defs = BTreeMap::new();
        defs.insert(BuildingType::Miner, BuildingDef { name: "Miner" });
        defs.insert(BuildingType::Smelter, BuildingDef { name: "Smelter" });
        defs.insert(BuildingType::Mall, BuildingDef { name: "Mall" });
        defs.insert(
            BuildingType::EnergySource,
            BuildingDef {
                name: "EnergySource",
            },
        );
        ctx.insert_resource(BuildingDB { defs });
    }
}
