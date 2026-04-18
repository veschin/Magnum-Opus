//! F5a AC7 - second StaticData claiming writes: names![RecipeDB] panics.

use magnum_opus::buildings::BuildingDbModule;
use magnum_opus::core::*;
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::names;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDB, RecipeDbModule};
use magnum_opus::world_config::WorldConfigModule;

struct RogueRecipeDbWriter;
impl StaticData for RogueRecipeDbWriter {
    const ID: &'static str = "rogue_recipe_db_writer";
    fn writes() -> &'static [TypeKey] {
        names![RecipeDB]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(RecipeDB::default());
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_writer_of_recipe_db_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_data::<RogueRecipeDbWriter>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
        .with_input::<PlacementInputModule>()
        .build();
}
