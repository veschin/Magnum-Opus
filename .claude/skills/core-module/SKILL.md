---
name: core-module
description: Invoke when writing or modifying a Bevy module (SimDomain/StaticData/View/InputUI) on the magnum_opus core. Enforces contract-first workflow.
---

# Developing on the magnum_opus core

The core lives in `magnum_opus/src/core/`. It is hardened against the v1 failure class
(shared ownership, string-based drift, skip_ flags). Every module is a declarative
contract plus a scoped installer. Break the contract and `cargo test` panics at build
time. This skill lists the non-negotiable rules.

Source of truth for the API: `docs/llm/20_contracts.md`.
Worked examples: `docs/llm/21_sketches.md`.
Test patterns: `magnum_opus/tests/`.

## Before writing a single line of code

1. **Choose the archetype** - it is NOT negotiable after the fact.

   | Need | Archetype | Schedule |
   |------|-----------|----------|
   | owns a slice of simulation state, mutates per tick | `SimDomain` | `Update` |
   | loads read-only reference data once | `StaticData` | `Startup` |
   | read-only projection of sim state (render, export) | `View` | `PostUpdate` |
   | reads input + sim, pushes commands | `InputUI` | `PreUpdate` |

2. **Draft the contract first, code second.** List every `TypeKey`:
   - What resources does the module own (`writes`)?
   - What resources does the module read (`reads`)?
   - What commands does it consume (Sim) or produce (Input) (`commands_in`/`commands_out`)?
   - What messages does it emit or listen to (`messages_out`/`messages_in`)?
   - What metrics does it publish (`metrics`)?

   A contract slot is a promise. The core catches every broken promise.

3. **Name resources with `names!`**. Never write raw `"Grid"` strings. Always:
   ```rust
   writes: names![Grid],
   messages_out: names![TilePlaced, PlaceRejected],
   ```
   `names!` binds to `TypeId`, so `a::Grid` and `b::Grid` never collide.

## Writing the module

Every `install(ctx: &mut XxxInstaller)` function MUST:

- For each `writes` entry: call `ctx.write_resource::<T>()` (needs `T: Resource + Default`)
  or `ctx.insert_resource(value)` (for non-Default).
- For each `reads` entry: call `ctx.read_resource::<T>()`.
- For each `messages_out` entry: call `ctx.emit_message::<T>()` (needs `T: Message`).
- For each `messages_in` entry (Sim only): call `ctx.read_message::<T>()`.
- For each `commands_in` entry (Sim only): call `ctx.consume_command::<T>()`.
- For each `commands_out` entry (Input only): call `ctx.emit_command::<T>()`.
- For Sim: at least one `ctx.add_system(..)` - the primary phase must actually be used.

The installer panics if any call targets a type NOT in the contract, AND panics at
`finalize()` if any declared slot was NEVER exercised. Contract drift is a panic,
not a silent mismatch.

### Scheduling systems

```rust
ctx.add_system(sys);                // primary phase (Sim: PRIMARY_PHASE; View/Input: Post/Pre-Update)
ctx.add_command_drain(sys);         // Sim only: Phase::Commands (for drain systems)
ctx.add_metric_publish(sys);        // Sim only: Phase::Metrics (for metric publishers)
ctx.add_startup_system(sys);        // Data only: Startup
```

Arbitrary phase routing does not exist. If you need a system in a phase not in
this list, the answer is "no" - either restructure or file an RFC to add a
whitelisted slot.

## Forbidden patterns (hard rules)

The compiler or installer catches some of these. Others are accepted escape hatches
the core CANNOT catch. Do not use them.

### Enforced by the core - will panic

- **Writing a resource not in your contract.** `ctx.write_resource::<T>()` with `T`
  not in `writes`.
- **Declaring without exercising.** A contract slot that the installer never uses.
- **Calling the same installer method twice with the same type in one install.**
- **`Sim` module without any `ctx.add_system` call.** Primary phase declared but
  never used.
- **Cross-module single-writer violation.** Two modules both claiming `writes: names![X]`.
- **Duplicate module id (including `"core"`).**
- **Registering modules after `finalize_modules()` / `Harness::build()`.**
- **`app.update()` without calling `finalize_modules()`.**

