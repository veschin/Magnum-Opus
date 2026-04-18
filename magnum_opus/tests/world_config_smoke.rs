//! F1 world-foundation / AC1: WorldConfig holds exact constants after build.

use magnum_opus::core::*;
use magnum_opus::world_config::{WorldConfig, WorldConfigModule};

#[test]
fn world_config_inserted_with_expected_constants() {
    let app = Harness::new().with_data::<WorldConfigModule>().build();
    let cfg = app.world().resource::<WorldConfig>();
    assert_eq!(cfg.width, 64);
    assert_eq!(cfg.height, 64);
    assert_eq!(cfg.seed, 0x9E37_79B9_7F4A_7C15);
}
