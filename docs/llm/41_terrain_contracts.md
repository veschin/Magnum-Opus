---
id: terrain-contracts
kind: spec
---

# Terrain generation contracts

Invariants and design decisions for `examples/grid_prototype.rs`.
Validated through 3 rounds of adversarial Opus critic review.

## Design decisions (critic-validated)

| # | Decision | Why | Alternative rejected |
|---|----------|-----|---------------------|
| 1 | Spring-based water | Controllable density, failure = too little (fixable) | D8 accumulation (floods map), basin stamps (fragile) |
| 2 | 11 height levels (-2..+3, 0.5 step) | Enough for gameplay without visual noise | 21 levels (crumpled paper on 32×32) |
| 3 | 2 noise octaves (16/2.5, 6/0.8) | Macro plateaus + organic edges | 3 octaves (scale-3 either useless or noisy) |
| 4 | Global generation | Seamless tile edges, trivial at <=256×256 | Per-tile with boundary handoff (complex, fragile) |
| 5 | Ocean as columns | Interior valleys visible, no z-fighting | Ocean plane (hides valleys, dominates camera) |
| 6 | Feature fixup | Guarantees all gameplay features exist | Rejection sampling (no placement control) |
| 7 | River carving -0.5 | Visible channel, creates cliffs alongside | No carving (rivers cosmetic only) |
| 8 | Smooth from mutated heights | Carving/fixup visible in both modes | Smooth from raw noise (bug: mutations invisible) |

## Invariants

### Height (H)

- **H1** Heights in [-2.0, +3.0], multiples of 0.5. Clamp before quantize.
- **H2** Border cells (dist < EDGE_MARGIN from world edge) below SEA_LEVEL.
- **H3** At least 1 interior cell above SEA_LEVEL when WORLD > 6×6.
- **H4** Re-quantize after any height mutation (carving, fixup).

### Water (W)

- **W1** Ocean = world-edge cells below SEA_LEVEL. Not flood-BFS.
- **W2** Lake = local minimum on a spring-traced path, >=2 cells.
- **W3** River = steepest-descent path from spring to ocean/water.
  Tiebreak on plateaus: prefer cell closest to world edge.
- **W4** No cell in multiple water categories.
- **W5** At least 1 river, 1 lake, 1 waterfall per world (fixup pass).

### Resources (R)

- **R1** All node cells dry and flat (not water, neighbor diff <=0.5).
- **R2** Region centers relocated if underwater.
- **R3** Fewer than target nodes: place as many as fit, no panic.

### Rendering (V)

- **V1** Ocean cells: columns at SEA_LEVEL, ocean material.
- **V2** Interior cells: columns at render_height (smooth or stepped).
- **V3** ClearColor = deep blue. No ocean plane.
- **V4** Waterfall = water ∩ cliff -> shallow material (visual marker).
- **V5** All columns extend to DEPTH_FLOOR.

### Scaling (S)

- **S1** Deterministic from constants.
- **S2** No panics for any CELLS >= 1, any TILES >= 1.

## Known improvements (not yet implemented)

| # | Improvement | Source | Impact |
|---|------------|--------|--------|
| 1 | Spring scoring: height + distance-to-ocean | Critic r3 | Longer, more interesting rivers |
| 2 | River carving depth varies downstream | Critic r3 | Visual variety in riverbeds |
| 3 | Exclude water cells from smoothing kernel | Critic r3 | Sharper river/lake banks |
| 4 | Fixup-created lake cells: move from river_cells | Critic r3 | Correct water classification |
| 5 | Minimum dry-land and flat-area invariants | Critic r3 | Puzzle-constructor completeness |
| 6 | Waterfall cells exempt from smoothing | Critic r3 | Carving not undone by blur |

## Pipeline (11 steps, order matters)

```
 1. Heightmap         2-octave noise (global coords) + edge falloff
 2. Clamp + quantize  [-2, +3], snap 0.5
 3. Ocean             world-edge rim below SEA_LEVEL
 4. Springs           highest cells, spacing constraint
 5. Rivers            steepest descent from springs
 6. Lakes             local minima on river paths
 6b. River carving    -0.5 along paths, re-quantize
 6c. Feature fixup    guarantee lake + waterfall
 7. Resources         dry + flat, after all water
 8. Shore gradient    BFS outward, 3 levels
 9. Water depth       BFS inward, 3 levels
10. Cliff detection   neighbor diff > 1.0
11. Render heights    box-filter from mutated heights (or identity)
```

## Constants (current values)

```
CELLS=32  TILES_X=2  TILES_Z=2  (world 64×64)
TERRAIN_SEED=0xABCD_EF01_2345_6789
SEA_LEVEL=0.0  EDGE_MARGIN=2  EDGE_SINK=1.5  DEPTH_FLOOR=-3.0
SPRINGS_PER_TILE=2  SPRING_MIN_SPACING=10
TERRAIN_SMOOTH=true
```
