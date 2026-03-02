/// World / terrain systems — map generation, hazard warning, hazard trigger,
/// element interactions, weather, fog of war, world placement.
/// Stub implementations — full behaviour arrives in the impl stage.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{
    BiomeHazard, HazardWarning, HazardKind, WorldTile, TileVisibility,
    TileEnhancement, EnhancementType, ElementalState, FogRevealer,
    SacrificeBuilding, RunePathSegment, Tier,
};
use crate::events::{
    BuildingDestroyed, SacrificeHit, SacrificeMiss, HazardTriggered,
    PlacementRejected,
};
use crate::resources::{
    WorldMap, SimTick, CurrentWeather,
    WorldPlacementCommands, FixedRng, CurrentTierWorld,
};

// ── Map generation ─────────────────────────────────────────────────────────

/// Stub: populates WorldMap tiles based on biome and seed.
pub fn map_generation_system(
    _commands: Commands,
    _world_map: ResMut<WorldMap>,
) {
    // TODO impl: seed-based noise, terrain distribution, hazard zones
}

// ── Tick advance ───────────────────────────────────────────────────────────

/// Increments the global simulation tick counter each frame.
/// Only increments if SimulationPlugin is NOT present (i.e., RunConfig absent).
/// When SimulationPlugin is active, tick_increment_system owns SimTick.
pub fn tick_advance_system(
    run_config: Option<Res<crate::resources::RunConfig>>,
    mut sim_tick: ResMut<SimTick>,
) {
    if run_config.is_none() {
        sim_tick.current += 1;
    }
}

// ── Hazard warning ─────────────────────────────────────────────────────────

/// Issues HazardWarning entities when simulation approaches next_event_tick.
pub fn hazard_warning_system(
    mut commands: Commands,
    sim_tick: Res<SimTick>,
    hazards: Query<(Entity, &BiomeHazard)>,
    existing_warnings: Query<Entity, With<HazardWarning>>,
) {
    for e in existing_warnings.iter() {
        commands.entity(e).despawn();
    }
    for (_, hazard) in hazards.iter() {
        let current = sim_tick.current as u32;
        let ticks_until = hazard.next_event_tick.saturating_sub(current);
        if ticks_until <= hazard.warning_ticks && ticks_until > 0 {
            commands.spawn(HazardWarning {
                hazard_kind: hazard.hazard_kind,
                center_x: hazard.center_x,
                center_y: hazard.center_y,
                ticks_remaining: ticks_until,
            });
        }
    }
}

// ── Hazard trigger ─────────────────────────────────────────────────────────

/// Fires hazard events when sim_tick reaches next_event_tick.
pub fn hazard_trigger_system(
    mut commands: Commands,
    sim_tick: Res<SimTick>,
    mut hazards: Query<&mut BiomeHazard>,
    world_tiles: Query<(Entity, &WorldTile)>,
    buildings: Query<(Entity, &crate::components::Position, &crate::components::Building)>,
    rune_paths: Query<(Entity, &RunePathSegment)>,
    sacrifice_q: Query<(Entity, &crate::components::Position, &SacrificeBuilding)>,
    fixed_rng: Res<FixedRng>,
    tier_res: Res<CurrentTierWorld>,
    mut ev_destroyed: MessageWriter<BuildingDestroyed>,
    mut ev_hit: MessageWriter<SacrificeHit>,
    mut ev_miss: MessageWriter<SacrificeMiss>,
    mut ev_triggered: MessageWriter<HazardTriggered>,
) {
    for mut hazard in hazards.iter_mut() {
        if (sim_tick.current as u32) < hazard.next_event_tick {
            continue;
        }

        let (etype, base_mag, dur, kills_buildings, kills_paths) =
            hazard_effect_params(hazard.hazard_kind);

        let _effective_intensity = match tier_res.tier {
            Tier::T1 => hazard.intensity,
            Tier::T2 => hazard.intensity * 1.3,
            Tier::T3 => hazard.intensity * 1.6,
        };

        ev_triggered.write(HazardTriggered {
            hazard_kind: hazard.hazard_kind,
            center_x: hazard.center_x,
            center_y: hazard.center_y,
            radius: hazard.radius,
            enhancement_type: etype,
            enhancement_magnitude: base_mag,
        });

        let cx = hazard.center_x;
        let cy = hazard.center_y;
        let r = hazard.radius;

        // Apply tile enhancements in zone
        for (tile_entity, tile) in world_tiles.iter() {
            if manhattan(tile.x, tile.y, cx, cy) <= r {
                commands.entity(tile_entity).insert(TileEnhancement {
                    enhancement_type: etype,
                    magnitude: base_mag,
                    remaining_ticks: dur,
                });
            }
        }

        // Handle sacrifice buildings in zone
        for (sac_entity, sac_pos, sac) in sacrifice_q.iter() {
            if manhattan(sac_pos.x, sac_pos.y, cx, cy) > r { continue; }
            if !sac.in_hazard_zone { continue; }
            let roll = fixed_rng.roll.unwrap_or(0.5);
            let chance = sac.success_chance.unwrap_or(0.6);
            if roll < chance {
                commands.entity(sac_entity).insert(TileEnhancement {
                    enhancement_type: etype,
                    magnitude: base_mag * 2.0,
                    remaining_ticks: dur,
                });
                ev_hit.write(SacrificeHit { sacrifice_entity: sac_entity });
            } else {
                ev_miss.write(SacrificeMiss { sacrifice_entity: sac_entity });
                commands.entity(sac_entity).despawn();
                ev_destroyed.write(BuildingDestroyed {
                    entity: sac_entity, x: sac_pos.x, y: sac_pos.y,
                });
            }
        }

        if kills_buildings {
            for (b_entity, b_pos, _) in buildings.iter() {
                if manhattan(b_pos.x, b_pos.y, cx, cy) <= r {
                    if sacrifice_q.get(b_entity).is_ok() { continue; }
                    commands.entity(b_entity).despawn();
                    ev_destroyed.write(BuildingDestroyed {
                        entity: b_entity, x: b_pos.x, y: b_pos.y,
                    });
                }
            }
        }

        if kills_paths {
            for (seg_entity, seg) in rune_paths.iter() {
                if manhattan(seg.x, seg.y, cx, cy) <= r {
                    commands.entity(seg_entity).despawn();
                }
            }
        }

        // Schedule next recurrence (variance applied at impl stage)
        hazard.next_event_tick += hazard.interval_ticks;
    }
}

