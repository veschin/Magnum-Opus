//! F2 world-generation / AC6: at least one cluster has >=5 veins within Manhattan-3.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::resources::{ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn ac6_at_least_one_cluster_has_five_veins_within_radius_three() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .build();
    app.update();
    app.update();

    let veins = app.world().resource::<ResourceVeins>();
    let positions: Vec<(u32, u32)> = veins.veins.keys().copied().collect();

    let mut best = 0;
    for &(cx, cy) in &positions {
        let count = positions
            .iter()
            .filter(|&&(x, y)| {
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;
                dx.abs() + dy.abs() <= 3
            })
            .count();
        if count > best {
            best = count;
        }
    }
    assert!(
        best >= 5,
        "expected at least one cluster with >=5 veins in Manhattan-3, got max {best}"
    );
}
