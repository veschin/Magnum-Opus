---
id: terrain-water-system
kind: spec
---

# Terrain & water system

Design spec for the heightmap-based terrain and hydrological model.
Prototype target: `magnum_opus/examples/grid_prototype.rs`.
Verified through two adversarial critic rounds (Opus).

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

Each octave uses `noise_octave(gx, gz, scale, seed)` -- parameterized
bilinear interpolation with smoothstep (`t = t*t*(3-2t)`).

### Edge falloff

Guarantees ocean at all borders regardless of map size:

```
dist = min(gx, CELLS-1-gx, gz, CELLS-1-gz)
falloff = min(dist / EDGE_MARGIN, 1.0)
height = noise * falloff - EDGE_SINK * (1.0 - falloff)
```

At border (`dist=0`): `falloff=0`, height = `-EDGE_SINK`.
At `EDGE_MARGIN` inward: full amplitude.

### Quantization

After edge falloff: `h = clamp(h, -5.0, 5.0)`, then `(h * 2).round() / 2`.
Clamping BEFORE quantization prevents out-of-range mesh indices.

### Basins

After noise + falloff, 2-4 circular depressions stamped to guarantee
lake-forming terrain:

```
center = random cell, inset 4+ from edge
radius = 2-4 cells
depth  = 1.5-2.5 blocks, linear falloff from center
height[cell] -= depth * (1.0 - dist/radius)
```

Re-clamp and re-quantize after stamping. Basin center is always at least
1.0 block below pre-stamp height (H4).

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
| Dimple | 1 | 0.5-1.0 | Too small for lake (W3) |
| Bowl | 3-6 | 1.0-1.5 | Pond |
| Basin | 6-14 | 2.0-3.0 | Lake |
| Wide basin | 14+ | 1.0-2.0 | Large lake |
| Valley | 2-4 wide, 10+ long | 1.0-3.0 | River channel |
| Canyon | 1-2 wide | 3.0+ | Gorge with waterfall |

## 2. Water model

### Sea level

**Sea level = 0.0 blocks.** All water bodies reference this datum.

### Ocean

A single large cuboid at `Y = SEA_LEVEL * CELL_SIZE`, extending 3x
the grid width in each direction. The grid is the "island" sitting on
top. The ocean is post-hoc visual framing, not part of the tile grid.

**Ocean cells get NO terrain column** (V2). Only the ocean plane
covers them. This eliminates z-fighting.

### Lakes (pour-point basin detection)

Lakes emerge from terrain, not from explicit placement:

```
1. Find all local minima (non-ocean cells <= all 4 neighbors)
2. For each minimum, priority-flood outward:
   - Min-priority queue, process lowest cell first
   - Add cell to basin, push 4-neighbors
   - First cell that is higher than basin floor AND
     borders a non-basin cell = POUR POINT
   - water_surface = pour_point height
3. Lake cells = basin cells with height < water_surface
4. Discard if < 2 lake cells (W3)
```

This finds lakes at ANY elevation. A basin at height +3 surrounded by
terrain at +4 produces a highland lake with surface at +4.

### Rivers (steepest descent)

```
1. Start at each lake's pour_point
2. Walk to lowest unvisited 4-neighbor
3. Tiebreak (equal height): prefer cell closest to border (manhattan)
4. Stop when: reaching ocean, existing water, or uphill (no carving)
5. Width 1-2 cells (hash-determined per river)
6. Width-2: perpendicular = 90-degree rotation of flow direction
```

River cells sit at terrain height. Where a river crosses a cliff
(consecutive cells differ by >1 block), a waterfall is implied.

Dead-end rivers (stuck on plateau with no downhill path) are rare
due to tiebreaking and are accepted as valid.

### Water-terrain interactions

```
River + cliff (diff > 1.0)       = WATERFALL
River + depression (no outlet)   = fills, becomes LAKE
River + flat plateau             = meanders via tiebreak to border
River + another river            = second terminates at junction
Lake overflow (pour point)       = RIVER SOURCE downhill
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

## 3. Rendering

### Column geometry

Each non-ocean cell is a tall cuboid from surface to `DEPTH_FLOOR`:

```
surface_y = height * CELL_SIZE
column_h  = (height - DEPTH_FLOOR) * CELL_SIZE
center_y  = surface_y - column_h / 2.0
```

`DEPTH_FLOOR = -6.0` (1 block below minimum terrain).

Adjacent cells at different heights expose the taller column's side
face -- natural cliff walls with no extra geometry.

21 pre-computed meshes (one per height level), shared via `Handle<Mesh>`.

### Lake cell rendering

Lake cells render at **water_surface height** (pour point), NOT at
terrain height. Column extends from water_surface to DEPTH_FLOOR.
Uses water-gradient material. Underwater terrain is hidden inside
the column.

`water_surfaces: BTreeMap<(i32,i32), f32>` maps each lake cell to
its basin's pour-point height.

### Ocean plane

Single cuboid: `width = 3 * grid_extent`, `height = 0.3 * CELL_SIZE`,
at `Y = SEA_LEVEL * CELL_SIZE`. Ocean material (darkest blue).

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

## 4. Pipeline (order matters)

```
 1. Heightmap         3-octave noise + edge falloff
 2. Basin stamp       2-4 circular depressions
 3. Clamp + quantize  clamp [-5,+5], snap to 0.5
 4. Ocean flood BFS   border cells through h <= SEA_LEVEL
 5. Lake detection    pour-point on non-ocean local minima
 6. Rivers            steepest descent from pour points (BEFORE resources)
 7. All water         ocean + lakes + rivers combined
 8. Resources         dry + flat cells only, centers relocated if underwater
 9. Shore gradient    BFS water -> outward, 3 levels