// ── Element interaction ─────────────────────────────────────────────────────

/// One tick of elemental interactions per tile (fire/water/cold/wind).
pub fn element_interaction_system(
    mut world_tiles: Query<(&WorldTile, &mut ElementalState)>,
    current_weather: Res<CurrentWeather>,
) {
    const FIRE_DECAY: f32 = 0.95;
    const WATER_DECAY: f32 = 0.99;
    const COLD_DECAY: f32 = 0.93;
    const FIRE_THRESHOLD: f32 = 0.3;
    const COLD_THRESHOLD: f32 = 0.4;
    const WATER_THRESHOLD: f32 = 0.2;
    const WIND_THRESHOLD: f32 = 0.1;
    const FREEZE_RATE: f32 = 0.1;
    const EVAPORATE_RATE: f32 = 0.15;
    const AMPLIFY_FACTOR: f32 = 1.1;
    const REDUCE_RATE: f32 = 0.2;
    const SPREAD_FACTOR: f32 = 0.15;
    const SPREAD_AMOUNT: f32 = 0.2;

    // Snapshot for fire spread
    let snapshot: Vec<((i32, i32), f32, f32)> = world_tiles
        .iter()
        .map(|(tile, st)| ((tile.x, tile.y), st.fire, st.wind))
        .collect();

    let mut spread_map: HashMap<(i32, i32), f32> = HashMap::new();
    for ((tx, ty), fire, wind) in &snapshot {
        if *fire > FIRE_THRESHOLD && *wind > 0.0 {
            let chance = fire * wind * SPREAD_FACTOR;
            for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                *spread_map.entry((tx + dx, ty + dy)).or_default() +=
                    chance * SPREAD_AMOUNT;
            }
        }
    }

    for (tile, mut st) in world_tiles.iter_mut() {
        let pos = (tile.x, tile.y);

        // Wind is set by weather each tick
        st.wind = current_weather.wind_effect;

        // Spread fire
        if let Some(&spread) = spread_map.get(&pos) {
            st.fire = (st.fire + spread).min(1.0);
        }

        // Interactions
        if st.fire > FIRE_THRESHOLD && st.water > 0.0 {
            st.water = (st.water - EVAPORATE_RATE).max(0.0);
        }
        if st.cold > COLD_THRESHOLD && st.water > 0.0 {
            st.water = (st.water - FREEZE_RATE).max(0.0);
        }
        if st.wind > WIND_THRESHOLD && st.fire > 0.1 {
            st.fire = (st.fire * AMPLIFY_FACTOR).min(1.0);
        }
        if st.water > WATER_THRESHOLD && st.fire > 0.0 {
            st.fire = (st.fire - REDUCE_RATE).max(0.0);
        }

        // Decay
        st.fire *= FIRE_DECAY;
        st.water *= WATER_DECAY;
        st.cold *= COLD_DECAY;
        // wind: no decay — set by weather
    }
}

