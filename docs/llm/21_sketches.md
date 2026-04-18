---
id: module-sketches
kind: sketch
---

# Module sketches

Drafts showing how real modules fit the four archetype traits. Not committed code.
Purpose: stress-test the contract shape before writing production modules.

See also: [20_contracts.md](20_contracts.md).

All sketches use the `names!` macro: binds a list of Rust types to
`&'static [TypeKey]`, so identity is by `TypeId` and name collisions
(`a::Grid` vs `b::Grid`) resolve to distinct keys.

## `grid` - SimDomain

Owns the tile grid. Consumes `PlaceTile` commands. Emits `TilePlaced` / `PlaceRejected`.

```rust
use magnum_opus::{names, core::*};

pub struct GridModule;

impl SimDomain for GridModule {
    const ID: &'static str = "grid";
    const PRIMARY_PHASE: Phase = Phase::Placement;

    fn contract() -> SimContract {
        SimContract {
            writes:       names![Grid],
            commands_in:  names![PlaceTile],
            messages_out: names![TilePlaced, PlaceRejected],
            metrics: &[
                MetricDesc { name: "grid.tiles_occupied", kind: MetricKind::Gauge },
                MetricDesc { name: "grid.place_rejected", kind: MetricKind::Counter },
            ],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.consume_command::<PlaceTile>();
        ctx.emit_message::<TilePlaced>();
        ctx.emit_message::<PlaceRejected>();
        ctx.add_system_in(Phase::Commands, drain_place_commands);
        ctx.add_system(apply_placement);                        // -> Placement
        ctx.add_system_in(Phase::Metrics, publish_grid_metrics);
    }
}
```

Fits: multi-phase install works through `add_system` (primary phase, auto-wrapped)
and `add_system_in(phase, sys)` for secondary roles. The installer verifies that
every declared `writes` / `commands_in` / `messages_out` entry was wired.

## `placement_cursor` - InputUI

Converts pointer clicks on revealed, buildable tiles into `PlaceTile` commands.
Owns the cursor-grid mapping state.

```rust
use magnum_opus::{names, core::*};

pub struct PlacementCursor;

impl InputUI for PlacementCursor {
    const ID: &'static str = "placement_cursor";

    fn reads()        -> &'static [TypeKey] { names![Grid, Inventory] }
    fn writes()       -> &'static [TypeKey] { names![CursorGridPos, InputMode] }
    fn commands_out() -> &'static [TypeKey] { names![PlaceTile] }
    fn metrics()      -> &'static [MetricDesc] {
        &[MetricDesc { name: "placement_cursor.clicks", kind: MetricKind::Counter }]
    }

    fn install(ctx: &mut InputInstaller) {
        ctx.write_resource::<CursorGridPos>();
        ctx.write_resource::<InputMode>();
        ctx.emit_command::<PlaceTile>();
        ctx.add_system(handle_place_click);     // -> PreUpdate
    }
}
```

Fits: `Grid` and `Inventory` must be declared as `writes` by some sim module
(closed-reads check). `CursorGridPos` and `InputMode` are declared here and
claimed under single-writer. `PlaceTile` matches the `commands_in` side of the
grid module.

## `grid_render` - View

Spawns, despawns, and updates scene entities to mirror the `Grid` resource.
Owns a scene cache as view-private state.

```rust
use magnum_opus::{names, core::*};

pub struct GridRender;

impl View for GridRender {
    const ID: &'static str = "grid_render";

    fn reads()   -> &'static [TypeKey] { names![Grid] }
    fn writes()  -> &'static [TypeKey] { names![GridSceneCache] }
    fn metrics() -> &'static [MetricDesc] {
        &[MetricDesc { name: "grid_render.scene_entities", kind: MetricKind::Gauge }]
    }

    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<GridSceneCache>();
        ctx.add_system(sync_grid_to_scene);    // -> PostUpdate
    }
}
```

Fits: reads sim-owned `Grid`, writes view-private `GridSceneCache`.
Single-writer rule claims `GridSceneCache` under `grid_render` - any other
module declaring it panics at App build.

## `recipe_db` - StaticData

Loads recipes from an embedded RON file during `Startup`.

```rust
use magnum_opus::{names, core::*};

pub struct RecipeDbModule;

impl StaticData for RecipeDbModule {
    const ID: &'static str = "recipe_db";

    fn writes()  -> &'static [TypeKey] { names![RecipeDB] }
    fn metrics() -> &'static [MetricDesc] {
        &[MetricDesc { name: "recipe_db.recipes_loaded", kind: MetricKind::Gauge }]
    }

    fn install(ctx: &mut DataInstaller) {
        ctx.write_resource::<RecipeDB>();
        ctx.add_startup_system(load_recipes);   // -> Startup
    }
}
```

Fits: trivial.

## Covered by the current core

- `names!` binds contract slots to `TypeId` via `TypeKey`; name collisions
  on simple names resolve correctly.
- Each archetype has a scoped installer; modules never see `&mut App`.
- Install-time drift check: every declared slot must be exercised by the
  matching installer call.
- `CommandBus<T>` init is idempotent and driven through `consume_command` /
  `emit_command` - typed payload, not string-bound.
- Cross-module closures: closed-reads / closed-messages / closed-commands,
  single-producer messages, single-consumer commands, single-writer resources.
- `finalize_modules()` freezes the registry; late `add_*` panics.

## Known non-enforcements

- **Exclusive systems.** A system with `fn(world: &mut World)` routed through
  any `add_system` call can ignore the contract entirely.
- **Interior mutability.** Resources exposed via `Res<T>` with `Mutex`/`Atomic*`
  fields let readers mutate. The core sees only the outer type.
- **Conditional compilation.** `#[cfg(feature = "...")]` module registrations
  that diverge across builds are checked separately; the core cannot enforce
  cross-build agreement.

Review rules mitigate. See `20_contracts.md` § Accepted escape hatches.
