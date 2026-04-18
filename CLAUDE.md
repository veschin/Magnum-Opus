# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

**Magnum Opus** - roguelike factory game. 1-2h runs, meta-progression, Factorio core loop in a fantasy setting. God-view, player = spirit, biome-native faceless minions. Target visual: 3D isometric + pixel-art shaders.

**Current stage: greenfield rewrite.** The previous implementation was deleted on 2026-04-17. The only things that survived are the design layer and the pipeline:

- `docs/` - architecture, ECS decomposition, gameplay flow, ideas registry, visual style. **Source of truth for WHAT to build.**
- `.ptsd/` - PRD, seeds, BDD scenarios, feature state, task tracking. **Source of truth for HOW we proceed.**
- `magnum_opus/` - empty Rust crate (`src/lib.rs` is a stub). Everything lives here once written.

**Do not attempt to restore or patch the old code.** It is gone from `src/` on purpose. Reference via `git log` if you need to see what the last implementation looked like, but treat it as cautionary, not canonical.

## Tech stack

- Rust edition 2024
- Bevy 0.18 (ECS), bevy_egui 0.39 (UI)
- Simulation-first architecture: zero engine dependencies in game logic, headless testing

## Build & run

```
cd magnum_opus && cargo build          # builds the empty crate
cd magnum_opus && cargo test           # no tests yet
cd magnum_opus && cargo run            # no binary yet - lib-only crate
```

A binary (`src/main.rs`) and tests will appear as the PTSD pipeline advances past the seed stage for each feature.

## Design inputs (load on demand)

When a task needs architectural context, load the relevant doc explicitly rather than assuming:

- Architecture principles, phase ordering, invariants: @docs/ARCH.md
- ECS decomposition (components, resources, systems): @docs/ECS.md
- Gameplay walkthrough from meta-hub to scoring: @docs/GAMEPLAY.md
- Visual/art direction: @docs/VISUALS.md
- Bevy ecosystem notes and PoC results: @docs/BEVY_ECOSYSTEM.md, @docs/ENGINE_PoC.md
- Ideas registry (scoped, tiered, most are locked): @docs/ideas.yaml
- Current PRD: @.ptsd/docs/PRD.md

These documents describe the **target state**, not what is currently in `magnum_opus/src/`. Treat counts, plugin lists, and system inventories in `docs/` as design intent until the corresponding feature reaches the impl stage in PTSD.

## Design vocabulary

- **Building Group** - adjacent buildings sharing a manifold (auto-formed)
- **Manifold** - shared resource pool inside a group
- **Rune Path / Pipe** - player-placed transport between groups (solids / liquids)
- **Opus Tree** - run goal: production throughput milestones + mini-opus branches
- **Mini-Opus** - optional side challenge for meta-currency
- **Tier Gate** - creature nest clearing unlocks next tier (T1->T2->T3)
- **Mall** - group that produces buildings into Inventory
- **Combat Group** - imp camp consuming weapons+food, provides protection and organic resources

## Bevy 0.18 API gotchas

- Events register via `app.add_message::<EventType>()` (not `add_event`)
- Systems registered with `app.add_systems(Update, system.in_set(Phase::X))`
- Resources: `app.insert_resource(val)` or `app.init_resource::<Type>()`

## Hooks and skills

- `.claude/hooks/cargo-fmt.sh` - runs `rustfmt --edition 2024` on every Edit/Write of a `.rs` file
- `.claude/hooks/ptsd-*.sh` - enforce PTSD pipeline gates (see block below)
- `.claude/hooks/check-file-size.sh` - blocks Read on files >500 lines (use offset/limit or Grep)
- `/reality-check` skill - audit build, tests, code debt, PTSD state, and CLAUDE.md drift. Run it whenever claims in this file feel suspicious.

If `cargo build` or `cargo test` numbers in this file start looking off, you were probably lied to by a past session. Run `/reality-check` and correct this file.

<!-- ---ptsd--- -->
# Claude Agent Instructions

## Authority Hierarchy (ENFORCED BY HOOKS)

PTSD (iron law) > User (context provider) > Assistant (executor)

- PTSD decides what CAN and CANNOT be done. Pipeline, gates, validation -- non-negotiable.
  Hooks enforce this automatically -- writes that violate pipeline are BLOCKED.
- User provides context and requirements. User also follows ptsd rules.
- Assistant executes within ptsd constraints. Writes code, docs, tests on behalf of user.

## Session Start Protocol

EVERY session, BEFORE any work:
1. Run: ptsd context --agent -- see full pipeline state
2. Run: ptsd task next --agent -- get next task
3. Follow output exactly.

## Commands (always use --agent flag)

