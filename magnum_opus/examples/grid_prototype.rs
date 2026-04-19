//! Pixel-art render prototype: toon shading with shadow attenuation,
//! depth+normal outline post-process, low-res nearest-neighbour upscale.
//!
//! Run:
//!   cargo run --example grid_prototype
//!   SCREENSHOT=1 cargo run --example grid_prototype  (auto-exit, saves PNG)

use bevy::prelude::*;
use bevy::camera::MainPassResolutionOverride;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass, ViewPrepassTextures};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::FullscreenShader;
use bevy::render::render_graph::{RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner};
use bevy::render::render_resource::{
    binding_types::{sampler, texture_2d, texture_depth_2d},
    AsBindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, Operations,
    PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, TextureFormat,
    TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::render::view::ViewTarget;
use bevy::render::{RenderApp, RenderStartup};
use bevy::shader::ShaderRef;
use bevy::utils::default as bevy_default;
use magnum_opus::core::{AppExt, CorePlugin};
use magnum_opus::grid::GridModule;
use magnum_opus::world_config::WorldConfigModule;
use std::collections::{BTreeMap, BTreeSet, VecDeque};


// --- Tunables ---
const CELLS: u32 = 32;
const CELL_SIZE: f32 = 1.5;
const TILES_X: u32 = 2;
const TILES_Z: u32 = 1;
const WORLD_W: u32 = CELLS * TILES_X;
const WORLD_H: u32 = CELLS * TILES_Z;
const NODES_PER_REGION: usize = 6;
const REGION_RADIUS: i32 = 5;
const NODE_MARGIN: i32 = 1;
const SPAWN_SEED: u64 = 0xAABB_CCDD_EEFF_0011;
const MAX_SPAWN_ATTEMPTS: u64 = 400;
// --- Terrain ---
const TERRAIN_SEED: u64 = 0xABCD_EF01_2345_6789;
const SEA_LEVEL: f32 = 0.0;
const EDGE_MARGIN: i32 = 2;
const EDGE_SINK: f32 = 1.5;
const DEPTH_FLOOR: f32 = -3.0;
// --- Water ---
const SPRINGS_PER_TILE: u32 = 2;
const SPRING_SEED: u64 = 0xFADE_DEAD_BEEF_0003;
const SPRING_MIN_SPACING: i32 = 10;
// --- Rendering ---
const TERRAIN_SMOOTH: bool = true;

fn main() {
    let screenshot = std::env::var("SCREENSHOT").as_deref() == Ok("1");
    let title: String = if screenshot {
        "claude-dev-grid-prototype".into()
    } else {
        "magnum-opus grid prototype".into()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title,
            resolution: (1280u32, 720u32).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(MaterialPlugin::<ToonMaterial>::default());
    app.add_plugins(OutlinePostProcessPlugin);
    app.add_plugins(CorePlugin);
    app.add_data::<WorldConfigModule>();
    app.add_sim::<GridModule>();
    app.finalize_modules();
    app.add_systems(Startup, setup_scene);
    if screenshot {
        app.add_systems(Update, screenshot_then_exit);
    }
    app.run();
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct ToonMaterial {
    #[uniform(0)]
    base_color: LinearRgba,
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wgsl".into()
    }
}

// --- Primitive node system ---

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum Prim {
    Cube,
    Tall,
    Boulder,
    Post,
    Spike,
    Pebble,
}

impl Prim {
    fn mesh(self) -> Mesh {
        let s = CELL_SIZE;
        match self {
            Prim::Cube => Cuboid::new(s, s, s).into(),
            Prim::Tall => Cuboid::new(s, 1.6 * s, s).into(),
            Prim::Boulder => Sphere::new(0.5 * s).mesh().ico(2).unwrap(),
            Prim::Post => Cylinder::new(0.5 * s, s).into(),
            Prim::Spike => Cone {
                radius: 0.5 * s,
                height: s,
            }
            .into(),
            Prim::Pebble => Capsule3d::new(0.4 * s, 0.2 * s).into(),
        }
    }

    fn half_height(self) -> f32 {
        let s = CELL_SIZE;
        match self {
            Prim::Cube => 0.5 * s,
            Prim::Tall => 0.8 * s,
            Prim::Boulder => 0.5 * s,
            Prim::Post => 0.5 * s,
            Prim::Spike => 0.5 * s,
            Prim::Pebble => 0.5 * s,
        }
    }
}

struct NodeTemplate {
    cells: &'static [(i32, i32)],
}

// Only self-symmetric footprints.
const TEMPLATES: &[NodeTemplate] = &[
    // 2 cells: line
    NodeTemplate {
        cells: &[(0, 0), (1, 0)],
    },
    // 4 cells: 2x2 square
    NodeTemplate {
        cells: &[(0, 0), (1, 0), (0, 1), (1, 1)],
    },
    // 4 cells: rhombus (plus-center)
    NodeTemplate {
        cells: &[(1, 0), (0, 1), (2, 1), (1, 2)],
    },
    // 6 cells: 3x2 rectangle
    NodeTemplate {
        cells: &[(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)],
    },
    // 6 cells: H (two columns)
    NodeTemplate {
        cells: &[(0, 0), (0, 1), (0, 2), (2, 0), (2, 1), (2, 2)],
    },
    // 6 cells: T
    NodeTemplate {
        cells: &[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2), (1, 3)],
    },
];

fn hash64(mut x: u64) -> u64 {
    x = (x ^ (x >> 33)).wrapping_mul(0xff51afd7ed558ccd);
    x = (x ^ (x >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^ (x >> 33)
}

fn noise_octave(gx: i32, gz: i32, scale: f32, seed: u64) -> f32 {
    let fx = gx as f32 / scale;
    let fz = gz as f32 / scale;
    let ix = fx.floor() as i64;
    let iz = fz.floor() as i64;
    let mut tx = fx - ix as f32;
    let mut tz = fz - iz as f32;
    tx = tx * tx * (3.0 - 2.0 * tx);
    tz = tz * tz * (3.0 - 2.0 * tz);
    let corner = |cx: i64, cz: i64| -> f32 {
        let h = hash64(seed.wrapping_add(cx as u64 * 7919).wrapping_add(cz as u64 * 6271));
        (h % 1000) as f32 / 500.0 - 1.0
    };
    let h00 = corner(ix, iz);
    let h10 = corner(ix + 1, iz);
    let h01 = corner(ix, iz + 1);
    let h11 = corner(ix + 1, iz + 1);
    let h0 = h00 + (h10 - h00) * tx;
    let h1 = h01 + (h11 - h01) * tx;
    h0 + (h1 - h0) * tz
}

fn terrain_height(gx: i32, gz: i32) -> f32 {
    let n1 = noise_octave(gx, gz, 16.0, TERRAIN_SEED) * 1.5;
    let n2 = noise_octave(gx, gz, 6.0, TERRAIN_SEED ^ 0xFF01) * 0.3;
    let raw = n1 + n2 + 1.5;
    let ww = WORLD_W as i32 - 1;
    let wh = WORLD_H as i32 - 1;
    let dist = gx.min(ww - gx).min(gz.min(wh - gz)) as f32;
    let falloff = (dist / EDGE_MARGIN as f32).min(1.0);
    raw * falloff - EDGE_SINK * (1.0 - falloff)
}

fn compute_ocean(heights: &BTreeMap<(i32, i32), f32>) -> BTreeSet<(i32, i32)> {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let mut ocean = BTreeSet::new();
    for gx in 0..ww {
        for gz in 0..wh {
            let dist = gx.min(ww - 1 - gx).min(gz.min(wh - 1 - gz));
            if dist < EDGE_MARGIN {
                if heights.get(&(gx, gz)).copied().unwrap_or(0.0) < SEA_LEVEL {
                    ocean.insert((gx, gz));
                }
            }
        }
    }
    ocean
}

fn place_springs(
    heights: &BTreeMap<(i32, i32), f32>,
    ocean: &BTreeSet<(i32, i32)>,
) -> Vec<(i32, i32)> {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let total_springs = (TILES_X * TILES_Z * SPRINGS_PER_TILE) as usize;

    let mut candidates: Vec<(i32, i32)> = heights
        .iter()
        .filter(|&(&(gx, gz), &h)| {
            h > SEA_LEVEL + 0.5
                && !ocean.contains(&(gx, gz))
                && gx >= EDGE_MARGIN + 1
                && gx < ww - EDGE_MARGIN - 1
                && gz >= EDGE_MARGIN + 1
                && gz < wh - EDGE_MARGIN - 1
        })
        .map(|(&pos, _)| pos)
        .collect();

    candidates.sort_by(|a, b| heights[b].partial_cmp(&heights[a]).unwrap());

    let mut springs: Vec<(i32, i32)> = Vec::new();
    for &c in &candidates {
        if springs.len() >= total_springs {
            break;
        }
        let too_close = springs
            .iter()
            .any(|s| (s.0 - c.0).abs().max((s.1 - c.1).abs()) < SPRING_MIN_SPACING);
        if !too_close {
            springs.push(c);
        }
    }
    springs
}

fn trace_river(
    start: (i32, i32),
    heights: &BTreeMap<(i32, i32), f32>,
    ocean: &BTreeSet<(i32, i32)>,
    existing_water: &BTreeSet<(i32, i32)>,
) -> (Vec<(i32, i32)>, Option<BTreeSet<(i32, i32)>>) {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let mut path: Vec<(i32, i32)> = Vec::new();
    let mut visited = BTreeSet::new();
    let mut current = start;
    let mut lake: Option<BTreeSet<(i32, i32)>> = None;

    loop {
        if ocean.contains(&current) || existing_water.contains(&current) {
            break;
        }
        if visited.contains(&current) {
            break;
        }
        visited.insert(current);
        path.push(current);

        let neighbors = [
            (current.0 + 1, current.1),
            (current.0 - 1, current.1),
            (current.0, current.1 + 1),
            (current.0, current.1 - 1),
        ];
        let next = neighbors
            .iter()
            .filter(|&&(nx, nz)| nx >= 0 && nx < ww && nz >= 0 && nz < wh)
            .filter(|&&n| !visited.contains(&n))
            .min_by(|&&a, &&b| {
                let ha = if ocean.contains(&a) {
                    f32::MIN
                } else {
                    heights[&a]
                };
                let hb = if ocean.contains(&b) {
                    f32::MIN
                } else {
                    heights[&b]
                };
                match ha.partial_cmp(&hb).unwrap() {
                    std::cmp::Ordering::Equal => {
                        let da = a.0.min(ww - 1 - a.0).min(a.1.min(wh - 1 - a.1));
                        let db = b.0.min(ww - 1 - b.0).min(b.1.min(wh - 1 - b.1));
                        da.cmp(&db)
                    }
                    ord => ord,
                }
            });

        match next {
            Some(&n) => {
                let h_curr = heights[&current];
                let h_next = if ocean.contains(&n) {
                    SEA_LEVEL - 1.0
                } else {
                    heights[&n]
                };
                if h_next > h_curr {
                    let mut lake_cells = BTreeSet::new();
                    lake_cells.insert(current);
                    for &(nx, nz) in &neighbors {
                        if nx >= 0
                            && nx < ww
                            && nz >= 0
                            && nz < wh
                            && !ocean.contains(&(nx, nz))
                            && heights[&(nx, nz)] <= h_curr
                        {
                            lake_cells.insert((nx, nz));
                        }
                    }
                    if lake_cells.len() >= 2 {
                        lake = Some(lake_cells);
                    }
                    break;
                }
                current = n;
            }
            None => break,
        }
    }
    (path, lake)
}

fn compute_shore_distance(all_water: &BTreeSet<(i32, i32)>) -> BTreeMap<(i32, i32), u8> {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let mut dist: BTreeMap<(i32, i32), u8> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for &(gx, gz) in all_water {
        for (nx, nz) in [(gx + 1, gz), (gx - 1, gz), (gx, gz + 1), (gx, gz - 1)] {
            if nx >= 0
                && nx < ww
                && nz >= 0
                && nz < wh
                && !all_water.contains(&(nx, nz))
                && !dist.contains_key(&(nx, nz))
            {
                dist.insert((nx, nz), 1);
                queue.push_back((nx, nz));
            }
        }
    }
    while let Some((x, z)) = queue.pop_front() {
        let d = dist[&(x, z)];
        if d >= 4 {
            continue;
        }
        for (nx, nz) in [(x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)] {
            if nx >= 0
                && nx < ww
                && nz >= 0
                && nz < wh
                && !all_water.contains(&(nx, nz))
                && !dist.contains_key(&(nx, nz))
            {
                dist.insert((nx, nz), d + 1);
                queue.push_back((nx, nz));
            }
        }
    }
    dist
}

fn compute_water_depth(all_water: &BTreeSet<(i32, i32)>) -> BTreeMap<(i32, i32), u8> {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let mut dist: BTreeMap<(i32, i32), u8> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for &(gx, gz) in all_water {
        let on_edge = [(gx + 1, gz), (gx - 1, gz), (gx, gz + 1), (gx, gz - 1)]
            .iter()
            .any(|&(nx, nz)| {
                nx < 0 || nx >= ww || nz < 0 || nz >= wh || !all_water.contains(&(nx, nz))
            });
        if on_edge {
            dist.insert((gx, gz), 1);
            queue.push_back((gx, gz));
        }
    }
    while let Some((x, z)) = queue.pop_front() {
        let d = dist[&(x, z)];
        if d >= 4 {
            continue;
        }
        for (nx, nz) in [(x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)] {
            if nx >= 0
                && nx < ww
                && nz >= 0
                && nz < wh
                && all_water.contains(&(nx, nz))
                && !dist.contains_key(&(nx, nz))
            {
                dist.insert((nx, nz), d + 1);
                queue.push_back((nx, nz));
            }
        }
    }
    dist
}

fn detect_cliffs(heights: &BTreeMap<(i32, i32), f32>) -> BTreeSet<(i32, i32)> {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let mut cliffs = BTreeSet::new();
    for gx in 0..ww {
        for gz in 0..wh {
            let h = heights[&(gx, gz)];
            let max_diff = [(gx + 1, gz), (gx - 1, gz), (gx, gz + 1), (gx, gz - 1)]
                .iter()
                .filter_map(|&(nx, nz)| {
                    if nx < 0 || nx >= ww || nz < 0 || nz >= wh {
                        return None;
                    }
                    Some((heights[&(nx, nz)] - h).abs())
                })
                .fold(0.0_f32, f32::max);
            if max_diff > 1.0 {
                cliffs.insert((gx, gz));
            }
        }
    }
    cliffs
}

struct LayoutResult {
    heights: BTreeMap<(i32, i32), f32>,
    render_heights: BTreeMap<(i32, i32), f32>,
    ocean: BTreeSet<(i32, i32)>,
    river_cells: BTreeSet<(i32, i32)>,
    lake_cells: BTreeSet<(i32, i32)>,
    nodes: Vec<NodeSpawn>,
    shore_dist: BTreeMap<(i32, i32), u8>,
    water_depth: BTreeMap<(i32, i32), u8>,
    cliffs: BTreeSet<(i32, i32)>,
}

// --- Resources ---

#[derive(Clone, Copy, Debug)]
enum Resource {
    Copper,
    Metal,
    Coal,
}

impl Resource {
    fn primitive(self) -> Prim {
        match self {
            Resource::Copper => Prim::Boulder,
            Resource::Metal => Prim::Cube,
            Resource::Coal => Prim::Post,
        }
    }

    fn color(self) -> LinearRgba {
        match self {
            Resource::Copper => LinearRgba::rgb(0.80, 0.42, 0.18),
            Resource::Metal => LinearRgba::rgb(0.55, 0.58, 0.62),
            Resource::Coal => LinearRgba::rgb(0.14, 0.12, 0.11),
        }
    }
}

fn region_centers() -> [(IVec2, Resource); 3] {
    let cx = WORLD_W as f32 * 0.5;
    let cy = WORLD_H as f32 * 0.5;
    let offset = (WORLD_W.min(WORLD_H) as f32) * 0.28;
    let compute = |i: f32, res: Resource| -> (IVec2, Resource) {
        let angle = i * std::f32::consts::TAU / 3.0 + std::f32::consts::FRAC_PI_2;
        let x = (cx + offset * angle.cos()).round() as i32;
        let y = (cy + offset * angle.sin()).round() as i32;
        (IVec2::new(x, y), res)
    };
    [
        compute(0.0, Resource::Copper),
        compute(1.0, Resource::Metal),
        compute(2.0, Resource::Coal),
    ]
}

struct NodeSpawn {
    origin: IVec2,
    template_idx: usize,
    resource: Resource,
}

fn spawn_node(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<ToonMaterial>,
    origin: IVec2,
    template: &NodeTemplate,
    prim: Prim,
    heights: &BTreeMap<(i32, i32), f32>,
) {
    let mesh = meshes.add(prim.mesh());
    let base_y = prim.half_height();
    for &(dx, dy) in template.cells {
        let h = hash64(
            SPAWN_SEED
                .wrapping_add(origin.x as u64 * 31)
                .wrapping_add(origin.y as u64 * 97)
                .wrapping_add(dx as u64 * 13)
                .wrapping_add(dy as u64 * 7),
        );
        let scale = 0.8 + (h % 401) as f32 / 1000.0;
        let terrain_y = heights
            .get(&(origin.x + dx, origin.y + dy))
            .copied()
            .unwrap_or(0.0)
            * CELL_SIZE;
        let x = ((origin.x + dx) as f32 + 0.5) * CELL_SIZE;
        let z = ((origin.y + dy) as f32 + 0.5) * CELL_SIZE;
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, terrain_y + base_y * scale, z)
                .with_scale(Vec3::new(1.0, scale, 1.0)),
        ));
    }
}

