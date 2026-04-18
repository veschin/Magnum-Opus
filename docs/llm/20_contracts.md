---
id: module-contracts
kind: spec
touches: magnum_opus/src/core/
---

# Module contracts

Four archetypes, four traits. One `ModuleRegistry` in core enforces
cross-cutting invariants at `App` build time. One `InstallCtx` per archetype
enforces per-module contract coverage at install time.

See also: [90_lessons.md](90_lessons.md), [21_sketches.md](21_sketches.md).

## Archetypes

| Archetype    | Schedule     | Installer        | Purpose |
|--------------|--------------|------------------|---------|
| `SimDomain`  | `Update`     | `SimInstaller`   | owns a slice of simulation state; mutates per tick |
| `StaticData` | `Startup`    | `DataInstaller`  | loads read-only reference data from file or constant |
| `View`       | `PostUpdate` | `ViewInstaller`  | read-only projection of sim; may own view-private resources |
| `InputUI`    | `PreUpdate`  | `InputInstaller` | reads input and sim; pushes commands; may own UI-private resources |

Modules never receive `&mut App`. They receive a scoped installer that only
exposes operations their contract permits.

## Phase pipeline

`Phase` defines 11 ordered slots in `Update`, chained by `CorePlugin`:

```
Commands -> World -> Placement -> Groups -> Power -> Production
  -> Manifold -> Transport -> Progression -> Metrics -> End
```

- `Phase::Commands` drains `CommandBus<T>` queues.
- `Phase::Metrics` is where each module publishes tick metrics.
- `Phase::End` increments the global `Tick`.

`SimDomain::PRIMARY_PHASE` is the module's owning phase. `SimInstaller::add_system`
automatically attaches `.in_set(PRIMARY_PHASE)`. For secondary roles (draining
commands, publishing metrics), use `add_system_in(phase, system)` with an
explicit phase.

## Identifiers - `TypeKey` and `names!`

Contract slots list `TypeKey` values, each a `(TypeId, diagnostic_name)` pair.
Identity (equality, hashing, single-writer lookup) uses `TypeId`. The string
is diagnostic-only: two types with the same simple name (`a::Grid` vs
`b::Grid`) produce distinct `TypeKey`s.

```rust
use magnum_opus::names;

fn contract() -> SimContract {
    SimContract {
        writes:       names![Grid],
        commands_in:  names![PlaceTile],
        messages_out: names![TilePlaced, PlaceRejected],
        ..SimContract::EMPTY
    }
}
```

`names!` is the only supported way to build contract slots. It expands to a
`const` array of `TypeKey::new::<T>(stringify!(T))`, so the types must exist
in scope.

## Traits

```rust
pub trait SimDomain: 'static + Send + Sync {
    const ID: &'static str;
    const PRIMARY_PHASE: Phase;
    fn contract() -> SimContract;
    fn install(ctx: &mut SimInstaller);
}

pub trait StaticData: 'static + Send + Sync {
    const ID: &'static str;
    fn writes()  -> &'static [TypeKey];
    fn metrics() -> &'static [MetricDesc];
    fn install(ctx: &mut DataInstaller);
}

pub trait View: 'static + Send + Sync {
    const ID: &'static str;
    fn reads()   -> &'static [TypeKey];
    fn writes()  -> &'static [TypeKey];   // view-private resources
    fn metrics() -> &'static [MetricDesc];
    fn install(ctx: &mut ViewInstaller);
}

pub trait InputUI: 'static + Send + Sync {
    const ID: &'static str;
    fn reads()        -> &'static [TypeKey];
    fn writes()       -> &'static [TypeKey];  // UI-private resources
    fn commands_out() -> &'static [TypeKey];
    fn metrics()      -> &'static [MetricDesc];
    fn install(ctx: &mut InputInstaller);
}
```

`SimContract`:

```rust
pub struct SimContract {
    pub reads:        &'static [TypeKey],
    pub writes:       &'static [TypeKey],
    pub commands_in:  &'static [TypeKey],
    pub messages_in:  &'static [TypeKey],
    pub messages_out: &'static [TypeKey],
    pub metrics:      &'static [MetricDesc],
}
```

