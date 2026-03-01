# Bevy 0.18 Ecosystem Research

Research date: 2026-03-01
Bevy 0.18.0 release date: 2026-01-13

## Summary Table

| Area | Crate(s) | Version | Bevy 0.18? | Recommendation |
|------|----------|---------|------------|----------------|
| Tilemap/Grid | `bevy_ecs_tilemap` | 0.18.1 | Yes | **Skip** — rendering crate, our grid is pure ECS |
| Tilemap/LDtk | `bevy_ecs_ldtk` | 0.14 | Yes | **Skip** — level editor integration, not needed |
| Pathfinding | `pathfinding` | 4.8 | N/A (pure Rust) | **Use** — A*, Dijkstra for transport routing |
| Data loading | `serde` + `ron` | serde 1.x, ron 0.12 | N/A (pure Rust) | **Use** — direct dependency, no wrapper needed |
| Data loading | `bevy_common_assets` | 0.15.0 | Yes | **Consider** — only if using Bevy asset pipeline |
| State machines | Bevy built-in States | (built-in) | Yes | **Use** — SubStates, ComputedStates, SystemSet |
| State machines | `seldom_state` | 0.15 | No (Bevy 0.17) | **Skip** — not 0.18-ready, Bevy built-in is enough |
| AI / Behavior | `bevy_behave` | 0.5.0 | Yes | **Consider** — behavior trees if creature AI is complex |
| AI / Behavior | `bevior_tree` | 0.10.0 | Yes | **Consider** — alternative behavior trees |
| AI / Utility | `big-brain` | 0.22.0 | No (archived, Bevy 0.15) | **Skip** — abandoned, archived Oct 2025 |
| AI / Utility | `bevy_observed_utility` | 0.2.0 | Likely (^0.18) | **Consider** — modern utility AI replacement |
| UI/Dev tools | `bevy_egui` | 0.39.1 | Yes | **Use** — proven egui integration |
| UI/Dev tools | `bevy-inspector-egui` | 0.36.0 | Yes | **Use** — ECS inspector, debugging |
| Noise | `fastnoise-lite` | 1.0.1 | N/A (pure Rust) | **Use** — fast, portable, no_std support |
| Noise | `noise` (noise-rs) | 0.9.0 | N/A (pure Rust) | **Alternative** — composable NoiseFn pipeline |
| Scheduling | Bevy SystemSet + Schedule | (built-in) | Yes | **Use** — already proven in spike |
| Testing | Bevy MinimalPlugins | (built-in) | Yes | **Use** — already proven in spike |

---

## 1. Tilemap / Grid

### bevy_ecs_tilemap v0.18.1

- **Repo:** https://github.com/StarArawn/bevy_ecs_tilemap
- **Bevy 0.18:** Yes, version-matched (0.18.x = Bevy 0.18)
- **What it does:** Tilemap rendering plugin. Each tile is an ECS entity. Supports isometric, hexagonal, chunked rendering, GPU animations, Tiled/LDtk integration.

### bevy_ecs_ldtk v0.14

- **Repo:** https://github.com/Trouv/bevy_ecs_ldtk
- **Bevy 0.18:** Yes (0.14 = Bevy 0.18, LDtk 1.5.3)
- **What it does:** LDtk level editor integration. Loads .ldtk projects as Bevy assets. Depends on `bevy_ecs_tilemap`.

### Recommendation: SKIP both

Both crates are rendering-focused tilemap solutions. Magnum Opus uses a simulation-first architecture where the grid is a pure ECS resource (`Grid` with `HashSet<(i32, i32)>` for occupancy). Tiles are entities with `Position` components. No tilemap renderer is needed until the rendering layer is built, and even then, we may want a custom isometric renderer with pixel-art shaders.

If we later need tilemap rendering, `bevy_ecs_tilemap` is the go-to choice with its isometric support and ECS-friendly design. But for now, the grid is just data.

---

## 2. Pathfinding

### pathfinding v4.8

