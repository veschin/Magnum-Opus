---
id: pixel-art-render-guide
kind: guide
source: https://www.davidhol.land/articles/3d-pixel-art-rendering/
references: docs/llm/refs/3d_pixel_art/
---

# 3D pixel-art render recipe

Reproduces the visual style documented in David Holland's 2024-09-24 article
"3D Pixel Art Rendering" (originally built in a custom Godot 4.3 fork). All
reference frames live in `refs/3d_pixel_art/`. The hero shot is
`refs/3d_pixel_art/01_meadow_godrays.webp`.

Goal: an orthographic 3D scene rendered at low resolution (~640x360),
nearest-neighbour upscaled, dressed with effects so the result reads as
hand-painted pixel art rather than blurry 3D.

This document is a recipe, not a Bevy implementation. No magnum_opus render
feature is owner-approved at the time of writing; treat the file as design
input for whatever render module the owner eventually requests.

## Visual signature

The aesthetic is the sum of small effects, none of which is novel on its
own. Drop any one and the result reverts to "low-res 3D":

1. Orthographic camera tilted ~30 deg down (dimetric flavour: refs 01, 04, 07).
2. Render-target resolution ~640x360, nearest upscale to window.
3. Toon shading: 2-3 light bands per material, no specular bloom (refs 01, 03, 09).
4. One-pixel outlines from depth + normal edge detection (ref 02).
5. Light edge highlights only on convex edges (ref 02 - pale top edges of rocks).
6. Cloud shadows: a single noise texture projected over the world (ref 03).
7. Volumetric god rays: raymarched parallel slabs aligned with the sun
   (refs 01, 07, 08).
8. Stylised water: planar reflection + horizontal wave texture + refraction
   (refs 04, 05, 06).
9. Billboarded grass and tree leaves with poisson-disk distribution
   (refs 03, 09).
10. 2D particles produced offline as tiling sprite sheets, played in scene.
11. Night: vignette, recoloured directional light, film grain.

## Effect 1 - Outlines

**Reference:** `refs/3d_pixel_art/02_rocks_outlines.webp`.

Algorithm: post-process pass over screen depth + normal textures. Kernel is
4 texels (up, down, left, right) - the minimum that produces single-pixel
outlines without aliasing.

Two outputs:

- **Object outlines.** Discontinuities in the depth texture mark silhouettes
  against the background.
- **Edge highlights.** Cross product of neighbouring normal texels, kept only
  when the dihedral indicates a convex edge. In ref 02 these are the pale
  one-pixel highlights along the top and front edges of the rocks.

Tuning is per-scene: there is no parameter set that works for every model
and camera angle at this resolution. Test against placeholder geometry
inside the actual scene.

References used by the source article:
- t3ssel8r - Creating a Pixel Art Scene in Realtime 3D
- Roystan - Outline Shader

**Bevy 0.18 mapping:** requires `DepthPrepass` + `NormalPrepass` on the
`Camera3d` (in `bevy::core_pipeline::prepass`). Outline runs as a
post-process node reading those prepass textures from WGSL. Bevy ships no
standard outline shader; write a custom material on a fullscreen quad that
samples both textures and runs the kernel.

## Effect 2 - Pixel-perfect camera

The temporal artefact (pixel creep, swimming) is invisible in stills; the
problem only appears once the camera scrolls.

Two-step fix for an orthographic camera:

1. **Snap** the camera position to a view-aligned grid sized one
   render-target texel. Kills creep but stutters motion.
2. **Shift** the final upscale blit by the snap error in screen space. Motion
   smooths out, no creep returns.

Snap and shift must use the same view-aligned axes, not world axes;
otherwise diagonals re-introduce creep.

**Bevy 0.18 mapping:** `Projection::Orthographic`. A system in the `Last`
schedule writes the snapped translation back to the camera transform and
publishes the sub-pixel error as a uniform read by the upscale blit
material.

## Effect 3 - Toon lighting

**References:** `01`, `03`, `09`. Note the 3-step gradient on rocks and the
flat top of the grass field.

Standard toon shading. Three deviations from a textbook setup:

- Smoothed attenuation on the directional light, to dampen flicker as the
  sun rotates slowly.
- Optional noise added to the surface normal, to break shadow popping on flat
  surfaces.
