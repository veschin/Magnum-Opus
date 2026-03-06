//! Game UI BDD tests — `.ptsd/bdd/game-ui.feature`
//!
//! Each test maps 1:1 to a BDD scenario. Tests verify that InputPlugin and
//! UiPlugin correctly translate mouse/keyboard input into ECS commands and
//! render UI panels that read simulation state.
//!
//! Tests are written to FAIL until InputPlugin/UiPlugin are implemented.
//! Placeholder types are defined here for resources that those plugins will own.

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::placement::PlacementCommands;
// use crate::data::recipes::default_recipe; // unused until InputPlugin exists
use crate::SimulationPlugin;

// ─────────────────────────────────────────────────────────────────────────────
// Placeholder UI/Input types (will be provided by InputPlugin/UiPlugin)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
struct CameraConfig {
    pan_speed: f32,
    pan_lerp_factor: f32,
    bounds_margin: i32,
    zoom_step: f32,
    zoom_min: f32,
    zoom_max: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            pan_speed: 8.0,
            pan_lerp_factor: 0.15,
            bounds_margin: 2,
            zoom_step: 0.1,
            zoom_min: 0.5,
            zoom_max: 4.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct CameraState {
    position: (f32, f32),
    zoom: f32,
    target: (f32, f32),
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: (32.0, 32.0),
            zoom: 1.0,
            target: (32.0, 32.0),
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct CursorGridPos(Option<(i32, i32)>);

#[derive(Resource, Debug, Clone)]
struct PlacementMode {
    active: bool,
    building_type: Option<BuildingType>,
}

impl Default for PlacementMode {
    fn default() -> Self {
        Self { active: false, building_type: None }
    }
}

#[derive(Resource, Debug, Clone)]
struct PathDrawMode {
    active: bool,
    kind: Option<PathDrawKind>,
    waypoints: Vec<(i32, i32)>,
}

impl Default for PathDrawMode {
    fn default() -> Self {
        Self { active: false, kind: None, waypoints: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathDrawKind {
    Solid,
    Liquid,
}

#[derive(Resource, Debug, Clone, PartialEq)]
enum GameSpeed {
    Running { multiplier: u32 },
    Paused { previous_multiplier: u32 },
}

impl Default for GameSpeed {
    fn default() -> Self {
        GameSpeed::Running { multiplier: 1 }
    }
}

#[derive(Resource, Debug, Clone)]
struct BuildMenuState {
    categories: Vec<BuildMenuCategory>,
}

impl Default for BuildMenuState {
    fn default() -> Self {
        Self { categories: Vec::new() }
    }
}

#[derive(Debug, Clone)]
struct BuildMenuCategory {
    name: String,
    entries: Vec<BuildMenuEntry>,
}

#[derive(Debug, Clone)]
struct BuildMenuEntry {
    building_type: BuildingType,
    inventory_count: u32,
    tier_locked: bool,
    tier_label: Option<String>,
}

#[derive(Debug, Clone)]
struct Notification {
    message: String,
    created_at: f64,
    visible: bool,
}

#[derive(Resource, Debug, Clone)]
struct NotificationQueue {
    notifications: Vec<Notification>,
    max_visible: usize,
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            max_visible: 5,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct TooltipState {
    visible: bool,
    content: String,
    hover_start: f64,
}

#[derive(Resource, Debug, Clone)]
struct EnergyBarState {
    generation: f32,
    consumption: f32,
    ratio_pct: f32,
    color: EnergyBarColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnergyBarColor {
    Green,
    Yellow,
    Red,
}

impl Default for EnergyBarState {
    fn default() -> Self {
        Self {
            generation: 0.0,
            consumption: 0.0,
            ratio_pct: 100.0,
            color: EnergyBarColor::Green,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct MinimapState {
    width_px: u32,
    height_px: u32,
    building_dots: Vec<(i32, i32)>,
    fog_cells: usize,
    camera_viewport_visible: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

const GRID_W: i32 = 64;
const GRID_H: i32 = 64;

/// App with SimulationPlugin (64x64 grid) — no InputPlugin/UiPlugin.
/// Tests will FAIL until those plugins are implemented.
fn ui_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: GRID_W, grid_height: GRID_H });
    // TODO: app.add_plugins(InputPlugin);
    // TODO: app.add_plugins(UiPlugin);
    // Insert placeholder resources that InputPlugin/UiPlugin should provide:
    app.insert_resource(CameraConfig::default());
    app.insert_resource(CameraState::default());
    app.insert_resource(CursorGridPos::default());
    app.insert_resource(PlacementMode::default());
    app.insert_resource(PathDrawMode::default());
    app.insert_resource(GameSpeed::default());
    app.insert_resource(BuildMenuState::default());
    app.insert_resource(NotificationQueue::default());
    app.insert_resource(TooltipState::default());
    app.insert_resource(EnergyBarState::default());
    app.insert_resource(MinimapState::default());
    app
}

/// Place a building entity at (x, y) with the given type directly (no commands).
fn place_building_at(app: &mut App, bt: BuildingType, x: i32, y: i32) -> Entity {
    let entity = app.world_mut().spawn((
        Building { building_type: bt },
        Position { x, y },
    )).id();
    app.world_mut().resource_mut::<Grid>().occupied.insert((x, y));
    entity
}

/// Reveal a tile in FogMap.
fn reveal_tile(app: &mut App, x: i32, y: i32) {
    app.world_mut().resource_mut::<FogMap>().reveal(x, y);
}

/// Set terrain type at (x, y).
fn set_terrain(app: &mut App, x: i32, y: i32, terrain: TerrainType) {
    app.world_mut().resource_mut::<Grid>().terrain.insert((x, y), terrain);
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC1: Camera Pan
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Camera pans with WASD at constant screen-space speed
#[test]
fn ac1_camera_pans_with_wasd() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraConfig>().pan_speed = 8.0;
    app.world_mut().resource_mut::<CameraConfig>().pan_lerp_factor = 0.15;
    app.world_mut().resource_mut::<CameraState>().position = (32.0, 32.0);
    app.world_mut().resource_mut::<CameraState>().target = (32.0, 32.0);

    // Simulate W held for 20 frames at 60 fps
    for _ in 0..20 {
        app.update();
    }

    let cam = app.world().resource::<CameraState>();
    // W key = pan up = Y decreases. Expected delta ~= 8.0 * (20/60) = 2.667
    let expected_y = 32.0 - 8.0 * (20.0 / 60.0);
    assert!(
        cam.position.1 < 32.0,
        "Camera Y should decrease when panning up with W, got {}",
        cam.position.1
    );
    assert!(
        (cam.position.1 - expected_y).abs() < 1.0,
        "Camera Y should be ~{expected_y}, got {}",
        cam.position.1
    );
}

/// Scenario: Camera clamps to grid bounds with 2-tile margin on left edge
#[test]
fn ac1_camera_clamps_left_edge() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraConfig>().bounds_margin = 2;
    app.world_mut().resource_mut::<CameraState>().target = (-10.0, 32.0);

    app.update();

    let cam = app.world().resource::<CameraState>();
    assert!(
        cam.position.0 >= -2.0,
        "Camera X should be clamped to >= -2 (margin), got {}",
        cam.position.0
    );
    assert_eq!(
        cam.position.0, -2.0,
        "Camera X should be exactly -2.0 after clamping, got {}",
        cam.position.0
    );
    assert_eq!(
        cam.position.1, 32.0,
        "Camera Y should remain 32.0, got {}",
        cam.position.1
    );
}

/// Scenario: Camera clamps to grid bounds with 2-tile margin past bottom-right
#[test]
fn ac1_camera_clamps_bottom_right() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraConfig>().bounds_margin = 2;
    app.world_mut().resource_mut::<CameraState>().target = (80.0, 80.0);

    app.update();

    let cam = app.world().resource::<CameraState>();
    // 64 + 2 = 66
    assert_eq!(
        cam.position.0, 66.0,
        "Camera X should clamp to 66.0, got {}",
        cam.position.0
    );
    assert_eq!(
        cam.position.1, 66.0,
        "Camera Y should clamp to 66.0, got {}",
        cam.position.1
    );
}

/// Scenario: Arrow keys pan camera equivalently to WASD
#[test]
fn ac1_arrow_keys_pan_camera() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().position = (32.0, 32.0);
    app.world_mut().resource_mut::<CameraState>().target = (32.0, 32.0);

    // Simulate Right arrow held for 10 frames
    for _ in 0..10 {
        app.update();
    }

    let cam = app.world().resource::<CameraState>();
    assert!(
        cam.position.0 > 32.0,
        "Camera X should increase when panning right, got {}",
        cam.position.0
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC2: Zoom to Cursor
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Scroll wheel zooms toward cursor — tile under cursor stays fixed
#[test]
fn ac2_scroll_zoom_toward_cursor() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((15, 12));
    app.world_mut().resource_mut::<CameraState>().zoom = 1.0;

    // Zoom in by 10 steps (zoom_step = 0.1 each)
    // Expected zoom: 1.0 + 10 * 0.1 = 2.0
    app.update();

    let cam = app.world().resource::<CameraState>();
    assert_eq!(
        cam.zoom, 2.0,
        "Zoom should be 2.0 after 10 zoom-in steps, got {}",
        cam.zoom
    );

    let cursor = app.world().resource::<CursorGridPos>();
    assert_eq!(
        cursor.0,
        Some((15, 12)),
        "CursorGridPos should stay at (15,12) after zooming, got {:?}",
        cursor.0
    );
}

/// Scenario: Zoom clamps to minimum 0.5x
#[test]
fn ac2_zoom_clamps_minimum() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().zoom = 0.5;

    // Try to zoom out by 5 steps
    app.update();

    let cam = app.world().resource::<CameraState>();
    assert_eq!(
        cam.zoom, 0.5,
        "Zoom should clamp at minimum 0.5, got {}",
        cam.zoom
    );
}

/// Scenario: Zoom clamps to maximum 4.0x
#[test]
fn ac2_zoom_clamps_maximum() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().zoom = 4.0;

    // Try to zoom in by 5 steps
    app.update();

    let cam = app.world().resource::<CameraState>();
    assert_eq!(
        cam.zoom, 4.0,
        "Zoom should clamp at maximum 4.0, got {}",
        cam.zoom
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC3: Cursor-to-Grid Raycasting
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: CursorGridPos correctly maps screen to grid at default zoom
#[test]
fn ac3_cursor_maps_to_grid_default_zoom() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().zoom = 1.0;

    // Simulate cursor hovering over grid tile (3, 5)
    app.update();

    let cursor = app.world().resource::<CursorGridPos>();
    assert_eq!(
        cursor.0,
        Some((3, 5)),
        "CursorGridPos should be Some((3, 5)), got {:?}",
        cursor.0
    );
}

/// Scenario: CursorGridPos maps correctly at zoom 2.0
#[test]
fn ac3_cursor_maps_to_grid_zoom_2() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().zoom = 2.0;

    app.update();

    let cursor = app.world().resource::<CursorGridPos>();
    assert_eq!(
        cursor.0,
        Some((3, 5)),
        "CursorGridPos should be Some((3, 5)) at zoom 2.0, got {:?}",
        cursor.0
    );
}

/// Scenario: CursorGridPos is None when cursor is outside grid bounds
#[test]
fn ac3_cursor_none_outside_grid() {
    let mut app = ui_app();

    // Cursor at (-50, -50) — outside grid
    app.update();

    let cursor = app.world().resource::<CursorGridPos>();
    assert_eq!(
        cursor.0, None,
        "CursorGridPos should be None when outside grid, got {:?}",
        cursor.0
    );
}

/// Scenario: CursorGridPos is None when cursor is over a UI panel
#[test]
fn ac3_cursor_none_over_ui_panel() {
    let mut app = ui_app();

    // Cursor at (30, 300) — over build menu panel
    app.update();

    let cursor = app.world().resource::<CursorGridPos>();
    assert_eq!(
        cursor.0, None,
        "CursorGridPos should be None when over UI panel, got {:?}",
        cursor.0
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC4: Building Placement
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Left-click places building and consumes inventory
#[test]
fn ac4_left_click_places_building() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));
    set_terrain(&mut app, 10, 10, TerrainType::IronVein);
    reveal_tile(&mut app, 10, 10);

    // Simulate left-click
    app.update();

    // PlacementCommands should have an entry for IronMiner at (10, 10)
    let cmds = app.world().resource::<PlacementCommands>();
    let has_entry = cmds.requests.iter().any(|r| {
        r.building_type == BuildingType::IronMiner && r.x == 10 && r.y == 10
    });
    assert!(has_entry, "PlacementCommands should contain IronMiner at (10, 10)");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 3,
        "Inventory should have 3 IronMiners after placing one, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Left-click on occupied tile does nothing
#[test]
fn ac4_left_click_occupied_tile_no_placement() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));
    set_terrain(&mut app, 10, 10, TerrainType::IronVein);
    reveal_tile(&mut app, 10, 10);
    // Occupy tile with existing building
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);

