# Visual Style & Render Pipeline

## Goal

Pixel-art aesthetic with dynamic lighting and volume. Every object reacts to light sources, casts correct shadows, and overlaps other objects with per-pixel depth accuracy — all from flat sprite quads, no 3D meshes at runtime.

## 1. Impostor Sprite Format

Every visible game object is a flat quad carrying three texture maps:

| Map | Channels | Encoding | Purpose |
|-----|----------|----------|---------|
| Albedo | RGBA | sRGB color, A = transparency mask | Base color, no lighting baked in |
| Normal | RGB | Tangent-space: R=X, G=Y, B=Z. Value 0.5 = zero, 0.0 = -1, 1.0 = +1. Flat surface = (0.5, 0.5, 1.0) | Per-pixel surface direction for lighting |
| Depth | R (grayscale) | 0.0 = flush with quad plane, 1.0 = maximum protrusion toward camera | Per-pixel height above the quad. Used for z-sorting, self-shadow, parallax |

Optional fourth map:
| Emission | RGB | HDR-ish: values > 0 = glow intensity per channel | Glowing parts (forges, runes, lava). Additive, ignores lighting |

**Atlas layout:** all maps for one object packed into a single texture atlas or stored as separate files with matching dimensions. Matching is by pixel — albedo pixel (x,y) corresponds to normal pixel (x,y) and depth pixel (x,y).

**Sprite size convention:** each game tile = N×N pixels in the low-res target (e.g. 16×16 or 32×32). Buildings occupying 2×2 tiles = 2N×2N sprite. Exact N chosen after target resolution is locked.

## 2. Camera

**Projection:** orthographic (no perspective distortion). Required for pixel-perfect grid alignment.

**Isometric angle:** true isometric = camera tilted 35.264° from horizontal (arctan(1/√2)), producing equal foreshortening on all three axes. X and Y axes appear at 30° from horizontal on screen.

**World-to-screen transform** (for a tile at grid position `(gx, gy)`):
```
screen_x = (gx - gy) * tile_half_width
screen_y = (gx + gy) * tile_half_height
```
Where `tile_half_width = N/2`, `tile_half_height = N/4` (2:1 diamond ratio, standard isometric).

**Camera is fixed** — no rotation, no zoom at runtime (zoom = art direction, locked at export time). Scroll only (WASD / edge pan).

## 3. Low-Resolution Render Target

Render the entire scene to a small off-screen texture, then upscale to the window.

**Resolution:** target around 320×180 to 480×270 (16:9). The exact value determines how "crunchy" the pixels look. 320×180 at 1920×1080 output = 6× upscale = very chunky pixels. 480×270 = 4× upscale = slightly finer.

**Upscale filter:** nearest-neighbor (point sampling). This is what creates the hard pixel edges. Bilinear/trilinear would blur and destroy the pixel-art look.

**Aspect ratio:** the low-res target and the window must share the same aspect ratio (or use letterboxing) to avoid non-square pixels.

## 4. Lighting Pass

Runs per-pixel on every sprite quad. Inputs: albedo, normal map, depth map, light uniforms.

### 4.1 Normal Decoding

Raw normal map stores values in [0, 1]. Decode to direction vector in [-1, 1]:

```
N = normalize(texture(normal_map, uv).rgb * 2.0 - 1.0)
```

This gives a tangent-space normal where:
- X+ = right on the sprite
- Y+ = up on the sprite
- Z+ = toward camera (out of screen)

For isometric, the sprite quad is screen-aligned, so tangent-space normals map directly to view-space — no extra transform needed.

### 4.2 Directional Light (sun / ambient)

```
L_dir = normalize(light_direction)  // e.g. (-0.5, -0.7, 0.5) — top-left, toward viewer
NdotL = max(dot(N, L_dir), 0.0)
diffuse = light_color * NdotL
```

### 4.3 Point Lights (torches, forges, spell effects)

Each point light has: `position` (screen-space), `color`, `radius`, `intensity`.

```
// pixel's world position approximated from sprite position + UV offset
pixel_pos = sprite_world_pos + vec2(uv - 0.5) * sprite_size
light_vec = light_pos - pixel_pos
dist = length(light_vec)
attenuation = max(1.0 - dist / light_radius, 0.0)  // linear falloff; can use quadratic
L = normalize(vec3(light_vec, light_z_offset))       // Z component: light is above the scene
NdotL = max(dot(N, L), 0.0)
point_diffuse += light_color * light_intensity * NdotL * attenuation
```

### 4.4 Ambient

Flat ambient term to prevent fully black shadows:
```
ambient = ambient_color * ambient_strength   // e.g. (0.15, 0.12, 0.18) * 0.3
```

### 4.5 Self-Shadow from Depth Map

