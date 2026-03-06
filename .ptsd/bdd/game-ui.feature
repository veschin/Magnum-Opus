@feature:game-ui
Feature: User Interaction — camera, input, panels, and notifications

  The game-ui layer translates mouse/keyboard input into ECS commands
  (PlacementCommands, RemoveBuildingCommands, TransportCommands, GameSpeed)
  and renders UI panels that read simulation state (EnergyPool, Inventory,
  OpusTreeResource, FogMap, Grid). After each app.update(), ECS resources
  reflect the cumulative effect of all player actions.

  # ────────────────────────────────────────────────
  # AC1: Camera Pan
  # ────────────────────────────────────────────────

  Scenario: Camera pans with WASD at constant screen-space speed
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And CameraConfig with pan_speed 8.0 and pan_lerp_factor 0.15
    And the camera is centered at grid position (32, 32)
    When the W key is held for 20 frames at 60 fps
    Then the camera Y position has decreased (panned up) by approximately 8.0 * (20/60) screen-space units
    And the camera movement is smooth (no frame has delta > pan_speed / 30)

  Scenario: Camera clamps to grid bounds with 2-tile margin on left edge
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    And CameraConfig with bounds_margin 2
    When the camera target is set to (-10, 32)
    And the app updates once
    Then the camera position is clamped to (-2, 32)

  Scenario: Camera clamps to grid bounds with 2-tile margin past bottom-right
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    And CameraConfig with bounds_margin 2
    When the camera target is set to (80, 80)
    And the app updates once
    Then the camera position is clamped to (66, 66)

  Scenario: Arrow keys pan camera equivalently to WASD
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And the camera is centered at grid position (32, 32)
    When the Right arrow key is held for 10 frames
    Then the camera X position has increased (panned right)

  # ────────────────────────────────────────────────
  # AC2: Zoom to Cursor
  # ────────────────────────────────────────────────

  Scenario: Scroll wheel zooms toward cursor — tile under cursor stays fixed
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And the cursor is at screen position (500, 300) mapping to grid (15, 12)
    And the current zoom scale is 1.0
    When the scroll wheel zooms in by 10 steps (zoom_step 0.1 each)
    And the app updates once
    Then the zoom scale is 2.0
    And CursorGridPos is still Some((15, 12))

  Scenario: Zoom clamps to minimum 0.5x
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And the current zoom scale is 0.5
    When the scroll wheel zooms out by 5 steps
    And the app updates once
    Then the zoom scale remains 0.5

  Scenario: Zoom clamps to maximum 4.0x
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And the current zoom scale is 4.0
    When the scroll wheel zooms in by 5 steps
    And the app updates once
    Then the zoom scale remains 4.0

  # ────────────────────────────────────────────────
  # AC3: Cursor-to-Grid Raycasting
  # ────────────────────────────────────────────────

  Scenario: CursorGridPos correctly maps screen to grid at default zoom
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    And the camera is at default position with zoom 1.0
    When the cursor hovers over the screen position corresponding to grid tile (3, 5)
    And the app updates once
    Then CursorGridPos is Some((3, 5))

  Scenario: CursorGridPos maps correctly at zoom 2.0
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    And the camera zoom is 2.0
    When the cursor hovers over the screen position corresponding to grid tile (3, 5)
    And the app updates once
    Then CursorGridPos is Some((3, 5))

  Scenario: CursorGridPos is None when cursor is outside grid bounds
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    When the cursor is at screen position (-50, -50) outside the grid
    And the app updates once
    Then CursorGridPos is None

  Scenario: CursorGridPos is None when cursor is over a UI panel
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And the build menu panel is open on the left side
    When the cursor is at screen position (30, 300) over the build menu panel
    And the app updates once
    Then CursorGridPos is None

  # ────────────────────────────────────────────────
  # AC4: Building Placement
  # ────────────────────────────────────────────────

  Scenario: Left-click places building and consumes inventory
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Inventory contains IronMiner=4
    And placement mode is active for IronMiner
    And CursorGridPos is Some((10, 10))
    And tile (10, 10) has terrain IronVein, is not occupied, and is revealed
    When the player left-clicks
    And the app updates once
    Then PlacementCommands contains an entry for IronMiner at (10, 10)
    And Inventory count for IronMiner equals 3

  Scenario: Left-click on occupied tile does nothing
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Inventory contains IronMiner=4
    And placement mode is active for IronMiner
    And CursorGridPos is Some((10, 10))
    And tile (10, 10) is occupied by an existing building
    When the player left-clicks
    And the app updates once
    Then PlacementCommands does not contain an entry at (10, 10)
    And Inventory count for IronMiner equals 4

  Scenario: Left-click on wrong terrain does nothing
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Inventory contains IronMiner=4
    And placement mode is active for IronMiner
    And CursorGridPos is Some((15, 15))
    And tile (15, 15) has terrain Grass
    When the player left-clicks
    And the app updates once
    Then PlacementCommands does not contain an entry at (15, 15)
    And Inventory count for IronMiner equals 4

  Scenario: Left-click on fogged tile does nothing
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Inventory contains IronMiner=4
    And placement mode is active for IronMiner
    And CursorGridPos is Some((50, 50))
    And tile (50, 50) is not revealed in FogMap
    When the player left-clicks
    And the app updates once
    Then PlacementCommands does not contain an entry at (50, 50)
    And Inventory count for IronMiner equals 4

  Scenario: Right-click during placement mode exits placement mode without removing
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And placement mode is active for IronMiner
    And CursorGridPos is Some((10, 10))
    And tile (10, 10) has an existing building
    When the player right-clicks
    And the app updates once
    Then placement mode is inactive
    And RemoveBuildingCommands is empty
    And the building at (10, 10) still exists

  Scenario: Escape exits placement mode
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And placement mode is active for IronMiner
    When the player presses Escape
    And the app updates once
    Then placement mode is inactive

  # ────────────────────────────────────────────────
  # AC5: Building Removal
  # ────────────────────────────────────────────────

  Scenario: Right-click on building outside placement mode removes it
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And placement mode is inactive
    And CursorGridPos is Some((10, 10))
    And tile (10, 10) has an IronMiner building
    And Inventory count for IronMiner equals 2
    When the player right-clicks
    And the app updates once
    Then RemoveBuildingCommands contains position (10, 10)
    And Inventory count for IronMiner equals 3

  Scenario: Right-click on empty tile outside placement mode does nothing
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And placement mode is inactive
    And CursorGridPos is Some((15, 15))
    And tile (15, 15) has no building
    When the player right-clicks
    And the app updates once
    Then RemoveBuildingCommands is empty

  # ────────────────────────────────────────────────
  # AC6: Transport Path Drawing
  # ────────────────────────────────────────────────

  Scenario: Path drawing produces valid TransportCommands entry for Solid path
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And path-draw mode is active with type Solid
    And group A has a boundary tile at (10, 10)
    And group B has a boundary tile at (14, 10)
    And tiles (11, 10), (12, 10), (13, 10) are empty and passable
    When the player clicks (10, 10), drags through (11, 10), (12, 10), (13, 10), and releases at (14, 10)
    And the app updates once
    Then TransportCommands.draw_path contains an entry with waypoints [(10,10),(11,10),(12,10),(13,10),(14,10)]
    And the entry has is_pipe == false

  Scenario: Path drawing over occupied tile does not commit
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And path-draw mode is active with type Solid
    And tile (12, 10) is occupied by a building
    When the player attempts to draw a path through (12, 10)
    And the app updates once
    Then TransportCommands.draw_path is empty

  Scenario: Path drawing cancelled by Escape discards partial path
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And path-draw mode is active with type Liquid
    And the player has started drawing from (10, 10) through (11, 10)
    When the player presses Escape
    And the app updates once
    Then TransportCommands.draw_path is empty
    And path-draw mode is inactive

  Scenario: Liquid path drawing sets is_pipe flag
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And path-draw mode is active with type Liquid
    And group A has a boundary tile at (10, 10)
    And group B has a boundary tile at (14, 10)
    And tiles (11, 10), (12, 10), (13, 10) are empty and passable
    When the player draws a path from (10, 10) to (14, 10)
    And the app updates once
    Then TransportCommands.draw_path contains an entry with is_pipe == true

  Scenario: Path drawing over existing path tile rejects
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And path-draw mode is active with type Solid
    And PathOccupancy has an entry at tile (12, 10)
    When the player attempts to draw a path through (12, 10)
    And the app updates once
    Then TransportCommands.draw_path is empty

  # ────────────────────────────────────────────────
  # AC7: Game Speed Control
  # ────────────────────────────────────────────────

  Scenario: Space toggles pause on
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And GameSpeed is Running with multiplier 1
    When the player presses Space
    And the app updates once
    Then GameSpeed is Paused

  Scenario: Space toggles pause off and restores previous speed
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And GameSpeed is Paused (was previously multiplier 1)
    When the player presses Space
    And the app updates once
    Then GameSpeed is Running with multiplier 1

  Scenario: Number keys set speed multiplier
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And GameSpeed is Running with multiplier 1
    When the player presses key "2"
    And the app updates once
    Then GameSpeed is Running with multiplier 2
    When the player presses key "3"
    And the app updates once
    Then GameSpeed is Running with multiplier 4

  Scenario: Placement works while paused — commands queue for unpause
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And GameSpeed is Paused
    And Inventory contains IronMiner=4
    And placement mode is active for IronMiner
    And CursorGridPos is Some((10, 10))
    And tile (10, 10) has terrain IronVein, is not occupied, and is revealed
    When the player left-clicks
    And the app updates once
    Then PlacementCommands contains an entry for IronMiner at (10, 10)
    And Inventory count for IronMiner equals 3

  Scenario: Path drawing works while paused
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And GameSpeed is Paused
    And path-draw mode is active with type Solid
    And group A has a boundary tile at (10, 10)
    And group B has a boundary tile at (14, 10)
    And tiles (11, 10), (12, 10), (13, 10) are empty and passable
    When the player draws a path from (10, 10) to (14, 10)
    And the app updates once
    Then TransportCommands.draw_path contains an entry with waypoints [(10,10),(11,10),(12,10),(13,10),(14,10)]

  # ────────────────────────────────────────────────
  # AC8: Build Menu Panel
  # ────────────────────────────────────────────────

  Scenario: Build menu shows correct categories and inventory counts
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And Inventory contains IronMiner=4, CopperMiner=2, StoneQuarry=2, WaterPump=2
    And Inventory contains IronSmelter=2, CopperSmelter=1, Sawmill=1
    And Inventory contains Constructor=1, WindTurbine=3, Watchtower=1
    And TierState.current_tier equals 1
    When the build menu is rendered
    Then category "Extraction" contains [IronMiner, CopperMiner, StoneQuarry, WaterPump]
    And category "Processing" contains [IronSmelter, CopperSmelter, Sawmill]
    And category "Mall" contains [Constructor, Toolmaker, Assembler]
    And category "Combat" contains [ImpCamp]
    And category "Energy" contains [WindTurbine, WaterWheel, LavaGenerator, ManaReactor]
    And category "Utility" contains [Watchtower, Trader, SacrificeAltar]
    And IronMiner entry shows inventory_count 4
    And WindTurbine entry shows inventory_count 3

  Scenario: Tier-locked buildings are grayed and unselectable
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And TierState.current_tier equals 1
    When the build menu is rendered
    Then Toolmaker is displayed grayed with label "T2"
    And Assembler is displayed grayed with label "T3"
    And LavaGenerator is displayed grayed with label "T2"
    And ManaReactor is displayed grayed with label "T3"
    And clicking Toolmaker does not enter placement mode

  Scenario: Clicking a building with count > 0 enters placement mode
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And Inventory contains IronMiner=4
    And TierState.current_tier equals 1
    When the player clicks IronMiner in the build menu
    And the app updates once
    Then placement mode is active for IronMiner

  Scenario: Clicking a building with count 0 does not enter placement mode
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And Inventory contains IronMiner=0
    When the player clicks IronMiner in the build menu
    And the app updates once
    Then placement mode is inactive

  # ────────────────────────────────────────────────
  # AC9: Energy Bar
  # ────────────────────────────────────────────────

  Scenario: Energy bar shows green when surplus >= 10%
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And EnergyPool has generation 100 and consumption 80
    And EnergyPool.ratio equals 1.25
    When the energy bar is rendered
    Then the energy bar displays generation=100, consumption=80, ratio=125%
    And the energy bar color is green

  Scenario: Energy bar shows yellow when surplus is 0-10%
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And EnergyPool has generation 100 and consumption 95
    And EnergyPool.ratio equals 1.053
    When the energy bar is rendered
    Then the energy bar displays ratio=105%
    And the energy bar color is yellow

  Scenario: Energy bar shows red when in deficit
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And EnergyPool has generation 80 and consumption 100
    And EnergyPool.ratio equals 0.8
    When the energy bar is rendered
    Then the energy bar displays ratio=80%
    And the energy bar color is red

  Scenario: Energy bar shows green when generation and consumption are both 0
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And EnergyPool has generation 0 and consumption 0
    And EnergyPool.ratio equals 1.0
    When the energy bar is rendered
    Then the energy bar color is green

  # ────────────────────────────────────────────────
  # AC10: Opus Tree Panel
  # ────────────────────────────────────────────────

  Scenario: Opus tree displays milestone nodes with correct rates and visual states
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And OpusTreeResource.main_path has 7 nodes
    And node 0 is IronBar with required_rate 2.0, current_rate 2.5, sustained true
    And node 1 is CopperBar with required_rate 1.5, current_rate 1.5, sustained false, sustain_progress 0.4
    And node 2 is Plank with required_rate 1.0, current_rate 0.0, sustained false
    When the opus tree panel is rendered
    Then node 0 (IronBar) shows completed_glow visual state
    And node 1 (CopperBar) shows in_progress_highlighted visual state with progress bar at 40%
    And node 2 (Plank) shows locked visual state

  Scenario: Opus tree progress bars update after production tick
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And OpusTreeResource node 1 (CopperBar) has current_rate 0.5 and required_rate 1.5
    When ProductionRates for CopperBar updates to 1.5
    And the app updates once
    Then OpusTreeResource node 1 current_rate equals 1.5

  # ────────────────────────────────────────────────
  # AC11: Minimap
  # ────────────────────────────────────────────────

  Scenario: Minimap renders terrain, buildings, fog, and camera viewport
    Given a fresh App with MinimalPlugins + SimulationPlugin + RenderPlugin + UiPlugin
    And Grid dimensions are 64x64
    And FogMap has 313 revealed cells
    And there are buildings at (10, 10) and (15, 15)
    When the minimap is rendered
    Then the minimap shows terrain colors for all 64x64 cells
    And the minimap shows building dots at (10, 10) and (15, 15)
    And the minimap shows fog overlay for unrevealed cells
    And the minimap shows the camera viewport rectangle

  Scenario: Clicking minimap pans camera to clicked grid position
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And the camera is centered at grid position (32, 32)
    And the minimap is 128x128 pixels representing a 64x64 grid
    When the player clicks the minimap at pixel (32, 32) corresponding to grid (16, 16)
    And the app updates once
    Then the camera target position is grid (16, 16)

  # ────────────────────────────────────────────────
  # AC12: Tooltips
  # ────────────────────────────────────────────────

  Scenario: Building tooltip appears after 300ms hover delay
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And an IronMiner building exists at (10, 10) in group "Alpha" running recipe IronOre with energy consumption 5
    And CursorGridPos is Some((10, 10))
    When the cursor hovers over (10, 10) for 300ms
    Then a tooltip is visible showing type=IronMiner, group="Alpha", recipe=IronOre, energy=5

  Scenario: Tooltip does not appear before 300ms
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And an IronMiner building exists at (10, 10)
    And CursorGridPos is Some((10, 10))
    When the cursor hovers over (10, 10) for 200ms
    Then no tooltip is visible

  Scenario: Tooltip disappears when cursor moves away
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And a tooltip is currently visible for the building at (10, 10)
    When the cursor moves to grid position (20, 20) which has no building
    And the app updates once
    Then no tooltip is visible

  Scenario: Transport path tooltip shows cargo and throughput
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And a Solid transport path exists at (12, 10) carrying IronOre at throughput 3.0 tier 1
    And CursorGridPos is Some((12, 10))
    When the cursor hovers over (12, 10) for 300ms
    Then a tooltip is visible showing cargo=IronOre, throughput=3.0, tier=1

  Scenario: Creature tooltip shows type, behavior, and health
    Given a fresh App with MinimalPlugins + SimulationPlugin + CreaturesPlugin + InputPlugin + UiPlugin
    And a creature of type Golem with behavior Patrol and health 50 exists at (30, 30)
    And CursorGridPos is Some((30, 30))
    When the cursor hovers over (30, 30) for 300ms
    Then a tooltip is visible showing type=Golem, behavior=Patrol, health=50

  # ────────────────────────────────────────────────
  # AC13: Notifications
  # ────────────────────────────────────────────────

  Scenario: MilestoneReached event triggers notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When a MilestoneReached event fires for resource IronBar
    And the app updates once
    Then a notification "Milestone reached: IronBar" is visible

  Scenario: TierUnlocked event triggers notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When a TierUnlocked event fires for tier 2
    And the app updates once
    Then a notification "Tier 2 unlocked!" is visible

  Scenario: EnergyDeficit event triggers notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When an EnergyDeficit event fires with 3 groups throttled
    And the app updates once
    Then a notification "Energy deficit — 3 groups throttled" is visible

  Scenario: HazardWarning event triggers notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When a HazardWarning event fires for lava eruption in 30 ticks
    And the app updates once
    Then a notification "Hazard warning: lava eruption in 30 ticks" is visible

  Scenario: InventoryEmpty event triggers notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When an InventoryEmpty event fires for WindTurbine
    And the app updates once
    Then a notification "Out of WindTurbine" is visible

  Scenario: Max 5 notifications visible — 6th causes oldest to hide
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And 5 notifications are already visible:
      | message                                    |
      | Milestone reached: IronBar                 |
      | Milestone reached: CopperBar               |
      | Energy deficit — 2 groups throttled         |
      | Hazard warning: lava eruption in 30 ticks   |
      | Tier 2 unlocked!                           |
    When a 6th notification "Nest cleared — rare loot" fires
    And the app updates once
    Then 5 notifications are visible
    And the oldest notification "Milestone reached: IronBar" is hidden

  Scenario: Notification auto-dismisses after 5 seconds
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    And a notification "Milestone reached: IronBar" appeared at time T
    When 5.0 seconds of real time elapse (game running, not paused)
    Then the notification "Milestone reached: IronBar" is no longer visible

  # ────────────────────────────────────────────────
  # Edge Cases
  # ────────────────────────────────────────────────

  Scenario: Window resize updates camera aspect ratio — minimap stays fixed
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And the window size is 1280x720
    When the window is resized to 1920x1080
    And the app updates once
    Then the camera orthographic projection aspect ratio matches 1920/1080
    And the minimap remains fixed at 128x128 pixels in the bottom-right corner

  Scenario: UI panel click priority — panel consumes click, no grid action
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And placement mode is active for IronMiner
    And Inventory contains IronMiner=4
    And the cursor is at screen position (30, 300) over the build menu panel
    When the player left-clicks
    And the app updates once
    Then PlacementCommands is empty
    And Inventory count for IronMiner equals 4

  Scenario: Inventory reaches 0 during placement — auto-exit with notification
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Inventory contains WindTurbine=1
    And placement mode is active for WindTurbine
    And CursorGridPos is Some((20, 20))
    And tile (20, 20) is valid for WindTurbine placement
    When the player left-clicks to place the last WindTurbine
    And the app updates once
    Then Inventory count for WindTurbine equals 0
    And placement mode is inactive
    And a notification "Out of WindTurbine" is visible

  Scenario: Zoom at grid edge — camera clamp prevents seeing beyond grid
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin
    And Grid dimensions are 64x64
    And CameraConfig with bounds_margin 2
    And the camera is at position (0, 0) near the top-left corner
    When the scroll wheel zooms out to scale 0.5 (widest view)
    And the app updates once
    Then the camera position is clamped so the viewport does not extend beyond margin (-2, -2) to (66, 66)

  Scenario: Two simultaneous notifications of same type — both shown, no dedup
    Given a fresh App with MinimalPlugins + SimulationPlugin + UiPlugin
    When two MilestoneReached events fire for IronBar in the same tick
    And the app updates once
    Then 2 notifications with text "Milestone reached: IronBar" are visible
    And they are stacked vertically

  Scenario: Pause during hazard countdown freezes notification dismiss timer
    Given a fresh App with MinimalPlugins + SimulationPlugin + InputPlugin + UiPlugin
    And a HazardWarning notification is visible: "Hazard warning: lava eruption in 30 ticks"
    And the notification has been visible for 3 seconds
    When GameSpeed is set to Paused
    And 10 seconds of real time elapse while paused
    Then the notification "Hazard warning: lava eruption in 30 ticks" is still visible
    And the notification dismiss timer has not advanced past 3 seconds
