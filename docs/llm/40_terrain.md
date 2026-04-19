---
id: terrain-water-system
kind: spec
---

# Terrain & water system

Design spec for the heightmap-based terrain and hydrological model.
Prototype target: `magnum_opus/examples/grid_prototype.rs`.

## 1. Height system

### Levels

Discrete heights from **-5 to +5** in **0.5-block** increments (21 levels).
One block = `CELL_SIZE` world units. A cell at height 3 has its surface at
`3.0 * CELL_SIZE` world-Y.

### Generation

Three-octave value noise with smoothstep interpolation:

| Octave | Scale (cells/period) | Amplitude (blocks) | Purpose |
|--------|---------------------|--------------------|----|
| 1 | 16 | 3.0 | Broad hills, valleys |
| 2 | 6 | 1.5 | Medium undulation |
| 3 | 3 | 0.5 | Local roughness |

Raw sum range: +-5.0. Quantized: `(raw * 2.0).round() / 2.0`.

Each octave uses a different seed derived from the master terrain seed
to avoid axis-aligned artifacts.

### Edge falloff

Guarantees ocean at all borders regardless of map size:

```
falloff(x, z) = min(dist_to_nearest_edge(x, z) / EDGE_MARGIN, 1.0)
height(x, z) = noise(x, z) * falloff(x, z) - EDGE_SINK * (1.0 - falloff(x, z))
```

`EDGE_MARGIN`: 3 cells (tunable). `EDGE_SINK`: 2.0 blocks.

At the border: `falloff = 0`, height = `-EDGE_SINK` = -2.0 (below sea level).
At margin+1 inward: full noise amplitude.

This creates a natural coastline slope. On a 2x2 map all cells are near the
edge, producing a low-lying islet. On a 64x64 map only the outer 3 cells
are affected.

### Cliffs

A cliff exists where adjacent cells differ by > 1.0 block (2+ height levels).
These emerge naturally from noise gradients and quantization. The steeper the
noise gradient at a quantization boundary, the taller the cliff.

Cliff cells receive a rock/earth material instead of grass, visually marking
the transition.

### Basins

After noise generation, 2-4 circular depressions are stamped into the
heightmap to guarantee lake-forming basins:

```
for each basin:
    center = random cell, inset 4+ from edge
    radius = 2-4 cells
    depth  = 1.5-2.5 blocks (gaussian falloff from center)
    height[cell] -= depth * (1.0 - dist/radius)
```

Basin cells below sea level become lake candidates.

## 2. Water model

### Sea level

**Sea level = 0.0 blocks.** All water bodies reference this datum.

### Ocean

A single large plane at Y = 0, extending 3x the grid width in each
direction. The grid (the "island") sits on top. Terrain above sea level
protrudes; terrain below is submerged.

The ocean is **not part of the tile grid** -- it is post-hoc visual framing.
Any grid size (2x2, 10x10, 64x64) works: the island shape adapts via edge
falloff, the ocean plane is always there.

Ocean material: darkest water shade.

### Lakes

Lakes are **not explicitly placed**. They emerge from the heightmap:

1. Generate heightmap with basins (section 1).
2. Flood from all border cells via BFS through cells <= sea level.
   These become **ocean-connected** (submerged coast).
3. Interior cells below sea level that are NOT reached by ocean BFS =
   **lake cells**. They form isolated inland basins.