- Lighting state is exposed back to the outline pass so outlines can be
  brightened or darkened in shadow.

**Bevy 0.18 mapping:** custom `Material` with a stepped diffuse term. Read
through `bevy_pbr::mesh_view_bindings::lights` to stay consistent with
`StandardMaterial` cascade behaviour.

## Effect 4 - Cloud shadows

**Reference:** `refs/3d_pixel_art/03_grass_clouds.webp`. The diagonal darker
bands across the grass are projected cloud shadow.

Implementation: one noise texture set as a global shader uniform. A helper
function `get_cloud_attenuation(world_pos)` is called by every shader that
participates in lighting. The texture is sampled in world XZ and scrolls
slowly. Water, grass, ground - everything sun-lit reads the same uniform.

**Bevy 0.18 mapping:** noise texture as a `Resource` exposing a
`Handle<Image>`, attached via shader defines / a custom render group. Treat
as a shared shader include for every lit material.

## Effect 5 - Grass

**References:** `01` (grass tufts in clearings), `03` (cloud shadows on a
grass field).

Geometry: many billboard quads, each sampling a small grass tuft texture.

The trick is the lighting, not the geometry. Without intervention, each
billboard fragment shades on its own quad normal and screen position - tufts
that should belong to one patch get different brightness, ruining the
boundary illusion. The fix: rewrite `VERTEX` in the fragment shader to the
world position of the quad's base, so every fragment reports the same world
position to the lighting model. The whole tuft picks up one shadow value.

In Godot >=4.4 there is `LIGHT_VERTEX` for this exact purpose; the article
predates that and patches the engine source.

**Bevy 0.18 mapping:** custom `Material` for grass. Pass the base world
position as an instance attribute and feed it to the diffuse / shadow
lookup in WGSL where Bevy normally uses `mesh.world_position`.

## Effect 6 - Water

**References:** `04` (full water scene), `05` (outline-below-water
comparison, red split line in middle), `06` (procedural wave RGBA texture).

Three sub-problems.

### 6a. Outlines below water

**Ref `05`:** the left side shows the stone column whose underwater section
has no outline (the outline shader was forced into the transparent pass);
the right side shows the fixed version - outlines visible below water.

The fix in Godot was an engine patch shifting back-buffer copies to keep
depth/normal textures up to date with the current frame. The general
principle: the outline pass must populate depth/normal **before** the water
shader samples them. In Bevy this is the natural order if outlines run as
part of (or right after) the prepass; verify by sampling the textures from
a debug post-process before relying on them.

### 6b. Wave texture

**Ref `06`:** a single Voronoi-cell texture. RGB channels encode separate
quantities per cell (direction / variance / timing). One texture lookup
in the water shader produces the small horizontal wave dashes seen across
ref `04`.

The shader builds world-space, view-aligned horizontal lines using the
texture as a lookup table. The source author generated it in Material
Maker; any node-based texture tool produces an equivalent. Suggested
encoding when re-creating:

- R = primary direction angle
- G = variance amplitude
- B = phase / timing offset
- A = mask

Verify by previewing per-channel splits before plugging into the shader.

### 6c. Planar reflections

Screen-space reflections looked wrong at low resolution. The author switched
to planar reflections: render the scene from a mirrored camera into a
viewport texture, then sample that texture on the water plane.

The non-trivial part is the projection matrix - the reflecting camera needs
an oblique near plane that aligns with the water surface so geometry below
the water clips correctly. Godot 4.2 lacked `Camera3D` custom projection;
the author merged a pending PR. Bevy's `Camera::projection` supports a
`CustomProjection` variant; oblique frustum maths is in Lengyel's paper.

References used by the source article:
- Roystan - Toon Water Shader
- Catlike Coding - Looking Through Water
- paddy-exe - Stylized Water with DepthFade
- Bramwell - Improving 3D Water in Godot 4
- eldskald - Planar Reflections for Unity
- Lengyel - Oblique View Frustum Depth Projection and Clipping
- Pranckevicius - Oblique Near-Plane Clipping with Orthographic Camera

## Effect 7 - Volumetric god rays

**References:** `01`, `07` (rays over scene), `08` (depth-blur A/B).

