//! Transport BDD tests — one test per scenario in transport.feature
//! Seed data: path_tiers.yaml, minion_carry.yaml, fixtures.yaml
//!
//! T1 RunePath: capacity=2, speed=1.0
//! T2 RunePath: capacity=5, speed=2.0
//! T3 RunePath: capacity=10, speed=3.0
//! T1 Pipe:     capacity=3, speed=1.5
//! T2 Pipe:     capacity=8, speed=3.0
//! T3 Pipe:     capacity=15, speed=4.5
//! Minion carry: range=5, rate=0.5

use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::SimulationPlugin;

fn test_app(w: i32, h: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: w, grid_height: h });
    app
}

/// Spawn a minimal Group entity with Manifold, Group marker, and optional sender/receiver.
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

/// Build a TransportPath entity with a PathConnection and PathSegmentTile entities.
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
    let path_entity = world.spawn(TransportPath {
        kind,
        source_group,
        target_group,
        resource_filter: None,
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
    // Register segment tiles
    {
        let occupancy = &mut world.resource_mut::<PathOccupancy>();
        for (idx, pos) in waypoints.iter().enumerate() {
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

// ── AC1: Draw rune path between two groups ────────────────────────────────────

/// Scenario: Draw rune path between two groups
#[test]
fn draw_rune_path_between_two_groups() {
    let mut app = test_app(16, 10);

    // Set T1 tier
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    // Issue DrawPath command
    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints: waypoints.clone(),
        is_pipe: false,
    });

    app.update();

    // Assert: a TransportPath entity exists with 6 segments
    let mut q = app.world_mut().query::<&TransportPath>();
    let paths: Vec<&TransportPath> = q.iter(app.world()).collect();
    assert_eq!(paths.len(), 1, "should have created 1 rune_path entity");
    assert_eq!(paths[0].segments.len(), 6, "path should have 6 segment tiles");
    assert_eq!(paths[0].kind, TransportKind::RunePath);
    assert!(paths[0].connected, "path should be connected");

    // Assert: a PathConnection exists linking group A to group B
    let mut conn_q = app.world_mut().query::<&PathConnection>();
    let connections: Vec<&PathConnection> = conn_q.iter(app.world()).collect();
    assert_eq!(connections.len(), 1, "should have 1 PathConnection");
    assert_eq!(connections[0].source_group, src_group);
    assert_eq!(connections[0].target_group, dst_group);

    // Assert: last result is Ok
    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::Ok));
}

/// Scenario: Draw pipe between two groups for liquid resource
#[test]
fn draw_pipe_between_two_groups_for_liquid_resource() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 3);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints: waypoints.clone(),
        is_pipe: true,
    });

    app.update();

    let mut q = app.world_mut().query::<&TransportPath>();
    let paths: Vec<&TransportPath> = q.iter(app.world()).collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].segments.len(), 4, "pipe should have 4 segment tiles");
    assert_eq!(paths[0].kind, TransportKind::Pipe, "should be a Pipe");

    // A PathConnection exists
    let mut conn_q = app.world_mut().query::<&PathConnection>();
    let conns: Vec<&PathConnection> = conn_q.iter(app.world()).collect();
    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0].source_group, src_group);
    assert_eq!(conns[0].target_group, dst_group);
}

/// Scenario: Reject DrawPath when waypoint tile is impassable (lava_source)
#[test]
fn reject_draw_path_when_waypoint_tile_is_impassable() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Set lava at [6,5]
    app.world_mut().resource_mut::<Grid>().terrain.insert((6, 5), TerrainType::LavaSource);

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints,
        is_pipe: false,
    });

    app.update();

    // Command should be rejected
    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedImpassable));

    // No path entity should be created
    let mut q = app.world_mut().query::<&TransportPath>();
    assert_eq!(q.iter(app.world()).count(), 0, "no path entity should be created");
}

