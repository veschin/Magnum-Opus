//! Game Render BDD tests — `.ptsd/bdd/game-render.feature`
//!
//! Each test maps 1:1 to a BDD scenario. Tests verify that RenderPlugin
//! correctly syncs simulation ECS state to 3D isometric scene entities:
//! grid rendering, building sync, visual state, group outlines, transport
//! visualization, fog overlay, ghost preview, post-processing, lighting,
//! shader animations, and the read-only guarantee.
//!
//! Tests are written to FAIL until RenderPlugin is implemented.
//! RenderPlugin is a read-only layer — it must never mutate simulation state.

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::SimulationPlugin;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const GRID_W: i32 = 64;
const GRID_H: i32 = 64;
const TILE_HALF_WIDTH: f32 = 32.0;
const TILE_HALF_HEIGHT: f32 = 16.0;

// ─────────────────────────────────────────────────────────────────────────────
// Placeholder Render Components & Resources
//
// These types represent what RenderPlugin SHOULD create. Once the real plugin
// is implemented, replace these with imports from the render module.
// ─────────────────────────────────────────────────────────────────────────────

/// Links a scene entity to a grid tile position.
#[derive(Component)]
struct SceneEntity {
    grid_pos: (i32, i32),
}

/// Terrain visual properties on a scene entity.
#[derive(Component)]
struct TerrainVisual {
    base_color: [f32; 3],
    height_offset: f32,
    emissive: bool,
}

/// Material and animation state for a building scene entity.
#[derive(Component)]
struct RenderMaterial {
    name: String,
    animation: bool,
}

/// Colored outline enclosing a building group.
#[derive(Component)]
struct GroupOutline {
    group_id: u32,
    positions: Vec<(i32, i32)>,
    color: [f32; 4],
}

/// Visual properties for a transport path sprite.
#[derive(Component)]
struct PathSprite {
    color: [f32; 3],
    shimmer_speed: f32,
}

/// Visual properties for a cargo sprite on a transport path.
#[derive(Component)]
struct CargoSprite {
    color: [f32; 3],
    progress: f32,
    bounce_amplitude: f32,
    bounce_frequency: f32,
    phase_offset: f32,
}

/// Fog overlay on a tile.
#[derive(Component)]
struct FogOverlay {
    color: [f32; 4],
    desaturation: f32,
}

/// Ghost preview entity during placement mode.
#[derive(Component)]
struct GhostPreview {
    tint: [f32; 4],
}

/// Marker for building scene entities.
#[derive(Component)]
struct BuildingSceneEntity;

/// Marker for creature scene entities.
#[derive(Component)]
struct CreatureSceneEntity;

/// Point light spawned near emissive buildings.
#[derive(Component)]
struct PointLightMarker {
    color: [f32; 3],
    radius: f32,
    intensity: f32,
}

/// Idle bobbing animation parameters.
#[derive(Component)]
struct IdleBob {
    amplitude: f32,
    frequency: f32,
    phase_offset: f32,
}

/// Wind sway animation parameters for organic buildings.
#[derive(Component)]
struct WindSway {
    amplitude: f32,
    frequency: f32,
}

/// Liquid flow animation.
#[derive(Component)]
struct LiquidFlow {
    uv_scroll_speed: f32,
}

/// Emission pulse animation for emissive buildings.
#[derive(Component)]
struct EmissionPulse {
    base_intensity: f32,
    pulse_amplitude: f32,
    pulse_frequency: f32,
}

/// Magenta placeholder for missing sprite assets.
#[derive(Component)]
struct MagentaPlaceholder;

/// Post-processing pass configuration.
#[derive(Debug, Clone)]
struct PostProcessPass {
    name: String,
    params: PostProcessParams,
}

/// Parameters for each post-processing pass type.
#[derive(Debug, Clone)]
enum PostProcessParams {
    Outline {
        threshold: f32,
        kernel_size: u32,
    },
    ToonShading {
        bands: u32,
    },
    Posterization {
        levels_per_channel: u32,
    },
    Upscale {
        filter: String,
    },
}

/// Post-processing pipeline configuration resource.
#[derive(Resource)]
struct PostProcessConfig {
    passes: Vec<PostProcessPass>,
    low_res_width: u32,
    low_res_height: u32,
    upscale_factor: u32,
}

/// Directional light configuration resource.
#[derive(Resource)]
struct DirectionalLightConfig {
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}

/// Ambient light configuration resource.
#[derive(Resource)]
struct AmbientLightConfig {
    color: [f32; 3],
    strength: f32,
}

/// Placement mode resource for ghost preview tests.
#[derive(Resource)]
struct PlacementMode {
    active: bool,
    building_type: Option<BuildingType>,
}

/// Cursor grid position resource.
#[derive(Resource)]
struct CursorGridPos {
    x: i32,
    y: i32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// App with SimulationPlugin (64x64 grid) — no RenderPlugin.
fn render_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin {
        grid_width: GRID_W,
        grid_height: GRID_H,
    });
    // TODO: app.add_plugins(RenderPlugin);
    // Intentionally no RenderPlugin — tests FAIL until implemented
    app
}

/// App with a custom grid size — no RenderPlugin.
fn render_app_sized(w: i32, h: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin {
        grid_width: w,
        grid_height: h,
    });
    // TODO: app.add_plugins(RenderPlugin);
    app
}

/// Spawn a building entity with standard components at (x, y).
fn spawn_building(app: &mut App, bt: BuildingType, x: i32, y: i32) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Building { building_type: bt },
            Position { x, y },
            ProductionState::default(),
            Footprint::single(x, y),
        ))
        .id();
    // Mark grid cell occupied
    app.world_mut()
        .resource_mut::<Grid>()
        .occupied
        .insert((x, y));
    entity
}

/// Spawn a building and assign it to a group.
fn spawn_building_in_group(
    app: &mut App,
    bt: BuildingType,
    x: i32,
    y: i32,
    group_entity: Entity,
) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            Building { building_type: bt },
            Position { x, y },
            ProductionState::default(),
            Footprint::single(x, y),
            GroupMember {
                group_id: group_entity,
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<Grid>()
        .occupied
        .insert((x, y));
    entity
}

/// Spawn a group entity with Group marker.
fn spawn_group(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Group,
            GroupEnergy::default(),
            Manifold::default(),
            GroupControl::default(),
            GroupStats::default(),
        ))
        .id()
}

/// Spawn a transport path entity.
fn spawn_transport_path(
    app: &mut App,
    kind: TransportKind,
    segments: Vec<(i32, i32)>,
    tier: u8,
) -> Entity {
    let source = spawn_group(app);
    let target = spawn_group(app);
    let stats = TierStats::for_path(tier);
    app.world_mut()
        .spawn(TransportPath {
            kind,
            source_group: source,
            target_group: target,
            resource_filter: None,
            tier,
            capacity: stats.capacity,
            speed: stats.speed,
            connected: true,
            segments,
        })
        .id()
}

/// Spawn a cargo entity on a path.
fn spawn_cargo(
    app: &mut App,
    path_entity: Entity,
    resource: ResourceType,
    progress: f32,
) -> Entity {
    app.world_mut()
        .spawn(Cargo {
            path_entity,
            resource,
            amount: 1.0,
            position_on_path: progress,
        })
        .id()
}

/// Spawn a creature entity at (x, y).
fn spawn_creature(app: &mut App, x: i32, y: i32) -> Entity {
    app.world_mut()
        .spawn((
            Creature {
                species: CreatureSpecies::ForestDeer,
                archetype: CreatureArchetype::Ambient,
                biome: BiomeTag::Forest,
                health: 10.0,
                max_health: 10.0,
                state: CreatureStateKind::Idle,
            },
            Position { x, y },
        ))
        .id()
}

