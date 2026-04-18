//! F2 world-generation / AC2: Two runs with identical WorldConfig produce identical cells.

use magnum_opus::core::*;
use magnum_opus::landscape::{Landscape, LandscapeModule};
use magnum_opus::world_config::WorldConfigModule;

fn gen_cells() -> Vec<magnum_opus::landscape::TerrainCell> {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .build();
    app.update();
    app.update();
    app.world().resource::<Landscape>().cells.clone()
}

#[test]
fn ac2_landscape_is_deterministic_across_two_runs() {
    let a = gen_cells();
    let b = gen_cells();
    assert_eq!(a, b);
}
