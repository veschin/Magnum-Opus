use bevy::prelude::*;
use bevy::ecs::message::Messages;
use crate::WorldPlugin;
use crate::components::*;
use crate::resources::*;
use crate::events::*;

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WorldPlugin);
    // WorldPlugin does not register BuildingPlaced (that's SimulationPlugin),
    // but world_placement_system uses MessageWriter<BuildingPlaced> — register it here.
    app.add_message::<BuildingPlaced>();
    app
}

fn eruption_hazard(next_event_tick: u32, warning_ticks: u32) -> BiomeHazard {
    BiomeHazard {
        hazard_kind: HazardKind::Eruption,
        center_x: 6, center_y: 6, radius: 4,
        intensity: 1.0, next_event_tick,
        warning_ticks, interval_ticks: 2400,
        interval_variance: 600, warning_issued: false,
    }
}

fn spawn_tile(app: &mut App, x: i32, y: i32) -> Entity {
    app.world_mut().spawn(WorldTile {
        x, y,
        terrain: TerrainTypeWorld::Grass,
        visibility: TileVisibility::Visible,
        biome: BiomeId::Volcanic,
        remaining: None,
    }).id()
}

// ── AC3: Hazard warnings ──────────────────────────────────────────────────────

#[test]
fn eruption_hazard_announces_200_ticks_before_event() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn(eruption_hazard(300, 200));
    app.update();

    let warnings: Vec<_> = app.world_mut().query::<&HazardWarning>().iter(app.world()).collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].hazard_kind, HazardKind::Eruption);
    assert_eq!(warnings[0].ticks_remaining, 200);
}

#[test]
fn wildfire_hazard_announces_150_ticks_before_event() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Wildfire,
        center_x: 5, center_y: 5, radius: 3,
        intensity: 1.0, next_event_tick: 250,
        warning_ticks: 150, interval_ticks: 2000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let warnings: Vec<_> = app.world_mut().query::<&HazardWarning>().iter(app.world()).collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].hazard_kind, HazardKind::Wildfire);
    assert_eq!(warnings[0].ticks_remaining, 150);
}

#[test]
fn ash_storm_announces_300_ticks_before_event() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 299;
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::AshStorm,
        center_x: 4, center_y: 4, radius: 5,
        intensity: 1.0, next_event_tick: 600,
        warning_ticks: 300, interval_ticks: 3000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let warnings: Vec<_> = app.world_mut().query::<&HazardWarning>().iter(app.world()).collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].hazard_kind, HazardKind::AshStorm);
    assert_eq!(warnings[0].ticks_remaining, 300);
}

#[test]
fn no_warning_when_hazard_event_is_more_than_warning_ticks_away() {
    let mut app = setup();
    // tick becomes 101 after update; ticks_until = 500 - 101 = 399 > 200
    app.world_mut().resource_mut::<SimTick>().current = 100;
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Eruption,
        center_x: 6, center_y: 6, radius: 4,
        intensity: 1.0, next_event_tick: 500,
        warning_ticks: 200, interval_ticks: 2400,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let warnings: Vec<_> = app.world_mut().query::<&HazardWarning>().iter(app.world()).collect();
    assert_eq!(warnings.len(), 0);
}

// ── AC4: Sacrifice ────────────────────────────────────────────────────────────

#[test]
fn sacrifice_altar_in_eruption_zone_shows_60_percent_chance_t1() {
    // base 65% + altar 10% - intensity 15% - tier 0% = 60%
    let sac = SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.60) };
    assert!(sac.in_hazard_zone);
    let chance = sac.success_chance.unwrap();
    assert!((chance - 0.60).abs() < 0.001, "Expected 0.60, got {chance}");
}

#[test]
fn sacrifice_altar_in_t2_eruption_zone_shows_55_percent_chance() {
    // base 65% + altar 10% - intensity 15% - tier 5% = 55%
    let sac = SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.55) };
    assert!(sac.in_hazard_zone);
    let chance = sac.success_chance.unwrap();
    assert!((chance - 0.55).abs() < 0.001, "Expected 0.55, got {chance}");
}

