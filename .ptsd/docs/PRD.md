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

---

<!-- feature:render-pipeline -->
### F18: render-pipeline

**Purpose:** Visual identity for the project - pixel-art look layered over 3D-style models via a low-res off-screen render target and a post-processing chain. Every other render feature draws INTO this feature's target.

**Problem:** Bevy's default render graph emits high-resolution smooth output. `docs/VISUALS.md` requires pixel-art: models render into a low-res framebuffer (480×270), then the pipeline applies outline (Sobel over depth+normal buffers), toon shading (luminance quantization), posterization (color quantization), and nearest-neighbor upscale to window size. Without this layer, impostor sprites look like ordinary 3D and the visual identity is lost. The pipeline is also the single control point: one parameter changes outline thickness / band count / palette for ALL visible content.

**Architecture tension (honest):** core modules live in `Update/PostUpdate/PreUpdate` schedules and work with `Res<T>` / `Query<T>`. Bevy's render graph is an App-setup concern - custom materials, render pipeline handles, extraction into the `RenderApp` subgraph. This framework-level work does not fit `SimInstaller` / `ViewInstaller`. F18 therefore ships **two artifacts**:

1. **`render_pipeline_config` (StaticData module)** - owns the configuration resource `RenderPipelineConfig`, PTSD-trackable under the usual single-writer invariant.
2. **`RenderPipelinePlugin` (plain bevy plugin, not a PTSD module)** - installs the render target, a camera rendering into it, the post-processing chain, and the upscale pass. Lives alongside `CorePlugin` as infrastructure plumbing. Reads `RenderPipelineConfig` at `build()`-time and applies the corresponding setup.

Config is under the PTSD contract; render plumbing is outside it (exactly as `CorePlugin` itself is). Tests are limited to headless validation of the config resource; visual correctness is verified through an example binary plus manual inspection.

**MVP scope (v0):** low-res target + nearest-neighbor upscale. No outline / toon / posterize yet. Proof that the pipeline is alive - an upscaled black window, which will look pixel-crisp once content arrives via F19. Shaders are added incrementally in v1-v2 after the base ships.

**Modules:**

1. **`render_pipeline_config` (StaticData)** - `RenderPipelineConfig { low_res_width: u32, low_res_height: u32, outline_enabled: bool, toon_bands: u8, posterize_levels: u8 }`. MVP values: 480×270, outline=false, toon_bands=0 (off), posterize_levels=0 (off). Config holder only; shaders activate when flags flip in v1-v2.

2. **`RenderPipelinePlugin` (bevy plugin)** - creates the off-screen texture, a scene camera that renders 3D content into it, and a fullscreen quad blit camera that samples the off-screen image with nearest-neighbor into the window. Scene camera is `Camera3d` with `Projection::Orthographic` and an isometric `Transform` (tilt ≈ 35.264°, yaw 45°); blit camera stays `Camera2d` over a `Material2d` quad so the outline post-process continues to run in 2D space. Reads `RenderPipelineConfig` in `build()` to size the target. Added by the App owner next to `CorePlugin` and Bevy's rendering plugins (`DefaultPlugins` or a partial subset).

**Acceptance criteria:**

- AC1: `Harness::new().with_data::<RenderPipelineConfigModule>().build()` builds successfully and `RenderPipelineConfig` is present with values `{ low_res_width: 480, low_res_height: 270, outline_enabled: false, toon_bands: 0, posterize_levels: 0 }`.
- AC2: `magnum_opus::render_pipeline::RenderPipelineConfig` derives `Resource, Clone, Debug, PartialEq` - plain data, no interior mutability.
- AC3: Registering a second StaticData module that also declares `writes: names![RenderPipelineConfig]` panics at build with substring `"single-writer"`.
- AC4: `cargo run --example render_smoke` opens a window with a nearest-neighbor upscaled black framebuffer and does not panic. Manual validation; not a unit test - asserted through example's run-loop banner and a doc comment at the top of `magnum_opus/examples/render_smoke.rs`. Impl must create the `examples/` directory and register the binary in `Cargo.toml`.
- AC5: The existing 61 tests continue to pass; `cargo test` runs clean after `RenderPipelineConfigModule` and any F18 tests are added. No existing test regresses.

**Non-goals:**

