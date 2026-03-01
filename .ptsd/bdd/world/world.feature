@feature:world
Feature: World & Biomes
  The procedurally generated environment: biomes, terrain, hazards, weather,
  and the systemic interactions between them. Each run generates a biome-specific
  map with interconnected environmental systems.

  # ============================================================================
  # AC1: Map generation produces biome-specific terrain with resource veins,
  #      liquid sources, and hazard zones
  # ============================================================================

  # --- Happy path: forest biome ---

  Scenario: Forest biome generates expected terrain distribution
    Given a new run with biome "forest" and map size 64x64
    When map generation completes
    Then at least 40% of tiles are "grass"
    Then at least 25% of tiles are "dense_forest"
    Then at least 10% of tiles are "water_source"
    Then at least 7% of tiles are "iron_vein"
    Then at least 5% of tiles are "copper_vein"
    Then at least 3% of tiles are "mana_node"

  Scenario: Forest biome generates resource veins within configured bounds
    Given a new run with biome "forest" and map size 64x64
    When map generation completes
    Then at least 12 resource veins exist on the map
    Then at most 20 resource veins exist on the map
    Then iron veins have avg_remaining near 500
    Then copper veins have avg_remaining near 400
    Then stone deposits have avg_remaining near 600

  Scenario: Forest biome water sources are infinite
    Given a new run with biome "forest" and map size 64x64
    When map generation completes
    Then water_source tiles have remaining set to infinite

  Scenario: Forest biome generates only wildfire and storm hazard zones
    Given a new run with biome "forest" and map size 64x64
    When map generation completes
    Then at least one hazard zone of type "wildfire" exists
    Then at least one hazard zone of type "storm" exists
    Then no hazard zone of type "eruption" exists
    Then no hazard zone of type "sandstorm" exists

  # --- Happy path: volcanic biome ---

  Scenario: Volcanic biome generates lava sources and obsidian veins
    Given a new run with biome "volcanic" and map size 64x64
    When map generation completes
    Then at least 15% of tiles are "lava_source"
    Then at least 12% of tiles are "obsidian_vein"
    Then at least 10% of tiles are "iron_vein"
    Then at least 30% of tiles are "scorched_rock"

  Scenario: Volcanic biome generates eruption and ash_storm hazards
    Given a new run with biome "volcanic" and map size 64x64
    When map generation completes
    Then at least one hazard zone of type "eruption" exists
    Then at least one hazard zone of type "ash_storm" exists
    Then no hazard zone of type "wildfire" exists

  # --- Happy path: desert biome ---

  Scenario: Desert biome generates mana nodes and copper at high density
    Given a new run with biome "desert" and map size 64x64
    When map generation completes
    Then at least 40% of tiles are "sand"
    Then at least 7% of tiles are "mana_node"
    Then at least 8% of tiles are "copper_vein"

  Scenario: Desert biome has no natural water sources
    Given a new run with biome "desert" and map size 64x64
    When map generation completes
    Then no tiles of type "water_source" exist

  Scenario: Desert biome generates sandstorm and heat_wave hazards
    Given a new run with biome "desert" and map size 64x64
    When map generation completes
    Then at least one hazard zone of type "sandstorm" exists
    Then at least one hazard zone of type "heat_wave" exists
    Then no hazard zone of type "wildfire" exists

  # --- Happy path: ocean biome ---

  Scenario: Ocean biome generates shallow water and coral reef tiles
    Given a new run with biome "ocean" and map size 64x64
    When map generation completes
    Then at least 30% of tiles are "shallow_water"
    Then at least 10% of tiles are "coral_reef"
    Then at least 8% of tiles are "water_source"

  Scenario: Ocean biome generates tsunami and storm hazards
    Given a new run with biome "ocean" and map size 64x64
    When map generation completes
    Then at least one hazard zone of type "tsunami" exists
    Then at least one hazard zone of type "storm" exists
    Then no hazard zone of type "eruption" exists

  # --- Determinism ---

  Scenario: Same seed produces identical map layout
    Given a new run with biome "forest" and map size 64x64 and seed 42
    When map generation completes
    Then I record the terrain hash as snapshot_A
    Given a new run with biome "forest" and map size 64x64 and seed 42
    When map generation completes
    Then the terrain hash matches snapshot_A

  # --- Starting area ---

  Scenario: Starting area is revealed and has resources nearby
    Given a new run with biome "forest" and map size 64x64
    When map generation completes
    Then all tiles within radius 8 of the map center are Visible
    Then at least one resource vein exists within radius 8 of the map center
    Then no hazard zone center exists within radius 8 of the map center

  # ============================================================================
  # AC2: Buildings with landscape requirements can only be placed on matching
  #      terrain tiles
  # ============================================================================

  # --- Happy path ---

  Scenario: Iron miner can be placed on iron_vein tile
    Given a forest biome map with an iron_vein tile at position [8, 10]
    When the player issues PlaceBuilding command for "iron_miner" at [8, 10]
    Then the building is placed successfully at [8, 10]
    Then a BuildingPlaced event is emitted

  Scenario: Lava siphon can be placed on lava_source tile
    Given a volcanic biome map with a lava_source tile at position [4, 4]
    When the player issues PlaceBuilding command for "lava_siphon" at [4, 4]
    Then the building is placed successfully at [4, 4]

  Scenario: Wind turbine can be placed on any buildable tile
    Given a forest biome map with a grass tile at position [10, 12]
    When the player issues PlaceBuilding command for "wind_turbine" at [10, 12]
    Then the building is placed successfully at [10, 12]

  # --- Error path ---

  Scenario: Iron miner cannot be placed on grass tile
    Given a forest biome map with a grass tile at position [10, 12]
    When the player issues PlaceBuilding command for "iron_miner" at [10, 12]
    Then the placement is rejected with reason "terrain_mismatch"
    Then no BuildingPlaced event is emitted

  Scenario: Lava siphon cannot be placed on scorched_rock tile
    Given a volcanic biome map with a scorched_rock tile at position [3, 3]
    When the player issues PlaceBuilding command for "lava_siphon" at [3, 3]
    Then the placement is rejected with reason "terrain_mismatch"

  Scenario: Building cannot be placed on impassable tile
    Given a forest biome map with an impassable tile at position [5, 5]
    When the player issues PlaceBuilding command for "wind_turbine" at [5, 5]
    Then the placement is rejected with reason "tile_not_buildable"

  # ============================================================================
  # AC3: Hazard events announce zone and timing N seconds in advance
  # ============================================================================

  # --- Happy path ---

  Scenario: Eruption hazard announces 200 ticks before event
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the eruption next_event_tick is 300
    When simulation reaches tick 100
    Then a hazard warning is active for "eruption" at center [6, 6] with 200 ticks remaining

  Scenario: Wildfire hazard announces 150 ticks before event
    Given a forest biome map with a wildfire hazard zone centered at [5, 5] with radius 5
    Given the wildfire next_event_tick is 250
    When simulation reaches tick 100
    Then a hazard warning is active for "wildfire" at center [5, 5] with 150 ticks remaining

  Scenario: Ash storm announces 300 ticks before event
    Given a volcanic biome map with an ash_storm hazard zone centered at [8, 8] with radius 6
    Given the ash_storm next_event_tick is 600
    When simulation reaches tick 300
    Then a hazard warning is active for "ash_storm" at center [8, 8] with 300 ticks remaining

  # --- Edge: no warning when event is far away ---

  Scenario: No warning when hazard event is more than warning_ticks away
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the eruption next_event_tick is 500 and warning_ticks is 200
    When simulation reaches tick 100
    Then no hazard warning is active for "eruption"

  # ============================================================================
  # AC4: Sacrifice building placed in hazard zone shows probability of bonus
  #      vs miss
  # ============================================================================

  # --- Happy path ---

  Scenario: Sacrifice altar in eruption zone shows base success chance
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 1.0
    Given the current tier is T1
    When a sacrifice_altar is placed at [6, 6]
    Then the sacrifice building shows success chance of 60%
    # Formula: base 65% + altar_bonus 10% - (intensity 1.0 * penalty 15%) - tier_penalty 0% = 60%

  Scenario: Sacrifice altar in T2 eruption zone shows reduced chance
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 1.0
    Given the current tier is T2
    When a sacrifice_altar is placed at [6, 6]
    Then the sacrifice building shows success chance of 55%
    # Formula: 65% + 10% - 15% - 5% (T2 penalty) = 55%

  Scenario: Sacrifice altar in T3 eruption zone shows further reduced chance
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 1.0
    Given the current tier is T3
    When a sacrifice_altar is placed at [6, 6]
    Then the sacrifice building shows success chance of 50%
    # Formula: 65% + 10% - 15% - 10% (T3 penalty) = 50%

  # --- Edge: chance clamping ---

  Scenario: Sacrifice chance is clamped to minimum 10%
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 5.0
    Given the current tier is T3
    When a sacrifice_altar is placed at [6, 6]
    Then the sacrifice building shows success chance of 10%
    # Formula: 65% + 10% - 75% - 10% = -10%, clamped to 10%

  Scenario: Sacrifice chance is clamped to maximum 90%
    Given a forest biome map with a storm hazard zone centered at [5, 5] with radius 8 and intensity 0.0
    Given the current tier is T1
    When a sacrifice_altar is placed at [5, 5]
    Then the sacrifice building shows success chance of 75%
    # Formula: 65% + 10% - 0% - 0% = 75% (within bounds)

  # ============================================================================
  # AC4 continued: Sacrifice building placed OUTSIDE hazard zone
  # (PRD edge case: no effect, building sits idle)
  # ============================================================================

  Scenario: Sacrifice altar outside any hazard zone has no effect
    Given a volcanic biome map with an eruption hazard zone centered at [12, 12] with radius 3
    When a sacrifice_altar is placed at [3, 3]
    Then the sacrifice building inHazardZone is false
    Then the sacrifice building has no success chance displayed

  # ============================================================================
  # AC5: Hazard destroying a tile applies the enhancement property to that tile
  # ============================================================================

  # --- Happy path ---

  Scenario: Eruption enhances tiles with enriched bonus after triggering
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given an iron_miner building at [5, 5] within the eruption zone
    Given the eruption next_event_tick is 100
    When simulation reaches tick 100
    Then the iron_miner at [5, 5] is destroyed
    Then tile [5, 5] has TileEnhancement of type "enriched" with magnitude 1.5
    Then tile [5, 5] enhancement duration is 6000 ticks

  Scenario: Wildfire enhances tiles with charred_fertile bonus
    Given a forest biome map with a wildfire hazard zone centered at [5, 5] with radius 5
    Given the wildfire next_event_tick is 100
    When simulation reaches tick 100
    Then tiles within radius 5 of [5, 5] have TileEnhancement of type "charred_fertile" with magnitude 1.3

  Scenario: Sandstorm enhances tiles with uncovered_deposit bonus
    Given a desert biome map with a sandstorm hazard zone centered at [5, 5] with radius 7
    Given the sandstorm next_event_tick is 100
    When simulation reaches tick 100
    Then tiles within radius 7 of [5, 5] have TileEnhancement of type "uncovered_deposit" with magnitude 1.4

  Scenario: Tsunami enhances tiles with tidal_deposit bonus
    Given an ocean biome map with a tsunami hazard zone centered at [5, 5] with radius 6
    Given the tsunami next_event_tick is 100
    When simulation reaches tick 100
    Then tiles within radius 6 of [5, 5] have TileEnhancement of type "tidal_deposit" with magnitude 1.6

  # --- Hazard destruction behavior ---

  Scenario: Eruption destroys buildings and paths in zone
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given an iron_miner building at [5, 5] within the eruption zone
    Given a rune_path passing through [6, 5] within the eruption zone
    Given the eruption next_event_tick is 100
    When simulation reaches tick 100
    Then the iron_miner at [5, 5] is destroyed
    Then the rune_path segment at [6, 5] is destroyed
    Then BuildingDestroyed events are emitted for destroyed entities

  Scenario: Ash storm does not destroy buildings but applies production penalty
    Given a volcanic biome map with an ash_storm hazard zone centered at [8, 8] with radius 6
    Given an iron_miner building at [8, 8] within the ash_storm zone
    Given the ash_storm next_event_tick is 100
    When simulation reaches tick 100
    Then the iron_miner at [8, 8] is NOT destroyed
    Then the iron_miner at [8, 8] has production speed reduced by 50%

  Scenario: Storm destroys paths but not buildings
    Given a forest biome map with a storm hazard zone centered at [5, 5] with radius 8
    Given a wind_turbine building at [5, 5] within the storm zone
    Given a rune_path passing through [6, 5] within the storm zone
    Given the storm next_event_tick is 100
    When simulation reaches tick 100
    Then the wind_turbine at [5, 5] is NOT destroyed
    Then the rune_path segment at [6, 5] is destroyed

  # --- Sacrifice success/miss during hazard ---

  Scenario: Sacrifice building hit on success yields double enhancement
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 1.0
    Given a sacrifice_altar at [6, 6] with success chance 60%
    Given the eruption next_event_tick is 100
    Given the RNG roll is 0.3 (below success chance)
    When simulation reaches tick 100
    Then a SacrificeHit event is emitted for the sacrifice_altar
    Then tile [6, 6] has TileEnhancement magnitude of 3.0
    # success_reward_multiplier 2.0 * base magnitude 1.5 = 3.0

  Scenario: Sacrifice building miss results in building destroyed
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4 and intensity 1.0
    Given a sacrifice_altar at [6, 6] with success chance 60%
    Given the eruption next_event_tick is 100
    Given the RNG roll is 0.8 (above success chance)
    When simulation reaches tick 100
    Then a SacrificeMiss event is emitted
    Then the sacrifice_altar at [6, 6] is destroyed

  # --- PRD edge case: hazard hits tile with no buildings ---

  Scenario: Hazard on empty area still applies tile enhancement
    Given a volcanic biome map with an eruption hazard zone centered at [5, 5] with radius 3
    Given no buildings exist in the eruption zone
    Given the eruption next_event_tick is 50
    When simulation reaches tick 50
    Then tiles within radius 3 of [5, 5] have TileEnhancement of type "enriched" with magnitude 1.5
    Then no BuildingDestroyed events are emitted

  # --- Hazard recurrence ---

  Scenario: Eruption hazard recurs at configured interval
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the eruption next_event_tick is 100 and interval_ticks is 2400
    When simulation reaches tick 100
    Then the eruption zone next_event_tick is updated to approximately 2500 (100 + 2400 +/- 600)

  # ============================================================================
  # AC6: At least 3 systemic element interactions are functional
  #      (fire+wind, rain+soil, cold+water)
  # ============================================================================

  # --- Interaction 1: fire + wind = spread ---

  Scenario: Fire spreads to cardinal neighbors when wind is present
    Given an 8x8 grid with dense_forest tiles
    Given tile [4, 4] has ElementalState fire=0.5 water=0.0 cold=0.0 wind=0.0
    Given weather is "wind" with element_effect wind=0.6
    When ElementInteractionSystem runs for 1 tick
    Then tile [4, 4] fire is above threshold 0.3
    Then tile [4, 4] wind is 0.6
    Then at least one cardinal neighbor of [4, 4] has fire > 0
    # spread_chance = 0.5 * 0.6 * 0.15 = 0.045 per neighbor

  Scenario: Fire does not spread when wind is zero
    Given an 8x8 grid with dense_forest tiles
    Given tile [4, 4] has ElementalState fire=0.5 water=0.0 cold=0.0 wind=0.0
    Given weather is "clear" with no element effects
    When ElementInteractionSystem runs for 1 tick
    Then no cardinal neighbor of [4, 4] has fire > 0

  # --- Interaction 2: rain fills water (rain + dry_soil = wet_soil) ---

  Scenario: Rain weather on dry soil changes terrain to wet soil
    Given a 10x10 grid
    Given tile [5, 5] has terrain type "dry_soil"
    Given tile [5, 5] has ElementalState fire=0.0 water=0.0 cold=0.0 wind=0.0
    Given weather is "rain" with element_effect water=0.05
    When WeatherTickSystem applies weather effects
    Then tile [5, 5] has water level above 0
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] terrain type is "wet_soil"

  # --- Interaction 3: cold freezes water ---

  Scenario: Cold above threshold freezes water on tile
    Given a 10x10 grid
    Given tile [5, 5] has terrain type "water_source"
    Given tile [5, 5] has ElementalState fire=0.0 water=0.5 cold=0.6 wind=0.0
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] water is reduced by freeze_rate 0.1
    Then tile [5, 5] terrain type is "ice"

  Scenario: Cold below threshold does not freeze water
    Given a 10x10 grid
    Given tile [5, 5] has terrain type "water_source"
    Given tile [5, 5] has ElementalState fire=0.0 water=0.5 cold=0.2 wind=0.0
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] terrain type is "water_source"
    Then tile [5, 5] water is not reduced

  # --- Interaction 4: fire evaporates water ---

  Scenario: Fire above threshold evaporates water
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=0.5 water=0.4 cold=0.0 wind=0.0
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] water is reduced by evaporate_rate 0.15

  # --- Interaction 5: wind fans fire ---

  Scenario: Wind above threshold amplifies fire intensity
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=0.2 water=0.0 cold=0.0 wind=0.3
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] fire is approximately 0.2 * 1.1 = 0.22

  # --- Interaction 6: water extinguishes fire ---

  Scenario: Water above threshold reduces fire
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=0.5 water=0.3 cold=0.0 wind=0.0
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] fire is reduced by reduce_rate 0.2

  # --- Element decay ---

  Scenario: Fire decays naturally over time
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=1.0 water=0.0 cold=0.0 wind=0.0
    Given weather is "clear" with no element effects
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] fire is approximately 1.0 * 0.95 = 0.95

  Scenario: Cold decays faster than fire
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=0.0 water=0.0 cold=1.0 wind=0.0
    Given weather is "clear" with no element effects
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] cold is approximately 1.0 * 0.93 = 0.93

  Scenario: Wind does not decay naturally
    Given a 10x10 grid
    Given tile [5, 5] has ElementalState fire=0.0 water=0.0 cold=0.0 wind=0.5
    When ElementInteractionSystem runs for 1 tick
    Then tile [5, 5] wind remains 0.5
    # wind decay_rate is 1.0 — set by weather directly

  # ============================================================================
  # AC7: World simulation runs independently of player camera position
  # ============================================================================

  Scenario: Hazard triggers on fogged tiles the player cannot see
    Given a forest biome map with fog set to all_hidden
    Given a wildfire hazard zone centered at [15, 15] with radius 5
    Given tile [15, 15] visibility is Hidden
    Given the wildfire next_event_tick is 100
    When simulation reaches tick 100
    Then tiles within radius 5 of [15, 15] have TileEnhancement of type "charred_fertile"
    Then tile [15, 15] visibility remains Hidden

  Scenario: Weather affects tiles regardless of visibility
    Given a forest biome map with fog set to all_hidden
    Given tile [15, 15] has ElementalState fire=0.0 water=0.0 cold=0.0 wind=0.0
    Given weather is "rain" with element_effect water=0.05
    When WeatherTickSystem runs for 1 tick
    Then tile [15, 15] water level is increased by 0.05
    Then tile [15, 15] visibility remains Hidden

  Scenario: Element interactions process on hidden tiles
    Given a forest biome map with fog set to all_hidden
    Given tile [15, 15] has ElementalState fire=0.5 water=0.0 cold=0.0 wind=0.0
    Given weather is "wind" with element_effect wind=0.6
    When ElementInteractionSystem runs for 1 tick
    Then fire spread calculations include tile [15, 15] and its neighbors

  # ============================================================================
  # AC8: Watchtower building reveals fog in configurable radius around it
  # ============================================================================

  # --- Happy path ---

  Scenario: Watchtower reveals tiles within radius 8
    Given a forest biome map 20x20 with all tiles Hidden
    Given a watchtower building at [10, 10] with FogRevealer radius 8
    When FogOfWarSystem runs
    Then all tiles within manhattan distance 8 of [10, 10] are Visible
    Then tiles outside manhattan distance 8 of [10, 10] remain Hidden

  Scenario: Multiple watchtowers combine reveal areas
    Given a forest biome map 20x20 with all tiles Hidden
    Given a watchtower building at [5, 5] with FogRevealer radius 8
    Given a watchtower building at [15, 15] with FogRevealer radius 8
    When FogOfWarSystem runs
    Then all tiles within manhattan distance 8 of [5, 5] are Visible
    Then all tiles within manhattan distance 8 of [15, 15] are Visible

  Scenario: Destroying watchtower removes revealed area on next tick
    Given a forest biome map 20x20 with all tiles Hidden
    Given a watchtower building at [10, 10] with FogRevealer radius 8
    When FogOfWarSystem runs
    Then tiles within radius 8 of [10, 10] are Visible
    When the watchtower at [10, 10] is destroyed
    When FogOfWarSystem runs
    Then tiles that were Visible become Revealed (previously seen but not currently visible)

  # --- Fog weather reduces reveal radius ---

  Scenario: Fog weather reduces watchtower reveal radius by 50%
    Given a forest biome map 20x20 with all tiles Hidden
    Given a watchtower building at [10, 10] with FogRevealer radius 8
    Given weather is "fog" with fog_penalty 0.5
    When FogOfWarSystem runs
    Then tiles within manhattan distance 4 of [10, 10] are Visible
    Then tiles between manhattan distance 5 and 8 of [10, 10] are NOT Visible

  # ============================================================================
  # AC9: Player cannot place buildings on hidden (fogged) tiles
  # ============================================================================

  # --- Error path ---

  Scenario: Building placement rejected on hidden tile
    Given a forest biome map 20x20 with all tiles Hidden
    Given tile [15, 15] is an iron_vein tile with visibility Hidden
    When the player issues PlaceBuilding command for "iron_miner" at [15, 15]
    Then the placement is rejected with reason "tile_hidden"
    Then no BuildingPlaced event is emitted

  Scenario: Building placement rejected on fogged tile even if terrain matches
    Given a forest biome map 20x20 with all tiles Hidden
    Given tile [15, 15] is a grass tile with visibility Hidden
    When the player issues PlaceBuilding command for "wind_turbine" at [15, 15]
    Then the placement is rejected with reason "tile_hidden"

  # --- Happy path: placement succeeds on Visible tile ---

  Scenario: Building placement succeeds on visible tile with matching terrain
    Given a forest biome map 20x20 with all tiles Hidden
    Given a watchtower at [10, 10] revealing radius 8
    Given tile [12, 10] is an iron_vein tile
    When FogOfWarSystem runs
    Then tile [12, 10] visibility is Visible
    When the player issues PlaceBuilding command for "iron_miner" at [12, 10]
    Then the building is placed successfully at [12, 10]

  # --- Edge: Revealed but not currently Visible ---

  Scenario: Building placement succeeds on Revealed tile
    Given a forest biome map 20x20
    Given tile [12, 10] is a grass tile with visibility Revealed
    When the player issues PlaceBuilding command for "wind_turbine" at [12, 10]
    Then the building is placed successfully at [12, 10]

  # ============================================================================
  # Quality Map (biome-contextual resource quality)
  # ============================================================================

  Scenario: Forest biome marks wood as high quality
    Given a run on biome "forest"
    When BiomeDB quality_map is queried for "wood"
    Then the quality is "high"

  Scenario: Forest biome marks iron_ore as normal quality
    Given a run on biome "forest"
    When BiomeDB quality_map is queried for "iron_ore"
    Then the quality is "normal"

  Scenario: Volcanic biome has no natural wood
    Given a run on biome "volcanic"
    When BiomeDB quality_map is queried for "wood"
    Then the quality is null (unavailable)

  Scenario: Volcanic biome has no natural water
    Given a run on biome "volcanic"
    When BiomeDB quality_map is queried for "water"
    Then the quality is null (unavailable)

  Scenario: Desert biome marks mana_crystal as high quality
    Given a run on biome "desert"
    When BiomeDB quality_map is queried for "mana_crystal"
    Then the quality is "high"

  Scenario: Ocean biome marks water as high quality
    Given a run on biome "ocean"
    When BiomeDB quality_map is queried for "water"
    Then the quality is "high"

  # ============================================================================
  # Weather system
  # ============================================================================

  Scenario: Weather changes at configured interval
    Given a run on biome "forest" with weather "clear"
    Given clear weather duration is between 400 and 800 ticks
    When simulation advances past the weather duration
    Then the weather type changes to one of: rain, heavy_rain, wind, fog
    Then the new weather's element effects begin applying to tiles

  Scenario: Rain weather increases water and decreases fire on tiles
    Given a run on biome "forest" with weather "rain"
    Given tile [5, 5] has ElementalState fire=0.1 water=0.0 cold=0.0 wind=0.0
    When WeatherTickSystem runs for 1 tick
    Then tile [5, 5] water is increased by 0.05
    Then tile [5, 5] fire is decreased by 0.02

  Scenario: Heavy rain applies stronger water effect than regular rain
    Given a run on biome "forest" with weather "heavy_rain"
    Given tile [5, 5] has ElementalState fire=0.0 water=0.0 cold=0.0 wind=0.0
    When WeatherTickSystem runs for 1 tick
    Then tile [5, 5] water is increased by 0.12

  Scenario: Cold snap weather applies cold element to tiles
    Given a run on biome "desert" with weather "cold_snap"
    Given tile [5, 5] has ElementalState fire=0.0 water=0.0 cold=0.0 wind=0.0
    When WeatherTickSystem runs for 1 tick
    Then tile [5, 5] cold is increased by 0.08

  # ============================================================================
  # PRD edge case: Two hazards overlap on same tiles
  # ============================================================================

  Scenario: Overlapping hazards both apply their effects
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 3
    Given an ash_storm hazard zone centered at [8, 6] with radius 4
    Given tiles [7, 6] and [8, 6] are in both hazard zones
    Given the eruption next_event_tick is 100
    Given the ash_storm next_event_tick is 120
    When simulation reaches tick 100
    Then tiles in eruption zone have TileEnhancement of type "enriched"
    When simulation reaches tick 120
    Then tile [7, 6] enhancement is the stronger of "enriched" (1.5) and "fertile_ash" (1.2)

  # ============================================================================
  # Hazard tier scaling
  # ============================================================================

  Scenario: Eruption intensity increases at T2
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the current tier is T2
    When the eruption hazard triggers
    Then the eruption intensity is 1.3 (base 1.0 * t2_intensity scaling)

  Scenario: Eruption intensity increases further at T3
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the current tier is T3
    When the eruption hazard triggers
    Then the eruption intensity is 1.6 (base 1.0 * t3_intensity scaling)

  # ============================================================================
  # Heat wave special behavior: drains water from manifolds
  # ============================================================================

  Scenario: Heat wave drains water from manifolds in affected zone
    Given a desert biome map with a heat_wave hazard zone centered at [5, 5] with radius 10
    Given a building group at [5, 5] with manifold containing water=10.0
    Given the heat_wave next_event_tick is 100
    When simulation reaches tick 100
    Then the group manifold water is reduced by water_drain_rate 0.1 per tick during the heat wave

  # ============================================================================
  # Tile enhancement duration and expiry
  # ============================================================================

  Scenario: Tile enhancement expires after configured duration
    Given a volcanic biome map with an eruption hazard zone centered at [6, 6] with radius 4
    Given the eruption next_event_tick is 100
    When simulation reaches tick 100
    Then tile [6, 6] has TileEnhancement of type "enriched" with duration 6000 ticks
    When simulation reaches tick 6100
    Then tile [6, 6] has no TileEnhancement

  # ============================================================================
  # Frozen water source stops extraction
  # ============================================================================

  Scenario: Frozen water source tile prevents water pump from extracting
    Given an ocean biome map with a water_source tile at [5, 5]
    Given a water_pump building at [5, 5]
    Given tile [5, 5] has ElementalState fire=0.0 water=0.5 cold=0.6 wind=0.0
    When ElementInteractionSystem runs and freezes the water source to ice
    Then the water_pump at [5, 5] stops producing because terrain is now "ice"
    Then the water_pump ProductionState active is false
