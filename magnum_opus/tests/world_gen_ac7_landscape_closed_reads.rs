//! F2 world-generation / AC7: LandscapeModule alone panics closed-reads.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;

#[test]
#[should_panic(expected = "closed-reads")]
fn ac7_landscape_without_world_config_panics() {
    let _ = Harness::new().with_sim::<LandscapeModule>().build();
}