Depth map encodes height. Neighbors with lower depth that face away from light are in shadow:
```
// simplified: compare depth at current pixel with depth at pixel offset toward light
depth_here = texture(depth_map, uv).r
depth_light_side = texture(depth_map, uv + light_offset * texel_size).r
shadow = depth_here > depth_light_side + shadow_bias ? shadow_strength : 1.0
```

This creates contact shadows in crevices and under overhangs — cheaply, per-sprite.

### 4.6 Final Lighting Composite

```
lit_color = albedo.rgb * (ambient + (diffuse + point_diffuse) * shadow)
           + emission.rgb   // additive, unaffected by lighting
```

## 5. Post-Processing Chain

Applied to the full low-res render target (not per-sprite). Order matters.

### 5.1 Outline

Edge detection on the scene's depth buffer and normal buffer using a Sobel operator:

```
// sample 3×3 neighborhood of depth values
Gx = sobel_horizontal_kernel * depth_samples  // horizontal gradient
Gy = sobel_vertical_kernel * depth_samples      // vertical gradient
depth_edge = length(vec2(Gx, Gy))

// repeat for normals (compare encoded RGB, or dot-product difference)
normal_edge = max_normal_discontinuity_in_3x3(normal_samples)

edge = max(depth_edge, normal_edge)
if edge > outline_threshold:
    pixel = outline_color  // typically black (0,0,0)
```

**Outline thickness** = 1 pixel in the low-res target = multiple screen pixels after upscale. For thicker outlines, use a larger kernel (5×5) or dilate.

Depth edges catch object silhouettes. Normal edges catch surface detail (ridges, seams, panel lines) even when depth is continuous.

### 5.2 Toon Shading

Quantize the luminance (or each light contribution) into discrete bands:

```
bands = 3  // tunable: 2 = very flat, 4 = subtle
luminance = dot(lit_color, vec3(0.299, 0.587, 0.114))
toon_factor = floor(luminance * bands + 0.5) / bands
lit_color *= toon_factor / max(luminance, 0.001)
```

This creates sharp light/shadow boundaries instead of smooth gradients — the "cel-shaded" look.

### 5.3 Posterization

Reduce color depth per channel:

```
levels = 8  // tunable: 4 = very retro, 16 = almost smooth
posterized.r = floor(color.r * levels + 0.5) / levels
posterized.g = floor(color.g * levels + 0.5) / levels
posterized.b = floor(color.b * levels + 0.5) / levels
```

Can also map to a fixed palette (LUT texture) instead of uniform quantization — this gives more artistic control over which colors appear.

### 5.4 Pixelation (optional)

If the low-res target doesn't produce chunky enough pixels, apply an additional downscale:

```
pixel_size = 2  // group NxN low-res pixels into one
snapped_uv = floor(uv * target_resolution / pixel_size) * pixel_size / target_resolution
color = texture(scene, snapped_uv)
```

Usually the low-res target is enough. This pass exists as a fallback for fine-tuning.

### 5.5 Upscale

Blit the low-res target to the screen framebuffer with **nearest-neighbor** sampling. No interpolation. This is the final step.

## 6. Per-Pixel Depth Sorting

In isometric 2D, sprites overlap. Standard painter's algorithm (sort by Y) breaks when objects have complex shapes. The depth map fixes this.

**Approach:** each sprite writes to a scene depth buffer during rendering.

```
sprite_base_z = sort_key_from_grid_position(gx, gy)  // e.g. gx + gy, higher = further
pixel_z = sprite_base_z - depth_map_value * depth_scale
```

Pixels with higher depth map values (protruding toward camera) get a smaller z → drawn in front. The GPU depth test (or a manual depth buffer in 2D) resolves per-pixel overlap correctly.

**Result:** a tree trunk correctly occludes a building behind it, but the building's roof peeks over the tree's canopy — per-pixel, not per-sprite.

## 7. Procedural Animations (Shader-Driven)

No extra sprite frames needed. All driven by uniforms: `time`, `entity_id` (for phase offset), tunable parameters.

### Vertex Displacement

```
// breathing / idle bob: offset Y by sin wave
offset.y = sin(time * frequency + entity_id * 0.7) * amplitude

// wind sway (trees, flags): offset X based on vertex height
offset.x = sin(time * wind_speed + world_pos.x * 0.3) * sway_strength * vertex_y_normalized
```

### UV Animation

```
// flowing liquid: scroll UV along one axis
uv.x += time * flow_speed

// spinning gear: rotate UV around a pivot
uv = rotate2d(uv - pivot, time * spin_speed) + pivot

// palette cycling (lava, magic glow): offset into a 1D palette texture
palette_index = fract(base_index + time * cycle_speed)
color = texture(palette_lut, vec2(palette_index, 0.0))
```