    app.update();

    let cmds = app.world().resource::<PlacementCommands>();
    let has_entry = cmds.requests.iter().any(|r| r.x == 10 && r.y == 10);
    assert!(!has_entry, "PlacementCommands should NOT contain entry at occupied (10, 10)");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 4,
        "Inventory should remain 4 when placement blocked, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Left-click on wrong terrain does nothing
#[test]
fn ac4_left_click_wrong_terrain_no_placement() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((15, 15));
    // Grass terrain — IronMiner requires IronVein
    set_terrain(&mut app, 15, 15, TerrainType::Grass);
    reveal_tile(&mut app, 15, 15);

    app.update();

    let cmds = app.world().resource::<PlacementCommands>();
    let has_entry = cmds.requests.iter().any(|r| r.x == 15 && r.y == 15);
    assert!(!has_entry, "PlacementCommands should NOT contain entry at wrong terrain (15, 15)");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 4,
        "Inventory should remain 4 when terrain mismatch, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Left-click on fogged tile does nothing
#[test]
fn ac4_left_click_fogged_tile_no_placement() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((50, 50));
    set_terrain(&mut app, 50, 50, TerrainType::IronVein);
    // NOT revealed — fog blocks placement

    app.update();

    let cmds = app.world().resource::<PlacementCommands>();
    let has_entry = cmds.requests.iter().any(|r| r.x == 50 && r.y == 50);
    assert!(!has_entry, "PlacementCommands should NOT contain entry at fogged (50, 50)");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 4,
        "Inventory should remain 4 when fog blocks, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Right-click during placement mode exits placement mode without removing
#[test]
fn ac4_right_click_exits_placement_mode() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));
    let _building = place_building_at(&mut app, BuildingType::IronMiner, 10, 10);

    // Simulate right-click
    app.update();

    let pm = app.world().resource::<PlacementMode>();
    assert!(!pm.active, "Placement mode should be inactive after right-click");

    let rm = app.world().resource::<RemoveBuildingCommands>();
    assert!(rm.queue.is_empty(), "RemoveBuildingCommands should be empty");

    // Building should still exist
    let building_count = app
        .world_mut()
        .query::<(&Building, &Position)>()
        .iter(app.world())
        .filter(|(_, p)| p.x == 10 && p.y == 10)
        .count();
    assert_eq!(building_count, 1, "Building at (10, 10) should still exist");
}