### NOT enforced - banned by convention, caught only by review

Treat these as build-breaking during review. A PR that uses any of them without an
explicit justification comment is rejected.

- **Exclusive systems.** `fn sys(world: &mut World)` routed through any
  `add_system`. Bypasses the whole contract. Use regular `Res`/`ResMut`/`Query`
  systems. If an exclusive system is genuinely required (e.g. iterating all
  archetypes), justify in a doc comment and make the system read-only.
- **`ResMut<T>` where `T` is owned by another module.** Even if Bevy's scheduler
  permits the borrow, the contract forbids writes outside the owning module.
- **Interior mutability in contract-visible resources.** No `Mutex`, `RefCell`,
  `AtomicU64`, `UnsafeCell` on types in any contract slot. Plain data only.
- **Direct `MetricsRegistry::set` / `inc` on a metric owned by another module.**
  The runtime does not reject it; review does.
- **Data modules depending on each other's Startup mutations.** Startup ordering
  between Data modules is undefined.
- **`#[cfg(feature = ...)]` modules with divergent contracts.** CI must build all
  feature combinations; contract drift across builds is invisible to the core.

## Testing the module

### Unit test via `Harness`

```rust
let app = Harness::new()
    .with_sim::<MyDomain>()
    .with_input::<MyInputSource>()
    .build();   // calls finalize_modules() internally
app.update();
```

`Harness::new()` is `App::new() + MinimalPlugins + CorePlugin`. Headless, deterministic.
No rendering, no async.

### Required negative tests

For every non-trivial contract, add a negative test:

- Module declares `writes: names![T]` but install forgets `ctx.write_resource::<T>()`.
  Expected panic: `install never performed the matching installer call`.
- Two modules claim the same resource. Expected: `single-writer violation`.
- A message consumer without a producer. Expected: `closed-messages`.
- A command drained by two sim modules. Expected: `single-consumer-commands`.

Model them after `tests/forgotten_write_panics.rs`, `tests/sim_single_writer.rs`,
`tests/multi_consumer_commands_panics.rs`.

### Metrics flow

If the module declares metrics, add a test that runs 2-3 ticks and asserts the
counter/gauge/rate values in `MetricsRegistry`. Pattern: `tests/metrics_flow.rs`.

## Review checklist

Before merging a module PR:

1. `cargo test` - 36 baseline tests + any new tests all green.
2. `cargo clippy --all-targets` - zero warnings.
3. `sh docs/llm/validate.sh` - docs links resolve.
4. Contract slots match install calls 1:1 (no unused declarations, no undeclared writes).
5. No exclusive systems (grep the module for `&mut World` inside `install`).
6. No `ResMut<T>` where `T` is owned by another module (cross-reference contracts).
7. No interior-mutable types in contract slots.
8. If the module emits messages or commands: there is at least one consumer
   elsewhere, or a tests-only stub that consumes them.

## When things look wrong

- **Test panics at build with "install never performed..."** - the contract
  declared a slot, the install never called the matching `ctx.xxx::<T>()`. Either
  wire the call or remove the declaration.
- **"single-writer violation"** - two modules compete for the same resource.
  Decide who owns it; the other reads via `reads`.
- **"closed-messages" / "closed-commands"** - consumer has no producer.
  Either add a producer or drop the `messages_in` / `commands_in` declaration.
- **"primary phase is a fiction"** - Sim module never called `ctx.add_system(..)`.
  Add a system or convert to `Data` archetype if there is nothing to schedule.
- **"module registered after finalize_modules()"** - someone is calling
  `app.add_sim::<X>()` after `Harness::build()` or `app.finalize_modules()`. Do
  not mutate the registry post-finalize.
- **"app.update() reached without finalize_modules()"** - raw `App::new()` path
  that skipped `finalize_modules()`. Call it, or use `Harness::build()`.

## When the core pushes back

If the core genuinely blocks a legitimate need (and not the sloppy-author failure
class), the fix is in the CORE, not an escape hatch in the module. Edit
`magnum_opus/src/core/` with a clear RFC in the PR description, add negative
tests for the new enforcement, and update `docs/llm/20_contracts.md`.

Do NOT bypass the core. The v1 rewrite started with one bypass and ended with 238
unwraps and 83 failing tests.
