# Magnum Opus - Product Requirements Document

## Overview

Magnum Opus is a roguelike factory game with 1-2 hour runs and meta-progression between runs. The player is a spirit commanding biome-native faceless minions to build production chains, manage resources, and complete a randomly generated tech-tree goal (the Opus) before the run ends.

### What the game IS

The Factorio early-game production loop - finding bottlenecks, building chains, optimizing throughput - transplanted into a fantasy roguelike setting. Every run presents a different biome, different Opus goal, and different events. The player wins through production management, not combat micromanagement.

### What the game is NOT

- Not a city builder (no population, no happiness, no zoning)
- Not a tower defense (combat is passive - defense buildings are production chains)
- Not a sandbox (runs have a clear goal and time pressure)
- Not a management sim (no spreadsheet UI - visual feedback everywhere)

### Core references

- **Factorio**: production chains, belt logistics, throughput optimization
- **Against the Storm**: roguelike run structure, event system, biome variety
- **Breath of the Wild**: interconnected systemic world (weather, elements, creatures)

### Core logistics model

Two resource types with fundamentally different logistics:
- **Buildings**: produced by Mall group -> player Inventory (global) -> placed on map via command
- **Resources**: exist only inside group manifolds and transport containers - never in inventory

Starting kit: small set of free buildings + resources to bootstrap first extraction group and Mall.

### Tech direction

- ECS architecture from day one - simulation-first (see docs/ARCH.md)
- Deterministic simulation: same seed + same commands = identical run
- 3D isometric with pixel-art post-processing shaders
- Fixed camera, god-view perspective

---

## Features

<!-- feature:building-groups -->
### F1: Building Groups

The universal production mechanic. All factory gameplay operates through building groups.

**Summary:** Adjacent buildings automatically form a group. Inside a group, resources distribute via manifold (automatic, no player-placed logistics). Between groups, the player routes rune paths and pipes manually.

**Problem:** Factory games require complex internal logistics (belts between every building). For 1-2h runs this is too much friction. Building Groups eliminate internal belt management while preserving the inter-cluster logistics puzzle.

**How it works:**
- Place buildings adjacent to each other -> they form a group
- Identical buildings chain freely (horizontal, vertical, grid): 4 miners in a square = one powerful block
- Different buildings in a group need one adjacency connection; system auto-optimizes internal flow
- Internal connections rendered as mini rune paths (visually consistent with external transport)
- Each group has configurable input receivers and output senders
- Groups are the unit of management in chain manager (energy allocation, priority, pause/resume)
- Buildings merge visually into factory structures as the group grows
- Buildings are produced by Mall -> go to player Inventory -> placed from Inventory (costs nothing extra beyond production)
- Each building has a decorative minion (1:1). Minions are visual-only, reflecting production state
- In combat buildings, visible minion count reflects supply ratio (low supply -> fewer visible warriors)

**Group types by purpose (same mechanic, different composition):**
- **Extraction group**: miners/collectors on natural veins - extract raw resources from terrain
- **Synthesis group**: farms/labs/refineries placed anywhere - convert base resources into other base resources (e.g. tree farm: water input -> wood output). Used when biome lacks a resource naturally.
- **Mall group**: constructor + toolmaker + assembler - produces buildings, tools, utility. Safe default destination for all resources.
- **Combat group**: imp camps + armory - consumes weapons/food, produces territory protection + organic resources. The PRIMARY way to obtain organics (grow or kill = same pipeline).
- **Opus group**: specialized buildings for Opus milestone production

**Acceptance criteria:**
- AC1: Placing a building adjacent to an existing building merges them into a group with shared manifold
- AC2: Resources produced by any building in a group are available to all other buildings in that group that need them
- AC3: Identical buildings placed adjacent chain without requiring manual connection
- AC4: Group displays aggregate input/output rates
- AC5: Player can place input receivers and output senders on group boundary
- AC6: Removing a building that bridges two sub-groups splits them into separate groups
- AC7: Chain manager displays groups as units with energy, priority, and status controls
- AC8: Synthesis groups function without terrain requirements (placeable on any valid tile)

**Non-goals:**
- Player-configurable internal routing within a group (manifold handles this)
- Multi-group merging into super-groups
- Group templates or blueprints (may be added post-MVP)

**Edge cases:**
- Single building = group of 1, fully functional with receivers/senders
- Building placed between two existing groups merges all three
- Group with no input receivers still functions if it contains self-sufficient buildings (miners)
- Removing the last building in a group destroys the group and disconnects all external paths
- Synthesis group with no input supply: buildings idle, no output, no crash

---

<!-- feature:transport -->
### F2: Transport

Resource movement between building groups. Two systems: rune paths for solids, pipes for liquids.

**Summary:** The player connects building groups by placing rune paths (solid resources - models roll along glowing runes) and pipes (liquids - magical aqueducts). Tier upgrades are global: unlocking T2 auto-upgrades all existing paths.

**Problem:** Logistics between production clusters is the core spatial puzzle. The transport system must be visually satisfying (resources visibly moving), mechanically clear (throughput limits, routing), and low-friction (global tier upgrades, no per-segment management).

**How it works:**
- **Early-game (before paths):** minions auto-carry resources between nearby groups - slow, short-range, automatic fallback transport
- **Rune paths:** solid resources. Player draws path from group output to group input. Resource models drop onto path and roll. Throughput = path tier capacity.
- **Pipes/channels:** liquid resources. Separate visual system (stone aqueducts with glowing liquid). Same routing mechanic as paths.
- Tiers: T1 (basic, slow) -> T2 (medium) -> T3 (fast, wide). Unlocking a new tier globally upgrades ALL existing paths and pipes automatically.
- Paths and pipes occupy map tiles. Routing around terrain and other groups is the puzzle.

**Acceptance criteria:**
- AC1: Player can draw rune path from output sender of group A to input receiver of group B
- AC2: Solid resource models visibly roll along rune paths at tier-appropriate speed
- AC3: Liquid resources visibly flow through pipes at tier-appropriate speed
- AC4: Unlocking T2 upgrades all T1 paths and pipes globally without player action
- AC5: Path throughput is capped by tier; excess resources queue at sender
- AC6: Paths and pipes cannot overlap on the same tile (must route around)
- AC7: Destroying a path segment disconnects the route; resources stop flowing
- AC8: Before any paths exist, minions auto-carry resources between nearby groups at reduced speed

**Non-goals:**
- Speed lines, packing, long-range teleport (cut from design)
- Underground routing or flyover paths
- Per-segment tier selection (all paths share the global tier)

**Edge cases:**
- Path drawn to a receiver that is already at max input rate: resources queue at sender
- Path crossing hazard zone: hazard can destroy path segment, requiring repair or reroute
- Disconnected path (middle segment destroyed): resources stop, both ends show warning

