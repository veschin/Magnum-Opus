# CLAUDE.md

Guidance for Claude Code (claude.ai/code) in this repository.

## Project state (authoritative; keep synced with `cargo test`)

**Magnum Opus** - greenfield Rust/Bevy experiment. Everything larger than the
core was rolled back on 2026-04-19 at the owner's direction because a previous
session had expanded the scope without permission.

What exists right now:

- `magnum_opus/src/core/` - module contract system (four archetypes, four
  installers, `ModuleRegistry`, `Phase` enum, `MetricsRegistry`, `Tick`).
- `magnum_opus/src/world_config/` - `WorldConfigModule` (StaticData). Writes
  `WorldConfig { width=64, height=64, seed=0x9E3779B97F4A7C15 }`.
- `magnum_opus/src/grid/` - `GridModule` (SimDomain, `Phase::World`). Reads
  `WorldConfig`, writes `Grid { width, height, occupancy: BTreeMap, dims_set }`,
  publishes `grid.occupancy_count` gauge. Occupancy stays empty in F1.
- `magnum_opus/tests/` - 29 tests, all green. Core contract suite + grid +
  world-config smoke.
- `magnum_opus/examples/grid_prototype.rs` - terrain generation prototype.
  Multi-tile heightmap (2×2, 64×64), spring-based water, toon shading,
  outline post-process. Owner-approved visual testbed, not gameplay code.
  See `docs/llm/40_terrain.md` for the terrain system spec.

What does not exist (and must not be written without an explicit ask):

- Any binary (`src/main.rs`). The crate is lib-only.
- Gameplay modules: buildings, recipes, manifold, groups, placement
  command bus, production, transport, fog, creatures, combat, progression.
- Seed/BDD artifacts. `.ptsd/bdd/` and `.ptsd/seeds/` are empty on purpose.

## Scope guard (READ BEFORE DOING ANYTHING)

A prior session turned "add a grid" into eleven features plus two rendering
rewrites over ~24 hours. The owner never asked for any of it. The full
post-mortem lives at `docs/llm/10_scope.md`.

Operating rules:

1. The owner's request defines scope **literally**. Quote it back before
   starting; anything not inside the quoted text is out of scope.
2. When the current feature is done, **stop and ask**. Do not start the
   "next logical feature". There is no next feature until the owner names one.
3. `.ptsd/docs/PRD.md`, the `.claude/skills/write-*` skills, and any old
   `docs/llm/` sketches describe **possible** futures. They are not a queue.
4. Tie-breakers in favour of doing less. If unsure whether something is in
   scope, it is out of scope.

## Build & test

```
cd magnum_opus && cargo build                            # lib
cd magnum_opus && cargo test                             # 29 tests
cd magnum_opus && cargo run --example grid_prototype     # terrain prototype
SCREENSHOT=1 cargo run --example grid_prototype          # auto-screenshot to /tmp/
```

## Tech stack

- Rust edition 2024
- Bevy 0.18 (ECS only; no renderer is wired up)

That is the entire dependency list. `bevy_egui` and everything else a past
session added are gone.

## Reference docs (load on demand)

The only live design docs are in `docs/llm/`, short and LLM-facing:

- [docs/llm/00_index.md](docs/llm/00_index.md) - graph of docs.
- [docs/llm/10_scope.md](docs/llm/10_scope.md) - the scope guard + scope-drift
  post-mortem. Read this if you are about to add a file outside `core/` /
  `grid/` / `world_config/`.
- [docs/llm/20_contracts.md](docs/llm/20_contracts.md) - module archetype
  traits, installer methods, the 18 enforced invariants. Source of truth for
  the core API.
- [docs/llm/21_sketches.md](docs/llm/21_sketches.md) - real `grid` and
  `world_config` module code, annotated as worked examples of the traits.
- [docs/llm/90_lessons.md](docs/llm/90_lessons.md) - v1 and scope-drift
  post-mortems.

