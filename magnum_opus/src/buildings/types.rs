//! Building type enum and static definition.
//!
//! MVP set. Closed enum: adding a variant requires a matching entry in
//! `BuildingDbModule::install` so `BuildingDB` stays in sync. The F5 recipe
//! feature will extend `BuildingDef` with inputs/outputs/duration fields.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuildingType {
    Miner,
    Smelter,
    Mall,
    EnergySource,
}

#[derive(Debug, Clone, Copy)]
pub struct BuildingDef {
    pub name: &'static str,
}