- ptsd context --agent              -- full pipeline state (auto-injected by hooks)
- ptsd status --agent               -- project overview
- ptsd task next --agent            -- next task to work on
- ptsd task update <id> --status WIP -- mark task in progress
- ptsd validate --agent             -- check pipeline before commit
- ptsd feature list --agent         -- list all features
- ptsd seed init <id> --agent       -- initialize seed directory
- ptsd gate-check --file <path> --agent -- check if file write is allowed
- ptsd test map --feature <id> <test-file> -- map test without BDD (for lite pipeline)
- ptsd feature pipeline <id> <profile> -- change feature pipeline
- ptsd migrate --agent            -- migrate project to current version

## Skills

PTSD pipeline skills are in `.claude/skills/` -- auto-loaded when relevant.

| Skill | When to Use |
|-------|------------|
| write-prd | Creating or updating a PRD section |
| write-seed | Creating seed data for a feature |
| write-bdd | Writing Gherkin BDD scenarios |
| write-tests | Writing tests from BDD scenarios |
| write-impl | Implementing to make tests pass |
| create-tasks | Adding tasks to tasks.yaml |
| review-prd | Reviewing PRD before advancing to seed |
| review-seed | Reviewing seed data before advancing to bdd |
| review-bdd | Reviewing BDD before advancing to tests |
| review-tests | Reviewing tests before advancing to impl |
| review-impl | Reviewing implementation after tests pass |
| workflow | Session start or when unsure what to do next |
| adopt | Bootstrapping existing project into PTSD |

Use the corresponding write skill, then review skill at each pipeline stage.

Note: write-seed is only required for full pipeline. write-bdd is required for full and standard pipelines. Lite pipeline skips both -- write tests directly from PRD.

## Pipeline Profiles

Each feature has a pipeline profile that determines required stages:

| Profile | Stages | Use For |
|---------|--------|---------|
| full | PRD -> Seed -> BDD -> Tests -> Impl | Complex, data-heavy features |
| standard | PRD -> BDD -> Tests -> Impl | Default. Most features |
| lite | PRD -> Tests -> Impl | Simple utilities, config |

Check feature pipeline: `ptsd feature show <id> --agent`
Change pipeline: `ptsd feature pipeline <id> full|standard|lite`

Each required stage needs review score >= 7 before advancing.
Hooks enforce gates automatically -- blocked writes show the reason.

## Rules

- NO mocks for internal code. Real tests, real files, temp directories.
- NO garbage files. Every file must link to a feature.
- NO hiding errors. Explain WHY something failed.
- NO over-engineering. Minimum code for the current task.
- ALWAYS run: ptsd validate --agent before committing.
- COMMIT FORMAT: [SCOPE] type: message
  Scopes: PRD, SEED, BDD, TEST, IMPL, TASK, STATUS
  Types: feat, add, fix, refactor, remove, update

## Troubleshooting

When ptsd status/validate shows unexpected results, debug with these steps:

| Symptom | Cause | Fix |
|---------|-------|-----|
| TESTS:0 but test files exist | Tests not mapped to features | `ptsd test map .ptsd/bdd/<id>.feature <test-file>` or `ptsd test map --feature <id> <test-file>` (lite pipeline) |
| BDD:0 but .feature files exist | State hashes empty, SyncState not run | `ptsd status --agent` triggers sync; if still 0, check `.ptsd/bdd/<id>.feature` has `@feature:<id>` tag on line 1 |
| Feature stuck at wrong stage | review-status.yaml stale or stage not advanced | Run `ptsd review <id> <stage> <score>` to advance; check `ptsd context --agent` for blockers |
| "no test files mapped" on `ptsd test run` | Test mapping missing in state.yaml | `ptsd test map .ptsd/bdd/<id>.feature <test-file>` or `--feature <id> <test-file>` |
| Gate blocks file write | File not in allowed list for current stage | Check `ptsd gate-check --file <path> --agent`; advance feature to correct stage first |
| Validate shows "mock detected" | Test file contains mock/stub patterns | Replace mocks with real file-based tests in temp directories |
| Regression warning on status | Artifact file changed after stage was reviewed | Re-review the stage: `ptsd review <id> <stage> <score>` |

### Debug flow
1. `ptsd context --agent` -- shows next action, blockers, stage per feature
2. `ptsd feature show <id> --agent` -- shows artifact counts and test stats
3. `ptsd validate --agent` -- shows all pipeline violations
4. Check `.ptsd/state.yaml` -- hashes, test mappings, stages
5. Check `.ptsd/review-status.yaml` -- review verdicts per feature

### Test mapping
Features need test files mapped to track results:
- Standard/full pipeline: `ptsd test map .ptsd/bdd/<id>.feature <test-file>` (reads @feature tag from BDD)
- Lite pipeline (no BDD): `ptsd test map --feature <id> <test-file>` (direct mapping)
Without mapping, ptsd cannot track test results per feature.

## Forbidden

- Mocking internal code
- Skipping pipeline steps
- Hiding errors or pretending something works
- Generating files not linked to a feature
- Using --force, --skip-validation, --no-verify

<!-- ---ptsd--- -->