Everything else (ARCH.md, ECS.md, GAMEPLAY.md, VISUALS.md, BEVY_ECOSYSTEM.md,
ENGINE_PoC.md, ideas.yaml, PRD_legacy_v1.md) was deleted on 2026-04-19.
`git log` is the archive.

## Bevy 0.18 API gotchas

- Events register via `app.add_message::<EventType>()`.
- Systems register through installers (`ctx.add_system`, `ctx.add_command_drain`,
  `ctx.add_metric_publish`). Modules never touch `&mut App`.
- Resources: `app.insert_resource(val)` or `app.init_resource::<Type>()`.

## Hooks and skills

- `.claude/hooks/cargo-fmt.sh` - formats `.rs` on Edit/Write.
- `.claude/hooks/check-file-size.sh` - blocks `Read` on files over 500 lines;
  use `offset` / `limit` or `Grep`.
- `.claude/hooks/ptsd-*.sh` - enforce PTSD pipeline gates on `.ptsd/` edits.
- `.claude/skills/core-module/` - skill for implementing a new core module.
  Only useful when the owner has explicitly asked for a new module.
- `.claude/skills/write-*` and `.claude/skills/review-*` - PTSD pipeline
  ritual skills. Only valid when the owner has greenlit a new feature. Not
  self-starting.

<!-- ---ptsd--- -->
# PTSD pipeline

PTSD is the per-feature state machine. It is a tool for executing an
owner-approved feature, not a reason to start one.

## Authority

Owner > PTSD > Assistant. The assistant never escalates from "no feature
requested" to "let's start a feature" on its own.

## Session start

When the owner gives a task:

1. Quote the task literally.
2. `ptsd status --agent` to see the live feature state.
3. If the task maps to an existing active feature, proceed with the matching
   stage skill. If it does not, ask the owner to confirm the feature name and
   scope before editing `.ptsd/`.

## Commands (always use `--agent`)

- `ptsd context --agent` - pipeline state (hooks auto-inject).
- `ptsd status --agent` - project overview.
- `ptsd validate --agent` - pre-commit check.
- `ptsd feature list --agent`, `ptsd feature show <id> --agent`
- `ptsd test map --feature <id> <test-file>` - lite-pipeline test mapping.
- `ptsd review <id> <stage> <score>` - advance review.
- `ptsd gate-check --file <path> --agent` - is this write allowed?

## Pipeline profiles

| Profile  | Stages                                | Use for                          |
|----------|---------------------------------------|----------------------------------|
| full     | PRD -> Seed -> BDD -> Tests -> Impl       | Complex, data-heavy features     |
| standard | PRD -> BDD -> Tests -> Impl              | Default                          |
| lite     | PRD -> Tests -> Impl                    | Simple utilities / config        |

F1 (`world-foundation`) is lite.

## Commit format

`[SCOPE] type: message`

- Scopes: `PRD`, `SEED`, `BDD`, `TEST`, `IMPL`, `TASK`, `STATUS`.
- Types: `feat`, `add`, `fix`, `refactor`, `remove`, `update`.
- One scope per commit. Hooks reject mixed-scope commits.

## Rules

- No mocks of internal code. Real tests, real files, temp dirs.
- No garbage files. Every file links to a feature.
- No hidden errors. Say why something failed.
- No over-engineering. Minimum code for the current task.
- Run `ptsd validate --agent` before committing.
- Never use `--force`, `--skip-validation`, `--no-verify`.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `TESTS:0` but test files exist | Tests unmapped | `ptsd test map --feature <id> <test-file>` (lite) or via BDD feature file |
| `BDD:0` but `.feature` files exist | State hashes empty | `ptsd status --agent` triggers sync; check `@feature:<id>` tag on line 1 |
| Feature stuck at wrong stage | review-status stale | `ptsd review <id> <stage> <score>` |
| "No test files mapped" on `ptsd test run` | Missing mapping | `ptsd test map ...` |
| Gate blocks file write | Stage disallows path | `ptsd gate-check --file <path> --agent`; advance stage first |
| Regression warning | Artifact changed after review | Re-review the stage |
<!-- ---ptsd--- -->