---

<!-- feature:world -->
### F3: World & Biomes

The procedurally generated environment: biomes, terrain, hazards, weather, and the systemic interactions between them.

**Summary:** Each run generates a biome-specific map with interconnected environmental systems. Weather affects terrain, elements interact (fire + wind = wildfire), hazards threaten and reward, and landscape determines where buildings can be placed.

**Problem:** The world must create varied, interesting constraints for each run. Same Opus in different biomes should require different strategies. The world runs independently - things happen whether the player is watching or not.

**How it works:**
- Biomes: volcanic, forest, ocean, desert, etc. Each has unique terrain, resources, creatures, hazards, and buildings.
- ~30% universal buildings (work everywhere), ~70% biome-specific.
- Landscape constrains building placement: miners only on ore veins, lava siphons only near lava, etc. This naturally limits group composition.
- Hazards: biome-specific (eruptions, storms, wildfires, sandstorms). Predictable zone and timing. Destroy buildings but enhance affected tiles.
- Sacrifice mechanic: place sacrifice buildings in hazard zones. Player sees odds (e.g. 70% bonus / 30% miss). Hit = tile enhanced + bonus. Miss = building lost.
- Systemic interactions: fire + wind = wildfire spread. Rain fills water. Cold freezes water. All affect production.
- **Fog of war:** map starts hidden. Watchtower buildings reveal cells in radius (like Factorio radar). Simulation runs regardless of visibility.
- **Resource quality is biome-contextual:** same resource has different quality per biome. Example: rotten wood = NORMAL in forest biome, HIGH in undead biome. Regular wood = HIGH in forest, unavailable in undead. Recipes specify quality requirements - biome determines which resources satisfy them.

**Acceptance criteria:**
- AC1: Map generation produces biome-specific terrain with resource veins, liquid sources, and hazard zones
- AC2: Buildings with landscape requirements can only be placed on matching terrain tiles
- AC3: Hazard events announce zone and timing N seconds in advance
- AC4: Sacrifice building placed in hazard zone shows probability of bonus vs miss
- AC5: Hazard destroying a tile applies the enhancement property to that tile
- AC6: At least 3 systemic element interactions are functional (fire+wind, rain+soil, cold+water)
- AC7: World simulation runs independently of player camera position
- AC8: Watchtower building reveals fog in configurable radius around it
- AC9: Player cannot place buildings on hidden (fogged) tiles

**Non-goals:**
- Terraforming (raising/lowering terrain)
- Underground layer
- Multi-story buildings
- Player-directed water flow

**Edge cases:**
- Hazard hits tile with no buildings: tile still gets (lesser) enhancement
- Sacrifice building placed outside any hazard zone: no effect, building sits idle
- Two hazards overlap on same tiles: effects stack or the stronger one wins (design decision per biome)

---

<!-- feature:creatures -->
### F4: Creatures & Combat

Living ecosystem of biome-native creatures. Combat groups are a core resource pipeline (organics), not optional defense.

**Summary:** Creatures are multi-role entities with 5 behavior archetypes. Combat groups (imp camps) are the PRIMARY way to obtain organic resources - growing and killing are the same production pipeline. This makes combat groups mandatory in every run, not an optional defense layer.

**Problem:** Organic resources (wood, hides, herbs, creature parts) cannot be mined from terrain. They must come from creatures - either by farming/breeding or by hunting/killing. The combat system must integrate into the production loop as another resource pipeline, not a separate minigame.

**How it works:**
- Behavior archetypes:
  - Ambient: live independently, resource for hunting/breeding/harvesting
  - Territorial: attack when player expands into their zone
  - Invasive: expand autonomously, reclaim player territory if unchecked
  - Event-born: spawn from Opus tree branch events
  - Opus-linked: tied to main opus, appear at key progression thresholds
- Combat groups = organic resource pipeline:
  - Imp camp consumes weapons + food -> produces territory protection + organic loot
  - Breeding pen consumes food + water -> produces renewable organic materials
  - Growing and killing are equivalent production paths to organics
  - T3 combat groups clear enemy zones for rare resources needed by Opus milestones
  - Combat groups scale like any resource group: more buildings = more throughput
- Minions: biome-native faceless workers. Different stat distributions per biome. No names, no personality.
- Idle minions auto-decorate buildings (player picks style). Beauty = emergent indicator of factory efficiency.
- **Creature nests** = tier gate encounters. Nests are map entities with tier and hostility:
  - T1->T2: clear a T1 creature nest (combat group overpowers it)
  - T2->T3: clear a T2 creature nest (requires stronger combat chains)
  - T3 unlocks "EXTRACT" mode on cleared nests: 2x consumption -> bonus organic output
  - Hostile factions = required targets for some mini-opus. Non-hostile can be killed/extracted by choice.
- **Trading:** Trader building converts surplus resources -> meta-currency with inflation (more you trade same resource -> worse rate). Satisfactory-ticket model.

**Acceptance criteria:**
- AC1: Each biome spawns creatures with at least 3 of the 5 archetypes
- AC2: Territorial creatures attack when player builds within their territory radius
- AC3: Invasive creatures expand their territory over time if unchecked
- AC4: Combat group (imp camp) consumes input resources and produces organic output + protection
- AC5: Under-supplied combat group loses effectiveness; enemies break through, organic output drops
- AC6: T3 combat group can clear an enemy zone and drop rare resources
- AC7: Organic resources are ONLY obtainable through combat/breeding groups (no terrain extraction)
- AC10: Creature nests exist on map as tier-gated entities; clearing a nest of matching tier unlocks next tier
- AC11: T3 EXTRACT mode on cleared nests doubles combat group consumption and output in nest vicinity
- AC12: Trader building accepts surplus resources and converts to meta-currency at diminishing rates
- AC8: Idle minions with no tasks auto-decorate nearby buildings
- AC9: Minion decoration activity ceases when all minions are assigned to tasks

**Non-goals:**
- Player-controlled combat (real-time unit selection, attack commands)
- Named/unique creatures with personality
- Creature evolution or genetics system
- Active player combat abilities
- Organics from terrain extraction (explicitly excluded - must come from creatures)

**Edge cases:**
- Combat group with no input supply: minions idle, no protection, no organic output
- All creatures in a zone killed: zone is safe but no renewable creature resources - player must find new zone or breed
- Invasive creatures reach a building group: they damage output senders first, disrupting logistics
- No combat group built: player has no access to organics, blocking recipes that require them

---

<!-- feature:progression -->
### F5: Progression

In-run progression through the Opus tree (production milestones + mini-opus branches), tier unlocks, and encounter-gated mechanics.

**Summary:** The Opus tree is a unified progression structure. Main path nodes are **production throughput milestones** ("produce X resource at Y/min"). Side branches are **mini-opus events** (challenges that award bonuses and meta-currency). One tree, one goal, everything inside it.

