# Magnum Opus — Product Requirements Document

## Overview

Magnum Opus is a roguelike factory game with 1–2 hour runs and meta-progression between runs. The player is a spirit commanding biome-native faceless minions to build production chains, manage resources, and complete a randomly generated tech-tree goal (the Opus) before the run ends.

### What the game IS

The Factorio early-game production loop — finding bottlenecks, building chains, optimizing throughput — transplanted into a fantasy roguelike setting. Every run presents a different biome, different Opus goal, and different events. The player wins through production management, not combat micromanagement.

### What the game is NOT

- Not a city builder (no population, no happiness, no zoning)
- Not a tower defense (combat is passive — defense buildings are production chains)
- Not a sandbox (runs have a clear goal and time pressure)
- Not a management sim (no spreadsheet UI — visual feedback everywhere)

### Core references

- **Factorio**: production chains, belt logistics, throughput optimization
- **Against the Storm**: roguelike run structure, event system, biome variety
- **Breath of the Wild**: interconnected systemic world (weather, elements, creatures)

### Tech direction

- ECS architecture from day one
- 3D isometric with pixel-art post-processing shaders
- Fixed camera, god-view perspective

---

## Features

<!-- feature:building-groups -->
### F1: Building Groups

The universal production mechanic. All factory gameplay operates through building groups.

**Summary:** Adjacent buildings automatically form a group. Inside a group, resources distribute via manifold (automatic, no player-placed logistics). Between groups, the player routes rune paths and pipes manually.

**Problem:** Factory games require complex internal logistics (belts between every building). For 1–2h runs this is too much friction. Building Groups eliminate internal belt management while preserving the inter-cluster logistics puzzle.

**How it works:**
- Place buildings adjacent to each other → they form a group
- Identical buildings chain freely (horizontal, vertical, grid): 4 miners in a square = one powerful block
- Different buildings in a group need one adjacency connection; system auto-optimizes internal flow
- Internal connections rendered as mini rune paths (visually consistent with external transport)
- Each group has configurable input receivers and output senders
- Groups are the unit of management in chain manager (energy allocation, priority, pause/resume)
- Buildings merge visually into factory structures as the group grows

**Group types by purpose (same mechanic, different composition):**
- **Extraction group**: miners/collectors on natural veins — extract raw resources from terrain
- **Synthesis group**: farms/labs/refineries placed anywhere — convert base resources into other base resources (e.g. tree farm: water input → wood output). Used when biome lacks a resource naturally.
- **Mall group**: constructor + toolmaker + assembler — produces buildings, tools, utility. Safe default destination for all resources.
- **Combat group**: imp camps + armory — consumes weapons/food, produces territory protection + organic resources. The PRIMARY way to obtain organics (grow or kill = same pipeline).
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

**Summary:** The player connects building groups by placing rune paths (solid resources — models roll along glowing runes) and pipes (liquids — magical aqueducts). Tier upgrades are global: unlocking T2 auto-upgrades all existing paths.

**Problem:** Logistics between production clusters is the core spatial puzzle. The transport system must be visually satisfying (resources visibly moving), mechanically clear (throughput limits, routing), and low-friction (global tier upgrades, no per-segment management).

**How it works:**
- Rune paths: solid resources. Player draws path from group output to group input. Resource models drop onto path and roll. Throughput = path tier capacity.
- Pipes/channels: liquid resources. Separate visual system (stone aqueducts with glowing liquid). Same routing mechanic as paths.
- Tiers: T1 (basic, slow) → T2 (medium) → T3 (fast, wide). Unlocking a new tier globally upgrades ALL existing paths and pipes automatically.
- Paths and pipes occupy map tiles. Routing around terrain and other groups is the puzzle.

**Acceptance criteria:**
- AC1: Player can draw rune path from output sender of group A to input receiver of group B
- AC2: Solid resource models visibly roll along rune paths at tier-appropriate speed
- AC3: Liquid resources visibly flow through pipes at tier-appropriate speed
- AC4: Unlocking T2 upgrades all T1 paths and pipes globally without player action
- AC5: Path throughput is capped by tier; excess resources queue at sender
- AC6: Paths and pipes cannot overlap on the same tile (must route around)
- AC7: Destroying a path segment disconnects the route; resources stop flowing

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

**Problem:** The world must create varied, interesting constraints for each run. Same Opus in different biomes should require different strategies. The world runs independently — things happen whether the player is watching or not.

**How it works:**
- Biomes: volcanic, forest, ocean, desert, etc. Each has unique terrain, resources, creatures, hazards, and buildings.
- ~30% universal buildings (work everywhere), ~70% biome-specific.
- Landscape constrains building placement: miners only on ore veins, lava siphons only near lava, etc. This naturally limits group composition.
- Hazards: biome-specific (eruptions, storms, wildfires, sandstorms). Predictable zone and timing. Destroy buildings but enhance affected tiles.
- Sacrifice mechanic: place sacrifice buildings in hazard zones. Player sees odds (e.g. 70% bonus / 30% miss). Hit = tile enhanced + bonus. Miss = building lost.
- Systemic interactions: fire + wind = wildfire spread. Rain fills water. Cold freezes water. All affect production.

