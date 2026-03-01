@feature:transport
Feature: Transport — Resource movement between building groups
  Two systems: rune paths for solids, pipes for liquids.
  Minion carry as automatic fallback before paths are built.
  Global tier upgrades auto-upgrade all existing paths and pipes.

  # ────────────────────────────────────────────────────────
  # AC1: Player can draw rune path from output sender of
  #      group A to input receiver of group B
  # ────────────────────────────────────────────────────────

  Scenario: Draw rune path between two groups
    Given a 16x10 grid
    Given an iron_vein tile at position [2, 5]
    Given a placed iron_miner at position [2, 5] in group A
    Given a placed wind_turbine at position [3, 5] in group A
    Given a placed iron_smelter at position [10, 5] in group B
    Given a placed wind_turbine at position [11, 5] in group B
    Given group A has an output sender for iron_ore
    Given group B has an input receiver for iron_ore
    When the player issues DrawPath command from group A output to group B input with waypoints [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5], [9, 5]]
    Then a rune_path entity is created with 6 segment tiles
    Then a PathConnection exists linking group A to group B
    Then a PathConnected event is emitted

  Scenario: Draw pipe between two groups for liquid resource
    Given a 12x10 grid
    Given a water_source tile at position [2, 5]
    Given a placed water_pump at position [2, 5] in group A
    Given a placed wind_turbine at position [3, 5] in group A
    Given a placed tree_farm at position [8, 5] in group B
    Given a placed wind_turbine at position [8, 7] in group B
    Given group A has an output sender for water
    Given group B has an input receiver for water
    When the player issues DrawPipe command from group A output to group B input with waypoints [[4, 5], [5, 5], [6, 5], [7, 5]]
    Then a pipe entity is created with 4 segment tiles
    Then a PathConnection exists linking group A to group B
    Then the pipe resourceClass is Liquid

  Scenario: Reject DrawPath when waypoint tile is impassable
    Given a 16x10 grid
    Given an iron_vein tile at position [2, 5]
    Given a lava_source tile at position [6, 5]
    Given a placed iron_miner at position [2, 5] in group A
    Given a placed iron_smelter at position [10, 5] in group B
    When the player issues DrawPath command from group A to group B with waypoints [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5], [9, 5]]
    Then the DrawPath command is rejected
    Then no path entity is created

  Scenario: Reject DrawPath when waypoint exceeds max path length of 32 tiles
    Given a 50x10 grid
    Given a placed iron_miner at position [2, 5] in group A
    Given a placed iron_smelter at position [40, 5] in group B
    When the player issues DrawPath command from group A to group B with 35 waypoint tiles
    Then the DrawPath command is rejected
    Then no path entity is created

  # ────────────────────────────────────────────────────────
  # AC2: Solid resource models visibly roll along rune paths
  #      at tier-appropriate speed
  # ────────────────────────────────────────────────────────

  Scenario: Solid cargo moves along T1 rune path at speed 1.0 cells per tick
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 2 iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the path with resource iron_ore
    Then the Cargo positionOnPath is 1.0

  Scenario: Solid cargo moves along T2 rune path at speed 2.0 cells per tick
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 2
    Given group A manifold contains 5 iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the path with resource iron_ore
    Then the Cargo positionOnPath is 2.0

  Scenario: Solid cargo moves along T3 rune path at speed 3.0 cells per tick
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 3
    Given group A manifold contains 10 iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the path with resource iron_ore
    Then the Cargo positionOnPath is 3.0

  Scenario: Cargo arriving at path end delivers resource to destination manifold
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 1
    Given a Cargo entity on the path with iron_ore amount 2 at positionOnPath 5.5
    Given the path has 6 segment tiles (length 6)
    When 1 simulation tick runs (Phase 5: Transport)
    Then the Cargo positionOnPath becomes 6.5 which exceeds path length
    Then 2 iron_ore is added to group B manifold
    Then the Cargo entity is destroyed

  # ────────────────────────────────────────────────────────
  # AC3: Liquid resources visibly flow through pipes at
  #      tier-appropriate speed
  # ────────────────────────────────────────────────────────

  Scenario: Liquid cargo moves through T1 pipe at speed 1.5 cells per tick
    Given a 12x10 grid with pipe_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 3 water
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the pipe with resource water
    Then the Cargo positionOnPath is 1.5

  Scenario: Liquid cargo moves through T2 pipe at speed 3.0 cells per tick
    Given a 12x10 grid with pipe_basic fixture
    Given the transport tier is 2
    Given group A manifold contains 8 water
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the pipe with resource water
    Then the Cargo positionOnPath is 3.0

  Scenario: Liquid cargo moves through T3 pipe at speed 4.5 cells per tick
    Given a 12x10 grid with pipe_basic fixture
    Given the transport tier is 3
    Given group A manifold contains 15 water
    When 1 simulation tick runs (Phase 5: Transport)
    Then a Cargo entity is created on the pipe with resource water
    Then the Cargo positionOnPath is 4.5

  # ────────────────────────────────────────────────────────
  # AC4: Unlocking T2 upgrades all T1 paths and pipes
  #      globally without player action
  # ────────────────────────────────────────────────────────

  Scenario: T2 unlock upgrades all existing T1 rune paths to T2
    Given a 16x10 grid with tier_upgrade fixture
    Given the transport tier is 1
    Given a rune_path exists from group A to group B at tier 1 with capacity 2 and speed 1.0
    When TierUnlocked event fires for tier 2
    Then the rune_path tier becomes 2
    Then the rune_path capacity becomes 5
    Then the rune_path speed becomes 2.0

  Scenario: T2 unlock upgrades all existing T1 pipes to T2
    Given a 12x10 grid with pipe_basic fixture
    Given the transport tier is 1
    Given a pipe exists from group A to group B at tier 1 with capacity 3 and speed 1.5
    When TierUnlocked event fires for tier 2
    Then the pipe tier becomes 2
    Then the pipe capacity becomes 8
    Then the pipe speed becomes 3.0

  Scenario: T3 unlock upgrades all existing paths and pipes to T3
    Given a 20x10 grid
    Given the transport tier is 2
    Given a rune_path at tier 2 with capacity 5 and speed 2.0
    Given a pipe at tier 2 with capacity 8 and speed 3.0
    When TierUnlocked event fires for tier 3
    Then the rune_path tier becomes 3 with capacity 10 and speed 3.0
    Then the pipe tier becomes 3 with capacity 15 and speed 4.5

  Scenario: Newly built path after T2 unlock is created at T2 tier
    Given the transport tier is 2
    Given a 16x10 grid
    Given group A with an output sender
    Given group B with an input receiver
    When the player issues DrawPath command from group A to group B
    Then the created rune_path has tier 2 with capacity 5 and speed 2.0

  # ────────────────────────────────────────────────────────
  # AC5: Path throughput is capped by tier; excess resources
  #      queue at sender
  # ────────────────────────────────────────────────────────

  Scenario: T1 rune path caps throughput at 2 items per tick
    Given a 16x10 grid with path_throughput_cap fixture
    Given the transport tier is 1
    Given group A manifold contains 10 iron_ore
    Given T1 rune_path capacity is 2 items per tick
    When 1 simulation tick runs (Phase 5: Transport)
    Then 2 iron_ore is launched as Cargo on the path
    Then 8 iron_ore remains in group A manifold

  Scenario: T2 rune path caps throughput at 5 items per tick
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 2
    Given group A manifold contains 12 iron_ore
    Given T2 rune_path capacity is 5 items per tick
    When 1 simulation tick runs (Phase 5: Transport)
    Then 5 iron_ore is launched as Cargo on the path
    Then 7 iron_ore remains in group A manifold

  Scenario: T1 pipe caps throughput at 3 units per tick
    Given a 12x10 grid with pipe_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 10 water
    Given T1 pipe capacity is 3 units per tick
    When 1 simulation tick runs (Phase 5: Transport)
    Then 3 water is launched as Cargo on the pipe
    Then 7 water remains in group A manifold

  Scenario: Flow is limited by destination demand when below capacity
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 10 iron_ore
    Given T1 rune_path capacity is 2 items per tick
    Given group B input demand for iron_ore is 1
    When 1 simulation tick runs (Phase 5: Transport)
    Then 1 iron_ore is launched as Cargo on the path
    Then 9 iron_ore remains in group A manifold

  # ────────────────────────────────────────────────────────
  # AC6: Paths and pipes cannot overlap on the same tile
  #      (must route around)
  # ────────────────────────────────────────────────────────

  Scenario: Reject second path through tiles already occupied by a path
    Given a 16x10 grid with path_overlap_attempt fixture
    Given a rune_path from group A to group C occupying tiles [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5]]
    When the player issues DrawPath from group B to group D with waypoints [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5]]
    Then the DrawPath command is rejected because tiles are already occupied

  Scenario: Reject pipe through tiles already occupied by a rune path
    Given a 16x10 grid
    Given a rune_path occupying tiles [[4, 5], [5, 5], [6, 5]]
    When the player issues DrawPipe with waypoints [[4, 5], [5, 5], [6, 5]]
    Then the DrawPipe command is rejected because tiles are already occupied

  Scenario: Reject rune path through tiles already occupied by a pipe
    Given a 16x10 grid
    Given a pipe occupying tiles [[4, 5], [5, 5], [6, 5]]
    When the player issues DrawPath with waypoints [[5, 5]]
    Then the DrawPath command is rejected because tiles are already occupied

  Scenario: Allow path on tiles adjacent to but not overlapping existing path
    Given a 16x10 grid
    Given a rune_path occupying tiles [[4, 5], [5, 5], [6, 5]]
    When the player issues DrawPath with waypoints [[4, 6], [5, 6], [6, 6]]
    Then the DrawPath command succeeds
    Then a new rune_path entity is created with 3 segment tiles

  # ────────────────────────────────────────────────────────
  # AC7: Destroying a path segment disconnects the route;
  #      resources stop flowing
  # ────────────────────────────────────────────────────────

  Scenario: Destroying middle segment stops resource flow
    Given a 16x10 grid with path_destroyed_mid_segment fixture
    Given a rune_path from group A to group B with waypoints [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5], [9, 5]]
    Given group A manifold contains 5 iron_ore
    When path segment at [6, 5] is destroyed
    Then a PathDisconnected event is emitted
    When 1 simulation tick runs (Phase 5: Transport)
    Then no Cargo is launched on the disconnected path
    Then 5 iron_ore remains in group A manifold

  Scenario: Cargo in transit is lost when path segment is destroyed
    Given a 16x10 grid with rune_path_basic fixture
    Given a Cargo entity on the path with iron_ore amount 2 at positionOnPath 3.0
    When path segment at [6, 5] is destroyed
    Then the Cargo entity is destroyed
    Then the iron_ore carried by the Cargo is lost (conservation exception: hazard destruction)
    Then a PathDisconnected event is emitted

  Scenario: Destroying first segment of path disconnects the route
    Given a 16x10 grid with rune_path_basic fixture
    Given a rune_path from group A to group B
    When path segment at [4, 5] is destroyed
    Then a PathDisconnected event is emitted
    Then no resources flow through the path

  Scenario: Destroying last segment of path disconnects the route
    Given a 16x10 grid with rune_path_basic fixture
    Given a rune_path from group A to group B
    When path segment at [9, 5] is destroyed
    Then a PathDisconnected event is emitted
    Then no resources flow through the path

  # ────────────────────────────────────────────────────────
  # AC8: Before any paths exist, minions auto-carry resources
  #      between nearby groups at reduced speed
  # ────────────────────────────────────────────────────────

  Scenario: Minions auto-carry surplus solid resources between nearby groups
    Given a 12x10 grid with minion_carry_basic fixture
    Given no paths or pipes exist
    Given group A (iron miners) has 3 surplus iron_ore in manifold
    Given group B (iron smelter) is at manhattan distance 4 from group A (within range 5)
    Given group B needs iron_ore as input
    When 1 simulation tick runs (Phase 5: Transport)
    Then 0.5 iron_ore is transferred from group A manifold to group B manifold

  Scenario: Minions do not carry when groups are beyond range 5
    Given a 20x10 grid with minion_carry_out_of_range fixture
    Given no paths or pipes exist
    Given group A (iron miner at [2, 5]) has 5 surplus iron_ore
    Given group B (iron smelter at [15, 5]) is at manhattan distance 13
    When 1 simulation tick runs (Phase 5: Transport)
    Then 0 iron_ore is transferred from group A to group B
    Then group A manifold still contains 5 iron_ore

  Scenario: Minions cannot carry liquid resources
    Given a 12x10 grid with minion_no_liquid fixture
    Given no paths or pipes exist
    Given group A (water pump at [2, 5]) has 5 surplus water
    Given group B (tree farm at [5, 5]) is at manhattan distance 3 (within range 5)
    Given group B needs water as input
    When 1 simulation tick runs (Phase 5: Transport)
    Then 0 water is transferred from group A to group B
    Then group A manifold still contains 5 water

  Scenario: Minions only carry surplus resources the source group does not need
    Given a 12x10 grid
    Given group A has iron_ore in manifold
    Given group A contains an iron_smelter that consumes iron_ore
    Given group B is within range 5 and needs iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then no iron_ore is transferred to group B because group A uses it internally

  Scenario: Minion carry rate is 0.5 items per tick (25% of T1 path capacity)
    Given a 12x10 grid with minion_carry_basic fixture
    Given no paths or pipes exist
    Given group A has 10 surplus iron_ore in manifold
    Given group B is within range 5 and needs iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then exactly 0.5 iron_ore is transferred per tick

  # ────────────────────────────────────────────────────────
  # Edge cases from PRD
  # ────────────────────────────────────────────────────────

  Scenario: Path drawn to receiver already at max input rate queues at sender
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 10 iron_ore
    Given group B input receiver is saturated (demand is 0)
    When 1 simulation tick runs (Phase 5: Transport)
    Then 0 iron_ore is launched as Cargo
    Then 10 iron_ore remains in group A manifold

  Scenario: Hazard destroys path segment crossing hazard zone
    Given a 16x10 grid with path_through_hazard fixture (volcanic biome)
    Given a rune_path from group A to group B crossing tiles [[4, 5], [5, 5], [6, 5], [7, 5], [8, 5], [9, 5], [10, 5], [11, 5]]
    Given an eruption hazard zone centered at [7, 5] with radius 2 and next_event_tick 100
    When SimClock reaches tick 100 and hazard fires
    Then path segments within the eruption zone are destroyed
    Then a PathDisconnected event is emitted
    Then resources stop flowing through the path

  Scenario: Disconnected path shows warning on both ends
    Given a 16x10 grid with path_destroyed_mid_segment fixture
    Given a rune_path from group A to group B
    When path segment at [6, 5] is destroyed
    Then a PathDisconnected event is emitted with fromGroup A and toGroup B
    Then group A output sender shows disconnected warning state
    Then group B input receiver shows disconnected warning state

  # ────────────────────────────────────────────────────────
  # Additional edge cases and error paths
  # ────────────────────────────────────────────────────────

  Scenario: Multi-path network delivers resources through chain
    Given a 20x10 grid with multi_path_network fixture
    Given a rune_path from group A (miners) to group B (smelter) via tiles [[5, 5], [6, 5], [7, 5], [8, 5], [9, 5]]
    Given a rune_path from group B (smelter) to group C (constructor) via tiles [[12, 5], [13, 5], [14, 5], [15, 5]]
    Given the transport tier is 1
    Given group A manifold contains 2 iron_ore
    When enough simulation ticks run for cargo to traverse path A-to-B
    Then iron_ore is delivered to group B manifold
    When group B processes iron_ore into iron_bar
    Then iron_bar appears in group B manifold
    When enough simulation ticks run for cargo to traverse path B-to-C
    Then iron_bar is delivered to group C manifold

  Scenario: Path with no source resources produces no cargo
    Given a 16x10 grid with rune_path_basic fixture
    Given the transport tier is 1
    Given group A manifold contains 0 iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then no Cargo entity is created
    Then no resources are moved

  Scenario: Minion carry coexists with paths — path takes priority
    Given a 16x10 grid
    Given group A and group B are within minion carry range 5
    Given a rune_path connects group A output to group B input
    Given group A has surplus iron_ore
    When 1 simulation tick runs (Phase 5: Transport)
    Then resources flow through the rune_path (not via minion carry)

  Scenario: Multiple paths from same group output to different groups
    Given a 20x10 grid
    Given group A has an output sender for iron_ore
    Given group B has an input receiver for iron_ore
    Given group C has an input receiver for iron_ore
    Given a rune_path from group A to group B
    When the player issues DrawPath from group A to group C
    Then a second rune_path entity is created from group A to group C
    Then both paths can carry iron_ore simultaneously

  Scenario: Destroying all path segments removes path entity entirely
    Given a 16x10 grid with rune_path_basic fixture
    Given a rune_path from group A to group B with 6 segments
    When all 6 path segments are destroyed
    Then the path entity is destroyed
    Then the PathConnection entity is destroyed
    Then a PathDisconnected event is emitted

  Scenario: DrawPath rejected when source group has no output sender
    Given a 16x10 grid
    Given group A has no output sender configured
    Given group B has an input receiver
    When the player issues DrawPath from group A to group B
    Then the DrawPath command is rejected

  Scenario: DrawPath rejected when destination group has no input receiver
    Given a 16x10 grid
    Given group A has an output sender
    Given group B has no input receiver configured
    When the player issues DrawPath from group A to group B
    Then the DrawPath command is rejected

  Scenario: DestroyPath command removes an existing path
    Given a 16x10 grid with rune_path_basic fixture
    Given a rune_path entity exists from group A to group B
    When the player issues DestroyPath command for the rune_path entity
    Then the rune_path entity is destroyed
    Then all segment tiles are freed
    Then a PathDisconnected event is emitted

  Scenario: Transport phase runs after production phase
    Given a 16x10 grid with rune_path_basic fixture
    Given group A iron_miner produces iron_ore in Phase 4
    When a full simulation tick runs
    Then Phase 4 (Production) runs before Phase 5 (Transport)
    Then iron_ore produced in Phase 4 is available for transport in Phase 5

  Scenario: Minion carry uses manhattan nearest to find destination
    Given a 12x10 grid
    Given group A has surplus iron_ore at position [2, 5]
    Given group B needs iron_ore at manhattan distance 3 from group A
    Given group C needs iron_ore at manhattan distance 5 from group A
    Given no paths exist
    When 1 simulation tick runs (Phase 5: Transport)
    Then minion carry transfers iron_ore to group B (nearest) first
