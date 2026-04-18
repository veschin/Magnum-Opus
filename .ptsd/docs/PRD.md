# Magnum Opus - Product Requirements Document

Greenfield rewrite v2. The v1 implementation was deleted on 2026-04-17; its PRD survives as `PRD_legacy_v1.md` for design-decision archaeology (opus tree, combat-as-organics pipeline, hazard/sacrifice mechanic). This document describes the features we will actually build, aligned with the current core architecture.

## §1 Architecture recap

Every feature below implements one or more **modules** on top of the core system (`magnum_opus/src/core/`). The core enforces 18 invariants at App-build time; features inherit those guarantees.

**Four archetypes, four scoped installers:**

| Archetype | Schedule | Installer | Purpose |
|---|---|---|---|
| `SimDomain` | `Update` (phased) | `SimInstaller` | Owns sim state; mutates per tick. |
| `StaticData` | `Startup` | `DataInstaller` | Loads read-only reference data. |
| `View` | `PostUpdate` | `ViewInstaller` | Read-only projection of sim; owns view-private state. |
| `InputUI` | `PreUpdate` | `InputInstaller` | Reads input + sim; pushes commands. |

**Phase pipeline (SimDomain):**

```
Commands -> World -> Placement -> Groups -> Power -> Production
  -> Manifold -> Transport -> Progression -> Metrics -> End
```

`Commands`, `Metrics`, `End` are reserved - modules cannot use them as `PRIMARY_PHASE`. Use `add_command_drain` / `add_metric_publish` to attach secondary systems into the reserved phases.

**Contract invariants (enforced at `finalize_modules`):**

- Single-writer per resource (per-type, cross-module).
- Closed-reads: every read has a declared writer.
- Closed-messages: every message consumer has a producer.
- Single-producer messages, single-consumer commands.
- Installer coverage: every declared contract slot must be exercised by the matching `ctx.xxx::<T>()` call.
- Reserved phases rejected as `PRIMARY_PHASE`.
- Late registration after `finalize_modules()` panics.
- No interior mutability on contract-visible types (review rule, not runtime check).

**Identifiers:** contract slots list `TypeKey` values from the `names![T, U]` macro. Identity is by `TypeId`, name is diagnostic-only - `a::Grid` and `b::Grid` never collide.

Full spec: `docs/llm/20_contracts.md`. Developer workflow: `.claude/skills/core-module/SKILL.md`.

---

## §2 Feature taxonomy

Features are units of product scope. Each feature ships one or more modules. Dependencies flow upward: a later feature reads resources that an earlier feature writes. Status values: `locked` (not started), `in-progress`, `done`.

### Phase 1 - Spatial foundation

| ID | Title | Archetypes | Status | Depends on |
|---|---|---|---|---|
| **F1** | `world-foundation` | StaticData + SimDomain | in-progress | - |
| **F2** | `world-generation` | SimDomain × 2 | locked | F1 |

### Phase 2 - Core loop (locked, not yet detailed)

| ID | Title | Archetypes | Notes |
|---|---|---|---|
| F3 | `placement` | InputUI + SimDomain-drain | Player places buildings from Inventory onto grid. Emits `CommandBus<PlaceTile>`; grid drains in `Phase::Commands`. |
| F4 | `buildings` | StaticData + SimDomain | `BuildingDB`, `Building` component lifecycle, entity spawn on placement. |
| F5 | `recipes` | StaticData + SimDomain | `RecipeDB`, `Recipe` + `ProductionState`, tick advancement (`Phase::Production`). |
| F6 | `manifold` | SimDomain | Per-group resource pool, collect-then-distribute (`Phase::Manifold`). |
| F7 | `group-formation` | SimDomain | Flood-fill connected components from adjacency (`Phase::Groups`). |

### Phase 3 - Extended systems (locked, taxonomy only)

| ID | Title | Purpose |
|---|---|---|
| F8 | `energy` | Power pool + per-group allocation. |
| F9 | `transport` | Rune paths + pipes between groups. |
| F10 | `creatures` | AI archetypes, ambient / territorial / invasive behavior. |
| F11 | `combat-groups` | Organic resource pipeline via imp camps. |
| F12 | `weather-hazards` | Elemental interactions, hazard zones, sacrifice mechanic. |
| F13 | `fog-of-war` | `FogMap`, watchtower reveal radius. |
| F14 | `opus-tree` | Main-path milestones + mini-opus branches. |
| F15 | `tier-gates` | Nest-clearing unlocks T2/T3. |
| F16 | `run-lifecycle` | Win/loss/abandon, scoring. |
| F17 | `meta-currency` | Gold / Souls / Knowledge persistence, inter-run unlocks. |

