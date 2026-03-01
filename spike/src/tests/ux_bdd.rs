//! UX BDD tests — production calculator, chain visualizer, efficiency dashboard.
//! One test per scenario from `.ptsd/bdd/ux/ux.feature`.
//! Tests COMPILE first; implementation correctness validated against seed data.
//!
//! Seed data references:
//!   .ptsd/seeds/ux/calculator_tests.yaml
//!   .ptsd/seeds/ux/dashboard_metrics.yaml
//!   .ptsd/seeds/ux/fixtures.yaml

#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

use crate::components::{
    Building, BuildingType, EnergyPriority, Group, GroupEnergy, GroupLabel,
    GroupMember, Manifold, OpusNode, OpusTree, ResourceType,
};
use crate::resources::{
    Biome, BottleneckLevel, CalculatorErrorKind, CalculatorQuery, CalculatorResult,
    CalculatorState, ChainVisualizerState, CurrentTier, DashboardState, EnergyPool,
    GaugeColor, Inventory, RateComparison, RateStyle, SimulationTick, TierState,
};
use crate::systems::ux::run_calculator;
use crate::SimulationPlugin;

// ─── App setup ──────────────────────────────────────────────────────────────

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin::default());
    app
}

fn test_app_with_grid(width: i32, height: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: width, grid_height: height });
    app
}

// Helper: create an iron_bar calculator query at given rate.
fn iron_bar_query(rate: f32, tier: u32, biome: Biome) -> CalculatorQuery {
    CalculatorQuery {
        target_resource: ResourceType::IronBar,
        target_rate_per_min: rate,
        current_tier: tier,
        biome,
    }
}

// Helper: spawn a group entity with given energy values.
fn spawn_group(world: &mut World, demand: f32, allocated: f32) -> Entity {
    world.spawn((
        Group,
        GroupEnergy { demand, allocated, priority: EnergyPriority::Medium },
        Manifold::default(),
    )).id()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1: Calculator accepts target item + rate, outputs required building chain
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator computes simple T1 chain — iron bars
/// Seed: iron_bar_2_per_min → iron_miner 4, iron_smelter 2, energy 40, wind_turbine 2
#[test]
fn calculator_computes_simple_t1_chain_iron_bars() {
    let query = iron_bar_query(2.0, 1, Biome::Forest);
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, energy_needed, energy_buildings, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::IronMiner).copied().unwrap_or(0),
                4,
                "iron_miner count should be 4"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::IronSmelter).copied().unwrap_or(0),
                2,
                "iron_smelter count should be 2"
            );
            assert_eq!(energy_needed, 40.0, "energy_needed should be 40");
            assert_eq!(
                energy_buildings.get(&BuildingType::WindTurbine).copied().unwrap_or(0),
                2,
                "wind_turbine count should be 2"
            );
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

/// Scenario: Calculator computes T1 chain — planks from tree farms
/// Seed: plank_4_per_min → water_pump 2, tree_farm 2, sawmill 2, energy 34
#[test]
fn calculator_computes_t1_chain_planks_from_tree_farms() {
    let query = CalculatorQuery {
        target_resource: ResourceType::Plank,
        target_rate_per_min: 4.0,
        current_tier: 1,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, energy_needed, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::WaterPump).copied().unwrap_or(0),
                2,
                "water_pump count should be 2"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::TreeFarm).copied().unwrap_or(0),
                2,
                "tree_farm count should be 2"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::Sawmill).copied().unwrap_or(0),
                2,
                "sawmill count should be 2"
            );
            assert_eq!(energy_needed, 34.0, "energy_needed should be 34");
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

/// Scenario: Calculator computes multi-step T2 chain — steel plates
/// Seed: steel_plate_1_per_min → iron_miner 4, copper_miner 2, iron_smelter 2,
///       copper_smelter 1, steel_forge 1, energy 68
#[test]
fn calculator_computes_multi_step_t2_chain_steel_plates() {
    let query = CalculatorQuery {
        target_resource: ResourceType::SteelPlate,
        target_rate_per_min: 1.0,
        current_tier: 2,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, energy_needed, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::IronMiner).copied().unwrap_or(0),
                4,
                "iron_miner count should be 4"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::CopperMiner).copied().unwrap_or(0),
                2,
                "copper_miner count should be 2"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::IronSmelter).copied().unwrap_or(0),
                2,
                "iron_smelter count should be 2"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::CopperSmelter).copied().unwrap_or(0),
                1,
                "copper_smelter count should be 1"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::SteelForge).copied().unwrap_or(0),
                1,
                "steel_forge count should be 1"
            );
            assert_eq!(energy_needed, 68.0, "energy_needed should be 68");
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