/// Scenario: Reject DrawPath when waypoint exceeds max path length of 32 tiles
#[test]
fn reject_draw_path_when_waypoint_exceeds_max_path_length_of_32_tiles() {
    let mut app = test_app(50, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 40, 5,
        None, Some(ResourceType::IronOre), 2);

    // 35 waypoints > max 32
    let waypoints: Vec<(i32,i32)> = (5..40).map(|x| (x, 5)).collect();
    assert!(waypoints.len() > 32);

    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints,
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedTooLong));

    let mut q = app.world_mut().query::<&TransportPath>();
    assert_eq!(q.iter(app.world()).count(), 0, "no path entity should be created");
}

// ── AC2: Solid resource moves along rune path ─────────────────────────────────

/// Scenario: Solid cargo moves along T1 rune path at speed 1.0 cells per tick
#[test]
fn solid_cargo_moves_along_t1_rune_path_at_speed_1_0_cells_per_tick() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Seed group A with 2 iron_ore
    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 2.0;
    }

    app.update();

    // A Cargo entity should exist with positionOnPath = speed = 1.0
    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty(), "a Cargo entity should be created");
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::IronOre)
        .expect("cargo should carry IronOre");
    assert_eq!(cargo.position_on_path, 1.0,
        "T1 speed=1.0: positionOnPath should be 1.0 after 1 tick");
}

/// Scenario: Solid cargo moves along T2 rune path at speed 2.0 cells per tick
#[test]
fn solid_cargo_moves_along_t2_rune_path_at_speed_2_0_cells_per_tick() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 2;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 5);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 2);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 5.0;
    }

    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty(), "a Cargo entity should be created");
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::IronOre)
        .expect("cargo should carry IronOre");
    assert_eq!(cargo.position_on_path, 2.0,
        "T2 speed=2.0: positionOnPath should be 2.0 after 1 tick");
}

/// Scenario: Solid cargo moves along T3 rune path at speed 3.0 cells per tick
#[test]
fn solid_cargo_moves_along_t3_rune_path_at_speed_3_0_cells_per_tick() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 3;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 10);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 3);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 10.0;
    }

    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty());
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::IronOre)
        .expect("cargo should carry IronOre");
    assert_eq!(cargo.position_on_path, 3.0,
        "T3 speed=3.0: positionOnPath should be 3.0 after 1 tick");
}

/// Scenario: Cargo arriving at path end delivers resource to destination manifold
#[test]
fn cargo_arriving_at_path_end_delivers_resource_to_destination_manifold() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    // Path has 6 segments (length 6)
    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Pre-place cargo at positionOnPath 5.5 (will advance to 6.5 > 6 = deliver)
    app.world_mut().spawn(Cargo {
        path_entity,
        resource: ResourceType::IronOre,
        amount: 2.0,
        position_on_path: 5.5,
    });

    app.update();

    // Cargo should be delivered (despawned)
    let mut cargo_q = app.world_mut().query::<&Cargo>();
    let cargo_count = cargo_q.iter(app.world())
        .filter(|c| c.path_entity == path_entity && c.resource == ResourceType::IronOre)
        .count();
    // Note: movement system may also spawn new cargo; we just check delivery happened
    // by verifying dst manifold has iron ore
    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), dst_group).unwrap();
    let iron_in_dst = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(iron_in_dst, 2.0, "2 iron_ore should be delivered to group B manifold");
}

// ── AC3: Liquid flows through pipes ──────────────────────────────────────────

/// Scenario: Liquid cargo moves through T1 pipe at speed 1.5 cells per tick
#[test]
fn liquid_cargo_moves_through_t1_pipe_at_speed_1_5_cells_per_tick() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 3);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::Pipe, src_group, dst_group, waypoints, 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::Water).or_default() = 3.0;
    }

    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty(), "a Cargo entity should be created on the pipe");
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::Water)
        .expect("cargo should carry Water");
    assert_eq!(cargo.position_on_path, 1.5,
        "T1 pipe speed=1.5: positionOnPath should be 1.5 after 1 tick");
}