### Phase 4 - Presentation (locked, taxonomy only)

Four features total. The pixel-art look comes from `render-pipeline` (post-processing), not from the sprites themselves - 3D-style impostors pass through the pipeline.

| ID | Title | Archetype | Purpose |
|---|---|---|---|
| F18 | `render-pipeline` | View | Low-res render target (480×270), Sobel outline, toon shading, posterization, nearest-neighbor upscale. The visual-identity layer; every render feature draws into its target. |
| F19 | `world-render` | View | Terrain tile quads from `Landscape.cells`, fog overlay from `FogMap`, vein markers from `ResourceVeins`. |
| F20 | `model-render` | View | Impostor sprites (albedo + normal + depth maps) for buildings, creatures, cargo. Per-pixel lighting read from normal/depth channels. |
| F21 | `camera-ui` | InputUI + View | Orthographic camera pan/zoom, cursor->grid raycasting, build menu, inventory, opus panel, minimap, tooltips, notifications. Emits `PlaceTile` / `RemoveBuilding` commands. |

**Dependency order:** `render-pipeline` first (owns the target + post-processing), `world-render` and `model-render` in parallel (both draw into the target), `camera-ui` last (UI overlay in screen-space, after upscale). Detailed plan: `~/.claude/plans/render-roadmap.md`.

### Legacy features (archived - do not extend)

`building-groups`, `transport`, `world`, `creatures`, `progression`, `meta`, `energy`, `ux`, `ecs-engine`, `game-startup`, `game-render`, `game-ui` - registered against v1 PRD. These predate the core rewrite and may overlap with Phase 2-4 above. Remove from active PTSD registry before starting Phase 2; refer to `PRD_legacy_v1.md` for their original acceptance criteria.

---

<!-- feature:world-foundation -->
### F1: world-foundation

**Purpose:** Provide `WorldConfig` and `Grid` resources as the shared spatial substrate. Every later feature that touches coordinates or the run seed depends on this.

**Problem:** Downstream features (landscape generation, placement, rendering) need two pieces of state before they can run: a deterministic seed + world dimensions, and a grid resource whose writer is already claimed under single-writer discipline. If each feature fabricated its own seed, cross-module generators would produce inconsistent state from the same "run." If each feature also tried to write `Grid`, single-writer would fire on the second registration. F1 settles both: `world_config` (StaticData) writes `WorldConfig`; `grid` (SimDomain, `Phase::World`) writes `Grid` forever. Grid mutation from placement commands arrives later via `add_command_drain`, preserving single-writer.

**Modules:**

1. **`world_config` (StaticData)**
   - Writes: `WorldConfig { width: u32, height: u32, seed: u64 }`
   - Installer: `ctx.insert_resource(WorldConfig { width: 64, height: 64, seed: 0x9E3779B97F4A7C15 })`. No startup system needed - `DataInstaller::finalize()` checks only `writes` coverage.
   - Metrics: none.

2. **`grid` (SimDomain, `PRIMARY_PHASE = Phase::World`)**
   - Writes: `Grid { width: u32, height: u32, occupancy: BTreeMap<(u32,u32), Entity>, dims_set: bool }`
   - Reads: `WorldConfig`
   - Installer: `write_resource::<Grid>()`, `read_resource::<WorldConfig>()`, `add_system(grid_bootstrap_system)`, `add_metric_publish(grid_metrics_system)`.
   - `grid_bootstrap_system`: `Local<bool>` guard; on first call reads `WorldConfig`, sets `grid.width/height`, flips `grid.dims_set = true`.
   - `grid_metrics_system`: publishes gauge `grid.occupancy_count = grid.occupancy.len() as f64`. Always 0 in F1.

**Acceptance criteria:**

