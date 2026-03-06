//! Game Startup BDD tests — `.ptsd/bdd/game-startup.feature`
//!
//! Each test maps 1:1 to a BDD scenario. Tests verify that GameStartupPlugin
//! initializes all required ECS state before the first simulation tick:
//! recipe validation, terrain generation, starting kit, opus tree, fog, and
//! run configuration.
//!
//! Tests are written to FAIL until GameStartupPlugin is implemented.
//! The plugin will run systems in the Bevy Startup schedule.

use bevy::prelude::*;

use crate::components::*;
use crate::data::recipes::default_recipe;
use crate::resources::*;
use crate::SimulationPlugin;

// ─────────────────────────────────────────────────────────────────────────────
// Constants from seed files
// ─────────────────────────────────────────────────────────────────────────────

const GRID_W: i32 = 64;
const GRID_H: i32 = 64;
const SPAWN_X: i32 = 15;
const SPAWN_Y: i32 = 15;
const FOG_REVEAL_RADIUS: i32 = 12;
const FOG_REVEALED_CELLS: usize = 313; // 2*r*(r+1)+1 for r=12
const OPUS_SUSTAIN_TICKS: u32 = 600;
const OPUS_NODE_COUNT: usize = 7;
const STARTING_KIT_TOTAL: u32 = 20;
const MAX_TICKS: u64 = 108_000;
const TPS: u32 = 20;

// ─────────────────────────────────────────────────────────────────────────────
// All BuildingType variants (35 total, from recipe_validation seed)
// ─────────────────────────────────────────────────────────────────────────────

const ALL_BUILDING_TYPES: [BuildingType; 35] = [
    // Legacy
    BuildingType::Miner,
    BuildingType::Smelter,
    BuildingType::EnergySource,
    // Extraction
    BuildingType::IronMiner,
    BuildingType::CopperMiner,
    BuildingType::StoneQuarry,
    BuildingType::WaterPump,
    BuildingType::ObsidianDrill,
    BuildingType::ManaExtractor,
    BuildingType::LavaSiphon,
    // Synthesis
    BuildingType::IronSmelter,
    BuildingType::CopperSmelter,
    BuildingType::TreeFarm,
    BuildingType::Sawmill,
    BuildingType::SteelForge,
    BuildingType::SteelSmelter,
    BuildingType::Tannery,
    BuildingType::CrystalRefinery,
    BuildingType::AlchemistLab,
    BuildingType::RunicForge,
    BuildingType::ArcaneDistillery,
    // Mall
    BuildingType::Constructor,
    BuildingType::Toolmaker,
    BuildingType::Assembler,
    // Combat
    BuildingType::ImpCamp,
    BuildingType::BreedingPen,
    BuildingType::WarLodge,
    // Energy
    BuildingType::WindTurbine,
    BuildingType::WaterWheel,
    BuildingType::LavaGenerator,
    BuildingType::ManaReactor,
    // Opus
    BuildingType::OpusForge,
    // Utility
    BuildingType::Watchtower,
    BuildingType::Trader,
    BuildingType::SacrificeAltar,
];

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// App with SimulationPlugin (64x64 grid) — no GameStartupPlugin.
fn sim_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: GRID_W, grid_height: GRID_H });
    app
}

/// App with SimulationPlugin + GameStartupPlugin (default seed).
/// TODO: replace with real GameStartupPlugin import once implemented.
/// For now we simulate what the plugin SHOULD do so tests compile.
/// When GameStartupPlugin exists, replace this body with:
///   app.add_plugins(GameStartupPlugin::default());
fn startup_app() -> App {
    let mut app = sim_app();
    // TODO: import GameStartupPlugin once implemented
    // app.add_plugins(GameStartupPlugin::default());
    apply_startup_manually(&mut app, 42);
    app
}

/// App with SimulationPlugin + GameStartupPlugin with a specific seed.
fn startup_app_with_seed(seed: u64) -> App {
    let mut app = sim_app();
    // TODO: import GameStartupPlugin once implemented
    // app.add_plugins(GameStartupPlugin { seed });
    apply_startup_manually(&mut app, seed);
    app
}