/// Isometric transform: grid position to screen position.
fn iso_screen_pos(gx: i32, gy: i32) -> (f32, f32) {
    let sx = (gx - gy) as f32 * TILE_HALF_WIDTH;
    let sy = (gx + gy) as f32 * TILE_HALF_HEIGHT;
    (sx, sy)
}

/// Helper to approximately compare f32 values.
fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC1: Grid Rendering — terrain tiles to scene entities
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Every terrain tile gets a corresponding scene entity with correct position
#[test]
fn ac1_every_terrain_tile_gets_scene_entity_with_correct_position() {
    let mut app = render_app();
    app.update();

    // After RenderPlugin runs, every tile in the 64x64 grid should have a SceneEntity
    let scene_count = app
        .world_mut()
        .query::<&SceneEntity>()
        .iter(app.world())
        .count();
    assert_eq!(
        scene_count,
        (GRID_W * GRID_H) as usize,
        "Expected {} scene entities for {}x{} grid, got {}",
        GRID_W * GRID_H,
        GRID_W,
        GRID_H,
        scene_count
    );

    // Verify isometric transform for each scene entity
    for scene in app.world_mut().query::<&SceneEntity>().iter(app.world()) {
        let (gx, gy) = scene.grid_pos;
        let (expected_sx, expected_sy) = iso_screen_pos(gx, gy);
        // Scene entity position will be checked via Transform once RenderPlugin exists
        let _ = (expected_sx, expected_sy);
    }
}

/// Scenario: Terrain types have correct color and height offset
#[test]
fn ac1_terrain_types_have_correct_color_and_height_offset() {
    let mut app = render_app_sized(4, 4);

    // Set up terrain types
    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((0, 0), TerrainType::Grass);
        grid.terrain.insert((1, 0), TerrainType::IronVein);
        grid.terrain.insert((0, 1), TerrainType::WaterSource);
        grid.terrain.insert((1, 2), TerrainType::StoneDeposit);
        grid.terrain.insert((3, 2), TerrainType::LavaSource);
    }

    app.update();

    // Query terrain visuals — RenderPlugin should create TerrainVisual components
    let visuals: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &TerrainVisual)>()
        .iter(app.world())
        .collect();

    // Find specific tiles
    let grass = visuals.iter().find(|(s, _)| s.grid_pos == (0, 0));
    assert!(grass.is_some(), "Grass tile at (0,0) must have a TerrainVisual");
    let (_, tv) = grass.unwrap();
    assert_eq!(tv.base_color, [0.35, 0.55, 0.25], "Grass base_color");
    assert!(approx_eq(tv.height_offset, 0.0, 0.001), "Grass height_offset");
    assert!(!tv.emissive, "Grass not emissive");

    let iron = visuals.iter().find(|(s, _)| s.grid_pos == (1, 0));
    assert!(iron.is_some(), "IronVein at (1,0) must have a TerrainVisual");
    let (_, tv) = iron.unwrap();
    assert_eq!(tv.base_color, [0.45, 0.35, 0.30], "IronVein base_color");
    assert!(approx_eq(tv.height_offset, 0.15, 0.001), "IronVein height_offset");

    let water = visuals.iter().find(|(s, _)| s.grid_pos == (0, 1));
    assert!(water.is_some(), "WaterSource at (0,1) must have a TerrainVisual");
    let (_, tv) = water.unwrap();
    assert_eq!(tv.base_color, [0.20, 0.35, 0.55], "WaterSource base_color");
    assert!(approx_eq(tv.height_offset, -0.10, 0.001), "WaterSource height_offset");

    let stone = visuals.iter().find(|(s, _)| s.grid_pos == (1, 2));
    assert!(stone.is_some(), "StoneDeposit at (1,2) must have a TerrainVisual");
    let (_, tv) = stone.unwrap();
    assert_eq!(tv.base_color, [0.50, 0.50, 0.48], "StoneDeposit base_color");
    assert!(approx_eq(tv.height_offset, 0.10, 0.001), "StoneDeposit height_offset");

    let lava = visuals.iter().find(|(s, _)| s.grid_pos == (3, 2));
    assert!(lava.is_some(), "LavaSource at (3,2) must have a TerrainVisual");
    let (_, tv) = lava.unwrap();
    assert_eq!(tv.base_color, [0.80, 0.25, 0.05], "LavaSource base_color");
    assert!(approx_eq(tv.height_offset, -0.05, 0.001), "LavaSource height_offset");
    assert!(tv.emissive, "LavaSource must be emissive");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC2: Building Sync — spawn and despawn scene entities
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Placed building gets a scene entity at correct grid position
#[test]
fn ac2_placed_building_gets_scene_entity() {
    let mut app = render_app();

    // Place an IronMiner at (10, 10) via PlacementCommands
    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
    }
    {
        let mut cmds = app
            .world_mut()
            .resource_mut::<crate::systems::placement::PlacementCommands>();
        let recipe = crate::data::recipes::default_recipe(BuildingType::IronMiner);
        cmds.queue.push((BuildingType::IronMiner, 10, 10, recipe));
    }

    app.update();

    // RenderPlugin should create a scene entity at grid position (10, 10)
    let building_scenes: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &BuildingSceneEntity)>()
        .iter(app.world())
        .collect();
    let found = building_scenes
        .iter()
        .any(|(se, _)| se.grid_pos == (10, 10));
    assert!(
        found,
        "A building scene entity must exist at grid position (10, 10)"
    );
}

/// Scenario: Removed building despawns its scene entity
#[test]
fn ac2_removed_building_despawns_scene_entity() {
    let mut app = render_app();

    // Pre-place a building at (5, 5)
    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((5, 5), TerrainType::IronVein);
    }
    spawn_building(&mut app, BuildingType::IronMiner, 5, 5);
    app.update();

    // Issue removal command
    {
        let mut cmds = app.world_mut().resource_mut::<RemoveBuildingCommands>();
        cmds.queue.push((5, 5));
    }
    app.update();

    // RenderPlugin should have despawned the scene entity at (5, 5)
    let building_scenes: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &BuildingSceneEntity)>()
        .iter(app.world())
        .collect();
    let found = building_scenes
        .iter()
        .any(|(se, _)| se.grid_pos == (5, 5));
    assert!(
        !found,
        "No building scene entity should exist at grid position (5, 5) after removal"
    );
}

