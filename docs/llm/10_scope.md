---
id: scope-guard
kind: lesson
---

# Scope guard

The owner asked for "core and grid". A prior session delivered eleven
features plus two rendering rewrites over ~24 hours. This document exists
so the failure does not repeat.

See also: [90_lessons.md](90_lessons.md) (v1 post-mortem) · [20_contracts.md](20_contracts.md).

## What the repo currently is

- `magnum_opus/src/core/` - module contract framework.
- `magnum_opus/src/world_config/` - one StaticData module, writes `WorldConfig`.
- `magnum_opus/src/grid/` - one SimDomain module, writes `Grid`.
- `magnum_opus/tests/` - 29 tests, all passing.
- `.ptsd/docs/PRD.md` - one feature: `F1 world-foundation`, implemented.

Nothing else is in scope.

## What to do when a task arrives

1. **Quote the task literally** before the first tool call. Example:
   > "The owner said: `add a grid to the project`. Scope: a grid. Anything
   > not inside that sentence is out of scope."
2. **Match the task to an existing feature.** If one fits, run the matching
   stage skill (PRD -> Tests -> Impl for lite, or PRD -> BDD -> Tests -> Impl).
3. **If no feature fits,** ask the owner to confirm the feature name and
   scope before creating entries in `.ptsd/`. A new feature does not appear
   through assistant inference.
4. **When the feature is done, stop.** No "next logical step". The next
   feature is whatever the owner names next, which may be nothing.

## Red flags (treat any as a full stop)

- You are reading `docs/llm/` or `.ptsd/docs/PRD.md` "for context" before
  the owner has named a feature.
- You are about to add a module to `src/` that is not `core/`, `grid/`, or
  `world_config/` without a matching active feature in `.ptsd/features.yaml`.
- You are about to write a `docs/` design file. The only design docs are
  the `docs/llm/*.md` referenced from the index; anything else is out of
  scope.
- The phrase "while I'm here" or "this will be needed for X anyway" appears
  in your internal reasoning. That is scope creep rationalising itself.
- A skill appears to require starting a new feature (`write-prd`,
  `create-tasks`). Skills do not constitute authorisation.

## Scope-drift post-mortem (2026-04-19)

Over one session the assistant added, committed, and partially re-committed:

| Module family                              | Status on 2026-04-19 |
|--------------------------------------------|----------------------|
| `placement` + `PlaceTile` command bus      | rolled back          |
| `buildings` + `BuildingDB`                 | rolled back          |
| `landscape` + resource generation          | rolled back          |
| `resources` (veins)                        | rolled back          |
| `group_formation`                          | rolled back          |
| `recipes_production`                       | rolled back          |
| `manifold` + `manifold-distribute`         | rolled back          |
| `render_pipeline` (v1 outline, v2 toon)    | rolled back          |
| `world_render`                             | rolled back          |
| `building_render`                          | rolled back          |
| `render-outline` post-process              | rolled back          |

Net effect: 48 test files and ~10 modules deleted on request, plus
archaeology in git.

### What enabled it

- No scope quote at the start. The original ask was never pinned down in
  text; the session's memory of it drifted.
- `docs/ARCH.md`, `docs/ECS.md`, `docs/GAMEPLAY.md`, `docs/VISUALS.md`,
  `docs/ideas.yaml` and `.ptsd/docs/PRD_legacy_v1.md` described a full
  factory-roguelike. Reading them for "context" was the same as adopting
  their feature list as a plan. All six documents were deleted.
- `.ptsd/docs/PRD.md` contained F1-F22 plus render addenda. "Depends on F1"
  was misread as "next logical step after F1". PRD is now collapsed to F1.
- PTSD skills (`write-prd`, `write-bdd`, `write-tests`, `write-impl`) created
  a ritual after each feature completion: finished -> write the next PRD. The
  skills do not authorise new features; the owner does.
- Each feature had a plausible justification for the next one (grid -> need a
  placement bus -> need building types -> need groups -> need a manifold -> need
  recipes -> need terrain so miner placement can validate -> need rendering to
  see it -> need a better outline shader). None of those justifications came
  from the owner.
- At no point did the assistant stop and ask. "Getting it working" became
  the terminal goal; the owner's explicit scope became a forgotten header.

### What now enforces non-repetition

- This document.
- `CLAUDE.md` opens with the current state and a scope guard, not with the
  design vision.
- The design docs that described the full game are deleted. `git log` is the
  archive; it does not self-inject into sessions.
- `.ptsd/docs/PRD.md` holds F1 only. New features arrive as additions
  requested by the owner, not as unarchived carry-overs.
- `.ptsd/features.yaml`, `.ptsd/state.yaml`, `.ptsd/review-status.yaml` list
  `world-foundation` and nothing else. `.ptsd/bdd/` and `.ptsd/seeds/` are
  empty.
- `Cargo.toml` carries `bevy` only. `bevy_egui` and every example entry are
  gone; no dead dep suggests a UI is planned.