**Problem:** A 1-2h run needs clear pacing and a legible win condition. The player should look at the Opus tree and immediately understand what production chains to build. No abstract artifacts - every node is a measurable throughput goal.

**How it works:**
- **Opus tree structure:**
  - Main path nodes = production milestones: "produce N of resource X per minute"
  - Side branches = mini-opus: optional challenges that give bonuses + meta-currency
  - Final node = sustain all main-path rates simultaneously for a duration
  - Tree is visible from run start - player can plan entire strategy upfront
  - Same Opus in different biomes requires different approaches (some resources must be synthesized)
- **Production milestones:**
  - Each node specifies a resource and a rate (e.g. "5 obsidian plates/min")
  - Milestone is checked when the player achieves and sustains the rate
  - Nodes unlock sequentially along the main path, some branches run in parallel
  - Milestones cover all resource types: extraction, synthesis, organics (from combat groups)
- **Mini-Opus (tree branches):**
  - Side branches attached to main path nodes
  - Trigger types: on-demand (activate when ready), time-based (deadline), conditional (state match)
  - Completed = bonus + meta-currency. Missed = no penalty on main path, but lost bonus
  - Examples: "survive ash storm while maintaining 5/min rate", "supply 10 wood to wandering trader"
- **3 Tiers (nest-gated):**
  - T1 (setup, ~25 min): basic extraction, simple recipes, T1 rune paths, first Mall group
  - T2 (expansion, ~35 min): complex recipes, biome buildings, pipes, T2 paths. Unlocked by clearing a T1 creature nest.
  - T3 (opus push, ~30 min): final recipes, T3 paths, opus groups. Unlocked by clearing a T2 creature nest.
  - All recipes for a tier unlock immediately on tier transition. Buildings auto-upgrade on tier unlock (no demolish+rebuild).

**Acceptance criteria:**
- AC1: Opus tree nodes are production throughput milestones (resource + rate), not item crafting goals
- AC2: Milestone is marked complete when player sustains the required rate for a verification period
- AC3: Opus tree UI shows all main nodes + side branches, current rates vs required, completion %
- AC4: Mini-opus branches are visually attached to their parent main-path node
- AC5: Completing a mini-opus branch awards meta-currency; skipping it has no main-path penalty
- AC6: Final Opus node requires simultaneous sustain of all main-path rates
- AC7: T2 buildings/recipes are inaccessible until T1 creature nest is cleared
- AC8: T3 buildings/recipes are inaccessible until T2 creature nest is cleared
- AC10: All recipes for a tier become available immediately on tier unlock
- AC11: Existing buildings auto-upgrade to new tier visually and functionally on tier unlock
- AC9: Completing the final Opus node triggers run-end sequence with scoring

**Non-goals:**
- More than 3 tiers (5-tier design was explicitly cut for run length)
- Artifact/item crafting as Opus goals (milestones are throughput-based)
- Mini-Opus as a separate system outside the tree (all events are tree branches)
- Difficulty selection within a run (difficulty = biome+opus mismatch)

**Edge cases:**
- Player's rate drops below milestone after initial completion: milestone stays completed (no regression)
- Time-based mini-opus deadline passes while player is in crisis: branch marked as missed, main path unaffected
- Opus requires a resource not naturally in biome: player must build synthesis group or trade via creatures
- Run timer expires before final node: partial scoring based on tree fill %
- All mini-opus branches missed: run still completable, but minimal meta-currency earned

---

<!-- feature:meta -->
### F6: Meta-Progression

Between-run progression: currencies earned from runs, permanent unlocks, and difficulty scaling.

**Summary:** Players earn 3 meta-currencies (Gold, Souls, Knowledge) from Mini-Opus completions, multiplied by the main Opus. Currencies buy permanent unlocks that expand options for future runs.

**Problem:** Roguelike replayability requires meaningful between-run progression. Each run should feel like it contributed to a larger journey, while individual runs remain self-contained.

**How it works:**
- 3 currencies: Gold (from economy/production mini-opus), Souls (from creature/combat mini-opus), Knowledge (from technology/discovery mini-opus).
- Opus multiplier: x1.5 / x2 / x3 based on Opus difficulty (biome mismatch = higher difficulty = higher multiplier).
- Permanent unlocks: new biomes, starting bonuses, expanded building pools, cosmetic styles.
- **In-run trading:** Trader building (special building group) accepts surplus resources -> converts to meta-currency during the run. Exchange rate has inflation - the more you trade the same resource, the worse the rate becomes. Satisfactory-ticket model.

**Acceptance criteria:**
- AC1: Run-end screen shows earned currencies with Opus multiplier applied
- AC2: Meta store displays available unlocks with currency costs
- AC3: Unlocked content persists across runs
- AC4: Opus multiplier is determined by biome-opus difficulty match
- AC5: Player can view total lifetime currency earnings and spending
- AC6: Trader building converts surplus resources to meta-currency during a run
- AC7: Trading the same resource repeatedly yields diminishing returns (inflation)

**Non-goals:**
- Pay-to-win or real-money currencies
- Season passes or time-limited content
- Leaderboards (may be added post-MVP)

**Edge cases:**
- Run abandoned before any Mini-Opus: 0 currencies earned
- All Mini-Opus failed (unprepared): reduced currency (penalty), but not zero
- Player has enough currency for an unlock mid-calculation: purchase atomic, no partial spending

---

<!-- feature:energy -->
### F7: Energy

Power generation, distribution, and the surplus/deficit throttle that drives production optimization.

**Summary:** Energy is the global throttle. Surplus speeds up production groups, deficit slows them down. The player allocates energy across groups via chain manager, creating the core optimization puzzle.

**Problem:** Without a shared constraint, production groups operate independently and optimization is trivial. Energy creates interdependence: expanding one group's output means taking energy from another.

**How it works:**
- Biome-specific energy sources (lava siphons, wind turbines, water wheels, etc.)
- Energy is generated by energy buildings and consumed by production groups
- Surplus: all groups speed up proportionally
- Deficit: player picks which groups to throttle via chain manager priorities
- Energy allocation is at group level, not building level

**Acceptance criteria:**
- AC1: Energy balance (generation - consumption) is displayed in real-time
- AC2: Surplus energy proportionally increases production speed of all groups
- AC3: Deficit energy reduces production speed; highest-priority groups are throttled last
- AC4: Player can set group energy priority (high/medium/low) in chain manager
- AC5: Building a new energy source immediately contributes to the energy pool
- AC6: Destroying an energy building immediately reduces generation

**Non-goals:**
- Energy storage/batteries (energy is instant, no buffering)
- Energy transmission lines (energy is global, not routed)
- Multiple energy types (one unified energy pool)