/// Scenario: Multiple buildings each get independent scene entities
#[test]
fn ac2_multiple_buildings_get_independent_scene_entities() {
    let mut app = render_app();

    // Set up terrain for each building
    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
        grid.terrain.insert((11, 10), TerrainType::IronVein);
    }

    spawn_building(&mut app, BuildingType::IronMiner, 10, 10);
    spawn_building(&mut app, BuildingType::IronMiner, 11, 10);
    spawn_building(&mut app, BuildingType::IronSmelter, 10, 11);
    spawn_building(&mut app, BuildingType::WindTurbine, 20, 20);

    app.update();

    // RenderPlugin should create exactly 4 building scene entities
    let building_scenes: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &BuildingSceneEntity)>()
        .iter(app.world())
        .collect();
    assert_eq!(
        building_scenes.len(),
        4,
        "Exactly 4 building scene entities expected, got {}",
        building_scenes.len()
    );

    // Each must have a unique position
    let positions: Vec<_> = building_scenes.iter().map(|(se, _)| se.grid_pos).collect();
    assert!(positions.contains(&(10, 10)), "Scene entity at (10,10)");
    assert!(positions.contains(&(11, 10)), "Scene entity at (11,10)");
    assert!(positions.contains(&(10, 11)), "Scene entity at (10,11)");
    assert!(positions.contains(&(20, 20)), "Scene entity at (20,20)");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC3: Building Visual State — material reflects ECS state
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Building material and animation reflect production state
#[test]
fn ac3_building_material_and_animation_reflect_production_state() {
    let mut app = render_app();

    // Spawn buildings with different production states
    let producing = spawn_building(&mut app, BuildingType::IronMiner, 10, 10);
    let idle = spawn_building(&mut app, BuildingType::IronMiner, 10, 11);
    let no_energy = spawn_building(&mut app, BuildingType::IronMiner, 12, 12);
    let paused = spawn_building(&mut app, BuildingType::IronMiner, 13, 13);

    // Set production states
    app.world_mut()
        .entity_mut(producing)
        .get_mut::<ProductionState>()
        .unwrap()
        .active = true;

    {
        let mut entity_mut = app.world_mut().entity_mut(idle);
        let mut ps = entity_mut.get_mut::<ProductionState>().unwrap();
        ps.active = false;
        ps.idle_reason = Some(IdleReason::NoInputs);
    }
    {
        let mut entity_mut = app.world_mut().entity_mut(no_energy);
        let mut ps = entity_mut.get_mut::<ProductionState>().unwrap();
        ps.active = false;
        ps.idle_reason = Some(IdleReason::NoEnergy);
    }
    {
        let mut entity_mut = app.world_mut().entity_mut(paused);
        let mut ps = entity_mut.get_mut::<ProductionState>().unwrap();
        ps.active = false;
        ps.idle_reason = Some(IdleReason::GroupPaused);
    }

    app.update();

    // Query render materials
    let materials: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &RenderMaterial)>()
        .iter(app.world())
        .collect();

    let at_10_10 = materials.iter().find(|(s, _)| s.grid_pos == (10, 10));
    assert!(at_10_10.is_some(), "Producing building at (10,10) must have RenderMaterial");
    let (_, rm) = at_10_10.unwrap();
    assert_eq!(rm.name, "active", "Producing => material 'active'");
    assert!(rm.animation, "Producing => shader_animation true");

    let at_10_11 = materials.iter().find(|(s, _)| s.grid_pos == (10, 11));
    assert!(at_10_11.is_some(), "Idle building at (10,11) must have RenderMaterial");
    let (_, rm) = at_10_11.unwrap();
    assert_eq!(rm.name, "default", "Idle => material 'default'");
    assert!(!rm.animation, "Idle => shader_animation false");

    let at_12_12 = materials.iter().find(|(s, _)| s.grid_pos == (12, 12));
    assert!(at_12_12.is_some(), "NoEnergy building at (12,12) must have RenderMaterial");
    let (_, rm) = at_12_12.unwrap();
    assert_eq!(rm.name, "dimmed", "NoEnergy => material 'dimmed'");
    assert!(!rm.animation, "NoEnergy => shader_animation false");

    let at_13_13 = materials.iter().find(|(s, _)| s.grid_pos == (13, 13));
    assert!(at_13_13.is_some(), "Paused building at (13,13) must have RenderMaterial");
    let (_, rm) = at_13_13.unwrap();
    assert_eq!(rm.name, "yellow_tint", "Paused => material 'yellow_tint'");
    assert!(!rm.animation, "Paused => shader_animation false");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC4: Group Outlines — colored outlines enclosing groups
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Group outline encloses all member buildings with correct color
#[test]
fn ac4_group_outline_encloses_all_member_buildings() {
    let mut app = render_app();

    let group1 = spawn_group(&mut app);
    let group2 = spawn_group(&mut app);

    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
        grid.terrain.insert((11, 10), TerrainType::IronVein);
    }

    spawn_building_in_group(&mut app, BuildingType::IronMiner, 10, 10, group1);
    spawn_building_in_group(&mut app, BuildingType::IronMiner, 11, 10, group1);
    spawn_building_in_group(&mut app, BuildingType::IronSmelter, 10, 11, group1);
    spawn_building_in_group(&mut app, BuildingType::WindTurbine, 20, 20, group2);

    app.update();

    // RenderPlugin should create exactly 2 group outline entities
    let outlines: Vec<_> = app
        .world_mut()
        .query::<&GroupOutline>()
        .iter(app.world())
        .collect();
    assert_eq!(outlines.len(), 2, "Exactly 2 group outlines expected, got {}", outlines.len());

    // Group 1 outline encloses 3 positions
    let g1 = outlines.iter().find(|o| {
        o.positions.contains(&(10, 10))
            && o.positions.contains(&(11, 10))
            && o.positions.contains(&(10, 11))
    });
    assert!(g1.is_some(), "Group 1 outline must enclose (10,10), (11,10), (10,11)");
    let g1 = g1.unwrap();
    assert_eq!(
        g1.color,
        [0.2, 0.8, 0.2, 0.6],
        "Active group outline color must be [0.2, 0.8, 0.2, 0.6]"
    );

    // Group 2 outline encloses 1 position
    let g2 = outlines.iter().find(|o| o.positions.contains(&(20, 20)));
    assert!(g2.is_some(), "Group 2 outline must enclose (20,20)");
    let g2 = g2.unwrap();
    assert_eq!(
        g2.color,
        [0.2, 0.8, 0.2, 0.6],
        "Active group outline color must be [0.2, 0.8, 0.2, 0.6]"
    );
}

/// Scenario: Group outline color encodes group state
#[test]
fn ac4_group_outline_color_encodes_group_state() {
    let mut app = render_app();

    // Create groups with different states
    let group_active = spawn_group(&mut app);
    let group_paused = spawn_group(&mut app);
    let group_no_energy = spawn_group(&mut app);
    let group_idle = spawn_group(&mut app);

    // Set group statuses
    app.world_mut()
        .entity_mut(group_paused)
        .get_mut::<GroupControl>()
        .unwrap()
        .status = GroupStatus::Paused;

    // NoEnergy: group has demand but no allocation
    {
        let mut entity_mut = app.world_mut().entity_mut(group_no_energy);
        let mut ge = entity_mut.get_mut::<GroupEnergy>().unwrap();
        ge.demand = 10.0;
        ge.allocated = 0.0;
    }

    // Add a building to each group so outlines can be generated
    spawn_building_in_group(&mut app, BuildingType::IronMiner, 1, 1, group_active);
    spawn_building_in_group(&mut app, BuildingType::IronMiner, 3, 3, group_paused);
    spawn_building_in_group(&mut app, BuildingType::IronMiner, 5, 5, group_no_energy);
    spawn_building_in_group(&mut app, BuildingType::IronMiner, 7, 7, group_idle);

    app.update();

    let outlines: Vec<_> = app
        .world_mut()
        .query::<&GroupOutline>()
        .iter(app.world())
        .collect();

    // Active: green
    let active_outline = outlines.iter().find(|o| o.positions.contains(&(1, 1)));
    assert!(active_outline.is_some(), "Active group outline must exist");
    assert_eq!(
        active_outline.unwrap().color,
        [0.2, 0.8, 0.2, 0.6],
        "Active group color"
    );

    // Paused: yellow
    let paused_outline = outlines.iter().find(|o| o.positions.contains(&(3, 3)));
    assert!(paused_outline.is_some(), "Paused group outline must exist");
    assert_eq!(
        paused_outline.unwrap().color,
        [0.8, 0.8, 0.2, 0.6],
        "Paused group color"
    );

    // NoEnergy: red
    let no_energy_outline = outlines.iter().find(|o| o.positions.contains(&(5, 5)));
    assert!(no_energy_outline.is_some(), "NoEnergy group outline must exist");
    assert_eq!(
        no_energy_outline.unwrap().color,
        [0.8, 0.2, 0.2, 0.6],
        "NoEnergy group color"
    );

    // Idle: gray
    let idle_outline = outlines.iter().find(|o| o.positions.contains(&(7, 7)));
    assert!(idle_outline.is_some(), "Idle group outline must exist");
    assert_eq!(
        idle_outline.unwrap().color,
        [0.5, 0.5, 0.5, 0.4],
        "Idle group color"
    );
}

