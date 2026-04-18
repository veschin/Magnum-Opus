---
name: adopt
description: Use when bootstrapping an existing project into PTSD pipeline
---

## Instructions

1. Run `ptsd init` in the project root (sets up .ptsd/, hooks, skills).
2. Run `ptsd map` to generate a codebase overview at `.ptsd/docs/CODEBASE.md`.
3. Read CODEBASE.md to understand directory structure, test coverage, and key files.
4. Decide what feature YOU want to build or change.
5. Run `ptsd feature add <id> "title" [--pipeline standard|lite|full]` to register your feature.
6. Run `ptsd feature status <id> in-progress` to activate it.
7. Follow the pipeline: `ptsd context --agent` tells you what to do next.

## Existing Code Is Context, Not Features

Do NOT register every existing module as a PTSD feature. Existing code is the context you're building on. Only register features you are actively developing or modifying.

## When to Use Each Pipeline

- **standard** (default) -- new features that need behavior specs and tests
- **lite** -- small changes, config, utilities -- write tests directly from PRD
- **full** -- complex data-heavy features that need seed data

## Common Mistakes

- Registering every test file as a feature -- only register what you're working on.
- Skipping `ptsd map` -- the codebase overview helps you and the AI understand what exists.
- Starting implementation before writing PRD -- the pipeline exists for a reason.