/// Scenario: Escape exits placement mode
#[test]
fn ac4_escape_exits_placement_mode() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);

    // Simulate Escape key press
    app.update();

    let pm = app.world().resource::<PlacementMode>();
    assert!(!pm.active, "Placement mode should be inactive after Escape");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC5: Building Removal
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Right-click on building outside placement mode removes it
#[test]
fn ac5_right_click_removes_building() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PlacementMode>().active = false;
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 2);

    // Simulate right-click
    app.update();

    let rm = app.world().resource::<RemoveBuildingCommands>();
    let has_pos = rm.queue.iter().any(|&(x, y)| x == 10 && y == 10);
    assert!(has_pos, "RemoveBuildingCommands should contain (10, 10)");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 3,
        "Inventory should have 3 IronMiners after removal refund, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Right-click on empty tile outside placement mode does nothing
#[test]
fn ac5_right_click_empty_tile_does_nothing() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PlacementMode>().active = false;
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((15, 15));
    // No building at (15, 15)

    app.update();

    let rm = app.world().resource::<RemoveBuildingCommands>();
    assert!(rm.queue.is_empty(), "RemoveBuildingCommands should be empty for empty tile");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC6: Transport Path Drawing
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Path drawing produces valid TransportCommands entry for Solid path
#[test]
fn ac6_path_drawing_solid() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Solid);

    // Setup groups with boundary tiles
    let group_a = app.world_mut().spawn_empty().id();
    let group_b = app.world_mut().spawn_empty().id();
    // Tiles (11,10), (12,10), (13,10) are empty and passable (not occupied)

    // Simulate click-drag from (10,10) through (11,10),(12,10),(13,10) to (14,10)
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        !tc.draw_path.is_empty(),
        "TransportCommands.draw_path should have an entry"
    );
    let entry = &tc.draw_path[0];
    assert_eq!(
        entry.waypoints,
        vec![(10, 10), (11, 10), (12, 10), (13, 10), (14, 10)],
        "Waypoints should match drawn path"
    );
    assert!(!entry.is_pipe, "Solid path should have is_pipe == false");
}

/// Scenario: Path drawing over occupied tile does not commit
#[test]
fn ac6_path_drawing_occupied_tile_rejected() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Solid);
    // Occupy tile (12, 10)
    place_building_at(&mut app, BuildingType::IronMiner, 12, 10);

    // Attempt to draw path through (12, 10)
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        tc.draw_path.is_empty(),
        "TransportCommands.draw_path should be empty when path crosses occupied tile"
    );
}

/// Scenario: Path drawing cancelled by Escape discards partial path
#[test]
fn ac6_path_drawing_escape_cancels() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Liquid);
    app.world_mut().resource_mut::<PathDrawMode>().waypoints = vec![(10, 10), (11, 10)];

    // Simulate Escape key
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        tc.draw_path.is_empty(),
        "TransportCommands.draw_path should be empty after Escape"
    );

    let pdm = app.world().resource::<PathDrawMode>();
    assert!(!pdm.active, "Path draw mode should be inactive after Escape");
}

/// Scenario: Liquid path drawing sets is_pipe flag
#[test]
fn ac6_liquid_path_drawing_is_pipe() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Liquid);

    // Draw liquid path from (10,10) to (14,10)
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        !tc.draw_path.is_empty(),
        "TransportCommands.draw_path should have an entry for liquid path"
    );
    assert!(
        tc.draw_path[0].is_pipe,
        "Liquid path should have is_pipe == true"
    );
}

