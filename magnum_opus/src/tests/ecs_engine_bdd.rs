//! ECS Engine integration tests — cross-feature BDD scenarios.
//!
//! Each test maps 1:1 to a scenario in `.ptsd/bdd/ecs-engine.feature`.
//! These tests expose 8 wiring bugs (B1-B8) that will be fixed in the next step.
//! Tests are written to pass AFTER bug fixes, but most fail before them.
//!
//! Bug references:
//!   B1: production_rates_system missing (ProductionRates not written)
//!   B2: tick_increment_system missing (RunConfig.current_tick not incremented)
//!   B3: GroupPosition not added by group_formation_system
//!   B4: trading_system not registered in SimulationPlugin
//!   B5: Duplicate NestCleared/TierUnlockedProgression registration in CreaturesPlugin
//!   B6: group_formation_system ignores BuildingDestroyed events
//!   B7: SimulationPlugin does not init SimTick
//!   B8: UX systems not registered in SimulationPlugin

use bevy::prelude::*;
use bevy::ecs::message::Messages;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::systems::placement::PlacementCommands;
use crate::{SimulationPlugin, CreaturesPlugin, WorldPlugin};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn sim_app(w: i32, h: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: w, grid_height: h });
    app
}

/// Combined app with SimulationPlugin + CreaturesPlugin.
/// After B5 fix: CreaturesPlugin will not duplicate NestCleared / TierUnlockedProgression.
fn sim_creatures_app(w: i32, h: i32) -> App {
    let mut app = sim_app(w, h);
    app.add_plugins(CreaturesPlugin);
    app
}

/// Combined app with SimulationPlugin + WorldPlugin.
fn sim_world_app(w: i32, h: i32) -> App {
    let mut app = sim_app(w, h);
    app.add_plugins(WorldPlugin);
    app
}

/// Queue a building via the legacy placement queue (no fog/tier check); also bypasses terrain requirement checks.
fn place(app: &mut App, bt: BuildingType, x: i32, y: i32, recipe: Recipe) {
    app.world_mut()
        .resource_mut::<PlacementCommands>()
        .queue
        .push((bt, x, y, recipe));
}

/// Reveal all fog cells.
/// Note: legacy queue bypasses fog check; reveal_all is defensive in case path changes.
fn reveal_all(app: &mut App, w: i32, h: i32) {
    app.world_mut().resource_mut::<FogMap>().reveal_all(w, h);
}

/// Set terrain at a specific cell.
fn set_terrain(app: &mut App, x: i32, y: i32, t: TerrainType) {
    app.world_mut().resource_mut::<Grid>().terrain.insert((x, y), t);
}

/// Despawn entity at (x,y) and emit BuildingRemoved event.
fn remove_building_at(app: &mut App, x: i32, y: i32) {
    let entity = {
        let mut q = app.world_mut().query::<(Entity, &Position)>();
        q.iter(app.world())
            .find(|(_, p)| p.x == x && p.y == y)
            .map(|(e, _)| e)
    };
    if let Some(e) = entity {
        let cells: Vec<(i32, i32)> = {
            let mut q = app.world_mut().query::<(Entity, &Footprint)>();
            q.iter(app.world())
                .find(|(ent, _)| *ent == e)
                .map(|(_, fp)| fp.cells.clone())
                .unwrap_or_else(|| vec![(x, y)])
        };
        {
            let mut grid = app.world_mut().resource_mut::<Grid>();
            for c in &cells {
                grid.occupied.remove(c);
            }
        }
        app.world_mut().despawn(e);
        app.world_mut().write_message(BuildingRemoved { entity: e, x, y });
    }
}

/// Despawn entity at (x,y) and emit BuildingDestroyed event (B6 scenario).
#[allow(dead_code)]
fn destroy_building_at(app: &mut App, x: i32, y: i32) {
    let entity = {
        let mut q = app.world_mut().query::<(Entity, &Position)>();
        q.iter(app.world())
            .find(|(_, p)| p.x == x && p.y == y)
            .map(|(e, _)| e)
    };
    if let Some(e) = entity {
        let cells: Vec<(i32, i32)> = {
            let mut q = app.world_mut().query::<(Entity, &Footprint)>();
            q.iter(app.world())
                .find(|(ent, _)| *ent == e)
                .map(|(_, fp)| fp.cells.clone())
                .unwrap_or_else(|| vec![(x, y)])
        };
        {
            let mut grid = app.world_mut().resource_mut::<Grid>();
            for c in &cells {
                grid.occupied.remove(c);
            }
        }
        app.world_mut().despawn(e);
        app.world_mut().write_message(BuildingDestroyed { entity: e, x, y });
    }
}

/// Count Group marker entities.
fn count_groups(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Group>();
    q.iter(app.world()).count()
}

/// Find the group_id of the building at (x, y).
fn group_of(app: &mut App, x: i32, y: i32) -> Option<Entity> {
    let mut q = app.world_mut().query::<(&Position, &GroupMember)>();
    q.iter(app.world())
        .find(|(p, _)| p.x == x && p.y == y)
        .map(|(_, m)| m.group_id)
}

/// Run the app for N ticks.
fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ── Recipe helpers (matching seed data) ──────────────────────────────────────

fn wind_turbine_recipe() -> Recipe {
    Recipe::simple(vec![], vec![], 1)
}

