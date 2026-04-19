---
id: terrain-water-system
kind: spec
---

# Terrain & water system

Design spec for the heightmap-based terrain and hydrological model.
Uses D8 flow accumulation to derive lakes and rivers from terrain topology.
Seamless noise-based generation across a multi-tile world.

## 1. Height system

### Levels

Discrete heights from **-5 to +5** in **0.5-block** increments (21 levels).
One block = `CELL_SIZE` world units. A cell at height 3 has its surface at
`3.0 * CELL_SIZE` world-Y.

### Generation

Three-octave value noise with smoothstep interpolation:

| Octave | Scale (cells/period) | Amplitude (blocks) | Seed | Purpose |
|--------|---------------------|--------------------|----|---|
| 1 | 16 | 3.0 | TERRAIN_SEED | Broad hills, valleys |
| 2 | 6 | 1.5 | TERRAIN_SEED ^ 0xFF01 | Medium undulation |
| 3 | 3 | 0.5 | TERRAIN_SEED ^ 0xFF02 | Local roughness |

Each octave uses `noise_octave(gx, gz, scale, seed)` - parameterized
bilinear interpolation with smoothstep (`t = t*t*(3-2t)`).

### Edge falloff

Guarantees ocean at all world boundaries:

```
dist = min(gx, WORLD_W-1-gx, gz, WORLD_H-1-gz)
falloff = min(dist / EDGE_MARGIN, 1.0)
height = noise * falloff - EDGE_SINK * (1.0 - falloff)
```

At border (`dist=0`): `falloff=0`, height = `-EDGE_SINK`.
At `EDGE_MARGIN` inward: full amplitude.

### Quantization

After edge falloff: `h = clamp(h, -5.0, 5.0)`, then `(h * 2).round() / 2`.
Clamping BEFORE quantization prevents out-of-range mesh indices.

### Terrain features (height transitions)

| Adjacent cell diff | Name | Visual | Water interaction |
|-------------------|------|--------|-------------------|
| 0.0 | Flat | Same level | Standing water, delta |
| 0.5 | Gentle slope | Barely visible step | Smooth flow |
| 1.0 | Step | Visible ledge | Small cascade |
| 1.5-2.0 | Cliff | Rock face visible | Waterfall (small) |
| 2.5-3.0 | High cliff | Tall rock face | Waterfall (medium) |
| 3.5+ | Wall | Maximum height | Waterfall (tall) |

Cliff threshold: diff > 1.0 block. Cells at or above this threshold
receive cliff material (rock/earth) on their top face.

### Depression types

| Shape | Cells | Depth from rim | Water body |
|-------|-------|---------------|------------|
| Dimple | 1 | 0.5-1.0 | Too small for lake (W2) |
| Bowl | 3-6 | 1.0-1.5 | Pond |
| Basin | 6-14 | 2.0-3.0 | Lake |
| Wide basin | 14+ | 1.0-2.0 | Large lake |
| Valley | 2-4 wide, 10+ long | 1.0-3.0 | River channel |
| Canyon | 1-2 wide | 3.0+ | Gorge with waterfall |

## 2. Multi-tile world

World = `TILES_X × TILES_Z` tiles. Each tile is a square grid of cells.

Seamless noise generation via global coordinates: `gx = tx * TILE_WIDTH + cx`,
`gz = tz * TILE_HEIGHT + cz`. Single `TERRAIN_SEED` produces identical edges
across tile boundaries (no seams).

Ocean covers all cells at world boundaries (`gx < 0`, `gx >= WORLD_W`, etc).
Land and water features tile seamlessly across the entire world.

## 3. Water model

### Sea level

**Sea level = 0.0 blocks.** All water bodies reference this datum.

### Ocean

A single large cuboid at `Y = SEA_LEVEL * CELL_SIZE`, extending beyond
the world boundary. The world is the "island" sitting on top. Ocean is
post-hoc visual framing, not part of the tile grid.

**Ocean cells get NO terrain column** (V2). Only the ocean plane
covers them. This eliminates z-fighting.

### Lakes (D8 flow-based local minima)

Lakes emerge from terrain via D8 flow accumulation:

```
1. For each non-ocean cell, compute D8 steepest descent (8-neighbor).
2. Trace flow downhill to find local minima (cells with no downhill neighbor).
3. A minimum is a LAKE if:
   - It has no outflow to ocean (isolated basin), AND
   - The connected component of minimum height cells >= 2 cells (W2)
4. Lake water surface = height of the minimum cell.
5. Lake cells = all cells in that connected component (same height).
```