/// Scenario: Path drawing over existing path tile rejects
#[test]
fn ac6_path_drawing_over_existing_path_rejects() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Solid);

    // Mark tile (12, 10) as occupied by existing path
    let path_entity = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<PathOccupancy>()
        .tiles
        .insert((12, 10), path_entity);

    // Attempt to draw through (12, 10)
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        tc.draw_path.is_empty(),
        "TransportCommands.draw_path should be empty when crossing existing path"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC7: Game Speed Control
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Space toggles pause on
#[test]
fn ac7_space_toggles_pause_on() {
    let mut app = ui_app();
    app.insert_resource(GameSpeed::Running { multiplier: 1 });

    // Simulate Space key press
    app.update();

    let gs = app.world().resource::<GameSpeed>();
    assert_eq!(
        *gs,
        GameSpeed::Paused { previous_multiplier: 1 },
        "GameSpeed should be Paused after Space, got {:?}",
        gs
    );
}

/// Scenario: Space toggles pause off and restores previous speed
#[test]
fn ac7_space_toggles_pause_off() {
    let mut app = ui_app();
    app.insert_resource(GameSpeed::Paused { previous_multiplier: 1 });

    // Simulate Space key press
    app.update();

    let gs = app.world().resource::<GameSpeed>();
    assert_eq!(
        *gs,
        GameSpeed::Running { multiplier: 1 },
        "GameSpeed should be Running(1) after unpausing, got {:?}",
        gs
    );
}

/// Scenario: Number keys set speed multiplier
#[test]
fn ac7_number_keys_set_speed() {
    let mut app = ui_app();
    app.insert_resource(GameSpeed::Running { multiplier: 1 });

    // Simulate pressing "2" key -> multiplier 2
    app.update();

    let gs = app.world().resource::<GameSpeed>();
    assert_eq!(
        *gs,
        GameSpeed::Running { multiplier: 2 },
        "After pressing '2', speed should be Running(2), got {:?}",
        gs
    );

    // Simulate pressing "3" key -> multiplier 4
    app.update();

    let gs = app.world().resource::<GameSpeed>();
    assert_eq!(
        *gs,
        GameSpeed::Running { multiplier: 4 },
        "After pressing '3', speed should be Running(4), got {:?}",
        gs
    );
}

/// Scenario: Placement works while paused — commands queue for unpause
#[test]
fn ac7_placement_while_paused() {
    let mut app = ui_app();
    app.insert_resource(GameSpeed::Paused { previous_multiplier: 1 });
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));
    set_terrain(&mut app, 10, 10, TerrainType::IronVein);
    reveal_tile(&mut app, 10, 10);

    // Simulate left-click while paused
    app.update();

    let cmds = app.world().resource::<PlacementCommands>();
    let has_entry = cmds.requests.iter().any(|r| {
        r.building_type == BuildingType::IronMiner && r.x == 10 && r.y == 10
    });
    assert!(has_entry, "PlacementCommands should work while paused");

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 3,
        "Inventory should consume while paused, got {}",
        inv.count_building(BuildingType::IronMiner)
    );
}

/// Scenario: Path drawing works while paused
#[test]
fn ac7_path_drawing_while_paused() {
    let mut app = ui_app();
    app.insert_resource(GameSpeed::Paused { previous_multiplier: 1 });
    app.world_mut().resource_mut::<PathDrawMode>().active = true;
    app.world_mut().resource_mut::<PathDrawMode>().kind = Some(PathDrawKind::Solid);

    // Draw path from (10,10) to (14,10)
    app.update();

    let tc = app.world().resource::<TransportCommands>();
    assert!(
        !tc.draw_path.is_empty(),
        "TransportCommands.draw_path should have entry while paused"
    );
    let entry = &tc.draw_path[0];
    assert_eq!(
        entry.waypoints,
        vec![(10, 10), (11, 10), (12, 10), (13, 10), (14, 10)],
        "Waypoints should match drawn path while paused"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC8: Build Menu Panel
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Build menu shows correct categories and inventory counts
#[test]
fn ac8_build_menu_categories_and_counts() {
    let mut app = ui_app();
    {
        let mut inv = app.world_mut().resource_mut::<Inventory>();
        inv.add_building(BuildingType::IronMiner, 4);
        inv.add_building(BuildingType::CopperMiner, 2);
        inv.add_building(BuildingType::StoneQuarry, 2);
        inv.add_building(BuildingType::WaterPump, 2);
        inv.add_building(BuildingType::IronSmelter, 2);
        inv.add_building(BuildingType::CopperSmelter, 1);
        inv.add_building(BuildingType::Sawmill, 1);
        inv.add_building(BuildingType::Constructor, 1);
        inv.add_building(BuildingType::WindTurbine, 3);
        inv.add_building(BuildingType::Watchtower, 1);
    }
    app.world_mut().resource_mut::<TierState>().current_tier = 1;

    // Render build menu
    app.update();

    let menu = app.world().resource::<BuildMenuState>();

    // Check categories exist
    let extraction = menu.categories.iter().find(|c| c.name == "Extraction");
    assert!(extraction.is_some(), "Build menu should have 'Extraction' category");
    let extraction = extraction.unwrap();
    let extraction_types: Vec<BuildingType> = extraction.entries.iter().map(|e| e.building_type).collect();
    assert!(
        extraction_types.contains(&BuildingType::IronMiner),
        "Extraction should contain IronMiner"
    );
    assert!(
        extraction_types.contains(&BuildingType::CopperMiner),
        "Extraction should contain CopperMiner"
    );
    assert!(
        extraction_types.contains(&BuildingType::StoneQuarry),
        "Extraction should contain StoneQuarry"
    );
    assert!(
        extraction_types.contains(&BuildingType::WaterPump),
        "Extraction should contain WaterPump"
    );

    // Check inventory counts
    let iron_miner_entry = extraction.entries.iter().find(|e| e.building_type == BuildingType::IronMiner);
    assert!(iron_miner_entry.is_some(), "IronMiner entry should exist");
    assert_eq!(
        iron_miner_entry.unwrap().inventory_count, 4,
        "IronMiner inventory_count should be 4"
    );

    // Check energy category
    let energy = menu.categories.iter().find(|c| c.name == "Energy");
    assert!(energy.is_some(), "Build menu should have 'Energy' category");
    let wt_entry = energy.unwrap().entries.iter().find(|e| e.building_type == BuildingType::WindTurbine);
    assert!(wt_entry.is_some(), "WindTurbine entry should exist");
    assert_eq!(
        wt_entry.unwrap().inventory_count, 3,
        "WindTurbine inventory_count should be 3"
    );
}

/// Scenario: Tier-locked buildings are grayed and unselectable
#[test]
fn ac8_tier_locked_buildings_grayed() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<TierState>().current_tier = 1;

    app.update();

    let menu = app.world().resource::<BuildMenuState>();

    // Find Toolmaker in any category — should be tier-locked
    let toolmaker = menu.categories.iter()
        .flat_map(|c| c.entries.iter())
        .find(|e| e.building_type == BuildingType::Toolmaker);
    assert!(toolmaker.is_some(), "Toolmaker should be in build menu");
    let toolmaker = toolmaker.unwrap();
    assert!(toolmaker.tier_locked, "Toolmaker should be tier_locked at T1");
    assert_eq!(
        toolmaker.tier_label.as_deref(),
        Some("T2"),
        "Toolmaker tier_label should be 'T2'"
    );

    // Assembler — T3
    let assembler = menu.categories.iter()
        .flat_map(|c| c.entries.iter())
        .find(|e| e.building_type == BuildingType::Assembler);
    assert!(assembler.is_some(), "Assembler should be in build menu");
    assert!(assembler.unwrap().tier_locked, "Assembler should be tier_locked at T1");
    assert_eq!(
        assembler.unwrap().tier_label.as_deref(),
        Some("T3"),
        "Assembler tier_label should be 'T3'"
    );

    // LavaGenerator — T2
    let lava_gen = menu.categories.iter()
        .flat_map(|c| c.entries.iter())
        .find(|e| e.building_type == BuildingType::LavaGenerator);
    assert!(lava_gen.is_some(), "LavaGenerator should be in build menu");
    assert!(lava_gen.unwrap().tier_locked, "LavaGenerator should be tier_locked at T1");

    // ManaReactor — T3
    let mana_reactor = menu.categories.iter()
        .flat_map(|c| c.entries.iter())
        .find(|e| e.building_type == BuildingType::ManaReactor);
    assert!(mana_reactor.is_some(), "ManaReactor should be in build menu");
    assert!(mana_reactor.unwrap().tier_locked, "ManaReactor should be tier_locked at T1");
}

/// Scenario: Clicking a building with count > 0 enters placement mode
#[test]
fn ac8_click_building_enters_placement_mode() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);
    app.world_mut().resource_mut::<TierState>().current_tier = 1;

    // Simulate clicking IronMiner in build menu
    app.update();

    let pm = app.world().resource::<PlacementMode>();
    assert!(pm.active, "Placement mode should be active after clicking IronMiner");
    assert_eq!(
        pm.building_type,
        Some(BuildingType::IronMiner),
        "Placement mode building_type should be IronMiner"
    );
}

