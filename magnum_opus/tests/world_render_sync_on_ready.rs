//! F19 world-render / AC1: full cache populated after Landscape + ResourceVeins are ready.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::resources::{ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::{WorldRenderModule, WorldSceneCache};

#[test]
fn ac1_cache_populated_after_two_ticks() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .with_view::<WorldRenderModule>()
        .build();
    app.update();
    app.update();

    let veins = app.world().resource::<ResourceVeins>();
    let vein_count = veins.veins.len();

    let cache = app.world().resource::<WorldSceneCache>();
    assert!(cache.synced);
    assert_eq!(cache.tiles.len(), 64 * 64);
    assert_eq!(cache.veins.len(), vein_count);
}
