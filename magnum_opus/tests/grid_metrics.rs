//! F1 world-foundation / AC3: grid.occupancy_count gauge published under "grid" owner.

use magnum_opus::core::*;
use magnum_opus::grid::GridModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn grid_publishes_occupancy_count_gauge() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();
    app.update();
    app.update();

    let reg = app.world().resource::<MetricsRegistry>();
    assert_eq!(reg.get("grid.occupancy_count"), Some(0.0));
    assert_eq!(reg.owner("grid.occupancy_count"), Some("grid"));
}
