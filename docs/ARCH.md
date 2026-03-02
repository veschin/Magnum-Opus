# Magnum Opus — Architecture Principles

Stack-agnostic guideline. Open this file to understand HOW we build and test.

---

## 1. Simulation-First

The game IS a deterministic numerical simulation. Rendering is a separate read-only layer.

```
┌─────────────────────────────────┐
│     PRESENTATION (engine)       │  reads state, handles input/audio/VFX
│     zero game logic here        │
├─────────────────────────────────┤
│     SIMULATION (pure library)   │  ECS world, all systems, all math
│     zero engine dependencies    │  single entry point: World.tick(commands)
├─────────────────────────────────┤
│     STATIC DATA (files)         │  recipes, buildings, biomes, creatures
│     loaded at init, immutable   │
└─────────────────────────────────┘
```

**Rules:**
- Simulation has zero imports from any game engine
- Simulation runs headlessly — all tests verify behavior through numbers
- Rendering reads ECS state + event queue, never writes to simulation
- Render-only data (animations, particles, interpolation) lives in separate render components
- Full 90-minute run (108000 ticks) must complete headlessly in <5 seconds

---

## 2. ECS as Core Paradigm

Everything is entities, components, systems, and resources. No exceptions.

| Concept | What it is | Rule |
|---------|-----------|------|
| Entity | Integer ID | No behavior, no methods, just an ID |
| Component | Pure data struct | No logic, no methods beyond accessors. Attached to entities |
| System | Function | Reads/writes components via queries. One concern per system |
| Resource | Global singleton | Shared state not tied to a specific entity (energy pool, clock, DBs) |

**Anti-patterns (forbidden):**
- Inheritance hierarchies for game objects
- God-objects that combine data + logic
- Components with methods that mutate other components
- Systems that directly call other systems
- Hidden state outside of ECS (global variables, static mutable)

---

## 3. Command Sourcing

Player actions are serialized commands, never direct world mutations.

```
Player input → Command{type, params} → CommandBuffer (queue)
                                            ↓
                                    Phase 0: CommandProcessSystem
                                            ↓
                                    Validation → Mutation (or reject)
```

**Command types:**
- PlaceBuilding {position, buildingType}
- RemoveBuilding {entityID}
- DrawPath {fromGroup, toGroup, waypoints}
- DrawPipe {fromGroup, toGroup, waypoints}
- DestroyPath {entityID}
- SetGroupPriority {groupID, priority}
- PauseGroup / ResumeGroup {groupID}
- PlaceSacrifice {position, buildingType}
- ActivateMiniOpus {miniOpusID}
- ExtractNest {nestID}

**This enables:**
- Deterministic replay: same seed + same commands = identical simulation
- Undo: reverse command application
- Network sync: send command streams, not world state
- Testing: feed command sequences, assert world state

---

## 4. Phase-Ordered Tick Pipeline

All game logic runs in discrete ticks at fixed timestep. Systems execute in strict phase order.

```
Tick N:
  Phase 0: Input        → CommandProcess
  Phase 1: World        → Weather → Elements → Hazards
  Phase 2: Creatures    → Spawn → Behavior
  Phase 3: Energy       → Generation → Consumption → Distribution
  Phase 4: Production   → ProductionTick → Manifold → GroupStats
  Phase 5: Transport    → MinionCarry → PathFlow
  Phase 6: Combat       → CombatGroup → TerritoryControl → NestClearing
  Phase 7: Progression  → RateMonitor → MilestoneCheck → MiniOpus → TierGate
  Phase 8: Cleanup      → VeinDepletion → Destruction → GroupRecalc → RunLifecycle
  Phase 9: Meta         → [on run end only] CurrencyAward
```

**Why this order matters — data flows forward:**

```
Energy budget (phase 3)
    → Production speed modifier (phase 4)
        → Resources produced (phase 4)
            → Resources transported (phase 5)
                → Combat group inputs (phase 6)
                    → Organic outputs (phase 6)
                        → Rates measured (phase 7)
                            → Milestones checked (phase 7)
```

Reversing any two adjacent phases breaks the pipeline. Energy MUST precede production. Production MUST precede transport. Transport MUST precede combat (combat groups need delivered inputs).

**Tick constants (tunable, defined in config):**

| Constant | Purpose |
|----------|---------|
| TICK_RATE | Ticks per second (display speed) |
| TICK_DURATION | 1 / TICK_RATE seconds |
| RUN_DURATION_TICKS | Total ticks in a run |

