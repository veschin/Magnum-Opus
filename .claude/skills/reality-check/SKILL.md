---
name: reality-check
description: Perform a hard audit of the Magnum Opus repo - build, tests, clippy warnings, unwrap/TODO counts, oversized files, PTSD pipeline state, and drift between CLAUDE.md claims and reality. Use at session start when CLAUDE.md claims feel stale, before committing large changes, or whenever asked to verify the true state of the code.
---

# Reality Check

Run the following checks and report results in the structure below. Keep the report under 400 words. Be brutally specific - file paths, line counts, exact numbers. No marketing language.

## Commands to run (in order, in parallel where independent)

1. **Build**
   ```
   cd magnum_opus && cargo build 2>&1 | tail -20
   ```

2. **Tests**
   ```
   cd magnum_opus && cargo test 2>&1 | tail -15
   ```
   Extract: `test result: ... N passed; M failed`.

3. **Clippy warnings**
   ```
   cd magnum_opus && cargo clippy 2>&1 | grep -c "^warning"
   ```

4. **Code debt counts** (use Grep, not bash)
   - `unwrap\(\)|expect\(|panic!\(|todo!\(|unimplemented!\(` in `magnum_opus/src/**/*.rs`
   - `TODO|FIXME|HACK|XXX` in `magnum_opus/src/**/*.rs`
   - `#\[allow\(` in `magnum_opus/src/**/*.rs`

5. **Oversized files** (>400 lines - rewrite candidates)
   ```
   find magnum_opus/src -name "*.rs" -exec wc -l {} + | sort -rn | head -10
   ```

6. **PTSD state**
   ```
   ptsd status --agent 2>&1 | head -40
   ```

7. **Drift check** - Read CLAUDE.md lines about test counts, component counts, phase counts, plugin lists. Compare against actual code. Flag every divergence.

## Report format

```
## Build
<compiles yes/no, warning count, compile time>

## Tests
<passed/failed/ignored - exact numbers from `test result:` line>

## Code debt
- unwrap/panic/todo: N occurrences across M files
- TODO/FIXME/HACK: N occurrences
- #[allow(...)]: N occurrences

## Oversized files (>400 lines)
<file_path:line_count list, rewrite candidates>

## PTSD pipeline state
<feature stages, failing gates, last advance>

## Drift from CLAUDE.md
<bullet list of every claim in CLAUDE.md that does not match reality - cite line numbers in CLAUDE.md and contradicting evidence>

## Verdict
<one sentence: is the codebase in the state CLAUDE.md claims, or not>
```

## Rules

- If `cargo build` fails: report the first error, do not attempt to fix.
- If `ptsd` binary is missing: skip section 6, note it.
- Do **not** update CLAUDE.md automatically - just report drift. The user decides whether to correct CLAUDE.md or the code.
- Do **not** run `cargo clean` or mutate any state.
