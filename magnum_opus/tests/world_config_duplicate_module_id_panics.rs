//! F1 world-foundation / edge case: duplicate module id panics at registration.

use magnum_opus::core::*;
use magnum_opus::world_config::WorldConfigModule;

#[test]
#[should_panic(expected = "duplicate module id")]
fn world_config_registered_twice_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<WorldConfigModule>()
        .build();
}