- **Crate:** https://crates.io/crates/pathfinding
- **Bevy 0.18:** N/A — pure Rust, no Bevy dependency
- **MSRV:** Rust 1.77.2
- **Algorithms:** A*, BFS, DFS, Dijkstra, Fringe, IDA*, IDDFS, Edmonds-Karp (max flow), strongly connected components, cycle detection (Brent, Floyd)

### Recommendation: USE

This is exactly what we need for two systems:

1. **Transport routing** — find shortest path between building groups for rune paths/pipes. A* or Dijkstra on the grid graph.
2. **Creature movement** — pathfinding for creature entities moving across the map (territorial, invasive archetypes).

The crate is generic over its arguments, so it works with any coordinate type. No Bevy coupling needed — call `astar()` or `dijkstra()` inside a system, passing grid data. Pure function, determinism-friendly.

No Bevy-specific pathfinding wrapper is needed. The `pathfinding` crate is mature (v4.8), well-maintained, and has zero engine dependencies.

Usage pattern:
```rust
use pathfinding::prelude::astar;

fn route_system(grid: Res<Grid>) {
    let result = astar(
        &start,
        |&pos| grid.neighbors(pos).map(|n| (n, 1)),
        |&pos| heuristic(pos, goal),
        |&pos| pos == goal,
    );
}
```

---

## 3. Serialization / Data Loading

### serde + RON (direct dependencies)

- **serde:** v1.x — the standard Rust serialization framework
- **ron:** v0.12 — Rusty Object Notation, Rust-syntax-like data format
- **Bevy 0.18 note:** `ron` is no longer re-exported from `bevy_scene` or `bevy_asset` as of 0.18. Must be added as a direct dependency.

### bevy_common_assets v0.15.0

- **Repo:** https://github.com/NiklasEi/bevy_common_assets
- **Bevy 0.18:** Yes (v0.15.0 depends on bevy 0.18.0)
- **Formats:** RON, JSON, YAML, TOML, MessagePack, XML, CSV, CBOR
- **What it does:** Generic `AssetLoader` plugins. Define a `Deserialize` struct, register a `RonAssetPlugin::<MyType>::new(&["my.ron"])`, and files are loaded as Bevy assets automatically.

### Recommendation: USE serde + RON directly, CONSIDER bevy_common_assets later

For the simulation layer (BuildingDB, RecipeDB, BiomeDB, etc.), we do not need Bevy's asset pipeline at all. These are static data tables loaded at startup. Direct `serde` + `ron` deserialization is simpler and keeps the simulation layer engine-independent:

```rust
#[derive(Deserialize)]
struct RecipeDB {
    recipes: Vec<RecipeEntry>,
}

let db: RecipeDB = ron::from_str(include_str!("data/recipes.ron"))?;
```

This approach:
- Keeps game data in `.ron` files (readable, Rust-like syntax)
- Works in headless tests without Bevy asset server
- Preserves simulation-first principle (no engine dependency in data loading)
- Supports hot-reloading via file watcher if needed later

When we build the rendering layer and need hot-reloading through Bevy's asset system, `bevy_common_assets` provides a clean bridge. But for simulation, direct `ron::from_str` is better.

**YAML alternative:** If YAML is preferred over RON for human-edited content (we already use YAML for `ideas.yaml`), `serde_yaml` works the same way. RON is recommended for game data because its syntax maps directly to Rust structs.

---

## 4. State Machines / AI

### Bevy Built-in States (recommended baseline)

Bevy 0.18 provides a mature state system:

- **States** — standard app-wide finite states, changed via `NextState<S>` resource
- **SubStates** — child states that only exist when a parent state matches a condition. Example: `GamePhase::Battle` only exists when `AppState::InGame`
- **ComputedStates** — deterministically derived from other states. Pure function: `fn compute(sources) -> Option<Self>`
- **State-scoped entities** — `DespawnOnEnter<S>`, `DespawnOnExit<S>` for lifecycle management
- **SystemSet ordering** — already proven in our spike with `Phase` enum and `.configure_sets()`

For production states (Idle -> Working -> Outputting), Bevy's built-in states are overkill because those are per-entity, not app-wide. Instead, use a simple enum component:

```rust
#[derive(Component)]
enum ProductionState { Idle, Working(u32), Outputting }
```

Systems match on the enum and transition by replacing the component. This is simpler and more ECS-idiomatic than any state machine plugin.

### seldom_state v0.15

- **Repo:** https://github.com/Seldom-SE/seldom_state
- **Bevy 0.18:** No. Latest release (0.15) supports Bevy 0.17 only.
- **What it does:** Component-based state machine. Adds a `StateMachine` component with triggers and transitions. Per-entity states unlike Bevy's app-wide States.

### bevy_behave v0.5.0

- **Repo:** https://github.com/RJ/bevy_behave
- **Bevy 0.18:** Yes (Cargo.toml specifies `bevy = "0.18"`)
- **What it does:** Behavior trees with dynamic entity spawning for task nodes. Minimal overhead. Runs in `FixedPreUpdate`. Action nodes spawn entities, wait for status triggers, then despawn.
- **Performance:** Author reports 100k entities at max framerate in release mode.

### bevior_tree v0.10.0

- **Repo:** https://github.com/hyranno/bevior_tree
- **Bevy 0.18:** Yes (0.10 = Bevy 0.18 per README compatibility table)
- **What it does:** Behavior tree plugin. Nodes include conditionals, decorators, parallel composites, sequential composites, and tasks. Inspired by `seldom_state`.

### big-brain v0.22.0

- **Repo:** https://github.com/zkat/big-brain (archived Oct 2025, moved to Codeberg)
- **Bevy 0.18:** No. Latest version (0.22.0) supports Bevy 0.15 only. Abandoned.

### bevy_observed_utility v0.2.0

- **Repo:** https://github.com/ItsDoot/bevy_observed_utility
- **Bevy 0.18:** Listed as ^0.18 on Bevy Assets page
- **What it does:** Utility AI using ECS observers. Scorers evaluate world state, pickers select actions. Supports real-time and turn-based. Modern replacement for big-brain.

### Recommendation: USE Bevy built-in + plain enum components, SKIP external state machines

For Magnum Opus creature AI (5 archetypes: ambient, territorial, invasive, event-born, opus-linked), the behavior is simple enough that enum components + systems cover it:

- **Ambient:** wander randomly, flee from combat groups
- **Territorial:** patrol zone, attack intruders
- **Invasive:** pathfind to player buildings, attack
- **Event-born:** spawn at event, execute scripted sequence
- **Opus-linked:** spawn at opus milestone, behave like territorial

None of these require deep behavior trees or utility AI. A `CreatureBehavior` enum component with dedicated systems per archetype is simpler, testable, and deterministic.

If creature AI grows more complex later, `bevy_behave` (v0.5.0, Bevy 0.18 compatible) is the best option. It is actively maintained, lightweight, and ECS-native. Avoid `big-brain` (abandoned) and `seldom_state` (not Bevy 0.18 ready).

---

## 5. UI / Dev Tools

### bevy_egui v0.39.1

- **Repo:** https://github.com/vladbat00/bevy_egui
- **Bevy 0.18:** Yes (0.39.x = Bevy 0.18, egui 0.33)
- **What it does:** Full egui integration for Bevy. Immediate-mode GUI. Handles input routing, rendering, picking order.

### bevy-inspector-egui v0.36.0

- **Repo:** https://github.com/jakobhellermann/bevy-inspector-egui
- **Bevy 0.18:** Yes (0.36.0 = Bevy 0.18, released 2026-01-14)
- **What it does:** Inspector plugin for Bevy. Reflect-based entity/component inspector. World inspector, resource inspector, custom UIs.

### Recommendation: USE both

These are essential for development:

- **bevy_egui** — for building the chain manager, production calculator, dashboard, and any in-game debug UI. Immediate-mode rendering means zero boilerplate for data-heavy panels.
- **bevy-inspector-egui** — for development-time entity inspection. See component values, resource state, system timings. Critical for debugging ECS state.