/// Manhattan distance between two points.
fn manhattan(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs() + (ay - by).abs()
}

// ─────────────────────────────────────────────────────────────────────────────
// Manual startup simulation (placeholder until GameStartupPlugin exists)
//
// This function applies what GameStartupPlugin SHOULD do. Once the real plugin
// is implemented, these tests should switch to using app.add_plugins(GameStartupPlugin)
// and this function should be deleted.
// ─────────────────────────────────────────────────────────────────────────────

fn apply_startup_manually(app: &mut App, _seed: u64) {
    // Intentionally empty — tests will FAIL until GameStartupPlugin is implemented.
    // When the plugin is ready, remove this function and use add_plugins instead.
    let _ = app;
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC1: Recipe Validation
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Every BuildingType variant has a valid default recipe
///
/// This test validates the recipe database independently of GameStartupPlugin.
/// default_recipe() is a pure function — it always works. The startup plugin
/// should additionally run validation at startup, but this test verifies the
/// function itself.
#[test]
fn ac1_every_building_type_has_valid_default_recipe() {
    // default_recipe is a pure function — call it for all 35 variants
    for bt in ALL_BUILDING_TYPES {
        let recipe = default_recipe(bt);
        // Recipe must have valid duration
        assert!(
            recipe.duration_ticks > 0,
            "{bt:?}: duration_ticks must be > 0, got {}",
            recipe.duration_ticks
        );
    }
}

/// Scenario: Extractors have empty inputs and mall buildings output to inventory
#[test]
fn ac1_extractors_empty_inputs_mall_output_to_inventory() {
    let extractors = [
        BuildingType::IronMiner,
        BuildingType::CopperMiner,
        BuildingType::StoneQuarry,
        BuildingType::WaterPump,
        BuildingType::ObsidianDrill,
        BuildingType::ManaExtractor,
        BuildingType::LavaSiphon,
    ];
    for bt in extractors {
        let recipe = default_recipe(bt);
        assert!(
            recipe.inputs.is_empty(),
            "{bt:?}: extractor must have inputs == [], got {:?}",
            recipe.inputs
        );
    }

    let mall = [
        BuildingType::Constructor,
        BuildingType::Toolmaker,
        BuildingType::Assembler,
    ];
    for bt in mall {
        let recipe = default_recipe(bt);
        assert!(
            recipe.output_to_inventory,
            "{bt:?}: mall building must have output_to_inventory == true"
        );
    }

    let energy = [
        BuildingType::WindTurbine,
        BuildingType::WaterWheel,
        BuildingType::LavaGenerator,
        BuildingType::ManaReactor,
    ];
    for bt in energy {
        let recipe = default_recipe(bt);
        assert_eq!(
            recipe.duration_ticks, 1,
            "{bt:?}: energy building must have duration_ticks == 1, got {}",
            recipe.duration_ticks
        );
    }

    let utility = [
        BuildingType::Watchtower,
        BuildingType::Trader,
        BuildingType::SacrificeAltar,
    ];
    for bt in utility {
        let recipe = default_recipe(bt);
        assert_eq!(
            recipe.duration_ticks, 1,
            "{bt:?}: utility building must have duration_ticks == 1, got {}",
            recipe.duration_ticks
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC2: Terrain Generation
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Terrain generation with seed 42 places resource clusters near spawn
#[test]
fn ac2_terrain_generation_seed_42_clusters_near_spawn() {
    let mut app = startup_app_with_seed(42);
    app.update();

    let grid = app.world().resource::<Grid>();

    // Grid dimensions
    assert_eq!(grid.width, GRID_W, "Grid width must be {GRID_W}");
    assert_eq!(grid.height, GRID_H, "Grid height must be {GRID_H}");

    // Check IronVein exists within Manhattan distance 15 of spawn
    let has_iron = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::IronVein && manhattan(x, y, SPAWN_X, SPAWN_Y) <= 15
    });
    assert!(has_iron, "IronVein must exist within Manhattan distance 15 of spawn");

    // Check CopperVein
    let has_copper = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::CopperVein && manhattan(x, y, SPAWN_X, SPAWN_Y) <= 15
    });
    assert!(has_copper, "CopperVein must exist within Manhattan distance 15 of spawn");

    // Check StoneDeposit
    let has_stone = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::StoneDeposit && manhattan(x, y, SPAWN_X, SPAWN_Y) <= 15
    });
    assert!(has_stone, "StoneDeposit must exist within Manhattan distance 15 of spawn");

    // Check WaterSource
    let has_water = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::WaterSource && manhattan(x, y, SPAWN_X, SPAWN_Y) <= 15
    });
    assert!(has_water, "WaterSource must exist within Manhattan distance 15 of spawn");

    // At least 50 non-Grass cells
    let non_grass = grid.terrain.values().filter(|&&t| t != TerrainType::Grass).count();
    assert!(
        non_grass >= 50,
        "At least 50 non-Grass terrain cells expected, got {non_grass}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC3: Terrain Determinism
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Same seed produces identical terrain maps
#[test]
fn ac3_same_seed_identical_terrain() {
    let mut app_a = startup_app_with_seed(42);
    let mut app_b = startup_app_with_seed(42);
    app_a.update();
    app_b.update();

    let grid_a = app_a.world().resource::<Grid>();
    let grid_b = app_b.world().resource::<Grid>();

    assert_eq!(grid_a.width, grid_b.width, "Grid widths must match");
    assert_eq!(grid_a.height, grid_b.height, "Grid heights must match");
    assert_eq!(
        grid_a.terrain.len(),
        grid_b.terrain.len(),
        "Terrain entry count must match"
    );

    // Cell-by-cell comparison
    for (&pos, &terrain_a) in &grid_a.terrain {
        let terrain_b = grid_b.terrain.get(&pos).copied().unwrap_or_default();
        assert_eq!(
            terrain_a, terrain_b,
            "Terrain at {pos:?} differs: {terrain_a:?} vs {terrain_b:?}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC4: Starting Kit
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Starting kit populates inventory with exact building counts
#[test]
fn ac4_starting_kit_exact_building_counts() {
    let mut app = startup_app();
    app.update();

    let inv = app.world().resource::<Inventory>();

    // Exact counts from starting_kit.yaml
    assert_eq!(inv.count_building(BuildingType::IronMiner), 4, "IronMiner=4");
    assert_eq!(inv.count_building(BuildingType::CopperMiner), 2, "CopperMiner=2");
    assert_eq!(inv.count_building(BuildingType::StoneQuarry), 2, "StoneQuarry=2");
    assert_eq!(inv.count_building(BuildingType::WaterPump), 2, "WaterPump=2");
    assert_eq!(inv.count_building(BuildingType::IronSmelter), 2, "IronSmelter=2");
    assert_eq!(inv.count_building(BuildingType::CopperSmelter), 1, "CopperSmelter=1");
    assert_eq!(inv.count_building(BuildingType::Sawmill), 1, "Sawmill=1");
    assert_eq!(inv.count_building(BuildingType::TreeFarm), 1, "TreeFarm=1");
    assert_eq!(inv.count_building(BuildingType::Constructor), 1, "Constructor=1");
    assert_eq!(inv.count_building(BuildingType::WindTurbine), 3, "WindTurbine=3");
    assert_eq!(inv.count_building(BuildingType::Watchtower), 1, "Watchtower=1");

    // Total count = 20
    let total: u32 = inv.buildings.values().sum();
    assert_eq!(total, STARTING_KIT_TOTAL, "Total building count must be {STARTING_KIT_TOTAL}");
}

/// Scenario: All starting kit buildings are tier 1
#[test]
fn ac4_starting_kit_all_tier_1() {
    let mut app = startup_app();
    app.update();

    let inv = app.world().resource::<Inventory>();

    for (&bt, &count) in &inv.buildings {
        if count > 0 {
            assert_eq!(
                bt.tier(), 1,
                "{bt:?} in starting kit must be tier 1, got tier {}",
                bt.tier()
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC5: Opus Tree Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Opus tree has 7 milestone nodes with correct initial state
#[test]
fn ac5_opus_tree_7_nodes_correct_state() {
    let mut app = startup_app();
    app.update();

    let opus = app.world().resource::<OpusTreeResource>();

    // Exactly 7 nodes
    assert_eq!(
        opus.main_path.len(),
        OPUS_NODE_COUNT,
        "OpusTreeResource.main_path must have {OPUS_NODE_COUNT} nodes"
    );

    // Node 0: IronBar, rate 2.0
    assert_eq!(opus.main_path[0].resource, ResourceType::IronBar);
    assert!((opus.main_path[0].required_rate - 2.0).abs() < f32::EPSILON);

    // Node 1: CopperBar, rate 1.5
    assert_eq!(opus.main_path[1].resource, ResourceType::CopperBar);
    assert!((opus.main_path[1].required_rate - 1.5).abs() < f32::EPSILON);

    // Node 2: Plank, rate 2.0
    assert_eq!(opus.main_path[2].resource, ResourceType::Plank);
    assert!((opus.main_path[2].required_rate - 2.0).abs() < f32::EPSILON);

    // Node 3: SteelPlate, rate 1.0
    assert_eq!(opus.main_path[3].resource, ResourceType::SteelPlate);
    assert!((opus.main_path[3].required_rate - 1.0).abs() < f32::EPSILON);

    // Node 4: RefinedCrystal, rate 0.5
    assert_eq!(opus.main_path[4].resource, ResourceType::RefinedCrystal);
    assert!((opus.main_path[4].required_rate - 0.5).abs() < f32::EPSILON);

    // Node 5: RunicAlloy, rate 0.3
    assert_eq!(opus.main_path[5].resource, ResourceType::RunicAlloy);
    assert!((opus.main_path[5].required_rate - 0.3).abs() < f32::EPSILON);

    // Node 6: OpusIngot, rate 0.1
    assert_eq!(opus.main_path[6].resource, ResourceType::OpusIngot);
    assert!((opus.main_path[6].required_rate - 0.1).abs() < f32::EPSILON);

    // All nodes: current_rate == 0.0, sustained == false
    for (i, node) in opus.main_path.iter().enumerate() {
        assert!(
            (node.current_rate - 0.0).abs() < f32::EPSILON,
            "Node {i}: current_rate must be 0.0, got {}",
            node.current_rate
        );
        assert!(
            !node.sustained,
            "Node {i}: sustained must be false"
        );
    }

    // sustain_ticks_required
    assert_eq!(
        opus.sustain_ticks_required, OPUS_SUSTAIN_TICKS,
        "sustain_ticks_required must be {OPUS_SUSTAIN_TICKS}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC6: Fog Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Fog reveals Manhattan distance 12 diamond around spawn
#[test]
fn ac6_fog_reveals_manhattan_diamond() {
    let mut app = startup_app();
    app.update();

    let fog = app.world().resource::<FogMap>();

    // Exactly 313 cells revealed
    assert_eq!(
        fog.revealed.len(),
        FOG_REVEALED_CELLS,
        "FogMap must reveal exactly {FOG_REVEALED_CELLS} cells, got {}",
        fog.revealed.len()
    );

    // All cells within Manhattan distance 12 of spawn are revealed
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let dist = manhattan(x, y, SPAWN_X, SPAWN_Y);
            if dist <= FOG_REVEAL_RADIUS {
                assert!(
                    fog.is_visible(x, y),
                    "Cell ({x},{y}) at distance {dist} must be revealed"
                );
            }
        }
    }

    // Resource cluster positions are within revealed area (from fog.yaml)
    // IronVein cluster at (10,10) — dist=10
    assert!(fog.is_visible(10, 10), "IronVein cluster at (10,10) must be revealed");
    // CopperVein cluster at (20,10) — dist=10
    assert!(fog.is_visible(20, 10), "CopperVein cluster at (20,10) must be revealed");
    // StoneDeposit cluster at (15,20) — dist=5
    assert!(fog.is_visible(15, 20), "StoneDeposit cluster at (15,20) must be revealed");
    // WaterSource cluster at (25,15) — dist=10
    assert!(fog.is_visible(25, 15), "WaterSource cluster at (25,15) must be revealed");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC7: Run Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Run config initializes with correct defaults
///
/// Note: tick_increment_system runs during Update and increments current_tick.
/// Startup initializes current_tick=0, then the first Update tick makes it 1.
/// We verify current_tick==1 after one update (proves startup set it to 0).
#[test]
fn ac7_run_config_correct_defaults() {
    let mut app = startup_app();
    app.update();

    let run_config = app.world().resource::<RunConfig>();
    // After startup (sets 0) + one tick_increment (adds 1) = 1
    assert_eq!(run_config.current_tick, 1, "current_tick must be 1 after startup(0) + first tick");
    assert_eq!(run_config.max_ticks, MAX_TICKS, "max_ticks must be {MAX_TICKS}");
    assert_eq!(run_config.biome, Biome::Forest, "biome must be Forest");
    assert_eq!(run_config.tps, TPS, "tps must be {TPS}");

    let tier = app.world().resource::<TierState>();
    assert_eq!(tier.current_tier, 1, "TierState.current_tier must be 1");

    let run_state = app.world().resource::<RunState>();
    assert_eq!(run_state.status, RunStatus::InProgress, "RunState.status must be InProgress");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC8: Startup Schedule Integration
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: All resources populated after single app update
#[test]
fn ac8_all_resources_populated_after_single_update() {
    let mut app = startup_app();
    app.update();

    // Grid: 64x64
    let grid = app.world().resource::<Grid>();
    assert_eq!(grid.width, GRID_W);
    assert_eq!(grid.height, GRID_H);

    // Inventory: 20 buildings
    let inv = app.world().resource::<Inventory>();
    let total: u32 = inv.buildings.values().sum();
    assert_eq!(total, STARTING_KIT_TOTAL, "Inventory must have {STARTING_KIT_TOTAL} buildings");

    // OpusTreeResource: 7 main_path nodes
    let opus = app.world().resource::<OpusTreeResource>();
    assert_eq!(opus.main_path.len(), OPUS_NODE_COUNT);

    // FogMap: 313 revealed cells
    let fog = app.world().resource::<FogMap>();
    assert_eq!(fog.revealed.len(), FOG_REVEALED_CELLS);

    // RunConfig: current_tick 1 (startup sets 0, tick_increment adds 1)
    let rc = app.world().resource::<RunConfig>();
    assert_eq!(rc.current_tick, 1);

    // TierState: current_tier 1
    let tier = app.world().resource::<TierState>();
    assert_eq!(tier.current_tier, 1);

    // RunState: InProgress
    let rs = app.world().resource::<RunState>();
    assert_eq!(rs.status, RunStatus::InProgress);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Seed 0 produces valid terrain without panic
#[test]
fn edge_seed_0_valid_terrain_no_panic() {
    let mut app = startup_app_with_seed(0);
    app.update(); // must not panic

    let grid = app.world().resource::<Grid>();
    let non_grass = grid.terrain.values().filter(|&&t| t != TerrainType::Grass).count();
    assert!(
        non_grass >= 50,
        "Seed 0: at least 50 non-Grass cells expected, got {non_grass}"
    );
}

/// Scenario: Boundary grid cells are valid terrain
#[test]
fn edge_boundary_cells_valid_terrain() {
    let mut app = startup_app_with_seed(42);
    app.update();

    let grid = app.world().resource::<Grid>();

    // Terrain must be populated (startup must have run)
    assert!(
        !grid.terrain.is_empty(),
        "Grid terrain must be non-empty (startup must populate it)"
    );

    // Boundary cells must be in bounds
    assert!(grid.in_bounds(0, 0), "(0,0) must be in bounds");
    assert!(grid.in_bounds(63, 63), "(63,63) must be in bounds");
    assert!(!grid.in_bounds(64, 64), "(64,64) must be out of bounds");

    // Boundary cells must have terrain entries from startup
    assert!(
        grid.terrain.contains_key(&(0, 0)),
        "(0,0) must have an explicit terrain entry from startup"
    );
    assert!(
        grid.terrain.contains_key(&(63, 63)),
        "(63,63) must have an explicit terrain entry from startup"
    );
}

/// Scenario: OpusTree Default state has empty main_path before startup runs
#[test]
fn edge_opus_tree_default_empty_before_startup() {
    // App with SimulationPlugin only — no GameStartupPlugin
    let app = sim_app();

    let opus = app.world().resource::<OpusTreeResource>();
    assert!(
        opus.main_path.is_empty(),
        "OpusTreeResource.main_path must be empty without GameStartupPlugin"
    );
    assert!(
        (opus.completion_pct - 0.0).abs() < f32::EPSILON,
        "OpusTreeResource.completion_pct must be 0.0 without GameStartupPlugin"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional coverage tests (derived from BDD + seed data)
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that all 35 BuildingType variants are covered by default_recipe
/// (exhaustive match, no panic on any variant).
#[test]
fn ac1_all_35_variants_covered() {
    assert_eq!(ALL_BUILDING_TYPES.len(), 35, "Must test all 35 BuildingType variants");
    for bt in ALL_BUILDING_TYPES {
        // This should not panic — match is exhaustive
        let _recipe = default_recipe(bt);
    }
}

/// Verify energy buildings with terrain_req also have no inputs
/// (WaterWheel requires WaterSource, LavaGenerator requires LavaSource)
#[test]
fn ac1_energy_terrain_extractors_no_inputs() {
    let energy_terrain = [
        BuildingType::WaterWheel,
        BuildingType::LavaGenerator,
    ];
    for bt in energy_terrain {
        let recipe = default_recipe(bt);
        assert!(
            recipe.inputs.is_empty(),
            "{bt:?}: energy building with terrain_req must have inputs == [], got {:?}",
            recipe.inputs
        );
    }
}

/// Verify starting kit has exactly 11 building types
#[test]
fn ac4_starting_kit_11_types() {
    let mut app = startup_app();
    app.update();

    let inv = app.world().resource::<Inventory>();
    let types_with_stock: usize = inv.buildings.iter().filter(|(_, c)| **c > 0).count();
    assert_eq!(types_with_stock, 11, "Starting kit must have exactly 11 building types");
}

/// Verify opus tree node tiers match seed data
#[test]
fn ac5_opus_tree_node_tiers() {
    let mut app = startup_app();
    app.update();

    let opus = app.world().resource::<OpusTreeResource>();
    assert_eq!(opus.main_path.len(), OPUS_NODE_COUNT);

    // From opus_tree.yaml: nodes 0-2 are T1, nodes 3-4 are T2, nodes 5-6 are T3
    assert_eq!(opus.main_path[0].tier, 1, "IronBar node tier");
    assert_eq!(opus.main_path[1].tier, 1, "CopperBar node tier");
    assert_eq!(opus.main_path[2].tier, 1, "Plank node tier");
    assert_eq!(opus.main_path[3].tier, 2, "SteelPlate node tier");
    assert_eq!(opus.main_path[4].tier, 2, "RefinedCrystal node tier");
    assert_eq!(opus.main_path[5].tier, 3, "RunicAlloy node tier");
    assert_eq!(opus.main_path[6].tier, 3, "OpusIngot node tier");
}

/// Verify fog does NOT reveal cells beyond the radius
#[test]
fn ac6_fog_does_not_reveal_beyond_radius() {
    let mut app = startup_app();
    app.update();

    let fog = app.world().resource::<FogMap>();

    // Fog must have been populated (startup must have run)
    assert!(
        !fog.revealed.is_empty(),
        "FogMap must be non-empty (startup must reveal cells)"
    );

    // Cells just outside radius should NOT be revealed
    // Manhattan distance 13 from spawn (15,15): e.g. (2,15) -> dist=13
    let outside_positions = [
        (2, 15),   // dist = 13
        (15, 2),   // dist = 13
        (28, 15),  // dist = 13
        (15, 28),  // dist = 13
        (0, 0),    // dist = 30
    ];
    for (x, y) in outside_positions {
        let dist = manhattan(x, y, SPAWN_X, SPAWN_Y);
        assert!(dist > FOG_REVEAL_RADIUS, "Sanity: ({x},{y}) dist={dist}");
        assert!(
            !fog.is_visible(x, y),
            "Cell ({x},{y}) at distance {dist} must NOT be revealed"
        );
    }
}

/// Verify terrain determinism with a different seed (seed 123 != seed 42)
#[test]
fn ac3_different_seeds_produce_different_terrain() {
    let mut app_42 = startup_app_with_seed(42);
    let mut app_123 = startup_app_with_seed(123);
    app_42.update();
    app_123.update();

    let grid_42 = app_42.world().resource::<Grid>();
    let grid_123 = app_123.world().resource::<Grid>();

    // Both must have populated terrain (startup must have run)
    assert!(
        !grid_42.terrain.is_empty(),
        "Seed 42 terrain must be non-empty (startup must populate it)"
    );
    assert!(
        !grid_123.terrain.is_empty(),
        "Seed 123 terrain must be non-empty (startup must populate it)"
    );

    // Different seeds must produce different terrain
    let differs = grid_42.terrain.iter().any(|(pos, &t42)| {
        grid_123.terrain.get(pos).copied().unwrap_or_default() != t42
    }) || grid_42.terrain.len() != grid_123.terrain.len();
    assert!(differs, "Different seeds must produce different terrain");
}

/// Verify RunConfig sustain window and sample interval from run_config.yaml
#[test]
fn ac7_run_config_sustain_and_sample() {
    let mut app = startup_app();
    app.update();

    let rc = app.world().resource::<RunConfig>();
    assert_eq!(rc.sustain_window_ticks, 600, "sustain_window_ticks must be 600");
    assert_eq!(rc.sample_interval_ticks, 20, "sample_interval_ticks must be 20");
    assert!(!rc.abandoned, "abandoned must be false at startup");
}

/// Verify RunState scoring fields start at zero
#[test]
fn ac7_run_state_scoring_zeroed() {
    let mut app = startup_app();
    app.update();

    let rs = app.world().resource::<RunState>();
    assert_eq!(rs.status, RunStatus::InProgress);
    assert!((rs.opus_completion - 0.0).abs() < f32::EPSILON, "opus_completion starts at 0");
    assert!((rs.mini_opus_score - 0.0).abs() < f32::EPSILON, "mini_opus_score starts at 0");
    assert!((rs.time_bonus - 0.0).abs() < f32::EPSILON, "time_bonus starts at 0");
    assert!((rs.raw_score - 0.0).abs() < f32::EPSILON, "raw_score starts at 0");
    assert_eq!(rs.final_score, 0, "final_score starts at 0");
    assert!((rs.currency_earned - 0.0).abs() < f32::EPSILON, "currency_earned starts at 0");
}

/// Verify terrain clusters are at expected positions from terrain.yaml
#[test]
fn ac2_terrain_clusters_at_seed_positions() {
    let mut app = startup_app_with_seed(42);
    app.update();

    let grid = app.world().resource::<Grid>();

    // From terrain.yaml: IronVein cluster centered at (10,10) radius 2
    // At minimum, the center cell should be IronVein
    let iron_near_center = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::IronVein && manhattan(x, y, 10, 10) <= 2
    });
    assert!(iron_near_center, "IronVein cluster expected near (10,10)");

    // CopperVein cluster centered at (20,10) radius 2
    let copper_near_center = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::CopperVein && manhattan(x, y, 20, 10) <= 2
    });
    assert!(copper_near_center, "CopperVein cluster expected near (20,10)");

    // StoneDeposit cluster centered at (15,20) radius 2
    let stone_near_center = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::StoneDeposit && manhattan(x, y, 15, 20) <= 2
    });
    assert!(stone_near_center, "StoneDeposit cluster expected near (15,20)");

    // WaterSource cluster centered at (25,15) radius 1
    let water_near_center = grid.terrain.iter().any(|(&(x, y), &t)| {
        t == TerrainType::WaterSource && manhattan(x, y, 25, 15) <= 1
    });
    assert!(water_near_center, "WaterSource cluster expected near (25,15)");
}
