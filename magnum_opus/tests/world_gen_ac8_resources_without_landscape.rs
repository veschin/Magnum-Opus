//! F2 world-generation / AC8: ResourcesModule without LandscapeModule - both substrings in panic.
//!
//! `#[should_panic(expected = "...")]` matches only one substring. We catch
//! the panic and verify BOTH `"closed-messages"` and `"closed-reads"` are present.

use magnum_opus::core::*;
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn ac8_resources_without_landscape_panics_both_substrings() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = Harness::new()
            .with_data::<WorldConfigModule>()
            .with_sim::<ResourcesModule>()
            .build();
    }));

    let err = result.expect_err("expected panic, got Ok");
    let msg = err
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| err.downcast_ref::<&str>().copied())
        .expect("panic payload not a string");

    assert!(msg.contains("closed-messages"), "missing closed-messages in: {msg}");
    assert!(msg.contains("closed-reads"), "missing closed-reads in: {msg}");
}