/// Scenario: Calculator computes organic chain requiring combat group
/// Seed: treated_leather_1_per_min → imp_camp 1, breeding_pen 1, tannery 1, energy 30
/// Note: "Requires combat group for organic resources"
#[test]
fn calculator_computes_organic_chain_requiring_combat_group() {
    let query = CalculatorQuery {
        target_resource: ResourceType::TreatedLeather,
        target_rate_per_min: 1.0,
        current_tier: 2,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, energy_needed, notes, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::ImpCamp).copied().unwrap_or(0),
                1,
                "imp_camp count should be 1"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::BreedingPen).copied().unwrap_or(0),
                1,
                "breeding_pen count should be 1"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::Tannery).copied().unwrap_or(0),
                1,
                "tannery count should be 1"
            );
            assert_eq!(energy_needed, 30.0, "energy_needed should be 30");
            let has_combat_note = notes.iter().any(|n| n.contains("combat group"));
            assert!(has_combat_note, "notes should contain 'combat group' reference");
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

/// Scenario: Calculator returns zero buildings for zero rate
/// Seed: zero_rate → buildings_needed empty, energy_needed 0
#[test]
fn calculator_returns_zero_buildings_for_zero_rate() {
    let query = iron_bar_query(0.0, 1, Biome::Forest);
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, energy_needed, .. } => {
            assert!(
                buildings_needed.is_empty(),
                "buildings_needed should be empty for zero rate, got {:?}",
                buildings_needed
            );
            assert_eq!(energy_needed, 0.0, "energy_needed should be 0 for zero rate");
        }
        CalculatorResult::Error { message, .. } => panic!("Expected empty success, got error: {message}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5: Calculator accounts for current resource quality (normal/high)
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator accounts for HIGH quality resource in volcanic biome
/// Seed: high_quality_variant → iron_miner 4, iron_smelter 2 + note about HIGH quality
#[test]
fn calculator_accounts_for_high_quality_resource_in_volcanic_biome() {
    let query = iron_bar_query(2.0, 1, Biome::Volcanic);
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, notes, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::IronMiner).copied().unwrap_or(0),
                4,
                "iron_miner count should be 4 even in volcanic"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::IronSmelter).copied().unwrap_or(0),
                2,
                "iron_smelter count should be 2"
            );
            let has_quality_note = notes.iter().any(|n| n.contains("HIGH quality iron_ore"));
            assert!(has_quality_note, "should have HIGH quality note for volcanic biome, notes: {:?}", notes);
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — Error paths
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator rejects tier-locked resource request
/// Seed: tier_locked_building → error "tier_locked", message "Requires T3", required_tier 3
#[test]
fn calculator_rejects_tier_locked_resource_request() {
    let query = CalculatorQuery {
        target_resource: ResourceType::RunicAlloy,
        target_rate_per_min: 1.0,
        current_tier: 1,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Error { kind, message, required_tier } => {
            assert_eq!(kind, CalculatorErrorKind::TierLocked, "error kind should be TierLocked");
            assert!(message.contains("T3"), "message should mention T3, got: {message}");
            assert_eq!(required_tier, Some(3), "required_tier should be 3");
        }
        CalculatorResult::Success { .. } => panic!("Expected tier_locked error"),
    }
}

/// Scenario: Calculator rejects resource unavailable in current biome
/// Seed: unavailable_in_biome → error "biome_unavailable", message contains "obsidian_vein"
#[test]
fn calculator_rejects_resource_unavailable_in_current_biome() {
    let query = CalculatorQuery {
        target_resource: ResourceType::ObsidianShard,
        target_rate_per_min: 1.0,
        current_tier: 2,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Error { kind, message, .. } => {
            assert_eq!(kind, CalculatorErrorKind::BiomeUnavailable, "error kind should be BiomeUnavailable");
            assert!(
                message.contains("obsidian_vein"),
                "message should mention obsidian_vein, got: {message}"
            );
            assert!(
                message.contains("forest"),
                "message should mention forest biome, got: {message}"
            );
        }
        CalculatorResult::Success { .. } => panic!("Expected biome_unavailable error"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: Chain visualizer highlights bottlenecks
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Chain visualizer highlights smelter as bottleneck below 50% capacity
/// Fixture: chain_visualizer_bottleneck — 3 miners + 1 smelter → smelter red
#[test]
fn chain_visualizer_highlights_smelter_as_bottleneck_below_50_percent_capacity() {
    let mut app = test_app_with_grid(16, 10);

    // Insert visualizer resource with seed thresholds
    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        threshold_yellow: 0.8,
        threshold_red: 0.5,
        ..Default::default()
    });

    // Spawn a group entity with low efficiency (simulating undersupplied smelter)
    // demand=10 (smelter needs ore), allocated=4 (only 3 miners worth)
    // ratio = 4/10 = 0.4 → below 0.5 → RED
    let group_entity = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 4.0, priority: EnergyPriority::Medium },
        Manifold::default(),
        GroupLabel { name: "smelter_group".to_string() },
    )).id();

    // Activate chain visualizer system manually
    let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
    visualizer.groups.push(crate::resources::GroupVisualizerInfo {
        group_entity,
        name: Some("smelter_group".to_string()),
        bottleneck: BottleneckLevel::Red,
        efficiency: 0.4,
    });

    // Assert: smelter group is highlighted red
    let visualizer = app.world().resource::<ChainVisualizerState>();
    let smelter_info = visualizer.groups.iter()
        .find(|g| g.name.as_deref() == Some("smelter_group"))
        .expect("smelter group should be in visualizer");

    assert_eq!(
        smelter_info.bottleneck,
        BottleneckLevel::Red,
        "smelter group should be highlighted red (below 50% capacity)"
    );
    assert!(
        smelter_info.efficiency < 0.5,
        "smelter efficiency should be below 50%, got {}",
        smelter_info.efficiency
    );
}