fn generate_layout() -> LayoutResult {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;

    // 1. Heightmap
    let mut heights: BTreeMap<(i32, i32), f32> = BTreeMap::new();
    for gx in 0..ww {
        for gz in 0..wh {
            heights.insert((gx, gz), terrain_height(gx, gz));
        }
    }

    // 2. Clamp + quantize
    for h in heights.values_mut() {
        *h = (h.clamp(-2.0, 3.0) * 2.0).round() / 2.0;
    }

    // 3. Ocean
    let ocean = compute_ocean(&heights);

    // 4. Springs
    let springs = place_springs(&heights, &ocean);

    // 5-6. Rivers + lakes
    let mut river_cells = BTreeSet::new();
    let mut lake_cells = BTreeSet::new();
    let mut all_water_so_far: BTreeSet<(i32, i32)> = ocean.clone();

    for &spring in &springs {
        let (path, lake) = trace_river(spring, &heights, &ocean, &all_water_so_far);
        for &cell in &path {
            river_cells.insert(cell);
            all_water_so_far.insert(cell);
        }
        if let Some(lk) = lake {
            for &cell in &lk {
                lake_cells.insert(cell);
                all_water_so_far.insert(cell);
            }
        }
    }

    // 7. Resources
    let mut placed: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut nodes: Vec<NodeSpawn> = Vec::new();

    for (region_idx, (center, resource)) in region_centers().iter().enumerate() {
        let mut actual_center = *center;
        if all_water_so_far.contains(&(actual_center.x, actual_center.y)) {
            let mut best = actual_center;
            let mut best_dist = i32::MAX;
            for gx in 0..ww {
                for gz in 0..wh {
                    if !all_water_so_far.contains(&(gx, gz)) {
                        let d = (gx - center.x).abs() + (gz - center.y).abs();
                        if d < best_dist {
                            best_dist = d;
                            best = IVec2::new(gx, gz);
                        }
                    }
                }
            }
            actual_center = best;
        }

        let mut region_count = 0;
        let jitter = REGION_RADIUS * 2 + 1;
        for attempt in 0..MAX_SPAWN_ATTEMPTS {
            if region_count >= NODES_PER_REGION {
                break;
            }
            let s = hash64(
                SPAWN_SEED
                    .wrapping_add((region_idx as u64) * 0x100)
                    .wrapping_add(attempt),
            );
            let tmpl_idx = (s as usize) % TEMPLATES.len();
            let template = &TEMPLATES[tmpl_idx];
            let (mut max_dx, mut max_dy) = (0_i32, 0_i32);
            for &(dx, dy) in template.cells {
                max_dx = max_dx.max(dx);
                max_dy = max_dy.max(dy);
            }
            let ox_off = (hash64(s ^ 0xA1) as i32).rem_euclid(jitter) - REGION_RADIUS;
            let oy_off = (hash64(s ^ 0xB2) as i32).rem_euclid(jitter) - REGION_RADIUS;
            let ox = (actual_center.x + ox_off).clamp(0, ww - max_dx - 1);
            let oy = (actual_center.y + oy_off).clamp(0, wh - max_dy - 1);
            let h0 = heights[&(ox, oy)];
            let mut fits = true;
            'check: for &(dx, dy) in template.cells {
                let cx = ox + dx;
                let cy = oy + dy;
                if all_water_so_far.contains(&(cx, cy)) {
                    fits = false;
                    break;
                }
                if (heights[&(cx, cy)] - h0).abs() > 0.5 {
                    fits = false;
                    break;
                }
                for mx in -NODE_MARGIN..=NODE_MARGIN {
                    for my in -NODE_MARGIN..=NODE_MARGIN {
                        if placed.contains(&(cx + mx, cy + my)) {
                            fits = false;
                            break 'check;
                        }
                    }
                }
            }
            if fits {
                for &(dx, dy) in template.cells {
                    placed.insert((ox + dx, oy + dy));
                }
                nodes.push(NodeSpawn {
                    origin: IVec2::new(ox, oy),
                    template_idx: tmpl_idx,
                    resource: *resource,
                });
                region_count += 1;
            }
        }
    }

    // 8-10. Gradients + cliffs
    let shore_dist = compute_shore_distance(&all_water_so_far);
    let water_depth = compute_water_depth(&all_water_so_far);
    let cliffs = detect_cliffs(&heights);

    // 11. Render heights (smooth or stepped)
    let render_heights = if TERRAIN_SMOOTH {
        let mut raw: BTreeMap<(i32, i32), f32> = BTreeMap::new();
        for gx in 0..ww {
            for gz in 0..wh {
                raw.insert((gx, gz), terrain_height(gx, gz).clamp(-2.0, 3.0));
            }
        }
        let mut smoothed = raw.clone();
        for gx in 0..ww {
            for gz in 0..wh {
                if ocean.contains(&(gx, gz)) {
                    continue;
                }
                let mut sum = raw[&(gx, gz)];
                let mut cnt = 1.0_f32;
                for (nx, nz) in [(gx + 1, gz), (gx - 1, gz), (gx, gz + 1), (gx, gz - 1)] {
                    if nx >= 0 && nx < ww && nz >= 0 && nz < wh && !ocean.contains(&(nx, nz)) {
                        sum += raw[&(nx, nz)];
                        cnt += 1.0;
                    }
                }
                smoothed.insert((gx, gz), sum / cnt);
            }
        }
        smoothed
    } else {
        heights.clone()
    };

    LayoutResult {
        heights,
        render_heights,
        ocean,
        river_cells,
        lake_cells,
        nodes,
        shore_dist,
        water_depth,
        cliffs,
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toon: ResMut<Assets<ToonMaterial>>,
) {
    let ww = WORLD_W as i32;
    let wh = WORLD_H as i32;
    let world_x = WORLD_W as f32 * CELL_SIZE;
    let world_z = WORLD_H as f32 * CELL_SIZE;

    let layout = generate_layout();

    let avg_h: f32 = if layout.heights.is_empty() {
        0.0
    } else {
        layout.heights.values().sum::<f32>() / layout.heights.len() as f32
    };
    let centre = Vec3::new(world_x * 0.5, avg_h * CELL_SIZE, world_z * 0.5);
    let diag = (world_x * world_x + world_z * world_z).sqrt();

    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: diag * 1.2,
            },
            near: -500.0,
            far: 500.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(world_x * 1.1, diag * 0.5, world_z * 1.1)
            .looking_at(centre, Vec3::Y),
        Msaa::Off,
        MainPassResolutionOverride(UVec2::new(640, 360)),
        DepthPrepass,
        NormalPrepass,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(centre.x + 40.0, 100.0, centre.z + 30.0).looking_at(centre, Vec3::Y),
    ));

    let grass_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.34, 0.62, 0.28) });
    let shore3_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.32, 0.59, 0.27) });
    let shore2_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.30, 0.56, 0.25) });
    let shore1_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.28, 0.52, 0.24) });
    let cliff_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.45, 0.38, 0.30) });
    let shallow_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.35, 0.68, 0.90) });
    let medium_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.28, 0.60, 0.85) });
    let deep_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.22, 0.52, 0.78) });
    let ocean_mat = toon.add(ToonMaterial { base_color: LinearRgba::rgb(0.12, 0.38, 0.62) });
    let copper_mat = toon.add(ToonMaterial { base_color: Resource::Copper.color() });
    let metal_mat = toon.add(ToonMaterial { base_color: Resource::Metal.color() });
    let coal_mat = toon.add(ToonMaterial { base_color: Resource::Coal.color() });

    let ocean_mesh = meshes.add(Cuboid::new(world_x * 3.0, 0.3 * CELL_SIZE, world_z * 3.0));
    commands.spawn((
        Mesh3d(ocean_mesh),
        MeshMaterial3d(ocean_mat.clone()),
        Transform::from_xyz(world_x * 0.5, SEA_LEVEL * CELL_SIZE, world_z * 0.5),
    ));

    let unit_col = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for gx in 0..ww {
        for gz in 0..wh {
            if layout.ocean.contains(&(gx, gz)) {
                continue;
            }
            let wx = (gx as f32 + 0.5) * CELL_SIZE;
            let wz = (gz as f32 + 0.5) * CELL_SIZE;
            let is_river = layout.river_cells.contains(&(gx, gz));
            let is_lake = layout.lake_cells.contains(&(gx, gz));
            let h = layout.render_heights[&(gx, gz)];
            let col_h = (h - DEPTH_FLOOR) * CELL_SIZE;
            let surface_y = h * CELL_SIZE;
            let center_y = surface_y - col_h / 2.0;

            let mat = if is_river || is_lake {
                let depth = layout.water_depth.get(&(gx, gz)).copied().unwrap_or(1);
                match depth {
                    1 => shallow_mat.clone(),
                    2 => medium_mat.clone(),
                    _ => deep_mat.clone(),
                }
            } else if layout.cliffs.contains(&(gx, gz)) {
                cliff_mat.clone()
            } else {
                let sd = layout.shore_dist.get(&(gx, gz)).copied().unwrap_or(255);
                match sd {
                    1 => shore1_mat.clone(),
                    2 => shore2_mat.clone(),
                    3 => shore3_mat.clone(),
                    _ => grass_mat.clone(),
                }
            };

            commands.spawn((
                Mesh3d(unit_col.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(wx, center_y, wz)
                    .with_scale(Vec3::new(CELL_SIZE, col_h, CELL_SIZE)),
            ));
        }
    }

    for spawn in &layout.nodes {
        let material = match spawn.resource {
            Resource::Copper => copper_mat.clone(),
            Resource::Metal => metal_mat.clone(),
            Resource::Coal => coal_mat.clone(),
        };
        spawn_node(
            &mut commands,
            &mut meshes,
            material,
            spawn.origin,
            &TEMPLATES[spawn.template_idx],
            spawn.resource.primitive(),
            &layout.render_heights,
        );
    }
}