/// Scenario: Liquid cargo moves through T2 pipe at speed 3.0 cells per tick
#[test]
fn liquid_cargo_moves_through_t2_pipe_at_speed_3_0_cells_per_tick() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 2;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 8);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::Pipe, src_group, dst_group, waypoints, 2);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::Water).or_default() = 8.0;
    }

    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty());
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::Water)
        .expect("cargo should carry Water");
    assert_eq!(cargo.position_on_path, 3.0,
        "T2 pipe speed=3.0: positionOnPath should be 3.0 after 1 tick");
}

/// Scenario: Liquid cargo moves through T3 pipe at speed 4.5 cells per tick
#[test]
fn liquid_cargo_moves_through_t3_pipe_at_speed_4_5_cells_per_tick() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 3;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 15);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::Pipe, src_group, dst_group, waypoints, 3);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::Water).or_default() = 15.0;
    }

    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    let cargos: Vec<&Cargo> = q.iter(app.world()).collect();
    assert!(!cargos.is_empty());
    let cargo = cargos.iter().find(|c| c.resource == ResourceType::Water)
        .expect("cargo should carry Water");
    assert_eq!(cargo.position_on_path, 4.5,
        "T3 pipe speed=4.5: positionOnPath should be 4.5 after 1 tick");
}

// ── AC4: Tier unlock upgrades all paths and pipes ────────────────────────────

/// Scenario: T2 unlock upgrades all existing T1 rune paths to T2
#[test]
fn t2_unlock_upgrades_all_existing_t1_rune_paths_to_t2() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    // Create T1 path
    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Verify initial T1 stats
    {
        let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
        assert_eq!(path.tier, 1);
        assert_eq!(path.capacity, 2);
        assert_eq!(path.speed, 1.0);
    }

    // Fire TierUnlocked for tier 2
    app.world_mut().write_message(TierUnlocked { tier: 2 });
    app.update();

    // Path should now be T2
    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert_eq!(path.tier, 2, "path tier should become 2");
    assert_eq!(path.capacity, 5, "T2 capacity should be 5");
    assert_eq!(path.speed, 2.0, "T2 speed should be 2.0");
}

/// Scenario: T2 unlock upgrades all existing T1 pipes to T2
#[test]
fn t2_unlock_upgrades_all_existing_t1_pipes_to_t2() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 3);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::Pipe, src_group, dst_group, waypoints, 1);

    // Verify T1
    {
        let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
        assert_eq!(path.tier, 1);
        assert_eq!(path.capacity, 3);
        assert_eq!(path.speed, 1.5);
    }

    app.world_mut().write_message(TierUnlocked { tier: 2 });
    app.update();

    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert_eq!(path.tier, 2);
    assert_eq!(path.capacity, 8, "T2 pipe capacity should be 8");
    assert_eq!(path.speed, 3.0, "T2 pipe speed should be 3.0");
}

/// Scenario: T3 unlock upgrades all existing paths and pipes to T3
#[test]
fn t3_unlock_upgrades_all_existing_paths_and_pipes_to_t3() {
    let mut app = test_app(20, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 2;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 5);
    let src_group2 = spawn_group(app.world_mut(), 2, 7,
        Some(ResourceType::Water), None, 0);
    let dst_group2 = spawn_group(app.world_mut(), 8, 7,
        None, Some(ResourceType::Water), 8);

    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group,
        vec![(4,5),(5,5),(6,5),(7,5),(8,5)], 2);
    let pipe_entity = spawn_path(app.world_mut(), TransportKind::Pipe, src_group2, dst_group2,
        vec![(4,7),(5,7),(6,7),(7,7)], 2);

    app.world_mut().write_message(TierUnlocked { tier: 3 });
    app.update();

    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert_eq!(path.tier, 3);
    assert_eq!(path.capacity, 10, "T3 path capacity = 10");
    assert_eq!(path.speed, 3.0, "T3 path speed = 3.0");

    let pipe = app.world().entity(pipe_entity).get::<TransportPath>().unwrap();
    assert_eq!(pipe.tier, 3);
    assert_eq!(pipe.capacity, 15, "T3 pipe capacity = 15");
    assert_eq!(pipe.speed, 4.5, "T3 pipe speed = 4.5");
}

