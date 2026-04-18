//! F2 world-generation / AC3: Landscape has at least 4 distinct TerrainKinds + gauge.

use magnum_opus::core::*;
use magnum_opus::landscape::{Landscape, LandscapeModule};
use magnum_opus::world_config::WorldConfigModule;
use std::collections::HashSet;

#[test]
fn ac3_landscape_shows_variety_of_terrain_kinds() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .build();
    app.update();
    app.update();

    let ls = app.world().resource::<Landscape>();
    let distinct: HashSet<_> = ls.cells.iter().map(|c| c.kind).collect();
    assert!(
        distinct.len() >= 4,
        "expected >=4 distinct TerrainKinds, got {}",
        distinct.len()
    );

    let reg = app.world().resource::<MetricsRegistry>();
    let gauge = reg.get("landscape.kinds_present").expect("gauge missing");
    assert_eq!(gauge as usize, distinct.len());
}