This finds lakes at ANY elevation. A depression at height +3 with no path
to sea produces a highland lake at surface +3.

### Rivers (flow accumulation above threshold)

Rivers emerge where flow-accumulation exceeds a threshold:

```
1. For each non-ocean cell, count how many cells flow into it (D8 accumulation).
2. Any cell with accumulation >= RIVER_THRESHOLD is a RIVER cell.
3. River cells remain at terrain height (no carving).
4. Rivers are continuous 4-connected paths of accumulation cells.
```

Rivers form naturally from terrain topology without explicit path-finding.

### Water-terrain interactions

```
River + cliff (diff > 1.0)       = WATERFALL
River + depression (no outlet)   = fills, becomes LAKE
River + flat plateau             = flows via lowest path
River + another river            = both are accumulation cells (merge)
Lake overflow (local minimum)    = RIVER SOURCE
Ocean + shore at height 0        = flat coastline
Ocean + shore at height 2+       = coastal cliff
```

### Gradients

**Shore gradient** (ground cells near water):
BFS from all water cells outward (4-connected).

```
d=1: rgb(0.28, 0.52, 0.24) -- dark wet earth
d=2: rgb(0.30, 0.56, 0.25) -- damp
d=3: rgb(0.32, 0.59, 0.27) -- slightly moist
d=4+: rgb(0.34, 0.62, 0.28) -- normal grass
```

**Water depth gradient** (water cells near shore):
BFS from all non-water cells inward (4-connected).

```
d=1: rgb(0.35, 0.68, 0.90) -- shallow edge
d=2: rgb(0.28, 0.60, 0.85) -- medium
d=3+: rgb(0.22, 0.52, 0.78) -- deep
ocean: rgb(0.12, 0.38, 0.62) -- always darkest
```

## 4. Rendering

### Column geometry

Each non-ocean cell is a tall cuboid from surface to `DEPTH_FLOOR`:

```
surface_y = height * CELL_SIZE
column_h  = (height - DEPTH_FLOOR) * CELL_SIZE
center_y  = surface_y - column_h / 2.0
```

`DEPTH_FLOOR = -6.0` (1 block below minimum terrain).

Adjacent cells at different heights expose the taller column's side
face - natural cliff walls with no extra geometry.

21 pre-computed meshes (one per height level), shared via `Handle<Mesh>`.

### Lake cell rendering

Lake cells render at **lake water surface height**, NOT at
terrain height. Column extends from water surface to DEPTH_FLOOR.
Uses water-gradient material. Underwater terrain is hidden.

`water_surfaces: BTreeMap<(i32,i32), f32>` maps each lake cell to
its basin's water surface height.

### Ocean plane

Single cuboid at `Y = SEA_LEVEL * CELL_SIZE`. Ocean material (darkest blue).

### Materials (12)

| # | Name | Color | Condition |
|---|------|-------|-----------|
| 1 | grass | rgb(0.34, 0.62, 0.28) | Ground, shore_dist >= 4 |
| 2 | shore_3 | rgb(0.32, 0.59, 0.27) | Ground, shore_dist = 3 |
| 3 | shore_2 | rgb(0.30, 0.56, 0.25) | Ground, shore_dist = 2 |
| 4 | shore_1 | rgb(0.28, 0.52, 0.24) | Ground, shore_dist = 1 |
| 5 | cliff | rgb(0.45, 0.38, 0.30) | Ground, max neighbor diff > 1.0 |
| 6 | shallow | rgb(0.35, 0.68, 0.90) | Water, depth = 1 |
| 7 | medium | rgb(0.28, 0.60, 0.85) | Water, depth = 2 |
| 8 | deep | rgb(0.22, 0.52, 0.78) | Water, depth >= 3 |
| 9 | ocean | rgb(0.12, 0.38, 0.62) | Ocean plane |
| 10-12 | copper, metal, coal | per-resource | Node primitives |

Cliff overrides shore gradient (cliff takes priority).

## 5. Pipeline (order matters)