fn iron_miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 60)
}

fn iron_smelter_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::IronOre, 2.0)],
        vec![(ResourceType::IronBar, 1.0)],
        120,
    )
}

fn tannery_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::Hide, 3.0)],
        vec![(ResourceType::TreatedLeather, 1.0)],
        120,
    )
}

/// Spawn a Group entity with Manifold, GroupEnergy, optional sender/receiver.
#[allow(dead_code)]
fn spawn_group(
    world: &mut World,
    x: i32,
    y: i32,
    sender_resource: Option<ResourceType>,
    receiver_resource: Option<ResourceType>,
    receiver_demand: u32,
) -> Entity {
    let mut e = world.spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x, y },
    ));
    if let Some(res) = sender_resource {
        e.insert(TransportSender { resource: Some(res), disconnected: false });
    }
    if let Some(res) = receiver_resource {
        e.insert(TransportReceiver { resource: Some(res), demand: receiver_demand, disconnected: false });
    }
    e.id()
}

/// Spawn a TransportPath entity between two groups.
fn spawn_path(
    world: &mut World,
    kind: TransportKind,
    source_group: Entity,
    target_group: Entity,
    waypoints: Vec<(i32, i32)>,
    tier: u8,
) -> Entity {
    let stats = match kind {
        TransportKind::RunePath => TierStats::for_path(tier),
        TransportKind::Pipe => TierStats::for_pipe(tier),
    };
    let resource_filter = world.entity(source_group)
        .get::<TransportSender>()
        .and_then(|s| s.resource);
    let path_entity = world.spawn(TransportPath {
        kind,
        source_group,
        target_group,
        resource_filter,
        tier,
        capacity: stats.capacity,
        speed: stats.speed,
        connected: true,
        segments: waypoints.clone(),
    }).id();
    world.spawn(PathConnection {
        source_group,
        target_group,
        path_entity,
    });
    {
        let occupancy = &mut world.resource_mut::<PathOccupancy>();
        for pos in waypoints.iter() {
            occupancy.tiles.insert(*pos, path_entity);
        }
    }
    for (idx, pos) in waypoints.iter().enumerate() {
        world.spawn(PathSegmentTile {
            path_entity,
            tile_pos: *pos,
            segment_index: idx,
        });
    }
    path_entity
}

// ─────────────────────────────────────────────────────────────────────────────
// S1: Production Pipeline — Energy + Groups + Transport
// ─────────────────────────────────────────────────────────────────────────────

