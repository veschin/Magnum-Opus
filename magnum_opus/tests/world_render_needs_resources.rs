//! F19 world-render / AC5: world_render without ResourcesModule panics closed-reads.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

#[test]
#[should_panic(expected = "closed-reads")]
fn ac5_missing_resources_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_view::<WorldRenderModule>()
        .build();
}