All rates in the game are expressed as `units/tick` internally. UI converts to `units/min` for display.

---

## 5. Event Bus

Systems communicate through events, never through direct calls.

```
HazardSystem → emits BuildingDestroyed{entityID}
                        ↓
            BuildingDestructionSystem (phase 8) picks up event
                        ↓
            GroupRecalculationSystem (phase 8) reacts to group change
```

**Core events:**
- BuildingPlaced, BuildingDestroyed
- GroupFormed, GroupSplit, GroupMerged
- PathConnected, PathDisconnected
- HazardTriggered, SacrificeHit, SacrificeMiss
- CreatureKilled, TerritoryBreach
- MilestoneReached, TierUnlocked
- MiniOpusCompleted, MiniOpusMissed
- NestCleared, NestExtracted
- RunWon, RunTimeUp

**Rules:**
- Events are value types (immutable data)
- Events are consumed within the same tick (no cross-tick persistence)
- A system emits events into the EventQueue resource
- Consumer systems read events in a later phase
- Render layer also reads events for VFX triggers

---

## 6. Data-Driven Content

All game content is static data loaded from files. No content hardcoded in system logic.

| Database | Contains | Format |
|----------|----------|--------|
| RecipeDB | All recipes: inputs, outputs, duration, tier, quality requirements | data file |
| BuildingDB | All building types: recipe, terrain req, tier, energy demand, cost | data file |
| BiomeDB | All biomes: terrain distribution, creatures, hazards, weather, quality map | data file |
| CreatureDB | All species: archetype, stats, loot table, territory radius | data file |
| HazardDB | All hazard types: zone pattern, interval, intensity, enhancement type | data file |
| ElementRulesDB | Element interaction rules: fire+wind→spread, rain→fill, cold→freeze | data file |

**Adding new content:**
- New building = new entry in BuildingDB + new entry in RecipeDB
- New biome = new entry in BiomeDB (terrain, creatures, hazards, quality map)
- New creature = new entry in CreatureDB
- Zero code changes for any of the above

**Systems are generic.** ProductionTickSystem doesn't know what "iron ore" is. It knows: building has recipe, recipe has inputs/outputs, process recipe. The specific resources are data.

---

## 7. Two Resource Types

Buildings and resources follow fundamentally different logistics.

### Buildings (inventory items)
```
Mall group produces building → Inventory (global, not spatial)
Player picks from Inventory → PlaceBuilding command → entity on map
```
- Buildings exist as inventory counts until placed
- Mall is the only source of buildings (except starting kit)
- Inventory is a global resource, not tied to any group

### Resources (manifold items)
```
Extractor produces ore → Group Manifold (shared pool)
Manifold feeds Smelter → Smelter produces bars → Group Manifold
Path carries bars → Destination Group Manifold
```
- Resources NEVER enter inventory
- Resources exist only in: manifolds, input/output buffers, cargo on paths
- Resources are spatial — they have a location (inside a group or on a path)

**Why this separation:** Buildings are a meta-layer (what to construct), resources are the production layer (what flows through the factory). Mixing them would create circular dependencies (need resources to build, need buildings to produce resources, need buildings to transport resources...). The Inventory breaks the cycle.

---

## 8. Transport Hierarchy

Two transport modes, unlocked progressively.

### Mode 1: Minion Carry (available from start)
- Automatic: minions carry resources between nearby groups without player action
- Slow: low throughput, short range
- Fallback: activates when no path/pipe connects two groups
- Purpose: bootstrap the factory before first rune path is built

### Mode 2: Rune Paths + Pipes (player-placed)
- Rune paths: solid resources. Player draws route. Models roll along glowing runes.
- Pipes: liquid resources. Stone aqueducts with glowing liquid.
- Occupy map tiles. Routing around terrain/groups is the puzzle.

### Global Tier Upgrade
- 3 tiers: T1 (slow) → T2 (medium) → T3 (fast)
- Unlocking T2 auto-upgrades ALL existing paths and pipes globally
- No per-segment management. One unlock = everything improves.

```
Tier | Throughput | Speed   | Unlock
T1   | low        | slow    | start of run
T2   | medium     | medium  | T2 tier gate
T3   | high       | fast    | T3 tier gate
```

---

## 9. Fog of War

Map starts hidden. Information is earned.