**Edge cases:**
- All energy buildings destroyed: all production stops, only manual minion actions remain
- Energy exactly at 0 balance: no speed bonus or penalty
- Single group set to HIGH priority with massive deficit: that group runs near-normal, all others nearly stop

---

<!-- feature:ux -->
### F8: UX Tools

Built-in production intelligence: calculator, chain visualizer, efficiency dashboard. No alt-tabbing to external tools.

**Summary:** The game provides Factorio-quality (or better) production analytics as first-class features. Players should never need external calculators or spreadsheets.

**Problem:** Factory games are math-heavy. Without built-in tools, players alt-tab to wikis and calculators, breaking immersion. For a roguelike with short runs, this friction is unacceptable.

**How it works:**
- Production calculator: "I need X items/min" -> shows required buildings, resources, groups
- Chain visualizer: overlay showing all groups, connections, throughput, bottlenecks, energy allocation
- Efficiency dashboard: real-time graphs of production rates, consumption, energy balance, minion allocation

**Acceptance criteria:**
- AC1: Calculator accepts target item + rate, outputs required building chain
- AC2: Chain visualizer highlights bottlenecks (groups producing below capacity)
- AC3: Dashboard shows at least: production rates, energy balance, resource stockpiles
- AC4: All UX tools are accessible without pausing the game
- AC5: Calculator accounts for current resource quality (normal/high) in its calculations

**Non-goals:**
- Auto-building from calculator output (calculator is information, not automation)
- Replay/recording of past run analytics
- Comparative analytics between runs

**Edge cases:**
- Calculator asked for item that requires unavailable (tier-locked) buildings: shows "requires T2/T3" label
- Chain visualizer with 0 groups: empty overlay, no crash
- Dashboard during run start (no data yet): shows zeros, not errors

---

<!-- feature:ecs-engine -->
### F9: Cross-Feature Integration (ECS Engine)

Verify that all 8 features work together through real ECS pipelines.

**Summary:** All 8 features (building-groups, transport, world, creatures, progression, meta, energy, ux) are implemented and pass isolated tests. This feature creates integration scenarios that combine 3+ plugins per test, exercising cross-feature pipelines through real `app.update()` cycles. The tests expose and fix wiring bugs that only surface when plugins are stacked.

**Problem:** Each feature was developed and tested in isolation. No test ever combines SimulationPlugin + WorldPlugin + CreaturesPlugin. Eight wiring bugs exist that prevent features from integrating: missing system registrations, duplicate event types, unwritten resources, and uncoordinated tick counters. These bugs are invisible in isolated tests but fatal in a real game run.

**How it works:**
- 12 integration scenarios each combine 3+ features and run real `app.update()` cycles (10-300 ticks)
- Scenarios cover the full gameplay pipeline: energy -> production -> manifold -> transport -> progression
- Each scenario exposes at least one wiring bug that must be fixed for the test to pass
- Wiring bug fixes are minimal - just registration, bridging, or initialization code

**Acceptance criteria:**
- AC1: Production pipeline - energy source powers miners+smelter in group A, processor in group B connected by rune path; after ~200 ticks cargo entities exist in transit and destination manifold receives resources
- AC2: Transport delivery with real production - group A produces IronOre, transports to group B smelter which consumes and produces IronBar; group B manifold contains IronBar after sufficient ticks
- AC3: Energy crisis cascade - working chain sustaining milestone loses energy source; production halts (idle_reason=NoEnergy), production rate drops to 0, sustain_ticks resets
- AC4: Nest clear -> tier progression - combat pressure exceeds nest strength; NestCleared fires, TierState advances to tier 2, transport tier upgrades, BuildingTier upgrades
- AC5: Organic supply chain - combat group produces Hide, transported to Tannery group, produces TreatedLeather; milestone sustain_ticks increments
- AC6: Group split on removal - 3 buildings in a row form 1 group; removing middle building yields 2 separate groups with correct manifold splits
- AC7: Full run win condition - opus nodes with low required_rate and short sustain window; production sustains all nodes, RunWon event emitted, RunState.status==Won
- AC8: Diamond network conservation - 4 groups in A->B, A->C, B->D, C->D diamond; total_produced == sum(manifolds + cargo) + total_consumed (no duplication, no loss)
- AC9: Determinism - identical setup run twice for 50 ticks; all manifold, energy, production, and cargo state matches exactly
- AC10: UX dashboard reads live state - production chain running; DashboardState reflects EnergyPool, TierState, and opus progress correctly
- AC11: Trader converts surplus - production creates surplus in manifold; Trader converts to Gold; TraderEarnings.gold > 0, inflation accumulates, second trade yields less per unit
- AC12: Hazard destroys building -> group reforms - 3 adjacent buildings form 1 group; hazard destroys middle building; group splits into 2, energy rebalances, production continues independently

**Non-goals:**
- Rendering or visual integration testing
- Performance benchmarking or load testing
- Testing more than 300 ticks per scenario
- Testing all biome variants (one biome per scenario is sufficient)

**Edge cases:**
- Stacking SimulationPlugin + CreaturesPlugin must not panic on duplicate event registration
- Combined SimTick (WorldPlugin) and SimulationTick (UX) counters must not desync
- Groups spawned by group_formation_system must have position data for combat_pressure_system range checks
- Trading system must be registered and functional when SimulationPlugin is active

---

<!-- feature:game-startup -->
### F10: Game Startup

Initialize all run state before the first simulation tick: recipe database validation, terrain generation, starting kit, opus tree, fog, and run configuration.

**Problem:** The simulation layer (8 features, 421 tests) assumes that Grid terrain is populated, Inventory contains a starting kit, OpusTreeResource has milestone nodes, FogMap has a revealed area, and RunConfig is configured. Currently, tests set these up manually per scenario. A real game run needs a single `GameStartupPlugin` that writes all required ECS state in the Bevy `Startup` schedule so that the first `Update` tick finds a fully initialized world. Without this, the simulation has no terrain to check, no buildings to place, no milestones to track, and no fog to enforce.

**How it works:**

1. **Recipe validation** - On startup, iterate every `BuildingType` variant and call `default_recipe(bt)`. Validate invariants: extractors (buildings with `terrain_req() == Some(_)`) must have empty `inputs`; mall buildings (Constructor, Toolmaker, Assembler) must have `output_to_inventory: true`; energy buildings (WindTurbine, WaterWheel, LavaGenerator, ManaReactor) and utility buildings (Watchtower, Trader, SacrificeAltar) must have `duration_ticks == 1`. Panic on violation - broken recipes must never reach the simulation.

