//! F5a AC1 - RecipeDB is populated with one entry per MVP building type.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, Harness};
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDB, RecipeDbModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn recipe_db_has_entry_per_building_type() {
    let app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
        .with_input::<PlacementInputModule>()
        .build();

    let db = app.world().resource::<RecipeDB>();
    assert_eq!(db.recipes.len(), 4);
    assert!(db.recipes.contains_key(&BuildingType::Miner));
    assert!(db.recipes.contains_key(&BuildingType::Smelter));
    assert!(db.recipes.contains_key(&BuildingType::Mall));
    assert!(db.recipes.contains_key(&BuildingType::EnergySource));
}
