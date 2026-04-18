//! Deterministic cluster placement for resource veins.
//! 16 candidate centers keyed to the seed, each expanded within Manhattan-3 with
//! a density falloff. Terrain rules decide which kind fits where.

use super::resource::{Quality, ResourceKind, Vein};
use crate::landscape::{Landscape, TerrainKind};
use std::collections::BTreeMap;

const CLUSTER_COUNT: u32 = 16;
const CLUSTER_RADIUS: i32 = 3;

fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hash3(seed: u64, a: u32, b: u32) -> u64 {
    splitmix64(seed ^ ((a as u64) << 32) ^ (b as u64))
}

fn terrain_at(landscape: &Landscape, x: u32, y: u32) -> TerrainKind {
    landscape.cells[(y * landscape.width + x) as usize].kind
}

fn has_pit_within_manhattan_2(landscape: &Landscape, cx: u32, cy: u32) -> bool {
    let w = landscape.width as i32;
    let h = landscape.height as i32;
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if dx.abs() + dy.abs() > 2 {
                continue;
            }
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
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

fn kind_for_terrain(
    landscape: &Landscape,
    x: u32,
    y: u32,
    terrain: TerrainKind,
    roll: u64,
) -> Option<ResourceKind> {
    let candidates: Vec<ResourceKind> = match terrain {
        TerrainKind::Rock => {
            let mut v = vec![ResourceKind::IronOre, ResourceKind::Stone];
            if has_pit_within_manhattan_2(landscape, x, y) {
                v.push(ResourceKind::Coal);
            }
            v.push(ResourceKind::CopperOre);
            v
        }
        TerrainKind::Mountain => vec![ResourceKind::Stone, ResourceKind::IronOre],
        TerrainKind::Sand => vec![ResourceKind::CopperOre],
        _ => return None,
    };
    let idx = (roll as usize) % candidates.len();
    Some(candidates[idx])
}

fn terrain_admits(
    landscape: &Landscape,
    x: u32,
    y: u32,
    kind: ResourceKind,
    terrain: TerrainKind,
) -> bool {
    match kind {
        ResourceKind::IronOre => matches!(terrain, TerrainKind::Rock | TerrainKind::Mountain),
        ResourceKind::CopperOre => matches!(terrain, TerrainKind::Rock | TerrainKind::Sand),
        ResourceKind::Stone => matches!(terrain, TerrainKind::Rock | TerrainKind::Mountain),
        ResourceKind::Coal => {
            terrain == TerrainKind::Rock && has_pit_within_manhattan_2(landscape, x, y)
        }
    }
}

pub fn generate_veins(
    seed: u64,
    landscape: &Landscape,
) -> (BTreeMap<(u32, u32), Vein>, u32) {
    let cluster_seed = splitmix64(seed ^ 0x0000_0000_00C0_FFEE);
    let mut veins: BTreeMap<(u32, u32), Vein> = BTreeMap::new();
    let mut clusters_placed: u32 = 0;

    let eligible: Vec<(u32, u32)> = (0..landscape.height)
        .flat_map(|y| (0..landscape.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            matches!(
                terrain_at(landscape, x, y),
                TerrainKind::Rock | TerrainKind::Mountain | TerrainKind::Sand
            )
        })
        .collect();

    if eligible.is_empty() {
        return (veins, 0);
    }

    for i in 0..CLUSTER_COUNT {
        let pick = (hash3(cluster_seed, i, 0) as usize) % eligible.len();
        let (cx, cy) = eligible[pick];
        let terrain = terrain_at(landscape, cx, cy);
        let kind_roll = hash3(cluster_seed, i, 2);
        let kind = match kind_for_terrain(landscape, cx, cy, terrain, kind_roll) {
            Some(k) => k,
            None => continue,
        };

        let mut placed_any = false;
        for dy in -CLUSTER_RADIUS..=CLUSTER_RADIUS {
            for dx in -CLUSTER_RADIUS..=CLUSTER_RADIUS {
                let dist = dx.abs() + dy.abs();
                if dist > CLUSTER_RADIUS {
                    continue;
                }
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= landscape.width as i32 || ny >= landscape.height as i32
                {
                    continue;
                }
                let density =
                    1.0 - (dist as f32 / CLUSTER_RADIUS as f32);
                let tile_terrain = terrain_at(landscape, nx as u32, ny as u32);
                if !terrain_admits(landscape, nx as u32, ny as u32, kind, tile_terrain) {
                    continue;
                }
                let roll = hash3(cluster_seed, i * 128 + (dy + 3) as u32 * 16 + (dx + 3) as u32, 42);
                let roll_frac = (roll >> 40) as f32 / (1u64 << 24) as f32;
                if roll_frac > density {
                    continue;
                }
                let qroll = hash3(cluster_seed, i * 128 + (dy + 3) as u32 * 16 + (dx + 3) as u32, 43);
                let quality = if (qroll % 100) < 20 {
                    Quality::High
                } else {
                    Quality::Normal
                };
                let amount_roll = hash3(cluster_seed, i, 99);
                let remaining = 500.0 + (amount_roll % 1000) as f32;

                veins.insert(
                    (nx as u32, ny as u32),
                    Vein {
                        kind,
                        quality,
                        remaining,
                    },
                );
                placed_any = true;
            }
        }
        if placed_any {
            clusters_placed += 1;
        }
    }

    (veins, clusters_placed)
}