2. **Terrain generation** - Deterministic seed-based generation on a 64x64 grid. Uses the run seed to place terrain types. Default terrain is Grass. Resource veins (IronVein, CopperVein, StoneDeposit, WaterSource) are placed as clusters near predetermined positions to guarantee extractable resources within the starting area. Advanced terrain (ObsidianVein, ManaNode, LavaSource) is placed farther from spawn for T2/T3 gameplay. Same seed always produces identical terrain (determinism invariant). Writes to `Grid.terrain`.

3. **Starting kit** - Populates `Inventory.buildings` with T1 buildings sufficient to bootstrap first extraction group and Mall: 4x IronMiner, 2x CopperMiner, 2x StoneQuarry, 2x WaterPump, 2x IronSmelter, 1x CopperSmelter, 1x Sawmill, 1x TreeFarm, 1x Constructor, 3x WindTurbine, 1x Watchtower. Total: 20 buildings across 10 types. All buildings in the kit must be tier 1 (`building.tier() == 1`).

4. **Opus tree initialization** - Populates `OpusTreeResource.main_path` with 7 milestone nodes representing the main production chain: IronBar (T1) -> CopperBar (T1) -> Plank (T1) -> SteelPlate (T2) -> RefinedCrystal (T2) -> RunicAlloy (T3) -> OpusIngot (T3). Each node specifies a `ResourceType` and `required_rate` (items/min) appropriate to its tier and recipe complexity. Sets `sustain_ticks_required = 600`. All nodes start with `current_rate = 0.0` and `sustained = false`.

5. **Fog initialization** - Reveals cells within Manhattan distance radius around the spawn point (center of map). Radius must be large enough that all starting-kit terrain requirements (IronVein, CopperVein, StoneDeposit, WaterSource) fall within revealed area. Writes to `FogMap.revealed`.

6. **Run configuration** - Initializes `RunConfig` with biome, tick limit (`max_ticks`), `current_tick = 0`, ticks-per-second, and sustain window. Initializes `TierState` to tier 1. All startup systems run in Bevy `Startup` schedule (execute once before the first `Update`).

**ECS connections (what startup writes, what simulation reads):**

| Startup System | Writes to | Read by (simulation) |
|---|---|---|
| recipe_validation | Validates RecipeDB / `default_recipe` | placement_system, production_system |
| terrain_gen | `Grid.terrain` | placement_system (terrain requirement check) |
| starting_kit | `Inventory.buildings` | placement_system (inventory consumption) |
| opus_tree_init | `OpusTreeResource.main_path`, `sustain_ticks_required` | opus_tree_sync_system, milestone_check_system |
| fog_init | `FogMap.revealed` | placement_system (fog visibility check) |
| run_config_init | `RunConfig`, `TierState` | run_lifecycle_system, tier_gate_system |

**Acceptance criteria:**

- AC1: After `GameStartupPlugin` runs, `default_recipe(bt)` returns a valid `Recipe` for every `BuildingType` variant - no panics, no missing arms. Extractors have empty inputs. Mall buildings have `output_to_inventory: true`. Energy and utility buildings have `duration_ticks == 1`.
- AC2: After terrain generation with a given seed, `Grid.terrain` contains at least one cluster each of IronVein, CopperVein, StoneDeposit, and WaterSource within Manhattan distance 15 of the map center. Grid dimensions are 64x64.
- AC3: Terrain generation is deterministic - running with the same seed twice produces identical `Grid.terrain` maps (same positions, same terrain types).
- AC4: After starting kit initialization, `Inventory.buildings` contains exactly: IronMiner=4, CopperMiner=2, StoneQuarry=2, WaterPump=2, IronSmelter=2, CopperSmelter=1, Sawmill=1, TreeFarm=1, Constructor=1, WindTurbine=3, Watchtower=1. Total = 20 buildings. Every building in the kit has `tier() == 1`.
- AC5: After opus tree initialization, `OpusTreeResource.main_path` contains exactly 7 nodes with resources [IronBar, CopperBar, Plank, SteelPlate, RefinedCrystal, RunicAlloy, OpusIngot] in order. Each node has `required_rate > 0.0`, `current_rate == 0.0`, `sustained == false`. `sustain_ticks_required == 600`.
- AC6: After fog initialization, `FogMap.revealed` contains all cells within Manhattan distance of the reveal radius from the spawn point. Every cell with starting-resource terrain (IronVein, CopperVein, StoneDeposit, WaterSource) that was placed by terrain_gen within the starting area is revealed.
- AC7: After run config initialization, `RunConfig.current_tick == 0`, `RunConfig.max_ticks > 0`, `TierState.current_tier == 1`. `RunState.status == InProgress`.
- AC8: All startup systems execute in Bevy `Startup` schedule. After one `app.update()` call on a fresh App with `MinimalPlugins + SimulationPlugin + GameStartupPlugin`, all resources (Grid, Inventory, OpusTreeResource, FogMap, RunConfig, TierState, RunState) are populated and queryable with correct initial values.

**Non-goals:**
- Biome-specific starting kits (all biomes use the same kit for MVP; biome variation is a meta-progression unlock)
- Procedural opus tree generation (milestone sequence is fixed for MVP; randomization is post-MVP)
- Save/load of startup state (runs are ephemeral)
- Rendering or visual feedback during startup (startup is instantaneous, one frame)

**Edge cases:**
- Seed value 0: must still produce valid terrain (no division-by-zero or empty map)
- Grid cell at map boundary (0,0) or (63,63): terrain generation and fog reveal must handle edges without out-of-bounds
- Starting kit building placed on a tile without matching terrain: placement_system rejects it (startup only stocks inventory, does not place buildings)
- OpusTreeResource queried before first Update: all nodes show `current_rate = 0.0`, `sustained = false`, `completion_pct = 0.0`
- Duplicate `GameStartupPlugin` registration: must not double-populate inventory or create duplicate opus nodes (idempotent or panic)

---

<!-- feature:game-render -->
### F11: Visual Rendering

Render the game world as a 3D isometric scene with pixel-art post-processing. Read-only layer over ECS state - never mutates simulation.

**Problem:** The simulation layer (9 features, 421 tests) runs headless. To play the game, every ECS entity and resource must be visualized: terrain grid, buildings, transport paths, creatures, fog, overlays. The render pipeline described in `docs/VISUALS.md` (impostor sprites with albedo+normal+depth maps, per-pixel lighting, outline/toon/posterization post-processing) must be implemented as a Bevy plugin that syncs ECS state to 3D scene entities each frame.

**How it works:**

1. **Grid rendering** - Orthographic camera looking at a 64×64 tile grid. Each tile is a flat quad with height derived from terrain type (resource veins slightly raised, water slightly depressed, lava glowing). Terrain type determines tile texture/color. Grid lines visible at default zoom, fade at distance. Tiles use the impostor sprite format: albedo + normal + depth maps for per-pixel lighting.