```
 1. Heightmap         3-octave noise + edge falloff (global coords)
 2. Clamp + quantize  clamp [-5,+5], snap to 0.5
 3. Ocean flood BFS   border cells & those <= SEA_LEVEL
 4. D8 flow descent   each cell to steepest downhill neighbor
 5. Flow accumulation count inflow from all cells per D8 descent
 6. Lake detection    local minima (no downhill) at ocean boundary
 7. River detection   cells with accumulation >= RIVER_THRESHOLD
 8. All water         ocean + lakes + rivers combined
 9. Resources         dry + flat cells only, centers relocated if underwater
10. Shore gradient    BFS water -> outward, 3 levels
11. Water depth       BFS shore -> inward, 3 levels
12. Cliff detection   max neighbor height diff > 1.0
```

Resources placed AFTER rivers to prevent river overwriting nodes.

## 6. Invariants

### Height (H)

- **H1** All heights in [-5.0, +5.0], multiples of 0.5.
  Enforced: clamp before quantize.
- **H2** All world boundary cells < SEA_LEVEL.
  Enforced: edge falloff on WORLD_W/WORLD_H (falloff=0 at boundary).
- **H3** At least 1 interior cell > SEA_LEVEL when world >= 6x6.
  Below 6: degenerate (all ocean) accepted.

### Water (W)

- **W1** Ocean = connected set via BFS through cells <= SEA_LEVEL from boundary.
- **W2** Lake = interior D8 local minimum (no downhill neighbor) with >= 2 cells.
  Water surface = minimum cell height.
  All lake cells = minimum height.
- **W3** Lake never connected to ocean.
- **W4** No cell has multiple water types (ocean/lake/river exclusive).
- **W5** River = cell with D8 flow accumulation >= RIVER_THRESHOLD.
  Connected 4-path of accumulation cells.
- **W6** River never flows uphill (D8 descent is monotonic).

### Resources (R)

- **R1** All node cells on dry ground (not in any water set).
- **R2** All template cells within 0.5 blocks of each other.
- **R3** Region center relocated to nearest dry cell if underwater.
- **R4** Fewer than NODES_PER_REGION valid positions: place as many as fit.

### Rendering (V)

- **V1** Ocean plane at Y = SEA_LEVEL * CELL_SIZE.
- **V2** Ocean cells: NO terrain column. Only ocean plane covers them.
- **V3** Lake cells: column at water surface height.
  Requires water_surfaces BTreeMap in layout result.
- **V4** River cells: column at terrain height, water-gradient material.
- **V5** Ground cells: column at terrain height, ground/shore/cliff material.
- **V6** All non-ocean columns extend to DEPTH_FLOOR.
- **V7** Cliff material on cells where max neighbor diff > 1.0.

### Scaling (S)

- **S1** Deterministic: same constants = identical output.
- **S2** No panics for any world size >= 1x1.

## 7. Corner cases

| Case | Behavior | Status |
|------|----------|--------|
| 1x1 world | Single ocean cell | Valid |
| 2x2 world | All cells near boundary, all ocean | Valid |
| All cells flat | No local minima, no rivers (accumulation=0) | Valid |
| Local minimum isolated by ocean | No lake (boundary condition) | No lake, valid |
| Two minima same elevation | Each forms separate lake | Multiple lakes |
| High elevation minimum | Highland lake at its height | Pour-point-less |
| Accumulation cell on cliff | Waterfall implied | Visual only |
| Multiple cells same accumulation | All are river cells (merged branch) | Valid |
| No dry cells for resources | All underwater or too steep | 0 nodes, no panic |
| Height below -5 after clamping | World boundary falloff | Clamped by H1 |

## 8. Constants

```
TERRAIN_SEED: u64
SEA_LEVEL: f32 = 0.0
EDGE_MARGIN: i32 = 3
EDGE_SINK: f32 = 2.0
DEPTH_FLOOR: f32 = -6.0
WORLD_W: i32
WORLD_H: i32
TILES_X: i32 = WORLD_W / TILE_WIDTH
TILES_Z: i32 = WORLD_H / TILE_HEIGHT
RIVER_THRESHOLD: u64
TERRAIN_SMOOTH: u32
```

## 9. Future extensions

- **Waterfall VFX**: animated water on cliff faces at river steps.
- **Biomes**: height-based materials (snow >4, rock 2-4, grass 0-2).
- **Erosion**: post-gen smoothing along rivers, valley widening.
- **Tides**: oscillating ocean level, coastal flooding.