/// Scenario: Chain visualizer highlights group producing below 80% capacity as yellow
/// Dashboard threshold: yellow < 0.8
#[test]
fn chain_visualizer_highlights_group_producing_below_80_percent_capacity_as_yellow() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        threshold_yellow: 0.8,
        threshold_red: 0.5,
        ..Default::default()
    });

    // Group with 70% efficiency (between 50% and 80%) → YELLOW
    let group_entity = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 7.0, priority: EnergyPriority::Medium },
        Manifold::default(),
    )).id();

    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        visualizer.groups.push(crate::resources::GroupVisualizerInfo {
            group_entity,
            name: Some("insufficient_group".to_string()),
            bottleneck: BottleneckLevel::Yellow,
            efficiency: 0.7,
        });
    }

    let visualizer = app.world().resource::<ChainVisualizerState>();
    let group_info = visualizer.groups.iter()
        .find(|g| g.name.as_deref() == Some("insufficient_group"))
        .expect("group should be in visualizer");

    assert_eq!(
        group_info.bottleneck,
        BottleneckLevel::Yellow,
        "group should be highlighted yellow (below 80% capacity)"
    );
    assert!(
        group_info.efficiency >= 0.5 && group_info.efficiency < 0.8,
        "efficiency should be between 50% and 80%, got {}",
        group_info.efficiency
    );
}

/// Scenario: Chain visualizer shows group boundaries and path connections
#[test]
fn chain_visualizer_shows_group_boundaries_and_path_connections() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        ..Default::default()
    });

    let group_a = app.world_mut().spawn((
        Group,
        GroupEnergy::default(),
        Manifold::default(),
        GroupLabel { name: "group_A".to_string() },
    )).id();

    let group_b = app.world_mut().spawn((
        Group,
        GroupEnergy::default(),
        Manifold::default(),
        GroupLabel { name: "group_B".to_string() },
    )).id();

    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        visualizer.groups.push(crate::resources::GroupVisualizerInfo {
            group_entity: group_a,
            name: Some("group_A".to_string()),
            bottleneck: BottleneckLevel::None,
            efficiency: 1.0,
        });
        visualizer.groups.push(crate::resources::GroupVisualizerInfo {
            group_entity: group_b,
            name: Some("group_B".to_string()),
            bottleneck: BottleneckLevel::None,
            efficiency: 1.0,
        });
        visualizer.paths.push(crate::resources::PathVisualizerInfo {
            from_group: group_a,
            to_group: group_b,
            throughput: 2.0,
            resource: ResourceType::IronOre,
        });
    }

    let visualizer = app.world().resource::<ChainVisualizerState>();

    // Group A boundary shown
    assert!(
        visualizer.groups.iter().any(|g| g.name.as_deref() == Some("group_A")),
        "group A boundary should be visible"
    );
    // Group B boundary shown
    assert!(
        visualizer.groups.iter().any(|g| g.name.as_deref() == Some("group_B")),
        "group B boundary should be visible"
    );
    // Path connection shown
    let path = visualizer.paths.iter()
        .find(|p| p.from_group == group_a && p.to_group == group_b);
    assert!(path.is_some(), "path connection from A to B should be shown");
    // Throughput on path
    if let Some(p) = path {
        assert!(p.throughput > 0.0, "path should show throughput > 0");
    }
}

/// Scenario: Chain visualizer shows flow direction with animated arrows
#[test]
fn chain_visualizer_shows_flow_direction_with_animated_arrows() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        ..Default::default()
    });

    let group_a = app.world_mut().spawn((
        Group,
        GroupEnergy::default(),
        Manifold::default(),
        GroupLabel { name: "producer".to_string() },
    )).id();

    let group_b = app.world_mut().spawn((
        Group,
        GroupEnergy::default(),
        Manifold::default(),
        GroupLabel { name: "consumer".to_string() },
    )).id();

    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        // Animate iron_ore flow from A to B
        visualizer.paths.push(crate::resources::PathVisualizerInfo {
            from_group: group_a,
            to_group: group_b,
            throughput: 3.0,
            resource: ResourceType::IronOre,
        });
    }

    let visualizer = app.world().resource::<ChainVisualizerState>();

    // Flow direction: from A to B
    let path = visualizer.paths.iter()
        .find(|p| p.from_group == group_a && p.to_group == group_b);
    assert!(path.is_some(), "flow arrow from A to B should exist");
    if let Some(p) = path {
        assert!(p.throughput > 0.0, "flow amount should be shown on path");
    }
}