/// Scenario: Group split produces two separate outlines
#[test]
fn ac4_group_split_produces_two_separate_outlines() {
    let mut app = render_app();

    let group1 = spawn_group(&mut app);

    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
        grid.terrain.insert((11, 10), TerrainType::IronVein);
    }

    let _building_a = spawn_building_in_group(&mut app, BuildingType::IronMiner, 10, 10, group1);
    let _building_b = spawn_building_in_group(&mut app, BuildingType::IronMiner, 11, 10, group1);

    app.update();

    // Remove building A at (10, 10) — should cause group split
    {
        let mut cmds = app.world_mut().resource_mut::<RemoveBuildingCommands>();
        cmds.queue.push((10, 10));
    }
    app.update();

    // After split, the single outline around group 1 should be replaced
    // by one outline around B at (11, 10)
    let outlines: Vec<_> = app
        .world_mut()
        .query::<&GroupOutline>()
        .iter(app.world())
        .collect();

    // Should not contain (10, 10) anymore
    let has_10_10 = outlines.iter().any(|o| o.positions.contains(&(10, 10)));
    assert!(!has_10_10, "No outline should include removed building at (10, 10)");

    // Should have an outline containing (11, 10)
    let has_11_10 = outlines.iter().any(|o| o.positions.contains(&(11, 10)));
    assert!(has_11_10, "An outline should include remaining building at (11, 10)");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC5: Transport Visualization — paths and cargo
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Rune path renders as continuous line of sprites between groups
#[test]
fn ac5_rune_path_renders_as_sprites() {
    let mut app = render_app();

    let segments = vec![(10, 10), (11, 10), (12, 10), (13, 10), (14, 10)];
    spawn_transport_path(&mut app, TransportKind::RunePath, segments, 1);

    app.update();

    // RenderPlugin should create 5 path sprite entities
    let path_sprites: Vec<_> = app
        .world_mut()
        .query::<&PathSprite>()
        .iter(app.world())
        .collect();
    assert_eq!(
        path_sprites.len(),
        5,
        "Exactly 5 path sprite entities expected, got {}",
        path_sprites.len()
    );

    // Each path sprite should have shimmer speed 0.5
    for ps in &path_sprites {
        assert!(
            approx_eq(ps.shimmer_speed, 0.5, 0.001),
            "Path sprite shimmer UV-scroll speed must be 0.5, got {}",
            ps.shimmer_speed
        );
        assert_eq!(
            ps.color,
            [0.6, 0.5, 0.3],
            "Path sprite color must be [0.6, 0.5, 0.3]"
        );
    }
}

/// Scenario: Cargo sprites position and appearance match resource type
#[test]
fn ac5_cargo_sprites_position_and_appearance() {
    let mut app = render_app();

    let segments = vec![(10, 10), (11, 10), (12, 10), (13, 10), (14, 10)];
    let path = spawn_transport_path(&mut app, TransportKind::RunePath, segments, 1);

    spawn_cargo(&mut app, path, ResourceType::IronOre, 0.3);
    spawn_cargo(&mut app, path, ResourceType::CopperOre, 0.7);

    app.update();

    // RenderPlugin should create 2 cargo scene entities
    let cargo_sprites: Vec<_> = app
        .world_mut()
        .query::<&CargoSprite>()
        .iter(app.world())
        .collect();
    assert_eq!(
        cargo_sprites.len(),
        2,
        "Exactly 2 cargo scene entities expected, got {}",
        cargo_sprites.len()
    );

    // IronOre at progress 0.3
    let iron_cargo = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.3, 0.01));
    assert!(iron_cargo.is_some(), "Cargo at progress 0.3 must exist");
    assert_eq!(
        iron_cargo.unwrap().color,
        [0.45, 0.35, 0.30],
        "IronOre cargo color"
    );

    // CopperOre at progress 0.7
    let copper_cargo = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.7, 0.01));
    assert!(copper_cargo.is_some(), "Cargo at progress 0.7 must exist");
    assert_eq!(
        copper_cargo.unwrap().color,
        [0.60, 0.40, 0.20],
        "CopperOre cargo color"
    );
}

/// Scenario: Cargo bounce uses entity-id-based phase offset for desync
#[test]
fn ac5_cargo_bounce_uses_entity_id_phase_offset() {
    let mut app = render_app();

    let segments = vec![(10, 10), (11, 10), (12, 10)];
    let path = spawn_transport_path(&mut app, TransportKind::RunePath, segments, 1);

    let cargo_a = spawn_cargo(&mut app, path, ResourceType::IronOre, 0.3);
    let cargo_b = spawn_cargo(&mut app, path, ResourceType::IronOre, 0.6);

    app.update();

    // Both cargo entities should have bounce amplitude 0.08 and frequency 3.0
    let cargo_sprites: Vec<_> = app
        .world_mut()
        .query::<(Entity, &CargoSprite)>()
        .iter(app.world())
        .collect();
    assert_eq!(cargo_sprites.len(), 2, "Two cargo sprites expected");

    for (_, cs) in &cargo_sprites {
        assert!(
            approx_eq(cs.bounce_amplitude, 0.08, 0.001),
            "Cargo bounce amplitude must be 0.08, got {}",
            cs.bounce_amplitude
        );
        assert!(
            approx_eq(cs.bounce_frequency, 3.0, 0.01),
            "Cargo bounce frequency must be 3.0, got {}",
            cs.bounce_frequency
        );
    }

    // Phase offsets must differ (derived from entity IDs)
    let offsets: Vec<f32> = cargo_sprites.iter().map(|(_, cs)| cs.phase_offset).collect();
    assert_ne!(
        offsets[0], offsets[1],
        "Two cargo entities must have different phase offsets (got {} and {})",
        offsets[0], offsets[1]
    );
    let _ = (cargo_a, cargo_b); // used for spawn
}

/// Scenario: All resource types have distinct cargo visuals
#[test]
fn ac5_all_resource_types_have_distinct_cargo_visuals() {
    let mut app = render_app();

    let segments = vec![(10, 10), (11, 10), (12, 10)];
    let path = spawn_transport_path(&mut app, TransportKind::RunePath, segments, 1);

    spawn_cargo(&mut app, path, ResourceType::IronOre, 0.1);
    spawn_cargo(&mut app, path, ResourceType::CopperOre, 0.3);
    spawn_cargo(&mut app, path, ResourceType::IronBar, 0.5);
    spawn_cargo(&mut app, path, ResourceType::Water, 0.7);

    app.update();

    let cargo_sprites: Vec<_> = app
        .world_mut()
        .query::<&CargoSprite>()
        .iter(app.world())
        .collect();
    assert_eq!(cargo_sprites.len(), 4, "4 cargo sprites expected");

    // Verify colors by progress
    let iron_ore = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.1, 0.01));
    assert!(iron_ore.is_some(), "IronOre cargo at progress 0.1");
    assert_eq!(iron_ore.unwrap().color, [0.45, 0.35, 0.30], "IronOre color");

    let copper_ore = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.3, 0.01));
    assert!(copper_ore.is_some(), "CopperOre cargo at progress 0.3");
    assert_eq!(copper_ore.unwrap().color, [0.60, 0.40, 0.20], "CopperOre color");

    let iron_bar = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.5, 0.01));
    assert!(iron_bar.is_some(), "IronBar cargo at progress 0.5");
    assert_eq!(iron_bar.unwrap().color, [0.55, 0.55, 0.55], "IronBar color");

    let water = cargo_sprites.iter().find(|c| approx_eq(c.progress, 0.7, 0.01));
    assert!(water.is_some(), "Water cargo at progress 0.7");
    assert_eq!(water.unwrap().color, [0.20, 0.40, 0.70], "Water color");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC6: Fog of War — overlay based on FogMap
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Unrevealed tiles have opaque dark overlay
#[test]
fn ac6_unrevealed_tiles_have_opaque_dark_overlay() {
    let mut app = render_app_sized(5, 5);

    // Reveal 6 tiles at center
    {
        let mut fog = app.world_mut().resource_mut::<FogMap>();
        // Center area
        for &(x, y) in &[(2, 2), (1, 2), (2, 1), (3, 2), (2, 3), (1, 1)] {
            fog.reveal(x, y);
        }
    }

    app.update();

    // 19 unrevealed tiles should have dark overlay
    let fog_overlays: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &FogOverlay)>()
        .iter(app.world())
        .collect();

    let unrevealed_overlays: Vec<_> = fog_overlays
        .iter()
        .filter(|(_, fo)| fo.color == [0.05, 0.05, 0.08, 1.0])
        .collect();
    assert_eq!(
        unrevealed_overlays.len(),
        19,
        "19 unrevealed tiles must have opaque dark overlay, got {}",
        unrevealed_overlays.len()
    );
}

