//! Bevy plugin that installs the pixel-art render pipeline.
//!
//! Topology:
//! 1. Low-res off-screen `Image` target sized from `RenderPipelineConfig`.
//! 2. `Camera3d` (orthographic isometric, `RenderLayers::layer(1)`) rendering
//!    every scene mesh into the off-screen image.
//! 3. `Camera2d` (window, `RenderLayers::layer(2)`) rendering a fullscreen
//!    quad with `MeshMaterial2d<PostProcessMaterial>` that samples the
//!    off-screen image with nearest-neighbour, runs the Sobel outline +
//!    posterize pass, and upscales to the window size.
//!
//! Scene lighting is baked into `ToonMaterial` uniforms (`sun_direction`,
//! `ambient_color` pulled from `RenderPipelineConfig` at material creation
//! time). Bevy's `DirectionalLight` / `AmbientLight` are NOT used - the toon
//! material shades flat + banded against a single uniform sun direction and
//! ambient floor, which gives sharper cel-shaded silhouettes than PBR.

use super::post_process::{PostProcessMaterial, PostProcessParams};
use super::resource::RenderPipelineConfig;
use super::toon::ToonMaterial;
use bevy::asset::{AssetApp, embedded_asset};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{OrthographicProjection, Projection, RenderTarget, ScalingMode};
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite_render::{Material2dPlugin, MeshMaterial2d};

/// `RenderLayers` bit reserved for scene meshes that the 3D iso camera renders
/// into the low-res target. Terrain, veins, buildings must all be spawned on
/// this layer to become visible in the pipeline.
pub const SCENE_LAYER: usize = 1;
/// `RenderLayers` bit reserved for the window-space blit quad.
pub const WINDOW_LAYER: usize = 2;

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "toon.wgsl");
        embedded_asset!(app, "post_process.wgsl");

        app.init_asset::<ToonMaterial>();
        app.init_asset::<PostProcessMaterial>();
        app.add_plugins(MaterialPlugin::<ToonMaterial>::default());
        app.add_plugins(Material2dPlugin::<PostProcessMaterial>::default());
        app.insert_resource(ClearColor(Color::srgb(0.60, 0.78, 0.92)));
        app.add_systems(Startup, setup_pipeline);
        app.add_systems(Update, fit_blit_quad_to_window);
    }
}

#[derive(Component)]
struct BlitQuad;

fn setup_pipeline(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_process_materials: ResMut<Assets<PostProcessMaterial>>,
    cfg: Option<Res<RenderPipelineConfig>>,
) {
    let cfg = cfg.expect(
        "RenderPipelineConfig resource missing \
         - register RenderPipelineConfigModule before adding RenderPipelinePlugin",
    );

    let size = Extent3d {
        width: cfg.low_res_width,
        height: cfg.low_res_height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("render_pipeline.low_res_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler: ImageSampler::Descriptor(ImageSamplerDescriptor::nearest()),
        ..default()
    };
    image.resize(size);
    let handle = images.add(image);

    let scene_layer = RenderLayers::layer(SCENE_LAYER);
    let window_layer = RenderLayers::layer(WINDOW_LAYER);

    // Camera 0: iso 3D scene camera rendering into the low-res off-screen image.
    // Position along (1, 1, 1) + looking_at(ORIGIN) yields true-isometric angles
    // (tilt arctan(1/sqrt(2)) ≈ 35.264°, yaw 45°). Orthographic projection keeps
    // pixel alignment; viewport_height is tuned so the 64×64 grid occupies the
    // middle of the frame with buildings big enough to read individually at
    // the 480×270 low-res resolution.
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.60, 0.78, 0.92)),
            order: 0,
            ..default()
        },
        RenderTarget::Image(handle.clone().into()),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 50.0,
            },
            near: 0.1,
            far: 500.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        scene_layer,
    ));

    // Camera 1: renders the window-layer blit quad to the actual window.
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            order: 1,
            ..default()
        },
        window_layer.clone(),
    ));

    // Fullscreen blit quad. Sized to the low-res resolution; `fit_blit_quad_to_window`
    // rescales each frame to cover the window while preserving pixel-snapped integer
    // scale factors (nearest-neighbour upscale).
    let mesh = meshes.add(Rectangle::new(
        cfg.low_res_width as f32,
        cfg.low_res_height as f32,
    ));
    let material = post_process_materials.add(PostProcessMaterial {
        params: PostProcessParams {
            outline_color: cfg.outline_color,
            outline_threshold: cfg.outline_threshold,
            posterize_levels: cfg.posterize_levels as f32,
            outline_enabled: if cfg.outline_enabled { 1.0 } else { 0.0 },
            _pad: 0.0,
        },
        source: handle,
    });
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        BlitQuad,
        window_layer,
    ));
}

fn fit_blit_quad_to_window(
    windows: Query<&Window>,
    cfg: Res<RenderPipelineConfig>,
    mut quads: Query<&mut Transform, With<BlitQuad>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    // Integer upscale only - nearest-neighbour is pixel-perfect only when the
    // scale factor is a whole number; fractional scales smear low-res pixels
    // across uneven screen-pixel counts.
    let scale_x = window.width() / cfg.low_res_width as f32;
    let scale_y = window.height() / cfg.low_res_height as f32;
    let scale = scale_x.min(scale_y).floor().max(1.0);
    for mut transform in &mut quads {
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
}