Neither affects the simulation layer. They are rendering-layer dependencies only.

---

## 6. Procedural Generation / Noise

### fastnoise-lite v1.0.1

- **Crate:** https://crates.io/crates/fastnoise-lite
- **Bevy 0.18:** N/A — pure Rust, no engine dependency
- **Features:** Perlin, OpenSimplex, Cellular/Voronoi, Value noise. 2D and 3D. Domain warping. `no_std` support via `libm` feature. f32 by default, `f64` feature available.
- **Design:** Single `FastNoiseLite` struct with setter methods. Configure noise type, frequency, fractal settings, then sample with `get_noise_2d(x, y)`.

### noise (noise-rs) v0.9.0

- **Crate:** https://crates.io/crates/noise
- **Bevy 0.18:** N/A — pure Rust, no engine dependency
- **Features:** Perlin, Simplex, Worley/Cell, Value, RidgedMulti, Fbm, Billow, Turbulence. Composable `NoiseFn` trait — chain generators together.
- **Design:** Each noise type is a separate struct implementing `NoiseFn`. Combinators allow complex noise from simple parts. Image output feature for debugging.

### noise-functions (alternative)

- **Crate:** https://crates.io/crates/noise-functions
- **Updated:** January 2026
- **Design:** Functional approach vs fastnoise-lite's struct approach. `Sample<2>` trait. Composable. Better for known noise configurations.

### Recommendation: USE fastnoise-lite

For biome map generation in Magnum Opus:

- **fastnoise-lite** is the best fit. It is portable, fast, has a simple API, and covers all noise types we need (Perlin for terrain height, Cellular for biome boundaries, domain warping for organic shapes).
- **noise-rs** is more composable but over-engineered for our use case. We need one noise function per biome layer, not a complex noise pipeline.

Usage pattern for biome generation:
```rust
let mut noise = FastNoiseLite::with_seed(run_seed);
noise.set_noise_type(Some(NoiseType::OpenSimplex2));
noise.set_frequency(Some(0.02));

for x in 0..width {
    for y in 0..height {
        let value = noise.get_noise_2d(x as f32, y as f32);
        let biome = match value {
            v if v < -0.3 => Biome::Swamp,
            v if v < 0.1  => Biome::Forest,
            v if v < 0.5  => Biome::Plains,
            _             => Biome::Mountain,
        };
    }
}
```

Both crates are pure Rust with no engine coupling. Deterministic with seed. Safe for headless tests.

---

## 7. Scheduling / Phases

### Bevy Built-in Schedule System

Bevy 0.18 provides everything we need for the 10-phase tick pipeline:

**SystemSet** — group systems into named phases, order with `.before()` / `.after()` / `.chain()`:
```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    Input, World, Creatures, Energy,
    Production, Transport, Combat,
    Progression, Cleanup, Meta,
}

app.configure_sets(Update, (
    Phase::Input,
    Phase::World,
    Phase::Creatures,
    // ...
).chain());
```

**Key features already available:**
- `.chain()` — shorthand for linear ordering of sets
- `.in_set()` — assign systems to phases
- `configure_sets()` — define ordering constraints
- Run conditions — conditionally skip systems
- `remove_systems_in_set()` — new in 0.18, completely remove systems from schedule

**Already proven in spike:** Our `SimulationPlugin` uses 5 phases (Input, Groups, Power, Production, Manifold) with `SystemSet` ordering. Scaling to 10 phases is trivial — just add more enum variants and chain them.

### Recommendation: USE Bevy built-in (already using it)

No external scheduling crate is needed. Bevy's `SystemSet` with `.chain()` maps perfectly to our 10-phase pipeline. The spike already demonstrates this pattern.

---

## 8. Testing

### Bevy MinimalPlugins (built-in)

The established headless testing pattern works perfectly with Bevy 0.18:

```rust
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin::default());
    app
}

#[test]
fn test_production_output() {
    let mut app = test_app();
    // spawn entities, insert resources
    app.update(); // run all systems in phase order
    // query and assert
}
```