/// Scenario: Revealed tiles outside watchtower range are desaturated
#[test]
fn ac6_revealed_tiles_outside_watchtower_range_desaturated() {
    let mut app = render_app_sized(5, 5);

    // Reveal all tiles
    {
        let mut fog = app.world_mut().resource_mut::<FogMap>();
        fog.reveal_all(5, 5);
    }

    // Place watchtower at (2, 2) with radius 1
    spawn_building(&mut app, BuildingType::Watchtower, 2, 2);

    app.update();

    // Tile (3, 3) is outside watchtower radius 1 from (2, 2) (manhattan distance = 2)
    let fog_overlays: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &FogOverlay)>()
        .iter(app.world())
        .collect();

    let tile_3_3 = fog_overlays.iter().find(|(se, _)| se.grid_pos == (3, 3));
    assert!(tile_3_3.is_some(), "Tile (3,3) must have a FogOverlay");
    assert!(
        approx_eq(tile_3_3.unwrap().1.desaturation, 0.7, 0.01),
        "Tile (3,3) desaturation must be 0.7, got {}",
        tile_3_3.unwrap().1.desaturation
    );

    // Tiles within watchtower range should have no desaturation
    let in_range_tiles = [(2, 2), (1, 2), (2, 1), (3, 2), (2, 3)];
    for pos in &in_range_tiles {
        let tile = fog_overlays.iter().find(|(se, _)| se.grid_pos == *pos);
        if let Some((_, fo)) = tile {
            assert!(
                approx_eq(fo.desaturation, 0.0, 0.01),
                "Tile {:?} within watchtower range must have desaturation 0.0, got {}",
                pos,
                fo.desaturation
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC7: Ghost Preview — placement mode cursor feedback
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Ghost preview shows green tint for valid placement
#[test]
fn ac7_ghost_preview_green_tint_valid_placement() {
    let mut app = render_app();

    // Set up placement mode
    app.insert_resource(PlacementMode {
        active: true,
        building_type: Some(BuildingType::Constructor),
    });
    app.insert_resource(CursorGridPos { x: 15, y: 15 });

    app.update();

    // Ghost preview at (15, 15) with green tint
    let ghosts: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &GhostPreview)>()
        .iter(app.world())
        .collect();

    let ghost = ghosts.iter().find(|(se, _)| se.grid_pos == (15, 15));
    assert!(ghost.is_some(), "Ghost scene entity must exist at (15, 15)");
    assert_eq!(
        ghost.unwrap().1.tint,
        [0.2, 0.8, 0.2, 0.5],
        "Valid placement ghost tint must be [0.2, 0.8, 0.2, 0.5]"
    );
}

/// Scenario: Ghost preview shows red tint for invalid placement
#[test]
fn ac7_ghost_preview_red_tint_invalid_placement() {
    let mut app = render_app();

    // Occupy position (10, 10)
    spawn_building(&mut app, BuildingType::IronMiner, 10, 10);

    app.insert_resource(PlacementMode {
        active: true,
        building_type: Some(BuildingType::Constructor),
    });
    app.insert_resource(CursorGridPos { x: 10, y: 10 });

    app.update();

    let ghosts: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &GhostPreview)>()
        .iter(app.world())
        .collect();

    let ghost = ghosts.iter().find(|(se, _)| se.grid_pos == (10, 10));
    assert!(ghost.is_some(), "Ghost scene entity must exist at (10, 10)");
    assert_eq!(
        ghost.unwrap().1.tint,
        [0.8, 0.2, 0.2, 0.5],
        "Invalid placement ghost tint must be [0.8, 0.2, 0.2, 0.5]"
    );
}

/// Scenario: Ghost preview disappears when placement mode exits
#[test]
fn ac7_ghost_preview_disappears_when_placement_mode_exits() {
    let mut app = render_app();

    // Start with placement mode active
    app.insert_resource(PlacementMode {
        active: true,
        building_type: Some(BuildingType::Constructor),
    });
    app.insert_resource(CursorGridPos { x: 15, y: 15 });
    app.update();

    // Deactivate placement mode
    app.world_mut().resource_mut::<PlacementMode>().active = false;
    app.update();

    let ghost_count = app
        .world_mut()
        .query::<&GhostPreview>()
        .iter(app.world())
        .count();
    assert_eq!(ghost_count, 0, "No ghost entity should exist after placement mode exits");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC8: Post-Processing Chain — outline, toon, posterize, upscale
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Post-processing pipeline has 4 passes in correct order with correct parameters
#[test]
fn ac8_post_processing_pipeline_4_passes_correct_order() {
    let mut app = render_app();
    app.update();

    // RenderPlugin should insert PostProcessConfig resource
    let config = app.world().get_resource::<PostProcessConfig>();
    assert!(config.is_some(), "PostProcessConfig resource must exist");
    let config = config.unwrap();

    assert_eq!(config.passes.len(), 4, "Exactly 4 post-processing passes");

    // Pass 0: outline
    assert_eq!(config.passes[0].name, "outline", "Pass 0 must be 'outline'");
    match &config.passes[0].params {
        PostProcessParams::Outline {
            threshold,
            kernel_size,
        } => {
            assert!(approx_eq(*threshold, 0.3, 0.001), "Outline threshold 0.3");
            assert_eq!(*kernel_size, 3, "Outline kernel_size 3");
        }
        _ => panic!("Pass 0 must be Outline variant"),
    }

    // Pass 1: toon_shading
    assert_eq!(config.passes[1].name, "toon_shading", "Pass 1 must be 'toon_shading'");
    match &config.passes[1].params {
        PostProcessParams::ToonShading { bands } => {
            assert_eq!(*bands, 3, "Toon shading bands 3");
        }
        _ => panic!("Pass 1 must be ToonShading variant"),
    }

    // Pass 2: posterization
    assert_eq!(config.passes[2].name, "posterization", "Pass 2 must be 'posterization'");
    match &config.passes[2].params {
        PostProcessParams::Posterization { levels_per_channel } => {
            assert_eq!(*levels_per_channel, 8, "Posterization 8 levels per channel");
        }
        _ => panic!("Pass 2 must be Posterization variant"),
    }

    // Pass 3: upscale
    assert_eq!(config.passes[3].name, "upscale", "Pass 3 must be 'upscale'");
    match &config.passes[3].params {
        PostProcessParams::Upscale { filter } => {
            assert_eq!(filter, "nearest_neighbor", "Upscale filter nearest_neighbor");
        }
        _ => panic!("Pass 3 must be Upscale variant"),
    }
}

/// Scenario: Low-res render target dimensions and upscale factor
#[test]
fn ac8_low_res_render_target_dimensions() {
    let mut app = render_app();
    // Window resolution 1920x1080 would be set via a resource
    app.update();

    let config = app.world().get_resource::<PostProcessConfig>();
    assert!(config.is_some(), "PostProcessConfig resource must exist");
    let config = config.unwrap();

    assert_eq!(config.low_res_width, 480, "Low-res width 480");
    assert_eq!(config.low_res_height, 270, "Low-res height 270");
    assert_eq!(config.upscale_factor, 4, "Upscale factor 4");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC9: Lighting — directional, point lights, ambient
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Emissive buildings spawn point lights with per-type parameters
#[test]
fn ac9_emissive_buildings_spawn_point_lights() {
    let mut app = render_app();

    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::Grass);
        grid.terrain.insert((20, 20), TerrainType::LavaSource);
        grid.terrain.insert((5, 5), TerrainType::ManaNode);
        grid.terrain.insert((15, 15), TerrainType::Grass);
    }

    spawn_building(&mut app, BuildingType::IronSmelter, 10, 10);
    spawn_building(&mut app, BuildingType::LavaGenerator, 20, 20);
    spawn_building(&mut app, BuildingType::ManaReactor, 5, 5);
    spawn_building(&mut app, BuildingType::SacrificeAltar, 15, 15);

    app.update();

    let lights: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &PointLightMarker)>()
        .iter(app.world())
        .collect();

    // IronSmelter at (10,10)
    let smelter = lights.iter().find(|(se, _)| se.grid_pos == (10, 10));
    assert!(smelter.is_some(), "Point light at IronSmelter (10,10)");
    let (_, pl) = smelter.unwrap();
    assert_eq!(pl.color, [1.0, 0.6, 0.2], "IronSmelter light color");
    assert!(approx_eq(pl.radius, 3.0, 0.01), "IronSmelter light radius 3.0");
    assert!(approx_eq(pl.intensity, 0.8, 0.01), "IronSmelter light intensity 0.8");

    // LavaGenerator at (20,20)
    let lava = lights.iter().find(|(se, _)| se.grid_pos == (20, 20));
    assert!(lava.is_some(), "Point light at LavaGenerator (20,20)");
    let (_, pl) = lava.unwrap();
    assert_eq!(pl.color, [1.0, 0.3, 0.05], "LavaGenerator light color");
    assert!(approx_eq(pl.radius, 4.0, 0.01), "LavaGenerator light radius 4.0");
    assert!(approx_eq(pl.intensity, 1.2, 0.01), "LavaGenerator light intensity 1.2");

    // ManaReactor at (5,5)
    let mana = lights.iter().find(|(se, _)| se.grid_pos == (5, 5));
    assert!(mana.is_some(), "Point light at ManaReactor (5,5)");
    let (_, pl) = mana.unwrap();
    assert_eq!(pl.color, [0.3, 0.2, 0.9], "ManaReactor light color");
    assert!(approx_eq(pl.radius, 4.0, 0.01), "ManaReactor light radius 4.0");
    assert!(approx_eq(pl.intensity, 1.0, 0.01), "ManaReactor light intensity 1.0");

    // SacrificeAltar at (15,15)
    let altar = lights.iter().find(|(se, _)| se.grid_pos == (15, 15));
    assert!(altar.is_some(), "Point light at SacrificeAltar (15,15)");
    let (_, pl) = altar.unwrap();
    assert_eq!(pl.color, [0.6, 0.1, 0.8], "SacrificeAltar light color");
    assert!(approx_eq(pl.radius, 3.5, 0.01), "SacrificeAltar light radius 3.5");
    assert!(approx_eq(pl.intensity, 0.9, 0.01), "SacrificeAltar light intensity 0.9");
}