2. **Building sync** - Each ECS entity with `Building` + `Position` components gets a corresponding 3D scene entity. Sync runs every frame: spawn scene entity when building appears, despawn when removed, update visual state (idle, producing, no-energy, paused) via material parameters. Buildings use impostor sprites rendered as textured quads. Modular buildings merge visuals based on adjacency (extended wall segments for same-type neighbors, connection arches for different-type neighbors).

3. **Group outlines** - Each building group gets a colored outline rendered as a convex hull or per-tile border around its members. Outline color encodes group state: active (green), paused (yellow), no-energy (red), idle (gray). Labels show group name and aggregate throughput.

4. **Transport visualization** - Rune paths rendered as tiled ground sprites with UV-scroll shimmer. Pipes as tiled sprites with internal liquid UV-scroll + palette cycling. Cargo entities (resource items in transit) rendered as small impostor sprites translating along the path spline. Cargo bounce via vertex displacement `sin(time + path_progress)`.

5. **Creature and nest rendering** - Creatures and nests synced from ECS entities with `Creature`/`Nest` components. Creatures use frame-based spritesheet animation (idle, move, attack, death) with 4 facing directions. Nests rendered as static impostors with emission pulse indicating activity level.

6. **Fog of war** - Unrevealed tiles rendered as opaque dark overlay. Revealed but out-of-watchtower-range tiles rendered with desaturation shader. Fully visible tiles rendered normally. Fog state read from `FogMap` resource.

7. **Ghost preview** - When the player is about to place a building, a semi-transparent ghost sprite follows the cursor grid position. Green tint = valid placement, red tint = invalid (wrong terrain, occupied, fogged). Ghost reads placement validation from the same rules as `placement_system`.

8. **Post-processing chain** - Applied to the full low-res render target in order: (a) outline via Sobel edge detection on depth+normal buffers, (b) toon shading - quantize luminance into 3 bands, (c) posterization - reduce color depth to 8 levels per channel, (d) nearest-neighbor upscale to window resolution. Low-res target: 480×270 (4× upscale at 1920×1080).

9. **Lighting** - One directional light (sun) providing base illumination via normal-map dot product. Point lights on forges, rune paths, lava tiles, and magic buildings. Ambient term prevents full-black shadows. Self-shadow from depth map comparison. Emission maps for glowing elements (additive, ignores lighting).

10. **Shader animations** - No extra sprite frames. Vertex displacement for idle bob and wind sway. UV animation for flowing liquids, spinning gears, palette cycling. Emission pulse for forges and runes. Normal map rotation for animated sub-regions.

**ECS connections (what render reads, never writes):**

| Render System | Reads from | Visualizes as |
|---|---|---|
| grid_render_sync | `Grid.terrain` | Terrain tile quads with height + texture |
| building_render_sync | `Building`, `Position`, `GroupMember`, `ProductionState` | Impostor sprites with state-driven materials |
| group_outline_sync | `GroupMember` queries, `EnergyPool` | Colored outlines + labels |
| transport_render_sync | `Path`, `CargoContainer` | Path sprites + cargo sprites |
| creature_render_sync | `Creature`, `Nest`, `Position` | Animated spritesheets + nest impostors |
| fog_render_sync | `FogMap` | Dark overlay / desaturation shader |
| ghost_render_sync | cursor position + placement validation | Semi-transparent ghost sprite |

**Acceptance criteria:**

- AC1: After `GameStartupPlugin` + `RenderPlugin` run, every tile in `Grid.terrain` has a corresponding scene entity with correct position. Terrain types are visually distinguishable (different color/texture per type).
- AC2: When a building is placed via `PlacementCommands`, a scene entity appears at the correct grid position by the next frame (render sync runs after simulation). When removed, the scene entity despawns by the next frame.
- AC3: Building visual state reflects ECS state: producing buildings show activity (shader animation), idle buildings are static, no-energy buildings show dimmed material, paused buildings show yellow tint.
- AC4: Group outlines enclose all buildings in a group. Outline color matches group state. When a group splits, outlines update to show two separate groups.
- AC5: Transport paths are visible as continuous lines between group boundaries. Cargo sprites move along the path at the correct speed. Cargo type is visually distinguishable (different sprite per ResourceType).
- AC6: Fog overlay covers unrevealed tiles completely. Revealed tiles within watchtower range render at full brightness. Tiles outside watchtower range render desaturated.
- AC7: Ghost preview appears at cursor grid position during placement mode. Ghost color reflects placement validity (green=valid, red=invalid). Ghost disappears when placement mode exits.
- AC8: Post-processing chain produces visible outlines on all object silhouettes, toon-shaded lighting with discrete shadow bands, and posterized colors. Upscale uses nearest-neighbor (no blur).
- AC9: Point lights on emissive buildings (forges, lava generators, mana reactors) illuminate nearby sprites via normal-map lighting. Light radius and color are per-building-type.
- AC10: Shader animations run without extra sprite frames: buildings idle-bob, trees sway, liquids flow in pipes, rune paths shimmer. Animations are time-based and desync between entities (per-entity phase offset from entity ID).
- AC11: All render systems are read-only - no system in `RenderPlugin` writes to any simulation component, resource, or event. Render plugin can be removed without affecting simulation behavior.

**Non-goals:**
- Runtime camera rotation or perspective switching (fixed orthographic isometric)
- Dynamic shadow maps or real-time global illumination (lighting is per-sprite via normal maps)
- Asset pipeline or procedural generation of sprites (assets are pre-baked PNGs loaded at startup)
- LOD (level of detail) system - the low-res target handles visual simplification
- Particle systems (post-MVP; shader animations cover MVP visual effects)

**Edge cases:**
- Building placed at grid edge (0,0) or (63,63): scene entity must render at correct screen position without clipping
- 200+ buildings on screen simultaneously: render sync must not drop below 30 FPS on low-res target
- Transport path with 0 cargo: path sprites render but no cargo sprites exist - no crash
- Creature dies (entity despawned): scene entity removed in same frame, no orphaned sprites
- FogMap with all tiles revealed: fog overlay system runs but produces no visible effect
- Weather change (CurrentWeather): affects directional light color/intensity - no discontinuous pop (lerp over ~10 frames)
- Building type with no loaded sprite asset: render with magenta placeholder quad (never panic, never invisible)

---

<!-- feature:game-ui -->
### F12: User Interaction

Camera control, player input, and all UI panels. The layer between the human and the simulation.

**Problem:** The simulation accepts commands via ECS resources (`PlacementCommands`, `TransportCommands`, `RemoveBuildingCommands`). The render layer visualizes state. Between them, nothing exists to translate mouse clicks into grid coordinates, display build menus, show inventory, or control game speed. Without this plugin the player cannot interact with the game at all.

**How it works:**