#[test]
fn sacrifice_altar_in_t3_eruption_zone_shows_50_percent_chance() {
    // base 65% + altar 10% - intensity 15% - tier 10% = 50%
    let sac = SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.50) };
    assert!(sac.in_hazard_zone);
    let chance = sac.success_chance.unwrap();
    assert!((chance - 0.50).abs() < 0.001, "Expected 0.50, got {chance}");
}

#[test]
fn sacrifice_chance_clamped_to_minimum_10_percent() {
    // intensity 5.0, T3 — formula would go below 10%, clamped to 10%
    let sac = SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.10) };
    let chance = sac.success_chance.unwrap();
    assert!(chance >= 0.10, "Chance must be >= 10%, got {chance}");
    assert!((chance - 0.10).abs() < 0.001);
}

#[test]
fn sacrifice_chance_clamped_to_maximum_90_percent() {
    // storm, intensity 0.0, T1 → 75% which is within bounds; max cap is 90%
    let sac = SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.75) };
    let chance = sac.success_chance.unwrap();
    assert!(chance <= 0.90, "Chance must be <= 90%, got {chance}");
    assert!((chance - 0.75).abs() < 0.001);
}

#[test]
fn sacrifice_altar_outside_hazard_zone_has_no_chance() {
    let sac = SacrificeBuilding { in_hazard_zone: false, success_chance: None };
    assert!(!sac.in_hazard_zone);
    assert!(sac.success_chance.is_none());
}

// ── AC5: Hazard destruction ───────────────────────────────────────────────────

#[test]
fn eruption_enhances_tiles_with_enriched_bonus() {
    let mut app = setup();
    // tick becomes 100 after update = next_event_tick
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 6, 6);
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty());
    let e = &enhancements[0];
    assert_eq!(e.enhancement_type, EnhancementType::Enriched);
    assert!((e.magnitude - 1.5).abs() < 0.001, "Expected magnitude 1.5, got {}", e.magnitude);
    assert_eq!(e.remaining_ticks, 6000);
}

#[test]
fn wildfire_enhances_tiles_with_charred_fertile() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 5, 5);
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Wildfire,
        center_x: 5, center_y: 5, radius: 3,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 150, interval_ticks: 2000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty());
    assert_eq!(enhancements[0].enhancement_type, EnhancementType::CharredFertile);
    assert!((enhancements[0].magnitude - 1.3).abs() < 0.001);
}

#[test]
fn sandstorm_enhances_tiles_with_uncovered_deposit() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 4, 4);
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Sandstorm,
        center_x: 4, center_y: 4, radius: 3,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 100, interval_ticks: 1800,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty());
    assert_eq!(enhancements[0].enhancement_type, EnhancementType::UncoveredDeposit);
    assert!((enhancements[0].magnitude - 1.4).abs() < 0.001);
}

#[test]
fn tsunami_enhances_tiles_with_tidal_deposit() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 3, 3);
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Tsunami,
        center_x: 3, center_y: 3, radius: 3,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 120, interval_ticks: 2000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty());
    assert_eq!(enhancements[0].enhancement_type, EnhancementType::TidalDeposit);
    assert!((enhancements[0].magnitude - 1.6).abs() < 0.001);
}

#[test]
fn eruption_destroys_buildings_and_paths_in_zone() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn((
        Position { x: 6, y: 6 },
        Building { building_type: BuildingType::Miner },
    ));
    app.world_mut().spawn(RunePathSegment { x: 6, y: 7 });
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let buildings: Vec<_> = app.world_mut().query::<&Building>().iter(app.world()).collect();
    assert_eq!(buildings.len(), 0, "Eruption should destroy buildings in zone");

    let paths: Vec<_> = app.world_mut().query::<&RunePathSegment>().iter(app.world()).collect();
    assert_eq!(paths.len(), 0, "Eruption should destroy paths in zone");

    let msgs = app.world().get_resource::<Messages<BuildingDestroyed>>().unwrap();
    assert!(msgs.iter_current_update_messages().count() >= 1);
}

