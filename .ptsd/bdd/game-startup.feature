@feature:game-startup
Feature: Game Startup — initialize all run state before the first simulation tick

  GameStartupPlugin writes all required ECS state in the Bevy Startup
  schedule: recipe validation, terrain generation, starting kit, opus tree,
  fog reveal, and run configuration. After one app.update() the simulation
  finds a fully initialized world ready for the first tick.

  # ────────────────────────────────────────────────
  # AC1: Recipe Validation
  # ────────────────────────────────────────────────

  Scenario: Every BuildingType variant has a valid default recipe
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then default_recipe(bt) returns a Recipe for all 35 BuildingType variants without panic

  Scenario: Extractors have empty inputs and mall buildings output to inventory
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then extractors [IronMiner, CopperMiner, StoneQuarry, WaterPump, ObsidianDrill, ManaExtractor, LavaSiphon] have inputs == []
    Then mall buildings [Constructor, Toolmaker, Assembler] have output_to_inventory == true
    Then energy buildings [WindTurbine, WaterWheel, LavaGenerator, ManaReactor] have duration_ticks == 1
    Then utility buildings [Watchtower, Trader, SacrificeAltar] have duration_ticks == 1

  # ────────────────────────────────────────────────
  # AC2: Terrain Generation
  # ────────────────────────────────────────────────

  Scenario: Terrain generation with seed 42 places resource clusters near spawn
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin with seed 42
    When the app updates once
    Then Grid dimensions are 64x64 with 4096 total cells
    Then Grid.terrain contains IronVein cells within Manhattan distance 15 of spawn (15,15)
    Then Grid.terrain contains CopperVein cells within Manhattan distance 15 of spawn (15,15)
    Then Grid.terrain contains StoneDeposit cells within Manhattan distance 15 of spawn (15,15)
    Then Grid.terrain contains WaterSource cells within Manhattan distance 15 of spawn (15,15)
    Then at least 50 cells are non-Grass terrain

  # ────────────────────────────────────────────────
  # AC3: Terrain Determinism
  # ────────────────────────────────────────────────

  Scenario: Same seed produces identical terrain maps
    Given two fresh Apps both configured with seed 42
    When both apps update once
    Then Grid.terrain from app A is identical to Grid.terrain from app B cell by cell

  # ────────────────────────────────────────────────
  # AC4: Starting Kit
  # ────────────────────────────────────────────────

  Scenario: Starting kit populates inventory with exact building counts
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then Inventory.buildings contains IronMiner=4
    Then Inventory.buildings contains CopperMiner=2
    Then Inventory.buildings contains StoneQuarry=2
    Then Inventory.buildings contains WaterPump=2
    Then Inventory.buildings contains IronSmelter=2
    Then Inventory.buildings contains CopperSmelter=1
    Then Inventory.buildings contains Sawmill=1
    Then Inventory.buildings contains TreeFarm=1
    Then Inventory.buildings contains Constructor=1
    Then Inventory.buildings contains WindTurbine=3
    Then Inventory.buildings contains Watchtower=1
    Then Inventory.buildings total count equals 20

  Scenario: All starting kit buildings are tier 1
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then every building type in Inventory.buildings has tier == 1

  # ────────────────────────────────────────────────
  # AC5: Opus Tree Initialization
  # ────────────────────────────────────────────────

  Scenario: Opus tree has 7 milestone nodes with correct initial state
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then OpusTreeResource.main_path has exactly 7 nodes
    Then node 0 resource is IronBar with required_rate 2.0
    Then node 1 resource is CopperBar with required_rate 1.5
    Then node 2 resource is Plank with required_rate 2.0
    Then node 3 resource is SteelPlate with required_rate 1.0
    Then node 4 resource is RefinedCrystal with required_rate 0.5
    Then node 5 resource is RunicAlloy with required_rate 0.3
    Then node 6 resource is OpusIngot with required_rate 0.1
    Then all 7 nodes have current_rate == 0.0 and sustained == false
    Then OpusTreeResource.sustain_ticks_required equals 600

  # ────────────────────────────────────────────────
  # AC6: Fog Initialization
  # ────────────────────────────────────────────────

  Scenario: Fog reveals Manhattan distance 12 diamond around spawn
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then FogMap.revealed contains exactly 313 cells
    Then all cells within Manhattan distance 12 of spawn (15,15) are revealed
    Then the IronVein cluster at (10,10) is within the revealed area
    Then the CopperVein cluster at (20,10) is within the revealed area
    Then the StoneDeposit cluster at (15,20) is within the revealed area
    Then the WaterSource cluster at (25,15) is within the revealed area

  # ────────────────────────────────────────────────
  # AC7: Run Configuration
  # ────────────────────────────────────────────────

  Scenario: Run config initializes with correct defaults
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then RunConfig.current_tick equals 1 (startup sets 0, first Update tick increments to 1)
    Then RunConfig.max_ticks equals 108000
    Then RunConfig.biome equals Forest
    Then RunConfig.tps equals 20
    Then TierState.current_tier equals 1
    Then RunState.status equals InProgress

  # ────────────────────────────────────────────────
  # AC8: Startup Schedule Integration
  # ────────────────────────────────────────────────

  Scenario: All resources populated after single app update
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin
    When the app updates once
    Then Grid resource exists and has 64x64 dimensions
    Then Inventory resource exists and has 20 buildings
    Then OpusTreeResource resource exists and has 7 main_path nodes
    Then FogMap resource exists and has 313 revealed cells
    Then RunConfig resource exists and has current_tick 0
    Then TierState resource exists and has current_tier 1
    Then RunState resource exists and has status InProgress

  # ────────────────────────────────────────────────
  # Edge Cases
  # ────────────────────────────────────────────────

  Scenario: Seed 0 produces valid terrain without panic
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin with seed 0
    When the app updates once
    Then Grid.terrain has at least 50 non-Grass cells
    Then no panic occurred during terrain generation

  Scenario: Boundary grid cells are valid terrain
    Given a fresh App with MinimalPlugins + SimulationPlugin + GameStartupPlugin with seed 42
    When the app updates once
    Then Grid.terrain at (0,0) is a valid TerrainType
    Then Grid.terrain at (63,63) is a valid TerrainType

  Scenario: OpusTree Default state has empty main_path before startup runs
    Given a fresh App with MinimalPlugins + SimulationPlugin (without GameStartupPlugin)
    Then OpusTreeResource.main_path is empty
    Then OpusTreeResource.completion_pct equals 0.0