1. **Camera** - Orthographic camera with fixed isometric angle (35.264° from horizontal). Controls: WASD/arrow keys for pan, scroll wheel for zoom-to-cursor (zoom changes orthographic scale, not position - the tile under cursor stays under cursor). No rotation. Camera bounds clamp to grid extents with margin. Camera smooth-follows pan input (lerp, not instant snap).

2. **Mouse-to-grid raycasting** - Every frame, cast a ray from mouse screen position through the orthographic camera onto the ground plane (y=0). Convert hit point to grid coordinates `(gx, gy)` using inverse isometric transform. Store result in `CursorGridPos` resource. If cursor is outside grid bounds or over a UI panel, `CursorGridPos` is `None`.

3. **Building placement** - Player selects a building type from the build menu. Entering placement mode shows the ghost preview (rendered by game-render). Left-click on a valid tile writes to `PlacementCommands` and consumes from `Inventory`. Right-click or Escape exits placement mode. If `Inventory` count for the selected type is 0, placement mode auto-exits with a notification.

4. **Building removal** - Right-click on an existing building writes to `RemoveBuildingCommands`. Building returns to `Inventory`. Confirmation is not required (undo via re-placement).

5. **Transport path drawing** - Player enters path-draw mode from UI. Click source group boundary tile, drag to destination group boundary tile. Intermediate waypoints snap to grid. On release, validate path (no overlap with buildings, no existing path on tiles) and write to `TransportCommands`. Player selects Solid (rune path) or Liquid (pipe) before drawing. Path type selector visible during draw mode.

6. **Game speed control** - Keyboard shortcuts: Space = pause/resume, 1/2/3 = speed multipliers (1×, 2×, 4×). Current speed shown in the top bar. While paused, player can still place buildings and draw paths (commands queue, execute on unpause).

7. **Build menu panel** - Docked left panel showing available buildings grouped by category (Extraction, Processing, Mall, Combat, Energy, Utility). Each entry shows: icon, name, inventory count, tier requirement. Tier-locked buildings shown grayed with "T2"/"T3" label. Click selects building for placement mode.

8. **Inventory panel** - Docked panel showing current building stock. Grouped by type. Shows count per building type. Updates live as Mall produces buildings or player places them.

9. **Energy bar** - Top bar showing: total generation, total consumption, ratio percentage, surplus/deficit indicator. Color: green (surplus >= 10%), yellow (0-10% surplus), red (deficit). Clicking opens energy allocation overlay (per-group priority sliders).

10. **Opus tree panel** - Docked right panel showing the milestone tree. Each node: resource icon, required rate, current rate, progress bar, sustained/not-sustained indicator. Main path nodes connected by lines. Mini-opus branches shown as side nodes. Completed nodes glow. Current target node highlighted.

11. **Minimap** - Small fixed panel (bottom-right corner) showing the full 64×64 grid at reduced scale. Terrain colors match main view. Building dots. Fog overlay. Camera viewport rectangle. Click on minimap pans camera to that location.

12. **Tooltips** - Hover over any building: shows building type, group name, current recipe, production state, energy consumption. Hover over transport path: shows cargo type, throughput, tier. Hover over creature/nest: shows type, behavior, health/strength. Tooltips appear after 300ms hover delay, dismiss on mouse move.

13. **Notifications** - Transient messages for game events: "Milestone reached: IronBar", "Nest cleared - Tier 2 unlocked!", "Energy deficit - 3 groups throttled", "Hazard warning: lava eruption in 30 ticks". Stack in top-center, auto-dismiss after 5 seconds. Max 5 visible simultaneously.

**ECS connections (what UI reads and writes):**

| UI System | Reads | Writes |
|---|---|---|
| camera_system | keyboard/mouse input | Camera transform |
| cursor_raycast_system | mouse position, Camera | `CursorGridPos` resource |
| placement_input_system | `CursorGridPos`, `Inventory`, mouse clicks | `PlacementCommands` |
| remove_input_system | `CursorGridPos`, mouse clicks | `RemoveBuildingCommands` |
| path_draw_system | `CursorGridPos`, mouse drag, `PathOccupancy` | `TransportCommands` |
| game_speed_system | keyboard input | `GameSpeed` resource (read by simulation tick) |
| build_menu_system | `Inventory`, `TierState`, `BuildingDB` | placement mode state |
| energy_bar_system | `EnergyPool` | energy allocation overlay |
| opus_tree_panel_system | `OpusTreeResource`, `ProductionRates` | - (read-only display) |
| minimap_system | `Grid`, `FogMap`, Camera | Camera transform (on click) |
| tooltip_system | hovered entity queries, `CursorGridPos` | - (read-only display) |
| notification_system | `Events<MilestoneReached>`, `Events<TierUnlocked>`, etc. | - (read-only display) |

**Acceptance criteria:**

- AC1: Camera pans with WASD/arrows at constant screen-space speed. Camera cannot pan beyond grid bounds (clamped with 2-tile margin).
- AC2: Scroll wheel zooms toward cursor position - the grid tile under the cursor stays under the cursor after zoom. Zoom range: 0.5× to 4× of default orthographic scale.
- AC3: `CursorGridPos` correctly maps mouse screen position to grid coordinates for all zoom levels and camera positions. Hovering over grid tile (3,5) at any zoom reports `Some((3,5))`. Hovering outside grid or over UI panel reports `None`.
- AC4: Left-click during placement mode places building at `CursorGridPos` if valid. `Inventory` count decreases by 1. `PlacementCommands` contains the placement command. Invalid click (occupied, fogged, wrong terrain) does nothing - no command written, no inventory consumed. Right-click or Escape during placement mode exits placement mode (does NOT remove any building).
- AC5: Right-click on building outside placement mode removes it. Building returns to `Inventory` (count increases by 1). `RemoveBuildingCommands` contains the removal command. Right-click on empty tile does nothing.
- AC6: Path drawing produces a valid `TransportCommands` entry connecting two group boundary tiles. Path segments snap to grid. Drawing over occupied tiles (buildings, existing paths) shows red highlight and does not commit.
- AC7: Space toggles pause. 1/2/3 sets speed. While paused, placement and path drawing still work - commands queue and execute when unpaused.
- AC8: Build menu shows all building types with correct inventory counts and tier states. Tier-locked buildings are grayed and unselectable. Clicking a building with count > 0 enters placement mode.
- AC9: Energy bar reflects `EnergyPool` values: generation, consumption, ratio. Color changes at correct thresholds (green >= 110%, yellow 100-110%, red < 100%).
- AC10: Opus tree panel shows all milestone nodes with correct current_rate, required_rate, and sustained status. Progress bars update each tick. Completed nodes visually distinct from in-progress and locked nodes.
- AC11: Minimap renders terrain, buildings, fog, and camera viewport rectangle. Clicking minimap pans camera to clicked grid position.
- AC12: Tooltips show correct data for hovered buildings, paths, and creatures. Tooltip appears after 300ms delay and disappears when cursor moves away.
- AC13: Notifications appear for MilestoneReached, TierUnlocked, hazard warnings, and energy deficit events. Max 5 visible, auto-dismiss after 5 seconds.