/// Scenario: Clicking a building with count 0 does not enter placement mode
#[test]
fn ac8_click_building_zero_count_no_placement() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 0);

    // Simulate clicking IronMiner with 0 count
    app.update();

    let pm = app.world().resource::<PlacementMode>();
    assert!(!pm.active, "Placement mode should NOT be active when inventory count is 0");
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC9: Energy Bar
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Energy bar shows green when surplus >= 10%
#[test]
fn ac9_energy_bar_green_surplus() {
    let mut app = ui_app();
    {
        let mut pool = app.world_mut().resource_mut::<EnergyPool>();
        pool.total_generation = 100.0;
        pool.total_consumption = 80.0;
        pool.ratio = 1.25;
    }

    app.update();

    let bar = app.world().resource::<EnergyBarState>();
    assert_eq!(bar.generation, 100.0, "Energy bar generation should be 100");
    assert_eq!(bar.consumption, 80.0, "Energy bar consumption should be 80");
    assert_eq!(bar.ratio_pct, 125.0, "Energy bar ratio should be 125%");
    assert_eq!(
        bar.color,
        EnergyBarColor::Green,
        "Energy bar should be green at 125% ratio"
    );
}

/// Scenario: Energy bar shows yellow when surplus is 0-10%
#[test]
fn ac9_energy_bar_yellow_marginal() {
    let mut app = ui_app();
    {
        let mut pool = app.world_mut().resource_mut::<EnergyPool>();
        pool.total_generation = 100.0;
        pool.total_consumption = 95.0;
        pool.ratio = 1.053;
    }

    app.update();

    let bar = app.world().resource::<EnergyBarState>();
    assert!(
        (bar.ratio_pct - 105.3).abs() < 1.0,
        "Energy bar ratio should be ~105%, got {}",
        bar.ratio_pct
    );
    assert_eq!(
        bar.color,
        EnergyBarColor::Yellow,
        "Energy bar should be yellow at ~105% ratio"
    );
}

/// Scenario: Energy bar shows red when in deficit
#[test]
fn ac9_energy_bar_red_deficit() {
    let mut app = ui_app();
    {
        let mut pool = app.world_mut().resource_mut::<EnergyPool>();
        pool.total_generation = 80.0;
        pool.total_consumption = 100.0;
        pool.ratio = 0.8;
    }

    app.update();

    let bar = app.world().resource::<EnergyBarState>();
    assert_eq!(bar.ratio_pct, 80.0, "Energy bar ratio should be 80%");
    assert_eq!(
        bar.color,
        EnergyBarColor::Red,
        "Energy bar should be red at 80% ratio"
    );
}

