//! F6 AC5 - empty grid never panics and never creates phantom manifolds.

use magnum_opus::buildings::BuildingDbModule;
use magnum_opus::core::{AppExt, Harness};
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex};
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn empty_grid_never_panics() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
        .with_sim::<ManifoldModule>()
        .with_input::<PlacementInputModule>()
        .build();

    for _ in 0..20 {
        app.update();
    }

    let index = app.world().resource::<GroupIndex>();
    assert!(index.groups.is_empty());
}