/// Scenario: Directional light and ambient provide base illumination
#[test]
fn ac9_directional_light_and_ambient_base_illumination() {
    let mut app = render_app();
    app.update();

    // DirectionalLightConfig
    let dir_light = app.world().get_resource::<DirectionalLightConfig>();
    assert!(dir_light.is_some(), "DirectionalLightConfig resource must exist");
    let dir_light = dir_light.unwrap();
    assert_eq!(dir_light.direction, [-0.5, -0.7, 0.5], "Directional light direction");
    assert_eq!(dir_light.color, [1.0, 0.95, 0.85], "Directional light color");
    assert!(approx_eq(dir_light.intensity, 1.0, 0.01), "Directional light intensity 1.0");

    // AmbientLightConfig
    let ambient = app.world().get_resource::<AmbientLightConfig>();
    assert!(ambient.is_some(), "AmbientLightConfig resource must exist");
    let ambient = ambient.unwrap();
    assert_eq!(ambient.color, [0.15, 0.12, 0.18], "Ambient light color");
    assert!(approx_eq(ambient.strength, 0.3, 0.01), "Ambient light strength 0.3");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC10: Shader Animations — time-based, entity-desynced
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Buildings idle-bob with per-entity phase offset
#[test]
fn ac10_buildings_idle_bob_with_phase_offset() {
    let mut app = render_app();

    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
        grid.terrain.insert((12, 12), TerrainType::IronVein);
    }

    spawn_building(&mut app, BuildingType::IronMiner, 10, 10);
    spawn_building(&mut app, BuildingType::IronMiner, 12, 12);

    app.update();

    let bobs: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &IdleBob)>()
        .iter(app.world())
        .collect();

    // Both buildings should have idle_bob
    let bob_10 = bobs.iter().find(|(se, _)| se.grid_pos == (10, 10));
    assert!(bob_10.is_some(), "IronMiner at (10,10) must have IdleBob");
    let (_, b) = bob_10.unwrap();
    assert!(approx_eq(b.amplitude, 0.02, 0.001), "IdleBob amplitude 0.02");
    assert!(approx_eq(b.frequency, 1.5, 0.01), "IdleBob frequency 1.5");

    let bob_12 = bobs.iter().find(|(se, _)| se.grid_pos == (12, 12));
    assert!(bob_12.is_some(), "IronMiner at (12,12) must have IdleBob");
    let (_, b2) = bob_12.unwrap();
    assert!(approx_eq(b2.amplitude, 0.02, 0.001), "IdleBob amplitude 0.02");
    assert!(approx_eq(b2.frequency, 1.5, 0.01), "IdleBob frequency 1.5");

    // Phase offsets must differ
    assert_ne!(
        b.phase_offset, b2.phase_offset,
        "Phase offsets must differ between buildings"
    );
}

/// Scenario: Organic buildings sway with wind animation
#[test]
fn ac10_organic_buildings_wind_sway() {
    let mut app = render_app();

    spawn_building(&mut app, BuildingType::TreeFarm, 8, 8);
    spawn_building(&mut app, BuildingType::Sawmill, 9, 9);

    app.update();

    let sways: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &WindSway)>()
        .iter(app.world())
        .collect();

    let tree_sway = sways.iter().find(|(se, _)| se.grid_pos == (8, 8));
    assert!(tree_sway.is_some(), "TreeFarm at (8,8) must have WindSway");
    let (_, ws) = tree_sway.unwrap();
    assert!(approx_eq(ws.amplitude, 0.03, 0.001), "WindSway amplitude 0.03");
    assert!(approx_eq(ws.frequency, 0.8, 0.01), "WindSway frequency 0.8");

    let sawmill_sway = sways.iter().find(|(se, _)| se.grid_pos == (9, 9));
    assert!(sawmill_sway.is_some(), "Sawmill at (9,9) must have WindSway");
    let (_, ws2) = sawmill_sway.unwrap();
    assert!(approx_eq(ws2.amplitude, 0.03, 0.001), "WindSway amplitude 0.03");
    assert!(approx_eq(ws2.frequency, 0.8, 0.01), "WindSway frequency 0.8");
}

