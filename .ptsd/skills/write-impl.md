---
name: write-impl
description: Use when implementing code to make failing tests pass
---

## Instructions

1. Make each failing test pass, one at a time.
2. Write only the code required -- no speculative features.
3. Follow the project's package/module structure.
4. Error format: use the project's error format (e.g., err:<category> <message>).
5. No mocks in implementation. Use real I/O.
6. Run the configured test runner after each change.

## Common Mistakes

- Writing more code than the tests require -- no speculative features.
- Putting logic in the wrong layer -- follow the project's module structure.
- Printing errors instead of returning them with the project's error format.
- Adding dependencies that violate the project's dependency policy.
- Not running tests after each change -- catching failures early is cheaper.
- Forgetting to run `ptsd review <feature> impl <score>` after committing -- the feature stays unreviewed.
