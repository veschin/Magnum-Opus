---
id: v1-post-mortem
kind: lesson
---

# v1 post-mortem

Context: greenfield reset on 2026-04-17 (commit `8bc169d`). What shipped did not match what was promised, and tests lied about the gap.

See also: [20_contracts.md](20_contracts.md) for the rules this lesson motivated, [10_scope.md](10_scope.md) for the scope-drift failure from the v2 session that followed.

## Measured gap between design and code

| Item                | Promised (legacy ECS.md, now deleted) | Actual (v1 code) |
|---------------------|-------------------|------------------|
| Components          | 37                | 67               |
| Resources           | 15                | 44               |
| Systems             | 40                | ~15              |
| Tests passing       | claimed 421/421   | actually 454/537 (83 failing) |
| `unwrap`/`panic`/`todo` | 0 (aspirational) | 238              |

`CLAUDE.md` claimed clean; reality was not. Any future claim in `CLAUDE.md` without a matching `cargo test` output is suspect.

## Root causes

### 1. Monolithic files without boundaries

- `components.rs` - 1697 lines
- `resources.rs` - 699 lines
- `ux.rs` - 568 lines
- `manifold.rs` - 37 lines (stub next to production code)

No owner per file. No cap on growth. Cross-cutting imports everywhere.

### 2. Shared resource ownership

`WorldPlugin::build()` contained:

> also in SimulationPlugin - `init_resource`/`add_message` are idempotent

Multiple plugins registered the same resources "idempotently". No single writer, no audit of who mutates what.

### 3. Cross-plugin event hacks

Events emitted in a later-running phase consumed by an earlier-running phase in another plugin required tick-N+1 latency workarounds, documented as "designed exceptions" in the v1 ARCH.md §13 (since deleted). Exceptions outnumbered rules.

### 4. Schedule mixing

`WorldPlugin` ran systems directly in `Update`. `SimulationPlugin` used a `Phase` enum with `SystemSet` ordering. When both were loaded, `WorldPlugin` ran at "indeterminate position relative to Phase systems". Determinism became aspirational.

### 5. Legacy-compatibility shims leaked into production

```rust
pub struct PlacementRequest {
    pub skip_inventory_check: bool,
    pub skip_fog_check: bool,
    // ...
}
```

Flags added so pre-existing tests would keep passing. These bypasses became part of the production surface.

### 6. Test explosion without contract

Each feature had its own `_bdd.rs` at 2000+ lines with bespoke helpers. No shared harness. Assertion styles varied. Per-feature metric conventions diverged.

## Rules for v2

Enforced by `core::ModuleRegistry` at `App` build time:

1. One writer per resource - `register_sim` / `register_data` panic on collision.
2. Commands and messages form a closed set across registered modules - `finalize_modules()` panics on dangling consumers.
3. Archetype determines schedule - `SimDomain` in `Update`, `StaticData` in `Startup`, `View` in `PostUpdate`, `InputUI` in `PreUpdate`. Single `Phase` slot per sim module.
4. Unified `Harness` and `MetricsRegistry` - all tests go through the same entry; metrics have uniform name-kind-owner shape across modules.
5. No `skip_*` bypass fields, no legacy queues, no dual-writer exceptions. If a test needs a backdoor, the backdoor becomes the contract and gets validated like everything else.
