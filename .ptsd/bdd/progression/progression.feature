@feature:progression
Feature: Progression — Opus Tree, Milestones, Mini-Opus, Tier Gates, Scoring

  The Opus tree is a unified progression structure for each run.
  Main path nodes are production throughput milestones.
  Side branches are mini-opus challenges awarding meta-currency.
  Tier gates are unlocked by clearing creature nests.
  Scoring combines opus completion, mini-opus, and time bonus.

  # ── AC1: Opus tree nodes are production throughput milestones ──

  Scenario: Opus tree nodes define resource and rate requirements
    Given a forest biome run using the "standard" opus template
    Given the opus tree has 5 main-path nodes generated from template
    Given node 1 requires iron_ore at 4.0 per minute at tier 1
    Given node 2 requires iron_bar at 3.0 per minute at tier 1
    Given node 3 requires steel_plate at 2.0 per minute at tier 2
    Given node 4 requires hide at 1.6 per minute at tier 2
    Given node 5 requires opus_ingot at 1.0 per minute at tier 3
    When the opus tree is initialized
    Then each node has a resource field and a requiredRate field
    Then no node defines an item-crafting goal

  Scenario: Opus tree scales rates by difficulty multiplier
    Given a run with difficulty "hard" and rate multiplier 1.4
    Given template node 1 has rate_base 2.0 for resource_class raw_solid
    When the opus tree is generated
    Then node 1 requiredRate equals 2.8 per minute

  # ── AC2: Milestone sustained after verification period ──

  Scenario: Milestone completes when rate sustained for 600 ticks
    Given an opus node for iron_ore at 4.0 per minute with sustained false
    Given the sustain_window_ticks is 600
    Given the sample_interval_ticks is 20
    Given the production rate of iron_ore is 4.5 per minute for 600 consecutive ticks
    When the MilestoneCheckSystem runs
    Then the opus node sustained field is true
    Then a MilestoneReached event is emitted for the node

  Scenario: Milestone does not complete when rate held for fewer than 600 ticks
    Given an opus node for iron_ore at 4.0 per minute with sustained false
    Given the sustain_window_ticks is 600
    Given the production rate of iron_ore is 4.5 per minute for 500 consecutive ticks
    When the MilestoneCheckSystem runs
    Then the opus node sustained field is false

  Scenario: Milestone does not complete when rate is below required
    Given an opus node for iron_ore at 4.0 per minute with sustained false
    Given the production rate of iron_ore is 3.5 per minute for 600 consecutive ticks
    When the MilestoneCheckSystem runs
    Then the opus node sustained field is false

  # ── AC3: Opus tree UI shows nodes, rates, completion % ──

  Scenario: Opus tree exposes data for UI display
    Given a forest standard run with 5 main-path nodes and 2 mini-opus branches
    Given node 1 is sustained with current rate 4.5 vs required 4.0
    Given node 2 is not sustained with current rate 1.0 vs required 3.0
    When the OpusTree resource is queried
    Then the tree contains 5 main-path node entries
    Then the tree contains 2 mini-opus branch entries
    Then the tree completionPct equals 0.2

  # ── AC4: Mini-opus branches attached to parent main-path node ──

  Scenario: Mini-opus branch references its parent main-path node
    Given a standard opus tree with branch_points at nodes 1, 3, 5
    Given a mini-opus "trade_5_wood" attached to parent node 1
    Given a mini-opus "fast_steel" attached to parent node 3
    When the opus tree is queried
    Then mini-opus "trade_5_wood" has parent_node equal to 1
    Then mini-opus "fast_steel" has parent_node equal to 3

  # ── AC5: Mini-opus awards meta-currency; skipping has no penalty ──

  Scenario: Completing on-demand mini-opus awards gold currency
    Given a mini-opus "trade_5_wood" of type trade_surplus with trigger on_demand
    Given the condition is trade 5 units of wood
    Given the reward is 50 gold
    Given the player trades 5 wood to the wandering trader
    When the MiniOpusSystem runs
    Then the mini-opus "trade_5_wood" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 50 gold

  Scenario: Completing time-based mini-opus before deadline awards knowledge
    Given a mini-opus "fast_steel" of type speed_production with trigger time_based
    Given the deadline is tick 50000
    Given the condition is produce steel_plate at 3.0 per minute
    Given the current tick is 45000
    Given the sustained rate of steel_plate is 3.2 per minute
    When the MiniOpusSystem runs
    Then the mini-opus "fast_steel" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 60 knowledge

  Scenario: Completing conditional mini-opus awards souls
    Given a mini-opus "clear_nest_fast" of type clear_nest_fast with trigger conditional
    Given the condition is clear a nest within 600 ticks of discovery
    Given the reward is 70 souls
    Given a nest was discovered at tick 10000
    Given the nest is cleared at tick 10400
    When the MiniOpusSystem runs
    Then the mini-opus "clear_nest_fast" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 70 souls

  Scenario: Skipping a mini-opus does not affect main-path progression
    Given a mini-opus "trade_5_wood" attached to main-path node 1
    Given node 1 requires iron_ore at 4.0 per minute
    Given the mini-opus "trade_5_wood" is missed
    Given the sustained rate of iron_ore is 4.5 per minute for 600 ticks
    When the MilestoneCheckSystem runs
    Then node 1 sustained field is true

  # ── AC6: Final Opus node requires simultaneous sustain ──

  Scenario: Final node completes when all main-path rates sustained simultaneously
    Given a standard opus tree with 5 main-path nodes
    Given the final node requires simultaneous_sustain for 600 ticks
    Given all 5 nodes have sustained true simultaneously
    Given the simultaneous sustain has lasted 600 ticks
    When the RunLifecycleSystem runs
    Then a RunWon event is emitted

  Scenario: Final node does not complete when one rate drops during sustain window
    Given a standard opus tree with 5 main-path nodes
    Given the final node requires simultaneous_sustain for 600 ticks
    Given nodes 1-4 are sustained but node 5 rate drops below 1.0 per minute at tick 300
    When the RunLifecycleSystem checks at tick 599
    Then no RunWon event is emitted

  # ── AC7: T2 inaccessible until T1 nest cleared ──

  Scenario: T2 buildings cannot be placed before T1 nest is cleared
    Given the current tier is 1
    Given a TierGate for tier 2 linked to nest "forest_wolf_den" with unlocked false
    Given a T2 building type "steel_smelter" in the player inventory
    When the player issues a PlaceBuilding command for "steel_smelter"
    Then the command is rejected
    Then the building is not placed

  Scenario: Clearing T1 nest unlocks T2
    Given the current tier is 1
    Given a TierGate for tier 2 linked to nest "forest_wolf_den" with unlocked false
    Given a NestCleared event for nest "forest_wolf_den"
    When the TierGateSystem runs
    Then the TierGate for tier 2 has unlocked true
    Then the TierState currentTier is 2
    Then a TierUnlocked event is emitted for tier 2

  # ── AC8: T3 inaccessible until T2 nest cleared ──

  Scenario: T3 buildings cannot be placed before T2 nest is cleared
    Given the current tier is 2
    Given a TierGate for tier 3 linked to nest "forest_ancient_treant" with unlocked false
    Given a T3 building type "opus_forge" in the player inventory
    When the player issues a PlaceBuilding command for "opus_forge"
    Then the command is rejected
    Then the building is not placed

  Scenario: Clearing T2 nest unlocks T3
    Given the current tier is 2
    Given a TierGate for tier 3 linked to nest "forest_ancient_treant" with unlocked false
    Given a NestCleared event for nest "forest_ancient_treant"
    When the TierGateSystem runs
    Then the TierGate for tier 3 has unlocked true
    Then the TierState currentTier is 3
    Then a TierUnlocked event is emitted for tier 3

  # ── AC9: Final Opus node triggers run-end with scoring ──

  Scenario: Run ends with scoring when final node is completed
    Given a standard opus tree with 5 main-path nodes all sustained
    Given 1 of 2 mini-opus branches completed
    Given the run elapsed 72000 ticks out of 108000 max ticks
    Given the opus difficulty is medium with multiplier 1.5
    When the RunLifecycleSystem detects RunWon
    Then the raw score equals 0.5 * 1.0 + 0.3 * 0.5 + 0.2 * 0.333 which is 0.717
    Then the final display score equals 717
    Then the currency earned equals base_currency * 0.717 * 1.5

  # ── AC10: All recipes for tier available immediately on unlock ──

  Scenario: T2 recipes become available immediately when T2 is unlocked
    Given the current tier is 1
    Given a T2 recipe "steel_plate_recipe" is locked
    Given a TierUnlocked event for tier 2
    When the tier transition is processed
    Then the recipe "steel_plate_recipe" is available for use
    Then all T2 recipes in RecipeDB are available

  # ── AC11: Existing buildings auto-upgrade on tier unlock ──

  Scenario: Existing T1 buildings auto-upgrade when T2 unlocks
    Given a placed iron_smelter building at tier 1
    Given a TierUnlocked event for tier 2
    When the tier transition is processed
    Then the iron_smelter building tier is 2
    Then the building retains its position and group membership

  Scenario: Auto-upgrade applies to all existing buildings of lower tier
    Given 3 placed buildings at tier 1 and 2 placed buildings at tier 2
    Given a TierUnlocked event for tier 3
    When the tier transition is processed
    Then all 5 buildings have tier 3

  # ── Edge Case: Milestone no-regression ──

  Scenario: Rate drop after sustain does not revoke milestone
    Given an opus node for iron_ore at 4.0 per minute with sustained true
    Given the production rate of iron_ore drops to 2.0 per minute
    When the MilestoneCheckSystem runs
    Then the opus node sustained field is still true

  # ── Edge Case: Time-based mini-opus missed ──

  Scenario: Time-based mini-opus marked missed after deadline
    Given a mini-opus "fast_steel" of type speed_production with trigger time_based
    Given the deadline is tick 50000
    Given the current tick is 51000
    Given the condition is not met
    When the MiniOpusSystem runs
    Then the mini-opus "fast_steel" is marked missed
    Then a MiniOpusMissed event is emitted
    Then main-path node 3 is unaffected

  # ── Edge Case: Opus requires non-biome resource ──

  Scenario: Opus node requires resource unavailable in biome
    Given a volcanic biome run
    Given opus node 1 requires wood at 2.0 per minute at tier 1
    Given volcanic biome has no natural wood resource veins
    When the player builds a tree_farm synthesis group
    When the tree_farm produces wood at 2.0 per minute for 600 ticks
    Then the opus node sustained field is true

  # ── Edge Case: Run timeout with partial completion ──

  Scenario: Run timeout awards partial score based on tree fill
    Given a standard opus tree with 5 main-path nodes
    Given nodes 1, 2, 3 are sustained and nodes 4, 5 are not sustained
    Given 0 of 2 mini-opus branches completed
    Given the current tick reaches 108000 (max_ticks)
    When the RunLifecycleSystem runs
    Then a RunTimeUp event is emitted
    Then the opus_completion equals 0.6
    Then the mini_opus_score equals 0.0
    Then the time_bonus equals 0.0
    Then the raw score equals 0.5 * 0.6 + 0.3 * 0.0 + 0.2 * 0.0 which is 0.3
    Then the final display score equals 300

  # ── Edge Case: All mini-opus missed ──

  Scenario: Run completable with all mini-opus missed
    Given a standard opus tree with 5 main-path nodes all sustained
    Given 0 of 2 mini-opus branches completed and both missed
    Given the run elapsed 54000 ticks out of 108000 max ticks
    When the RunLifecycleSystem detects RunWon
    Then the opus_completion equals 1.0
    Then the mini_opus_score equals 0.0
    Then the time_bonus equals 0.5
    Then the raw score equals 0.5 * 1.0 + 0.3 * 0.0 + 0.2 * 0.5 which is 0.6

  # ── Edge Case: Run abandoned ──

  Scenario: Abandoned run earns zero meta-currency
    Given a run in progress with some milestones sustained
    Given the player issues an abandon command
    When the run ends with abandon status
    Then the abandon_currency_multiplier is 0.0
    Then no meta-currency is awarded

  # ── Starting Kit ──

  Scenario: Forest starting kit provides correct buildings
    Given a forest biome run with no meta unlocks
    When the starting kit is applied
    Then the player inventory contains 2 iron_miner
    Then the player inventory contains 1 water_pump
    Then the player inventory contains 1 iron_smelter
    Then the player inventory contains 1 sawmill
    Then the player inventory contains 1 tree_farm
    Then the player inventory contains 1 constructor
    Then the player inventory contains 2 wind_turbine
    Then the player inventory contains 1 watchtower

  Scenario: Volcanic starting kit has no wood or water buildings
    Given a volcanic biome run with no meta unlocks
    When the starting kit is applied
    Then the player inventory contains 2 iron_miner
    Then the player inventory contains 2 stone_quarry
    Then the player inventory contains 1 iron_smelter
    Then the player inventory contains 1 constructor
    Then the player inventory contains 3 wind_turbine
    Then the player inventory contains 1 watchtower
    Then the player inventory does not contain water_pump
    Then the player inventory does not contain sawmill

  Scenario: Starting kit enhanced by meta unlocks
    Given a forest biome run with meta unlocks [extra_starting_miner, extra_starting_turbine]
    When the starting kit is applied
    Then the player inventory contains 3 iron_miner
    Then the player inventory contains 3 wind_turbine

  # ── Mini-Opus Branch Generation ──

  Scenario: Branch points receive 1-2 mini-opus branches each
    Given a standard opus template with branch_points at nodes 1, 3, 5
    When the opus tree is generated for a forest biome
    Then node 1 has between 1 and 2 mini-opus branches
    Then node 3 has between 1 and 2 mini-opus branches
    Then node 5 has between 1 and 2 mini-opus branches

  Scenario: Non-branch-point nodes receive no mini-opus branches
    Given a standard opus template with branch_points at nodes 1, 3, 5
    When the opus tree is generated
    Then node 2 has 0 mini-opus branches
    Then node 4 has 0 mini-opus branches

  # ── Opus Difficulty Multiplier ──

  Scenario: Easy difficulty applies 1.0 opus multiplier to currency
    Given a completed run with opus difficulty easy
    Given a mini-opus with base reward 50 gold
    When meta-currency is calculated
    Then the gold earned from that mini-opus equals 50

  Scenario: Hard difficulty applies 2.0 opus multiplier to currency
    Given a completed run with opus difficulty hard
    Given a mini-opus with base reward 50 gold
    When meta-currency is calculated
    Then the gold earned from that mini-opus equals 100

  Scenario: Extreme difficulty applies 3.0 opus multiplier to currency
    Given a completed run with opus difficulty extreme
    Given a mini-opus with base reward 50 gold
    When meta-currency is calculated
    Then the gold earned from that mini-opus equals 150

  # ── Tier Gate: Transport Auto-Upgrade ──

  Scenario: Transport tier upgrades globally on tier unlock
    Given the TierState transportTier is 1
    Given a TierUnlocked event for tier 2
    When the TierGateSystem runs
    Then the TierState transportTier is 2

  # ── Conditional Mini-Opus: zero_waste ──

  Scenario: Zero waste mini-opus completes when no idle resources for 300 ticks
    Given a mini-opus "zero_waste" of type zero_waste with trigger conditional
    Given the condition is no resources idle in manifold for 300 consecutive ticks
    Given the reward is 55 knowledge
    Given no resources have been idle for 300 ticks
    When the MiniOpusSystem runs
    Then the mini-opus "zero_waste" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 55 knowledge

  # ── Conditional Mini-Opus: organic_surplus ──

  Scenario: Organic surplus mini-opus completes when threshold reached
    Given a mini-opus "organic_surplus" of type organic_surplus with trigger conditional
    Given the condition is produce 20 organic resources in a single combat group
    Given the reward is 45 souls
    Given a combat group has produced 20 organic resources
    When the MiniOpusSystem runs
    Then the mini-opus "organic_surplus" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 45 souls

  # ── Time-Based Mini-Opus: survive_hazard_producing ──

  Scenario: Survive hazard producing mini-opus requires maintaining rate during hazard
    Given a mini-opus "survive_hazard" of type survive_hazard_producing with trigger time_based
    Given the hazard duration is 400 ticks
    Given the condition is maintain 80% of current production rate
    Given the reward is 80 souls
    Given the current production rate is 5.0 per minute
    Given the rate stays at 4.2 per minute during the hazard (above 80% of 5.0)
    When the MiniOpusSystem runs after the hazard ends
    Then the mini-opus "survive_hazard" is marked completed
    Then a MiniOpusCompleted event is emitted with reward 80 souls

  # ── Run Lifecycle: Tier Timing ──

  Scenario: Tier timing targets are advisory only
    Given the tier_timing t1_end target is tick 30000
    Given the current tick is 35000
    Given the current tier is still 1
    When the simulation runs
    Then no penalty is applied for exceeding the tier timing target
    Then the player can still clear the T1 nest to unlock T2
