---
id: module-walkthroughs
kind: spec
touches: magnum_opus/src/world_config/, magnum_opus/src/grid/
---

# Module walkthroughs

Real modules currently in the repo, annotated against the trait contracts
from [20_contracts.md](20_contracts.md). Both implement the only feature
that exists today: `F1 world-foundation` (see `.ptsd/docs/PRD.md`).

See also: [90_lessons.md](90_lessons.md), [10_scope.md](10_scope.md).

## `world_config` - StaticData

Inserts the run-wide configuration resource at startup. No systems, no
startup-side initialiser - `DataInstaller` only needs a `writes` coverage.

```rust
// magnum_opus/src/world_config/resource.rs
#[derive(Resource, Debug, Clone)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
}

// magnum_opus/src/world_config/module.rs
pub struct WorldConfigModule;

impl StaticData for WorldConfigModule {
    const ID: &'static str = "world_config";

    fn writes() -> &'static [TypeKey] { names![WorldConfig] }
    fn metrics() -> &'static [MetricDesc] { &[] }

    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(WorldConfig {
            width: 64,
            height: 64,
            seed: 0x9E3779B97F4A7C15,
        });
    }
}
```

Fits the contract: `insert_resource` satisfies the declared `writes` slot,
no other calls are needed. `DataInstaller::finalize()` checks that every
declared `writes` entry was exercised by a matching
`write_resource` / `insert_resource`.

## `grid` - SimDomain

Owns the tile grid. Reads `WorldConfig` on the first tick to set its
dimensions, publishes `grid.occupancy_count` each tick.

```rust
// magnum_opus/src/grid/resource.rs
#[derive(Resource, Default, Debug)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub occupancy: BTreeMap<(u32, u32), Entity>,
    pub dims_set: bool,
}

// magnum_opus/src/grid/position.rs
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position { pub x: u32, pub y: u32 }

// magnum_opus/src/grid/module.rs
pub struct GridModule;

impl SimDomain for GridModule {
    const ID: &'static str = "grid";
    const PRIMARY_PHASE: Phase = Phase::World;

    fn contract() -> SimContract {
        SimContract {
            writes: names![Grid],
            reads:  names![WorldConfig],
            metrics: &[MetricDesc {
                name: "grid.occupancy_count",
                kind: MetricKind::Gauge,
            }],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.read_resource::<WorldConfig>();
        ctx.add_system(grid_bootstrap_system);       // Phase::World
        ctx.add_metric_publish(grid_metrics_system); // Phase::Metrics
    }
}
```

The primary system is auto-wrapped with `.in_set(Phase::World)` by
`SimInstaller::add_system`. The metric publisher attaches to the reserved
`Phase::Metrics` slot via `add_metric_publish`; that helper does not count
as "using the primary phase", so the primary system is still required (and
present).

Single-writer: `Grid` is claimed by `grid` in the registry. Any second
module declaring `writes: names![Grid]` panics at `add_sim` / `add_data` /
`add_view` / `add_input` time.

Closed-reads: `WorldConfig` must be declared as `writes` by some module in
the same App. `WorldConfigModule` above is that producer; removing it while
keeping `GridModule` panics with `"closed-reads"` at `finalize_modules()`.

## What neither module does

- No commands. `grid` has no `commands_in`; there is no `PlaceTile`
  payload or drainer in the repo. Adding one is a future feature, not an
  implicit grid responsibility.
- No messages. `grid` does not publish `TilePlaced` / `PlaceRejected`.
- No rendering. Neither module spawns entities. `Grid.occupancy` stays
  empty.

Those extensions were removed on 2026-04-19; see [10_scope.md](10_scope.md).

## Harness usage in tests

Both modules are exercised through `core::Harness`, which registers a
minimal Bevy `App` with the asset plugins the tests need and calls
`finalize_modules()` automatically on `build()`.

```rust
use magnum_opus::core::*;
use magnum_opus::grid::{Grid, GridModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn grid_bootstrap_copies_dims_from_world_config() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .build();
    app.update();

    let grid = app.world().resource::<Grid>();
    assert!(grid.dims_set);
    assert_eq!(grid.width, 64);
    assert_eq!(grid.height, 64);
    assert!(grid.occupancy.is_empty());
}
```

`Harness::build()` consumes the builder, so each test owns its own `App`.
