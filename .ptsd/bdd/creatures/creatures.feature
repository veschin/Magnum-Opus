@feature:creatures
Feature: Creatures & Combat
  Living ecosystem of biome-native creatures. Combat groups are a core
  resource pipeline (organics), not optional defense. Creatures have
  5 behavior archetypes. Organic resources come exclusively from
  combat/breeding groups.

  # ═══════════════════════════════════════════════════════════════════
  # AC1: Each biome spawns creatures with at least 3 of the 5 archetypes
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Forest biome spawns at least 3 creature archetypes
    Given a forest biome with max_creatures 30 and spawn_rate_base 0.01
    Given forest_deer is an ambient archetype creature with health 30
    Given forest_wolf is a territorial archetype creature with health 60
    Given forest_vine_creeper is an invasive archetype creature with health 40
    When the creature spawn system runs for 200 ticks
    Then at least 3 distinct archetypes have spawned in the forest biome

  Scenario: Volcanic biome spawns at least 3 creature archetypes
    Given a volcanic biome with max_creatures 20 and spawn_rate_base 0.008
    Given lava_salamander is a territorial archetype creature with health 80
    Given ash_swarm is an invasive archetype creature with health 25
    Given ember_wyrm is an event_born archetype creature with health 150
    When the creature spawn system runs for 200 ticks
    Then at least 3 distinct archetypes have spawned in the volcanic biome

  Scenario: Desert biome spawns at least 3 creature archetypes
    Given a desert biome with max_creatures 15 and spawn_rate_base 0.006
    Given sand_beetle is an ambient archetype creature with health 20
    Given dune_scorpion is a territorial archetype creature with health 90
    Given crystal_golem is an opus_linked archetype creature with health 300
    When the creature spawn system runs for 200 ticks
    Then at least 3 distinct archetypes have spawned in the desert biome

  Scenario: Ocean biome spawns at least 3 creature archetypes
    Given a ocean biome with max_creatures 25 and spawn_rate_base 0.01
    Given tide_crab is an ambient archetype creature with health 25
    Given reef_serpent is an invasive archetype creature with health 50
    Given storm_leviathan is an event_born archetype creature with health 400
    When the creature spawn system runs for 200 ticks
    Then at least 3 distinct archetypes have spawned in the ocean biome

  Scenario: Creature population does not exceed biome capacity
    Given a forest biome with max_creatures 30 and spawn_rate_base 0.01
    When the creature spawn system runs for 10000 ticks
    Then the total creature count in the forest biome is at most 30

  # ═══════════════════════════════════════════════════════════════════
  # AC2: Territorial creatures attack when player builds in territory
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Territorial wolf attacks building placed inside its territory
    Given a 16x16 forest biome grid
    Given a forest_wolf at position [8, 8] with territory_center [8, 8] and territory_radius 6
    Given an iron_miner placed at position [5, 5] within the wolf territory
    When the creature behavior system runs for 1 tick
    Then the forest_wolf state is AGGRESSIVE
    Then the forest_wolf moves toward the nearest player building

  Scenario: Territorial wolf does not attack building outside its territory
    Given a 16x16 forest biome grid
    Given a forest_wolf at position [8, 8] with territory_center [8, 8] and territory_radius 6
    Given an iron_miner placed at position [1, 1] outside the wolf territory
    When the creature behavior system runs for 1 tick
    Then the forest_wolf state is not AGGRESSIVE
    Then the forest_wolf patrols its territory border

  Scenario: Territorial creature damages output senders first on attack
    Given a 16x16 forest biome grid
    Given a forest_wolf at position [8, 8] with territory_center [8, 8] and territory_radius 6
    Given a building group at position [5, 5] with output senders within the wolf territory
    Given the forest_wolf has attack_dps 5 and attack_target output_senders
    When the forest_wolf reaches the building group
    Then the forest_wolf deals 5 damage per tick to the output senders
    Then the output senders take damage before any other building in the group

  Scenario: Lava salamander attacks with higher DPS in volcanic biome
    Given a 16x16 volcanic biome grid
    Given a lava_salamander at position [5, 5] with territory_center [5, 5] and territory_radius 5
    Given an iron_miner placed at position [3, 3] within the salamander territory
    Given the lava_salamander has attack_dps 8 and attack_target output_senders
    When the lava_salamander reaches the building group
    Then the lava_salamander deals 8 damage per tick to the output senders

  # ═══════════════════════════════════════════════════════════════════
  # AC3: Invasive creatures expand territory over time if unchecked
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Vine creeper territory expands when no combat group opposes it
    Given a 20x20 forest biome grid
    Given a forest_vine_creeper at position [10, 10] with territory_center [10, 10] and territory_radius 4
    Given the forest_vine_creeper has expansion_rate 0.02 per tick
    Given no combat groups exist on the grid
    When the creature behavior system runs for 100 ticks
    Then the forest_vine_creeper territory_radius is greater than 4

  Scenario: Vine creeper spawns children when territory reaches threshold
    Given a 20x20 forest biome grid
    Given a forest_vine_creeper at position [10, 10] with territory_center [10, 10] and territory_radius 7
    Given the forest_vine_creeper has spawn_children_at_radius 8 and child_spawn_rate 0.005
    When the forest_vine_creeper territory_radius reaches 8
    Then a new forest_vine_creeper child may spawn within the parent territory

  Scenario: Ash swarm expands faster than vine creeper
    Given a 20x20 volcanic biome grid
    Given an ash_swarm at position [10, 10] with territory_center [10, 10] and territory_radius 3
    Given the ash_swarm has expansion_rate 0.03 per tick
    Given no combat groups exist on the grid
    When the creature behavior system runs for 50 ticks
    Then the ash_swarm territory_radius is greater than 3

  Scenario: Combat group protection suppresses invasive expansion
    Given a 20x20 forest biome grid
    Given a forest_vine_creeper at position [12, 10] with territory_center [12, 10] and territory_radius 4
    Given an imp_camp at position [8, 10] with full supply and protection_radius 6
    When the creature behavior system runs for 100 ticks
    Then the forest_vine_creeper territory_radius has not increased

  # ═══════════════════════════════════════════════════════════════════
  # AC4: Combat group consumes inputs and produces organic output + protection
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Fully supplied imp camp produces organics and protection
    Given a 16x16 forest biome grid
    Given an imp_camp at position [5, 5] with base_organic_rate 1.0
    Given the imp_camp has base_protection_radius 6 and protection_dps 3.0
    Given the imp_camp group manifold contains iron_bar: 10 and herbs: 20
    When the combat group system runs for 1 production cycle
    Then the imp_camp produces 1.0 organic items per cycle
    Then the imp_camp provides protection in a radius of 6 tiles
    Then the imp_camp deals 3.0 damage per tick to creatures in protection radius

  Scenario: Breeding pen produces organics from food and water without protection
    Given a 16x16 forest biome grid
    Given a breeding_pen at position [5, 5] with base_organic_rate 0.6
    Given the breeding_pen has base_protection_radius 0
    Given the breeding_pen group manifold contains herbs: 20
    When the combat group system runs for 1 production cycle
    Then the breeding_pen produces 0.6 organic items per cycle
    Then the breeding_pen provides no protection radius

  Scenario: War lodge produces more organics and protection than imp camp
    Given a 16x16 forest biome grid
    Given a war_lodge at position [5, 5] with base_organic_rate 1.5
    Given the war_lodge has base_protection_radius 9 and protection_dps 6.0
    Given the war_lodge group manifold is fully supplied
    When the combat group system runs for 1 production cycle
    Then the war_lodge produces 1.5 organic items per cycle
    Then the war_lodge provides protection in a radius of 9 tiles
    Then the war_lodge deals 6.0 damage per tick to creatures in protection radius

  # ═══════════════════════════════════════════════════════════════════
  # AC5: Under-supplied combat group loses effectiveness
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Half-supplied imp camp produces half output and half protection
    Given a 16x16 forest biome grid
    Given an imp_camp at position [5, 5] with base_organic_rate 1.0
    Given the imp_camp has base_protection_radius 6 and protection_dps 3.0
    Given the imp_camp supply_ratio is 0.5
    When the combat group system runs for 1 production cycle
    Then the imp_camp produces 0.5 organic items per cycle
    Then the imp_camp provides protection in a radius of 3 tiles
    Then the imp_camp deals 1.5 damage per tick to creatures in protection radius

  Scenario: Imp camp below breach threshold allows enemies through
    Given a 16x16 forest biome grid
    Given an imp_camp at position [5, 5] with breach_threshold 0.3
    Given the imp_camp supply_ratio is 0.2
    Given a forest_wolf at position [8, 8] with territory_center [8, 8] and territory_radius 6
    When the combat group system runs for 1 tick
    Then a TerritoryBreach event is emitted for the imp_camp group
    Then enemies damage the output senders at 2.0 damage per tick

  Scenario: War lodge with lower breach threshold holds longer under deficit
    Given a 16x16 forest biome grid
    Given a war_lodge at position [5, 5] with breach_threshold 0.25
    Given the war_lodge supply_ratio is 0.27
    When the combat group system runs for 1 tick
    Then no TerritoryBreach event is emitted for the war_lodge group

  Scenario: Visible minion count reflects supply ratio
    Given an imp_camp with max_minions 4
    Given the imp_camp supply_ratio is 0.5
    When the minion display is calculated
    Then the visible minion count is 2

  Scenario: Visible minion count at zero supply is zero
    Given an imp_camp with max_minions 4
    Given the imp_camp supply_ratio is 0.0
    When the minion display is calculated
    Then the visible minion count is 0

  Scenario: War lodge visible minion count at full supply
    Given a war_lodge with max_minions 6
    Given the war_lodge supply_ratio is 1.0
    When the minion display is calculated
    Then the visible minion count is 6

  # ═══════════════════════════════════════════════════════════════════
  # AC6: T3 combat group clears enemy zone and drops rare resources
  # ═══════════════════════════════════════════════════════════════════

  Scenario: T3 combat group clears a creature zone
    Given a 20x20 forest biome grid
    Given a war_lodge at position [5, 5] with full supply at tier 3
    Given a creature zone with 3 forest_wolf creatures at position [8, 8]
    When the combat group applies sustained combat pressure over multiple ticks
    Then all creatures in the zone are killed
    Then each forest_wolf drops hide: 3 and herbs: 1 into the nearest combat group manifold

  Scenario: Crystal golem drops rare mana_crystal on death
    Given a 20x20 desert biome grid
    Given a war_lodge at position [5, 5] with full supply at tier 3
    Given a crystal_golem at position [8, 8] with health 300
    When the war_lodge combat pressure kills the crystal_golem
    Then the crystal_golem drops mana_crystal: 5 and sinew: 3

  # ═══════════════════════════════════════════════════════════════════
  # AC7: Organic resources are ONLY obtainable through combat/breeding
  # ═══════════════════════════════════════════════════════════════════

  Scenario: No terrain vein produces organic resources
    Given a forest biome with all terrain types generated
    When querying all ResourceVein entities in the biome
    Then no ResourceVein has resource type hide
    Then no ResourceVein has resource type herbs
    Then no ResourceVein has resource type bone_meal
    Then no ResourceVein has resource type sinew
    Then no ResourceVein has resource type venom

  Scenario: Tannery without combat group cannot get hide input
    Given a 16x16 forest biome grid
    Given an iron_miner at position [5, 5] on an iron_vein
    Given a tannery at position [6, 5] requiring hide as input
    Given no combat group or breeding pen exists on the grid
    When the production system runs for 100 ticks
    Then the tannery never starts production due to missing hide input
    Then the tannery output is 0

  # ═══════════════════════════════════════════════════════════════════
  # AC8: Idle minions with no tasks auto-decorate nearby buildings
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Idle minions decorate buildings when no tasks available
    Given a building group with 3 buildings
    Given 2 minions have no assigned tasks
    When the minion behavior system runs for 1 tick
    Then the 2 idle minions begin decorating nearby buildings

  # ═══════════════════════════════════════════════════════════════════
  # AC9: Decoration ceases when all minions are assigned
  # ═══════════════════════════════════════════════════════════════════

  Scenario: All minions assigned stops decoration activity
    Given a building group with 3 buildings
    Given 2 minions are decorating buildings
    When all 2 minions are assigned to production tasks
    Then no minions are in decoration state
    Then decoration activity ceases on all buildings

  # ═══════════════════════════════════════════════════════════════════
  # AC10: Creature nests as tier-gated entities; clearing unlocks tier
  # ═══════════════════════════════════════════════════════════════════

  Scenario: T1 forest wolf den exists as hostile nest with strength 50
    Given a 20x20 forest biome grid
    Given a forest_wolf_den nest at position [12, 12] with tier 1 and hostility hostile
    Given the forest_wolf_den has strength 50 and territory_radius 8
    Given the forest_wolf_den contains 5 forest_wolf creatures
    When querying the nest entity
    Then the nest cleared flag is false
    Then the nest is a tier 1 gate entity

  Scenario: Clearing T1 nest unlocks T2
    Given a 20x20 forest biome grid at tier 1
    Given a forest_wolf_den nest at position [12, 12] with strength 50
    Given two imp_camps at positions [10, 10] and [10, 12] with full supply
    Given the combined combat pressure exceeds 50
    When the nest clearing system runs
    Then the forest_wolf_den cleared flag is true
    Then a NestCleared event is emitted for forest_wolf_den
    Then the TierState.currentTier advances to 2
    Then the forest_wolf_den drops hide: 10 and herbs: 5

  Scenario: Clearing T2 nest unlocks T3
    Given a 20x20 forest biome grid at tier 2
    Given a forest_vine_heart nest at position [12, 12] with strength 120
    Given combat groups applying combined pressure exceeding 120
    When the nest clearing system runs
    Then the forest_vine_heart cleared flag is true
    Then the TierState.currentTier advances to 3
    Then the forest_vine_heart drops herbs: 15 and wood: 10 and sinew: 3

  Scenario: Combat pressure below nest strength does not clear the nest
    Given a 20x20 forest biome grid at tier 1
    Given a forest_wolf_den nest at position [12, 12] with strength 50
    Given one imp_camp at position [10, 10] with combat pressure of 30
    When the nest clearing system runs
    Then the forest_wolf_den cleared flag is false
    Then no NestCleared event is emitted

  Scenario: Volcanic T1 nest has higher strength than forest
    Given a 20x20 volcanic biome grid at tier 1
    Given a volcanic_salamander_nest at position [10, 10] with strength 60
    Given combat groups applying combined pressure of 55
    When the nest clearing system runs
    Then the volcanic_salamander_nest cleared flag is false

  Scenario: Optional neutral nest can be cleared without blocking tier progression
    Given a 20x20 forest biome grid
    Given a forest_deer_grove neutral nest at position [8, 8] with strength 20
    Given an imp_camp at position [6, 6] with combat pressure of 25
    When the nest clearing system runs
    Then the forest_deer_grove cleared flag is true
    Then the forest_deer_grove drops hide: 15 and herbs: 8
    Then no TierUnlocked event is emitted

  # ═══════════════════════════════════════════════════════════════════
  # AC11: T3 EXTRACT mode doubles consumption and output
  # ═══════════════════════════════════════════════════════════════════

  Scenario: T3 EXTRACT mode on cleared nest doubles combat group output
    Given a 20x20 forest biome grid at tier 3
    Given a forest_vine_heart nest at position [12, 12] that is already cleared
    Given a war_lodge at position [10, 10] within extract range 8 of the cleared nest
    Given the war_lodge has base_organic_rate 1.5
    When EXTRACT mode is enabled on the forest_vine_heart nest
    Then the war_lodge consumption_multiplier is 2.0
    Then the war_lodge output_multiplier is 2.0
    Then the war_lodge produces 3.0 organic items per cycle

  Scenario: EXTRACT mode requires tier 3
    Given a 20x20 forest biome grid at tier 2
    Given a forest_vine_heart nest at position [12, 12] that is already cleared
    When the player issues an ExtractNest command for forest_vine_heart
    Then the command is rejected because TierState is below 3
    Then the nest extracting flag remains false

  Scenario: EXTRACT mode only applies to combat groups within range 8
    Given a 20x20 forest biome grid at tier 3
    Given a forest_vine_heart nest at position [12, 12] that is already cleared and extracting
    Given a war_lodge at position [1, 1] which is farther than 8 tiles from the nest
    When the combat group system runs for 1 production cycle
    Then the war_lodge has no extract multiplier applied
    Then the war_lodge produces 1.5 organic items per cycle

  Scenario: EXTRACT mode cannot be enabled on uncleared nest
    Given a 20x20 forest biome grid at tier 3
    Given a forest_vine_heart nest at position [12, 12] that is NOT cleared
    When the player issues an ExtractNest command for forest_vine_heart
    Then the command is rejected because the nest is not cleared
    Then the nest extracting flag remains false

  # ═══════════════════════════════════════════════════════════════════
  # AC12: Trader building converts surplus to meta-currency with inflation
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Trader building converts surplus resources to meta-currency
    Given a trader building in a group with manifold containing iron_bar: 10
    Given the trader exchange rate for iron_bar is 1.0 Gold per unit
    Given the trader inflation for iron_bar is 0.0
    When the trading system runs for 1 tick
    Then 10.0 Gold is added to MetaState.currencies
    Then the trader manifold iron_bar is 0

  Scenario: Repeated trading of same resource yields diminishing returns
    Given a trader building with INFLATION_FACTOR 0.3
    Given the trader has already traded 10 iron_bar (inflation accumulated)
    Given the trader exchange rate for iron_bar is 1.0 Gold per unit
    When the trader receives 10 more iron_bar in manifold
    When the trading system runs for 1 tick
    Then the Gold earned is less than 10.0 due to inflation
    Then the trader inflation for iron_bar has increased further

  Scenario: Trading different resources does not share inflation
    Given a trader building with INFLATION_FACTOR 0.3
    Given the trader has already traded 20 iron_bar (high inflation)
    Given the trader manifold contains herbs: 10 with zero inflation
    Given the trader exchange rate for herbs is 1.0 Souls per unit
    When the trading system runs for 1 tick
    Then 10.0 Souls is earned for herbs at full rate
    Then the herbs inflation does not affect iron_bar inflation

  # ═══════════════════════════════════════════════════════════════════
  # Edge Cases
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Combat group with no input supply idles completely
    Given a 16x16 forest biome grid
    Given an imp_camp at position [5, 5] with base_organic_rate 1.0
    Given the imp_camp group manifold is empty (no iron_bar, no herbs)
    When the combat group system runs for 1 production cycle
    Then the imp_camp produces 0 organic items
    Then the imp_camp provides 0 tiles of protection radius
    Then the imp_camp deals 0 damage to creatures

  Scenario: All creatures in zone killed leaves no renewable source
    Given a 16x16 forest biome grid
    Given the creature list for the zone is empty (all killed)
    Given a breeding_pen at position [5, 5] with full supply
    When the combat group system runs for 10 production cycles
    Then the breeding_pen still produces organics from its inputs
    Then no wild creature loot is available in this zone

  Scenario: Invasive creature reaching a building group damages output senders first
    Given a 16x16 forest biome grid
    Given a forest_vine_creeper at position [6, 6] whose territory has expanded to include position [5, 5]
    Given a building group at position [5, 5] with output senders
    When the invasive creature reaches the building group
    Then the forest_vine_creeper damages the output senders first
    Then transport paths connected to those output senders are disrupted

  Scenario: No combat group means no access to organic resources
    Given a 16x16 forest biome grid
    Given an iron_miner at position [5, 5] on an iron_vein
    Given a tannery at position [6, 5] requiring hide as input
    Given no combat group or breeding pen exists
    When the production system runs for 500 ticks
    Then the tannery manifold contains 0 hide
    Then no organic resources exist anywhere in the simulation

  # ═══════════════════════════════════════════════════════════════════
  # Creature Behavior Edge Cases
  # ═══════════════════════════════════════════════════════════════════

  Scenario: Ambient creature flees when health drops below threshold
    Given a 16x16 forest biome grid
    Given a forest_deer at position [8, 8] with health 30 and flee_threshold 0.5
    Given the forest_deer has taken 16 damage (health is 14, below 50% of 30)
    When the creature behavior system runs for 1 tick
    Then the forest_deer moves away from the danger source

  Scenario: Ambient creature wanders within home range
    Given a 16x16 forest biome grid
    Given a forest_deer at position [8, 8] with wander_range 6
    When the creature behavior system runs for 50 ticks
    Then the forest_deer position is always within 6 tiles of [8, 8]

  Scenario: Event-born creature despawns after lifetime expires
    Given a 16x16 volcanic biome grid
    Given an ember_wyrm spawned at position [10, 10] with lifetime_ticks 600
    Given the ember_wyrm has aggression always and attack_dps 12
    When the creature behavior system runs for 600 ticks
    Then the ember_wyrm is despawned and removed from the world

  Scenario: Event-born creature attacks nearest building during lifetime
    Given a 16x16 volcanic biome grid
    Given an ember_wyrm spawned at position [10, 10] with lifetime_ticks 600
    Given the ember_wyrm has attack_target nearest_building and attack_dps 12
    Given an iron_miner at position [8, 8]
    When the creature behavior system runs for 1 tick
    Then the ember_wyrm moves toward the iron_miner
    Then the ember_wyrm deals 12 damage per tick on arrival

  Scenario: Opus-linked creature spawns at opus milestone
    Given a 20x20 desert biome grid
    Given a crystal_golem with spawn_trigger opus_milestone_3
    Given the 3rd main opus milestone has just been sustained
    When the creature spawn system runs for 1 tick
    Then a crystal_golem spawns near a mana_node tile
    Then the crystal_golem has health 300 and territory_radius 8

  Scenario: Opus-linked creature does not spawn before its trigger milestone
    Given a 20x20 desert biome grid
    Given a crystal_golem with spawn_trigger opus_milestone_3
    Given only 2 main opus milestones have been sustained
    When the creature spawn system runs for 100 ticks
    Then no crystal_golem has spawned

  Scenario: Killed creature drops loot into nearest combat group manifold
    Given a 16x16 forest biome grid
    Given a forest_wolf at position [8, 8] with loot hide: 3 and herbs: 1
    Given an imp_camp at position [6, 6] with full supply
    When the forest_wolf health reaches 0
    Then a CreatureKilled event is emitted
    Then hide: 3 is added to the nearest combat group manifold
    Then herbs: 1 is added to the nearest combat group manifold