10. Water depth       BFS shore -> inward, 3 levels
11. Cliff detection   max neighbor height diff > 1.0
```

Resources placed AFTER rivers to prevent river overwriting nodes.

## 5. Invariants

### Height (H)

- **H1** All heights in [-5.0, +5.0], multiples of 0.5.
  Enforced: clamp after basin stamp, before quantize.
- **H2** All border cells < SEA_LEVEL.
  Enforced: edge falloff (falloff=0 at border, EDGE_SINK > 0).
- **H3** At least 1 interior cell > SEA_LEVEL when CELLS >= 6.
  Below 6: degenerate (all ocean) accepted.
- **H4** Basin center >= 1.0 block below pre-stamp height.
  Enforced: BASIN_DEPTH_MIN = 1.5.

### Water (W)

- **W1** Ocean = connected set of all border cells,
  BFS through cells <= SEA_LEVEL.
- **W2** Lake = interior basin NOT connected to ocean.
  Every lake cell height < water_surface.
  Water surface = pour point height.
- **W3** Lake minimum 2 cells. Single-cell depressions = ground.
- **W4** No cell has multiple water types (ocean/lake/river exclusive).
- **W5** River = connected 4-path, pour_point to ocean/lake.
  Tiebreak on plateaus: prefer cell closest to border.
  Dead-end if truly stuck (rare, accepted). No terrain carving.
- **W6** River never flows uphill. Plateau (equal height) allowed.

### Resources (R)

- **R1** All node cells on dry ground (not in any water set).
- **R2** All template cells within 0.5 blocks of each other.
- **R3** Region center relocated to nearest dry cell if underwater.
- **R4** Fewer than NODES_PER_REGION valid positions: place as many
  as fit. No panic.

### Rendering (V)

- **V1** Ocean plane at Y = SEA_LEVEL * CELL_SIZE.
  No z-fighting because V2 prevents ocean columns.
- **V2** Ocean cells: NO terrain column. Only ocean plane covers them.
- **V3** Lake cells: column at water_surface height, not terrain height.
  Requires water_surfaces BTreeMap in layout result.
- **V4** River cells: column at terrain height, water-gradient material.
- **V5** Ground cells: column at terrain height, ground/shore/cliff material.
- **V6** All non-ocean columns extend to DEPTH_FLOOR. No gaps.
- **V7** Cliff material on cells where max neighbor diff > 1.0.
  Applied to entire cell top face (known visual compromise for prototype).

### Scaling (S)

- **S1** Deterministic: same constants = identical output.
- **S2** No panics for any CELLS >= 1.

## 6. Corner cases

| Case | Behavior | Status |
|------|----------|--------|
| CELLS=1 | Single border cell, all ocean | Valid |
| CELLS=2-3 | All cells near border, 0-1 land | Valid |
| CELLS=4-5 | Tiny islet, 0-1 basins | Valid |
| All basins merge with ocean | Basin near coast, ocean BFS reaches it | No lake, valid |
| Overlapping basins | Combined deeper depression | One large lake |
| Basin on high terrain (+4) | Highland lake if rim > center | Pour-point handles it |
| River on flat plateau | All neighbors same height | Tiebreak: closest to border |
| River enters another lake | Steepest descent reaches lake B | Terminates at B |
| River stuck uphill | Single-cell dip, not lake-sized | Terminates, rare |
| Two rivers converge | River B hits river A cells | B terminates at A |
| Region center underwater | Heightmap floods region | BFS relocate to nearest dry |
| No valid node positions | All dry cells occupied/steep | 0 nodes, no panic |
| Height below -5 after stamp | Basin + edge sink | Clamped by H1 |
| Ocean cell at height 0.0 | Border, exactly SEA_LEVEL | Ocean, no column (V2) |
| Deep lake (-3, surface +1) | Column from +1 to DEPTH_FLOOR | All blue, terrain hidden |

## 7. Constants

```
TERRAIN_SEED: u64
SEA_LEVEL: f32 = 0.0
EDGE_MARGIN: i32 = 3
EDGE_SINK: f32 = 2.0
DEPTH_FLOOR: f32 = -6.0
BASIN_SEED: u64
BASIN_COUNT_MIN: u64 = 2
BASIN_COUNT_MAX: u64 = 4
BASIN_RADIUS_MIN: i32 = 2
BASIN_RADIUS_MAX: i32 = 4
BASIN_DEPTH_MIN: f32 = 1.5
BASIN_DEPTH_MAX: f32 = 2.5
RIVER_SEED: u64
RIVER_MAX: u64 = 3
```

## 8. Scaling

| Grid | Land cells (approx) | Character |
|------|-------------------|-----------|
| 2x2 | 0-2 | Tiny rock in ocean |
| 4x4 | 2-6 | Small islet |
| 10x10 | ~50 | Small island |
| 32x32 | ~750 | Medium island, inland lakes |
| 64x64 | ~3400 | Large island, river networks |

Noise parameters fixed. Basin count scales: `max(1, area / 256)`.

## 9. Future extensions

- **Waterfall VFX**: animated water on cliff faces at river steps.
- **Biomes**: height-based materials (snow >4, rock 2-4, grass 0-2).
- **Erosion**: post-gen smoothing along rivers, valley widening.
- **Tides**: oscillating ocean level, coastal flooding.