### Emission Pulse

```
// forge glow, rune shimmer: modulate emission intensity
emission_strength = base_emission + sin(time * pulse_freq) * pulse_amplitude
final_emission = emission_map.rgb * emission_strength
```

### Normal Map Rotation for Animated Parts

When a sprite sub-region rotates (gear, fan), the normal map must rotate with it so lighting stays correct:

```
rotated_normal.xy = mat2(cos(a), -sin(a), sin(a), cos(a)) * normal.xy
rotated_normal.z = normal.z  // Z (toward camera) unchanged
```

## 8. Asset Categories

### Static (buildings, terrain, trees, decorations)

One impostor per object: albedo + normal + depth (+ optional emission). Drawn as a single quad. Animated only via procedural shader effects (section 7).

### Modular Buildings

Each module (wall, roof, window, tower, balcony) = separate impostor with own maps. Assembled at runtime by grid rules:
- Same-type adjacency → merge visuals (extended wall segments)
- Different-type adjacency → connection module (arch, bridge)
- Module-level transforms for assembly animation: translate from off-screen, scale from 0, rotate into place

### Transport

- Resource items: one impostor per type, translated along a spline path. Bounce via vertex displacement `sin(time + path_progress)`.
- Rune paths: tiled ground sprites with UV scroll for shimmer.
- Pipes: tiled sprites, liquid inside = UV scroll along pipe axis + palette cycling.

### Creatures & Minions (open — requires separate research)

Need frame-based animation (walk, idle, attack, death) across 4+ facing directions.

Planned pipeline: AI-generate 3D model → auto-rig (Mixamo) → animate → Blender headless renders spritesheet with albedo + normal + depth per frame. In-engine: standard spritesheet playback, lighting shader reads per-frame normal/depth.

Open problems:
- Non-humanoid rigging (Mixamo doesn't cover)
- Style consistency across generated models
- Direction count (4 mirrored to 8 vs native 8)

## 9. Visual Paths (Harmony vs Knowledge)

Two aesthetic themes controlled by:
1. **Module sprite sets** — different albedo/normal/emission art per path
2. **Shader parameters** — different posterization palette (LUT), outline color, emission tint

| | Harmony | Knowledge |
|---|---|---|
| Shapes | Organic, rounded, botanical | Angular, twisted, eldritch |
| Palette | Green, gold, warm tones | Purple, deep blue, acid green |
| Emission | Warm glow (amber, soft white) | Cold pulse (cyan, magenta) |
| Idle FX | Slow breathing, gentle sway | Rapid flicker, tremor |
| Outline | Soft brown or dark green | Harsh black or deep purple |

## 10. Asset Generation Pipeline

All assets AI-generated. No hand-drawn art. 3D used as intermediate format — never rendered at runtime.

**Why 3D intermediate:**
- Normal + depth maps extracted from geometry are mathematically exact (vs neural net guesses from 2D)
- One shader parameter change reskins every asset (impossible with baked 2D style)
- Animation from rigging, not frame-by-frame regeneration (which drifts)
- Scriptable: new asset = generate model + run batch pipeline

**Pipeline:**

```
Source:     2D concept (Flux / SD / InvokeAI)
               │
Convert:    2D → 3D (PartCrafter / Hi3DGen) or direct 3D gen (Meshy / Tripo)
               │
Branch:     ┌─ Static → render 1 isometric view
            ├─ Modular → split into parts → render each part
            └─ Animated → rig → animate → render spritesheet
               │
Render:     Blender headless, isometric ortho camera:
            → albedo pass (diffuse color, no lighting)
            → normal pass (world-space normals → remap to tangent-space)
            → depth pass (Z-buffer normalized to [0,1])
            → emission pass (if applicable)
               │
Output:     PNG atlas per object → engine loads as sprite material
```

**Tool categories** (specific tools evolve — maintain a separate registry):
- 2D generation: Flux, Stable Diffusion, InvokeAI (self-hosted)
- 2D → 3D: PartCrafter (part-aware, up to 16 meshes), Hi3DGen (high detail)
- 3D generation: Meshy, Tripo, Sloyd.ai
- Depth from 2D (fallback for simple assets): DepthAnything v2, Laigter
- Auto-rigging: Mixamo (humanoids)

## References

- [3D Pixel Art Rendering](https://www.davidhol.land/articles/3d-pixel-art-rendering/) — core technique (low-res target + nearest-neighbor + outline/posterization)
- Prior design: `~/work/game/tech/RENDER_STYLE.md`, `~/work/game/concepts/RENDER.md`
- Procedural buildings: `~/work/game/tech/PROCEDURAL_BUILDINGS.md`
- AI pipeline research: `~/work/game/tech/FAQ.md` (Q2)
