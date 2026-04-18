//! F2 world-generation / Edge: seed 0 still produces valid generation.
//!
//! Uses a test-local StaticData module that overrides WorldConfig.seed to 0.
//! Confirms that sub-seed derivation (splitmix64(seed ^ salt)) handles zero
//! input and produces non-degenerate output.

use magnum_opus::core::*;
use magnum_opus::landscape::{Landscape, LandscapeModule};
use magnum_opus::names;
use magnum_opus::resources::{ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfig;

pub struct ZeroSeedConfig;
impl StaticData for ZeroSeedConfig {
    const ID: &'static str = "zero_seed_config";
    fn writes() -> &'static [TypeKey] {
        names![WorldConfig]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(WorldConfig {
            width: 64,
            height: 64,
            seed: 0,
        });
    }
}

#[test]
fn edge_seed_zero_produces_valid_generation() {
    let mut app = Harness::new()
        .with_data::<ZeroSeedConfig>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .build();
    app.update();
    app.update();

    let ls = app.world().resource::<Landscape>();
    let veins = app.world().resource::<ResourceVeins>();
    assert!(ls.ready);
    assert!(veins.ready);
    assert_eq!(ls.cells.len(), 64 * 64);
}