4. Lake water surface = **rim height** of the basin (the lowest cell
   on the basin's perimeter that is above sea level). This means
   highland lakes can exist above sea level if the basin rim is high.

Lake size is controlled by basin depth and radius: a deeper/wider basin
floods more cells. With 2-4 basins of radius 2-4 cells, typical lakes
are 6-20 cells.

### Rivers

Rivers follow **steepest descent** from basin rims to the ocean:

```
1. For each lake, find the rim cell (lowest perimeter cell above water).
2. From the rim, greedily walk to the lowest unvisited neighbor.
3. Continue until reaching an ocean cell (border/below sea level)
   or an existing water body.
4. If the walk gets stuck (all neighbors higher), terminate.
```

Width: 1 cell by default, 2 cells for major rivers (hash-determined).
Width-2 rivers duplicate each cell with one perpendicular neighbor.

River cells are at terrain height, creating visible steps where the
river crosses a height boundary.

### Waterfalls

Where a river crosses a cliff (adjacent river cells differ by > 1 block),
a waterfall exists. In the prototype this is visible as a height gap in the
river column. Future: animated water material on the cliff face.

### Water gradient

BFS from all non-water cells inward. Distance to nearest shore determines
water shade:

```
d=1: shallow (lightest blue)
d=2: medium
d=3+: deep (darkest, before ocean)
ocean: deepest shade
```

### Shore gradient

BFS from all water cells outward. Distance to nearest water determines
ground shade:

```
d=1: dark wet earth
d=2: damp
d=3: slightly moist
d=4+: normal grass
```

Three-cell transition zone on each side of the waterline.

## 3. Rendering

### Column geometry

Each cell is a tall cuboid extending from its surface down to a fixed
depth floor (below all possible terrain):

```
surface_y = height * CELL_SIZE
column_height = (height - DEPTH_FLOOR) * CELL_SIZE
center_y = surface_y - column_height / 2.0
```

`DEPTH_FLOOR = -6.0` (1 block below minimum terrain).

Where adjacent cells differ in height, the taller column's side face is
exposed, naturally rendering cliff walls. No extra geometry needed.

One mesh per height level (21 meshes), shared across all cells at that
level. Total column geometry: CELLS^2 entities, 12 triangles each.

### Ocean plane

Single cuboid, width = 3 * grid_extent, depth = 0.5 * CELL_SIZE,
centered on grid center at Y = sea_level. Uses ocean material (darkest
blue). Rendered behind/below all terrain.

### Materials (12 total)

| # | Name | Approx color | Use |
|---|------|-------------|-----|
| 1 | grass | rgb(0.34, 0.62, 0.28) | Normal ground |
| 2 | shore_3 | rgb(0.32, 0.59, 0.27) | 3 cells from water |
| 3 | shore_2 | rgb(0.30, 0.56, 0.25) | 2 cells from water |
| 4 | shore_1 | rgb(0.28, 0.52, 0.24) | Adjacent to water |
| 5 | cliff | rgb(0.45, 0.38, 0.30) | Cliff face / steep cells |
| 6 | shallow | rgb(0.35, 0.68, 0.90) | Water edge |
| 7 | medium | rgb(0.28, 0.60, 0.85) | Water depth 2 |
| 8 | deep | rgb(0.22, 0.52, 0.78) | Water depth 3+ |
| 9 | ocean | rgb(0.12, 0.38, 0.62) | Ocean plane + submerged coast |
| 10-12 | copper, metal, coal | per-resource | Resource node primitives |

### Resource placement on terrain

Nodes only on cells **above sea level**. All cells in a node template
must be within 0.5 blocks of each other (no node spanning a cliff).
Node primitives sit on terrain surface:

```
y = terrain_height(gx, gz) * CELL_SIZE + prim.half_height() * scale
```

Height variation (+/-20%) per primitive within a node preserved from
the current implementation.

## 4. Scaling

The system adapts to any grid size via edge falloff:

| Grid | Land cells (approx) | Character |
|------|-------------------|-----------|
| 2x2 | 0-2 | Tiny rock in ocean |
| 4x4 | 2-6 | Small islet |
| 10x10 | ~50 | Small island |
| 32x32 | ~750 | Medium island with inland features |
| 64x64 | ~3400 | Large island with rivers, multiple lakes |

Noise parameters are fixed -- larger maps show more noise detail,
smaller maps show only the broadest features. Basin count can scale
with map area: `max(1, area / 256)`.

## 5. Generation pipeline (order matters)

```
1. Heightmap           3-octave noise + edge falloff + basin stamps
2. Quantize            snap to 0.5 blocks
3. Ocean flood BFS     from border through cells <= 0 -> mark ocean
4. Lake detection      interior cells below rim height, not ocean
5. Resource nodes      above sea level, height-compatible cells only
6. Rivers              steepest descent from lake rims to ocean
7. Shore gradient BFS  water->outward, 3 levels
8. Depth gradient BFS  shore->inward, 3 levels
9. Cliff detection     neighbor height diff > 1.0
```

## 6. Constants (tunables)

```
TERRAIN_SEED: u64
SEA_LEVEL: f32 = 0.0
EDGE_MARGIN: i32 = 3      // falloff width in cells
EDGE_SINK: f32 = 2.0      // height reduction at border
DEPTH_FLOOR: f32 = -6.0   // column bottom
BASIN_COUNT_MIN: u64 = 2
BASIN_COUNT_MAX: u64 = 4
BASIN_RADIUS_MIN: i32 = 2
BASIN_RADIUS_MAX: i32 = 4
BASIN_DEPTH_MIN: f32 = 1.5
BASIN_DEPTH_MAX: f32 = 2.5
RIVER_MAX: u64 = 3
RIVER_WIDTH_MAX: u32 = 2
```

## 7. Future extensions (not in prototype)

- **Waterfall VFX**: animated blue material on cliff faces at river
  height transitions.
- **Biomes**: height-based material selection (snow above 4, rock 2-4,
  grass 0-2, mud below 0).
- **Erosion pass**: post-gen smoothing along river paths, widening
  valleys.
- **Tide simulation**: ocean level oscillates, flooding/exposing
  coastal cells.
- **Cave/tunnel**: cells with height gap below terrain surface.