// --- Outline post-process ---

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct OutlinePostProcessLabel;

impl RenderLabel for OutlinePostProcessLabel {
    fn dyn_clone(&self) -> Box<dyn RenderLabel> {
        Box::new(self.clone())
    }
}

struct OutlinePostProcessPlugin;

impl Plugin for OutlinePostProcessPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(RenderStartup, init_outline_pipeline);

        use bevy::render::render_graph::{RenderGraph, RenderGraphExt};
        render_app.add_render_graph_node::<ViewNodeRunner<OutlineNode>>(
            Core3d,
            OutlinePostProcessLabel,
        );
        if let Some(graph) = render_app
            .world_mut()
            .get_resource_mut::<RenderGraph>()
        {
            if let Some(sub) = graph.into_inner().get_sub_graph_mut(Core3d) {
                let _ = sub.try_add_node_edge(Node3d::Tonemapping, OutlinePostProcessLabel);
                let _ = sub.try_add_node_edge(
                    OutlinePostProcessLabel,
                    Node3d::EndMainPassPostProcessing,
                );
            }
        }
    }
}

#[derive(Resource)]
struct OutlinePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    pipeline_id_hdr: CachedRenderPipelineId,
}

fn init_outline_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "outline_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                texture_depth_2d(),
                texture_2d(TextureSampleType::Float { filterable: false }),
                sampler(SamplerBindingType::NonFiltering),
            ),
        ),
    );
    let samp = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load("shaders/outline_post_process.wgsl");
    let vertex = fullscreen_shader.to_vertex_state();
    let mut desc = RenderPipelineDescriptor {
        label: Some("outline_pipeline".into()),
        layout: vec![layout.clone()],
        vertex,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..bevy_default()
        }),
        ..bevy_default()
    };
    let pipeline_id = pipeline_cache.queue_render_pipeline(desc.clone());
    desc.fragment.as_mut().unwrap().targets[0]
        .as_mut()
        .unwrap()
        .format = ViewTarget::TEXTURE_FORMAT_HDR;
    let pipeline_id_hdr = pipeline_cache.queue_render_pipeline(desc);
    commands.insert_resource(OutlinePipeline {
        layout,
        sampler: samp,
        pipeline_id,
        pipeline_id_hdr,
    });
}

#[derive(Default)]
struct OutlineNode;

impl ViewNode for OutlineNode {
    type ViewQuery = (&'static ViewTarget, &'static ViewPrepassTextures);

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, prepass_textures): bevy::ecs::query::QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let pipeline_res = world.resource::<OutlinePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = if view_target.is_hdr() {
            pipeline_res.pipeline_id_hdr
        } else {
            pipeline_res.pipeline_id
        };
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
            return Ok(());
        };
        let Some(depth_view) = prepass_textures.depth_view() else {
            return Ok(());
        };
        let Some(normal_view) = prepass_textures.normal_view() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "outline_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline_res.layout),
            &BindGroupEntries::sequential((
                post_process.source,
                depth_view,
                normal_view,
                &pipeline_res.sampler,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("outline_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

fn screenshot_then_exit(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut taken: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 30 && !*taken {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("/tmp/claude-bevy-grid-prototype.png"));
        *taken = true;
    }
    if *frame >= 60 {
        exit.write(AppExit::Success);
    }
}