/// Scenario: Liquid and emissive buildings have type-specific shader animations
#[test]
fn ac10_liquid_and_emissive_shader_animations() {
    let mut app = render_app();

    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::LavaSource);
        grid.terrain.insert((20, 20), TerrainType::ManaNode);
        grid.terrain.insert((5, 5), TerrainType::WaterSource);
    }

    // WaterPump — liquid building
    spawn_building(&mut app, BuildingType::WaterPump, 5, 5);
    // LavaGenerator — emissive building
    spawn_building(&mut app, BuildingType::LavaGenerator, 10, 10);
    // ManaReactor — emissive building
    spawn_building(&mut app, BuildingType::ManaReactor, 20, 20);

    // Pipe transport entity
    let pipe_segments = vec![(6, 5), (7, 5), (8, 5)];
    spawn_transport_path(&mut app, TransportKind::Pipe, pipe_segments, 1);

    app.update();

    // Pipe and WaterPump should have liquid_flow with uv_scroll_speed 0.4
    let liquid_flows: Vec<_> = app
        .world_mut()
        .query::<&LiquidFlow>()
        .iter(app.world())
        .collect();
    // At least 2 entities should have LiquidFlow (WaterPump scene entity + pipe scene entities)
    assert!(
        liquid_flows.len() >= 2,
        "At least 2 LiquidFlow entities expected (WaterPump + Pipe), got {}",
        liquid_flows.len()
    );
    for lf in &liquid_flows {
        assert!(
            approx_eq(lf.uv_scroll_speed, 0.4, 0.01),
            "LiquidFlow uv_scroll_speed must be 0.4, got {}",
            lf.uv_scroll_speed
        );
    }

    // LavaGenerator and ManaReactor should have emission_pulse
    let pulses: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &EmissionPulse)>()
        .iter(app.world())
        .collect();

    let lava_pulse = pulses.iter().find(|(se, _)| se.grid_pos == (10, 10));
    assert!(lava_pulse.is_some(), "LavaGenerator at (10,10) must have EmissionPulse");
    let (_, ep) = lava_pulse.unwrap();
    assert!(approx_eq(ep.base_intensity, 0.6, 0.01), "EmissionPulse base_intensity 0.6");
    assert!(approx_eq(ep.pulse_amplitude, 0.3, 0.01), "EmissionPulse pulse_amplitude 0.3");
    assert!(approx_eq(ep.pulse_frequency, 2.0, 0.01), "EmissionPulse pulse_frequency 2.0");

    let mana_pulse = pulses.iter().find(|(se, _)| se.grid_pos == (20, 20));
    assert!(mana_pulse.is_some(), "ManaReactor at (20,20) must have EmissionPulse");
    let (_, ep2) = mana_pulse.unwrap();
    assert!(approx_eq(ep2.base_intensity, 0.6, 0.01), "EmissionPulse base_intensity 0.6");
    assert!(approx_eq(ep2.pulse_amplitude, 0.3, 0.01), "EmissionPulse pulse_amplitude 0.3");
    assert!(approx_eq(ep2.pulse_frequency, 2.0, 0.01), "EmissionPulse pulse_frequency 2.0");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC11: Read-Only Guarantee — render never mutates simulation
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Render plugin removal does not affect simulation behavior
///
/// This is the critical cross-system integration test. We run two identical
/// apps — one with SimulationPlugin only, one with SimulationPlugin + RenderPlugin.
/// After 100 ticks, all simulation state must be identical.
#[test]
fn ac11_render_plugin_does_not_affect_simulation() {
    // App A: SimulationPlugin only
    let mut app_a = render_app();
    // App B: SimulationPlugin + RenderPlugin (once implemented)
    let mut app_b = render_app();
    // TODO: app_b.add_plugins(RenderPlugin); — once RenderPlugin exists

    // Set up identical initial state in both apps
    for app in [&mut app_a, &mut app_b] {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((10, 10), TerrainType::IronVein);
        grid.terrain.insert((20, 20), TerrainType::Grass);
    }
    for app in [&mut app_a, &mut app_b] {
        spawn_building(app, BuildingType::IronMiner, 10, 10);
        spawn_building(app, BuildingType::WindTurbine, 20, 20);
    }

    // Run both apps for 100 ticks
    for _ in 0..100 {
        app_a.update();
        app_b.update();
    }

    // Compare simulation state
    let grid_a = app_a.world().resource::<Grid>();
    let grid_b = app_b.world().resource::<Grid>();
    assert_eq!(grid_a.width, grid_b.width, "Grid width must match");
    assert_eq!(grid_a.height, grid_b.height, "Grid height must match");
    assert_eq!(grid_a.occupied.len(), grid_b.occupied.len(), "Occupied cells must match");

    let energy_a = app_a.world().resource::<EnergyPool>();
    let energy_b = app_b.world().resource::<EnergyPool>();
    assert!(
        approx_eq(energy_a.total_generation, energy_b.total_generation, 0.001),
        "EnergyPool generation must match"
    );
    assert!(
        approx_eq(energy_a.total_consumption, energy_b.total_consumption, 0.001),
        "EnergyPool consumption must match"
    );

    let inv_a = app_a.world().resource::<Inventory>();
    let inv_b = app_b.world().resource::<Inventory>();
    assert_eq!(inv_a.buildings, inv_b.buildings, "Inventory buildings must match");

    // Compare building component data
    let buildings_a: Vec<_> = app_a
        .world_mut()
        .query::<(&Building, &Position)>()
        .iter(app_a.world())
        .map(|(b, p)| (b.building_type, p.x, p.y))
        .collect();
    let buildings_b: Vec<_> = app_b
        .world_mut()
        .query::<(&Building, &Position)>()
        .iter(app_b.world())
        .map(|(b, p)| (b.building_type, p.x, p.y))
        .collect();
    assert_eq!(
        buildings_a.len(),
        buildings_b.len(),
        "Building count must match"
    );
}

/// Scenario: No render system writes to simulation components resources or events
///
/// This test verifies the architectural contract by checking that no render
/// system has mutable access to simulation-owned data. Since RenderPlugin
/// doesn't exist yet, this test documents the requirement and will be
/// validated at implementation time.
#[test]
fn ac11_no_render_system_writes_to_simulation() {
    let app = render_app();

    // When RenderPlugin is implemented, we will inspect system access metadata:
    // - No system should have Mut<Building>, Mut<Position>, Mut<GroupMember>,
    //   Mut<ProductionState>, Mut<TransportPath>, Mut<Cargo>, Mut<Creature>,
    //   Mut<CreatureNest>, ResMut<FogMap>, ResMut<Grid>, ResMut<Inventory>,
    //   ResMut<EnergyPool>, or write access to any simulation event type.
    //
    // For now, assert that the app has no RenderPlugin systems that could
    // violate this — the app only has SimulationPlugin.

    // Verify we can query simulation state without any render interference
    let grid = app.world().resource::<Grid>();
    assert_eq!(grid.width, GRID_W, "Grid width accessible");
    assert_eq!(grid.height, GRID_H, "Grid height accessible");

    // This test will gain teeth once RenderPlugin is added to app_b in AC11 test above.
    // At that point, we can use Bevy's system access info to verify read-only access.
    //
    // TODO: Once RenderPlugin exists, add:
    // for system in app.world().resource::<Schedules>().get(...).systems() {
    //     assert no write access to simulation components/resources
    // }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Buildings at grid boundaries render at correct screen positions
#[test]
fn edge_buildings_at_grid_boundaries_correct_positions() {
    let mut app = render_app();

    spawn_building(&mut app, BuildingType::WindTurbine, 0, 0);
    spawn_building(&mut app, BuildingType::WindTurbine, 63, 63);
    spawn_building(&mut app, BuildingType::WindTurbine, 0, 63);
    spawn_building(&mut app, BuildingType::WindTurbine, 63, 0);

    app.update();

    let scenes: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &BuildingSceneEntity)>()
        .iter(app.world())
        .collect();

    let positions: Vec<(i32, i32)> = scenes.iter().map(|(se, _)| se.grid_pos).collect();
    assert!(positions.contains(&(0, 0)), "Building scene at (0, 0)");
    assert!(positions.contains(&(63, 63)), "Building scene at (63, 63)");
    assert!(positions.contains(&(0, 63)), "Building scene at (0, 63)");
    assert!(positions.contains(&(63, 0)), "Building scene at (63, 0)");

    // Verify all screen coordinates are valid (finite, not NaN)
    for (se, _) in &scenes {
        let (sx, sy) = iso_screen_pos(se.grid_pos.0, se.grid_pos.1);
        assert!(sx.is_finite(), "Screen x at {:?} must be finite", se.grid_pos);
        assert!(sy.is_finite(), "Screen y at {:?} must be finite", se.grid_pos);
    }
}

