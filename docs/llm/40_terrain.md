---
id: terrain-water-system
kind: spec
---

# Terrain & water system (v7)

Spring-based water on a multi-tile heightmap. Prototype target:
`magnum_opus/examples/grid_prototype.rs`. Contracts and invariants
in `41_terrain_contracts.md`.

## Height system

11 discrete levels from **-2 to +3** in **0.5-block** steps.
One block = `CELL_SIZE` world units.

Two noise octaves with smoothstep interpolation:

| Octave | Scale | Amplitude | Purpose |
|--------|-------|-----------|---------|
| 1 | 16 | 2.5 | Broad plateaus, valleys |
| 2 | 6 | 0.8 | Organic edge roughness |

Offset = +1.0. Raw range [-2.3, +4.3], clamped [-2, +3].
~60% cells at height 0-2 (buildable). ~20% hills. ~20% valleys.

Edge falloff at world boundary (EDGE_MARGIN=2, EDGE_SINK=1.5)
guarantees ocean rim.

## Multi-tile world

World = TILES_X × TILES_Z tiles, each CELLS × CELLS cells.
Global noise coordinates -> seamless edges across tile boundaries.
Edge falloff at WORLD boundary, not per-tile.
Generated as a single pass (heightmap + water + resources).

## Water model

### Springs

2 springs per tile placed at highest-elevation cells with spacing
constraint. Each spring traces steepest descent to ocean. Path = river.
Local minimum on path = lake. Cliff crossing on path = waterfall.

### River carving

Terrain lowered -0.5 along each river path after tracing. Creates
visible channels. Can amplify existing height differences into cliffs.
Re-quantized after carving.

### Feature fixup

After generation, validates: >=1 lake and >=1 waterfall exist. If not,
injects them by terrain mutation at optimal points on existing rivers.

### Ocean

World-boundary cells (dist < EDGE_MARGIN) below SEA_LEVEL. Rendered
as columns at SEA_LEVEL with ocean material. No ocean plane - ClearColor
(deep blue) shows through gaps at world edge.

### Gradients

Shore: BFS from water outward, 3 levels of darkening ground.
Depth: BFS from shore inward, 3 levels of darkening water.

## Rendering

Column geometry: each cell is a unit cuboid scaled to
`(CELL_SIZE, column_height, CELL_SIZE)`. Column extends from
surface to DEPTH_FLOOR.

12 materials: 4 ground gradient (grass -> shore), cliff, 3 water
gradient (shallow -> deep), ocean, 3 resource colors.

Smooth mode: box-filter from mutated heights (carving included).
Stepped mode: quantized heights directly.

## Terrain features as gameplay

Each terrain type corresponds to a building type. Generator
guarantees at least one instance of each feature:

| Feature | Condition | Guarantee |
|---------|-----------|-----------|
| Flat plain | h=0-1, neighbors ±0.5 | Noise offset -> majority |
| Hill | h >= 2 | Noise amplitude |
| Valley | h <= -1, dry | Noise amplitude |
| Cliff | neighbor diff > 1.0 | Noise gradient |
| River | spring path | >=2 springs |
| Lake | minimum on path | Fixup if missing |
| Waterfall | river ∩ cliff | Fixup if missing |
| Shore | land adjacent ocean | Edge falloff |
