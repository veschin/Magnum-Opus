@feature:game-render
Feature: Visual Rendering — sync ECS state to 3D isometric scene with pixel-art post-processing

  RenderPlugin is a read-only layer over ECS state. It syncs terrain, buildings,
  transport, creatures, fog, and overlays to Bevy scene entities each frame.
  Post-processing applies outline, toon shading, posterization, and nearest-neighbor
  upscale. No render system ever mutates simulation state.

  # ────────────────────────────────────────────────
  # AC1: Grid Rendering — terrain tiles to scene entities
  # ────────────────────────────────────────────────

  Scenario: Every terrain tile gets a corresponding scene entity with correct position
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin + RenderPlugin
    When the app updates once
    Then every tile in Grid.terrain (64x64 = 4096 tiles) has a corresponding scene entity
    And each scene entity position matches its grid tile using isometric transform: screen_x = (gx - gy) * tile_half_width, screen_y = (gx + gy) * tile_half_height

  Scenario: Terrain types have correct color and height offset
    Given a 4x4 test grid with tiles: Grass at (0,0), IronVein at (1,0), WaterSource at (0,1), StoneDeposit at (1,2), LavaSource at (3,2)
    And RenderPlugin is active
    When the app updates once
    Then the scene entity at (0,0) has base color [0.35, 0.55, 0.25] and height offset 0.0
    And the scene entity at (1,0) has base color [0.45, 0.35, 0.30] and height offset 0.15
    And the scene entity at (0,1) has base color [0.20, 0.35, 0.55] and height offset -0.10
    And the scene entity at (1,2) has base color [0.50, 0.50, 0.48] and height offset 0.10
    And the scene entity at (3,2) has base color [0.80, 0.25, 0.05] and height offset -0.05
    And the scene entity at (3,2) has emissive == true
    And the scene entity at (0,0) has emissive == false

  # ────────────────────────────────────────────────
  # AC2: Building Sync — spawn and despawn scene entities
  # ────────────────────────────────────────────────

  Scenario: Placed building gets a scene entity at correct grid position
    Given a fresh App with SimulationPlugin + RenderPlugin and a 64x64 grid
    And a PlacementCommand for IronMiner at (10, 10)
    When the app updates once
    Then a scene entity exists for the IronMiner at grid position (10, 10)

  Scenario: Removed building despawns its scene entity
    Given a fresh App with SimulationPlugin + RenderPlugin and a 64x64 grid
    And an IronMiner building exists at (5, 5) with a corresponding scene entity
    When a RemoveBuildingCommand is issued for (5, 5)
    And the app updates once
    Then no scene entity exists at grid position (5, 5)

  Scenario: Multiple buildings each get independent scene entities
    Given a fresh App with SimulationPlugin + RenderPlugin
    And buildings placed: IronMiner at (10,10), IronMiner at (11,10), IronSmelter at (10,11), WindTurbine at (20,20)
    When the app updates once
    Then exactly 4 building scene entities exist
    And each has a unique position matching its grid tile

  # ────────────────────────────────────────────────
  # AC3: Building Visual State — material reflects ECS state
  # ────────────────────────────────────────────────

  Scenario: Building material and animation reflect production state
    Given buildings with each production state:
      | position | state     |
      | (10,10)  | Producing |
      | (10,11)  | Idle      |
      | (12,12)  | NoEnergy  |
      | (13,13)  | Paused    |
    And RenderPlugin is active
    When the app updates once
    Then the scene entity at (10,10) has material == "active" and shader_animation == true
    And the scene entity at (10,11) has material == "default" and shader_animation == false
    And the scene entity at (12,12) has material == "dimmed" and shader_animation == false
    And the scene entity at (13,13) has material == "yellow_tint" and shader_animation == false

  # ────────────────────────────────────────────────
  # AC4: Group Outlines — colored outlines enclosing groups
  # ────────────────────────────────────────────────

  Scenario: Group outline encloses all member buildings with correct color
    Given buildings IronMiner at (10,10), IronMiner at (11,10), IronSmelter at (10,11) in group 1 with state Active
    And WindTurbine at (20,20) in group 2 with state Active
    And RenderPlugin is active
    When the app updates once
    Then exactly 2 group outline entities exist
    And group 1 outline encloses positions (10,10), (11,10), (10,11)
    And group 1 outline color is [0.2, 0.8, 0.2, 0.6]
    And group 2 outline encloses position (20,20)
    And group 2 outline color is [0.2, 0.8, 0.2, 0.6]

  Scenario: Group outline color encodes group state
    Given groups with states: Active, Paused, NoEnergy, Idle
    And RenderPlugin is active
    When the app updates once
    Then Active group outline color is [0.2, 0.8, 0.2, 0.6]
    And Paused group outline color is [0.8, 0.8, 0.2, 0.6]
    And NoEnergy group outline color is [0.8, 0.2, 0.2, 0.6]
    And Idle group outline color is [0.5, 0.5, 0.5, 0.4]

  Scenario: Group split produces two separate outlines
    Given buildings A at (10,10) and B at (11,10) in group 1
    And RenderPlugin is active
    When building at (10,10) is removed causing group 1 to split
    And the app updates once
    Then the single group-1 outline is replaced by one outline around B at (11,10)

  # ────────────────────────────────────────────────
  # AC5: Transport Visualization — paths and cargo
  # ────────────────────────────────────────────────

  Scenario: Rune path renders as continuous line of sprites between groups
    Given a RunePath tier 1 with segments [(10,10), (11,10), (12,10), (13,10), (14,10)]
    And RenderPlugin is active
    When the app updates once
    Then exactly 5 path sprite entities exist along the path
    And each path sprite has shimmer UV-scroll speed 0.5
    And path sprites have color [0.6, 0.5, 0.3]

  Scenario: Cargo sprites position and appearance match resource type
    Given a RunePath with cargo: IronOre at progress 0.3, CopperOre at progress 0.7
    And RenderPlugin is active
    When the app updates once
    Then exactly 2 cargo scene entities exist
    And cargo at progress 0.3 is positioned at 30% along the path spline with color [0.45, 0.35, 0.30]
    And cargo at progress 0.7 is positioned at 70% along the path spline with color [0.60, 0.40, 0.20]

  Scenario: Cargo bounce uses entity-id-based phase offset for desync
    Given two cargo entities with different entity IDs on the same path
    And RenderPlugin is active
    When the app updates once
    Then each cargo entity has bounce amplitude 0.08 and frequency 3.0
    And the two cargo entities have different phase offsets derived from their entity IDs

  Scenario: All resource types have distinct cargo visuals
    Given cargo entities for IronOre, CopperOre, IronBar, and Water on paths
    And RenderPlugin is active
    When the app updates once
    Then IronOre cargo has color [0.45, 0.35, 0.30]
    And CopperOre cargo has color [0.60, 0.40, 0.20]
    And IronBar cargo has color [0.55, 0.55, 0.55]
    And Water cargo has color [0.20, 0.40, 0.70]

  # ────────────────────────────────────────────────
  # AC6: Fog of War — overlay based on FogMap
  # ────────────────────────────────────────────────

  Scenario: Unrevealed tiles have opaque dark overlay
    Given a 5x5 grid with 6 revealed tiles at center and 19 unrevealed tiles
    And RenderPlugin is active
    When the app updates once
    Then unrevealed tiles have overlay color [0.05, 0.05, 0.08, 1.0]
    And unrevealed tiles completely occlude terrain beneath

  Scenario: Revealed tiles outside watchtower range are desaturated
    Given a 5x5 grid with watchtower at (2,2) radius 1 and tile (3,3) revealed but outside range
    And RenderPlugin is active
    When the app updates once
    Then tile (3,3) has desaturation_factor 0.7
    And tiles (2,2), (1,2), (2,1), (3,2), (2,3) within watchtower range render at full brightness with no desaturation

  # ────────────────────────────────────────────────
  # AC7: Ghost Preview — placement mode cursor feedback
  # ────────────────────────────────────────────────

  Scenario: Ghost preview shows green tint for valid placement
    Given placement mode is active for building Constructor
    And CursorGridPos is (15, 15) on unoccupied Grass terrain
    And RenderPlugin is active
    When the app updates once
    Then a ghost scene entity exists at grid position (15, 15)
    And the ghost entity has tint [0.2, 0.8, 0.2, 0.5]

  Scenario: Ghost preview shows red tint for invalid placement
    Given placement mode is active for building Constructor
    And CursorGridPos is (10, 10) which is occupied by an existing building
    And RenderPlugin is active
    When the app updates once
    Then a ghost scene entity exists at grid position (10, 10)
    And the ghost entity has tint [0.8, 0.2, 0.2, 0.5]

  Scenario: Ghost preview disappears when placement mode exits
    Given placement mode was active and a ghost entity existed
    When placement mode is deactivated
    And the app updates once
    Then no ghost scene entity exists

  # ────────────────────────────────────────────────
  # AC8: Post-Processing Chain — outline, toon, posterize, upscale
  # ────────────────────────────────────────────────

  Scenario: Post-processing pipeline has 4 passes in correct order with correct parameters
    Given RenderPlugin is active
    When the post-processing chain is queried
    Then exactly 4 passes exist in order: outline, toon_shading, posterization, upscale
    And the outline pass uses sobel_edge_detection on depth_buffer and normal_buffer with threshold 0.3 and kernel_size 3
    And the toon_shading pass uses luminance_quantization with 3 bands
    And the posterization pass uses color_depth_reduction with 8 levels per channel
    And the upscale pass uses nearest_neighbor_blit from low_res_target to window_framebuffer

  Scenario: Low-res render target dimensions and upscale factor
    Given RenderPlugin is active with window resolution 1920x1080
    When the render target is queried
    Then low_res render target dimensions are 480x270
    And upscale factor is 4
    And upscale filter is nearest_neighbor

  # ────────────────────────────────────────────────
  # AC9: Lighting — directional, point lights, ambient
  # ────────────────────────────────────────────────

  Scenario: Emissive buildings spawn point lights with per-type parameters
    Given an IronSmelter at (10,10), a LavaGenerator at (20,20), a ManaReactor at (5,5), and a SacrificeAltar at (15,15)
    And RenderPlugin is active
    When the app updates once
    Then a point light exists at (10,10) with color [1.0, 0.6, 0.2], radius 3.0, intensity 0.8
    And a point light exists at (20,20) with color [1.0, 0.3, 0.05], radius 4.0, intensity 1.2
    And a point light exists at (5,5) with color [0.3, 0.2, 0.9], radius 4.0, intensity 1.0
    And a point light exists at (15,15) with color [0.6, 0.1, 0.8], radius 3.5, intensity 0.9

  Scenario: Directional light and ambient provide base illumination
    Given RenderPlugin is active with Clear weather
    When the app updates once
    Then directional light has direction [-0.5, -0.7, 0.5] and color [1.0, 0.95, 0.85] and intensity 1.0
    And ambient light has color [0.15, 0.12, 0.18] and strength 0.3

  # ────────────────────────────────────────────────
  # AC10: Shader Animations — time-based, entity-desynced
  # ────────────────────────────────────────────────

  Scenario: Buildings idle-bob with per-entity phase offset
    Given two IronMiner buildings at (10,10) and (12,12)
    And RenderPlugin is active
    When the app updates once
    Then both buildings have idle_bob with amplitude 0.02 and frequency 1.5
    And their phase offsets differ (derived from entity ID)

  Scenario: Organic buildings sway with wind animation
    Given a TreeFarm at (8, 8) and a Sawmill at (9, 9)
    And RenderPlugin is active
    When the app updates once
    Then both buildings have wind_sway animation with amplitude 0.03 and frequency 0.8

  Scenario: Liquid and emissive buildings have type-specific shader animations
    Given a Pipe transport entity, a WaterPump building, a LavaGenerator at (10,10), and a ManaReactor at (20,20)
    And RenderPlugin is active
    When the app updates once
    Then Pipe and WaterPump have liquid_flow animation with uv_scroll_speed 0.4
    And LavaGenerator and ManaReactor have emission_pulse with base_intensity 0.6, pulse_amplitude 0.3, pulse_frequency 2.0

  # ────────────────────────────────────────────────
  # AC11: Read-Only Guarantee — render never mutates simulation
  # ────────────────────────────────────────────────

  Scenario: Render plugin removal does not affect simulation behavior
    Given App A with SimulationPlugin only and App B with SimulationPlugin + RenderPlugin
    And both apps have identical initial state with buildings, transport, and creatures
    When both apps update for 100 ticks
    Then all simulation components (Building, Position, GroupMember, Path, Creature, Inventory, EnergyPool) are identical between App A and App B

  Scenario: No render system writes to simulation components resources or events
    Given RenderPlugin is active
    When the system access metadata for all systems in RenderPlugin is inspected
    Then no system has write access to Building, Position, GroupMember, ProductionState, Path, CargoContainer, Creature, Nest, FogMap, Grid, Inventory, EnergyPool, or any simulation event type

  # ────────────────────────────────────────────────
  # Edge Cases
  # ────────────────────────────────────────────────

  Scenario: Buildings at grid boundaries render at correct screen positions
    Given buildings at grid positions (0,0), (63,63), (0,63), and (63,0) on a 64x64 grid
    And RenderPlugin is active
    When the app updates once
    Then scene entities exist at all four boundary positions with valid screen coordinates
    And no scene entity is clipped or positioned off-screen

  Scenario: Transport path with zero cargo renders without crash
    Given a Pipe tier 1 with segments [(5,5), (6,5), (7,5)] and zero cargo entities
    And RenderPlugin is active
    When the app updates once
    Then exactly 3 path sprite entities exist
    And exactly 0 cargo sprite entities exist
    And no panic occurred

  Scenario: Creature despawn removes scene entity in same frame
    Given a Creature entity at (30, 30) with a corresponding scene entity
    And RenderPlugin is active
    When the Creature entity is despawned
    And the app updates once
    Then no scene entity exists for the despawned creature
    And no orphaned sprite entities remain at (30, 30)

  Scenario: FogMap with all tiles revealed produces no visible fog effect
    Given a 64x64 grid with all 4096 tiles revealed in FogMap
    And RenderPlugin is active
    When the app updates once
    Then zero fog overlay entities exist
    And all tiles render at full brightness with no desaturation

  Scenario: Weather change lerps directional light over 10 frames without pop
    Given RenderPlugin is active with CurrentWeather == Clear
    And directional light color is [1.0, 0.95, 0.85] and intensity 1.0
    When CurrentWeather changes to Rain
    And the app updates 5 times
    Then directional light color is interpolating between Clear [1.0, 0.95, 0.85] and Rain-modified [0.70, 0.7125, 0.7225]
    And directional light intensity is interpolating between 1.0 and 0.6
    When the app updates 5 more times (10 total since weather change)
    Then directional light color equals [0.70, 0.7125, 0.7225] fully transitioned
    And directional light intensity equals 0.6

  Scenario: Building with missing sprite asset renders magenta placeholder
    Given a building entity with an unrecognized or missing sprite asset
    And RenderPlugin is active
    When the app updates once
    Then the building scene entity renders as a magenta placeholder quad
    And no panic occurred

  Scenario: Non-emissive buildings do not spawn point lights
    Given a Watchtower at (10,10) and a WindTurbine at (20,20)
    And RenderPlugin is active
    When the app updates once
    Then no point light entities exist for Watchtower or WindTurbine
    And the scene is still lit by directional light and ambient

  Scenario: No tiles revealed produces fully opaque fog over entire grid
    Given a 5x5 grid with FogMap.revealed == empty
    And RenderPlugin is active
    When the app updates once
    Then all 25 tiles have opaque dark overlay [0.05, 0.05, 0.08, 1.0]