/// Applies weather element effects to all tile elemental states.
pub fn weather_tick_system(
    mut world_tiles: Query<&mut ElementalState>,
    weather: Res<CurrentWeather>,
) {
    for mut st in world_tiles.iter_mut() {
        st.fire  = (st.fire  + weather.fire_effect ).clamp(0.0, 1.0);
        st.water = (st.water + weather.water_effect).clamp(0.0, 1.0);
        st.cold  = (st.cold  + weather.cold_effect ).clamp(0.0, 1.0);
    }
}

// ── Fog of war ─────────────────────────────────────────────────────────────

/// Updates tile visibility based on FogRevealer buildings.
pub fn fog_of_war_system(
    mut world_tiles: Query<&mut WorldTile>,
    fog_revealers: Query<(&crate::components::Position, &FogRevealer)>,
    weather: Res<CurrentWeather>,
) {
    let revealers: Vec<(i32, i32, i32)> = fog_revealers
        .iter()
        .map(|(pos, rev)| {
            let r = if weather.fog_penalty > 0.0 {
                ((rev.radius as f32) * (1.0 - weather.fog_penalty)).floor() as i32
            } else {
                rev.radius
            };
            (pos.x, pos.y, r)
        })
        .collect();

    for mut tile in world_tiles.iter_mut() {
        let was_visible = tile.visibility == TileVisibility::Visible;
        let now_visible = revealers
            .iter()
            .any(|(rx, ry, r)| manhattan(tile.x, tile.y, *rx, *ry) <= *r);

        if now_visible {
            tile.visibility = TileVisibility::Visible;
        } else if was_visible {
            tile.visibility = TileVisibility::Revealed;
        }
    }
}

// ── World placement ─────────────────────────────────────────────────────────

/// Handles WorldPlacementCommands: checks terrain and visibility constraints.
pub fn world_placement_system(
    mut commands: Commands,
    mut cmds_res: ResMut<WorldPlacementCommands>,
    world_tiles: Query<&WorldTile>,
    mut ev_placed: MessageWriter<crate::events::BuildingPlaced>,
    mut ev_rejected: MessageWriter<PlacementRejected>,
) {
    for cmd in cmds_res.queue.drain(..).collect::<Vec<_>>() {
        let tile_opt = world_tiles.iter().find(|t| t.x == cmd.x && t.y == cmd.y);
        let Some(tile) = tile_opt else {
            cmds_res.last_rejection = Some("tile_not_found");
            ev_rejected.write(PlacementRejected { x: cmd.x, y: cmd.y, reason: "tile_not_found" });
            continue;
        };

        if tile.visibility == TileVisibility::Hidden {
            cmds_res.last_rejection = Some("tile_hidden");
            ev_rejected.write(PlacementRejected { x: cmd.x, y: cmd.y, reason: "tile_hidden" });
            continue;
        }

        if !tile.terrain.is_buildable() {
            cmds_res.last_rejection = Some("tile_not_buildable");
            ev_rejected.write(PlacementRejected { x: cmd.x, y: cmd.y, reason: "tile_not_buildable" });
            continue;
        }

        if let Some(required) = cmd.required_terrain {
            if tile.terrain != required {
                cmds_res.last_rejection = Some("terrain_mismatch");
                ev_rejected.write(PlacementRejected { x: cmd.x, y: cmd.y, reason: "terrain_mismatch" });
                continue;
            }
        }

        let entity = commands.spawn((
            crate::components::Position { x: cmd.x, y: cmd.y },
            crate::components::Building { building_type: cmd.building_type },
            crate::components::ProductionState { progress: 0.0, active: true, idle_reason: None },
            crate::components::InputBuffer::default(),
            crate::components::OutputBuffer::default(),
        )).id();

        cmds_res.last_rejection = None;
        ev_placed.write(crate::events::BuildingPlaced { entity, x: cmd.x, y: cmd.y });
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub fn manhattan(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs() + (ay - by).abs()
}

fn hazard_effect_params(kind: HazardKind) -> (EnhancementType, f32, u32, bool, bool) {
    // (enhancement_type, magnitude, duration_ticks, kills_buildings, kills_paths)
    match kind {
        HazardKind::Eruption   => (EnhancementType::Enriched,         1.5, 6000, true,  true),
        HazardKind::AshStorm   => (EnhancementType::FertileAsh,       1.2, 4000, false, false),
        HazardKind::Wildfire   => (EnhancementType::CharredFertile,   1.3, 5000, true,  true),
        HazardKind::Storm      => (EnhancementType::Waterlogged,      1.1, 3000, false, true),
        HazardKind::Sandstorm  => (EnhancementType::UncoveredDeposit, 1.4, 4000, true,  false),
        HazardKind::HeatWave   => (EnhancementType::GlassSand,        1.2, 3500, false, false),
        HazardKind::Tsunami    => (EnhancementType::TidalDeposit,     1.6, 5000, true,  true),
    }
}