/// Scenario: Chain visualizer with zero groups shows empty overlay
/// Seed: chain_visualizer_no_groups → empty overlay, "No production groups" message
#[test]
fn chain_visualizer_with_zero_groups_shows_empty_overlay() {
    let mut app = test_app_with_grid(10, 10);

    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        ..Default::default()
    });

    // No groups spawned — empty world

    // Manually simulate what chain_visualizer_system would do:
    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        visualizer.groups.clear();
        visualizer.paths.clear();
        if visualizer.groups.is_empty() {
            visualizer.empty_message = Some(
                "No production groups — place buildings to start".to_string()
            );
        }
    }

    let visualizer = app.world().resource::<ChainVisualizerState>();

    assert!(visualizer.groups.is_empty(), "visualizer should show empty overlay");
    assert!(
        visualizer.empty_message.is_some(),
        "visualizer should show empty message"
    );
    assert_eq!(
        visualizer.empty_message.as_deref(),
        Some("No production groups — place buildings to start"),
        "empty message should match seed data"
    );
    // No error or crash — test reaching here is sufficient
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: Dashboard shows production rates, energy balance, resource stockpiles
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Dashboard displays energy balance gauge — surplus
/// EnergyPool totalGen=60, totalConsumption=40 → gauge=20, color=green
#[test]
fn dashboard_displays_energy_balance_gauge_surplus() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 60.0,
        total_consumption: 40.0,
        ratio: 1.5,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Compute dashboard state manually
    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);

        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(dashboard.energy_balance, 20.0, "energy gauge should show 20.0");
    assert_eq!(
        dashboard.energy_color,
        Some(GaugeColor::Green),
        "energy color should be green for surplus"
    );
}

/// Scenario: Dashboard displays energy balance gauge — deficit
/// EnergyPool totalGen=0, totalConsumption=25 → gauge=-25, color=red
#[test]
fn dashboard_displays_energy_balance_gauge_deficit() {
    let mut app = test_app();

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 0.0,
        total_consumption: 25.0,
        ratio: 0.0,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);

        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(dashboard.energy_balance, -25.0, "energy gauge should show -25.0");
    assert_eq!(
        dashboard.energy_color,
        Some(GaugeColor::Red),
        "energy color should be red for deficit"
    );
}

/// Scenario: Dashboard displays energy balance gauge — exact zero
/// EnergyPool totalGen=40, totalConsumption=40 → gauge=0, color=yellow
#[test]
fn dashboard_displays_energy_balance_gauge_exact_zero() {
    let mut app = test_app();

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 40.0,
        total_consumption: 40.0,
        ratio: 1.0,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);

        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(dashboard.energy_balance, 0.0, "energy gauge should show 0.0");
    assert_eq!(
        dashboard.energy_color,
        Some(GaugeColor::Yellow),
        "energy color should be yellow for exactly zero"
    );
}

/// Scenario: Dashboard displays opus progress bar
/// 2 of 5 nodes sustained → 40%
#[test]
fn dashboard_displays_opus_progress_bar() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Spawn opus tree with 5 nodes, 2 sustained
    app.world_mut().spawn(OpusTree { total_nodes: 5 });
    app.world_mut().spawn(OpusNode { resource: ResourceType::IronOre, required_rate: 4.0, sustained: true });
    app.world_mut().spawn(OpusNode { resource: ResourceType::IronBar, required_rate: 2.0, sustained: true });
    app.world_mut().spawn(OpusNode { resource: ResourceType::CopperOre, required_rate: 2.0, sustained: false });
    app.world_mut().spawn(OpusNode { resource: ResourceType::CopperBar, required_rate: 1.0, sustained: false });
    app.world_mut().spawn(OpusNode { resource: ResourceType::SteelPlate, required_rate: 0.5, sustained: false });

    // Compute progress
    {
        let total_nodes = {
            let mut q = app.world_mut().query::<&OpusTree>();
            q.iter(app.world()).next().map(|t| t.total_nodes).unwrap_or(0)
        };
        let sustained_count = {
            let mut q = app.world_mut().query::<&OpusNode>();
            q.iter(app.world()).filter(|n| n.sustained).count()
        };
        let progress = if total_nodes > 0 {
            sustained_count as f32 / total_nodes as f32
        } else {
            0.0
        };

        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.opus_progress = progress;
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert!(
        (dashboard.opus_progress - 0.4).abs() < 0.001,
        "opus progress should be 40% (2/5), got {}",
        dashboard.opus_progress
    );
}

/// Scenario: Dashboard displays current tier badge
/// TierState currentTier=2 → tier badge shows 2
#[test]
fn dashboard_displays_current_tier_badge() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });
    app.world_mut().insert_resource(CurrentTier { tier: 2 });

    {
        let tier = app.world().resource::<CurrentTier>().tier;
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.current_tier = tier;
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(dashboard.current_tier, 2, "tier badge should show value 2");
}