- Watchtower buildings reveal cells in radius (like Factorio radar)
- Fog state is per-tile: hidden / revealed / currently-visible
- Simulation runs regardless of visibility — creatures move, hazards tick, weather changes
- Player cannot place buildings on hidden tiles (must reveal first)
- Opus tree is always visible (it's abstract, not spatial)

---

## 10. Quality = Biome Context

Resource quality is not an intrinsic property of the resource. It depends on the biome.

```
BiomeDB["forest"].qualityMap = {
    wood: HIGH,
    rotten_wood: NORMAL,
    iron_ore: NORMAL,
    crystal: NORMAL
}

BiomeDB["undead"].qualityMap = {
    rotten_wood: HIGH,     // rotten wood is the premium resource here
    bone: HIGH,
    iron_ore: NORMAL,
    wood: UNAVAILABLE       // no living wood in undead biome
}
```

- Recipes specify quality requirements: "requires HIGH quality wood"
- Same recipe in different biomes needs different source resources
- This creates biome-specific strategy without separate quality mechanics
- 2 levels only: NORMAL and HIGH (simplified from earlier 3-level design)

---

## 11. Critical Invariants

Rules that must ALWAYS hold. Tested after every tick in debug mode.

1. **Conservation:** resources are never created except by recipes, veins, creature loot, and starting kit. Never destroyed except by consumption and hazards. `total_produced - total_consumed = total_existing` always.
2. **Grid alignment:** every entity position is integer. No fractional positions in simulation.
3. **Determinism:** same seed + same command sequence = identical tick-by-tick state.
4. **Group connectivity:** every building in a group is reachable from every other building via cardinal adjacency. No disconnected members.
5. **Single group membership:** a building belongs to exactly one group. Never zero, never two.
6. **Transport exclusivity:** a tile has at most one path OR one pipe, never both.
7. **Tier monotonicity:** tiers only increase within a run, never decrease.
8. **Energy non-negative:** allocated energy >= 0 for every group.
9. **Organic exclusivity:** organic resources have zero terrain vein sources. Only obtainable from combat/breeding groups.
10. **Milestone persistence:** once a milestone is sustained, it stays completed regardless of later rate drops.
11. **Inventory integrity:** buildings in inventory are non-negative integers. Placement decrements, production increments. Never negative.

---

## 12. Testing Strategy

Every system is testable in isolation with pure numbers. No rendering required.

### Test levels

| Level | What | Example |
|-------|------|---------|
| Unit | Single system, known inputs → assert outputs | Place building → entity exists at position |
| Integration | 2-3 systems chained | Production → Manifold → Transport: ore reaches smelter |
| Determinism | Full sim with seed, hash state | Run 1000 ticks twice → hashes match |
| Invariant | Assert all 11 invariants after every tick | Resource conservation holds after hazard destroys buildings |
| Scenario | Player-like sequence of commands | "Build miners + smelter + path" → iron bars flow after N ticks |
| Benchmark | Full run headlessly | 108000 ticks completes in <5 seconds |

### How to write a test

```
1. Create empty ECS world
2. Add only the components/resources needed for this test
3. Run only the system(s) under test
4. Assert component/resource values changed as expected
```

No mocks for internal code. Real systems, real components, real math.
External dependencies (file I/O, rendering, network) are behind interfaces and never used in tests.

---

## 13. Phase Ordering Invariants

This section documents the exact phase execution order, cross-plugin constraints, resource ownership
rules, and event flow guarantees. Three ordering bugs were found during ecs-engine integration
(SimTick double-write, UX reading stale data, nest_clearing before combat_pressure). These invariants
prevent that class of bugs from recurring.

### 13.1 Full Phase Execution Order

The `Phase` enum defines 9 phases executed strictly in order each tick:

```
Phase::Input       → Phase::Groups → Phase::Power    → Phase::Production
→ Phase::Manifold  → Phase::Transport               → Phase::Progression
→ Phase::Creatures → Phase::World
```

`Phase::Creatures` and `Phase::World` are NOT part of `SimulationPlugin`. They belong to separate
optional plugins layered on top.

#### SimulationPlugin (always present)

Registers and orders the core phases via `configure_sets`:

```
Input → Groups → Power → Production → Manifold → Transport → Progression
```

After `Phase::Progression`, the UX systems run unbounded (`.after(Phase::Progression)`):

```
Progression → [tick_system, dashboard_system, chain_visualizer_system]  (no Phase set)
```

#### CreaturesPlugin (optional, added on top of SimulationPlugin)

Adds one additional ordering constraint:

```
Transport → Creatures
```

Systems in `Phase::Creatures` (with intra-phase ordering):

```
combat_pressure_system          ─┐
combat_group_system              ├─ unordered among themselves
creature_behavior_system         │
invasive_expansion_system        │
creature_loot_system             │
minion_task_system              ─┘
nest_clearing_system              .after(combat_pressure_system)
```

`nest_clearing_system` is explicitly `.after(combat_pressure_system)` — pressure must accumulate
before the clearing check runs. This was one of the ordering bugs found in integration testing.

#### WorldPlugin (independent, does NOT share phases with SimulationPlugin)

`WorldPlugin` does NOT call `configure_sets`. All its systems run in `Update` as a single `.chain()`:

```
tick_advance_system → hazard_warning_system → hazard_trigger_system
→ element_interaction_system → weather_tick_system → fog_of_war_system
→ world_placement_system
```

`WorldPlugin` uses `Update` directly — it does not participate in the `Phase` enum ordering.
When used alone (world BDD tests), it is a standalone pipeline.
When used together with `SimulationPlugin`, these systems run in `Update` alongside the Phase sets,
and Bevy does not guarantee their position relative to Phase systems.

#### Full per-tick execution timeline (when all three plugins present)

```
1. Phase::Input        placement_system
2. Phase::Groups       group_formation_system, group_priority_system, group_pause_system
3. Phase::Power        energy_system
4. Phase::Production   production_system
5. Phase::Manifold     manifold_system → production_rates_system, trading_system
6. Phase::Transport    transport_destroy_system → transport_placement_system,
                       transport_tier_upgrade_system → transport_movement_system
7. Phase::Progression  tick_increment_system → milestone_check_system
                       → opus_tree_sync_system → run_lifecycle_system
                       tier_gate_system → building_tier_upgrade_system
                       mini_opus_system  (unordered relative to tier_gate chain)
                       NOTE: tier_gate_system has no .after(tick_increment_system) constraint;
                       it is unordered relative to tick_increment_system within Phase::Progression.
8. Phase::Creatures    combat_pressure_system, combat_group_system,
                       creature_behavior_system, invasive_expansion_system,
                       creature_loot_system, minion_task_system
                       nest_clearing_system  [after combat_pressure_system]
9. [after Progression] tick_system, dashboard_system, chain_visualizer_system
X. WorldPlugin chain   (Update, no Phase — runs at indeterminate position relative to above)
```

### 13.2 Resource Ownership (Single-Writer-Per-Tick Rule)

Each resource has exactly one system that writes it per tick. Reading it from another system is safe
only if that system runs in a later phase.

| Resource | Writer System | Phase | Notes |
|---|---|---|---|
| `Grid` | `placement_system` | Input | Read-only after Input phase |
| `Inventory` | `placement_system`, `production_system` | Input / Production | **Designed exception — see note below** |
| `EnergyPool` | `energy_system` | Power | Written once; read by dashboard (after Progression) |
| `Manifold` (component) | `manifold_system`, `trading_system`, `combat_group_system` | Manifold / Creatures | **Designed exception — see note below** |
| `ProductionRates` | `production_rates_system` | Manifold | Read by `milestone_check_system` (Phase::Progression) |
| `FogMap` | no system writer | — | Read-only at runtime; only `placement_system` reads it for fog checks. Cells are revealed via direct resource mutation outside the tick pipeline (test helpers, starting-kit setup). `fog_of_war_system` (WorldPlugin) writes `WorldTile.visibility` components, NOT this resource. |
| `PathOccupancy` | `transport_placement_system`, `transport_destroy_system` | Transport | Destroy runs before placement within the phase |
| `LastDrawPathResult` | `transport_placement_system` | Transport | |
| `TransportCommands` | drained by `transport_placement_system` / `transport_destroy_system` | Transport | |
| `OpusTreeResource` | `opus_tree_sync_system` | Progression | Read by `run_lifecycle_system` (same phase, runs after) |
| `RunConfig.current_tick` | `tick_increment_system` | Progression | |
| `RunState` | `run_lifecycle_system` | Progression | |
| `TierState` | `tier_gate_system`, `nest_clearing_system` | Progression / Creatures | Two writers in different phases; Creatures runs after Progression — no same-tick conflict for `tier_gate_system` write; `nest_clearing_system` write is tick N, read by Progression systems in tick N+1 (see event table note) |
| `SimTick` | `tick_increment_system` (when `RunConfig` present) | Progression | `tick_advance_system` (WorldPlugin) writes it only when `RunConfig` is absent (guard in source) |
| `SimulationTick` | `tick_system` (UX) | after Progression | Separate resource from `SimTick` |
| `DashboardState` | `dashboard_system` (UX) | after Progression | Reads final-state of all other resources |
| `ChainVisualizerState` | `chain_visualizer_system` (UX) | after Progression | Read-only snapshot of groups/paths |
| `CurrentWeather` | `weather_tick_system` | WorldPlugin chain | |
| `WorldMap` | `hazard_trigger_system`, `element_interaction_system`, `world_placement_system` | WorldPlugin chain | chained — no conflicts |

**SimTick ownership note:** `SimTick` has a conditional dual-writer guard. `tick_advance_system`
checks `Option<Res<RunConfig>>` and only writes `SimTick` when `RunConfig` is absent. When
`SimulationPlugin` is active, `tick_increment_system` owns `SimTick` exclusively. This prevents the
double-write bug found during integration.

**Inventory dual-writer exception:** `placement_system` (Phase::Input) decrements `Inventory` when
consuming a building from stock, and `production_system` (Phase::Production) increments it for
Mall-type buildings (`output_to_inventory = true`). This is a designed exception to the
single-writer rule. It is safe because Input always runs before Production within a tick — the
decrement is fully committed before any increment can occur. Inventory integrity (invariant 11) is
preserved: the net count never goes negative within a tick.

**Manifold multi-writer exception:** Three systems write `Manifold` components across two phases.
`manifold_system` (Phase::Manifold) performs the primary balancing pass. `trading_system`
(Phase::Manifold, runs after `manifold_system`) drains manifold resources into meta-currency.
`combat_group_system` (Phase::Creatures) consumes herbs from the manifold and deposits organic
output. This is a designed exception: the three writers operate on distinct aspects (balance,
drain-to-currency, combat I/O) and run in a fixed order guaranteed by phase sequencing and intra-phase
`.after()` constraints. No two writers race on the same resource entry in the same phase.

### 13.3 Event Emission and Consumption Phase Mapping

Events are value types emitted once and consumed in a later phase within the same tick. Cross-tick
event persistence does NOT occur.

| Event | Emitter | Emitter Phase | Consumer | Consumer Phase |
|---|---|---|---|---|
| `BuildingPlaced` | `placement_system` | Input | `group_formation_system` | Groups |
| `BuildingRemoved` | not yet implemented in any system (†) | — | `group_formation_system` | Groups |
| `BuildingDestroyed` | `hazard_trigger_system` (WorldPlugin) | WorldPlugin (‡) | `group_formation_system` | Groups |
| `SetGroupPriority` | external command / test | — | `group_priority_system` | Groups |
| `PauseGroup` | external command / test | — | `group_pause_system` | Groups |
| `ResumeGroup` | external command / test | — | `group_pause_system` | Groups |
| `PathConnected` | `transport_placement_system` | Transport | render layer / tests | — |
| `PathDisconnected` | `transport_destroy_system` | Transport | render layer / tests | — |
| `TierUnlocked` | `transport_tier_upgrade_system` | Transport | render layer / tests | — |
| `NestCleared` | `nest_clearing_system` | Creatures | `tier_gate_system` | Progression (*) |
| `TierUnlockedProgression` | `tier_gate_system` | Progression | `building_tier_upgrade_system` | Progression |
| `TierUnlockedProgression` | `nest_clearing_system` | Creatures | `building_tier_upgrade_system` | Progression (*) |
| `MilestoneReached` | `milestone_check_system` | Progression | `opus_tree_sync_system` (indirectly via component state) | Progression |
| `MiniOpusCompleted` | `mini_opus_system` | Progression | render layer / scoring | — |
| `MiniOpusMissed` | `mini_opus_system` | Progression | render layer / scoring | — |
| `RunWon` | `run_lifecycle_system` | Progression | render layer / scoring | — |
| `RunTimeUp` | `run_lifecycle_system` | Progression | render layer / scoring | — |
| `RunAbandoned` | `run_lifecycle_system` | Progression | render layer / scoring | — |
| `SacrificeHit` | `hazard_trigger_system` | WorldPlugin | render layer | — |
| `SacrificeMiss` | `hazard_trigger_system` | WorldPlugin | render layer | — |
| `PlacementRejected` | `world_placement_system` | WorldPlugin | render layer / tests | — |
| `HazardTriggered` | `hazard_trigger_system` | WorldPlugin | render layer / tests | — |

(†) **BuildingRemoved emitter not implemented:** No system currently emits `BuildingRemoved`. The
event type is registered and `group_formation_system` listens for it, but the removal path
(player-initiated building removal) does not yet have a system that writes this event. Tests
inject it directly via `world.write_message()`. When a removal system is added, it must run in
Phase::Input so the event is consumed by `group_formation_system` in the same tick.

(*) **Cross-plugin tick-boundary note (Creatures → Progression):** Both `NestCleared` and
`TierUnlockedProgression` emitted by `nest_clearing_system` in `Phase::Creatures` share the same
tick-boundary constraint: their consumers (`tier_gate_system` and `building_tier_upgrade_system`)
run in `Phase::Progression`, which executes BEFORE `Phase::Creatures` in each tick. Events emitted
in `Phase::Creatures` at tick N are therefore consumed by Progression systems in tick N+1, not the
same tick. This is the intended design: nest clearing causes tier advancement one tick later.

(‡) **BuildingDestroyed cross-plugin tick-boundary note:** `BuildingDestroyed` is emitted by
`hazard_trigger_system` in the WorldPlugin chain, which runs at an indeterminate position relative
to the Phase-ordered pipeline (see section 13.1). When the WorldPlugin chain runs after
`Phase::Groups` has already executed, `group_formation_system` (the consumer, in Phase::Groups)
will not process the event until tick N+1. This is accepted design: hazard-triggered group splits
take one tick to propagate.

### 13.4 Additions to Critical Invariants

The following invariants extend section 11 with phase-ordering guarantees:

12. **Single-writer-per-tick:** every resource has at most one system writing it per tick, with two
    designed exceptions. (a) `Inventory`: written by both `placement_system` (Phase::Input,
    decrement) and `production_system` (Phase::Production, increment); safe because Input precedes
    Production in every tick. (b) `Manifold`: written by `manifold_system`, `trading_system`, and
    `combat_group_system` across two phases; safe because each writer operates on a distinct aspect
    and the phase order is fixed. All other resources must have a single writer. See the ownership
    table in section 13.2.
13. **Event ordering guarantee (Phase-ordered pipeline only):** within the Phase-ordered pipeline
    (SimulationPlugin + CreaturesPlugin), an event emitted in phase X is never consumed in the same
    tick by a system in phase Y where Y <= X. Events flow strictly forward through phases within a
    tick; there is no backward event delivery. This guarantee does NOT extend to WorldPlugin
    systems, which run in `Update` outside the Phase ordering and may emit or consume events at any
    position relative to the Phase pipeline. Cross-plugin event flows (e.g., `BuildingDestroyed`
    from WorldPlugin consumed by Phase::Groups) are subject to tick N+1 latency as documented in
    the event table above.
14. **UX systems are terminal readers:** `tick_system`, `dashboard_system`, and
    `chain_visualizer_system` run after all Phase sets. They read final-state resources and never
    write resources that earlier systems depend on. Adding a UX system that reads mid-tick state
    (e.g., after Phase::Power but before Phase::Production) is forbidden.
15. **SimTick single ownership:** `SimTick` is written by `tick_increment_system`
    (`Phase::Progression`) when `SimulationPlugin` is active. `tick_advance_system`
    (`WorldPlugin`) contains an explicit guard that skips the write when `RunConfig` is present.
    This invariant must be preserved when adding new tick-counting systems.

---

## 14. Simulation-Render Boundary

Explicit contract between simulation and presentation layers.

### Simulation provides (read-only for renderer):
- All entity positions and component state
- Event queue (for VFX triggers)
- GroupStats (for UI overlays)
- OpusTree state (for progression UI)
- EnergyPool state (for energy bar)
- FogOfWar state (for visibility overlay)

### Renderer provides (to simulation):
- Nothing. Renderer never writes to ECS.
- Player input → Command objects → CommandBuffer. That's the only feedback path.

### Interpolation
- Simulation runs at TICK_RATE (fixed, could be 20/s)
- Rendering runs at display refresh rate (60fps, 144fps, whatever)
- Renderer interpolates between tick N and tick N+1 for smooth visuals
- Interpolation is render-only math, never affects simulation state
