//! F2 world-generation / AC1: Landscape fills 4096 cells after two ticks.

use magnum_opus::core::*;
use magnum_opus::landscape::{Landscape, LandscapeModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn ac1_landscape_generates_full_grid_after_two_ticks() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .build();
    app.update();
    app.update();

    let ls = app.world().resource::<Landscape>();
    assert!(ls.ready);
    assert_eq!(ls.cells.len(), 64 * 64);
}