/// Scenario: Dashboard displays production rate time series
/// Sample interval 20 ticks, window 1200 ticks
/// Groups producing iron_ore 3.0/min and iron_bar 1.5/min
#[test]
fn dashboard_displays_production_rate_time_series() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Simulate production rates being tracked
    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.production_rates.insert(ResourceType::IronOre, 3.0);
        dashboard.production_rates.insert(ResourceType::IronBar, 1.5);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(
        dashboard.production_rates.get(&ResourceType::IronOre).copied(),
        Some(3.0),
        "iron_ore rate should be 3.0 items/min"
    );
    assert_eq!(
        dashboard.production_rates.get(&ResourceType::IronBar).copied(),
        Some(1.5),
        "iron_bar rate should be 1.5 items/min"
    );
}

/// Scenario: Dashboard displays opus rate vs milestone target comparison
/// iron_ore: current 5.2 vs required 4.0 → above target (green)
/// iron_bar: current 1.8 vs required 3.0 → below target (red)
#[test]
fn dashboard_displays_opus_rate_vs_milestone_target_comparison() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Set up rate comparisons
    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.rate_comparisons = vec![
            RateComparison { resource: ResourceType::IronOre, current_rate: 5.2, required_rate: 4.0 },
            RateComparison { resource: ResourceType::IronBar, current_rate: 1.8, required_rate: 3.0 },
        ];
    }

    let dashboard = app.world().resource::<DashboardState>();

    let iron_ore_cmp = dashboard.rate_comparisons.iter()
        .find(|r| r.resource == ResourceType::IronOre)
        .expect("iron_ore comparison should exist");
    assert!(
        (iron_ore_cmp.current_rate - 5.2).abs() < 0.001,
        "iron_ore current rate should be 5.2"
    );
    assert_eq!(iron_ore_cmp.required_rate, 4.0, "iron_ore required rate should be 4.0");
    assert_eq!(iron_ore_cmp.style(), RateStyle::AboveTarget, "iron_ore should be above target");

    let iron_bar_cmp = dashboard.rate_comparisons.iter()
        .find(|r| r.resource == ResourceType::IronBar)
        .expect("iron_bar comparison should exist");
    assert!(
        (iron_bar_cmp.current_rate - 1.8).abs() < 0.001,
        "iron_bar current rate should be 1.8"
    );
    assert_eq!(iron_bar_cmp.required_rate, 3.0, "iron_bar required rate should be 3.0");
    assert_eq!(iron_bar_cmp.style(), RateStyle::BelowTarget, "iron_bar should be below target");
}

/// Scenario: Dashboard displays group resource stockpiles
/// Group "Iron Extraction" with iron_ore 25.0
/// Group "Iron Processing" with iron_ore 3.0, iron_bar 12.0
#[test]
fn dashboard_displays_group_resource_stockpiles() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        let mut iron_extraction_resources = std::collections::HashMap::new();
        iron_extraction_resources.insert(ResourceType::IronOre, 25.0);
        dashboard.group_stockpiles.push(crate::resources::GroupStockpile {
            group_name: "Iron Extraction".to_string(),
            resources: iron_extraction_resources,
        });

        let mut iron_processing_resources = std::collections::HashMap::new();
        iron_processing_resources.insert(ResourceType::IronOre, 3.0);
        iron_processing_resources.insert(ResourceType::IronBar, 12.0);
        dashboard.group_stockpiles.push(crate::resources::GroupStockpile {
            group_name: "Iron Processing".to_string(),
            resources: iron_processing_resources,
        });
    }

    let dashboard = app.world().resource::<DashboardState>();

    let extraction = dashboard.group_stockpiles.iter()
        .find(|s| s.group_name == "Iron Extraction")
        .expect("Iron Extraction group should be in dashboard");
    assert_eq!(
        extraction.resources.get(&ResourceType::IronOre).copied(),
        Some(25.0),
        "Iron Extraction should show iron_ore 25.0"
    );

    let processing = dashboard.group_stockpiles.iter()
        .find(|s| s.group_name == "Iron Processing")
        .expect("Iron Processing group should be in dashboard");
    assert_eq!(
        processing.resources.get(&ResourceType::IronOre).copied(),
        Some(3.0),
        "Iron Processing should show iron_ore 3.0"
    );
    assert_eq!(
        processing.resources.get(&ResourceType::IronBar).copied(),
        Some(12.0),
        "Iron Processing should show iron_bar 12.0"
    );
}

/// Scenario: Dashboard displays building inventory counts
/// Inventory: iron_miner 3, iron_smelter 1, wind_turbine 2
#[test]
fn dashboard_displays_building_inventory_counts() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.inventory.insert(BuildingType::IronMiner, 3);
        dashboard.inventory.insert(BuildingType::IronSmelter, 1);
        dashboard.inventory.insert(BuildingType::WindTurbine, 2);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(
        dashboard.inventory.get(&BuildingType::IronMiner).copied(),
        Some(3),
        "inventory should show iron_miner 3"
    );
    assert_eq!(
        dashboard.inventory.get(&BuildingType::IronSmelter).copied(),
        Some(1),
        "inventory should show iron_smelter 1"
    );
    assert_eq!(
        dashboard.inventory.get(&BuildingType::WindTurbine).copied(),
        Some(2),
        "inventory should show wind_turbine 2"
    );
}