- AC1: After `Harness::new().with_data::<WorldConfigModule>().build()` (zero ticks), `world.resource::<WorldConfig>()` returns `WorldConfig { width: 64, height: 64, seed: 0x9E3779B97F4A7C15 }`. Seed value is locked to the constant in the implementation; tests compare against that exact constant.
- AC2: After `Harness::new().with_data::<WorldConfigModule>().with_sim::<GridModule>().build(); app.update();` the resource `Grid` has `dims_set == true`, `width == 64`, `height == 64`, `occupancy.is_empty() == true`.
- AC3: After two `app.update()` calls, `MetricsRegistry` exposes gauge `"grid.occupancy_count"` with value `0.0` and owner `"grid"`.
- AC4: `Harness::new().with_sim::<GridModule>().build()` (grid alone, without `WorldConfigModule`) panics with substring `"closed-reads"` - grid reads `WorldConfig`, no writer registered.
- AC5: Registering a second StaticData module that also declares `writes: names![Grid]` panics at registration with substring `"single-writer"`. (Negative test via a stub module in the test file.)
- AC6: `Grid::occupancy` uses `std::collections::BTreeMap<(u32,u32), Entity>`. `HashMap` is forbidden - determinism replay requires stable iteration order, which `BTreeMap` guarantees and `bevy::utils::HashMap` does not.
- AC7: Coordinate keys use `u32`, not `i32`. `Grid.occupancy: BTreeMap<(u32,u32), Entity>` - negative coordinates are not representable, matching the `u32` dimensions.

**Implementation constraints (review-only, not runtime-asserted):**

- `WorldConfig` derives only `Resource, Clone, Debug`. No `Mutex`, `RefCell`, `AtomicU64`, `UnsafeCell` anywhere in `magnum_opus/src/world_config/`. Enforced by code review + clippy, not unit tests.

**Non-goals:**

- Loading seed or dimensions from CLI, environment, save file, or config file (post-MVP - hardcoded constants ship here).
- `Grid.occupancy` insertion. Writers live in F3 (`placement`) and drain a `CommandBus<PlaceTile>` from `Phase::Commands`. F1 ships an empty occupancy map and a stub bootstrap.
- Terrain data, resource veins, fog, or any content from Phase 2+.
- Biome variation, run-specific config, seed randomization.
- Grid spatial queries or helper API (lookups live in the consuming feature).

**Edge cases:**

- `WorldConfigModule` registered without `GridModule`: no panic. `WorldConfig` exists, no `Grid`. Valid intermediate state for features that read `WorldConfig` but not `Grid`.
- Registering `WorldConfigModule` twice via `app.add_data::<...>()`: panics with substring `"duplicate module id"` (registry invariant from `registry.rs`).
- `Grid` queried before first `app.update()`: returns `Grid::default()` - `dims_set == false`, `width == 0`, `height == 0`, empty occupancy. Consumers must check `dims_set` before acting on dimensions.
- `grid_metrics_system` on tick 1 before `grid_bootstrap_system` in the same tick: since both are in `Update` and the metric publishes via `add_metric_publish` (`Phase::Metrics`, after `Phase::World`), the bootstrap always runs first. Metric reads `occupancy.len() == 0` regardless of `dims_set`.
- `Harness::build()` consumes `self` (move semantics) - it cannot be invoked twice on the same Harness. Attempting to register a second module with the same id via `app.add_data::<>()` / `app.add_sim::<>()` is covered by edge case 2 (`"duplicate module id"`).

---

<!-- feature:world-generation -->
### F2: world-generation

**Purpose:** Deterministic terrain generation with ore-vein clusters. Populates `Landscape` (per-cell terrain type, elevation, moisture) and `ResourceVeins` (sparse vein map) from the run seed.

**Problem:** Factory gameplay requires varied terrain (where can I extract stone? where does water block routes?) and clustered resource deposits (the positioning puzzle). Without generation, every run looks identical and the spatial dimension collapses. F2 writes `Landscape.cells` on the first tick from seed-derived fBm noise (three channels: elevation, moisture, lava mask), and writes `ResourceVeins.veins` one tick later from clustered placement keyed to terrain type. Both resources are owned under single-writer; future hazard and extraction features will mutate them via `Phase::World` and `Phase::Production` writes, so the SimDomain archetype is required (StaticData would freeze them post-startup).

**Modules:**

1. **`landscape` (SimDomain, `PRIMARY_PHASE = Phase::World`)**
   - Writes: `Landscape { width, height, cells: Vec<TerrainCell>, ready: bool }`
   - Reads: `WorldConfig`
   - Messages out: `LandscapeGenerated`
   - Metrics: gauge `landscape.cells`, gauge `landscape.kinds_present`
   - Installer: `write_resource`, `read_resource`, `emit_message`, `add_system(landscape_bootstrap_system)`, `add_metric_publish(landscape_metrics_system)`.
   - `landscape_bootstrap_system`: `Local<bool>` guard; on first call generates terrain, sets `ready = true`, writes `LandscapeGenerated`.

