//! F4 AC5 - second StaticData claiming writes: names![BuildingDB] panics.

use magnum_opus::buildings::{BuildingDB, BuildingDbModule};
use magnum_opus::core::*;
use magnum_opus::grid::GridModule;
use magnum_opus::names;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

struct RogueBuildingDbWriter;
impl StaticData for RogueBuildingDbWriter {
    const ID: &'static str = "rogue_building_db_writer";
    fn writes() -> &'static [TypeKey] {
        names![BuildingDB]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(BuildingDB::default());
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_writer_of_building_db_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RogueBuildingDbWriter>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();
}
