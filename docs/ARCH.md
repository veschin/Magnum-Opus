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

## 13. Simulation-Render Boundary

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