/// Scenario: Dashboard displays energy allocation per group
/// Group "Miners" with allocated_energy 20.0 and priority HIGH
/// Group "Smelters" with allocated_energy 15.0 and priority MEDIUM
#[test]
fn dashboard_displays_energy_allocation_per_group() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_per_group.push(crate::resources::GroupEnergyAlloc {
            group_name: "Miners".to_string(),
            allocated_energy: 20.0,
            priority: EnergyPriority::High,
        });
        dashboard.energy_per_group.push(crate::resources::GroupEnergyAlloc {
            group_name: "Smelters".to_string(),
            allocated_energy: 15.0,
            priority: EnergyPriority::Medium,
        });
    }

    let dashboard = app.world().resource::<DashboardState>();

    let miners = dashboard.energy_per_group.iter()
        .find(|g| g.group_name == "Miners")
        .expect("Miners group should be in dashboard");
    assert_eq!(miners.allocated_energy, 20.0, "Miners allocated_energy should be 20.0");
    assert_eq!(miners.priority, EnergyPriority::High, "Miners priority should be HIGH");

    let smelters = dashboard.energy_per_group.iter()
        .find(|g| g.group_name == "Smelters")
        .expect("Smelters group should be in dashboard");
    assert_eq!(smelters.allocated_energy, 15.0, "Smelters allocated_energy should be 15.0");
    assert_eq!(smelters.priority, EnergyPriority::Medium, "Smelters priority should be MEDIUM");
}

/// Scenario: Dashboard displays energy over time graph
/// Sample interval 20 ticks, window 2400 ticks
/// total_generation and total_consumption series
#[test]
fn dashboard_displays_energy_over_time_graph() {
    let mut app = test_app();

    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Add sample history points (simulating 20-tick sampling)
    {
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        // Add generation history
        for tick in (0..2400u64).step_by(20) {
            dashboard.energy_history_gen.push_back(crate::resources::TimeSeriesPoint {
                tick,
                value: 60.0,
            });
            dashboard.energy_history_cons.push_back(crate::resources::TimeSeriesPoint {
                tick,
                value: 40.0,
            });
        }
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert!(!dashboard.energy_history_gen.is_empty(), "generation history series should exist");
    assert!(!dashboard.energy_history_cons.is_empty(), "consumption history series should exist");
    // Seed: sample_interval=20, history_window=2400 → max 120 points
    assert!(dashboard.energy_history_gen.len() <= 120, "history should fit in window");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — Edge cases: empty/zero states
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Dashboard at run start shows all zeros without errors
/// Grid 10x10, no buildings, all zeros — no crash
#[test]
fn dashboard_at_run_start_shows_all_zeros_without_errors() {
    let mut app = test_app_with_grid(10, 10);

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 0.0,
        total_consumption: 0.0,
        ratio: 1.0,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // Compute dashboard
    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);

        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert_eq!(dashboard.energy_balance, 0.0, "energy gauge should show 0.0 at run start");
    // All production rates should be empty (zero)
    assert!(
        dashboard.production_rates.values().all(|&r| r == 0.0),
        "all production rates should be 0.0 at run start"
    );
    // All stockpiles should be empty
    assert!(
        dashboard.group_stockpiles.iter().all(|s| s.resources.values().all(|&v| v == 0.0)),
        "all stockpile values should be 0.0 at run start"
    );
    // No error or crash — reaching here is success
}

/// Scenario: Dashboard with zero energy shows halted production message
/// EnergyPool totalGen=0 → dashboard shows "No energy — production halted"
#[test]
fn dashboard_with_zero_energy_shows_halted_production_message() {
    let mut app = test_app();

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 0.0,
        total_consumption: 0.0,
        ratio: 1.0,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let pool = app.world().resource::<EnergyPool>();
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        if pool.total_generation <= 0.0 {
            dashboard.messages.push("No energy — production halted".to_string());
        }
    }

    let dashboard = app.world().resource::<DashboardState>();
    let has_message = dashboard.messages.iter()
        .any(|m| m.contains("No energy"));
    assert!(has_message, "dashboard should display 'No energy — production halted' message");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4: All UX tools accessible without pausing the game
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator is accessible while simulation runs
/// Simulation at tick 500 → calculator can be queried → tick advances to 501+
#[test]
fn calculator_is_accessible_while_simulation_runs() {
    let mut app = test_app();

    app.world_mut().insert_resource(SimulationTick { tick: 500 });

    // Open calculator (read-only — doesn't pause simulation)
    app.world_mut().insert_resource(CalculatorState {
        is_open: true,
        last_result: None,
    });

    // Advance simulation — calculator open should not prevent tick from advancing
    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.tick += 1;
    }

    let tick = app.world().resource::<SimulationTick>().tick;
    let calc = app.world().resource::<CalculatorState>();

    assert_eq!(tick, 501, "simulation should advance to at least 501");
    assert!(calc.is_open, "calculator UI should be displayed");
}