**What MinimalPlugins provides:**
- `ScheduleRunnerPlugin` — drives the update loop without a window
- Core ECS, App, scheduling infrastructure
- No rendering, no windowing, no GPU — works in CI, works in headless

**What it does NOT include (by design):**
- No `WinitPlugin` (window event loop) — not needed for tests
- No `RenderPlugin` — not needed, avoids GPU requirement
- No asset server — not needed if data is loaded directly via serde

**Time travel limitation:** By default, `app.update()` advances one tick. For time-dependent logic, either:
- Call `app.update()` multiple times in a loop
- Manually advance `Time` resource
- Use tick-based logic (we already do — `duration_ticks` in recipes)

### Recommendation: USE (already using it)

Our spike already has 8 tests using this exact pattern. It works. No additional testing crate is needed.

Key principles already established:
- No mocks for internal code
- Real ECS, real data, real systems
- Headless via `MinimalPlugins`
- One `app.update()` = one full tick through all phases

---

## Dependencies Summary

### Cargo.toml additions (when each area is needed)

```toml
[dependencies]
# Engine (already present)
bevy = { version = "0.18", default-features = false, features = ["std"] }

# Data loading (add when building RecipeDB, BuildingDB, etc.)
serde = { version = "1", features = ["derive"] }
ron = "0.12"

# Pathfinding (add when building transport routing or creature movement)
pathfinding = "4.8"

# Noise (add when building biome map generation)
fastnoise-lite = "1.0.1"

# UI — rendering layer only (add when building dev tools)
# bevy_egui = "0.39"
# bevy-inspector-egui = "0.36"
```

### Not adding (and why)

| Crate | Why not |
|-------|---------|
| `bevy_ecs_tilemap` | Rendering crate, simulation uses pure ECS grid |
| `bevy_ecs_ldtk` | Level editor integration, maps are procedurally generated |
| `seldom_state` | Not Bevy 0.18 compatible, Bevy built-in States is sufficient |
| `big-brain` | Archived/abandoned (Oct 2025), stuck on Bevy 0.15 |
| `bevy_behave` | Not needed now, creature AI is simple enum-based. Revisit if AI complexity grows |
| `bevy_common_assets` | Not needed for simulation-first data loading. Revisit for hot-reload in rendering layer |

---

## Sources

- [Bevy 0.18 Release Notes](https://bevy.org/news/bevy-0-18/)
- [Bevy 0.17 to 0.18 Migration Guide](https://bevy.org/learn/migration-guides/0-17-to-0-18/)
- [bevy_ecs_tilemap — GitHub](https://github.com/StarArawn/bevy_ecs_tilemap)
- [bevy_ecs_ldtk — GitHub](https://github.com/Trouv/bevy_ecs_ldtk)
- [pathfinding — crates.io](https://crates.io/crates/pathfinding)
- [fastnoise-lite — crates.io](https://crates.io/crates/fastnoise-lite)
- [noise-rs — GitHub](https://github.com/Razaekel/noise-rs)
- [seldom_state — GitHub](https://github.com/Seldom-SE/seldom_state)
- [big-brain — GitHub (archived)](https://github.com/zkat/big-brain)
- [bevy_behave — GitHub](https://github.com/RJ/bevy_behave)
- [bevior_tree — GitHub](https://github.com/hyranno/bevior_tree)
- [bevy_observed_utility — GitHub](https://github.com/ItsDoot/bevy_observed_utility)
- [bevy_egui — GitHub](https://github.com/vladbat00/bevy_egui)
- [bevy-inspector-egui — GitHub](https://github.com/jakobhellermann/bevy-inspector-egui)
- [bevy_common_assets — GitHub](https://github.com/NiklasEi/bevy_common_assets)
- [Bevy Scheduling — Unofficial Cheat Book](https://bevy-cheatbook.github.io/programming/schedules.html)
- [Bevy States — docs.rs](https://docs.rs/bevy/latest/bevy/state/index.html)
- [MinimalPlugins — docs.rs](https://docs.rs/bevy/latest/bevy/struct.MinimalPlugins.html)
