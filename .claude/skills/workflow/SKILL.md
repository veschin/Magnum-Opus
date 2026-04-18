---
name: workflow
description: Use at session start or when unsure what to do next
---

## Pipeline Profiles

Each feature has a pipeline profile. Check: `ptsd feature show <id> --agent`

| Profile | Stages |
|---------|--------|
| full | PRD -> Seed -> BDD -> Tests -> Impl |
| standard | PRD -> BDD -> Tests -> Impl |
| lite | PRD -> Tests -> Impl |

Context tells you what to do: `ptsd context --agent`
Follow the `action=` field. Skip stages not in the feature's profile.

### At each stage

| Stage | Create skill | Review skill |
|-------|-------------|--------------|
| PRD | write-prd | review-prd |
| Seed | write-seed | review-seed |
| BDD | write-bdd | review-bdd |
| Tests | write-tests | review-tests |
| Impl | write-impl | review-impl |

### Session protocol

1. Run `ptsd context --agent` -- see where each feature is and what to do next.
2. Pick the next feature/stage from the `next:` lines.
3. Apply the write-<stage> skill -> create artifacts.
4. Commit with `[SCOPE] type: message` format.
5. Run `ptsd review <feature> <stage> <score>` -- score 0-10, honest self-assessment.
6. Move to the next stage or feature.

### Stage cycle (repeat for every feature × every stage)

```
write artifacts -> commit [SCOPE] -> ptsd review <feature> <stage> <score> -> next
```

Do NOT skip the `ptsd review` step. It records review verdicts. Without it the feature stays `review: pending` forever.

### Gate rules

- No BDD without seed initialized (full pipeline only)
- No tests without BDD written (full/standard only)
- No impl without passing test review
- No stage advance without review score >= min_score (default 7)

### Existing project (after ptsd adopt)

If you joined a project that was bootstrapped with `ptsd adopt`:

1. Run `ptsd context --agent` -- see which features exist and their stages.
2. Check for unmapped tests: `ptsd status --agent` -- if TESTS:0 but test files exist, they need mapping.
3. Check for missing PRDs: features without `<!-- feature:<id> -->` anchors in PRD.md will be blocked.
4. For features with passing tests but no BDD: set pipeline to `lite` with `ptsd feature pipeline <id> lite`.
5. For features that need better test coverage: keep `standard` pipeline, write BDD scenarios first.

### Multiple blocked features

When `ptsd context --agent` shows several features blocked:

1. Start with features that have no dependencies on other features.
2. If features share state (e.g., shared storage format), design the data model in the first feature's seed/BDD, then reference it in dependent features.
3. Complete one feature fully before starting the next -- avoid spreading work across many features at once.

## Common Mistakes

- Starting implementation without checking review-status.yaml first.
- Skipping the review skill after the create skill -- both are required at each stage.
- Forgetting to update review-status.yaml immediately after completing work.
- Working on a feature that is blocked by a gate (e.g., writing tests before BDD exists).
