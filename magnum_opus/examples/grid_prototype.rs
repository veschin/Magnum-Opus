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
use std::collections::BTreeSet;

// --- Tunables ---
const CELLS: u32 = 32;                       // visible grid side (cells)
const CELL_SIZE: f32 = 1.5;                  // world units per cell
const NODES_PER_REGION: usize = 6;           // resource cluster density
const REGION_RADIUS: i32 = 5;                // placement jitter around center
const REGION_OFFSET: f32 = 9.0;              // distance from map center
const NODE_MARGIN: i32 = 1;                  // empty cells between nodes
const SPAWN_SEED: u64 = 0xAABB_CCDD_EEFF_0011;
const MAX_SPAWN_ATTEMPTS: u64 = 400;
const WATER_CLUSTERS: u64 = 10;              // water puddle seeds
const WATER_MAX_CELLS: u32 = 5;              // max cells per puddle
const WATER_SEED: u64 = 0x1234_5678_9ABC_DEF0;
const GROUND_THICKNESS: f32 = 1.0;           // × CELL_SIZE
const WATER_DEPTH: f32 = 0.25;               // × CELL_SIZE below ground top
const WATER_THICKNESS: f32 = 0.35;           // × CELL_SIZE

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

// Three region centres arranged as an equilateral triangle around map centre.
fn region_centers() -> [(IVec2, Resource); 3] {
    let cx = CELLS as f32 * 0.5;
    let cy = CELLS as f32 * 0.5;
    let compute = |i: f32, res: Resource| -> (IVec2, Resource) {
        let angle =
            i * std::f32::consts::TAU / 3.0 + std::f32::consts::FRAC_PI_2;
        let x = (cx + REGION_OFFSET * angle.cos()).round() as i32;
        let y = (cy + REGION_OFFSET * angle.sin()).round() as i32;
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
) {
    let mesh = meshes.add(prim.mesh());
    let y = prim.half_height();
    for &(dx, dy) in template.cells {
        let x = ((origin.x + dx) as f32 + 0.5) * CELL_SIZE;
        let z = ((origin.y + dy) as f32 + 0.5) * CELL_SIZE;
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, y, z),
        ));
    }
}

// Deterministic layout: resource nodes per-region (each resource clusters
// around its triangle-vertex center), then water clusters in remaining cells.
fn generate_layout() -> (Vec<NodeSpawn>, Vec<(i32, i32)>) {
    let mut placed: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut nodes: Vec<NodeSpawn> = Vec::new();

    for (region_idx, (center, resource)) in region_centers().iter().enumerate() {
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
            let ox = (center.x + ox_off).clamp(0, CELLS as i32 - max_dx - 1);
            let oy = (center.y + oy_off).clamp(0, CELLS as i32 - max_dy - 1);

            let mut fits = true;
            'check: for &(dx, dy) in template.cells {
                let cx = ox + dx;
                let cy = oy + dy;
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

    // Water: flood-fill puddles from seed cells, skipping node-occupied cells.
    let mut taken = placed.clone();
    let mut water: Vec<(i32, i32)> = Vec::new();
    for cluster in 0..WATER_CLUSTERS {
        let s = hash64(WATER_SEED.wrapping_add(cluster));
        let sx = (hash64(s ^ 0x11) as i32).rem_euclid(CELLS as i32);
        let sy = (hash64(s ^ 0x22) as i32).rem_euclid(CELLS as i32);
        if taken.contains(&(sx, sy)) {
            continue;
        }
        let target = ((hash64(s ^ 0x33) as u32) % WATER_MAX_CELLS).max(1);
        let mut frontier: Vec<(i32, i32)> = vec![(sx, sy)];
        let mut grown: u32 = 0;
        while let Some(cell) = frontier.pop() {
            if grown >= target {
                break;
            }
            let (x, y) = cell;
            if x < 0 || x >= CELLS as i32 || y < 0 || y >= CELLS as i32 {
                continue;
            }
            if taken.contains(&cell) {
                continue;
            }
            taken.insert(cell);
            water.push(cell);
            grown += 1;
            let bias = hash64(s.wrapping_add(grown as u64));
            let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];
            for (i, n) in neighbors.iter().enumerate() {
                if (bias >> i) & 1 == 1 {
                    frontier.insert(0, *n);
                } else {
                    frontier.push(*n);
                }
            }
        }
    }

    (nodes, water)
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toon: ResMut<Assets<ToonMaterial>>,
) {
    let cells = CELLS as f32;
    let world_extent = cells * CELL_SIZE;
    let centre = Vec3::new(world_extent * 0.5, 0.0, world_extent * 0.5);

    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: world_extent * 1.4,
            },
            near: -500.0,
            far: 500.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(world_extent * 1.2, world_extent * 0.9, world_extent * 1.2)
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

    let ground_mat = toon.add(ToonMaterial {
        base_color: LinearRgba::rgb(0.34, 0.62, 0.28),
    });
    let water_mat = toon.add(ToonMaterial {
        base_color: LinearRgba::rgb(0.22, 0.55, 0.82),
    });
    let copper_mat = toon.add(ToonMaterial {
        base_color: Resource::Copper.color(),
    });
    let metal_mat = toon.add(ToonMaterial {
        base_color: Resource::Metal.color(),
    });
    let coal_mat = toon.add(ToonMaterial {
        base_color: Resource::Coal.color(),
    });

    let (node_spawns, water_cells) = generate_layout();
    let water_set: BTreeSet<(i32, i32)> = water_cells.iter().copied().collect();

    // Cellular ground: one cuboid per cell. Water cells get a thinner,
    // lowered tile so the surface reads as a depression.
    let ground_mesh = meshes.add(Cuboid::new(
        CELL_SIZE,
        GROUND_THICKNESS * CELL_SIZE,
        CELL_SIZE,
    ));
    let water_mesh = meshes.add(Cuboid::new(
        CELL_SIZE,
        WATER_THICKNESS * CELL_SIZE,
        CELL_SIZE,
    ));
    let ground_y = -GROUND_THICKNESS * 0.5 * CELL_SIZE;
    let water_y = (-WATER_DEPTH - WATER_THICKNESS * 0.5) * CELL_SIZE;

    for gx in 0..CELLS as i32 {
        for gz in 0..CELLS as i32 {
            let wx = (gx as f32 + 0.5) * CELL_SIZE;
            let wz = (gz as f32 + 0.5) * CELL_SIZE;
            if water_set.contains(&(gx, gz)) {
                commands.spawn((
                    Mesh3d(water_mesh.clone()),
                    MeshMaterial3d(water_mat.clone()),
                    Transform::from_xyz(wx, water_y, wz),
                ));
            } else {
                commands.spawn((
                    Mesh3d(ground_mesh.clone()),
                    MeshMaterial3d(ground_mat.clone()),
                    Transform::from_xyz(wx, ground_y, wz),
                ));
            }
        }
    }

    for spawn in node_spawns {
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
