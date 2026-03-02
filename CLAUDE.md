# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Magnum Opus — roguelike factory game. 1–2h runs, meta-progression, Factorio core loop in a fantasy setting. God-view, player = spirit, biome-native faceless minions. 3D isometric + pixel-art shaders.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Engine:** Bevy 0.18 (ECS-only, no rendering features yet)
- **Architecture:** Simulation-first — zero engine dependencies in game logic, headless testing

## Build & Test

All code lives in `magnum_opus/` crate:

```bash
cd magnum_opus && cargo build          # build
cd magnum_opus && cargo test           # run all tests
cd magnum_opus && cargo test <name>    # run single test by name
```

## Architecture (docs/ARCH.md)

- **Simulation-first:** game is a deterministic numerical simulation; rendering is a separate read-only layer
- **ECS paradigm:** entities (IDs), components (pure data structs), systems (functions), resources (global singletons)
- **Command sourcing:** player actions are serialized commands, never direct world mutations
- **Phase-ordered tick pipeline:** design targets 10 phases (Input → World → Creatures → Energy → Production → Transport → Combat → Progression → Cleanup → Meta); currently 9 implemented in `Phase` enum: Input → Groups → Power → Production → Manifold → Transport → Progression → Creatures → World
- **Event bus:** systems communicate via events, never direct calls
- **Data-driven content:** all game content loaded from static data files (RecipeDB, BuildingDB, BiomeDB, etc.)

### Critical Invariants (always hold)

Resource conservation, grid alignment, determinism, group connectivity, single group membership, transport exclusivity, tier monotonicity, energy non-negative, organic exclusivity, milestone persistence, inventory integrity.

## Key Files

| Path | Purpose |
|------|---------|
| `docs/ARCH.md` | Architecture principles — HOW we build |
| `docs/ECS.md` | Full ECS decomposition (37 components, 15 resources, 40 systems) — WHAT we build |
| `docs/GAMEPLAY.md` | Gameplay flow walkthrough from meta-hub to scoring |
| `.ptsd/docs/PRD.md` | Product requirements — 8 features |
| `docs/ideas.yaml` | Ideas registry (35 ideas) |
| `docs/ENGINE_PoC.md` | Bevy ECS proof-of-concept results |
| `magnum_opus/` | Main crate — 384 tests, 8 features |

## Code Structure (magnum_opus/src/)

```
lib.rs              — plugin registration, phase ordering
components.rs       — all ECS component structs
resources.rs        — global singleton resources (Grid, PlacementCommands, EnergyPool)
events.rs           — event types (BuildingPlaced, BuildingRemoved)
systems/            — one file per system (placement, groups, power, production, manifold, transport, ux, terrain, trading)
tests/              — BDD tests (one file per feature) + legacy unit tests
```

### Plugins (defined in lib.rs)

- **SimulationPlugin** — registers all core systems (placement, groups, energy, production, manifold, transport, progression) with phase ordering, plus core resources (Grid, EnergyPool, Inventory, etc.) and events. Used by most feature tests.
- **WorldPlugin** — registers world/biome systems (tick, hazards, weather, fog, element interactions) with separate resources (SimTick, CurrentWeather, ActiveBiome). Used by world BDD tests.
- **CreaturesPlugin** — registers creature & combat systems (behavior, expansion, combat, pressure, nests, loot, minions) in Phase::Creatures. Added on top of SimulationPlugin for creature scenarios.

### Bevy 0.18 API Notes

- Events use `app.add_message::<EventType>()` (not `add_event`)
- Systems registered with `app.add_systems(Update, system.in_set(Phase::X))`
- Resources: `app.insert_resource(val)` or `app.init_resource::<Type>()`

### Test Pattern

```
1. Create App with MinimalPlugins + SimulationPlugin or WorldPlugin
2. Insert commands/resources
3. app.update() — runs all systems in phase order
4. Query components and assert values
```