**Non-goals:**
- Key rebinding or input customization (post-MVP)
- Controller/gamepad support (mouse+keyboard only)
- Drag-select multiple buildings (single-click interaction only)
- In-game settings menu (resolution, volume, etc. - post-MVP)
- Tutorial or onboarding flow
- Production calculator UI (covered by F8: UX Tools - separate feature)
- Chain visualizer overlay (covered by F8: UX Tools)

**Edge cases:**
- Window resize: camera aspect ratio updates, UI panels reflow, minimap stays fixed size
- Click on building that overlaps with UI panel: UI panel consumes the click, no placement/removal happens
- Path drawing interrupted by Escape: partial path discarded, no command written
- Inventory reaches 0 during placement: current ghost disappears, placement mode exits, notification "Out of [building type]"
- Zoom at grid edge: camera clamp prevents scrolling to see empty space beyond grid
- Two simultaneous notifications of same type: show both (no dedup), stack vertically
- Pause during hazard countdown: countdown ticks freeze (simulation paused), hazard warning notification stays visible

---

<!-- feature:world-foundation -->
### F13: World Foundation

Shared spatial and configuration substrate for all world-gen, placement, and visualization features.

**Problem:** Every later feature (landscape, resources, placement, render) needs the same two pieces of state to start: (a) the run seed plus world dimensions (deterministic generation depends on them), and (b) a grid resource that future placement commands will mutate under single-writer discipline. Without this foundation, each downstream feature would either fabricate its own seed (breaking determinism across modules) or fail the core's single-writer check by also claiming `Grid`. F13 solves both by declaring ownership up-front: `world_config` (StaticData) writes `WorldConfig`, `grid` (SimDomain, `Phase::World`) owns `Grid`. The grid bootstrap this feature ships is intentionally minimal - it copies dims from `WorldConfig` into `Grid` on the first tick. Real grid mutation (`occupancy` updates from placement commands) arrives with a later feature that emits `CommandBus<PlaceTile>` for grid to drain.

**How it works:**

1. **`world_config` module (StaticData)** - owns `WorldConfig { width: u32, height: u32, seed: u64 }`. Installer calls `ctx.insert_resource(WorldConfig { .. })` with hardcoded defaults (64×64, fixed seed constant) for MVP. No startup system needed - `DataInstaller::finalize()` asserts only `writes` coverage.

2. **`grid` module (SimDomain, `PRIMARY_PHASE = Phase::World`)** - owns `Grid { width, height, occupancy: BTreeMap<(i32,i32), Entity>, dims_set: bool }`. `BTreeMap` (not `HashMap`) to guarantee deterministic iteration order across runs. Reads `WorldConfig`. Installer wires `write_resource::<Grid>()`, `read_resource::<WorldConfig>()`, `add_system(grid_bootstrap_system)`, `add_metric_publish(grid_metrics_system)`.

3. **`grid_bootstrap_system`** - runs on `Phase::World` every tick. Guard: `Local<bool>`. On first call: reads `WorldConfig`, sets `grid.width/height`, flips `grid.dims_set = true`, flips `Local<bool>`. Subsequent calls early-return.

4. **`grid_metrics_system`** - runs in `Phase::Metrics`. Publishes gauge `grid.occupancy_count = grid.occupancy.len()`. Always 0 in this feature; keeps the metric surface in place for future placement work.

5. **No messages, no commands in this feature.** Cross-module integration (placement, landscape, resources) happens in later features.

**Acceptance criteria:**

- AC1: After `Harness::new().with_data::<WorldConfigModule>().build()` and zero ticks, `World::resource::<WorldConfig>()` returns `WorldConfig { width: 64, height: 64, seed: <nonzero> }`.
- AC2: After `Harness::new().with_data::<WorldConfigModule>().with_sim::<GridModule>().build(); app.update();` the resource `Grid` has `dims_set == true`, `width == 64`, `height == 64`, `occupancy.is_empty() == true`.
- AC3: After two `app.update()` calls the `MetricsRegistry` gauge `grid.occupancy_count` equals `0.0` and has owner `"grid"`.
- AC4: Building `Harness::new().with_sim::<GridModule>().build()` (grid alone, without `WorldConfigModule`) panics at `finalize_modules()` with substring `"closed-reads"` - grid reads `WorldConfig` which has no writer.
- AC5: Two modules cannot both declare `writes: names![Grid]`. Adding a hypothetical second module that claims `Grid` writes panics at build with substring `"single-writer"`. (Negative test via a stub second module in the test file.)
- AC6: `Grid::occupancy` is a `BTreeMap`, not a `HashMap`. Determinism replay requires deterministic iteration; `std::collections::BTreeMap` guarantees it, `bevy::utils::HashMap` does not.
- AC7: `WorldConfig` is `Resource + Clone + Debug`. No interior mutability (`Mutex`/`RefCell`/`Atomic*`) - contract-visible types must be plain data.

**Non-goals:**
- Loading seed or dimensions from CLI, environment, save file, or RON config (post-MVP; hardcoded default is sufficient here).
- `Grid.occupancy` population (writes originate from a future placement feature via `CommandBus<PlaceTile>` drained in `Phase::Commands`; F13 ships an empty occupancy map and a stub bootstrap).
- Terrain data, resource veins, or any content that belongs to F14 (`world-generation`).
- Biome-specific or run-specific config variations (single biome, single seed for this feature).
- Grid queries or spatial API beyond the raw resource (lookups live where consumers need them).

**Edge cases:**
- Building the App without `GridModule` but with `WorldConfigModule`: no panic. `WorldConfig` exists, no `Grid`. Later features that need `Grid` must register `GridModule`.
- Registering `WorldConfigModule` twice via `app.add_data::<...>()`: panics at build with substring `"duplicate module id"` (registry invariant).
- Hardcoded seed value of `0`: `WorldConfig.seed == 0` must still be valid; downstream generators derive sub-seeds via `splitmix64(seed ^ salt)` which produces distinct non-zero streams even for zero input. F13 does not depend on seed value itself.
- `Grid.occupancy` inserted into before `dims_set == true`: this feature guarantees it cannot happen because the bootstrap system is the only writer in F13 and it only sets dimensions, never inserts entities. Downstream placement code must assert `dims_set` before inserting.
- `Grid::default()` produces `width == 0`, `height == 0`, `occupancy.is_empty()`, `dims_set == false`. Downstream systems that query `Grid` before the first `app.update()` will see zero dims; this is the intended pre-bootstrap state.
