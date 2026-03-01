@feature:ux
Feature: UX Tools — Production Calculator, Chain Visualizer, Efficiency Dashboard

  Built-in production intelligence: calculator, chain visualizer, efficiency dashboard.
  Players should never need external calculators or spreadsheets.
  All tools are read-only overlays on simulation state — they never mutate the ECS world.

  # ─────────────────────────────────────────────
  # AC1: Calculator accepts target item + rate, outputs required building chain
  # ─────────────────────────────────────────────

  Scenario: Calculator computes simple T1 chain — iron bars
    Given the current tier is 1
    Given the biome is "forest"
    Given RecipeDB contains recipe "smelt_iron" with inputs [iron_ore: 2] and outputs [iron_bar: 1] and duration 120 ticks
    Given BuildingDB contains "iron_miner" with energy_demand 5 producing iron_ore at 1.0 per 60 ticks
    Given BuildingDB contains "iron_smelter" with energy_demand 10 consuming recipe "smelt_iron"
    Given BuildingDB contains "wind_turbine" with energy_output 20
    When the calculator receives target_resource "iron_bar" at target_rate 2.0 per minute
    Then the calculator outputs buildings_needed: iron_miner 4, iron_smelter 2
    Then the calculator outputs energy_needed 40
    Then the calculator outputs energy_buildings: wind_turbine 2

  Scenario: Calculator computes T1 chain — planks from tree farms
    Given the current tier is 1
    Given the biome is "forest"
    Given RecipeDB contains recipe "grow_wood" with inputs [water: 3] and outputs [wood: 2] and duration 180 ticks
    Given RecipeDB contains recipe "saw_planks" with inputs [wood: 1] and outputs [plank: 2] and duration 80 ticks
    Given BuildingDB contains "water_pump" with energy_demand 3
    Given BuildingDB contains "tree_farm" with energy_demand 8 consuming recipe "grow_wood"
    Given BuildingDB contains "sawmill" with energy_demand 6 consuming recipe "saw_planks"
    When the calculator receives target_resource "plank" at target_rate 4.0 per minute
    Then the calculator outputs buildings_needed: water_pump 2, tree_farm 2, sawmill 2
    Then the calculator outputs energy_needed 34

  Scenario: Calculator computes multi-step T2 chain — steel plates
    Given the current tier is 2
    Given the biome is "forest"
    Given BuildingDB contains "iron_miner" with energy_demand 5
    Given BuildingDB contains "copper_miner" with energy_demand 5
    Given BuildingDB contains "iron_smelter" with energy_demand 10
    Given BuildingDB contains "copper_smelter" with energy_demand 10
    Given BuildingDB contains "steel_forge" with energy_demand 18
    When the calculator receives target_resource "steel_plate" at target_rate 1.0 per minute
    Then the calculator outputs buildings_needed: iron_miner 4, copper_miner 2, iron_smelter 2, copper_smelter 1, steel_forge 1
    Then the calculator outputs energy_needed 68

  Scenario: Calculator computes organic chain requiring combat group
    Given the current tier is 2
    Given the biome is "forest"
    Given BuildingDB contains "imp_camp" with energy_demand 10
    Given BuildingDB contains "breeding_pen" with energy_demand 8
    Given BuildingDB contains "tannery" with energy_demand 12
    Given RecipeDB contains recipe "tan_leather" with inputs [hide: 3, herbs: 1] and outputs [treated_leather: 1]
    When the calculator receives target_resource "treated_leather" at target_rate 1.0 per minute
    Then the calculator outputs buildings_needed: imp_camp 1, breeding_pen 1, tannery 1
    Then the calculator outputs energy_needed 30
    Then the calculator outputs a note containing "Requires combat group for organic resources"

  Scenario: Calculator returns zero buildings for zero rate
    Given the current tier is 1
    Given the biome is "forest"
    When the calculator receives target_resource "iron_bar" at target_rate 0.0 per minute
    Then the calculator outputs buildings_needed: empty
    Then the calculator outputs energy_needed 0

  # ─────────────────────────────────────────────
  # AC5: Calculator accounts for current resource quality (normal/high)
  # ─────────────────────────────────────────────

  Scenario: Calculator accounts for HIGH quality resource in volcanic biome
    Given the current tier is 1
    Given the biome is "volcanic"
    Given BiomeDB quality map for "volcanic" has iron_ore as HIGH quality
    Given BuildingDB contains "iron_miner" with energy_demand 5
    Given BuildingDB contains "iron_smelter" with energy_demand 10
    When the calculator receives target_resource "iron_bar" at target_rate 2.0 per minute
    Then the calculator outputs buildings_needed: iron_miner 4, iron_smelter 2
    Then the calculator outputs a note containing "HIGH quality iron_ore in volcanic biome"

  # ─────────────────────────────────────────────
  # AC1 — Error paths
  # ─────────────────────────────────────────────

  Scenario: Calculator rejects tier-locked resource request
    Given the current tier is 1
    Given the biome is "forest"
    Given BuildingDB contains "runic_forge" at tier 3
    When the calculator receives target_resource "runic_alloy" at target_rate 1.0 per minute
    Then the calculator outputs error "tier_locked"
    Then the calculator outputs message containing "Requires T3"
    Then the calculator outputs required_tier 3

  Scenario: Calculator rejects resource unavailable in current biome
    Given the current tier is 2
    Given the biome is "forest"
    Given BiomeDB does not contain terrain "obsidian_vein" in biome "forest"
    When the calculator receives target_resource "obsidian_shard" at target_rate 1.0 per minute
    Then the calculator outputs error "biome_unavailable"
    Then the calculator outputs message containing "obsidian_vein terrain (not available in forest biome)"

  # ─────────────────────────────────────────────
  # AC2: Chain visualizer highlights bottlenecks
  # ─────────────────────────────────────────────

  Scenario: Chain visualizer highlights smelter as bottleneck below 50% capacity
    Given a 16x10 grid
    Given terrain tile iron_vein at positions [2,5], [3,5], [2,6]
    Given a placed iron_miner at [2,5]
    Given a placed iron_miner at [3,5]
    Given a placed iron_miner at [2,6]
    Given a placed iron_smelter at [4,5]
    Given a placed wind_turbine at [4,6]
    Given the bottleneck threshold red is 0.5
    Given the bottleneck threshold yellow is 0.8
    When the chain visualizer is activated
    Then the group containing iron_smelter is highlighted red
    Then the visualizer shows the smelter group is producing below 50% of potential output

  Scenario: Chain visualizer highlights group producing below 80% capacity as yellow
    Given a 16x10 grid
    Given terrain tile iron_vein at positions [2,5], [3,5]
    Given a placed iron_miner at [2,5]
    Given a placed iron_miner at [3,5]
    Given a placed iron_smelter at [4,5]
    Given a placed iron_smelter at [3,6]
    Given a placed wind_turbine at [5,5]
    Given the bottleneck threshold yellow is 0.8
    When the chain visualizer is activated
    Then the group with insufficient input is highlighted yellow

  Scenario: Chain visualizer shows group boundaries and path connections
    Given a 16x10 grid
    Given group A with 2 iron_miners at [2,5] and [3,5]
    Given group B with 1 iron_smelter at [6,5]
    Given a rune path connecting group A output to group B input
    When the chain visualizer is activated
    Then the visualizer shows group A boundary
    Then the visualizer shows group B boundary
    Then the visualizer shows the path connection between group A and group B
    Then the visualizer shows throughput on the path

  Scenario: Chain visualizer shows flow direction with animated arrows
    Given a 16x10 grid
    Given group A producing iron_ore
    Given group B consuming iron_ore
    Given a rune path from group A to group B
    When the chain visualizer is activated
    Then the visualizer shows animated arrows from group A to group B
    Then the visualizer shows the resource amount flowing on the path

  Scenario: Chain visualizer with zero groups shows empty overlay
    Given a 10x10 grid
    Given no buildings placed
    When the chain visualizer is activated
    Then the visualizer shows an empty overlay
    Then the visualizer displays message "No production groups — place buildings to start"
    Then no error or crash occurs

  # ─────────────────────────────────────────────
  # AC3: Dashboard shows production rates, energy balance, resource stockpiles
  # ─────────────────────────────────────────────

  Scenario: Dashboard displays energy balance gauge — surplus
    Given a 16x10 grid
    Given EnergyPool totalGen is 60.0
    Given EnergyPool totalConsumption is 40.0
    When the dashboard is rendered
    Then the energy gauge shows value 20.0
    Then the energy gauge color is green

  Scenario: Dashboard displays energy balance gauge — deficit
    Given a 16x10 grid
    Given terrain tile iron_vein at position [2,5]
    Given a placed iron_miner at [2,5]
    Given a placed iron_smelter at [3,5]
    Given a placed constructor at [4,5]
    Given no energy buildings placed
    Given EnergyPool totalGen is 0.0
    Given EnergyPool totalConsumption is 25.0
    When the dashboard is rendered
    Then the energy gauge shows value -25.0
    Then the energy gauge color is red

  Scenario: Dashboard displays energy balance gauge — exact zero
    Given EnergyPool totalGen is 40.0
    Given EnergyPool totalConsumption is 40.0
    When the dashboard is rendered
    Then the energy gauge shows value 0.0
    Then the energy gauge color is yellow

  Scenario: Dashboard displays opus progress bar
    Given an OpusTree with 5 total nodes
    Given 2 nodes have sustained == true
    When the dashboard is rendered
    Then the opus progress bar shows 40%

  Scenario: Dashboard displays current tier badge
    Given TierState currentTier is 2
    When the dashboard is rendered
    Then the tier badge shows value 2

  Scenario: Dashboard displays production rate time series
    Given a sample_interval of 20 ticks
    Given a history_window of 1200 ticks
    Given group A producing iron_ore at 3.0 items/min
    Given group B producing iron_bar at 1.5 items/min
    When the dashboard is rendered
    Then the production rates graph shows series for iron_ore at 3.0 items/min
    Then the production rates graph shows series for iron_bar at 1.5 items/min

  Scenario: Dashboard displays opus rate vs milestone target comparison
    Given opus node for iron_ore with required_rate 4.0
    Given opus node for iron_bar with required_rate 3.0
    Given current production rate of iron_ore is 5.2
    Given current production rate of iron_bar is 1.8
    When the dashboard is rendered
    Then the rate comparison bar for iron_ore shows current 5.2 vs required 4.0
    Then the rate comparison bar for iron_bar shows current 1.8 vs required 3.0
    Then the iron_ore comparison is styled as above-target
    Then the iron_bar comparison is styled as below-target

  Scenario: Dashboard displays group resource stockpiles
    Given group "Iron Extraction" with manifold containing iron_ore 25.0
    Given group "Iron Processing" with manifold containing iron_ore 3.0, iron_bar 12.0
    When the dashboard is rendered
    Then the stockpiles section shows group "Iron Extraction" with iron_ore 25.0
    Then the stockpiles section shows group "Iron Processing" with iron_ore 3.0 and iron_bar 12.0

  Scenario: Dashboard displays building inventory counts
    Given Inventory contains iron_miner 3, iron_smelter 1, wind_turbine 2
    When the dashboard is rendered
    Then the inventory list shows iron_miner 3
    Then the inventory list shows iron_smelter 1
    Then the inventory list shows wind_turbine 2

  Scenario: Dashboard displays energy allocation per group
    Given group "Miners" with allocated_energy 20.0 and priority HIGH
    Given group "Smelters" with allocated_energy 15.0 and priority MEDIUM
    When the dashboard is rendered
    Then the energy allocation chart shows group "Miners" with 20.0 energy and priority HIGH
    Then the energy allocation chart shows group "Smelters" with 15.0 energy and priority MEDIUM

  Scenario: Dashboard displays energy over time graph
    Given a sample_interval of 20 ticks
    Given a history_window of 2400 ticks
    Given total_generation history series
    Given total_consumption history series
    When the dashboard is rendered
    Then the energy history graph shows total_generation series
    Then the energy history graph shows total_consumption series

  # ─────────────────────────────────────────────
  # AC3 — Edge cases: empty/zero states
  # ─────────────────────────────────────────────

  Scenario: Dashboard at run start shows all zeros without errors
    Given a 10x10 grid
    Given no buildings placed
    Given EnergyPool totalGen is 0.0
    Given EnergyPool totalConsumption is 0.0
    Given Inventory is empty
    When the dashboard is rendered
    Then the energy gauge shows value 0.0
    Then all production rate values show 0.0
    Then all stockpile values show 0.0
    Then no error or crash occurs

  Scenario: Dashboard with zero energy shows halted production message
    Given no energy buildings placed
    Given EnergyPool totalGen is 0.0
    When the dashboard is rendered
    Then the dashboard displays message "No energy — production halted"

  # ─────────────────────────────────────────────
  # AC4: All UX tools accessible without pausing the game
  # ─────────────────────────────────────────────

  Scenario: Calculator is accessible while simulation runs
    Given the simulation is running at tick 500
    When the player opens the calculator
    Then the calculator UI is displayed
    Then the simulation tick advances to at least 501

  Scenario: Chain visualizer is accessible while simulation runs
    Given the simulation is running at tick 500
    When the player activates the chain visualizer overlay
    Then the chain visualizer overlay is displayed
    Then the simulation tick advances to at least 501

  Scenario: Dashboard is accessible while simulation runs
    Given the simulation is running at tick 500
    When the player opens the dashboard
    Then the dashboard UI is displayed
    Then the simulation tick advances to at least 501

  Scenario: Multiple UX tools can be open simultaneously
    Given the simulation is running at tick 500
    When the player opens the calculator
    When the player opens the dashboard
    Then both the calculator and dashboard are displayed
    Then the simulation tick advances to at least 501

  # ─────────────────────────────────────────────
  # AC5 — additional quality edge cases
  # ─────────────────────────────────────────────

  Scenario: Calculator shows NORMAL quality when biome has no quality bonus
    Given the current tier is 1
    Given the biome is "forest"
    Given BiomeDB quality map for "forest" has iron_ore as NORMAL quality
    When the calculator receives target_resource "iron_bar" at target_rate 2.0 per minute
    Then the calculator outputs buildings_needed: iron_miner 4, iron_smelter 2
    Then the calculator does not output a quality note

  # ─────────────────────────────────────────────
  # Cross-feature: calculator + progression integration
  # ─────────────────────────────────────────────

  Scenario: Calculator at T2 can compute T1 and T2 chains
    Given the current tier is 2
    Given the biome is "forest"
    When the calculator receives target_resource "iron_bar" at target_rate 2.0 per minute
    Then the calculator outputs buildings_needed without any tier_locked error
    When the calculator receives target_resource "steel_plate" at target_rate 1.0 per minute
    Then the calculator outputs buildings_needed without any tier_locked error

  Scenario: Calculator at T1 rejects T2 resource request
    Given the current tier is 1
    Given the biome is "forest"
    Given BuildingDB contains "steel_forge" at tier 2
    When the calculator receives target_resource "steel_plate" at target_rate 1.0 per minute
    Then the calculator outputs error "tier_locked"
    Then the calculator outputs required_tier 2

  # ─────────────────────────────────────────────
  # Data freshness: tools reflect live simulation state
  # ─────────────────────────────────────────────

  Scenario: Dashboard updates when a new energy building is placed
    Given a 16x10 grid
    Given a placed wind_turbine at [5,5] with energy_output 20
    Given EnergyPool totalGen is 20.0
    Given EnergyPool totalConsumption is 15.0
    When the dashboard is rendered
    Then the energy gauge shows value 5.0
    Given a second wind_turbine placed at [6,5] with energy_output 20
    Given EnergyPool totalGen is updated to 40.0
    When the dashboard is rendered again
    Then the energy gauge shows value 25.0

  Scenario: Chain visualizer updates when a building is destroyed
    Given a 16x10 grid
    Given group A with 3 iron_miners
    Given group B with 1 iron_smelter
    Given the smelter group is highlighted red as bottleneck
    When 2 iron_miners are removed from group A
    When the chain visualizer is re-rendered
    Then the smelter group is no longer highlighted as bottleneck

  Scenario: Dashboard reflects rate drop to zero when all energy destroyed
    Given a 16x10 grid
    Given a placed wind_turbine at [5,5]
    Given production groups running normally
    When all energy buildings are destroyed
    When the dashboard is rendered
    Then the energy gauge shows value negative or zero
    Then all production rates show 0.0 or near-zero