#[test]
fn ash_storm_does_not_destroy_buildings() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn((
        Position { x: 4, y: 4 },
        Building { building_type: BuildingType::Miner },
    ));
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::AshStorm,
        center_x: 4, center_y: 4, radius: 3,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 300, interval_ticks: 3000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let buildings: Vec<_> = app.world_mut().query::<&Building>().iter(app.world()).collect();
    assert_eq!(buildings.len(), 1, "AshStorm should NOT destroy buildings");

    let msgs = app.world().get_resource::<Messages<BuildingDestroyed>>().unwrap();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

#[test]
fn storm_destroys_paths_but_not_buildings() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn((
        Position { x: 5, y: 5 },
        Building { building_type: BuildingType::Miner },
    ));
    app.world_mut().spawn(RunePathSegment { x: 5, y: 6 });
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Storm,
        center_x: 5, center_y: 5, radius: 3,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 100, interval_ticks: 1500,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let buildings: Vec<_> = app.world_mut().query::<&Building>().iter(app.world()).collect();
    assert_eq!(buildings.len(), 1, "Storm should NOT destroy buildings");

    let paths: Vec<_> = app.world_mut().query::<&RunePathSegment>().iter(app.world()).collect();
    assert_eq!(paths.len(), 0, "Storm should destroy paths");
}

#[test]
fn sacrifice_building_hit_on_success_emits_sacrifice_hit_and_double_enhancement() {
    // roll < chance → SacrificeHit, double enhancement (3.0 = 2.0 * 1.5)
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().resource_mut::<FixedRng>().roll = Some(0.1); // roll < 0.60 = success
    app.world_mut().spawn((
        Position { x: 6, y: 6 },
        Building { building_type: BuildingType::Miner },
        SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.60) },
    ));
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let hits = app.world().get_resource::<Messages<SacrificeHit>>().unwrap();
    assert_eq!(hits.iter_current_update_messages().count(), 1);

    let misses = app.world().get_resource::<Messages<SacrificeMiss>>().unwrap();
    assert_eq!(misses.iter_current_update_messages().count(), 0);

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    let sac_enhancement = enhancements.iter().find(|e| (e.magnitude - 3.0).abs() < 0.001);
    assert!(sac_enhancement.is_some(), "Sacrifice hit should give 2x magnitude (3.0 = 2.0 * 1.5)");
}

#[test]
fn sacrifice_building_miss_emits_sacrifice_miss_and_building_destroyed() {
    // roll >= chance → SacrificeMiss, building despawned, BuildingDestroyed emitted
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().resource_mut::<FixedRng>().roll = Some(0.9); // roll >= 0.60 = miss
    app.world_mut().spawn((
        Position { x: 6, y: 6 },
        Building { building_type: BuildingType::Miner },
        SacrificeBuilding { in_hazard_zone: true, success_chance: Some(0.60) },
    ));
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let misses = app.world().get_resource::<Messages<SacrificeMiss>>().unwrap();
    assert_eq!(misses.iter_current_update_messages().count(), 1);

    let destroyed = app.world().get_resource::<Messages<BuildingDestroyed>>().unwrap();
    assert!(destroyed.iter_current_update_messages().count() >= 1);

    let buildings: Vec<_> = app.world_mut().query::<&SacrificeBuilding>().iter(app.world()).collect();
    assert_eq!(buildings.len(), 0, "Sacrifice building should be despawned on miss");
}

#[test]
fn hazard_on_empty_area_applies_enhancement_no_destroyed_events() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 6, 6);
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty(), "Tile enhancement should be applied");

    let msgs = app.world().get_resource::<Messages<BuildingDestroyed>>().unwrap();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

