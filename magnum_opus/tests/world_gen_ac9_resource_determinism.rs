//! F2 world-generation / AC9: Resources are deterministic across two runs.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::resources::{ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfigModule;
use std::collections::BTreeMap;

fn gen_veins() -> BTreeMap<(u32, u32), magnum_opus::resources::Vein> {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .build();
    app.update();
    app.update();
    app.world().resource::<ResourceVeins>().veins.clone()
}

#[test]
fn ac9_resource_veins_are_deterministic() {
    let a = gen_veins();
    let b = gen_veins();
    assert_eq!(a, b);
}