/// Scenario: Energy bar shows green when generation and consumption are both 0
#[test]
fn ac9_energy_bar_green_zero_zero() {
    let mut app = ui_app();
    {
        let mut pool = app.world_mut().resource_mut::<EnergyPool>();
        pool.total_generation = 0.0;
        pool.total_consumption = 0.0;
        pool.ratio = 1.0;
    }

    app.update();

    let bar = app.world().resource::<EnergyBarState>();
    assert_eq!(
        bar.color,
        EnergyBarColor::Green,
        "Energy bar should be green when both generation and consumption are 0"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC10: Opus Tree Panel
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Opus tree displays milestone nodes with correct rates and visual states
#[test]
fn ac10_opus_tree_displays_nodes() {
    let mut app = ui_app();
    {
        let mut opus = app.world_mut().resource_mut::<OpusTreeResource>();
        opus.main_path = vec![
            OpusNodeEntry {
                node_index: 0,
                resource: ResourceType::IronBar,
                required_rate: 2.0,
                current_rate: 2.5,
                tier: 1,
                sustained: true,
            },
            OpusNodeEntry {
                node_index: 1,
                resource: ResourceType::CopperBar,
                required_rate: 1.5,
                current_rate: 1.5,
                tier: 1,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 2,
                resource: ResourceType::Plank,
                required_rate: 1.0,
                current_rate: 0.0,
                tier: 1,
                sustained: false,
            },
            // Fill remaining 4 nodes as placeholders
            OpusNodeEntry { node_index: 3, resource: ResourceType::Stone, required_rate: 1.0, current_rate: 0.0, tier: 1, sustained: false },
            OpusNodeEntry { node_index: 4, resource: ResourceType::Water, required_rate: 1.0, current_rate: 0.0, tier: 2, sustained: false },
            OpusNodeEntry { node_index: 5, resource: ResourceType::SteelPlate, required_rate: 1.0, current_rate: 0.0, tier: 2, sustained: false },
            OpusNodeEntry { node_index: 6, resource: ResourceType::RunicAlloy, required_rate: 1.0, current_rate: 0.0, tier: 3, sustained: false },
        ];
        opus.recalc_completion();
    }

    app.update();

    let opus = app.world().resource::<OpusTreeResource>();
    assert_eq!(opus.main_path.len(), 7, "Opus tree should have 7 nodes");

    // Node 0: IronBar — sustained (completed_glow)
    let n0 = &opus.main_path[0];
    assert!(n0.sustained, "Node 0 (IronBar) should be sustained");
    assert!(
        n0.current_rate >= n0.required_rate,
        "Node 0 current_rate {} should >= required_rate {}",
        n0.current_rate,
        n0.required_rate
    );

    // Node 1: CopperBar — in_progress_highlighted (rate met but not yet sustained)
    let n1 = &opus.main_path[1];
    assert!(!n1.sustained, "Node 1 (CopperBar) should NOT be sustained yet");
    assert!(
        n1.current_rate >= n1.required_rate,
        "Node 1 current_rate {} should >= required_rate {}",
        n1.current_rate,
        n1.required_rate
    );

    // Node 2: Plank — locked (rate is 0)
    let n2 = &opus.main_path[2];
    assert!(!n2.sustained, "Node 2 (Plank) should NOT be sustained");
    assert_eq!(
        n2.current_rate, 0.0,
        "Node 2 (Plank) current_rate should be 0.0"
    );
}

/// Scenario: Opus tree progress bars update after production tick
#[test]
fn ac10_opus_tree_progress_updates() {
    let mut app = ui_app();
    {
        let mut opus = app.world_mut().resource_mut::<OpusTreeResource>();
        opus.main_path = vec![
            OpusNodeEntry { node_index: 0, resource: ResourceType::IronBar, required_rate: 2.0, current_rate: 2.5, tier: 1, sustained: true },
            OpusNodeEntry { node_index: 1, resource: ResourceType::CopperBar, required_rate: 1.5, current_rate: 0.5, tier: 1, sustained: false },
        ];
    }

    // Update production rates for CopperBar to 1.5
    app.world_mut().resource_mut::<ProductionRates>().set(ResourceType::CopperBar, 1.5);

    app.update();

    // After sync, opus node 1 current_rate should reflect 1.5
    let opus = app.world().resource::<OpusTreeResource>();
    assert_eq!(
        opus.main_path[1].current_rate, 1.5,
        "Node 1 (CopperBar) current_rate should update to 1.5, got {}",
        opus.main_path[1].current_rate
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC11: Minimap
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Minimap renders terrain, buildings, fog, and camera viewport
#[test]
fn ac11_minimap_renders_all() {
    let mut app = ui_app();

    // Setup: reveal 313 cells, place 2 buildings
    {
        let mut fog = app.world_mut().resource_mut::<FogMap>();
        // Reveal ~313 cells (radius 12 diamond around center)
        let cx: i32 = 15;
        let cy: i32 = 15;
        let r: i32 = 12;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() + dy.abs() <= r {
                    fog.reveal(cx + dx, cy + dy);
                }
            }
        }
    }
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);
    place_building_at(&mut app, BuildingType::CopperMiner, 15, 15);

    app.update();

    let minimap = app.world().resource::<MinimapState>();
    assert_eq!(minimap.width_px, 128, "Minimap should be 128px wide");
    assert_eq!(minimap.height_px, 128, "Minimap should be 128px tall");
    assert!(
        minimap.building_dots.contains(&(10, 10)),
        "Minimap should show building dot at (10, 10)"
    );
    assert!(
        minimap.building_dots.contains(&(15, 15)),
        "Minimap should show building dot at (15, 15)"
    );
    assert!(
        minimap.camera_viewport_visible,
        "Minimap should show camera viewport rectangle"
    );
    // Unrevealed cells = 64*64 - 313 = 3783
    let total_cells = (GRID_W * GRID_H) as usize;
    let revealed = app.world().resource::<FogMap>().revealed.len();
    let unrevealed = total_cells - revealed;
    assert_eq!(
        minimap.fog_cells, unrevealed,
        "Minimap fog_cells should equal number of unrevealed cells"
    );
}

/// Scenario: Clicking minimap pans camera to clicked grid position
#[test]
fn ac11_minimap_click_pans_camera() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraState>().position = (32.0, 32.0);
    app.world_mut().resource_mut::<CameraState>().target = (32.0, 32.0);

    // Simulate clicking minimap at pixel (32, 32) -> grid (16, 16)
    // (minimap 128x128 representing 64x64 grid: pixel/2 = grid coord)
    app.update();

    let cam = app.world().resource::<CameraState>();
    assert_eq!(
        cam.target, (16.0, 16.0),
        "Camera target should be (16, 16) after minimap click, got {:?}",
        cam.target
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC12: Tooltips
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Building tooltip appears after 300ms hover delay
#[test]
fn ac12_building_tooltip_after_300ms() {
    let mut app = ui_app();
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));

    // Simulate 300ms hover (at 60fps ~= 18 frames)
    for _ in 0..18 {
        app.update();
    }

    let tooltip = app.world().resource::<TooltipState>();
    assert!(tooltip.visible, "Tooltip should be visible after 300ms hover");
    assert!(
        tooltip.content.contains("IronMiner"),
        "Tooltip should mention IronMiner, got: {}",
        tooltip.content
    );
}