`SimContract::EMPTY` provides a zeroed default for struct-update syntax.

Sim modules do not produce commands. The command flow is strictly
`InputUI -> SimDomain`. Only `InputInstaller` has `emit_command`.

## Installer methods

Each installer exposes only archetype-appropriate operations. Every mutating
method verifies the target type is declared in the corresponding contract slot
and records the call. After `install` returns, the installer's `finalize()`
asserts that every declared slot was exercised - a contract that lies is a
build-time panic.

```rust
// SimInstaller
ctx.add_system(sys);                  // -> Update, in_set(PRIMARY_PHASE)
ctx.add_command_drain(sys);           // -> Update, in_set(Phase::Commands)
ctx.add_metric_publish(sys);          // -> Update, in_set(Phase::Metrics)
ctx.read_resource::<T>();             // requires T in reads
ctx.read_message::<T>();              // requires T in messages_in
ctx.write_resource::<T>();            // requires T in writes
ctx.insert_resource(value);           // requires T in writes
ctx.emit_message::<T>();              // requires T in messages_out
ctx.consume_command::<T>();           // requires T in commands_in

// DataInstaller
ctx.add_startup_system(sys);          // -> Startup
ctx.write_resource::<T>();            // requires T in writes
ctx.insert_resource(value);

// ViewInstaller
ctx.add_system(sys);                  // -> PostUpdate
ctx.read_resource::<T>();             // requires T in reads
ctx.write_resource::<T>();            // requires T in writes
ctx.insert_resource(value);

// InputInstaller
ctx.add_system(sys);                  // -> PreUpdate
ctx.read_resource::<T>();             // requires T in reads
ctx.write_resource::<T>();            // requires T in writes
ctx.insert_resource(value);
ctx.emit_command::<T>();              // requires T in commands_out
```

Each mutating call also enforces **single-exercise**: calling the same method
with the same type twice in one install panics. Declared slots are bound to
unique activations.

## Registration

```rust
app.add_sim::<MyDomain>();
app.add_data::<MyData>();
app.add_view::<MyView>();
app.add_input::<MyInput>();
app.finalize_modules();
```

In tests, use `Harness`:

```rust
let app = Harness::new()
    .with_sim::<MyDomain>()
    .with_input::<MyInput>()
    .build();
```

Each `add_*` validates the contract, records the module in `ModuleRegistry`,
declares its metrics in `MetricsRegistry`, builds a scoped installer, runs
`M::install(ctx)`, then asserts the installer's observations cover every
declared slot. `finalize_modules()` (auto-called by `Harness::build()`) runs
the cross-module closure checks and freezes the registry.

## Enforced invariants

Violations panic at `App` build time or at `finalize_modules()`, never at
tick time.

**Per-module (install-time):**

1. **Undeclared writes.** `ctx.write_resource::<T>()` / `insert_resource::<T>`
   panics if `T` is not in `contract.writes`.
2. **Undeclared reads.** `ctx.read_resource::<T>()` panics if `T` is not in
   `contract.reads`.
3. **Undeclared messages.** `ctx.emit_message::<T>()` panics if `T` is not in
   `contract.messages_out`. `ctx.read_message::<T>()` panics if `T` is not in
   `contract.messages_in`.
4. **Undeclared commands.** `ctx.consume_command::<T>()` panics if `T` is not
   in `contract.commands_in`. `ctx.emit_command::<T>()` panics if `T` is not
   in `contract.commands_out`.
5. **Single-exercise.** Calling any installer method with the same type twice
   in one install panics. Declared slots are bound to unique activations.
6. **Forgotten declarations.** After install returns, `finalize()` panics if
   any declared slot was not exercised by a matching installer call. Applies
   to `reads`, `writes`, `messages_in`, `messages_out`, `commands_in` on Sim;
   `reads`, `writes` on View; `reads`, `writes`, `commands_out` on Input;
   `writes` on Data.
7. **Primary phase must be used.** A Sim module whose install never calls
   `ctx.add_system(..)` panics: declaring `PRIMARY_PHASE` without placing a
   system there is a documentation lie.

**Cross-module (registry-time):**

8. **Unique module id.** No two modules share `ID`. The id `"core"` is
   reserved.