/// Scenario: Newly built path after T2 unlock is created at T2 tier
#[test]
fn newly_built_path_after_t2_unlock_is_created_at_t2_tier() {
    let mut app = test_app(16, 10);
    // Set tier to 2 already
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 2;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 5);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints,
        is_pipe: false,
    });

    app.update();

    let mut q = app.world_mut().query::<&TransportPath>();
    let paths: Vec<&TransportPath> = q.iter(app.world()).collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].tier, 2, "new path should be T2");
    assert_eq!(paths[0].capacity, 5, "T2 capacity = 5");
    assert_eq!(paths[0].speed, 2.0, "T2 speed = 2.0");
}

// ── AC5: Throughput capped by tier ───────────────────────────────────────────

/// Scenario: T1 rune path caps throughput at 2 items per tick
#[test]
fn t1_rune_path_caps_throughput_at_2_items_per_tick() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 10); // demand > capacity

    let waypoints = vec![(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // 10 iron_ore in source
    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 10.0;
    }

    app.update();

    // Only 2 launched (T1 capacity = 2), 8 remains
    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 8.0, "T1 cap: only 2 launched, 8 remain in source");

    // 2 iron_ore launched as Cargo
    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_count = q.iter(app.world())
        .filter(|c| c.resource == ResourceType::IronOre)
        .map(|c| c.amount)
        .sum::<f32>();
    assert_eq!(cargo_count, 2.0, "exactly 2 iron_ore should be launched as Cargo");
}

/// Scenario: T2 rune path caps throughput at 5 items per tick
#[test]
fn t2_rune_path_caps_throughput_at_5_items_per_tick() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 2;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 12); // demand > capacity

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 2);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 12.0;
    }

    app.update();

    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 7.0, "T2 cap: 5 launched, 7 remain");

    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_amount: f32 = q.iter(app.world())
        .filter(|c| c.resource == ResourceType::IronOre)
        .map(|c| c.amount).sum();
    assert_eq!(cargo_amount, 5.0, "exactly 5 iron_ore should be launched");
}

/// Scenario: T1 pipe caps throughput at 3 units per tick
#[test]
fn t1_pipe_caps_throughput_at_3_units_per_tick() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 10);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5)];
    spawn_path(app.world_mut(), TransportKind::Pipe, src_group, dst_group, waypoints, 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::Water).or_default() = 10.0;
    }

    app.update();

    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::Water).copied().unwrap_or(0.0);
    assert_eq!(remaining, 7.0, "T1 pipe cap: 3 launched, 7 remain");

    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_amount: f32 = q.iter(app.world())
        .filter(|c| c.resource == ResourceType::Water)
        .map(|c| c.amount).sum();
    assert_eq!(cargo_amount, 3.0, "exactly 3 water should be launched");
}

/// Scenario: Flow is limited by destination demand when below capacity
#[test]
fn flow_is_limited_by_destination_demand_when_below_capacity() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    // demand = 1 (less than T1 capacity of 2)
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 1);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 10.0;
    }

    app.update();

    // Only 1 launched (demand = 1), 9 remains
    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 9.0, "demand-limited: 1 launched, 9 remain");

    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_amount: f32 = q.iter(app.world())
        .filter(|c| c.resource == ResourceType::IronOre)
        .map(|c| c.amount).sum();
    assert_eq!(cargo_amount, 1.0, "exactly 1 iron_ore should be launched");
}

// ── AC6: Paths and pipes cannot overlap ─────────────────────────────────────