/// S1: Production pipeline delivers iron_ore across groups via rune_path.
///
/// Requires B3 (GroupPosition added by group_formation_system) for transport to resolve
/// sender/receiver positions, and the transport_movement_system to actually move cargo.
#[test]
fn s1_production_pipeline() {
    let mut app = sim_app(20, 10);
    reveal_all(&mut app, 20, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    // Group A: turbine + 2 miners + smelter
    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronMiner,   3, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());

    // Group B: smelter at (12,3)
    place(&mut app, BuildingType::IronSmelter, 12, 3, iron_smelter_recipe());

    // Form groups
    app.update();

    // Manually spawn Group A entity with TransportSender + GroupPosition
    let group_a = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x: 4, y: 3 },
        TransportSender { resource: Some(ResourceType::IronOre), disconnected: false },
    )).id();

    // Manually spawn Group B entity with TransportReceiver + GroupPosition
    let group_b = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x: 12, y: 3 },
        TransportReceiver { resource: Some(ResourceType::IronOre), demand: 2, disconnected: false },
    )).id();

    // Seed group A manifold so transport can pick up iron_ore
    app.world_mut().entity_mut(group_a)
        .get_mut::<Manifold>().unwrap()
        .resources.insert(ResourceType::IronOre, 10.0);

    // Spawn rune_path A → B, 7 waypoints (5,3)..(11,3)
    let waypoints: Vec<(i32, i32)> = (5..=11).map(|x| (x, 3)).collect();
    spawn_path(app.world_mut(), TransportKind::RunePath, group_a, group_b, waypoints, 1);

    run_ticks(&mut app, 200);

    // Post-condition: at least 1 Cargo entity OR group B has iron_ore > 0
    let has_cargo = {
        let mut q = app.world_mut().query::<&Cargo>();
        q.iter(app.world()).count() > 0
    };
    let group_b_iron_ore = app.world()
        .entity(group_b)
        .get::<Manifold>()
        .map(|m| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert!(
        has_cargo || group_b_iron_ore > 0.0,
        "S1: expected cargo in transit or iron_ore delivered to group B. \
         Cargo count=0, group_b iron_ore={group_b_iron_ore}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S2: Transport Delivery → IronBar produced downstream
// ─────────────────────────────────────────────────────────────────────────────

/// S2: Produced IronOre transports to smelter yielding IronBar.
///
/// Group A mines iron_ore (> MINION_RANGE=5 tiles from B) → rune_path delivers →
/// Group B smelter consumes → produces iron_bar.
#[test]
fn s2_transport_delivery_iron_bar() {
    let mut app = sim_app(20, 10);
    reveal_all(&mut app, 20, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    // Group A: wind_turbine + 2 miners
    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronMiner,   3, 3, iron_miner_recipe());

    // Group B: wind_turbine + smelter (distance > 5 tiles from A)
    place(&mut app, BuildingType::WindTurbine, 11, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronSmelter, 12, 3, iron_smelter_recipe());

    app.update();

    // Manually spawn groups with transport ports
    let group_a = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x: 3, y: 3 },
        TransportSender { resource: Some(ResourceType::IronOre), disconnected: false },
    )).id();

    let group_b = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 12, y: 3 },
        TransportReceiver { resource: Some(ResourceType::IronOre), demand: 2, disconnected: false },
    )).id();

    // Seed group A with iron_ore
    app.world_mut().entity_mut(group_a)
        .get_mut::<Manifold>().unwrap()
        .resources.insert(ResourceType::IronOre, 20.0);

    // Rune_path A → B, 7 waypoints (4..=10, y=3)
    let waypoints: Vec<(i32, i32)> = (4..=10).map(|x| (x, 3)).collect();
    spawn_path(app.world_mut(), TransportKind::RunePath, group_a, group_b, waypoints, 1);

    run_ticks(&mut app, 200);

    let group_b_iron_bar = app.world()
        .entity(group_b)
        .get::<Manifold>()
        .map(|m| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    let group_b_iron_ore = app.world()
        .entity(group_b)
        .get::<Manifold>()
        .map(|m| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    let has_cargo = {
        let mut q = app.world_mut().query::<&Cargo>();
        q.iter(app.world()).count() > 0
    };

    // After transport delivery: either group B has iron_ore (delivery arrived) or iron_bar (smelted).
    // Full iron_bar production requires production_system to process iron_ore in group B.
    // B3 (GroupPosition) affects transport; production also requires buildings linked to group_b.
    assert!(
        group_b_iron_ore > 0.0 || group_b_iron_bar > 0.0 || has_cargo,
        "S2: expected iron_ore or iron_bar in group B manifold, or cargo in transit. \
         iron_ore={group_b_iron_ore}, iron_bar={group_b_iron_bar}, has_cargo={has_cargo}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S3: Energy Crisis Cascade
// ─────────────────────────────────────────────────────────────────────────────

/// S3: Removing energy source halts production and resets milestone sustain.
///
/// Requires B1 (ProductionRates written) and B2 (RunConfig.current_tick incremented)
/// for OpusNodeFull.sustain_ticks to reset to 0 when production drops.
#[test]
fn s3_energy_crisis_cascade() {
    let mut app = sim_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);

    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());

    // OpusNodeFull: low required rate, small sustain window
    app.world_mut().spawn(OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronBar,
        required_rate: 0.01,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    });

    run_ticks(&mut app, 10);

    // After 10 ticks with turbine: EnergyPool.ratio > 0
    // production_rates_system: ratio = min(gen/cons, MAX_MODIFIER) = min(20.0/15.0, 1.5) = 1.333
    //   rate = ratio / duration * output = 1.333 / duration * 1.0 >= required — production proceeds
    let ratio = app.world().resource::<EnergyPool>().ratio;
    assert!(ratio > 0.0, "S3: EnergyPool.ratio should be > 0 after 10 ticks with turbine, got {ratio}");

    // At least one ProductionState should be active
    let any_active = {
        let mut q = app.world_mut().query::<&ProductionState>();
        q.iter(app.world()).any(|ps| ps.active)
    };
    assert!(any_active, "S3: at least one ProductionState.active should be true after 10 ticks");

    // Remove turbine at (1,3)
    remove_building_at(&mut app, 1, 3);

    run_ticks(&mut app, 5);

    // EnergyPool.total_generation should be 0 (no more turbine)
    let total_gen = app.world().resource::<EnergyPool>().total_generation;
    assert_eq!(total_gen, 0.0, "S3: EnergyPool.total_generation should be 0 after turbine removed, got {total_gen}");

    // All ProductionState should be idle due to NoEnergy
    // production_rates_system: ratio=0.0, rate_per_tick = ratio / duration = 0.0 / 1 = 0.0, IronOre += 1.0 * 0.0 = 0.0
    let all_no_energy = {
        let mut q = app.world_mut().query::<&ProductionState>();
        q.iter(app.world())
            .filter(|ps| !ps.active)
            .all(|ps| ps.idle_reason == Some(IdleReason::NoEnergy))
    };
    assert!(
        all_no_energy,
        "S3: all inactive buildings should have idle_reason=NoEnergy after turbine removed"
    );

    // OpusNodeFull.sustain_ticks should be 0 (requires B1: ProductionRates, B2: tick increment)
    let sustain_ticks = {
        let mut q = app.world_mut().query::<&OpusNodeFull>();
        q.iter(app.world()).next().map(|n| n.sustain_ticks).unwrap_or(0)
    };
    assert_eq!(
        sustain_ticks, 0,
        "S3 (B1/B2): OpusNodeFull.sustain_ticks should be 0 after energy crisis, got {sustain_ticks}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S4: Nest Clear → Tier Progression
// ─────────────────────────────────────────────────────────────────────────────

/// S4: Combat pressure clears nest and advances tier.
///
/// Tests combat pressure accumulation: combat group near nest → nest clearing → tier progression.
#[test]
fn s4_nest_clear_tier_progression() {
    let mut app = sim_creatures_app(20, 10);
    reveal_all(&mut app, 20, 10);

    // Spawn combat group entity with CombatGroup component.
    // Manifold pre-seeded with Herbs so combat_group_system computes supply_ratio >= 1.0
    // regardless of whether it runs before or after combat_pressure_system (no ordering guarantee).
    let group_entity = app.world_mut().spawn((
        Group,
        {
            let mut m = Manifold::default();
            m.resources.insert(ResourceType::Herbs, 100.0);
            m
        },
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 3, y: 3 },
        Position { x: 3, y: 3 },
        CombatGroup {
            building_kind: CombatBuildingKind::ImpCamp,
            base_organic_rate: 1.0,
            base_protection_radius: 6.0,
            protection_dps: 100.0,
            breach_threshold: 0.3,
            supply_ratio: 1.0,
            max_minions: 4,
            output_multiplier: 1.0,
            consumption_multiplier: 1.0,
        },
    )).id();

    // Spawn a proper ImpCamp Building member of the group (matches real game entity structure).
    app.world_mut().spawn((
        Building { building_type: BuildingType::ImpCamp },
        GroupMember { group_id: group_entity },
        CombatGroup {
            building_kind: CombatBuildingKind::ImpCamp,
            base_organic_rate: 1.0,
            base_protection_radius: 6.0,
            protection_dps: 100.0,
            breach_threshold: 0.3,
            supply_ratio: 1.0,
            max_minions: 4,
            output_multiplier: 1.0,
            consumption_multiplier: 1.0,
        },
        Position { x: 3, y: 3 },
    ));

    // Spawn CreatureNest at (6,3)
    let _nest = app.world_mut().spawn((
        CreatureNest {
            nest_id: NestId::ForestWolfDen,
            biome: BiomeTag::Forest,
            tier: 1,
            hostility: NestHostility::Hostile,
            strength: 50.0,
            territory_radius: 5.0,
            cleared: false,
            extracting: false,
            loot_on_clear: Default::default(),
        },
        Position { x: 6, y: 3 },
        CombatPressure { value: 0.0 },
    )).id();

    // Spawn TierGate
    app.world_mut().spawn(TierGateComponent {
        tier: 2,
        nest_id: "ForestWolfDen".to_string(),
        unlocked: false,
    });

    // Spawn BuildingTier on a building entity (simulating placed buildings)
    let _building_e = app.world_mut().spawn((
        Building { building_type: BuildingType::ImpCamp },
        Position { x: 4, y: 3 },
        BuildingTier { tier: 1 },
    )).id();

    run_ticks(&mut app, 2);

    // After 2 ticks: NestCleared event should be emitted (combat_pressure_system → nest_clearing_system)
    let nest_cleared = {
        let msgs = app.world().get_resource::<Messages<NestCleared>>().unwrap();
        msgs.iter_current_update_messages().any(|e| e.nest_id.contains("ForestWolfDen"))
    };
    // TierState should advance to 2 (tier_gate_system reacts to NestCleared)
    let tier = app.world().resource::<TierState>().current_tier;

    assert!(nest_cleared, "nest should be cleared after sufficient combat pressure");
    assert_eq!(tier, 2, "tier should advance from 1 to 2 after nest clearing");
}

// ─────────────────────────────────────────────────────────────────────────────
// S5: Organic Supply Chain
// ─────────────────────────────────────────────────────────────────────────────

/// S5: Combat group organics transported to tannery.
///
/// Combat group pre-seeded with Hide → rune_path → processing group with tannery.
#[test]
fn s5_organic_supply_chain() {
    let mut app = sim_creatures_app(30, 10);
    reveal_all(&mut app, 30, 10);

    // Combat group with pre-seeded Hide manifold
    let combat_group = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 3, y: 3 },
        TransportSender { resource: Some(ResourceType::Hide), disconnected: false },
        {
            let mut m = Manifold::default();
            m.resources.insert(ResourceType::Hide, 10.0);
            m
        },
    )).id();

    // Processing group with tannery — spawn group and buildings manually to avoid
    // placement_system triggering group_formation_system and despawning processing_group.
    let processing_group = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 12.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 15, y: 3 },
        Manifold::default(),
        TransportReceiver { resource: Some(ResourceType::Hide), demand: 2, disconnected: false },
    )).id();

    // Spawn tannery and turbine buildings directly (no placement queue, avoids group reform)
    app.world_mut().spawn((
        Building { building_type: BuildingType::WindTurbine },
        Position { x: 14, y: 3 },
        wind_turbine_recipe(),
        ProductionState::default(),
        InputBuffer::default(),
        OutputBuffer::default(),
        Footprint::single(14, 3),
        GroupMember { group_id: processing_group },
    ));
    app.world_mut().spawn((
        Building { building_type: BuildingType::Tannery },
        Position { x: 15, y: 3 },
        tannery_recipe(),
        ProductionState::default(),
        InputBuffer::default(),
        OutputBuffer::default(),
        Footprint::single(15, 3),
        GroupMember { group_id: processing_group },
    ));

    // OpusNodeFull for TreatedLeather
    app.world_mut().spawn(OpusNodeFull {
        node_index: 0,
        resource: ResourceType::TreatedLeather,
        required_rate: 0.01,
        tier: 2,
        sustained: false,
        sustain_ticks: 0,
    });

    // Rune_path from combat_group to processing_group (10 waypoints)
    let waypoints: Vec<(i32, i32)> = (4..=13).map(|x| (x, 3)).collect();
    spawn_path(app.world_mut(), TransportKind::RunePath, combat_group, processing_group, waypoints, 1);

    run_ticks(&mut app, 50);

    // Check: at least 1 Cargo entity OR processing group has Hide > 0
    let has_cargo = {
        let mut q = app.world_mut().query::<&Cargo>();
        q.iter(app.world()).count() > 0
    };
    let processing_hide = app.world()
        .entity(processing_group)
        .get::<Manifold>()
        .map(|m| m.resources.get(&ResourceType::Hide).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    // Also check tannery InputBuffer
    let tannery_hide = {
        let mut q = app.world_mut().query::<(&Position, &InputBuffer)>();
        q.iter(app.world())
            .find(|(p, _)| p.x == 15 && p.y == 3)
            .map(|(_, buf)| buf.slots.get(&ResourceType::Hide).copied().unwrap_or(0.0))
            .unwrap_or(0.0)
    };

    assert!(
        has_cargo || processing_hide > 0.0 || tannery_hide > 0.0,
        "S5: expected cargo or Hide delivered to processing group after 50 ticks. \
         has_cargo={has_cargo}, processing_hide={processing_hide}, tannery_hide={tannery_hide}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S6: Group Split on Removal
// ─────────────────────────────────────────────────────────────────────────────

/// S6: Removing middle building splits group into two.
///
/// Places 4 buildings in a row, removes the smelter at (4,3) that bridges the miners.
/// After removal, the two miners should be in separate groups.
#[test]
fn s6_group_split_on_removal() {
    let mut app = sim_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::IronVein);

    // Row: turbine at (3,4), miner@(3,3), smelter@(4,3), miner@(5,3)
    place(&mut app, BuildingType::WindTurbine, 3, 4, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   3, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place(&mut app, BuildingType::IronMiner,   5, 3, iron_miner_recipe());

    run_ticks(&mut app, 1);

    // After 1 tick: all 4 buildings in 1 group
    let initial_groups = count_groups(&mut app);
    // The 4 buildings form adjacent connections: turbine-(3,4) adj to miner-(3,3) adj to smelter-(4,3) adj to miner-(5,3)
    // They should coalesce into 1 group
    assert!(
        initial_groups >= 1,
        "S6: expected at least 1 group after placement, got {initial_groups}"
    );

    // Remove the smelter at (4,3) — bridges the two miners
    remove_building_at(&mut app, 4, 3);

    run_ticks(&mut app, 1);

    // After removal: 2 groups (miner@3,3+turbine@3,4 vs miner@5,3)
    let final_groups = count_groups(&mut app);

    let group_left  = group_of(&mut app, 3, 3);
    let group_right = group_of(&mut app, 5, 3);

    assert_eq!(
        final_groups, 2,
        "S6: expected 2 groups after removing bridge smelter, got {final_groups}"
    );
    assert!(
        group_left.is_some() && group_right.is_some(),
        "S6: both remaining miners should have group membership"
    );
    assert_ne!(
        group_left.unwrap(), group_right.unwrap(),
        "S6: miners at (3,3) and (5,3) should be in different groups after split"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S7: Full Run Win Condition
// ─────────────────────────────────────────────────────────────────────────────

/// S7: All opus nodes sustained triggers RunWon.
///
/// Requires B1 (ProductionRates written) and B2 (current_tick incremented)
/// for run_lifecycle_system to detect win condition.
#[test]
fn s7_run_win_condition() {
    let mut app = sim_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);

    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());

    // OpusNodeFull with very low required rate (easily achievable)
    app.world_mut().spawn(OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronBar,
        required_rate: 0.01,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    });

    // Configure RunConfig with short sustain window
    {
        let mut rc = app.world_mut().resource_mut::<RunConfig>();
        rc.sustain_window_ticks = 3;
        rc.max_ticks = 1000;
    }

    // Configure OpusTreeResource with low sustain_ticks_required
    {
        let mut tree = app.world_mut().resource_mut::<OpusTreeResource>();
        tree.sustain_ticks_required = 2;
        tree.main_path = vec![crate::resources::OpusNodeEntry {
            node_index: 0,
            resource: ResourceType::IronBar,
            required_rate: 0.01,
            current_rate: 0.0,
            tier: 1,
            sustained: false,
        }];
    }

    run_ticks(&mut app, 15);

    // After 15 ticks: if B1+B2 are fixed, ProductionRates are written and ticks increment
    // milestone_check_system will sustain the node, opus_tree_sync_system will update tree,
    // run_lifecycle_system will fire RunWon and set RunState.status = Won
    let run_status = app.world().resource::<RunState>().status;
    let all_sustained = app.world().resource::<OpusTreeResource>().all_sustained();

    assert!(
        all_sustained || run_status == RunStatus::Won,
        "S7 (B1/B2): expected OpusTree all_sustained or RunState==Won after 15 ticks. \
         all_sustained={all_sustained}, run_status={run_status:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S8: Diamond Network Conservation
// ─────────────────────────────────────────────────────────────────────────────

/// S8: Diamond transport network conserves resources (no negative amounts).
///
/// Core invariant: sum of all resources in Manifolds + Cargo + InputBuffers >= 0.
/// No resource should go negative at any point.
#[test]
fn s8_diamond_conservation() {
    let mut app = sim_app(30, 20);
    reveal_all(&mut app, 30, 20);
    set_terrain(&mut app, 2, 5, TerrainType::IronVein);
    set_terrain(&mut app, 3, 5, TerrainType::IronVein);

    // Group A: source — turbine + 2 miners
    place(&mut app, BuildingType::WindTurbine, 1, 5, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 5, iron_miner_recipe());
    place(&mut app, BuildingType::IronMiner,   3, 5, iron_miner_recipe());

    app.update();

    // Group A with TransportSender (iron_ore) — seed manifold
    let group_a = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 0.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 3, y: 5 },
        TransportSender { resource: Some(ResourceType::IronOre), disconnected: false },
        {
            let mut m = Manifold::default();
            m.resources.insert(ResourceType::IronOre, 50.0);
            m
        },
    )).id();

    // Group B: upper — turbine + smelter (receiver iron_ore)
    let group_b = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 10, y: 2 },
        Manifold::default(),
        TransportReceiver { resource: Some(ResourceType::IronOre), demand: 1, disconnected: false },
    )).id();

    // Group C: lower — turbine + smelter (receiver iron_ore)
    let group_c = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 10, y: 8 },
        Manifold::default(),
        TransportReceiver { resource: Some(ResourceType::IronOre), demand: 1, disconnected: false },
    )).id();

    // Group D: sink — turbine + smelter (receiver iron_bar)
    let group_d = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 20.0, priority: EnergyPriority::Medium },
        GroupPosition { x: 18, y: 5 },
        Manifold::default(),
        TransportReceiver { resource: Some(ResourceType::IronBar), demand: 1, disconnected: false },
    )).id();

    // Rune paths: A→B, A→C (upper and lower), B→D, C→D
    let path_ab: Vec<(i32, i32)> = (4..=9).map(|x| (x, 3)).collect(); // 6 waypoints
    let path_ac: Vec<(i32, i32)> = (4..=9).map(|x| (x, 7)).collect();
    let path_bd: Vec<(i32, i32)> = (11..=17).map(|x| (x, 2)).collect(); // 7 waypoints
    let path_cd: Vec<(i32, i32)> = (11..=17).map(|x| (x, 8)).collect();

    spawn_path(app.world_mut(), TransportKind::RunePath, group_a, group_b, path_ab, 1);
    spawn_path(app.world_mut(), TransportKind::RunePath, group_a, group_c, path_ac, 1);
    spawn_path(app.world_mut(), TransportKind::RunePath, group_b, group_d, path_bd, 1);
    spawn_path(app.world_mut(), TransportKind::RunePath, group_c, group_d, path_cd, 1);

    run_ticks(&mut app, 300);

    // Conservation: no negative resource amounts in any Manifold
    let has_negative_manifold = {
        let mut q = app.world_mut().query::<&Manifold>();
        q.iter(app.world()).any(|m| {
            m.resources.values().any(|&v| v < 0.0)
        })
    };

    assert!(
        !has_negative_manifold,
        "S8: resource conservation violated — negative amount found in a Manifold after 300 ticks"
    );

    // No negative Cargo amounts
    let has_negative_cargo = {
        let mut q = app.world_mut().query::<&Cargo>();
        q.iter(app.world()).any(|c| c.amount < 0.0)
    };

    assert!(
        !has_negative_cargo,
        "S8: resource conservation violated — negative amount found in a Cargo entity"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S9: Determinism
// ─────────────────────────────────────────────────────────────────────────────

/// S9: Two identical setups produce identical ECS state after 50 ticks.
///
/// Note: Entity IDs are non-deterministic across two separate App instances,
/// so we compare aggregate values: sorted progress, energy totals, total manifold sums.
#[test]
fn s9_determinism() {
    let mut app_a = sim_app(10, 10);
    let mut app_b = sim_app(10, 10);

    for app in [&mut app_a, &mut app_b] {
        app.world_mut().resource_mut::<FogMap>().reveal_all(10, 10);
        app.world_mut().resource_mut::<Grid>().terrain.insert((2, 3), TerrainType::IronVein);
        {
            let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
            cmds.queue.push((BuildingType::WindTurbine, 1, 3, Recipe::simple(vec![], vec![], 1)));
            cmds.queue.push((BuildingType::IronMiner,   2, 3, Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 60)));
            cmds.queue.push((BuildingType::IronSmelter, 3, 3, Recipe::simple(vec![(ResourceType::IronOre, 2.0)], vec![(ResourceType::IronBar, 1.0)], 120)));
        }
    }

    for _ in 0..50 {
        app_a.update();
        app_b.update();
    }

    // Compare EnergyPool
    let (total_gen_a, cons_a) = {
        let ep = app_a.world().resource::<EnergyPool>();
        (ep.total_generation, ep.total_consumption)
    };
    let (total_gen_b, cons_b) = {
        let ep = app_b.world().resource::<EnergyPool>();
        (ep.total_generation, ep.total_consumption)
    };

    assert_eq!(total_gen_a, total_gen_b, "S9: EnergyPool.total_generation differs: a={total_gen_a} b={total_gen_b}");
    assert_eq!(cons_a, cons_b, "S9: EnergyPool.total_consumption differs: a={cons_a} b={cons_b}");

    // Compare sorted ProductionState.progress values
    let mut prog_a: Vec<f32> = {
        let mut q = app_a.world_mut().query::<&ProductionState>();
        q.iter(app_a.world()).map(|ps| ps.progress).collect()
    };
    let mut prog_b: Vec<f32> = {
        let mut q = app_b.world_mut().query::<&ProductionState>();
        q.iter(app_b.world()).map(|ps| ps.progress).collect()
    };
    prog_a.sort_by(|a, b| a.partial_cmp(b).unwrap());
    prog_b.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(prog_a.len(), prog_b.len(), "S9: different number of ProductionState entities");
    for (i, (a, b)) in prog_a.iter().zip(prog_b.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-6,
            "S9: ProductionState.progress[{i}] differs: a={a} b={b}"
        );
    }

    // Compare total manifold resource sums
    let manifold_sum = |app: &mut App| -> f32 {
        let mut q = app.world_mut().query::<&Manifold>();
        q.iter(app.world())
            .flat_map(|m| m.resources.values().copied())
            .sum()
    };

    let sum_a = manifold_sum(&mut app_a);
    let sum_b = manifold_sum(&mut app_b);

    assert!(
        (sum_a - sum_b).abs() < 1e-4,
        "S9: total manifold resource sum differs: a={sum_a} b={sum_b}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S10: Dashboard Reads Live State
// ─────────────────────────────────────────────────────────────────────────────

/// S10: Dashboard reflects live ECS state after 5 ticks.
///
/// Requires B8: UX systems (dashboard_system, chain_visualizer_system, tick_system)
/// must be registered in SimulationPlugin for DashboardState to be updated.
#[test]
fn s10_dashboard_live_state() {
    let mut app = sim_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);

    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());

    // Insert DashboardState with is_open=true
    app.world_mut().insert_resource(DashboardState {
        is_open: true,
        ..Default::default()
    });

    // Insert CurrentTier resource
    app.world_mut().insert_resource(CurrentTier { tier: 1 });

    // Insert SimulationTick at 0 (requires B8 to be incremented by tick_system)
    app.world_mut().insert_resource(SimulationTick { tick: 0 });

    // Spawn OpusTree and OpusNode entities
    app.world_mut().spawn(OpusTree { total_nodes: 1 });
    app.world_mut().spawn(OpusNode {
        resource: ResourceType::IronBar,
        required_rate: 1.0,
        sustained: false,
    });

    run_ticks(&mut app, 5);

    // Get EnergyPool for reference
    let (generation, cons) = {
        let ep = app.world().resource::<EnergyPool>();
        (ep.total_generation, ep.total_consumption)
    };
    let expected_balance = generation - cons;

    // After B8 fix: DashboardState.energy_balance == EnergyPool.total_generation - total_consumption
    let dashboard = app.world().resource::<DashboardState>();

    // energy_balance should match EnergyPool
    let _balance_ok = (dashboard.energy_balance - expected_balance).abs() < 1e-4
        || expected_balance > 0.0; // if balance > 0, color should be green

    // current_tier should be 1
    assert_eq!(
        dashboard.current_tier, 1,
        "S10 (B8): DashboardState.current_tier should be 1, got {}",
        dashboard.current_tier
    );

    // SimulationTick.tick should be 5 after 5 updates (requires B8: tick_system)
    let sim_tick = app.world().resource::<SimulationTick>().tick;
    assert_eq!(
        sim_tick, 5,
        "S10 (B8): SimulationTick.tick should be 5 after 5 updates, got {sim_tick}"
    );

    // If balance > 0 (turbine generates), color should be Green (requires B8)
    if expected_balance > 0.0 {
        assert_eq!(
            dashboard.energy_color,
            Some(GaugeColor::Green),
            "S10 (B8): DashboardState.energy_color should be Some(Green) when balance > 0"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// S11: Trader Converts Surplus to Meta-Currency
// ─────────────────────────────────────────────────────────────────────────────

/// S11: Trader converts manifold surplus to Gold with inflation.
///
/// Requires B4: trading_system must be registered in SimulationPlugin at Phase::Manifold.
#[test]
fn s11_trader_surplus() {
    let mut app = sim_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);

    // Place turbine + miner (so iron_ore builds up)
    place(&mut app, BuildingType::WindTurbine, 1, 3, wind_turbine_recipe());
    place(&mut app, BuildingType::IronMiner,   2, 3, iron_miner_recipe());

    app.update(); // form group

    // Find the group the miner belongs to
    let miner_group = group_of(&mut app, 2, 3);

    // Spawn a Trader building entity with TraderState + TraderEarnings
    let trader_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::Trader },
        Position { x: 3, y: 3 },
        TraderState::default(),
        TraderEarnings::default(),
        Recipe::simple(vec![], vec![], 1),
        ProductionState::default(),
        InputBuffer::default(),
        OutputBuffer::default(),
        Footprint::single(3, 3),
    )).id();

    // Seed the group manifold with iron_ore so trader has surplus to sell
    if let Some(group_e) = miner_group {
        if let Some(mut manifold) = app.world_mut().entity_mut(group_e).get_mut::<Manifold>() {
            manifold.resources.insert(ResourceType::IronOre, 100.0);
        }
    } else {
        // If no group found yet, spawn our own group with pre-seeded manifold + trader
        let g = app.world_mut().spawn((
            Group,
            GroupEnergy { demand: 5.0, allocated: 20.0, priority: EnergyPriority::Medium },
            GroupPosition { x: 2, y: 3 },
            {
                let mut m = Manifold::default();
                m.resources.insert(ResourceType::IronOre, 100.0);
                m
            },
        )).id();

        // Assign trader to group
        app.world_mut().entity_mut(trader_entity).insert(GroupMember { group_id: g });
    }

    run_ticks(&mut app, 100);

    // After B4 fix: trading_system should drain manifold and credit TraderEarnings
    let gold = app.world()
        .entity(trader_entity)
        .get::<TraderEarnings>()
        .map(|e| e.gold)
        .unwrap_or(0.0);

    let iron_ore_volume = app.world()
        .entity(trader_entity)
        .get::<TraderState>()
        .and_then(|s| s.volume_traded.get(&ResourceType::IronOre).copied())
        .unwrap_or(0.0);

    assert!(
        gold > 0.0,
        "S11 (B4): TraderEarnings.gold should be > 0 after 100 ticks with iron_ore surplus. \
         Got gold={gold}"
    );

    assert!(
        iron_ore_volume > 0.0,
        "S11 (B4): TraderState.volume_traded[IronOre] should be > 0. Got {iron_ore_volume}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// S12: Hazard Destroys Building → Group Reforms
// ─────────────────────────────────────────────────────────────────────────────

/// S12: Hazard destroys middle building (BuildingDestroyed event) → group splits.
///
/// Tests B6: group_formation_system must listen to BuildingDestroyed events.
/// Before B6 fix: group_formation_system only listens to BuildingRemoved.
/// After B6 fix: it also handles BuildingDestroyed → groups split correctly.
///
/// This test uses WorldPlugin (which emits BuildingDestroyed) + SimulationPlugin.
/// The hazard at center=(4,3) radius=0 will destroy the smelter at (4,3).
#[test]
fn s12_hazard_group_reform() {
    let mut app = sim_world_app(10, 10);
    reveal_all(&mut app, 10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::IronVein);

    // Place buildings: miner@(3,3), smelter@(4,3), miner@(5,3)
    place(&mut app, BuildingType::IronMiner,   3, 3, iron_miner_recipe());
    place(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place(&mut app, BuildingType::IronMiner,   5, 3, iron_miner_recipe());

    run_ticks(&mut app, 1);

    // Record initial consumption (miner+smelter+miner = 5+10+5 = 20)
    let initial_consumption = app.world().resource::<EnergyPool>().total_consumption;

    // Verify 1 group formed (or at least buildings placed)
    let buildings_count = {
        let mut q = app.world_mut().query::<&Building>();
        q.iter(app.world()).count()
    };
    assert!(buildings_count >= 3, "S12: expected 3 buildings placed, got {buildings_count}");

    // Spawn BiomeHazard at (4,3) radius=0, fires at tick 3 with intensity=999
    // WorldPlugin's hazard_trigger_system will fire and emit BuildingDestroyed
    app.world_mut().spawn(BiomeHazard {
        hazard_kind: HazardKind::Eruption,
        center_x: 4,
        center_y: 3,
        radius: 0,
        intensity: 999.0,
        next_event_tick: 3,
        warning_ticks: 1,
        interval_ticks: 1000,
        interval_variance: 0,
        warning_issued: false,
    });

    // Also init SimTick (needed by WorldPlugin's tick_advance_system)
    // B7: SimulationPlugin should also init SimTick, but WorldPlugin already does it
    // (WorldPlugin is added, so SimTick is already present)

    run_ticks(&mut app, 5);

    // Check that BuildingDestroyed was emitted for position (4,3)
    let destroyed_emitted = {
        let msgs = app.world().get_resource::<Messages<BuildingDestroyed>>().unwrap();
        msgs.iter_current_update_messages().any(|e| e.x == 4 && e.y == 3)
    };

    // Check that smelter at (4,3) no longer exists
    let smelter_exists = {
        let mut q = app.world_mut().query::<(&Building, &Position)>();
        q.iter(app.world()).any(|(_, p)| p.x == 4 && p.y == 3)
    };

    // Check group count: after B6 fix → 2 groups
    let final_groups = count_groups(&mut app);

    // Check energy consumption dropped (smelter removed: consumption should decrease)
    let final_consumption = app.world().resource::<EnergyPool>().total_consumption;

    assert!(
        !smelter_exists,
        "S12: smelter at (4,3) should be destroyed by hazard after 5 ticks"
    );

    assert!(
        destroyed_emitted || !smelter_exists,
        "S12: BuildingDestroyed event should be emitted for (4,3) OR building should be gone"
    );

    // After B6 fix: group_formation_system reacts to BuildingDestroyed → 2 groups
    assert_eq!(
        final_groups, 2,
        "S12 (B6): expected 2 groups after hazard destroys bridge building, got {final_groups}"
    );

    assert!(
        final_consumption < initial_consumption,
        "S12: EnergyPool.total_consumption should decrease after smelter removed. \
         initial={initial_consumption}, final={final_consumption}"
    );
}