**Acceptance criteria:**
- AC1: Map generation produces biome-specific terrain with resource veins, liquid sources, and hazard zones
- AC2: Buildings with landscape requirements can only be placed on matching terrain tiles
- AC3: Hazard events announce zone and timing N seconds in advance
- AC4: Sacrifice building placed in hazard zone shows probability of bonus vs miss
- AC5: Hazard destroying a tile applies the enhancement property to that tile
- AC6: At least 3 systemic element interactions are functional (fire+wind, rain+soil, cold+water)
- AC7: World simulation runs independently of player camera position

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

**Summary:** Creatures are multi-role entities with 5 behavior archetypes. Combat groups (imp camps) are the PRIMARY way to obtain organic resources — growing and killing are the same production pipeline. This makes combat groups mandatory in every run, not an optional defense layer.

**Problem:** Organic resources (wood, hides, herbs, creature parts) cannot be mined from terrain. They must come from creatures — either by farming/breeding or by hunting/killing. The combat system must integrate into the production loop as another resource pipeline, not a separate minigame.

**How it works:**
- Behavior archetypes:
  - Ambient: live independently, resource for hunting/breeding/harvesting
  - Territorial: attack when player expands into their zone
  - Invasive: expand autonomously, reclaim player territory if unchecked
  - Event-born: spawn from Opus tree branch events
  - Opus-linked: tied to main opus, appear at key progression thresholds
- Combat groups = organic resource pipeline:
  - Imp camp consumes weapons + food → produces territory protection + organic loot
  - Breeding pen consumes food + water → produces renewable organic materials
  - Growing and killing are equivalent production paths to organics
  - T3 combat groups clear enemy zones for rare resources needed by Opus milestones
  - Combat groups scale like any resource group: more buildings = more throughput
- Minions: biome-native faceless workers. Different stat distributions per biome. No names, no personality.
- Idle minions auto-decorate buildings (player picks style). Beauty = emergent indicator of factory efficiency.
- Creature trading: biome-abundant creatures can be traded for resources the biome lacks.

**Acceptance criteria:**
- AC1: Each biome spawns creatures with at least 3 of the 5 archetypes
- AC2: Territorial creatures attack when player builds within their territory radius
- AC3: Invasive creatures expand their territory over time if unchecked
- AC4: Combat group (imp camp) consumes input resources and produces organic output + protection
- AC5: Under-supplied combat group loses effectiveness; enemies break through, organic output drops
- AC6: T3 combat group can clear an enemy zone and drop rare resources
- AC7: Organic resources are ONLY obtainable through combat/breeding groups (no terrain extraction)
- AC8: Idle minions with no tasks auto-decorate nearby buildings
- AC9: Minion decoration activity ceases when all minions are assigned to tasks

**Non-goals:**
- Player-controlled combat (real-time unit selection, attack commands)
- Named/unique creatures with personality
- Creature evolution or genetics system
- Active player combat abilities
- Organics from terrain extraction (explicitly excluded — must come from creatures)

**Edge cases:**
- Combat group with no input supply: minions idle, no protection, no organic output
- All creatures in a zone killed: zone is safe but no renewable creature resources — player must find new zone or breed
- Invasive creatures reach a building group: they damage output senders first, disrupting logistics
- No combat group built: player has no access to organics, blocking recipes that require them

---

<!-- feature:progression -->
### F5: Progression

In-run progression through the Opus tree (production milestones + mini-opus branches), tier unlocks, and encounter-gated mechanics.

**Summary:** The Opus tree is a unified progression structure. Main path nodes are **production throughput milestones** ("produce X resource at Y/min"). Side branches are **mini-opus events** (challenges that award bonuses and meta-currency). One tree, one goal, everything inside it.

**Problem:** A 1–2h run needs clear pacing and a legible win condition. The player should look at the Opus tree and immediately understand what production chains to build. No abstract artifacts — every node is a measurable throughput goal.

**How it works:**
- **Opus tree structure:**
  - Main path nodes = production milestones: "produce N of resource X per minute"
  - Side branches = mini-opus: optional challenges that give bonuses + meta-currency
  - Final node = sustain all main-path rates simultaneously for a duration
  - Tree is visible from run start — player can plan entire strategy upfront
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
- **3 Tiers (encounter-gated):**
  - T1 (setup, ~25 min): basic extraction, simple recipes, T1 rune paths, first Mall group
  - T2 (expansion, ~35 min): complex recipes, biome buildings, pipes, T2 paths
  - T3 (opus push, ~30 min): final recipes, T3 paths, opus groups
  - Encounters on the map unlock next tier when completed

**Acceptance criteria:**
- AC1: Opus tree nodes are production throughput milestones (resource + rate), not item crafting goals
- AC2: Milestone is marked complete when player sustains the required rate for a verification period
- AC3: Opus tree UI shows all main nodes + side branches, current rates vs required, completion %
- AC4: Mini-opus branches are visually attached to their parent main-path node
- AC5: Completing a mini-opus branch awards meta-currency; skipping it has no main-path penalty
- AC6: Final Opus node requires simultaneous sustain of all main-path rates
- AC7: T2 buildings/recipes are inaccessible until T1 encounter is completed
- AC8: T3 buildings/recipes are inaccessible until T2 encounter is completed
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

**Acceptance criteria:**
- AC1: Run-end screen shows earned currencies with Opus multiplier applied
- AC2: Meta store displays available unlocks with currency costs
- AC3: Unlocked content persists across runs
- AC4: Opus multiplier is determined by biome-opus difficulty match
- AC5: Player can view total lifetime currency earnings and spending

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
- Production calculator: "I need X items/min" → shows required buildings, resources, groups
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