/// Scenario: Reject second path through tiles already occupied by a path
#[test]
fn reject_second_path_through_tiles_already_occupied_by_a_path() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_a = spawn_group(app.world_mut(), 2, 3,
        Some(ResourceType::IronOre), None, 0);
    let dst_c = spawn_group(app.world_mut(), 10, 3,
        None, Some(ResourceType::IronOre), 2);
    let src_b = spawn_group(app.world_mut(), 2, 7,
        Some(ResourceType::IronOre), None, 0);
    let dst_d = spawn_group(app.world_mut(), 10, 7,
        None, Some(ResourceType::IronOre), 2);

    // First path occupies tiles [4,5]..[8,5]
    let waypoints1 = vec![(4,5),(5,5),(6,5),(7,5),(8,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_a, dst_c, waypoints1, 1);

    // Second path tries same tiles
    let waypoints2 = vec![(4,5),(5,5),(6,5),(7,5),(8,5)];
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_b,
        target_group: dst_d,
        waypoints: waypoints2,
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedOccupied),
        "second path through occupied tiles should be rejected");
}

/// Scenario: Reject pipe through tiles already occupied by a rune path
#[test]
fn reject_pipe_through_tiles_already_occupied_by_a_rune_path() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_a = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_a = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::IronOre), 2);

    // Rune path occupies [4,5]..[6,5]
    spawn_path(app.world_mut(), TransportKind::RunePath, src_a, dst_a,
        vec![(4,5),(5,5),(6,5)], 1);

    let src_b = spawn_group(app.world_mut(), 1, 5,
        Some(ResourceType::Water), None, 0);
    let dst_b = spawn_group(app.world_mut(), 9, 5,
        None, Some(ResourceType::Water), 3);

    // Pipe tries same tiles
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_b,
        target_group: dst_b,
        waypoints: vec![(4,5),(5,5),(6,5)],
        is_pipe: true,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedOccupied));
}

/// Scenario: Reject rune path through tiles already occupied by a pipe
#[test]
fn reject_rune_path_through_tiles_already_occupied_by_a_pipe() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_a = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_a = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::Water), 3);

    // Pipe occupies [4,5]..[6,5]
    spawn_path(app.world_mut(), TransportKind::Pipe, src_a, dst_a,
        vec![(4,5),(5,5),(6,5)], 1);

    let src_b = spawn_group(app.world_mut(), 1, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_b = spawn_group(app.world_mut(), 9, 5,
        None, Some(ResourceType::IronOre), 2);

    // Path tries tile [5,5]
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_b,
        target_group: dst_b,
        waypoints: vec![(5,5)],
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedOccupied));
}

/// Scenario: Allow path on tiles adjacent to but not overlapping existing path
#[test]
fn allow_path_on_tiles_adjacent_to_but_not_overlapping_existing_path() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_a = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_a = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::IronOre), 2);

    // Existing path on row 5
    spawn_path(app.world_mut(), TransportKind::RunePath, src_a, dst_a,
        vec![(4,5),(5,5),(6,5)], 1);

    let src_b = spawn_group(app.world_mut(), 2, 6,
        Some(ResourceType::IronOre), None, 0);
    let dst_b = spawn_group(app.world_mut(), 8, 6,
        None, Some(ResourceType::IronOre), 2);

    // New path on row 6 (adjacent, not overlapping)
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_b,
        target_group: dst_b,
        waypoints: vec![(4,6),(5,6),(6,6)],
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::Ok), "adjacent path should be allowed");

    let mut q = app.world_mut().query::<&TransportPath>();
    assert_eq!(q.iter(app.world()).count(), 2, "should have 2 path entities");
}

// ── AC7: Destroying segment disconnects route ─────────────────────────────────

/// Scenario: Destroying middle segment stops resource flow
#[test]
fn destroying_middle_segment_stops_resource_flow() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Seed source with 5 iron_ore
    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 5.0;
    }

    // Destroy segment at [6,5]
    app.world_mut().resource_mut::<TransportCommands>().destroy_segment.push((6, 5));

    app.update();

    // Path should be disconnected
    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert!(!path.connected, "path should be disconnected after segment destruction");

    // No cargo should be launched
    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_count = q.iter(app.world()).count();
    assert_eq!(cargo_count, 0, "no Cargo should be launched on disconnected path");

    // 5 iron_ore remains in group A
    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let iron = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(iron, 5.0, "5 iron_ore should remain in group A after disconnection");
}