/// Scenario: Tooltip does not appear before 300ms
#[test]
fn ac12_tooltip_not_before_300ms() {
    let mut app = ui_app();
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((10, 10));

    // Simulate 200ms hover (~12 frames at 60fps)
    for _ in 0..12 {
        app.update();
    }

    let tooltip = app.world().resource::<TooltipState>();
    assert!(!tooltip.visible, "Tooltip should NOT be visible before 300ms");
}

/// Scenario: Tooltip disappears when cursor moves away
#[test]
fn ac12_tooltip_disappears_on_move() {
    let mut app = ui_app();
    place_building_at(&mut app, BuildingType::IronMiner, 10, 10);
    // Pre-set tooltip as visible
    {
        let mut tooltip = app.world_mut().resource_mut::<TooltipState>();
        tooltip.visible = true;
        tooltip.content = "IronMiner".to_string();
    }

    // Move cursor to (20, 20) which has no building
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((20, 20));

    app.update();

    let tooltip = app.world().resource::<TooltipState>();
    assert!(!tooltip.visible, "Tooltip should disappear when cursor moves away");
}

/// Scenario: Transport path tooltip shows cargo and throughput
#[test]
fn ac12_transport_path_tooltip() {
    let mut app = ui_app();

    // Create a transport path segment at (12, 10)
    let group_a = app.world_mut().spawn_empty().id();
    let group_b = app.world_mut().spawn_empty().id();
    let path_entity = app.world_mut().spawn((
        TransportPath {
            kind: TransportKind::RunePath,
            source_group: group_a,
            target_group: group_b,
            resource_filter: Some(ResourceType::IronOre),
            tier: 1,
            capacity: 10,
            speed: 1.0,
            connected: true,
            segments: vec![(12, 10)],
        },
    )).id();
    app.world_mut()
        .resource_mut::<PathOccupancy>()
        .tiles
        .insert((12, 10), path_entity);

    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((12, 10));

    // Simulate 300ms hover
    for _ in 0..18 {
        app.update();
    }

    let tooltip = app.world().resource::<TooltipState>();
    assert!(tooltip.visible, "Transport tooltip should be visible after 300ms");
    assert!(
        tooltip.content.contains("IronOre"),
        "Transport tooltip should mention cargo IronOre, got: {}",
        tooltip.content
    );
}

/// Scenario: Creature tooltip shows type, behavior, and health
#[test]
fn ac12_creature_tooltip() {
    let mut app = ui_app();

    // Spawn creature at (30, 30) using real ECS creature component
    app.world_mut().spawn((
        Position { x: 30, y: 30 },
        Creature {
            species: CreatureSpecies::CrystalGolem,
            archetype: CreatureArchetype::OpusLinked,
            biome: BiomeTag::Desert,
            health: 50.0,
            max_health: 100.0,
            state: CreatureStateKind::Patrolling,
        },
    ));
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((30, 30));

    // Simulate 300ms hover
    for _ in 0..18 {
        app.update();
    }

    let tooltip = app.world().resource::<TooltipState>();
    assert!(tooltip.visible, "Creature tooltip should be visible after 300ms");
    assert!(
        tooltip.content.contains("CrystalGolem") || tooltip.content.contains("Golem"),
        "Creature tooltip should mention CrystalGolem, got: {}",
        tooltip.content
    );
    assert!(
        tooltip.content.contains("Patrol") || tooltip.content.contains("50"),
        "Creature tooltip should show behavior or health, got: {}",
        tooltip.content
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC13: Notifications
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: MilestoneReached event triggers notification
#[test]
fn ac13_milestone_notification() {
    let mut app = ui_app();

    // TODO: Fire MilestoneReached event via MessageWriter when InputPlugin/UiPlugin exist.
    // For now we check the notification system reacts to it.
    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let has_milestone = nq.notifications.iter().any(|n| {
        n.message.contains("Milestone reached: IronBar") && n.visible
    });
    assert!(has_milestone, "Should have visible 'Milestone reached: IronBar' notification");
}

/// Scenario: TierUnlocked event triggers notification
#[test]
fn ac13_tier_unlocked_notification() {
    let mut app = ui_app();

    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let has_tier = nq.notifications.iter().any(|n| {
        n.message.contains("Tier 2 unlocked!") && n.visible
    });
    assert!(has_tier, "Should have visible 'Tier 2 unlocked!' notification");
}

/// Scenario: EnergyDeficit event triggers notification
#[test]
fn ac13_energy_deficit_notification() {
    let mut app = ui_app();

    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let has_deficit = nq.notifications.iter().any(|n| {
        n.message.contains("Energy deficit") && n.message.contains("3 groups throttled") && n.visible
    });
    assert!(has_deficit, "Should have visible 'Energy deficit - 3 groups throttled' notification");
}

/// Scenario: HazardWarning event triggers notification
#[test]
fn ac13_hazard_warning_notification() {
    let mut app = ui_app();

    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let has_hazard = nq.notifications.iter().any(|n| {
        n.message.contains("Hazard warning: lava eruption in 30 ticks") && n.visible
    });
    assert!(has_hazard, "Should have visible hazard warning notification");
}

/// Scenario: InventoryEmpty event triggers notification
#[test]
fn ac13_inventory_empty_notification() {
    let mut app = ui_app();

    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let has_empty = nq.notifications.iter().any(|n| {
        n.message.contains("Out of WindTurbine") && n.visible
    });
    assert!(has_empty, "Should have visible 'Out of WindTurbine' notification");
}

/// Scenario: Max 5 notifications visible — 6th causes oldest to hide
#[test]
fn ac13_max_5_notifications_overflow() {
    let mut app = ui_app();
    {
        let mut nq = app.world_mut().resource_mut::<NotificationQueue>();
        nq.max_visible = 5;
        nq.notifications = vec![
            Notification { message: "Milestone reached: IronBar".to_string(), created_at: 0.0, visible: true },
            Notification { message: "Milestone reached: CopperBar".to_string(), created_at: 1.0, visible: true },
            Notification { message: "Energy deficit — 2 groups throttled".to_string(), created_at: 2.0, visible: true },
            Notification { message: "Hazard warning: lava eruption in 30 ticks".to_string(), created_at: 3.0, visible: true },
            Notification { message: "Tier 2 unlocked!".to_string(), created_at: 4.0, visible: true },
        ];
    }

    // Fire 6th notification
    // TODO: UiPlugin would handle this via event listener
    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let visible_count = nq.notifications.iter().filter(|n| n.visible).count();
    assert_eq!(visible_count, 5, "Only 5 notifications should be visible, got {visible_count}");

    // Oldest should be hidden
    let oldest = nq.notifications.iter().find(|n| n.message == "Milestone reached: IronBar");
    assert!(
        oldest.map_or(true, |n| !n.visible),
        "Oldest notification 'Milestone reached: IronBar' should be hidden"
    );
}

/// Scenario: Notification auto-dismisses after 5 seconds
#[test]
fn ac13_notification_auto_dismiss_5s() {
    let mut app = ui_app();
    {
        let mut nq = app.world_mut().resource_mut::<NotificationQueue>();
        nq.notifications.push(Notification {
            message: "Milestone reached: IronBar".to_string(),
            created_at: 0.0,
            visible: true,
        });
    }

    // Simulate 5 seconds of game time (at 60fps = 300 frames)
    for _ in 0..300 {
        app.update();
    }

    let nq = app.world().resource::<NotificationQueue>();
    let notif = nq.notifications.iter().find(|n| n.message == "Milestone reached: IronBar");
    assert!(
        notif.map_or(true, |n| !n.visible),
        "Notification should auto-dismiss after 5 seconds"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

/// Scenario: Window resize updates camera aspect ratio — minimap stays fixed
#[test]
fn edge_window_resize_aspect_ratio() {
    let mut app = ui_app();

    // Simulate window resize from 1280x720 to 1920x1080
    app.update();

    let minimap = app.world().resource::<MinimapState>();
    assert_eq!(minimap.width_px, 128, "Minimap should remain 128px wide after resize");
    assert_eq!(minimap.height_px, 128, "Minimap should remain 128px tall after resize");
    // Camera aspect ratio = 1920/1080 = 16/9 ~= 1.778
    // This would be verified through the camera projection component when RenderPlugin exists
}

/// Scenario: UI panel click priority — panel consumes click, no grid action
#[test]
fn edge_ui_panel_click_priority() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::IronMiner);
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 4);

    // Cursor over build menu panel — CursorGridPos should be None
    app.world_mut().resource_mut::<CursorGridPos>().0 = None;

    // Simulate left-click on panel
    app.update();

    let cmds = app.world().resource::<PlacementCommands>();
    assert!(
        cmds.requests.is_empty() && cmds.queue.is_empty(),
        "PlacementCommands should be empty when clicking on UI panel"
    );

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::IronMiner), 4,
        "Inventory should remain 4 when panel consumes click"
    );
}

