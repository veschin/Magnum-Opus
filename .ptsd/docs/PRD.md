# Magnum Opus - Product Requirements Document

Greenfield rewrite v3. The v1 implementation was deleted on 2026-04-17; the v2 pass (2026-04-18) added landscape generation, production, and a render-pipeline v2 but was rolled back when the scope drifted past what the owner asked for. This document now describes only what is actually in the repo: core + world_config + grid.

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

Features are units of product scope. Each feature ships one or more modules.

### Phase 1 - Spatial foundation

| ID | Title | Archetypes | Status | Depends on |
|---|---|---|---|---|
| **F1** | `world-foundation` | StaticData + SimDomain | implemented | - |

Further features (terrain generation, placement commands, production, render pipeline, UI) existed as specs in earlier PRD revisions and as partially landed code on `main`. They were rolled back on 2026-04-19 at the owner's direction because the scope exceeded the explicit ask ("core + grid"). New feature specs go here when the owner requests them.

---

<!-- feature:world-foundation -->
### F1: world-foundation

**Purpose:** Provide `WorldConfig` and `Grid` resources as the shared spatial substrate. Every later feature that touches coordinates or the run seed depends on this.

**Problem:** Downstream features need two pieces of state before they can run: a deterministic seed + world dimensions, and a grid resource whose writer is already claimed under single-writer discipline. If each feature fabricated its own seed, cross-module generators would produce inconsistent state from the same "run." If each feature also tried to write `Grid`, single-writer would fire on the second registration. F1 settles both: `world_config` (StaticData) writes `WorldConfig`; `grid` (SimDomain, `Phase::World`) writes `Grid` forever. The grid resource stays empty - occupancy insertion arrives with a future placement feature that attaches a `CommandBus<PlaceTile>` drainer via `add_command_drain`, preserving single-writer.

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
- `Grid.occupancy` insertion. Writers live in a future placement feature and drain a `CommandBus<PlaceTile>` from `Phase::Commands`. F1 ships an empty occupancy map and a stub bootstrap.
- Terrain data, resource veins, fog, rendering, or any content from later features.
- Biome variation, run-specific config, seed randomization.
- Grid spatial queries or helper API (lookups live in the consuming feature).

**Edge cases:**

- `WorldConfigModule` registered without `GridModule`: no panic. `WorldConfig` exists, no `Grid`. Valid intermediate state for features that read `WorldConfig` but not `Grid`.
- Registering `WorldConfigModule` twice via `app.add_data::<...>()`: panics with substring `"duplicate module id"` (registry invariant from `registry.rs`).
- `Grid` queried before first `app.update()`: returns `Grid::default()` - `dims_set == false`, `width == 0`, `height == 0`, empty occupancy. Consumers must check `dims_set` before acting on dimensions.
- `grid_metrics_system` on tick 1 before `grid_bootstrap_system` in the same tick: since both are in `Update` and the metric publishes via `add_metric_publish` (`Phase::Metrics`, after `Phase::World`), the bootstrap always runs first. Metric reads `occupancy.len() == 0` regardless of `dims_set`.
- `Harness::build()` consumes `self` (move semantics) - it cannot be invoked twice on the same Harness. Attempting to register a second module with the same id via `app.add_data::<>()` / `app.add_sim::<>()` is covered by edge case 2 (`"duplicate module id"`).