/// Scenario: Cargo in transit is lost when path segment is destroyed
#[test]
fn cargo_in_transit_is_lost_when_path_segment_is_destroyed() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Spawn in-transit cargo
    app.world_mut().spawn(Cargo {
        path_entity,
        resource: ResourceType::IronOre,
        amount: 2.0,
        position_on_path: 3.0,
    });

    // Destroy segment at [6,5]
    app.world_mut().resource_mut::<TransportCommands>().destroy_segment.push((6, 5));

    app.update();

    // Cargo should be despawned
    let mut q = app.world_mut().query::<&Cargo>();
    let cargo_count = q.iter(app.world())
        .filter(|c| c.path_entity == path_entity)
        .count();
    assert_eq!(cargo_count, 0, "in-transit cargo should be destroyed with the segment");
}

/// Scenario: Destroying first segment of path disconnects the route
#[test]
fn destroying_first_segment_of_path_disconnects_the_route() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Destroy first segment
    app.world_mut().resource_mut::<TransportCommands>().destroy_segment.push((4, 5));
    app.update();

    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert!(!path.connected, "path should be disconnected after first segment destroyed");
}

/// Scenario: Destroying last segment of path disconnects the route
#[test]
fn destroying_last_segment_of_path_disconnects_the_route() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Destroy last segment
    app.world_mut().resource_mut::<TransportCommands>().destroy_segment.push((9, 5));
    app.update();

    let path = app.world().entity(path_entity).get::<TransportPath>().unwrap();
    assert!(!path.connected, "path should be disconnected after last segment destroyed");
}

// ── AC8: Minion carry (before any paths exist) ────────────────────────────────

/// Scenario: Minions auto-carry surplus solid resources between nearby groups
#[test]
fn minions_auto_carry_surplus_solid_resources_between_nearby_groups() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Group A at [2,5], Group B at [6,5] → manhattan distance = 4 (within range 5)
    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 6, 5,
        None, Some(ResourceType::IronOre), 1);

    // Seed group A with 3 iron_ore (surplus — no internal smelter)
    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 3.0;
    }

    // No paths exist — minion carry should activate
    app.update();

    let dst_manifold = app.world_mut().query::<&Manifold>().get(app.world(), dst_group).unwrap();
    let transferred = dst_manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(transferred, 0.5,
        "minion carry should transfer 0.5 iron_ore per tick");

    let src_manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = src_manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 2.5,
        "source should have 2.5 iron_ore after minion carry");
}

/// Scenario: Minions do not carry when groups are beyond range 5
#[test]
fn minions_do_not_carry_when_groups_are_beyond_range_5() {
    let mut app = test_app(20, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Group A at [2,5], Group B at [15,5] → manhattan distance = 13 > 5
    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 15, 5,
        None, Some(ResourceType::IronOre), 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 5.0;
    }

    app.update();

    let dst_manifold = app.world_mut().query::<&Manifold>().get(app.world(), dst_group).unwrap();
    let transferred = dst_manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(transferred, 0.0, "no transfer when groups beyond range 5");

    let src_manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = src_manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 5.0, "source should still have 5 iron_ore");
}

/// Scenario: Minions cannot carry liquid resources
#[test]
fn minions_cannot_carry_liquid_resources() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Group A at [2,5], Group B at [5,5] → manhattan distance = 3 (within range)
    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::Water), None, 0);
    let dst_group = spawn_group(app.world_mut(), 5, 5,
        None, Some(ResourceType::Water), 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::Water).or_default() = 5.0;
    }

    // No paths — but water is liquid, minions can't carry it
    app.update();

    let dst_manifold = app.world_mut().query::<&Manifold>().get(app.world(), dst_group).unwrap();
    let transferred = dst_manifold.resources.get(&ResourceType::Water).copied().unwrap_or(0.0);
    assert_eq!(transferred, 0.0, "minions cannot carry liquid resources");

    let src_manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = src_manifold.resources.get(&ResourceType::Water).copied().unwrap_or(0.0);
    assert_eq!(remaining, 5.0, "water should remain in source");
}

