use bevy::prelude::*;

use crate::components::*;
use crate::events::BuildingPlaced;
use crate::resources::{FogMap, Grid, Inventory, TierState};

/// A single placement request.
#[derive(Clone)]
pub struct PlacementRequest {
    pub building_type: BuildingType,
    pub x: i32,
    pub y: i32,
    pub recipe: Recipe,
    /// If true, ignore inventory check (used by legacy tests and starting-kit placements).
    pub skip_inventory_check: bool,
    /// If true, ignore fog check.
    pub skip_fog_check: bool,
}

impl PlacementRequest {
    /// Convenience: legacy-compatible constructor, no extra checks.
    pub fn legacy(building_type: BuildingType, x: i32, y: i32, recipe: Recipe) -> Self {
        Self {
            building_type,
            x,
            y,
            recipe,
            skip_inventory_check: true,
            skip_fog_check: true,
        }
    }

    /// Standard placement with all checks enabled.
    pub fn new(building_type: BuildingType, x: i32, y: i32, recipe: Recipe) -> Self {
        Self {
            building_type,
            x,
            y,
            recipe,
            skip_inventory_check: false,
            skip_fog_check: false,
        }
    }
}

/// Command resource: queue of buildings to place this tick.
#[derive(Resource, Default)]
pub struct PlacementCommands {
    /// Legacy queue (existing tests) — Vec of (BuildingType, x, y, Recipe).
    pub queue: Vec<(BuildingType, i32, i32, Recipe)>,
    /// New queue supporting full validation.
    pub requests: Vec<PlacementRequest>,
    /// Track last-tick placement results (true = success).
    pub last_results: Vec<bool>,
}

pub fn placement_system(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut placement: ResMut<PlacementCommands>,
    mut ev_placed: MessageWriter<BuildingPlaced>,
    tier: Option<Res<TierState>>,
    fog: Option<Res<FogMap>>,
    mut inventory: Option<ResMut<Inventory>>,
) {
    placement.last_results.clear();

    // ── Legacy queue (no extra validation) ──────────────────────────────────
    let legacy: Vec<_> = placement.queue.drain(..).collect();
    for (building_type, x, y, recipe) in legacy {
        if grid.occupied.contains(&(x, y)) {
            continue;
        }
        if !grid.in_bounds(x, y) {
            continue;
        }

        let (fw, fh) = building_type.footprint();
        let cells = footprint_cells(x, y, fw, fh);
        if cells.iter().any(|c| grid.occupied.contains(c) || !grid.in_bounds(c.0, c.1)) {
            continue;
        }

        let entity = commands
            .spawn((
                Position { x, y },
                Building { building_type },
                recipe,
                ProductionState::default(),
                InputBuffer::default(),
                OutputBuffer::default(),
                Footprint::rect(x, y, fw, fh),
            ))
            .id();

        for c in &cells {
            grid.occupied.insert(*c);
        }
        ev_placed.write(BuildingPlaced { entity, x, y });
    }

    // ── Request queue (full validation) ─────────────────────────────────────
    let requests: Vec<_> = placement.requests.drain(..).collect();
    for req in requests {
        let mut ok = true;

        // Bounds check
        if !grid.in_bounds(req.x, req.y) {
            placement.last_results.push(false);
            continue;
        }

        let (fw, fh) = req.building_type.footprint();
        let cells = footprint_cells(req.x, req.y, fw, fh);

        // All footprint cells must be in bounds and unoccupied
        for c in &cells {
            if !grid.in_bounds(c.0, c.1) || grid.occupied.contains(c) {
                ok = false;
                break;
            }
        }

        // Terrain check
        if ok {
            if let Some(required) = req.building_type.terrain_req() {
                let actual = grid.terrain_at(req.x, req.y);
                if actual != required {
                    ok = false;
                }
            }
        }

        // Tier check
        if ok {
            let building_tier = req.building_type.tier() as u8;
            let current_tier = tier.as_ref().map(|t| t.current_tier).unwrap_or(1);
            if building_tier > current_tier {
                ok = false;
            }
        }

        // Inventory check
        if ok && !req.skip_inventory_check {
            if let Some(ref mut inv) = inventory {
                if inv.count_building(req.building_type) == 0 {
                    ok = false;
                }
            }
        }

        // Fog check
        if ok && !req.skip_fog_check {
            if let Some(ref f) = fog {
                if !f.is_visible(req.x, req.y) {
                    ok = false;
                }
            }
        }

        if ok {
            // Consume from inventory if check active
            if !req.skip_inventory_check {
                if let Some(ref mut inv) = inventory {
                    inv.consume_building(req.building_type);
                }
            }

            let entity = commands
                .spawn((
                    Position { x: req.x, y: req.y },
                    Building { building_type: req.building_type },
                    req.recipe,
                    ProductionState::default(),
                    InputBuffer::default(),
                    OutputBuffer::default(),
                    Footprint::rect(req.x, req.y, fw, fh),
                ))
                .id();

            for c in &cells {
                grid.occupied.insert(*c);
            }
            ev_placed.write(BuildingPlaced { entity, x: req.x, y: req.y });
        }

        placement.last_results.push(ok);
    }
}

fn footprint_cells(ox: i32, oy: i32, w: i32, h: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::with_capacity((w * h) as usize);
    for dy in 0..h {
        for dx in 0..w {
            v.push((ox + dx, oy + dy));
        }
    }
    v
}