- Shaders (outline, toon, posterize) - deferred to v1-v2. MVP ships zero shaders.
- Async asset loading - pipeline operates on a procedurally-created texture, no `AssetServer::load` calls.
- Anti-aliasing, MSAA, HDR - low-res + nearest-neighbor is intentionally aliased.
- Content rendering - the MVP window is black; visible content arrives when F19 (`world-render`) uses the target.
- Performance tuning - 60 FPS is not a contract in MVP, correctness is.
- Cross-platform shader variants - focus on Linux / native OpenGL / Vulkan from Bevy's default backend.
- Multi-window support - the pipeline blits to exactly one window. Any second window created by external code receives no upscaled output.
- Headless CI rendering - the example binary requires a live display; CI coverage for render is an open problem deferred to a later feature (see `~/.claude/plans/render-roadmap.md` §Open Questions #2).

**Edge cases:**

- Window closed immediately after open: `render_smoke` uses the standard winit loop and exits when the user closes the window. Test expectation is "does not panic on close", not "runs N frames".
- Target resolution > window resolution: upscale becomes downscale; nearest-neighbor sampling still works. Not tested in MVP - assume `Target < Window`.
- `RenderPipelineConfig` missing when `RenderPipelinePlugin` builds: plugin reads the resource via `World::get_resource` and panics with substring `"RenderPipelineConfig resource missing"` if absent. Requires `render_pipeline_config` module to be registered BEFORE the plugin.
- Multiple `RenderPipelinePlugin` registrations in a single App: bevy emits its standard plugin-dedup panic. Not our responsibility.

**Implementation constraints (review-only, not runtime-asserted):**

- `RenderPipelineConfig` contains only `u32`/`u8`/`bool` fields; zero interior mutability (`Mutex`/`RefCell`/`Atomic*`/`UnsafeCell`).
- `RenderPipelinePlugin::build(&self, app: &mut App)` does not mutate existing sim resources (`Grid`, `Landscape`, `ResourceVeins`, etc.) - read-only access to `RenderPipelineConfig`, writes only to render-private resources it owns.
- Post-processing shaders (v1+) are WGSL, embedded via `include_str!`, never runtime-loaded from disk.
- The impl creates `magnum_opus/examples/` directory and the `render_smoke.rs` binary; `Cargo.toml` declares it under `[[example]]` with `name = "render_smoke"`.

---

<!-- feature:world-render -->
### F19: world-render

**Purpose:** First visible content - terrain tiles from `Landscape.cells` rendered into the low-res target produced by F18, plus vein markers from `ResourceVeins`. Turns the black window into a procedurally-generated 64×64 map of colored squares.

**Problem:** F2 (`world-generation`) produces `Landscape.cells: Vec<TerrainCell>` headless - the data exists but nothing renders it. F18 (`render-pipeline`) provides a low-res framebuffer and nearest-neighbor upscale - but nothing draws into it. F19 closes that gap: a View module reads `Landscape` and spawns one horizontal mesh per tile on the scene render layer, so F18's iso 3D scene camera captures them into the low-res target. For MVP each tile is a flat-shaded `Plane3d` quad on the ground plane (`y = 0`) with `StandardMaterial { unlit: true, base_color: ... }`; one color per `TerrainKind`. Vein markers layer on top as smaller unlit planes slightly above the ground. Impostor textures (albedo + normal + depth) are deferred to F20 and v1+ of this feature.

**Architecture fit:** Standard View archetype. Reads sim-owned resources (`Landscape`, `ResourceVeins`), writes a view-private cache (`WorldSceneCache`) tracking the entities it has spawned. Runs in `PostUpdate`. No commands, no messages.

**Modules:**

1. **`world_render` (View)**
   - Reads: `Landscape`, `ResourceVeins`
   - Writes: `WorldSceneCache { tiles: BTreeMap<(u32, u32), Entity>, veins: BTreeMap<(u32, u32), Entity>, synced: bool }`
   - Metrics: gauge `world_render.tiles_drawn`, gauge `world_render.veins_drawn`
   - Installer: `read_resource::<Landscape>()`, `read_resource::<ResourceVeins>()`, `write_resource::<WorldSceneCache>()`, `add_system(world_render_system)`.
   - `world_render_system`: on each tick, if `Landscape.ready && !cache.synced`, spawns a tile entity per cell with a shared `Plane3d` mesh + unlit `StandardMaterial` on `RenderLayers::layer(1)` (the scene layer F18's iso 3D camera watches) at world-space coordinates. Then does the same for each `Vein` with a smaller plane slightly above the ground. Sets `cache.synced = true`.

**Tile-to-world mapping:**

- Tile size: `TILE_WORLD_SIZE = 1.0` world unit per tile. 64 tiles × 1.0 = 64 units across.
- Tile `(x, y)` centers at Bevy world position `(x as f32 - GRID_HALF, 0.0, y as f32 - GRID_HALF)` where `GRID_HALF = 32.0` (grid centered on origin). Y is the vertical axis (ground plane is `y = 0`); tiles extend on the XZ plane.
- Vein markers: smaller `Plane3d` (`VEIN_WORLD_SIZE = 0.5`) at the same `(x, z)` with `y = 0.02` so they draw above their tile without Z-fighting.

**Color palette (MVP flat colors):**

| TerrainKind | Color (sRGB hex) |
|---|---|
| Grass | 4a7b2c |
| Rock | 6c6c6c |
| Water | 2c4e7b |
| Lava | c84a1e |
| Sand | d4b878 |
| Mountain | 9c9c9c |
| Pit | 1c1c1c |

| ResourceKind | Marker color |
|---|---|
| IronOre | c87858 |
| CopperOre | b87840 |
| Stone | b0b0b0 |
| Coal | 282828 |

**Acceptance criteria:**

- AC1: After `Harness::new().with_data::<WorldConfigModule>().with_sim::<LandscapeModule>().with_sim::<ResourcesModule>().with_view::<WorldRenderModule>().build(); app.update(); app.update();` the resource `WorldSceneCache` has `synced == true`, `tiles.len() == 64 * 64`, and `veins.len() == ResourceVeins.veins.len()`.
- AC2: Before `Landscape.ready`, `WorldSceneCache.synced == false` and `tiles.is_empty() == true`. The View system is resilient to not-ready sim state - no panic, no partial sync.
- AC3: `WorldSceneCache.tiles` and `WorldSceneCache.veins` both use `BTreeMap<(u32, u32), Entity>`. `HashMap` forbidden (same determinism rationale as `Grid.occupancy`).
- AC4: Registering `WorldRenderModule` without `LandscapeModule` panics with `"closed-reads"` on `Landscape`.
- AC5: Registering `WorldRenderModule` without `ResourcesModule` panics with `"closed-reads"` on `ResourceVeins`.
- AC6: Registering a second View module that declares `writes: names![WorldSceneCache]` panics at build with `"single-writer"`.
- AC7: `cargo run --example world_render_smoke` opens a window showing a 64×64 iso-projected tile grid with distinct colors per `TerrainKind` and vein markers on matching tiles. Manual validation, captured via the screenshot harness (`SCREENSHOT=1`). PNG output path `/tmp/claude-bevy-world_render_smoke.png`.

**Non-goals:**

- Impostor meshes (albedo + normal + depth textures). MVP is flat-shaded unlit.
- Per-pixel lighting, PBR, real-time shadows. Tile and vein meshes use `StandardMaterial { unlit: true }`; no light sources are registered by this feature.
- Diff-based incremental sync. The MVP syncs once on first ready tick; runtime terrain mutation (F12 hazards) is the trigger for adding diffs later.
- Fog-of-war overlay. Comes with F13.
- Camera control. Uses the static iso camera installed by F18 (`RenderPipelinePlugin`). Runtime pan/zoom/click is F21.
- Scaled tile sizes. `TILE_WORLD_SIZE = 1.0` is a hardcoded constant until a future tuning feature touches it.
- UI panels, tooltips, notifications. All belong to F21.

**Edge cases:**

- Tick 0 (before first `app.update()`): `WorldSceneCache` default returns `synced = false`, empty maps. View system hasn't run yet.
- Landscape ready but ResourceVeins not ready yet: the View system syncs tiles immediately; veins get synced on the tick after both resources are ready. Test AC1 uses two ticks so both are done before assertion.
- Duplicate `WorldRenderModule` registration: `duplicate module id` panic (generic registry invariant).
- Vein position outside grid bounds: impossible by construction - veins are only placed by F2's generator inside `[0, width) × [0, height)`. If a rogue test inserts an out-of-bounds vein, the world coord calculation still produces a valid Bevy position and the mesh renders off-screen; no panic.

**Implementation constraints (review-only):**

- `WorldSceneCache` uses `BTreeMap`, not `HashMap`. Zero interior mutability.
- The color palette lives in one `const`/`match` block in `world_render/palette.rs`. Changes require a single-file touch.
- World-space coordinate math lives in one function `tile_world_pos(x, y) -> Vec3`, not inlined into the spawn loop. Return value uses `Vec3::new(x - GRID_HALF, 0.0, y - GRID_HALF)` with `GRID_HALF = 32.0`.
- Tile and vein meshes are shared handles (one `Plane3d` per tile, one per vein marker), created once per sync and reused. Per-terrain / per-resource materials are lazily cached in a local `BTreeMap<TerrainKind, Handle<StandardMaterial>>` inside the system body.
- The example `examples/world_render_smoke.rs` copies the `SCREENSHOT` env pattern from `render_smoke.rs`; PNG output path `/tmp/claude-bevy-world_render_smoke.png`.

---

<!-- feature:render-outline -->
### F22: render-outline

**Purpose:** Activate the `outline_enabled` flag on `RenderPipelineConfig` with a real shader pass. Sobel over the low-res framebuffer paints black pixels on tile-boundary edges, pushing the flat-colored map from "minimalist pixel" into "fantasy-retro pixel-art" - the visual identity the project was founded on.

**Problem:** F18 reserved `outline_enabled: bool` in the config but shipped no shader (v0 is a pass-through blit). F19 produces sharply-defined tile regions on the low-res target; Sobel on color gradients is the cheapest way to silhouette them without geometry or per-sprite normal maps. Without outlines the scene reads as a modern minimalist palette; with outlines it reads as pixel-art. The change is entirely confined to `RenderPipelinePlugin` - no new sim state, no new PTSD module, no migration of existing render entities.

**Architecture fit:** Pure extension of an existing bevy `Plugin`. Zero new PTSD-tracked resources. The pipeline plugin reads `RenderPipelineConfig.outline_enabled` at `build()`-time (same pattern as the low-res size). When true, the blit entity uses `Mesh2d(Rectangle)` + `MeshMaterial2d<OutlineMaterial>` instead of `Sprite::from_image`. The WGSL source is embedded via `embedded_asset!` per the existing F18 implementation constraint.

**Shader:** `outline.wgsl` is a standard `Material2d` fragment. Samples a 3×3 neighborhood of the source texture via the material sampler, computes Sobel `Gx/Gy` over luminance (`dot(rgb, [0.299, 0.587, 0.114])`), returns the outline color when `sqrt(Gx² + Gy²) > threshold`, otherwise returns the unmodified sample. Texel offsets come from `textureDimensions(source)` - resolution-independent.

**Modules:** None added to the PTSD registry. This feature ships:

1. **`OutlineMaterial` (Rust type in `render_pipeline/outline.rs`)** - derives `AsBindGroup, Asset, TypePath, Clone, Debug`. Fields: `#[uniform(0)] params: OutlineParams`, `#[texture(1)] #[sampler(2)] source: Handle<Image>`. Implements `Material2d` with `fragment_shader()` returning `"embedded://magnum_opus/render_pipeline/outline.wgsl"`.
2. **`OutlineParams` (Rust type)** - derives `ShaderType, Clone, Debug`. Fields: `threshold: f32`, `color: LinearRgba`.
3. **`outline.wgsl`** - embedded via `embedded_asset!(app, "outline.wgsl")` in `RenderPipelinePlugin::build`.
4. **`RenderPipelinePlugin` modification** - registers `Material2dPlugin::<OutlineMaterial>::default()` unconditionally; `setup_low_res_target` branches on `cfg.outline_enabled` to pick `Sprite::from_image` vs `Mesh2d + MeshMaterial2d<OutlineMaterial>`.

**Acceptance criteria:**

- AC1: `OutlineMaterial` is a public type in `magnum_opus::render_pipeline` and derives `AsBindGroup, Asset, TypePath, Clone, Debug`. Fields: `params: OutlineParams` with `#[uniform(0)]`, `source: Handle<Image>` with `#[texture(1)] #[sampler(2)]`. Compile-time check - a test constructs one with a default handle and a default `OutlineParams`.
- AC2: `OutlineParams` derives `ShaderType, Clone, Debug, Default` with fields `threshold: f32` (default `0.08`) and `color: LinearRgba` (default `LinearRgba::BLACK`).
- AC3: `RenderPipelinePlugin::build` registers `Material2dPlugin::<OutlineMaterial>::default()` whether or not outline is enabled. Test: construct a minimal `App` with `RenderPipelinePlugin` + required bevy plugins; resource `Assets<OutlineMaterial>` exists after `app.finish()`.
- AC4: With `outline_enabled = false` (default), `setup_low_res_target` spawns the blit entity via `Sprite::from_image(handle)` (existing behavior, no regression). Checked through the existing MVP screenshot test path - `cargo run --example render_smoke` still produces the black upscaled framebuffer.
- AC5: `cargo run --example world_render_smoke` with env `OUTLINE=1 SCREENSHOT=1` writes PNG to `/tmp/claude-bevy-world_render_smoke_outline.png`. The example overrides `RenderPipelineConfig.outline_enabled` to `true` before `app.run()` when the env var is set. Manual validation: the PNG visibly shows thin black lines between differently-colored tile regions; adjacent same-color tiles have no line.
- AC6: `cargo test` - all existing 70 tests continue to pass, plus the F22 compile-time test for `OutlineMaterial`. Zero regressions.

**Implementation constraints (review-only):**

- `outline.wgsl` lives at `magnum_opus/src/render_pipeline/outline.wgsl` and is embedded via `embedded_asset!` - never loaded from disk at runtime.
- Shader path in `Material2d::fragment_shader()` is `"embedded://magnum_opus/render_pipeline/outline.wgsl"` - hardcoded, not constructed at runtime.
- `OutlineMaterial` does NOT carry interior mutability on any field.
- No new resource types added to `RenderPipelineConfig` - threshold and color are embedded in `OutlineParams` attached to the material instance, not exposed as a global config. A future feature may lift them into config if runtime tweaking becomes necessary.
- The luminance formula is fixed: `dot(rgb, [0.299, 0.587, 0.114])` - standard Rec. 601. No parametrization.

**Non-goals:**

- Toon-shading (luminance quantization) - F23.
- Posterization (color-channel quantization) - F23.
- Depth or normal buffer input - flat 2D scene has no useful depth/normal signal; Sobel over color is sufficient for this milestone.
- Runtime toggling. The flag is read once at plugin build; toggling at runtime requires rebuilding the blit entity, out of scope here.
- Tunable threshold/color from the UI. Hardcoded defaults until a player-facing settings panel exists (F21+).
- Outline thickness > 1 low-res pixel. The 3×3 Sobel kernel produces exactly-one-pixel edges; thicker outlines need a dilate pass, deferred.
- Anti-aliased edges. The output is deliberately hard-edged to match the pixel-art aesthetic.

**Edge cases:**

- `outline_enabled = false` and no attempt to instantiate `OutlineMaterial`: `Assets<OutlineMaterial>` remains empty. `Material2dPlugin` registration is harmless - asset type exists, no materials use it, zero render cost.
- Uniform tile color region larger than the Sobel kernel: interior pixels have zero gradient, no outline drawn - correct behavior (outline is a boundary, not a fill).
- Tile boundary between two tiles whose luminance happens to be identical (e.g. two palette entries with the same grayscale): no outline drawn on that boundary. Accepted - the palette is tuned so no two adjacent terrain types share luminance (verified by eye during impl review).
- Shader compile failure at startup: bevy panics from the render thread. Not caught by the PTSD pipeline; manifests as a runtime crash when `world_render_smoke --outline` is launched. Detection is the `cargo run --example` step in AC5.
- Window closed before shader compiles: same winit exit path as F18 MVP. No panic.
- Multi-GPU / backend differences: shader uses no backend-specific features. WebGPU, Vulkan, Metal, OpenGL all compile standard WGSL 1.0.

---

<!-- feature:placement -->
### F3: placement

**Purpose:** Turn player placement intent into grid mutation. Adds the `PlaceTile` command payload and a drain system inside the `grid` module so that pushing a `PlaceTile` onto its `CommandBus` spawns a bare entity with a `Position` component and reserves a cell in `Grid.occupancy`. First real sim-cycle feature - unblocks F4 (buildings attach components to those entities) and F21 (mouse/cursor will push `PlaceTile` from an InputUI module).

**Problem:** F1 shipped `Grid.occupancy` empty on purpose. Single-writer means grid is the only module allowed to mutate it, so the placement command must be drained **inside** grid rather than a separate writer. F3 closes that loop by extending the grid module contract with `commands_in: names![PlaceTile]` and a drain system registered in `Phase::Commands`. The command payload is defined in `grid/commands.rs`; the drain validates bounds + occupancy and spawns a new entity with a `Position` component.

**Architecture fit:**

1. **`grid` module extension** (same `SimDomain`, same `PRIMARY_PHASE = Phase::World`) - contract gains `commands_in: names![PlaceTile]`. `install` gains `ctx.consume_command::<PlaceTile>()` and `ctx.add_command_drain(grid_placement_drain_system)`. Writer single-ness of `Grid` preserved.
2. **New types:**
   - `PlaceTile { x: u32, y: u32 }` - public payload struct, `Send + Sync + 'static`. No `Entity` field; drain spawns the entity.
   - `Position { x: u32, y: u32 }` - public `Component` attached to every placed entity. Minimal MVP shape; F4 will add `Building`, F5 `Recipe`, etc.
3. **No placement-input module yet.** Tests push `PlaceTile` into `CommandBus<PlaceTile>` directly via `app.world_mut().resource_mut::<CommandBus<PlaceTile>>()`. Real mouse -> command translation is F21's job.

**Drain algorithm:**

```text
for cmd in bus.drain():
    if !grid.dims_set: continue                   # bootstrap not yet done
    if cmd.x >= grid.width or cmd.y >= grid.height: continue   # bounds
    if grid.occupancy.contains_key(&(cmd.x, cmd.y)): continue  # occupied
    let entity = commands.spawn(Position { x: cmd.x, y: cmd.y }).id()
    grid.occupancy.insert((cmd.x, cmd.y), entity)
```

Rejected commands are silently dropped. A future error-reporting feature can surface them via a `MessageWriter<PlacementRejected>`; F3 scope is silent-drop because nothing consumes rejection yet.

**Acceptance criteria:**

- AC1: `Harness::new().with_data::<WorldConfigModule>().with_sim::<GridModule>().build(); app.world_mut().resource_mut::<CommandBus<PlaceTile>>().push(PlaceTile { x: 3, y: 4 }); app.update(); app.update();` - after the two ticks, `grid.occupancy.contains_key(&(3, 4)) == true`, `grid.occupancy.len() == 1`, and the referenced entity carries `Position { x: 3, y: 4 }`.
- AC2: Pushing `PlaceTile { x: 100, y: 100 }` (out of bounds for default 64×64 grid) produces zero entities and zero `occupancy` entries after two ticks. No panic.
- AC3: Pushing two `PlaceTile { x: 7, y: 7 }` commands in sequence then ticking twice - `occupancy.len() == 1` at `(7, 7)`; only the first wins. The system does not overwrite an occupied cell.
- AC4: `Harness::new().with_sim::<GridModule>().build()` (without `WorldConfigModule`) still panics with `"closed-reads"` on `WorldConfig` - the new `commands_in` entry does not affect the existing read contract.
- AC5: `CommandBus<PlaceTile>` is initialized when grid is registered, even if no `PlaceTile` has ever been pushed. `app.world().get_resource::<CommandBus<PlaceTile>>().is_some() == true`.
- AC6: All pre-F3 tests continue to pass. Adding F3 raises the count by the number of new tests in this feature without regressing any existing one.

**Implementation constraints (review-only):**

- `PlaceTile` is a plain data struct (`Send + Sync + 'static`), no interior mutability.
- `Position` derives `Component, Clone, Copy, Debug, PartialEq, Eq` - zero methods on it in F3.
- Drain system signature uses `Commands`, `ResMut<CommandBus<PlaceTile>>`, `ResMut<Grid>`, and runs in `Phase::Commands` via `ctx.add_command_drain(..)`. Spawning happens through `Commands::spawn`; the entity id is captured from `.id()` and inserted into `grid.occupancy` in the same system.
- The drain system is guarded by `if !grid.dims_set { return; }` - during tick 1 the bootstrap system may run after the drain (Phase::Commands precedes Phase::World), so pushing a PlaceTile before tick 2 is a no-op and must not panic.
- Grid stays the sole writer of `Grid.occupancy`. `single-writer` invariant holds for `Grid`.

**Non-goals:**

- `Building` component, `BuildingDB`, or any building-specific logic - F4.
- `RemoveBuilding` / `Destroy` commands - out of scope; add when combat or hazards need it.
- Mouse input, cursor-to-grid raycast, UI feedback - F21.
- Terrain-type validation (miner must be on Rock/Mountain). Requires `BuildingDB` and its recipe-terrain rules, introduced in F4.
- Rejection messages (`PlacementRejected { reason }`). Silent drop is sufficient for F3 because nothing consumes rejections yet.

**Edge cases:**

- Empty bus: drain iterates zero times. No state change. Metric `grid.occupancy_count` stays at its previous value.
- Push before `grid_bootstrap_system` runs (tick 1, `dims_set = false`): drain early-returns for that command; it is consumed by `bus.drain(..)` and dropped. Tests must call at least two `app.update()` calls before asserting, so bootstrap is guaranteed done for commands pushed between tick 1 and tick 2.
- Bus fed with 100 `PlaceTile` in one tick: all drained, each validated individually. Occupancy grows by the number of distinct in-bounds, unoccupied coordinates among them. No batching or rate-limiting.
- Bus fed concurrently from multiple InputUI emitters in the future: `CommandBus<T>` is a single Resource; Bevy serializes writes automatically. F3 does not introduce concurrency.
- Grid width/height change at runtime: not supported - `grid_bootstrap_system` sets them once and the drain reads the fixed values. A future resize feature would need to invalidate occupancy; out of scope.

---

<!-- feature:buildings -->
### F4: buildings

**Purpose:** Give placed entities a concrete identity. F3 spawns a bare `Position` entity on every `PlaceTile`; F4 introduces `BuildingDB` with a fixed MVP set of building types (Miner, Smelter, Mall, EnergySource), a `Building` component tagging entity kind, and extends the `PlaceTile` payload so the grid drain can spawn typed entities. Unblocks F5 (recipes attached per building type) and F7 (groups formed from adjacent Building entities).

**Problem:** The MVP production loop needs entities that know what they produce and consume. A tile with a `Position` alone is a pin on a map; a tile with `Position + Building(Miner)` is a resource-producing factory node. F4 closes that gap by loading building definitions from a `BuildingDB` StaticData and attaching the matching `Building` tag at placement time. No recipes/energy/groups yet - the drain merely stamps type metadata; F5 adds the production bits.

**Architecture fit:**

1. **`buildings` module** (`StaticData`) - owns `BuildingDB { defs: BTreeMap<BuildingType, BuildingDef> }`. MVP defs list four types; each carries a short human-readable name. Full recipe/terrain metadata is deferred to F5 when it is actually consumed. `install` inserts the default DB populated from a `const` table in source. No systems.
2. **`Building` component** (`#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]`) - single field `building_type: BuildingType`. Lives in `buildings/component.rs` and re-exported at module root.
3. **`BuildingType` enum** - `Miner | Smelter | Mall | EnergySource`. Derives `Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd` (Ord for BTreeMap key stability).
4. **`PlaceTile` payload extension** - new field `building_type: Option<BuildingType>`. `None` = F3 behavior (raw Position entity). `Some(t)` = spawn entity with `Position` + `Building { building_type: t }`.
5. **Grid drain update** - after the existing bounds + occupancy checks, when `cmd.building_type.is_some()` the entity is spawned with both components in one `.spawn((..., ...))` call.

**Acceptance criteria:**

- AC1: `Harness::new().with_data::<WorldConfigModule>().with_data::<BuildingDbModule>().with_sim::<GridModule>().with_input::<PlacementInputModule>().build();` compiles and `BuildingDB` resource exists with four entries after build. `db.defs.len() == 4` and `db.defs.contains_key(&BuildingType::Miner)`.
- AC2: Push `PlaceTile { x: 5, y: 5, building_type: Some(BuildingType::Miner) }`, tick twice. The entity at `(5, 5)` carries both `Position { x: 5, y: 5 }` and `Building { building_type: BuildingType::Miner }`.
- AC3: Push `PlaceTile { x: 6, y: 6, building_type: None }`, tick twice. The entity at `(6, 6)` has `Position` but **no** `Building` component (backwards compatibility preserved).
- AC4: `BuildingType` enum derives `Ord, Hash, PartialEq, Eq` - verified by constructing a `BTreeMap<BuildingType, u32>` and a `HashMap<BuildingType, u32>` in the test with all four variants as keys.
- AC5: Registering a second `StaticData` module writing `BuildingDB` panics with `"single-writer"` substring.
- AC6: Adding F4 does not regress any existing 76 tests. Test count rises by the count of new tests only.

**Implementation constraints (review-only):**

- `BuildingDB` uses `BTreeMap`, not `HashMap` (determinism rule from `Grid.occupancy`).
- `BuildingDef` contains only plain data in F4: `pub name: &'static str`. Recipe and terrain metadata join at F5.
- `BuildingType` variants are closed - adding a variant requires a matching entry in the `const` definition table and in `BuildingDbModule::install`.
- `PlaceTile.building_type: Option<BuildingType>` must default to `None` via `#[derive(Default)]` or an explicit constructor if tests rely on it. For F4 the existing tests construct the struct literally with field names, so no default is strictly required - but add `Default` anyway for ergonomics (`PlaceTile::default()` yields zeros + None).
- No runtime mutation of `BuildingDB`. It is genuinely read-only after install.

**Non-goals:**

- Recipes, ProductionState, InputBuffer, OutputBuffer - F5.
- Terrain-type validation (Miner only on Rock/Mountain). Requires `Landscape.cells` access inside the drain + a `terrain_requirement` field in `BuildingDef`; add when placement-from-inventory constraint matters. F4 accepts any building on any tile that is in-bounds and unoccupied.
- Inventory counting (`Mall produces Miner -> inventory[Miner] += 1`). Requires a separate `Inventory` resource and production output routing; belongs to F5+.
- Upgrade / tier mechanics.
- Building removal or destruction commands.
- UI for selecting which building to place (F21).

**Edge cases:**

- `BuildingType` matches nothing in `BuildingDB`: impossible - the enum is closed and the const table defines every variant. If a test injects a rogue value via transmute, behavior is undefined; not our contract.
- `PlaceTile { building_type: Some(Miner) }` on tile already occupied by another entity: same rejection as F3. No partial insertion; the new entity is never spawned.
- `PlaceTile { building_type: None }` on valid tile: F3 behavior preserved - single `Position` entity, no `Building` component.
- BuildingDB read before any tick: resource is inserted at `Startup` (StaticData installer pushes via `insert_resource`), so it's already in the world by the time the first `app.update()` runs.
- Adding a new `BuildingType` variant without updating the DB table: compile-time exhaustiveness check - the `const` table is a `[(BuildingType, BuildingDef); N]` with explicit variants; missing variants produce a compiler warning at best unless we build the table via `match` on the enum inside the installer. MVP accepts the warning path; can harden later.

---

<!-- feature:group-formation -->
### F7: group-formation

**Purpose:** Fold adjacent `Building` entities into shared groups, the sim unit downstream features operate on (manifold, energy allocation, production stats). Each tick, the module walks all Building positions in the grid, runs a flood-fill over 4-connected adjacency, and attaches every member to a freshly-spawned Group entity. This unlocks F6 (one Manifold per group) and F5 (ProductionState advances only when the group is non-empty and energized).

**Problem:** F4 shipped isolated Building entities. The production loop requires a collective notion: five adjacent miners share an ore buffer; disconnected clusters each operate independently. Without the group abstraction, every downstream system would have to re-derive connectivity on its own. F7 centralises that computation inside a single `SimDomain` module in `Phase::Groups`. Behavior is deliberately simple: full recompute each tick. Incremental updates via `BuildingPlaced`/`BuildingRemoved` messages are deferred - MVP chose correctness over throughput.

**Architecture fit:**

1. **`group_formation` module** (`SimDomain`, `PRIMARY_PHASE = Phase::Groups`). Writes `GroupIndex`. Reads `Grid` (occupancy -> tile-entity map is consulted directly). Queries `Building` and `Position` components (no core declaration required - components are Query-only).
2. **New types:**
   - `Group` - ZST marker component on each group entity.
   - `GroupMember { group: Entity }` - component attached to every Building in a group.
   - `GroupIndex { groups: BTreeSet<Entity>, member_to_group: BTreeMap<Entity, Entity> }` - Resource owned by the module. `groups` lists every group entity currently alive; `member_to_group` answers "which group does this building belong to" in O(log n).
3. **Algorithm per tick:**
   1. Despawn every entity carrying `Group` (previous-tick groups).
   2. Remove `GroupMember` from every Building that has one (previous-tick attachments).
   3. Collect all Building entities and their positions into a working set.
   4. Flood-fill via iterative BFS. Neighbor check = cardinal (N/S/E/W). Only tiles that belong to a Building count as adjacent; non-Building Position-only entities are invisible.
   5. For each connected component, spawn a fresh Group entity and attach `GroupMember { group }` to each member.
   6. Rebuild `GroupIndex` from the new state.

**Acceptance criteria:**

- AC1: Three Miners placed at `(5, 5)`, `(5, 6)`, `(6, 5)` (L-shape, all cardinal-adjacent) - after two ticks past placement, `GroupIndex.groups.len() == 1` and all three entities have `GroupMember` pointing to the same group entity.
- AC2: Two disjoint clusters - `(3, 3), (3, 4)` and `(20, 20), (20, 21)` - produce `GroupIndex.groups.len() == 2`. The two GroupMember attachments within each cluster reference the same group; across clusters the groups differ.
- AC3: An entity spawned with `Position { x: 10, y: 10 }` but without a `Building` component (F3-style untyped placement) contributes nothing to any group; it never receives `GroupMember`. Adjacent Building tiles do not "see" it as a bridge.
- AC4: Empty grid (no placements) - `GroupIndex.groups.is_empty() == true`, `GroupIndex.member_to_group.is_empty() == true` after any number of ticks.
- AC5: Single-writer holds: registering a second `SimDomain` that claims `writes: names![GroupIndex]` panics with `"single-writer"`.
- AC6: Diagonal-only adjacency is NOT treated as adjacency. Two Miners at `(5, 5)` and `(6, 6)` produce two groups, not one.
- AC7: All 81 pre-F7 tests still pass plus the new AC set.

**Implementation constraints (review-only):**

- `GroupIndex` uses `BTreeSet` and `BTreeMap` (determinism rule - consistent with `Grid.occupancy`).
- Flood-fill is iterative with an explicit `Vec<(u32, u32)>` stack. Recursion forbidden - arbitrary map sizes could blow the stack.
- Neighbor enumeration avoids `u32` underflow with `checked_sub(1)` and compares against `grid.width`/`grid.height` for bounds. No `wrapping_add` on coordinate math.
- The system despawns previous group entities via `Commands::despawn(..)` and removes `GroupMember` via `Commands::entity(e).remove::<GroupMember>()`. Deferred ECS mutation; all observable changes land in the same tick due to `Commands` apply.
- Full-rebuild semantics mean group entity ids are unstable across ticks. Downstream systems must resolve groups through `GroupIndex.member_to_group` or the `GroupMember` component on a known Building, never by caching a group Entity across ticks.

**Non-goals:**

- Incremental updates on `BuildingPlaced` / `BuildingRemoved`. Performance optimization left for later; MVP prefers correctness.
- Group-type tagging (Combat group, Mall group, etc.). Relies on a wider tagging scheme that F11 (combat-groups) will introduce.
- Merge / split events emitted to downstream systems. With full-rebuild semantics every tick, the concept of "split this tick" is moot - the event table applies when incremental mode lands.
- Non-cardinal adjacency (diagonals, hex layouts).
- Energy / priority / pause attributes on the group - F8.
- Per-group statistics like `GroupStats.productionRates` - requires F5 production outputs; this is a read-only view built in a later feature.

**Edge cases:**

- One Building alone at `(0, 0)`: group of size 1. `GroupIndex.groups.len() == 1`, that single entity is its own group.
- Two Buildings at grid corners, far apart: two separate groups of size 1 each.
- Building and non-Building tile side-by-side: flood-fill from the Building halts at the non-Building neighbor. The non-Building tile does not receive a GroupMember.
- Building removed between ticks (future feature): next tick's full rebuild produces the correct group shape without any special handling. F7's algorithm is idempotent over the current world state.
- Buildings occupying the same tile: impossible by grid invariant - F3 drops duplicates.

---

<!-- feature:recipes-production -->
### F5a: recipes-production

**Purpose:** First tickable output. Adds `ResourceType` enum, a `RecipeDB` StaticData that maps each `BuildingType` to a production rule, and the per-tick `production_system` that advances an `Eleclipse` ProductionState on every Building entity. Miner buildings have no inputs, so they always produce on completion; Smelter / Mall / EnergySource are defined in the DB but their production waits for F5b (input consumption). MVP-visible effect: after several ticks a Miner's `OutputBuffer` holds a positive amount of `IronOre`.

**Problem:** Prior features build spatial structure (grid, placement, groups) but nothing runs "per tick" in the simulation sense. The first actual tick-driven behavior needs a compact set of components (Recipe, ProductionState, OutputBuffer) and a system in `Phase::Production` that reads the DB, advances state, and writes outputs. Deferring this to a later feature would leave the whole production loop inert.

**Architecture fit:**

1. **`recipes_db` module** (`StaticData`) - owns `RecipeDB { recipes: BTreeMap<BuildingType, RecipeDef> }`. Each `RecipeDef` lists `inputs`, `outputs`, and `duration_ticks`. MVP content:
   - Miner: inputs = `[]`, outputs = `[(IronOre, 1.0)]`, duration = 4
   - Smelter: inputs = `[(IronOre, 2.0)]`, outputs = `[(IronBar, 1.0)]`, duration = 4
   - Mall: inputs = `[]`, outputs = `[]`, duration = 1 (placeholder, F5b will extend)
   - EnergySource: inputs = `[]`, outputs = `[]`, duration = 1 (placeholder)
2. **`production` module** (`SimDomain`, `PRIMARY_PHASE = Phase::Production`). Writes nothing in terms of Resources; it only touches components. Declared reads: `RecipeDB`, `BuildingDB`. The installer registers a single system.
3. **Components:**
   - `ProductionState { progress: f32, active: bool }` - progress in `[0.0, 1.0]`, advances by `1.0 / duration_ticks` each tick when `active`.
   - `OutputBuffer { slots: BTreeMap<ResourceType, f32> }` - stores produced amounts pending collection (F6).
   - `InputBuffer { slots: BTreeMap<ResourceType, f32> }` - defined but unused in F5a (present for component coverage / determinism).
4. **`ResourceType` enum**: `Wood, Stone, IronOre, IronBar, Coal` (MVP set). Derives `Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd` for map keys.
5. **Component attachment:** on every tick, the production system queries Building entities that do NOT yet have `ProductionState`; for each, inserts the triple `(ProductionState, OutputBuffer, InputBuffer)` by looking up the Recipe in `RecipeDB`. Lazy attachment avoids a second drain path in F4.
6. **Advancement logic** (`production_system`):
   - For each Building with attached ProductionState:
     - If `!active`: if recipe has `inputs.is_empty()` then `active = true`; otherwise leave as-is (Smelter waits until F5b).
     - If `active`: `progress += 1.0 / duration_ticks`. When `progress >= 1.0`, add outputs to `OutputBuffer`, reset `progress = 0.0`, set `active = false` (will re-trigger next tick if inputs still satisfied).

**Acceptance criteria:**

- AC1: `RecipeDB.recipes.len() == 4` after Harness build. Each MVP `BuildingType` has an entry.
- AC2: After placing a Miner and running 5 ticks, the Miner entity has `ProductionState`, `OutputBuffer`, and `InputBuffer` components attached.
- AC3: After placing a Miner and running `duration_ticks + 2` ticks (e.g. 6), the Miner's `OutputBuffer.slots[IronOre]` is `>= 1.0`.
- AC4: A Smelter placed without F6 manifold never produces: `OutputBuffer.slots[IronBar]` remains 0.0 after any number of ticks (no input arrives).
- AC5: A building type (Mall, EnergySource) with empty outputs never increments OutputBuffer but still advances ProductionState.progress each tick (placeholder behavior is documented; F5b may rework).
- AC6: `ResourceType` derives `Hash, Ord, PartialEq, Eq` - compile-time verified via `BTreeMap<ResourceType, u32>` and `HashMap<ResourceType, u32>` construction.
- AC7: Single-writer holds on `RecipeDB` - second StaticData claiming `writes: names![RecipeDB]` panics.
- AC8: All 87 prior tests still pass.

**Implementation constraints (review-only):**

- `RecipeDef.inputs` and `.outputs` are `Vec<(ResourceType, f32)>` (not a BTreeMap) - tuples preserve author ordering and match the F4 BuildingDB shape.
- `OutputBuffer` / `InputBuffer` use `BTreeMap<ResourceType, f32>` for stable iteration.
- `production_system` does NOT mutate `Grid` or `GroupIndex` - it writes only to components on Building entities.
- Duration rounds to tick count; a duration of 1 completes in one tick (progress +1.0). A duration of 0 is forbidden and would divide by zero - the `const` table must not contain zeroes.
- Float arithmetic uses `f32` across the module. Determinism: single-threaded, no SIMD auto-vectorization at default opt levels -> bit-exact across runs.

**Non-goals:**

- Input consumption (Smelter / synthesis). Deferred to F5b once manifold routing is in place.
- Recipe switching (player selecting which recipe a building runs). MVP: one recipe per BuildingType, hard-coded.
- Energy modulation of speed. Energy feature is F8.
- Quality multipliers. Biome-contextual quality is F2-land / F12.
- Manifold collection / distribution. Explicit F6 concern.
- UI feedback of production progress.

**Edge cases:**

- Miner placed at tick 1, queried at tick 1: ProductionState not yet attached (attachment fires at tick 2 because `Added<Building>` filter sees the component only after placement's Commands apply). AC2 uses 5 ticks for safety.
- Building destroyed mid-production: no destruction path in MVP, so irrelevant. Future: production components are removed along with the entity.
- Recipe with output already in OutputBuffer: accumulates, not overwrites. `slots.entry(r).or_default() += amount`.
- Concurrent placement of 100 Miners: attachment and advancement are O(n) per tick each; no batching, no panics.
- Building variant not in RecipeDB (future): production_system skips it. Warning log in impl. MVP has complete coverage so not tested.

---

<!-- feature:manifold -->
### F6: manifold

**Purpose:** Aggregate every building output inside a group into a shared `Manifold` pool. First system that spans multiple members of the same group: every Miner in a cluster dumps its `OutputBuffer` into the group's `Manifold.slots` each tick. F5b will add the distribute pass (fill InputBuffers from Manifold) so that a Smelter in the same cluster can finally consume; F6 ships collect-only.

**Problem:** F5a produces resources inside per-building OutputBuffers - they pile up forever without a collection path. F7 groups Buildings spatially but does not carry state. F6 marries the two: every Group entity gets a `Manifold` component that acts as the group's resource tank. Collect pass runs in `Phase::Manifold`, downstream of `Phase::Production` (outputs already refreshed this tick) and `Phase::Groups` (group topology already rebuilt).

**Architecture fit:**

1. **`manifold` module** (`SimDomain`, `PRIMARY_PHASE = Phase::Manifold`). No resource writes - operates only on components. Reads `GroupIndex` for diagnostics; declared because the module conceptually depends on the group topology even though at MVP the logic walks Components directly. Single system `manifold_collect_system` runs attach + drain in one pass.
2. **`Manifold` component** - `{ slots: BTreeMap<ResourceType, f32> }` attached to every Group entity. Created lazily on first tick after a Group spawns.
3. **Execution order** (guaranteed by `Phase::Groups -> Phase::Production -> Phase::Manifold` chain):
   - Phase::Groups: `group_formation_system` despawns old groups and spawns new ones. `Commands` apply at end of phase - Group entities exist before Phase::Production begins.
   - Phase::Production: `production_advance_system` deposits into OutputBuffers.
   - Phase::Manifold: `manifold_collect_system` runs. First pass: attach `Manifold::default()` to every Group entity that lacks one (via `Commands`). Second pass (same system): for every Building that has a `GroupMember`, drain its `OutputBuffer` into the group's `Manifold.slots`. Buildings without a GroupMember (transient - group_formation removed their GroupMember this tick but the next rebuild will re-attach) are skipped.
4. **No writes to OutputBuffer from other systems** - collect pass owns the drain. Production writes only when producing; manifold drains after production has written.

**Acceptance criteria:**

- AC1: After placing a Miner + ticking 10 times, the Group entity containing the Miner has a `Manifold` component with `slots[IronOre] > 0.0`.
- AC2: After collect runs, the Miner's `OutputBuffer.slots[IronOre]` is `0.0` - resources have moved from building to group pool.
- AC3: Two adjacent Miners in one group, ticked 15 times - the shared `Manifold.slots[IronOre]` equals the sum of both miners' production. Each miner's `OutputBuffer` is empty.
- AC4: Two disjoint groups each with a Miner - each group's `Manifold.slots[IronOre]` tracks only its own miner's output; no cross-pollination.
- AC5: Empty grid after many ticks - there are zero Group entities and zero Manifold components. No panic.
- AC6: All 93 prior tests continue to pass.

**Implementation constraints (review-only):**

- `Manifold` uses `BTreeMap` (determinism).
- `manifold_collect_system` signature:
  ```
  fn manifold_collect_system(
      mut commands: Commands,
      new_groups_q: Query<Entity, (With<Group>, Without<Manifold>)>,
      mut group_manifold_q: Query<&mut Manifold, With<Group>>,
      mut buildings_q: Query<(&GroupMember, &mut OutputBuffer)>,
  )
  ```
- Drain uses `std::mem::take(&mut output.slots)` to move the old BTreeMap out and replace with empty, then iterate into manifold - a single allocation swap per Building per tick.
- Entities spawned this tick via `commands.spawn(Manifold::default())` are not visible to `group_manifold_q.get_mut(..)` in the same system invocation (deferred Commands). Attach lands before drain **on the next tick** - first tick of a brand-new group does not collect, next tick does. AC1 ticks 10 times, so this is invisible.
- No interior mutability on `Manifold`.

**Non-goals:**

- Distribute pass (Manifold -> InputBuffer) - F5b.
- Priority / fairness policies when multiple input consumers compete. Round-robin, proportional, FIFO - all deferred.
- Manifold overflow / capacity limits.
- Cross-group transport (rune paths, pipes) - F9.
- Manifold observability / debug UI.

**Edge cases:**

- Miner placed at tick 1, checked at tick 2: production hasn't produced yet (duration 4), manifold is empty - correct.
- Building removed mid-production (future): transition to F5b covers this; F6 alone has no removal path.
- Group respawned by F7: old Manifold is on the now-despawned entity, new Manifold attaches fresh (empty) on next tick. Resources in the old pool are lost - acceptable because F7 fully rebuilds each tick, not an incremental split.
- Large collect with 1000 buildings: O(n) walk, no batching. MVP.
- Building without GroupMember (position-only or mid-rebuild): skipped silently.

---

<!-- feature:manifold-distribute -->
### F5b: manifold-distribute

**Purpose:** Close the production cycle: move resources from a group's `Manifold.slots` into its members' `InputBuffer.slots` so that Smelter-style buildings (ones with non-empty `Recipe.inputs`) can finally trigger. Completes the factory loop - after F5b a Miner next to a Smelter converts IronOre into IronBar automatically with no player intervention beyond placement.

**Problem:** F5a advances production only when every input is already on the building's InputBuffer, and F6 moves production outputs into the group pool. Nothing bridges pool -> next consumer. F5b closes that gap with a second system running in `Phase::Manifold`, explicitly ordered **after** the collect pass so outputs from this tick are eligible for distribution the same tick.

**Architecture fit:**

1. **`manifold` module extension** - `manifold_distribute_system` is added alongside the collect system. Both run in `Phase::Manifold`; intra-phase ordering is enforced with `.chain()`: `collect -> distribute`. Contract gains `reads: names![RecipeDB]`.
2. **Distribution policy (MVP greedy):**
   - For every Building with `Recipe + InputBuffer + GroupMember` that is **not** currently active (`!ProductionState.active`):
     - Look up the RecipeDef.
     - If `inputs.is_empty()`: skip (nothing to distribute).
     - For each `(resource, amount)` in inputs, compute `needed = amount - input_buffer.slots[resource]`. If all needed <= 0: already satisfied, skip.
     - Attempt to pull the shortfall from the group's Manifold. If the pool has enough of **every** needed resource, deduct from the pool and add to the building's InputBuffer.
     - If the pool cannot fully satisfy the recipe, leave InputBuffer unchanged (no partial draws). MVP keeps atomicity: building either gets a full batch or nothing. No fairness between contenders.
3. **Ordering guarantees:**
   - collect (Phase::Manifold, chained first) drains OutputBuffers into Manifold.slots.
   - distribute (Phase::Manifold, chained second) moves Manifold -> InputBuffer.
   - Production advance runs next tick (Phase::Production < Phase::Manifold in chain order means production runs BEFORE manifold within the tick; the InputBuffer population is therefore consumed on the following tick).
4. **No new modules, no new components.** Pure extension of manifold.

**Acceptance criteria:**

- AC1: After placing a Miner at `(5, 5)` and a Smelter at `(5, 6)` (cardinal adjacent -> same group), ticking 30 times, the Smelter's `OutputBuffer.slots[IronBar]` is `>= 1.0`.
- AC2: In the same setup, after 30 ticks the group's Manifold still holds no more IronOre than the Smelter's current InputBuffer could not absorb - i.e. IronOre flows through the pool, not pools up indefinitely. Strictly: `manifold.slots[IronOre] < 10.0` after 30 ticks.
- AC3: A lone Smelter (no Miner in the group) after 30 ticks has `OutputBuffer.slots[IronBar] == 0.0` - still blocked on inputs as in F5a AC4.
- AC4: Two Miners + one Smelter in one group, 40 ticks - Smelter has produced multiple IronBars (`>= 2.0`), Manifold.slots[IronOre] remains bounded (`<= 10.0`).
- AC5: No regression - 98 prior tests continue to pass.

**Implementation constraints (review-only):**

- Distribution is **atomic per recipe cycle**: all inputs for one recipe start are drawn in a single pass or none are drawn. This prevents deadlocks where two buildings each take half an input and leave each other starved.
- Distribution does not consider priority, building order, or turn-taking. First Building the query iterator yields wins; BTreeMap iteration order provides determinism.
- `manifold_distribute_system` signature:
  ```
  fn manifold_distribute_system(
      db: Res<RecipeDB>,
      mut groups_q: Query<&mut Manifold, With<Group>>,
      mut buildings_q: Query<(&Recipe, &ProductionState, &GroupMember, &mut InputBuffer)>,
  )
  ```
- The two systems (`collect`, `distribute`) are added via `ctx.add_system((collect, distribute).chain())` inside the module installer.
- `ProductionState` is read-only in distribute - only production_advance_system flips `active`.
- Float arithmetic: deducts are `*slots.entry(*r).or_default() -= amount`, strictly greater-equal pre-check guarantees no negative pool.

**Non-goals:**

- Fairness policies. A follow-up feature can replace greedy first-come with round-robin / proportional / priority-based.
- Cross-group distribution (rune paths / pipes). F9 territory.
- Quality-aware distribution (HIGH vs NORMAL preference). Resolved when F14 opus-quality lands.
- Energy modulation. F8.
- Partial fills or buffer spillover.

**Edge cases:**

- Recipe with zero inputs (Miner, Mall, EnergySource): distribute skips - nothing to move. Collect still drains outputs normally.
- Manifold empty: distribute walks all buildings, none satisfied, pool unchanged.
- InputBuffer already full (building already has amount >= required): no additional draw, pool unchanged.
- Multiple Smelters in one group, one Miner: the first Smelter the BTreeMap iterator serves gets the batch; the second waits for the next complete batch. Determinism preserved by BTreeMap order.
- Active building (`ProductionState.active == true`): distribute skips - inputs already consumed at cycle start, adding more would create drift. Correct behavior.

---

<!-- feature:building-render -->
### F-building-render: building-render

**Purpose:** First visible trace of placement + production inside the render window. A View module that walks every `Building` entity and spawns a matching upright cuboid on the scene render layer, colored by `BuildingType`. Complements `world_render` (terrain + veins) and makes `world_render_smoke` demonstrable: placing a Miner + Smelter + Mall in the example produces distinct volumetric blocks on the iso-projected pixel-art map.

**Problem:** F19 renders terrain and veins but ignores Buildings - a placed Miner is invisible, so the whole sim loop from placement onward has no on-screen evidence. F-building-render closes that gap with a second View module that owns a dedicated `BuildingSceneCache`, diff-syncs `Cuboid` meshes against live Building entities each tick, and paints each BuildingType in a reserved color with a reserved height so the silhouette in iso projection is recognizable and the underlying tile colour peeks around the base.

**Modules:**

1. **`building_render` module** (`View`).
   - Reads: no sim resources required (walks components via Query).
   - Writes: `BuildingSceneCache { entities: BTreeMap<Entity, Entity>, synced_frames: u64 }`.
   - Metrics: gauge `building_render.sprites`.

**Tile-to-mesh mapping:**

- `BUILDING_WORLD_SIZE = 0.7` (footprint smaller than `TILE_WORLD_SIZE = 1.0`, so the tile peeks around the base). Height varies per `BuildingType` via `building_height(btype)`: Miner 0.5, Smelter 1.0, Mall 1.5, EnergySource 1.2.
- World position reuses `tile_world_pos(x, y)` from `world_render::palette` (imported; no local duplicate). Translation adds `Vec3::new(0.0, building_height(btype) / 2.0, 0.0)` so the cuboid rests on the ground plane instead of intersecting it.

**Color palette:**

| BuildingType | Color (sRGB hex) |
|---|---|
| Miner | d4a54a |
| Smelter | c86428 |
| Mall | a850c8 |
| EnergySource | 64d6ff |

**Acceptance criteria:**

- AC1: After `Harness` build with building-render module + placing a Miner, ticking twice, `BuildingSceneCache.entities.len() == 1` and maps the Miner entity to a freshly-spawned mesh entity.
- AC2: Placing four different types (Miner, Smelter, Mall, EnergySource) produces four mesh entities after two ticks.
- AC3: `BuildingSceneCache.entities` uses `BTreeMap` (determinism).
- AC4: Registering a second View module with `writes: names![BuildingSceneCache]` panics with `"single-writer"`.
- AC5: `cargo run --example world_render_smoke SCREENSHOT=1` - manual validation - PNG shows terrain + veins + several placed Building cuboids in distinct colors and heights, silhouetted against the iso-projected ground plane.
- AC6: All 98 prior tests still pass.

**Implementation constraints (review-only):**

- `BuildingSceneCache` uses `BTreeMap`. Zero interior mutability.
- Mesh spawn uses `Mesh3d(meshes.add(Cuboid::new(BUILDING_WORLD_SIZE, building_height(btype), BUILDING_WORLD_SIZE)))` + `MeshMaterial3d(materials.add(StandardMaterial { base_color: building_color(btype), unlit: true, ..default() }))` on `RenderLayers::layer(1)` (scene layer that feeds F18's iso 3D camera).
- Diff sync: every tick, walk Building Query, compare against cache, spawn missing / despawn removed. O(n) per tick; n = number of Buildings.
- Mesh and material handles are shared per `BuildingType` via a local `BTreeMap<BuildingType, (Handle<Mesh>, Handle<StandardMaterial>)>` accumulator inside the system.
- Same Commands-based roundtrip pattern as `world_render_system`.

**Non-goals:**

- Impostor textures (albedo + normal + depth).
- Per-pixel lighting on buildings. `unlit: true` keeps each face one colour - required so F22 outline (Sobel over luminance) still catches silhouettes cleanly.
- Animation / procedural effects.
- Selected / hover highlights (F21).
- Showing production progress / manifold state on the cuboid.

**Edge cases:**

- Building despawn (future): sync loop catches missing entity, despawns paired mesh.
- Building placed but still not visible in Query this tick: mesh appears next tick.
- Very large building count (>1k): full-scan sync is O(n) per tick, acceptable for MVP.

---

### F-render-v2: pixel-art pipeline v2

**Purpose:** Bring the render-pipeline from a black framebuffer + passthrough blit (F18 v0 / F22) to a visible pixel-art look matching `docs/VISUALS.md`: flat-shaded banded toon lighting on every scene mesh, Sobel edge outline over the low-res framebuffer, per-channel posterization, pixel-perfect nearest-neighbour upscale. Consolidates F18/F19/F22/F-building-render updates landed on 2026-04-18.

**Problem:** v0 shipped an alive pipeline but nothing rendered through it - `world_render_smoke` bypassed the plugin entirely and drew into its own perspective camera with PBR `StandardMaterial` + `DirectionalLight`, producing a flat coloured diamond with no visual identity. `RenderPipelineConfig.toon_bands` and `posterize_levels` existed as fields but had no consumer. v2 wires the missing pieces: a `ToonMaterial` Material3d baked onto every terrain tile, vein marker and building cuboid; a `PostProcessMaterial` Material2d running Sobel-over-luminance + posterize on the low-res blit; per-cell elevation jitter on terrain height so neighbouring same-kind tiles stagger vertically and their side faces actually read under iso projection.

**Modules delta (no new PTSD modules, refactor of existing surface):**

1. **`render_pipeline` crate** - adds `ToonMaterial` (Material3d, `uniform: ToonParams { base_color, ambient, sun_dir, bands }`) and `PostProcessMaterial` (Material2d, `uniform: PostProcessParams { outline_color, outline_threshold, posterize_levels, outline_enabled }`). Both shaders embedded via `embedded_asset!`. `RenderPipelinePlugin` adds `MaterialPlugin::<ToonMaterial>` + `Material2dPlugin::<PostProcessMaterial>`, spawns an iso ortho `Camera3d` on `RenderLayers::layer(1)` rendering into the low-res image, a `Camera2d` on `layer(2)` rendering the blit quad, and a fullscreen `Mesh2d + MeshMaterial2d<PostProcessMaterial>` that Sobel-posterize-outputs the low-res target. Pixel-perfect upscale enforced by flooring the scale factor (`scale.floor().max(1.0)`).
2. **`RenderPipelineConfig`** - defaults flipped: `outline_enabled: true`, `outline_threshold: 0.18`, `toon_bands: 5`, `posterize_levels: 10`. Derive relaxed from `PartialEq, Eq` to `PartialEq` (`LinearRgba` / `Vec3` fields introduce `f32` which breaks `Eq`).
3. **`world_render`** - every tile uses a single shared unit cuboid mesh with per-cell `Transform::scale.y = cell_top_height(cell)`. `terrain_height + cell_elevation_offset(elevation)` produces ±0.35 jitter on top of the base kind height, so same-kind patches stagger visibly. `terrain_top_y` now takes `TerrainCell` (not `TerrainKind`) so stacked objects respect the jitter. Meshes spawn with `RenderLayers::layer(1)` and `MeshMaterial3d<ToonMaterial>`.
4. **`building_render`** - cuboids use `ToonMaterial` on `RenderLayers::layer(1)`; building base Y follows the jittered terrain top via the new `terrain_top_y(cell)` signature.
5. **`core::harness`** - `init_asset::<ToonMaterial>()` + `init_asset::<PostProcessMaterial>()` so headless tests can spawn the new asset types without a full render app.
6. **`world_render_smoke`** - removes its private camera/light setup. Registers `RenderPipelineConfigModule` and adds `RenderPipelinePlugin` so the full chain (scene camera -> iso ortho -> toon material -> low-res target -> post-process -> upscale) runs end-to-end.
7. **`outline.rs/outline.wgsl`** removed; replaced by `post_process.rs/post_process.wgsl`.

**Acceptance criteria:**

- AC1: `RenderPipelineConfig::default()` returns `{ low_res_width: 480, low_res_height: 270, outline_enabled: true, outline_threshold: 0.18, toon_bands: 5, posterize_levels: 10, outline_color: near-black }`.
- AC2: `PostProcessMaterial` and `PostProcessParams` are public, `PostProcessParams::default()` returns `{ outline_threshold: 0.18, outline_color: BLACK, posterize_levels: 8.0, outline_enabled: 1.0 }`. Both derive `Clone, Debug` and `PostProcessParams` also `PartialEq` + `ShaderType`.
- AC3: `ToonMaterial::default()` returns a material with `bands = 5`, `sun_dir` normalized from `(-1.0, -1.5, -0.3)`, ambient approximately `(0.20, 0.22, 0.28)` linear.
- AC4: `Harness::new()` registers `Assets<ToonMaterial>` and `Assets<PostProcessMaterial>` - headless harness tests that spawn scene meshes through `world_render` or `building_render` do not panic with "resource does not exist".
- AC5: `world_render` spawns exactly one `Mesh3d` entity per grid cell (4096 for 64×64), every tile carries `RenderLayers::layer(1)` and `MeshMaterial3d<ToonMaterial>`. Cache mapping stable via `BTreeMap`.
- AC6: `building_render` spawns `MeshMaterial3d<ToonMaterial>` with `RenderLayers::layer(1)`; base Y = `terrain_top_y(cell)` so buildings never float above or clip into the jittered tile top.
- AC7: `cargo run --example world_render_smoke SCREENSHOT=1` produces a PNG at `/tmp/claude-bevy-world_render_smoke.png` that visibly demonstrates: (i) outline contours between differently-coloured regions, (ii) posterized palette (discrete colour steps), (iii) banded toon shading showing a visibly different colour between top faces and the iso-visible side faces on buildings and mountain tiles. Manual validation.
- AC8: Existing 81+ headless tests continue to pass. `render_pipeline_config_smoke` and `render_outline_material_shape` updated to the v2 defaults; all other tests untouched.

**Non-goals:**

- Depth/normal prepass-based outline (Roystan / Roberts cross). Current outline is Sobel-over-luminance on the low-res colour target - cheap and single-pass. Depth-aware edges are a fast-follow when the current look plateaus.
- Pixel-snap temporal stabilisation (camera view-aligned snap + inverse shift). Static camera makes this irrelevant until pan/zoom lands with F21.
- Impostor sprites (albedo + normal + depth textures). `ToonMaterial` reads mesh-geometry normals; impostors remain a full-scale follow-up.
- Runtime tuning UI for `outline_threshold` / `toon_bands` / `posterize_levels`. Defaults hardcoded until a settings panel exists (F21+).
- Shadow maps. `ToonMaterial` uses a single directional NdotL; no shadow casters.
- Outline colour modulation from config. `RenderPipelineConfig.outline_color` exists but `PostProcessParams::default()` retains `BLACK`; runtime override happens only in `setup_pipeline` which reads the config once at plugin build.

**Edge cases:**

- `ToonMaterial` on a mesh without vertex normals: `in.world_normal` is undefined, `normalize` yields NaN, fragment becomes black/undefined. All MVP scene meshes (`Cuboid`, `Sphere`) carry normals - no impact today.
- Outline false-positives inside same-kind tiles: prevented by the luminance palette (see §VISUALS.md) where no two neighbouring terrain kinds share brightness. Grass neighbours Water and Sand, all have distinct luminance.
- Elevation jitter producing negative heights: `cell_top_height` clamps to `0.05` minimum.
- Window resolution below low-res: `scale.floor().max(1.0)` returns `1.0`, low-res target renders 1:1 with aliasing from the edges; acceptable degraded state.
- Multiple `RenderPipelinePlugin` registrations: Bevy's standard plugin-dedup panic applies.

**Implementation constraints (review-only):**

- `ToonParams` fields are plain data (`LinearRgba`, `Vec3`, `u32`); zero interior mutability. Uniform layout handled by `encase`/`ShaderType` derive.
- Shader paths are embedded via `embedded_asset!` macro; never loaded at runtime.
- `RenderPipelinePlugin::build` reads `RenderPipelineConfig` once via `Option<Res<RenderPipelineConfig>>` and panics with substring `"RenderPipelineConfig resource missing"` if absent.
- Pixel-perfect scale enforcement: `scale_x.min(scale_y).floor().max(1.0)` in `fit_blit_quad_to_window`. Fractional scales are never applied.
- `DepthPrepass + NormalPrepass` components are NOT attached to the scene camera in v2 - they were allocated in a pre-v2 draft but never sampled, wasting GPU on unused passes. When a depth-aware outline lands they get reintroduced together with a custom `ViewNode`.
- `bevy::sprite_render::Material2d` is the correct 0.18 path for `Material2d` (the `bevy::sprite` crate no longer carries it; `sprite_render::prelude::*` re-exports into `bevy::prelude`).

---
