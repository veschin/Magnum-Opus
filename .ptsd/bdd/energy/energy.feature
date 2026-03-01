@feature:energy
Feature: Energy — power generation, distribution, and production throttle

  Energy is the global constraint that creates interdependence between
  production groups. Energy buildings generate power into a global pool.
  Surplus speeds up all groups; deficit forces the player to prioritize
  which groups get power first.

  Energy is instant (no storage, no routing). One unified pool.
  Allocation is at group level, not building level.

  # ────────────────────────────────────────────────
  # AC1: Energy balance displayed in real-time
  # ────────────────────────────────────────────────

  Scenario: Single energy building shows generation in energy pool
    Given a 10x10 grid
    Given a wind_turbine placed at position [5, 5]
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then EnergyPool.totalConsumption equals 0
    Then the energy balance (generation minus consumption) equals 20

  Scenario: Energy balance reflects both generation and consumption
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given a wind_turbine placed at [4, 3] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then EnergyPool.totalConsumption equals 5
    Then the energy balance equals 15

  Scenario: Multiple energy buildings sum their generation
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given a wind_turbine placed at [4, 3] with energy_generation 20
    Given a wind_turbine placed at [4, 4] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 40
    Then EnergyPool.totalConsumption equals 5
    Then the energy balance equals 35

  Scenario: Energy pool updates every tick as buildings change
    Given a 10x10 grid
    Given a wind_turbine placed at [5, 5] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    When an iron_smelter is placed at [5, 6] with energy_consumption 10
    When the simulation ticks once
    Then EnergyPool.totalConsumption equals 10
    Then the energy balance equals 10

  # ────────────────────────────────────────────────
  # AC2: Surplus energy proportionally increases production speed
  # ────────────────────────────────────────────────

  Scenario: Surplus energy speeds up production proportionally
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given a wind_turbine placed at [4, 3] with energy_generation 20
    Given a wind_turbine placed at [4, 4] with energy_generation 20
    When the simulation ticks once
    Then the energy ratio (totalGen / totalConsumption) equals 8.0
    Then the production speed modifier for the miner group is clamped to 1.5

  Scenario: Surplus modifier is capped at 1.5 regardless of excess energy
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given 4 wind_turbines placed with total energy_generation 80
    When the simulation ticks once
    Then the energy ratio equals 16.0
    Then the production speed modifier is clamped at max_modifier 1.5

  Scenario: Moderate surplus provides proportional speed boost
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given an iron_smelter placed at [4, 3] with energy_consumption 10
    Given a wind_turbine placed at [3, 4] with energy_generation 20
    When the simulation ticks once
    Then the energy ratio equals 1.333
    Then the production speed modifier equals 1.333 (between 1.0 and 1.5)

  # ────────────────────────────────────────────────
  # AC3: Deficit energy reduces speed; highest-priority groups throttled last
  # ────────────────────────────────────────────────

  Scenario: Deficit reduces production speed for all groups uniformly at same priority
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] with energy_consumption 5
    Given an iron_smelter placed at [4, 3] with energy_consumption 10
    Given a constructor placed at [5, 3] with energy_consumption 15
    Given a wind_turbine placed at [3, 4] with energy_generation 20
    Given all groups are at default priority medium
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then EnergyPool.totalConsumption equals 30
    Then all medium-priority groups share 20 energy proportional to their demand

  Scenario: High-priority group gets energy first during deficit
    Given a 16x10 grid
    Given an iron_vein tile at [2, 3] and a copper_vein tile at [10, 3]
    Given an iron_miner at [2, 3] and iron_smelter at [3, 3] forming group A
    Given a copper_miner at [10, 3] and copper_smelter at [11, 3] forming group B
    Given a wind_turbine at [6, 3] with energy_generation 20
    Given group A priority is set to high
    Given group B priority is set to low
    When the simulation ticks once
    Then group A (demand 15) receives its full 15 energy allocation
    Then group B (demand 15) receives the remaining 5 energy allocation
    Then group A speed modifier is 1.0
    Then group B speed modifier is 0.333

  Scenario: Three priority tiers distribute energy in order high then medium then low
    Given a 20x10 grid
    Given group A (demand 10) at priority high
    Given group B (demand 10) at priority medium
    Given group C (demand 10) at priority low
    Given total energy generation is 15
    When the simulation ticks once
    Then group A receives 10 energy (full demand)
    Then group B receives 5 energy (partial)
    Then group C receives 0 energy (starved)

  Scenario: Multiple groups at same priority share energy proportionally during deficit
    Given a 16x10 grid
    Given an iron_vein tile at [2, 3] and a copper_vein tile at [10, 3]
    Given group A (iron_miner + iron_smelter, demand 15) at priority high
    Given group B (copper_miner + copper_smelter, demand 15) at priority high
    Given a wind_turbine at [6, 3] with energy_generation 20
    When the simulation ticks once
    Then group A receives 10 energy (proportional: 15/30 * 20)
    Then group B receives 10 energy (proportional: 15/30 * 20)
    Then both groups have speed modifier 0.667

  # ────────────────────────────────────────────────
  # AC4: Player can set group energy priority
  # ────────────────────────────────────────────────

  Scenario: New group defaults to medium priority
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3]
    When the simulation ticks once
    Then the group containing iron_miner has GroupEnergy.priority equal to medium

  Scenario: Player sets group priority to high via command
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] in group A
    Given group A has default priority medium
    When a SetGroupPriority command is issued for group A with priority high
    When the simulation ticks once
    Then group A has GroupEnergy.priority equal to high

  Scenario: Player sets group priority to low via command
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner placed at [3, 3] in group A
    When a SetGroupPriority command is issued for group A with priority low
    When the simulation ticks once
    Then group A has GroupEnergy.priority equal to low

  Scenario: Changing priority mid-deficit immediately reallocates energy
    Given a 16x10 grid
    Given group A (demand 15) at priority low
    Given group B (demand 15) at priority high
    Given total energy generation is 20
    When the simulation ticks once
    Then group B receives 15 energy and group A receives 5 energy
    When a SetGroupPriority command swaps group A to high and group B to low
    When the simulation ticks once
    Then group A receives 15 energy and group B receives 5 energy

  # ────────────────────────────────────────────────
  # AC5: Building a new energy source immediately contributes
  # ────────────────────────────────────────────────

  Scenario: Placing wind turbine immediately adds to energy pool
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given no energy buildings exist
    When the simulation ticks once
    Then EnergyPool.totalGen equals 0
    When a wind_turbine is placed at [4, 3]
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20

  Scenario: Placing water wheel on water source in ocean biome adds generation with biome bonus
    Given a 10x10 grid in ocean biome
    Given a water_source tile at [5, 5]
    Given a water_wheel placed at [5, 5] with base energy_generation 25
    When the simulation ticks once
    Then EnergyPool.totalGen equals 35 (25 * 1.4 ocean biome bonus)

  Scenario: Placing lava generator at T2 adds to energy pool
    Given a 10x10 grid in volcanic biome at tier 2
    Given a lava_source tile at [5, 5]
    Given a lava_generator placed at [5, 5] with energy_generation 50
    When the simulation ticks once
    Then EnergyPool.totalGen equals 50

  Scenario: Mana reactor at T3 generates energy while consuming fuel
    Given a 10x10 grid at tier 3
    Given a mana_reactor placed at [5, 5] with energy_generation 80
    Given the mana_reactor has fuel_recipe requiring 1 mana_crystal per 300 ticks
    Given the mana_reactor group manifold contains 1 mana_crystal
    When the simulation ticks once
    Then EnergyPool.totalGen includes 80 from the mana_reactor
    Then the mana_reactor begins consuming its fuel recipe

  Scenario: Mana reactor without fuel does not generate energy
    Given a 10x10 grid at tier 3
    Given a mana_reactor placed at [5, 5] with energy_generation 80
    Given the mana_reactor group manifold contains 0 mana_crystal
    When the simulation ticks once
    Then EnergyPool.totalGen does not include the mana_reactor output

  # ────────────────────────────────────────────────
  # AC6: Destroying an energy building immediately reduces generation
  # ────────────────────────────────────────────────

  Scenario: Removing wind turbine immediately drops generation
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given a wind_turbine at [4, 3] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then the energy balance equals 15
    When a RemoveBuilding command destroys the wind_turbine at [4, 3]
    When the simulation ticks once
    Then EnergyPool.totalGen equals 0
    Then the energy balance equals -5

  Scenario: Hazard destroying energy building reduces generation immediately
    Given a 10x10 grid with a hazard zone covering [4, 3]
    Given a wind_turbine at [4, 3] with energy_generation 20
    Given an iron_miner at [3, 3] with energy_consumption 5
    When the hazard event triggers and destroys the wind_turbine
    When the simulation ticks once
    Then EnergyPool.totalGen equals 0
    Then the energy balance equals -5

  Scenario: Destroying one of multiple energy buildings reduces but does not zero generation
    Given a 10x10 grid
    Given a wind_turbine at [4, 3] with energy_generation 20
    Given a wind_turbine at [4, 4] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 40
    When a RemoveBuilding command destroys the wind_turbine at [4, 3]
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20

  # ────────────────────────────────────────────────
  # Edge Case: All energy buildings destroyed
  # ────────────────────────────────────────────────

  Scenario: All energy buildings destroyed stops all production
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given an iron_smelter at [4, 3] with energy_consumption 10
    Given a wind_turbine at [3, 4] with energy_generation 20
    When the simulation ticks once
    Then production groups are running with speed modifier above 0
    When a RemoveBuilding command destroys the wind_turbine at [3, 4]
    When the simulation ticks once
    Then EnergyPool.totalGen equals 0
    Then all production groups have speed modifier 0.0 (min_modifier)
    Then all production buildings are effectively stopped

  # ────────────────────────────────────────────────
  # Edge Case: Energy exactly at zero balance
  # ────────────────────────────────────────────────

  Scenario: Exact energy balance gives no bonus and no penalty
    Given a 10x10 grid with iron_vein tiles at [3, 3] and [3, 4]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given an iron_miner at [3, 4] with energy_consumption 5
    Given an iron_smelter at [4, 3] with energy_consumption 10
    Given a wind_turbine at [4, 4] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then EnergyPool.totalConsumption equals 20
    Then the energy ratio equals 1.0
    Then all groups have speed modifier exactly 1.0

  # ────────────────────────────────────────────────
  # Edge Case: Single HIGH priority group with massive deficit
  # ────────────────────────────────────────────────

  Scenario: Single high-priority group gets near-normal speed while others nearly stop
    Given a 20x10 grid
    Given group A (iron_miner + iron_smelter, demand 15) at priority high
    Given group B (copper_miner + copper_smelter, demand 15) at priority low
    Given group C (stone_quarry + sawmill, demand 10) at priority low
    Given a single wind_turbine with energy_generation 20
    When the simulation ticks once
    Then total consumption is 40 and total generation is 20
    Then group A (high) receives its full 15 demand
    Then remaining 5 energy is split between group B and group C proportionally
    Then group B receives 3 energy (15/25 * 5) with speed modifier 0.2
    Then group C receives 2 energy (10/25 * 5) with speed modifier 0.2
    Then group A runs at speed modifier 1.0

  # ────────────────────────────────────────────────
  # Error paths
  # ────────────────────────────────────────────────

  Scenario: Energy buildings with no consumers produce idle surplus
    Given a 10x10 grid
    Given a wind_turbine at [5, 5] with energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20
    Then EnergyPool.totalConsumption equals 0
    Then the energy ratio is treated as 1.0 (no division by zero)
    Then no speed modifier anomaly occurs

  Scenario: Zero consumption results in ratio 1.0 not division by zero
    Given a 10x10 grid
    Given 3 wind_turbines with total energy_generation 60
    Given no production buildings exist
    When the simulation ticks once
    Then the energy ratio is 1.0 (div-by-zero guard: if consumption equals 0 then ratio equals 1)

  Scenario: Placing T2 energy building before T2 is unlocked is rejected
    Given a 10x10 grid in volcanic biome at tier 1
    Given a lava_source tile at [5, 5]
    When a PlaceBuilding command for lava_generator at [5, 5] is issued
    Then the command is rejected because lava_generator requires tier 2
    Then EnergyPool.totalGen remains unchanged

  Scenario: Placing energy building on wrong terrain is rejected
    Given a 10x10 grid with a grass tile at [5, 5]
    When a PlaceBuilding command for water_wheel at [5, 5] is issued
    Then the command is rejected because water_wheel requires water_source terrain
    Then EnergyPool.totalGen remains unchanged

  Scenario: SetGroupPriority command for nonexistent group is rejected
    Given a 10x10 grid
    When a SetGroupPriority command is issued for a nonexistent group ID
    Then the command is rejected
    Then no energy allocation changes occur

  # ────────────────────────────────────────────────
  # Biome bonus interactions
  # ────────────────────────────────────────────────

  Scenario: Wind turbine in desert biome gets 1.3x bonus
    Given a 10x10 grid in desert biome
    Given a wind_turbine placed at [5, 5] with base energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 26 (20 * 1.3 desert biome bonus)

  Scenario: Wind turbine in ocean biome gets 1.1x bonus
    Given a 10x10 grid in ocean biome
    Given a wind_turbine placed at [5, 5] with base energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 22 (20 * 1.1 ocean biome bonus)

  Scenario: Wind turbine in forest biome gets no bonus
    Given a 10x10 grid in forest biome
    Given a wind_turbine placed at [5, 5] with base energy_generation 20
    When the simulation ticks once
    Then EnergyPool.totalGen equals 20 (no biome bonus for wind_turbine in forest)

  # ────────────────────────────────────────────────
  # Energy non-negative invariant
  # ────────────────────────────────────────────────

  Scenario: Allocated energy is never negative for any group
    Given a 20x10 grid
    Given group A (demand 15) at priority high
    Given group B (demand 15) at priority medium
    Given group C (demand 15) at priority low
    Given total energy generation is 10
    When the simulation ticks once
    Then group A allocated energy is greater than or equal to 0
    Then group B allocated energy is greater than or equal to 0
    Then group C allocated energy is greater than or equal to 0

  Scenario: Even with zero total generation all allocated values are zero not negative
    Given a 10x10 grid
    Given group A (demand 15) at priority high
    Given no energy buildings exist
    When the simulation ticks once
    Then group A GroupEnergy.allocated equals 0
    Then group A speed modifier equals 0.0

  # ────────────────────────────────────────────────
  # Integration: energy modifier flows into production
  # ────────────────────────────────────────────────

  Scenario: Surplus energy modifier accelerates recipe progress
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] producing iron_ore
    Given 2 wind_turbines with total energy_generation 40 and miner consumption 5
    Given the energy ratio is 8.0 clamped to speed modifier 1.5
    When the simulation ticks 10 times
    Then the iron_miner recipe progress is 1.5x faster than baseline (10 * 1.5 / duration)

  Scenario: Deficit energy modifier slows recipe progress
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given an iron_smelter at [4, 3] with energy_consumption 10
    Given a wind_turbine with energy_generation 20 and total consumption 15
    Given the miner and smelter are in one group at priority medium
    Given the energy ratio is 1.333 and speed modifier is 1.333
    When the simulation ticks 10 times
    Then production progress advances at 1.333x baseline speed

  Scenario: Fully starved group makes zero recipe progress
    Given a 10x10 grid with an iron_vein tile at [3, 3]
    Given an iron_miner at [3, 3] with energy_consumption 5
    Given no energy buildings exist
    When the simulation ticks 10 times
    Then the iron_miner recipe progress remains at 0 (speed modifier is 0.0)