Visual goal: shafts of light cutting through the scene, softened where they
intersect terrain and grass.

### 7a. Geometry-based version (initial attempt)

A stack of parallel quads aligned with the sun direction. For each fragment,
sample the directional shadow map at that fragment's world position; if lit,
contribute light scattering. Conceptually equivalent to shell texturing with
the shells oriented to the light, not the surface (Acerola's videos cover
the same idea).

### 7b. Depth fade and the grass problem

**Ref `08` left:** the rays harshly outline every individual grass billboard
because grass writes to depth.

Fix: box-blur the depth texture before testing it for the depth-fade
falloff, so per-tuft depth variation flattens out. Right side of `08` shows
the result - rays now read as soft shafts.

### 7c. Performance fix - raymarched post-process

The blur ran per plane x per fragment and was prohibitive even at 640x360.
Final implementation: drop the geometry, do everything in a single
fullscreen post-process pass. Define the planes mathematically, raymarch
into the scene, sample shadow map at each step. The depth blur runs once.
Cost on the author's GTX 1060: ~1.7 ms.

This requires the renderer to expose a "sample directional shadow at world
position" shader function. Bevy's `bevy_pbr::shadows` exposes
`fetch_directional_shadow`; check the binding layout when invoking it from
a post-process pipeline (different bind group than a forward material).

**Bevy 0.18 mapping:** post-process node bound after the main pass. WGSL
implements: ray = sun-aligned slab traversal -> at each step, reconstruct
world position from depth, sample shadow map, accumulate. Uniforms: sun
direction, slab spacing, march step count, max distance.

## Effect 8 - Trees

**Reference:** `refs/3d_pixel_art/09_tree.webp`.

Trunk is a small mesh. Leaves are many billboards distributed across the
canopy mesh by **poisson-disk sampling**: uniformly random points on a
mesh surface with a minimum-distance constraint between accepted samples.

Source-article implementation: a multithreaded Godot C++ module based on:

- "Fast Adaptive Blue Noise on Polygonal Surfaces" - Medeirosa, Ingridb,
  Pesco, Silvac
- "Parallel Poisson Disk Sampling with Spectrum Analysis on Surfaces" -
  Bowers, Wang, Wei, Maletz

Algorithm sketch:

1. Pick triangles weighted by surface area.
2. Sample a uniform random point inside each picked triangle.
3. Hash points into a spatial grid sized to the minimum radius.
4. Reject any point within the radius of an already-accepted neighbour.
5. Repeat until point density saturates.

Leaf shader is essentially the grass shader: billboard, fragment-stage
position rewrite, toon lighting.

For magnum_opus: poisson sampling is a pure CPU step done at asset build
time. No need to ship a runtime sampler.

## Effect 9 - Particles and night

**References:** `10` (Pixel Composer UI screenshot), `01` (small flying-leaf
particles in scene).

Approach: do not attempt to generate 2D-style particle frames inside the 3D
engine. Render seamlessly tiling, looping particle sheets in an offline
tool (the source article uses Pixel Composer - node-based, GameMaker-built,
open source). Play them in scene as world-space sprite sheets.

Suitable for: rain, water splash, dust, drifting leaves, sparkles.

For night:

- Recolour the directional light (cool desaturated blue).
- Add a screen-space vignette (radial darkening).
- Add film grain (animated noise overlay, low intensity).

These three together sell the time-of-day shift more than any change to the
sun colour alone.

## Render pipeline order (Bevy 0.18, suggested)

```
Prepass:
  - depth pre-pass (opaque + grass)
  - normal pre-pass

Main:
  - shadow caster pass for sun
  - opaque pass (toon material reads cloud-shadow uniform + sun shadow;
    grass uses base-position lighting hack)
  - reflective camera renders (water planar, oblique projection)
  - water transparent pass

Post:
  - outlines (depth + normal kernel) -> writes into colour buffer
  - god rays (raymarched, samples blurred depth + sun shadow)
  - colour grading (palette quantisation if used)
  - vignette
  - film grain (night only)
  - upscale blit with sub-pixel offset (closes the pixel-snap camera loop)
```

## Asset specs

Numbers below are inferred from inspecting the reference images, not stated
in the source article. Treat as starting points for iteration on
screenshots, not parameter sheets.