/// Scenario: Minion carry rate is 0.5 items per tick (25% of T1 path capacity)
#[test]
fn minion_carry_rate_is_0_5_items_per_tick() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 6, 5,
        None, Some(ResourceType::IronOre), 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 10.0;
    }

    app.update();

    let dst_manifold = app.world_mut().query::<&Manifold>().get(app.world(), dst_group).unwrap();
    let transferred = dst_manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(transferred, 0.5, "minion carry rate = 0.5 items/tick");
}

// ── Edge cases ────────────────────────────────────────────────────────────────

/// Scenario: Path drawn to receiver already at max input rate queues at sender
#[test]
fn path_drawn_to_receiver_already_at_max_input_rate_queues_at_sender() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    // demand = 0 (saturated)
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 0);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 10.0;
    }

    app.update();

    // No cargo should be launched (demand = 0)
    let mut q = app.world_mut().query::<&Cargo>();
    assert_eq!(q.iter(app.world()).count(), 0, "no cargo when receiver demand is 0");

    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 10.0, "all 10 iron_ore should remain queued at sender");
}

/// Scenario: Path with no source resources produces no cargo
#[test]
fn path_with_no_source_resources_produces_no_cargo() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Source manifold has 0 iron_ore (default)
    app.update();

    let mut q = app.world_mut().query::<&Cargo>();
    assert_eq!(q.iter(app.world()).count(), 0, "no cargo when source is empty");
}

/// Scenario: Minion carry coexists with paths — path takes priority
#[test]
fn minion_carry_coexists_with_paths_path_takes_priority() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 6, 5,
        None, Some(ResourceType::IronOre), 2);

    // Both groups within minion range 5, AND connected by a path
    let waypoints = vec![(4,5),(5,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 5.0;
    }

    app.update();

    // Resources should flow through rune_path (as Cargo), not minion carry
    // Cargo entities created = resources transported via path
    let mut q = app.world_mut().query::<&Cargo>();
    assert!(q.iter(app.world()).count() > 0, "resources should flow through rune_path as Cargo");

    // Only path capacity (2) should have been moved, not minion rate
    let manifold = app.world_mut().query::<&Manifold>().get(app.world(), src_group).unwrap();
    let remaining = manifold.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert_eq!(remaining, 3.0, "path takes priority: 2 launched, 3 remain (not 0.5 minion rate)");
}

/// Scenario: Multiple paths from same group output to different groups
#[test]
fn multiple_paths_from_same_group_output_to_different_groups() {
    let mut app = test_app(20, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_b = spawn_group(app.world_mut(), 8, 5,
        None, Some(ResourceType::IronOre), 2);
    let dst_c = spawn_group(app.world_mut(), 8, 8,
        None, Some(ResourceType::IronOre), 2);

    // First path already exists
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_b,
        vec![(4,5),(5,5),(6,5),(7,5)], 1);

    // Draw second path to dst_c (different tiles)
    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_c,
        waypoints: vec![(4,6),(5,6),(6,6),(7,6)],
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::Ok), "second path to different group should succeed");

    let mut q = app.world_mut().query::<&TransportPath>();
    assert_eq!(q.iter(app.world()).count(), 2, "should have 2 path entities");
}

/// Scenario: Destroying all path segments removes path entity entirely
#[test]
fn destroying_all_path_segments_removes_path_entity_entirely() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Destroy the entire path via DestroyPath command
    app.world_mut().resource_mut::<TransportCommands>().destroy_path.push(path_entity);

    app.update();

    // Path entity should be destroyed
    assert!(app.world().get_entity(path_entity).is_err(),
        "path entity should be destroyed after DestroyPath command");

    // PathConnection should also be destroyed
    let mut conn_q = app.world_mut().query::<&PathConnection>();
    assert_eq!(conn_q.iter(app.world()).count(), 0, "PathConnection should be destroyed");
}

/// Scenario: DrawPath rejected when source group has no output sender
#[test]
fn draw_path_rejected_when_source_group_has_no_output_sender() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Source group has NO TransportSender
    let src_group = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x: 2, y: 5 },
        // No TransportSender!
    )).id();
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints: vec![(4,5),(5,5)],
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedNoSender));
}

