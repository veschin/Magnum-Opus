//! F1 world-foundation / AC4: grid module without world_config fails closed-reads.

use magnum_opus::core::*;
use magnum_opus::grid::GridModule;

#[test]
#[should_panic(expected = "closed-reads")]
fn grid_without_world_config_panics_closed_reads() {
    let _ = Harness::new().with_sim::<GridModule>().build();
}
