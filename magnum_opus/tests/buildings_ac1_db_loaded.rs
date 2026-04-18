//! F4 AC1 - BuildingDB is populated with all MVP building types on build.

use magnum_opus::buildings::{BuildingDB, BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, Harness};
use magnum_opus::grid::GridModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn building_db_contains_all_mvp_types() {
    let app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    let db = app.world().resource::<BuildingDB>();
    assert_eq!(db.defs.len(), 4, "MVP set should have exactly 4 types");
    assert!(db.defs.contains_key(&BuildingType::Miner));
    assert!(db.defs.contains_key(&BuildingType::Smelter));
    assert!(db.defs.contains_key(&BuildingType::Mall));
    assert!(db.defs.contains_key(&BuildingType::EnergySource));
}