- **Internal render resolution:** ~640x360. Window scale: 3-4x.
- **Camera:** orthographic. Pitch ~30 deg, yaw 45 deg (classic dimetric).
  Verify by matching rock silhouettes in `01`.
- **Palette:** 4-5 ramp steps per material. Rocks: dark grey-blue ->
  light grey -> off-white edge. Grass: 5 greens dark->bright, plus pale
  speckles for cloud-shadow boundaries.
- **Wave texture:** 256x256 Voronoi RGBA, channel-encoded per 6b.
- **Particle sheets:** 128x128 tiles, 16-frame loops typical.
- **Outline kernel:** 4-tap (up/down/left/right).
- **Edge-highlight angle threshold:** convex only, dihedral > ~90 deg.

## Implementation order for magnum_opus

If/when an owner-approved render feature lands, the cheapest order is:

1. Low-resolution `Camera3d` + ortho projection + nearest-neighbour upscale
   blit. Get resolution and camera angle correct first; everything tunes
   against this.
2. Stepped toon `Material`. Verify shading bands match reference.
3. Depth + normal prepass + outline post-process. Iterate kernel until
   `02_rocks_outlines.webp` is reproduced on a single placeholder mesh.
4. Pixel-snap camera + sub-pixel shift. Test by orbiting a placeholder
   mesh; outlines must not crawl.
5. Cloud shadow uniform.
6. Billboard grass with base-position lighting.
7. Volumetric god rays as a raymarched post-process.
8. Water (planar reflection, wave texture, refraction).
9. Particles (offline-baked sprite sheets played in scene).
10. Vignette + grain + night palette.

Each of 1-10 is a separable PTSD feature. Do not bundle.

## Reference image index

| File | Effect | Use as |
|------|--------|--------|
| `refs/3d_pixel_art/01_meadow_godrays.webp`     | overall target               | hero shot - match this |
| `refs/3d_pixel_art/02_rocks_outlines.webp`     | outlines + edge highlights   | tuning reference for outline kernel |
| `refs/3d_pixel_art/03_grass_clouds.webp`       | cloud shadow on grass        | match shadow softness and band size |
| `refs/3d_pixel_art/04_water_scene.webp`        | water full stack             | match transparency, reflection, wave density |
| `refs/3d_pixel_art/05_water_outline_compare.webp` | underwater outlines fix   | A/B proof that outlines wire under water |
| `refs/3d_pixel_art/06_wave_texture.webp`       | wave RGBA encoding           | input texture for water shader |
| `refs/3d_pixel_art/07_godrays.webp`            | god rays + soft intersections | verify depth fade |
| `refs/3d_pixel_art/08_godrays_depth_blur.webp` | grass-outline artefact fix   | A/B before vs after depth blur |
| `refs/3d_pixel_art/09_tree.webp`               | tree leaf billboards         | poisson-disk density target |
| `refs/3d_pixel_art/10_pixel_composer_splash.webp` | Pixel Composer UI         | tool reference for offline particles |
| `refs/3d_pixel_art/11_raylib_wip.png`          | author's later raylib WIP    | minimal pipeline sanity check |

## External resources

Mirror of the source article's reference list, kept here so they are one
click away:

- t3ssel8r - Creating a Pixel Art Scene in Realtime 3D (YouTube)
- Roystan - Outline Shader, Toon Water Shader
- Catlike Coding - Looking Through Water
- CaptainProton42 - Flexible Toon Shader for Godot
- Acerola - shell texturing, graphics programming
- aarthifical - pixel snap camera (2D)
- Lengyel - Oblique View Frustum Depth Projection and Clipping
- Pranckevicius - Oblique Near-Plane Clipping with Orthographic Camera
- Pixel Composer - particle authoring tool
- Source article: David Holland, davidhol.land/articles/3d-pixel-art-rendering/,
  2024-09-24, with 2025-05-26 update on volumetrics.

Engine context: the source article was built in a custom Godot v4.3 fork.
Several of its engine patches have since landed upstream in Godot, and
Bevy 0.18 may already cover the same ground (prepass exposure, custom
projections, directional shadow sampling from post-process). Verify the
current Bevy 0.18 capability before duplicating any workaround the author
needed in Godot.