2. **`resources` (SimDomain, `PRIMARY_PHASE = Phase::World`)**
   - Writes: `ResourceVeins { veins: BTreeMap<(u32,u32), Vein>, clusters: u32, ready: bool }`
   - Reads: `WorldConfig`, `Landscape`
   - Messages in: `LandscapeGenerated`
   - Messages out: `VeinsGenerated { count: u32 }`
   - Metrics: gauge `resources.vein_count`, gauge `resources.cluster_count`, gauge `resources.total_amount`
   - `resources_bootstrap_system`: `Local<bool>` guard + runtime check `if !landscape.ready { return; }`. Generates clusters when both conditions hold, sets `ready = true`.

**Data shapes:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainKind { Grass, Rock, Water, Lava, Sand, Mountain, Pit }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainCell {
    pub kind: TerrainKind,
    pub elevation: i8,   // -64..63
    pub depth: u8,       // 0 for land, >0 for Water/Pit
    pub moisture: u8,    // 0..255
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind { IronOre, CopperOre, Stone, Coal }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality { Normal, High }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vein { pub kind: ResourceKind, pub quality: Quality, pub remaining: f32 }
```

**Generation sketch (non-normative - details live in the seed stage):**

The following is a reference implementation outline. ACs test behavioral guarantees (variety, clustering, terrain rules, determinism), not specific hash constants or threshold values. The seed-stage artifact will lock the final numbers.

Hash primitive: `splitmix64(u64) -> u64` (standard seed-a-seed PRNG). Per-cell 24-bit float via `(hash3(seed, x, y) >> 40) as f32 / (1u64 << 24) as f32` ∈ `[0, 1)`. Value noise with smoothstep interpolation over a 4-corner lattice, fBm by summing 5 octaves with halving amplitude and doubling frequency.

Three sub-seeds derived via `splitmix64(seed ^ salt)` (not XOR alone - salts collide unpredictably):
- `elevation_seed = splitmix64(seed ^ 0xE1E7)`
- `moisture_seed  = splitmix64(seed ^ 0x407F)`
- `lava_seed      = splitmix64(seed ^ 0x1A7A)`
- `cluster_seed   = splitmix64(seed ^ 0xC0FFEE)`

Terrain classification (all thresholds in one `const` block):

| elevation (normalized) | lava mask | moisture | kind | depth |
|---|---|---|---|---|
| < −0.40 | - | - | Pit | `((−0.40−e)×100) as u8` |
| < −0.15 | - | - | Water | `((−0.15−e)×40) as u8` |
| < 0.15 | > 0.70 | - | Lava | 0 |
| < 0.15 | <= 0.70 | < 0.30 | Sand | 0 |
| < 0.15 | <= 0.70 | >= 0.30 | Grass | 0 |
| < 0.50 | - | - | Rock | 0 |
| >= 0.50 | - | - | Mountain | 0 |

Resource cluster placement:
1. Derive `N = 16` cluster centers via `hash3(cluster_seed, i, 0) % width` / `% height`.
2. For each center, classify terrain at center. If terrain does not match any resource rule, skip the cluster.
3. Expand radius-3 (Manhattan) around each center with density `1 - (d / 3)`. Per-tile hash-roll; pass -> insert `Vein`.
4. Resource rules by terrain:

   | Resource | Acceptable terrain |
   |---|---|
   | IronOre | Rock, Mountain |
   | CopperOre | Rock, Sand |
   | Stone | Mountain, Rock |
   | Coal | Rock within Manhattan-2 of a Pit |

5. Quality: 20% High (hash-roll), 80% Normal. Remaining: `500..1500` scaled by `(1 + elevation_factor)`.

**Bootstrap timing (executor-dependent, intentional):**

Both `landscape_bootstrap_system` and `resources_bootstrap_system` live in `Phase::World` without inter-module ordering (installer does not expose cross-module `.after()`). Bevy serializes them on the `Landscape` resource conflict: landscape writes, resources reads. Which runs first is executor-determined. Because landscape finishes its work atomically on the first call (`Local<bool>` flips, resource fully populated), resources will observe a complete `Landscape` whenever it runs - either same-tick (if landscape scheduled first) or next-tick via Bevy's 2-tick message retention. Tests must not assert "vein generation happens on tick N"; they assert "after M ticks, veins are ready" for sufficiently large M (2 is safe).

The `LandscapeGenerated` message is kept explicitly - it satisfies the `closed-messages` cross-module invariant (resources declares `messages_in: names![LandscapeGenerated]`, landscape declares `messages_out`). Without it, the two modules would have no event coupling, and the closure check would not catch a future orphaned reader.

**Acceptance criteria:**

- AC1: After `Harness` with `WorldConfigModule + LandscapeModule` and two `app.update()` calls, `Landscape.ready == true`, `cells.len() == 64 * 64`.
- AC2: Two `Harness` builds with identical `WorldConfig` produce bit-identical `Landscape.cells` vectors after two ticks. Test compares `Vec<TerrainCell>` by `==`.
- AC3: At least **4** distinct `TerrainKind` values appear in `Landscape.cells` after generation with the default seed. The `landscape.kinds_present` gauge equals that count.
- AC4: After two `app.update()` calls with `WorldConfigModule + LandscapeModule + ResourcesModule`, `ResourceVeins.ready == true` and `veins.len() > 0`. Tick-1 state is executor-dependent and not asserted; see "Bootstrap timing" note above.
- AC5: For every vein in `ResourceVeins.veins`, the terrain at its position satisfies the resource rule. `IronOre` only on `Rock`/`Mountain`; `CopperOre` only on `Rock`/`Sand`; `Stone` only on `Rock`/`Mountain`; `Coal` only on `Rock` with a `Pit` neighbor within Manhattan-2.
- AC6: Cluster distribution is spatial, not uniform. At least one of the 16 cluster centers produces >= 5 veins within radius 3.
- AC7: Registering `LandscapeModule` without `WorldConfigModule` panics with substring `"closed-reads"` on `WorldConfig`.
- AC8: Registering `ResourcesModule` without `LandscapeModule` panics with a joined error message that contains **both** substrings: `"closed-messages"` (from the missing `LandscapeGenerated` producer) AND `"closed-reads"` (from the missing `Landscape` writer). Both must be present - a single-substring match is insufficient.
- AC9: Two `Harness` builds with identical `WorldConfig` produce bit-identical `ResourceVeins.veins` maps after two `app.update()` calls, compared by `BTreeMap` equality. `BTreeMap` is mandatory to guarantee stable iteration; `HashMap` is forbidden (non-deterministic seed across runs).

**Implementation constraints (review-only, not runtime-asserted):**

- No interior mutability (`Mutex`, `RefCell`, `Atomic*`, `UnsafeCell`) on any contract-visible type in `magnum_opus/src/landscape/` or `magnum_opus/src/resources/`. Enforced by code review + clippy, not unit tests.

**Non-goals:**

- Runtime terrain mutation (hazards changing `TerrainKind`): contract allows it - landscape is `SimDomain` - but F2 ships only initial generation. Hazard writes belong to F12 (`weather-hazards`).
- Vein depletion: contract path is `messages_in: names![VeinExtracted { pos, amount }]` - belongs to F5 (`recipes`) when mining is implemented.
- Biome-specific terrain rules (forest vs desert vs volcanic). Single biome for now; biome variation is post-MVP.
- Hex grids. Square tiling only.
- Resource rarity tiers beyond Normal/High. Three-tier quality was cut from design; do not add it back.

**Edge cases:**

- Seed `0`: `splitmix64(0 ^ salt)` produces non-zero sub-seeds, so generation proceeds normally. AC1 uses the fixed default from F1, but a seed-0 variant test must not crash.
- Grid cell at boundary `(0, 0)` or `(63, 63)`: terrain classification references no out-of-bounds neighbors. Coal's "adjacent Pit" check uses Manhattan-2 - at corner cells, only in-bounds neighbors are considered; rule silently skips if no Pit is reachable.
- Cluster center lands on unacceptable terrain (e.g. on `Water` where no rule matches): cluster is skipped entirely. `ResourceVeins.clusters` counter NOT incremented. Valid run may have fewer than 16 actual clusters.
- Two cluster centers within Manhattan-distance 3 of each other: their veins overlap. `BTreeMap::insert` overwrites - last write wins. Documented in AC5 (rule conformance still holds for surviving veins).
- `Landscape.cells` queried before tick 2: may return `Vec::new()` (default) or a fully populated vector, depending on executor ordering. Consumers must gate on `landscape.ready`.
- All four resource rules simultaneously exclude terrain under a cluster center (e.g. Grass-only area): cluster produces zero veins even if center is valid. Acceptable.
- IEEE-754 determinism across platforms: f32 math is scalar (no SIMD auto-vectorization at default opt levels), arithmetic order is fixed by single-threaded iteration. Same seed -> identical `cells` on x86 and ARM. Tested in AC2.