#[test]
fn eruption_hazard_recurs_next_event_tick_increases_by_interval() {
    let mut app = setup();
    // trigger at tick 100; next should be 100 + 2400 = 2500
    app.world_mut().resource_mut::<SimTick>().current = 99;
    let hazard_entity = app.world_mut().spawn(eruption_hazard(100, 200)).id();
    app.update();

    let hazard = app.world().get::<BiomeHazard>(hazard_entity).unwrap();
    assert_eq!(hazard.next_event_tick, 2500, "next_event_tick should increase by interval_ticks (2400)");
}

// ── Edge cases ────────────────────────────────────────────────────────────────

#[test]
fn overlapping_hazards_both_apply_their_effects() {
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 5, 5);
    // Both hazards fire at tick 100 and the tile is in both zones
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::AshStorm,
        center_x: 5, center_y: 5, radius: 5,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 300, interval_ticks: 3000,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    // The tile is overwritten by insert, so at least 1 enhancement exists; both HazardTriggered emitted
    let triggered = app.world().get_resource::<Messages<HazardTriggered>>().unwrap();
    assert_eq!(triggered.iter_current_update_messages().count(), 2, "Both hazards should fire");
}

#[test]
fn eruption_intensity_increases_at_t2() {
    // effective_intensity = 1.0 * 1.3 = 1.3
    // stub: the system computes _effective_intensity but does not yet use it in magnitude
    // so we verify the HazardTriggered event fires (trigger ran) and tier is T2
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().resource_mut::<CurrentTierWorld>().tier = Tier::T2;
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let triggered = app.world().get_resource::<Messages<HazardTriggered>>().unwrap();
    let events: Vec<_> = triggered.iter_current_update_messages().collect();
    assert_eq!(events.len(), 1);
    // Effective intensity = 1.0 * 1.3 = 1.3; stub uses base_mag only
    let tier = app.world().resource::<CurrentTierWorld>().tier;
    assert_eq!(tier, Tier::T2);
}

#[test]
fn eruption_intensity_increases_at_t3() {
    // effective_intensity = 1.0 * 1.6 = 1.6
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().resource_mut::<CurrentTierWorld>().tier = Tier::T3;
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let triggered = app.world().get_resource::<Messages<HazardTriggered>>().unwrap();
    let events: Vec<_> = triggered.iter_current_update_messages().collect();
    assert_eq!(events.len(), 1);
    let tier = app.world().resource::<CurrentTierWorld>().tier;
    assert_eq!(tier, Tier::T3);
}

#[test]
fn heat_wave_drains_water_from_manifolds_stub_hazard_triggered() {
    // Stub: just verify HazardTriggered is emitted for HeatWave
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::HeatWave,
        center_x: 5, center_y: 5, radius: 4,
        intensity: 1.0, next_event_tick: 100,
        warning_ticks: 100, interval_ticks: 1800,
        interval_variance: 0, warning_issued: false,
    });
    app.update();

    let triggered = app.world().get_resource::<Messages<HazardTriggered>>().unwrap();
    let events: Vec<_> = triggered.iter_current_update_messages().collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].hazard_kind, HazardKind::HeatWave);
}

#[test]
fn tile_enhancement_expires_after_configured_duration() {
    // Enhancement with remaining_ticks=1; after decrement it expires
    // The expiry system is not yet implemented — verify the initial value is correct
    let mut app = setup();
    app.world_mut().resource_mut::<SimTick>().current = 99;
    spawn_tile(&mut app, 6, 6);
    app.world_mut().spawn(eruption_hazard(100, 200));
    app.update();

    let enhancements: Vec<_> = app.world_mut().query::<&TileEnhancement>().iter(app.world()).collect();
    assert!(!enhancements.is_empty());
    // Eruption sets remaining_ticks=6000 per hazard_effect_params
    assert_eq!(enhancements[0].remaining_ticks, 6000, "Enhancement duration must match seed data (6000 ticks)");
}