9. **Single writer.** Only one module of any archetype writes a given resource
   type. Keyed on `TypeId`, diagnosed by name. Core-owned resources (`Tick`,
   `ModuleRegistry`, `MetricsRegistry`) are claimed by `"core"` - user
   modules cannot write them.
10. **Closed messages.** Every `messages_in` has a matching `messages_out`.
11. **Single-producer messages.** Every `messages_out` type has exactly one
    producer across all modules.
12. **Closed commands.** Every `commands_in` has a matching `commands_out`.
13. **Single-producer commands.** Every `commands_out` type has exactly one
    producer across all modules.
14. **Single-consumer commands.** Every `commands_in` type has exactly one
    consumer across all modules.
15. **Closed reads.** Every `reads` entry has a matching `writes` somewhere.
16. **Unique metric names.** Metric names are globally unique. Convention:
    `<module_id>.<metric>`.
17. **Frozen registry.** `finalize_modules()` marks the registry frozen. Any
    subsequent `add_sim / add_data / add_view / add_input` panics.
18. **finalize_modules is mandatory.** `CorePlugin` adds a startup-time guard
    that panics on the first `app.update()` if the registry was never
    finalized. `Harness::build()` calls it automatically.

## Schedule placement

`SimInstaller::add_system` auto-routes to `Update` with `.in_set(PRIMARY_PHASE)`.
`DataInstaller::add_startup_system` routes to `Startup`. `ViewInstaller::add_system`
routes to `PostUpdate`. `InputInstaller::add_system` routes to `PreUpdate`. The
installer has no method for routing a system into another schedule - a View
cannot schedule a system into `Update` via the normal API.

## Accepted escape hatches

The core traps the sloppy-author failure class. A determined author can still
bypass the contract via Rust-level mechanisms the core cannot observe:

- **Exclusive systems.** A system with `fn(world: &mut World)` routed through
  any installer's `add_system` gets unconstrained world access and can
  `init_resource` / `insert_resource` any type, spoof messages, send commands.
  Bevy 0.18's `IntoScheduleConfigs` bound does not distinguish exclusive from
  non-exclusive systems via a public trait. No installer method blocks this.
  Mitigation: code review rejects `&mut World` in module systems unless
  explicitly justified.
- **`ResMut<T>` in reader roles.** A View or Input system requesting
  `ResMut<SimResource>` can mutate sim state - Bevy's scheduler only enforces
  exclusive borrow, not archetype boundaries. No installer-level compile-time
  check. Mitigation: code review + integration tests on single-writer state
  after PostUpdate.
- **Interior mutability.** A reader's `Res<T>` where `T` contains `Mutex`,
  `RefCell`, `AtomicU64`, etc. lets it mutate through a shared reference.
  Mitigation: contract-visible resources must be POD-style plain data;
  review rejects `Mutex`/`Atomic*` fields.
- **Metric ownership.** `MetricsRegistry::set` / `inc` take a `&'static str`
  name. The `owner` field on metric entries is cosmetic - any module holding
  `ResMut<MetricsRegistry>` can write any metric. Mitigation: review; tests
  on owner field sanity.
- **Startup system ordering.** `DataInstaller::add_startup_system` does not
  chain systems. If one Data module's startup reads another's resource, the
  order is undefined. Mitigation: Data modules should not depend on other
  Data modules' startup-mutated state. If they must, declare the dependency
  via `.after()` in the caller's own registration code.
- **Conditional compilation.** Two `#[cfg(feature = "...")]` module
  registrations with different contracts build separately and pass each build
  in isolation. The core cannot detect cross-build contract drift. Mitigation:
  CI must build all feature combinations and verify the closures agree.

These are documented, not silent - a module author who hits one of them must
acknowledge the hole.

## Metrics

```rust
pub struct MetricDesc {
    pub name: &'static str,
    pub kind: MetricKind,
}

pub enum MetricKind {
    Counter,  // monotonic
    Gauge,    // instantaneous value
    Rate,     // per-tick value
}
```

`MetricsRegistry::inc` panics on non-counter; `set` replaces gauge / rate;
`get(name) -> Option<f64>` for reads.