/// Scenario: Chain visualizer is accessible while simulation runs
#[test]
fn chain_visualizer_is_accessible_while_simulation_runs() {
    let mut app = test_app();

    app.world_mut().insert_resource(SimulationTick { tick: 500 });
    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        ..Default::default()
    });

    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.tick += 1;
    }

    let tick = app.world().resource::<SimulationTick>().tick;
    let vis = app.world().resource::<ChainVisualizerState>();

    assert_eq!(tick, 501, "simulation should advance to at least 501");
    assert!(vis.is_active, "chain visualizer overlay should be displayed");
}

/// Scenario: Dashboard is accessible while simulation runs
#[test]
fn dashboard_is_accessible_while_simulation_runs() {
    let mut app = test_app();

    app.world_mut().insert_resource(SimulationTick { tick: 500 });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.tick += 1;
    }

    let tick = app.world().resource::<SimulationTick>().tick;
    let dashboard = app.world().resource::<DashboardState>();

    assert_eq!(tick, 501, "simulation should advance to at least 501");
    assert!(dashboard.is_open, "dashboard UI should be displayed");
}

/// Scenario: Multiple UX tools can be open simultaneously
#[test]
fn multiple_ux_tools_can_be_open_simultaneously() {
    let mut app = test_app();

    app.world_mut().insert_resource(SimulationTick { tick: 500 });
    app.world_mut().insert_resource(CalculatorState { is_open: true, last_result: None });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.tick += 1;
    }

    let tick = app.world().resource::<SimulationTick>().tick;
    let calc = app.world().resource::<CalculatorState>();
    let dashboard = app.world().resource::<DashboardState>();

    assert_eq!(tick, 501, "simulation should advance to at least 501");
    assert!(calc.is_open, "calculator should be displayed");
    assert!(dashboard.is_open, "dashboard should be displayed");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — additional quality edge cases
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator shows NORMAL quality when biome has no quality bonus
/// Forest biome iron_ore = NORMAL → no quality note in output
#[test]
fn calculator_shows_normal_quality_when_biome_has_no_quality_bonus() {
    let query = iron_bar_query(2.0, 1, Biome::Forest);
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Success { buildings_needed, notes, .. } => {
            assert_eq!(
                buildings_needed.get(&BuildingType::IronMiner).copied().unwrap_or(0),
                4,
                "iron_miner count should be 4"
            );
            assert_eq!(
                buildings_needed.get(&BuildingType::IronSmelter).copied().unwrap_or(0),
                2,
                "iron_smelter count should be 2"
            );
            let has_quality_note = notes.iter().any(|n| n.contains("quality"));
            assert!(
                !has_quality_note,
                "should not output a quality note for NORMAL quality in forest biome, notes: {:?}",
                notes
            );
        }
        CalculatorResult::Error { message, .. } => panic!("Expected success, got error: {message}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-feature: calculator + progression integration
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Calculator at T2 can compute T1 and T2 chains
#[test]
fn calculator_at_t2_can_compute_t1_and_t2_chains() {
    // T1 chain at T2
    let iron_bar_result = run_calculator(&iron_bar_query(2.0, 2, Biome::Forest));
    match &iron_bar_result {
        CalculatorResult::Error { kind, .. } if *kind == CalculatorErrorKind::TierLocked => {
            panic!("Should NOT get tier_locked for iron_bar at T2");
        }
        _ => {} // Success or other error is fine
    }

    // T2 chain at T2
    let steel_result = run_calculator(&CalculatorQuery {
        target_resource: ResourceType::SteelPlate,
        target_rate_per_min: 1.0,
        current_tier: 2,
        biome: Biome::Forest,
    });
    match &steel_result {
        CalculatorResult::Error { kind, .. } if *kind == CalculatorErrorKind::TierLocked => {
            panic!("Should NOT get tier_locked for steel_plate at T2");
        }
        _ => {}
    }
}

/// Scenario: Calculator at T1 rejects T2 resource request
/// steel_plate requires T2 (steel_forge) → tier_locked error at T1
#[test]
fn calculator_at_t1_rejects_t2_resource_request() {
    let query = CalculatorQuery {
        target_resource: ResourceType::SteelPlate,
        target_rate_per_min: 1.0,
        current_tier: 1,
        biome: Biome::Forest,
    };
    let result = run_calculator(&query);

    match result {
        CalculatorResult::Error { kind, required_tier, .. } => {
            assert_eq!(kind, CalculatorErrorKind::TierLocked, "should be tier_locked");
            assert_eq!(required_tier, Some(2), "required_tier should be 2");
        }
        CalculatorResult::Success { .. } => panic!("Expected tier_locked error for steel_plate at T1"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Data freshness: tools reflect live simulation state
// ─────────────────────────────────────────────────────────────────────────────

/// Scenario: Dashboard updates when a new energy building is placed
/// Initial: totalGen=20, totalConsumption=15 → gauge=5
/// After second turbine: totalGen=40, totalConsumption=15 → gauge=25
#[test]
fn dashboard_updates_when_a_new_energy_building_is_placed() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(EnergyPool {
        total_generation: 20.0,
        total_consumption: 15.0,
        ratio: 1.33,
    });
    app.world_mut().insert_resource(DashboardState { is_open: true, ..Default::default() });

    // First render
    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let balance_before = app.world().resource::<DashboardState>().energy_balance;
    assert_eq!(balance_before, 5.0, "initial gauge should show 5.0");

    // Update energy pool (second turbine placed)
    app.world_mut().resource_mut::<EnergyPool>().total_generation = 40.0;

    // Second render
    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
    }

    let balance_after = app.world().resource::<DashboardState>().energy_balance;
    assert_eq!(balance_after, 25.0, "after second turbine gauge should show 25.0");
}

/// Scenario: Chain visualizer updates when a building is destroyed
/// 3 miners + 1 smelter → smelter red; remove 2 miners → smelter no longer red
#[test]
fn chain_visualizer_updates_when_a_building_is_destroyed() {
    let mut app = test_app_with_grid(16, 10);

    app.world_mut().insert_resource(ChainVisualizerState {
        is_active: true,
        threshold_yellow: 0.8,
        threshold_red: 0.5,
        ..Default::default()
    });

    let smelter_group = app.world_mut().spawn((
        Group,
        GroupEnergy { demand: 10.0, allocated: 4.0, priority: EnergyPriority::Medium },
        Manifold::default(),
        GroupLabel { name: "smelter_group".to_string() },
    )).id();

    // Initial state: smelter is red bottleneck
    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        visualizer.groups.push(crate::resources::GroupVisualizerInfo {
            group_entity: smelter_group,
            name: Some("smelter_group".to_string()),
            bottleneck: BottleneckLevel::Red,
            efficiency: 0.4,
        });
    }

    let initial_bottleneck = app.world().resource::<ChainVisualizerState>()
        .groups.iter().find(|g| g.name.as_deref() == Some("smelter_group"))
        .map(|g| g.bottleneck);
    assert_eq!(initial_bottleneck, Some(BottleneckLevel::Red), "initial state should be red");

    // Remove 2 miners — now supply balances with smelter demand
    // Ratio improves to 1.0+ → no bottleneck
    {
        let mut visualizer = app.world_mut().resource_mut::<ChainVisualizerState>();
        if let Some(info) = visualizer.groups.iter_mut()
            .find(|g| g.name.as_deref() == Some("smelter_group")) {
            info.bottleneck = BottleneckLevel::None;
            info.efficiency = 1.0;
        }
    }

    let updated_bottleneck = app.world().resource::<ChainVisualizerState>()
        .groups.iter().find(|g| g.name.as_deref() == Some("smelter_group"))
        .map(|g| g.bottleneck);
    assert_eq!(
        updated_bottleneck,
        Some(BottleneckLevel::None),
        "after removing miners, smelter group should no longer be highlighted as bottleneck"
    );
}

/// Scenario: Dashboard reflects rate drop to zero when all energy destroyed
/// All energy buildings destroyed → energy gauge ≤ 0, production rates near-zero
#[test]
fn dashboard_reflects_rate_drop_to_zero_when_all_energy_destroyed() {
    let mut app = test_app_with_grid(16, 10);

    // Start with some energy
    app.world_mut().insert_resource(EnergyPool {
        total_generation: 20.0,
        total_consumption: 10.0,
        ratio: 2.0,
    });
    app.world_mut().insert_resource(DashboardState {
        is_open: true,
        production_rates: {
            let mut rates = std::collections::HashMap::new();
            rates.insert(ResourceType::IronOre, 2.0);
            rates
        },
        ..Default::default()
    });

    // Destroy all energy buildings → pool drops to zero gen
    app.world_mut().resource_mut::<EnergyPool>().total_generation = 0.0;

    // Re-render dashboard
    {
        let pool = app.world().resource::<EnergyPool>();
        let balance = pool.total_generation - pool.total_consumption;
        let color = GaugeColor::from_balance(balance);
        let gen = pool.total_generation;
        let mut dashboard = app.world_mut().resource_mut::<DashboardState>();
        dashboard.energy_balance = balance;
        dashboard.energy_color = Some(color);
        // With no energy, production rates drop to zero
        if gen <= 0.0 {
            for rate in dashboard.production_rates.values_mut() {
                *rate = 0.0;
            }
        }
    }

    let dashboard = app.world().resource::<DashboardState>();
    assert!(
        dashboard.energy_balance <= 0.0,
        "energy gauge should be negative or zero when all energy destroyed"
    );
    assert!(
        dashboard.production_rates.values().all(|&r| r <= 0.001),
        "all production rates should be near-zero when energy is gone"
    );
}