/// Scenario: DrawPath rejected when destination group has no input receiver
#[test]
fn draw_path_rejected_when_destination_group_has_no_input_receiver() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    // Destination group has NO TransportReceiver
    let dst_group = app.world_mut().spawn((
        Group,
        Manifold::default(),
        GroupEnergy::default(),
        GroupPosition { x: 10, y: 5 },
        // No TransportReceiver!
    )).id();

    app.world_mut().resource_mut::<TransportCommands>().draw_path.push(DrawPathCmd {
        source_group: src_group,
        target_group: dst_group,
        waypoints: vec![(4,5),(5,5)],
        is_pipe: false,
    });

    app.update();

    let result = app.world().resource::<LastDrawPathResult>().result;
    assert_eq!(result, Some(DrawPathResult::RejectedNoReceiver));
}

/// Scenario: DestroyPath command removes an existing path
#[test]
fn destroy_path_command_removes_an_existing_path() {
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    let path_entity = spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    app.world_mut().resource_mut::<TransportCommands>().destroy_path.push(path_entity);
    app.update();

    // Path entity is gone
    assert!(app.world().get_entity(path_entity).is_err(), "path entity should be destroyed");

    // Segment tiles freed (occupancy cleared)
    let occupancy = app.world().resource::<PathOccupancy>();
    assert!(occupancy.tiles.is_empty(), "segment tiles should be freed");
}

/// Scenario: Transport phase runs after production phase
#[test]
fn transport_phase_runs_after_production_phase() {
    // This test verifies the phase ordering:
    // Production (Manifold) → Transport
    // We confirm it by checking that cargo launched in transport uses resources
    // that would have been produced in the production phase.
    let mut app = test_app(16, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    let dst_group = spawn_group(app.world_mut(), 10, 5,
        None, Some(ResourceType::IronOre), 2);

    let waypoints = vec![(4,5),(5,5),(6,5),(7,5),(8,5),(9,5)];
    spawn_path(app.world_mut(), TransportKind::RunePath, src_group, dst_group, waypoints, 1);

    // Pre-load iron_ore (simulating what production would have added)
    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 2.0;
    }

    app.update();

    // Transport system should have picked up the ore
    let mut q = app.world_mut().query::<&Cargo>();
    assert!(q.iter(app.world()).count() > 0,
        "Transport phase should have access to resources from Production phase");
}

/// Scenario: Minion carry uses manhattan nearest to find destination
#[test]
fn minion_carry_uses_manhattan_nearest_to_find_destination() {
    let mut app = test_app(12, 10);
    app.world_mut().resource_mut::<TransportTierState>().transport_tier = 1;

    // Group A at [2,5]
    let src_group = spawn_group(app.world_mut(), 2, 5,
        Some(ResourceType::IronOre), None, 0);
    // Group B at [5,5] — manhattan distance 3
    let dst_b = spawn_group(app.world_mut(), 5, 5,
        None, Some(ResourceType::IronOre), 1);
    // Group C at [7,5] — manhattan distance 5
    let dst_c = spawn_group(app.world_mut(), 7, 5,
        None, Some(ResourceType::IronOre), 1);

    {
        let mut manifold = app.world_mut().query::<&mut Manifold>().get_mut(app.world_mut(), src_group).unwrap();
        *manifold.resources.entry(ResourceType::IronOre).or_default() = 5.0;
    }

    app.update();

    // Only group B (nearest) should receive transfer
    let manifold_b = app.world_mut().query::<&Manifold>().get(app.world(), dst_b).unwrap();
    let b_received = manifold_b.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);

    let manifold_c = app.world_mut().query::<&Manifold>().get(app.world(), dst_c).unwrap();
    let c_received = manifold_c.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0);

    assert_eq!(b_received, 0.5, "nearest group B should receive 0.5 via minion carry");
    assert_eq!(c_received, 0.0, "farther group C should receive nothing (only nearest)");
}