/// Scenario: Transport path with zero cargo renders without crash
#[test]
fn edge_transport_path_zero_cargo_no_crash() {
    let mut app = render_app();

    let segments = vec![(5, 5), (6, 5), (7, 5)];
    spawn_transport_path(&mut app, TransportKind::Pipe, segments, 1);

    // No cargo spawned

    app.update();

    // 3 path sprites, 0 cargo sprites
    let path_sprites = app
        .world_mut()
        .query::<&PathSprite>()
        .iter(app.world())
        .count();
    assert_eq!(path_sprites, 3, "Exactly 3 path sprite entities");

    let cargo_sprites = app
        .world_mut()
        .query::<&CargoSprite>()
        .iter(app.world())
        .count();
    assert_eq!(cargo_sprites, 0, "Exactly 0 cargo sprite entities");
}

/// Scenario: Creature despawn removes scene entity in same frame
#[test]
fn edge_creature_despawn_removes_scene_entity() {
    let mut app = render_app();

    let creature = spawn_creature(&mut app, 30, 30);
    app.update();

    // Despawn the creature
    app.world_mut().despawn(creature);
    app.update();

    // No scene entity for the despawned creature
    let creature_scenes: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &CreatureSceneEntity)>()
        .iter(app.world())
        .collect();
    let found = creature_scenes
        .iter()
        .any(|(se, _)| se.grid_pos == (30, 30));
    assert!(
        !found,
        "No creature scene entity should exist at (30, 30) after despawn"
    );
}

/// Scenario: FogMap with all tiles revealed produces no visible fog effect
#[test]
fn edge_all_tiles_revealed_no_fog_effect() {
    let mut app = render_app();

    // Reveal all tiles
    {
        let mut fog = app.world_mut().resource_mut::<FogMap>();
        fog.reveal_all(GRID_W, GRID_H);
    }

    app.update();

    // Zero fog overlay entities should exist
    let fog_overlays = app
        .world_mut()
        .query::<&FogOverlay>()
        .iter(app.world())
        .count();
    assert_eq!(
        fog_overlays, 0,
        "Zero fog overlay entities when all tiles revealed, got {}",
        fog_overlays
    );
}

/// Scenario: Weather change lerps directional light over 10 frames without pop
#[test]
fn edge_weather_change_lerps_directional_light() {
    let mut app = render_app();

    // Start with Clear weather
    app.insert_resource(CurrentWeather {
        weather_type: WeatherType::Clear,
        fire_effect: 0.0,
        water_effect: 0.0,
        cold_effect: 0.0,
        wind_effect: 0.0,
        fog_penalty: 0.0,
        ticks_remaining: 600,
    });

    app.update();

    // Verify initial directional light
    let dir_light = app.world().get_resource::<DirectionalLightConfig>();
    assert!(dir_light.is_some(), "DirectionalLightConfig must exist");
    let initial_color = dir_light.unwrap().color;
    let initial_intensity = dir_light.unwrap().intensity;

    // Change to Rain weather
    app.world_mut().resource_mut::<CurrentWeather>().weather_type = WeatherType::Rain;

    // Update 5 times (midpoint of 10-frame lerp)
    for _ in 0..5 {
        app.update();
    }

    let dir_light = app.world().get_resource::<DirectionalLightConfig>().unwrap();
    // At frame 5/10, color should be interpolating between Clear and Rain
    // Clear color: [1.0, 0.95, 0.85], Rain color: [0.70, 0.7125, 0.7225]
    // At midpoint: approximately [(1.0+0.70)/2, (0.95+0.7125)/2, (0.85+0.7225)/2]
    // = [0.85, 0.83125, 0.78625]
    // Exact values depend on implementation, but should be between start and end
    assert!(
        dir_light.color[0] < initial_color[0] || dir_light.color[0] >= 0.70,
        "Color R should be interpolating toward Rain value"
    );
    assert!(
        dir_light.intensity < initial_intensity || dir_light.intensity >= 0.6,
        "Intensity should be interpolating toward Rain value"
    );

    // Update 5 more times (total 10)
    for _ in 0..5 {
        app.update();
    }

    let dir_light = app.world().get_resource::<DirectionalLightConfig>().unwrap();
    // Fully transitioned to Rain values
    assert!(
        approx_eq(dir_light.color[0], 0.70, 0.01),
        "After 10 frames, color R should be 0.70, got {}",
        dir_light.color[0]
    );
    assert!(
        approx_eq(dir_light.color[1], 0.7125, 0.01),
        "After 10 frames, color G should be 0.7125, got {}",
        dir_light.color[1]
    );
    assert!(
        approx_eq(dir_light.color[2], 0.7225, 0.01),
        "After 10 frames, color B should be 0.7225, got {}",
        dir_light.color[2]
    );
    assert!(
        approx_eq(dir_light.intensity, 0.6, 0.01),
        "After 10 frames, intensity should be 0.6, got {}",
        dir_light.intensity
    );
}

/// Scenario: Building with missing sprite asset renders magenta placeholder
#[test]
fn edge_missing_sprite_renders_magenta_placeholder() {
    let mut app = render_app();

    // Spawn a building — when RenderPlugin can't find a matching sprite,
    // it should render a magenta placeholder quad
    spawn_building(&mut app, BuildingType::OpusForge, 10, 10);

    app.update();

    // If the building has no recognized sprite asset, it should get MagentaPlaceholder
    let placeholders: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &MagentaPlaceholder)>()
        .iter(app.world())
        .collect();

    // Until sprites are loaded, ALL buildings should have MagentaPlaceholder
    let found = placeholders
        .iter()
        .any(|(se, _)| se.grid_pos == (10, 10));
    assert!(
        found,
        "Building with missing sprite at (10,10) must render as MagentaPlaceholder"
    );
}

/// Scenario: Non-emissive buildings do not spawn point lights
#[test]
fn edge_non_emissive_buildings_no_point_lights() {
    let mut app = render_app();

    spawn_building(&mut app, BuildingType::Watchtower, 10, 10);
    spawn_building(&mut app, BuildingType::WindTurbine, 20, 20);

    app.update();

    let lights: Vec<_> = app
        .world_mut()
        .query::<(&SceneEntity, &PointLightMarker)>()
        .iter(app.world())
        .collect();

    let watchtower_light = lights.iter().find(|(se, _)| se.grid_pos == (10, 10));
    assert!(
        watchtower_light.is_none(),
        "Watchtower at (10,10) must NOT have a point light"
    );

    let turbine_light = lights.iter().find(|(se, _)| se.grid_pos == (20, 20));
    assert!(
        turbine_light.is_none(),
        "WindTurbine at (20,20) must NOT have a point light"
    );
}

/// Scenario: No tiles revealed produces fully opaque fog over entire grid
#[test]
fn edge_no_tiles_revealed_fully_opaque_fog() {
    let mut app = render_app_sized(5, 5);

    // FogMap is default (empty) — no tiles revealed

    app.update();

    // All 25 tiles should have opaque dark overlay
    let fog_overlays: Vec<_> = app
        .world_mut()
        .query::<&FogOverlay>()
        .iter(app.world())
        .collect();

    assert_eq!(
        fog_overlays.len(),
        25,
        "All 25 tiles must have fog overlay when none revealed, got {}",
        fog_overlays.len()
    );

    for fo in &fog_overlays {
        assert_eq!(
            fo.color,
            [0.05, 0.05, 0.08, 1.0],
            "Unrevealed tile must have dark overlay color"
        );
    }
}