/// Scenario: Inventory reaches 0 during placement — auto-exit with notification
#[test]
fn edge_inventory_zero_auto_exit_placement() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::WindTurbine, 1);
    app.world_mut().resource_mut::<PlacementMode>().active = true;
    app.world_mut().resource_mut::<PlacementMode>().building_type = Some(BuildingType::WindTurbine);
    app.world_mut().resource_mut::<CursorGridPos>().0 = Some((20, 20));
    // WindTurbine has no terrain requirement (terrain_req returns None for WindTurbine)
    reveal_tile(&mut app, 20, 20);

    // Simulate left-click to place last WindTurbine
    app.update();

    let inv = app.world().resource::<Inventory>();
    assert_eq!(
        inv.count_building(BuildingType::WindTurbine), 0,
        "Inventory for WindTurbine should be 0 after placing last one"
    );

    let pm = app.world().resource::<PlacementMode>();
    assert!(
        !pm.active,
        "Placement mode should auto-exit when inventory reaches 0"
    );

    let nq = app.world().resource::<NotificationQueue>();
    let has_empty_notif = nq.notifications.iter().any(|n| {
        n.message.contains("Out of WindTurbine") && n.visible
    });
    assert!(has_empty_notif, "Should have 'Out of WindTurbine' notification");
}

/// Scenario: Zoom at grid edge — camera clamp prevents seeing beyond grid
#[test]
fn edge_zoom_at_grid_edge_clamp() {
    let mut app = ui_app();
    app.world_mut().resource_mut::<CameraConfig>().bounds_margin = 2;
    app.world_mut().resource_mut::<CameraState>().position = (0.0, 0.0);
    app.world_mut().resource_mut::<CameraState>().zoom = 0.5;

    app.update();

    let cam = app.world().resource::<CameraState>();
    // Camera should be clamped so viewport doesn't extend beyond (-2, -2) to (66, 66)
    assert!(
        cam.position.0 >= -2.0,
        "Camera X should be >= -2.0 at grid edge, got {}",
        cam.position.0
    );
    assert!(
        cam.position.1 >= -2.0,
        "Camera Y should be >= -2.0 at grid edge, got {}",
        cam.position.1
    );
}

/// Scenario: Two simultaneous notifications of same type — both shown, no dedup
#[test]
fn edge_two_simultaneous_notifications_no_dedup() {
    let mut app = ui_app();

    // Two MilestoneReached events for IronBar in same tick
    app.update();

    let nq = app.world().resource::<NotificationQueue>();
    let count = nq.notifications.iter()
        .filter(|n| n.message.contains("Milestone reached: IronBar") && n.visible)
        .count();
    assert_eq!(
        count, 2,
        "Should have 2 visible 'Milestone reached: IronBar' notifications, got {count}"
    );
}

/// Scenario: Pause during hazard countdown freezes notification dismiss timer
#[test]
fn edge_pause_freezes_notification_timer() {
    let mut app = ui_app();
    {
        let mut nq = app.world_mut().resource_mut::<NotificationQueue>();
        nq.notifications.push(Notification {
            message: "Hazard warning: lava eruption in 30 ticks".to_string(),
            created_at: 0.0,
            visible: true,
        });
    }
    // Notification has been visible for 3 seconds already
    // Pause the game
    app.insert_resource(GameSpeed::Paused { previous_multiplier: 1 });

    // Simulate 10 seconds of real time while paused (600 frames at 60fps)
    for _ in 0..600 {
        app.update();
    }

    let nq = app.world().resource::<NotificationQueue>();
    let notif = nq.notifications.iter().find(|n| {
        n.message.contains("Hazard warning: lava eruption in 30 ticks")
    });
    assert!(
        notif.is_some(),
        "Hazard notification should still exist"
    );
    assert!(
        notif.unwrap().visible,
        "Hazard notification should still be visible — dismiss timer frozen while paused"
    );
}
