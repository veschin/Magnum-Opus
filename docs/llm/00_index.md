---
id: llm-index
kind: index
---

# docs/llm - LLM-facing reference

Operational reference graph. English only. Short, structured, cheap to load.
Each entry declares `id` and `kind` in front-matter.

## Entries

- [10_scope.md](10_scope.md) - scope guard and scope-drift post-mortem
  (read this before touching anything outside `core/`, `grid/`, `world_config/`).
- [20_contracts.md](20_contracts.md) - module archetypes, traits,
  `ModuleRegistry` rules (source of truth for the module interface).
- [21_sketches.md](21_sketches.md) - annotated walkthrough of the two
  modules currently in the repo (`world_config`, `grid`).
- [30_pixel_art_render.md](30_pixel_art_render.md) - 3D pixel-art render
  recipe, derived from davidhol.land's 2024 article. Reference frames in
  [refs/3d_pixel_art/](refs/3d_pixel_art/). Recipe only - no render
  feature is owner-approved yet.
- [90_lessons.md](90_lessons.md) - v1 post-mortem, rules that must hold in v2.

## Kinds

- `index` - this file.
- `spec` - interface / contract; read before changing the related code.
- `guide` - synthesised recipe / how-to derived from external material.
- `lesson` - post-mortem notes.

## Update rule

Any change to a public contract (trait signature, `Phase` variant, resource
name, metric name, invariant) updates the matching spec in the same commit.

## Validation

`./validate.sh` - checks every relative markdown link in `docs/llm/` resolves
to an existing file. Exit code 0 = clean.
