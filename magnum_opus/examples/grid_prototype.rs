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
use magnum_opus::world_config::{WorldConfig, WorldConfigModule};

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

fn setup_scene(
    mut commands: Commands,
    cfg: Res<WorldConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toon: ResMut<Assets<ToonMaterial>>,
) {
    let w = cfg.width as f32;
    let h = cfg.height as f32;
    let centre = Vec3::new(w * 0.5, 0.0, h * 0.5);

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: h * 1.4,
            },
            near: -500.0,
            far: 500.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(w * 1.2, h * 0.9, h * 1.2).looking_at(centre, Vec3::Y),
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
        Transform::from_xyz(w * 0.5 + 40.0, 100.0, h * 0.5 + 30.0).looking_at(centre, Vec3::Y),
    ));

    let ground = toon.add(ToonMaterial {
        base_color: LinearRgba::rgb(0.34, 0.62, 0.28),
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(w, h))),
        MeshMaterial3d(ground),
        Transform::from_translation(centre),
    ));

    let stone = toon.add(ToonMaterial {
        base_color: LinearRgba::rgb(0.62, 0.60, 0.55),
    });
    for (gx, gy, height) in [(20u32, 24u32, 5.0_f32), (40, 38, 7.0)] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, height, 3.0))),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(gx as f32 + 0.5, height * 0.5, gy as f32 + 0.5),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 2.0, 1.0))),
        MeshMaterial3d(stone.clone()),
        Transform::from_xyz(30.0, 1.0, 30.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.0, 1.5))),
        MeshMaterial3d(stone.clone()),
        Transform::from_xyz(15.0, 0.5, 40.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 10.0, 1.0))),
        MeshMaterial3d(stone.clone()),
        Transform::from_xyz(45.0, 5.0, 20.0),
    ));
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
