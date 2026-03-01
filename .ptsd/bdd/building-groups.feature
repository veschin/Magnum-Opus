@feature:building-groups
Feature: Building Groups
  Adjacent buildings automatically form a group with a shared manifold.
  Inside a group, resources distribute automatically.
  Groups are the unit of management: energy, priority, pause/resume.

  Background:
    Given the simulation runs at 20 ticks per second

  # ═══════════════════════════════════════════════════════════
  # AC1: Placing a building adjacent to an existing building
  #      merges them into a group with shared manifold
  # ═══════════════════════════════════════════════════════════

  Scenario: Single building forms a group of one
    Given a 10x10 grid with iron_vein terrain at position (5, 5)
    When the player places an iron_miner at position (5, 5)
    Then there is exactly 1 group
    Then the iron_miner at (5, 5) belongs to that group
    Then the group has an empty manifold

  Scenario: Two adjacent buildings merge into one group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (4, 3)
    When the player places an iron_miner at position (3, 3)
    When the player places an iron_miner at position (4, 3)
    Then there is exactly 1 group
    Then the group contains 2 buildings

  Scenario: Two non-adjacent buildings form separate groups
    Given a 12x10 grid with iron_vein terrain at position (2, 3) and copper_vein terrain at position (8, 3)
    When the player places an iron_miner at position (2, 3)
    When the player places a copper_miner at position (8, 3)
    Then there are exactly 2 groups
    Then the iron_miner at (2, 3) and the copper_miner at (8, 3) belong to different groups

  Scenario: Adjacent buildings share a single manifold
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    When the player places an iron_miner at position (3, 3)
    When the player places an iron_smelter at position (4, 3)
    Then both buildings belong to the same group
    Then the group has exactly 1 shared manifold

  Scenario: Diagonal buildings do not form a group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (4, 4)
    When the player places an iron_miner at position (3, 3)
    When the player places an iron_miner at position (4, 4)
    Then there are exactly 2 groups
    Then each building belongs to a separate group

  # ═══════════════════════════════════════════════════════════
  # AC2: Resources produced by any building in a group
  #      are available to all other buildings that need them
  # ═══════════════════════════════════════════════════════════

  Scenario: Miner output is available to smelter via manifold
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) and an iron_smelter at (4, 3) in the same group
    Given a wind_turbine at (4, 4) providing 20 energy
    When the simulation runs for 60 ticks
    Then the group manifold contains iron_ore
    Then the iron_smelter begins consuming iron_ore from the manifold

  Scenario: Smelter consumes miner output within same group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (3, 4)
    Given 2 iron_miners at (3, 3) and (3, 4) and an iron_smelter at (4, 3) in the same group
    Given a wind_turbine at (4, 4) providing 20 energy
    When the simulation runs for 240 ticks
    Then the group manifold contains iron_bar produced by the smelter
    Then the smelter consumed 2 iron_ore per cycle from the manifold

  Scenario: Multiple consumers share manifold resources proportionally
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (3, 4), and (3, 5)
    Given 3 iron_miners at (3, 3), (3, 4), (3, 5) and 1 iron_smelter at (4, 3) in the same group
    Given a wind_turbine at (4, 4) providing 20 energy
    When the simulation runs for 240 ticks
    Then iron_ore accumulates in the manifold because miners produce faster than the smelter consumes

  Scenario: Mall building output goes to Inventory not manifold
    Given a 10x10 grid
    Given a constructor at (3, 3) in a group with resources iron_bar: 3 and plank: 1 in the manifold
    Given a wind_turbine at (5, 4) providing 20 energy
    When the simulation runs for 300 ticks
    Then the Inventory contains 1 iron_miner
    Then the group manifold does not contain iron_miner

  # ═══════════════════════════════════════════════════════════
  # AC3: Identical buildings placed adjacent chain without
  #      requiring manual connection
  # ═══════════════════════════════════════════════════════════

  Scenario: Two identical miners chain automatically
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (4, 3)
    When the player places an iron_miner at position (3, 3)
    When the player places an iron_miner at position (4, 3)
    Then both miners belong to the same group
    Then no manual connection step was required

  Scenario: Four identical miners in a square form one group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (4, 3), (3, 4), and (4, 4)
    When the player places iron_miners at (3, 3), (4, 3), (3, 4), and (4, 4)
    Then there is exactly 1 group
    Then the group contains 4 buildings

  Scenario: L-shaped group of identical and different buildings
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (4, 3), and (3, 4)
    When the player places an iron_miner at (3, 3)
    When the player places an iron_miner at (4, 3)
    When the player places an iron_miner at (3, 4)
    When the player places an iron_smelter at (3, 5)
    Then there is exactly 1 group
    Then the group contains 4 buildings

  # ═══════════════════════════════════════════════════════════
  # AC4: Group displays aggregate input/output rates
  # ═══════════════════════════════════════════════════════════

  Scenario: Group stats show aggregate production rate
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (3, 4)
    Given 2 iron_miners at (3, 3) and (3, 4) and an iron_smelter at (4, 3) in the same group
    Given a wind_turbine at (4, 4) providing 20 energy
    When the simulation runs for 120 ticks
    Then the group stats show iron_ore output rate as 2 units per 60 ticks
    Then the group stats show iron_ore input rate as 2 units per 120 ticks

  Scenario: Group stats for single miner
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) in a group
    Given a wind_turbine at (4, 3) providing 20 energy
    When the simulation runs for 60 ticks
    Then the group stats show iron_ore output rate as 1 unit per 60 ticks

  # ═══════════════════════════════════════════════════════════
  # AC5: Player can place input receivers and output senders
  #      on group boundary
  # ═══════════════════════════════════════════════════════════

  Scenario: Group of one has configurable receivers and senders
    Given a 10x10 grid with iron_vein terrain at position (5, 5)
    Given an iron_miner at (5, 5) forming a group of 1
    Then the group has at least one output sender on the boundary
    Then the group allows placing an input receiver on the boundary

  Scenario: Multi-building group has boundary ports
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) and an iron_smelter at (4, 3) in the same group
    Then the group boundary includes cells (2, 3), (3, 2), (3, 4), (5, 3), (4, 2), (4, 4)
    Then the player can place an output sender for iron_bar on the boundary
    Then the player can place an input receiver for iron_ore on the boundary

  Scenario: Output sender feeds transport path
    Given a group A with an iron_miner producing iron_ore and an output sender on the boundary
    Given a group B with an iron_smelter and an input receiver on the boundary
    Given a rune path connecting group A output to group B input
    When the simulation runs for 120 ticks
    Then iron_ore flows from group A manifold to group B manifold via the path

  # ═══════════════════════════════════════════════════════════
  # AC6: Removing a building that bridges two sub-groups
  #      splits them into separate groups
  # ═══════════════════════════════════════════════════════════

  Scenario: Removing bridge building splits group into two
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (5, 3)
    Given an iron_miner at (3, 3), an iron_smelter at (4, 3), and an iron_miner at (5, 3) in one group
    When the player removes the iron_smelter at (4, 3)
    Then there are exactly 2 groups
    Then the iron_miner at (3, 3) belongs to one group
    Then the iron_miner at (5, 3) belongs to a different group

  Scenario: Split groups get separate manifolds
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (5, 3)
    Given an iron_miner at (3, 3), an iron_smelter at (4, 3), and an iron_miner at (5, 3) in one group
    Given the group manifold contains iron_ore: 10
    When the player removes the iron_smelter at (4, 3)
    Then each new group has its own separate manifold

  Scenario: Removing non-bridge building does not split group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (4, 3), and (3, 4)
    Given iron_miners at (3, 3), (4, 3), and (3, 4) forming an L-shaped group
    When the player removes the iron_miner at (4, 3)
    Then there is exactly 1 group
    Then the group contains 2 buildings

  Scenario: Removing last building destroys the group
    Given a 10x10 grid with iron_vein terrain at position (5, 5)
    Given an iron_miner at (5, 5) as the only building in a group
    When the player removes the iron_miner at (5, 5)
    Then there are exactly 0 groups
    Then all external paths connected to the former group are disconnected

  # ═══════════════════════════════════════════════════════════
  # AC7: Chain manager displays groups as units with energy,
  #      priority, and status controls
  # ═══════════════════════════════════════════════════════════

  Scenario: Group has energy demand and allocation
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) with energy_consumption 5 in a group
    Given a wind_turbine at (4, 3) providing 20 energy to the pool
    When the simulation runs for 1 tick
    Then the group energy demand is 5
    Then the group energy allocated is greater than 0

  Scenario: Group priority can be set via command
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) in a group with default priority MEDIUM
    When the player sends a SetGroupPriority command to set priority to HIGH
    Then the group priority is HIGH

  Scenario: Group can be paused and resumed
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) in a group producing iron_ore
    Given a wind_turbine at (4, 3) providing 20 energy
    When the player sends a PauseGroup command for the group
    When the simulation runs for 60 ticks
    Then the group production is paused
    Then no iron_ore is produced during the paused period
    When the player sends a ResumeGroup command for the group
    When the simulation runs for 60 ticks
    Then the group resumes production

  Scenario: Chain manager shows each group as a manageable unit
    Given a 10x10 grid with two separate groups: group A (iron_miner) and group B (copper_miner)
    Then the chain manager lists 2 groups
    Then each group entry shows energy allocation, priority, and status

  # ═══════════════════════════════════════════════════════════
  # AC8: Synthesis groups function without terrain requirements
  #      (placeable on any valid tile)
  # ═══════════════════════════════════════════════════════════

  Scenario: Synthesis building placed on plain terrain
    Given a 10x10 grid with grass terrain at position (3, 3)
    When the player places an iron_smelter at position (3, 3)
    Then the placement succeeds
    Then the iron_smelter exists at (3, 3) in a group

  Scenario: Tree farm placed on any tile produces wood from water
    Given a 10x10 grid with grass terrain at position (3, 3)
    Given a tree_farm at (3, 3) in a group with water: 3 in the manifold
    Given a wind_turbine at (5, 3) providing 20 energy
    When the simulation runs for 180 ticks
    Then the group manifold contains wood: 2

  Scenario: Synthesis group idles when inputs unavailable
    Given a 10x10 grid with grass terrain at position (3, 3)
    Given an iron_smelter at (3, 3) in a group with an empty manifold
    Given a wind_turbine at (3, 4) providing 20 energy
    When the simulation runs for 240 ticks
    Then the iron_smelter production state is idle
    Then no iron_bar is produced
    Then the simulation does not crash

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: Building placed between two existing groups
  #            merges all into one
  # ═══════════════════════════════════════════════════════════

  Scenario: Building placed between two groups merges them
    Given a 10x10 grid with iron_vein terrain at position (3, 3) and copper_vein at position (5, 3)
    Given an iron_miner at (3, 3) in group A
    Given a copper_miner at (5, 3) in group B
    When the player places an iron_smelter at position (4, 3)
    Then there is exactly 1 group
    Then the group contains 3 buildings: iron_miner, iron_smelter, and copper_miner

  Scenario: Three-way merge preserves all manifold contents
    Given a 10x10 grid with iron_vein terrain at position (3, 3) and copper_vein at position (5, 3)
    Given group A with iron_ore: 5 in the manifold
    Given group B with copper_ore: 3 in the manifold
    When the player places an iron_smelter at position (4, 3) between them
    Then the merged group manifold contains iron_ore: 5 and copper_ore: 3

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: Group with no input receivers still functions
  #            if it contains self-sufficient buildings (miners)
  # ═══════════════════════════════════════════════════════════

  Scenario: Extraction group produces without input receivers
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) in a group with no input receivers configured
    Given a wind_turbine at (4, 3) providing 20 energy
    When the simulation runs for 60 ticks
    Then the group manifold contains iron_ore: 1
    Then the mine_iron recipe completed one cycle with no external inputs

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: No energy buildings — zero production speed
  # ═══════════════════════════════════════════════════════════

  Scenario: Group with no energy produces at zero speed
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner at (3, 3) and an iron_smelter at (4, 3) in a group
    Given no energy buildings exist
    When the simulation runs for 120 ticks
    Then the group energy allocated is 0
    Then no iron_ore is produced

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement on invalid terrain
  # ═══════════════════════════════════════════════════════════

  Scenario: Miner placement rejected on wrong terrain
    Given a 10x10 grid with grass terrain at position (3, 3)
    When the player places an iron_miner at position (3, 3)
    Then the placement is rejected
    Then no building exists at (3, 3)
    Then no group is created

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement on occupied tile
  # ═══════════════════════════════════════════════════════════

  Scenario: Placement rejected on already occupied tile
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given an iron_miner already placed at (3, 3)
    When the player places an iron_smelter at position (3, 3)
    Then the placement is rejected
    Then the iron_miner at (3, 3) is unchanged
    Then the group still contains only the iron_miner

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement out of grid bounds
  # ═══════════════════════════════════════════════════════════

  Scenario: Placement rejected outside grid bounds
    Given a 5x5 grid
    When the player places an iron_miner at position (10, 10)
    Then the placement is rejected
    Then no building exists at (10, 10)

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement of tier-locked building
  # ═══════════════════════════════════════════════════════════

  Scenario: T2 building rejected while at T1
    Given a 10x10 grid with obsidian_vein terrain at position (3, 3)
    Given the current tier is 1
    When the player places an obsidian_drill at position (3, 3)
    Then the placement is rejected because obsidian_drill requires tier 2
    Then no building exists at (3, 3)

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Footprint overlap
  # ═══════════════════════════════════════════════════════════

  Scenario: 2x2 building placement rejected when footprint overlaps existing building
    Given a 10x10 grid
    Given a constructor at (3, 3) occupying cells (3, 3), (4, 3), (3, 4), (4, 4)
    When the player places an iron_smelter at position (4, 3)
    Then the placement is rejected because cell (4, 3) is occupied by the constructor

  Scenario: Two 2x2 buildings cannot overlap footprints
    Given a 10x10 grid
    Given a constructor at (3, 3) occupying cells (3, 3), (4, 3), (3, 4), (4, 4)
    When the player places an imp_camp at position (4, 4)
    Then the placement is rejected because cell (4, 4) is occupied by the constructor

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement from empty inventory
  # ═══════════════════════════════════════════════════════════

  Scenario: Placement rejected when building not in inventory
    Given a 10x10 grid with iron_vein terrain at position (3, 3)
    Given the player inventory contains 0 iron_miners
    When the player places an iron_miner at position (3, 3)
    Then the placement is rejected because iron_miner is not available in inventory

  # ═══════════════════════════════════════════════════════════
  # ERROR PATH: Placement on hidden (fogged) tile
  # ═══════════════════════════════════════════════════════════

  Scenario: Placement rejected on hidden tile
    Given a 10x10 grid with iron_vein terrain at position (7, 7)
    Given position (7, 7) is hidden by fog of war
    When the player places an iron_miner at position (7, 7)
    Then the placement is rejected because tile (7, 7) is not visible

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: Manifold overflow — accumulation
  # ═══════════════════════════════════════════════════════════

  Scenario: Manifold accumulates when production exceeds consumption
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (3, 4), and (3, 5)
    Given 3 iron_miners at (3, 3), (3, 4), (3, 5) and 1 iron_smelter at (4, 3) in the same group
    Given a wind_turbine at (4, 4) providing 20 energy
    When the simulation runs for 240 ticks
    Then the manifold iron_ore amount is greater than 0
    Then the smelter consumed some iron_ore but surplus remains

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: 2x2 building adjacency
  # ═══════════════════════════════════════════════════════════

  Scenario: 1x1 building adjacent to 2x2 building forms a group
    Given a 10x10 grid
    Given a constructor at (3, 3) occupying cells (3, 3), (4, 3), (3, 4), (4, 4)
    When the player places a sawmill at position (5, 3)
    Then both buildings belong to the same group
    Then the sawmill is adjacent to the constructor via cell (4, 3)

  # ═══════════════════════════════════════════════════════════
  # EDGE CASE: Group type determination
  # ═══════════════════════════════════════════════════════════

  Scenario: Extraction group type assigned when group contains only miners
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (4, 3)
    When the player places iron_miners at (3, 3) and (4, 3)
    Then the group type is extraction

  Scenario: Combat group type assigned for imp camp and breeding pen
    Given a 10x10 grid
    When the player places an imp_camp at (3, 3) and a breeding_pen at (5, 3)
    Then the group type is combat

  # ═══════════════════════════════════════════════════════════
  # INVARIANT: Single group membership
  # ═══════════════════════════════════════════════════════════

  Scenario: Every building belongs to exactly one group
    Given a 10x10 grid with iron_vein terrain at positions (3, 3) and (4, 3)
    Given an iron_miner at (3, 3) and an iron_smelter at (4, 3) in the same group
    Then the iron_miner belongs to exactly 1 group
    Then the iron_smelter belongs to exactly 1 group

  # ═══════════════════════════════════════════════════════════
  # INVARIANT: Group connectivity
  # ═══════════════════════════════════════════════════════════

  Scenario: All buildings in a group are reachable via cardinal adjacency
    Given a 10x10 grid with iron_vein terrain at positions (3, 3), (4, 3), and (3, 4)
    Given 3 buildings forming an L-shaped group
    Then every building in the group is reachable from every other building via cardinal adjacency
    Then no building in the group is disconnected
