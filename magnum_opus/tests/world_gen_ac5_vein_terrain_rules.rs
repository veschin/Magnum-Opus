//! F2 world-generation / AC5: every vein sits on terrain matching its resource rule.

use magnum_opus::core::*;
use magnum_opus::landscape::{Landscape, LandscapeModule, TerrainKind};
use magnum_opus::resources::{ResourceKind, ResourceVeins, ResourcesModule};
use magnum_opus::world_config::WorldConfigModule;

fn terrain_at(landscape: &Landscape, x: u32, y: u32) -> TerrainKind {
    landscape.cells[(y * landscape.width + x) as usize].kind
}

fn has_pit_neighbor(landscape: &Landscape, x: u32, y: u32) -> bool {
    let w = landscape.width as i32;
    let h = landscape.height as i32;
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if dx.abs() + dy.abs() > 2 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            if terrain_at(landscape, nx as u32, ny as u32) == TerrainKind::Pit {
                return true;
            }
        }
    }
    false
}

#[test]
fn ac5_every_vein_matches_terrain_rule() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .build();
    app.update();
    app.update();

    let landscape = app.world().resource::<Landscape>();
    let veins = app.world().resource::<ResourceVeins>();

    for (&(x, y), vein) in veins.veins.iter() {
        let terrain = terrain_at(landscape, x, y);
        match vein.kind {
            ResourceKind::IronOre => assert!(
                matches!(terrain, TerrainKind::Rock | TerrainKind::Mountain),
                "IronOre at ({x},{y}) on {terrain:?}"
            ),
            ResourceKind::CopperOre => assert!(
                matches!(terrain, TerrainKind::Rock | TerrainKind::Sand),
                "CopperOre at ({x},{y}) on {terrain:?}"
            ),
            ResourceKind::Stone => assert!(
                matches!(terrain, TerrainKind::Rock | TerrainKind::Mountain),
                "Stone at ({x},{y}) on {terrain:?}"
            ),
            ResourceKind::Coal => {
                assert_eq!(terrain, TerrainKind::Rock, "Coal at ({x},{y}) on {terrain:?} not Rock");
                assert!(
                    has_pit_neighbor(landscape, x, y),
                    "Coal at ({x},{y}) has no Pit within Manhattan-2"
                );
            }
        }
    }
}