No mocks for internal code. Real ECS, real data.

## Features (8 total, all at impl stage)

building-groups, transport, world, creatures, progression, meta, energy, ux

All features implemented and reviewed (scores 7-8/10). 384 tests, 0 failures.

### System → Phase Mapping (lib.rs)

| Phase | Systems |
|-------|---------|
| Input | placement_system |
| Groups | group_formation_system, group_priority_system, group_pause_system |
| Power | energy_system |
| Production | production_system |
| Manifold | manifold_system |
| Transport | transport_destroy_system, transport_placement_system, transport_tier_upgrade_system, transport_movement_system |
| Progression | milestone_check_system, opus_tree_sync_system, run_lifecycle_system, tier_gate_system, building_tier_upgrade_system, mini_opus_system |
| Creatures (CreaturesPlugin) | creature_behavior_system, invasive_expansion_system, combat_group_system, combat_pressure_system, nest_clearing_system, creature_loot_system, minion_task_system |
| World (WorldPlugin) | tick_advance, hazard_warning, hazard_trigger, element_interaction, weather_tick, fog_of_war, world_placement |

## Design Vocabulary

- **Building Group:** adjacent buildings sharing a manifold (auto-formed, auto-optimized internal flow)
- **Manifold:** shared resource pool inside a group
- **Rune Path / Pipe:** player-placed transport between groups (solids / liquids)
- **Opus Tree:** run goal — production throughput milestones + mini-opus branches
- **Mini-Opus:** optional side challenges for meta-currency
- **Tier Gate:** creature nest clearing unlocks next tier (T1→T2→T3)
- **Mall:** group that produces buildings → Inventory
- **Combat Group:** imp camps consuming weapons+food → protection + organic resources

<!-- ---ptsd--- -->
# Claude Agent Instructions

## Authority Hierarchy (ENFORCED BY HOOKS)

PTSD (iron law) > User (context provider) > Assistant (executor)

- PTSD decides what CAN and CANNOT be done. Pipeline, gates, validation — non-negotiable.
  Hooks enforce this automatically — writes that violate pipeline are BLOCKED.
- User provides context and requirements. User also follows ptsd rules.
- Assistant executes within ptsd constraints. Writes code, docs, tests on behalf of user.

## Session Start Protocol

EVERY session, BEFORE any work:
1. Run: ptsd context --agent — see full pipeline state
2. Run: ptsd task next --agent — get next task
3. Follow output exactly.

## Commands (always use --agent flag)

- ptsd context --agent              — full pipeline state (auto-injected by hooks)
- ptsd status --agent               — project overview
- ptsd task next --agent            — next task to work on
- ptsd task update <id> --status WIP — mark task in progress
- ptsd validate --agent             — check pipeline before commit
- ptsd feature list --agent         — list all features
- ptsd seed init <id> --agent       — initialize seed directory
- ptsd gate-check --file <path> --agent — check if file write is allowed

## Skills

PTSD pipeline skills are in `.claude/skills/` — auto-loaded when relevant.

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

## Pipeline (strict order, no skipping)

PRD → Seed → BDD → Tests → Implementation

Each stage requires review score ≥ 7 before advancing.
Hooks enforce gates automatically — blocked writes show the reason.

## Rules

- NO mocks for internal code. Real tests, real files, temp directories.
- NO garbage files. Every file must link to a feature.
- NO hiding errors. Explain WHY something failed.
- NO over-engineering. Minimum code for the current task.
- ALWAYS run: ptsd validate --agent before committing.
- COMMIT FORMAT: [SCOPE] type: message
  Scopes: PRD, SEED, BDD, TEST, IMPL, TASK, STATUS
  Types: feat, add, fix, refactor, remove, update

## Forbidden

- Mocking internal code
- Skipping pipeline steps
- Hiding errors or pretending something works
- Generating files not linked to a feature
- Using --force, --skip-validation, --no-verify

<!-- ---ptsd--- -->
