//! F19 world-render / AC4: world_render without LandscapeModule panics closed-reads.

use magnum_opus::core::*;
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

#[test]
#[should_panic(expected = "closed-reads")]
fn ac4_missing_landscape_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<ResourcesModule>()
        .with_view::<WorldRenderModule>()
        .build();
}
