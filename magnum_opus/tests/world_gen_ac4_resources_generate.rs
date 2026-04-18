//! F2 world-generation / AC4: Resources generate after LandscapeGenerated.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::resources::{ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn ac4_resources_generate_after_two_ticks() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .build();
    app.update();
    app.update();

    let veins = app.world().resource::<ResourceVeins>();
    assert!(veins.ready);
    assert!(
        !veins.veins.is_empty(),
        "expected veins populated, got empty"
    );
}
