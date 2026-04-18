//! Bevy plugin that installs the low-res render target and the upscale pass.
//!
//! This plugin is deliberately NOT a core SimDomain/View module. It reaches into
//! Bevy's render graph via `&mut App` which the core installers do not expose.
//! The configuration resource `RenderPipelineConfig` is the only PTSD-tracked
//! surface; everything the plugin spawns is render-private.
//!
//! v0: low-res target + nearest-neighbor upscale blit via `Sprite::from_image`.
//! v1 (F22): when `outline_enabled = true`, the blit switches to a `Mesh2d` +
//! `MeshMaterial2d<OutlineMaterial>` pair so a Sobel shader runs during blit.

use super::outline::{OutlineMaterial, OutlineParams};
use super::resource::RenderPipelineConfig;
use bevy::asset::embedded_asset;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite_render::{Material2dPlugin, MeshMaterial2d};

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "outline.wgsl");
        app.add_plugins(Material2dPlugin::<OutlineMaterial>::default());
        app.insert_resource(ClearColor(Color::BLACK));
        app.add_systems(Startup, setup_low_res_target);
        app.add_systems(Update, fit_blit_sprite_to_window);
    }
}

#[derive(Component)]
struct BlitSprite;

fn setup_low_res_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut outline_materials: ResMut<Assets<OutlineMaterial>>,
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

    let scene_layer = RenderLayers::layer(1);
    let window_layer = RenderLayers::layer(2);

    // Camera 0: renders scene layer into the off-screen low-res image.
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            order: 0,
            ..default()
        },
        RenderTarget::Image(handle.clone().into()),
        scene_layer,
    ));

    // Camera 1: renders the window-layer blit entity to the actual window.
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            order: 1,
            ..default()
        },
        window_layer.clone(),
    ));

    // Blit entity: plain sprite when outline disabled, Material2d quad when enabled.
    if cfg.outline_enabled {
        let mesh = meshes.add(Rectangle::new(
            cfg.low_res_width as f32,
            cfg.low_res_height as f32,
        ));
        let material = outline_materials.add(OutlineMaterial {
            params: OutlineParams::default(),
            source: handle,
        });
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::default(),
            BlitSprite,
            window_layer,
        ));
    } else {
        commands.spawn((
            Sprite::from_image(handle),
            Transform::default(),
            BlitSprite,
            window_layer,
        ));
    }
}

fn fit_blit_sprite_to_window(
    windows: Query<&Window>,
    cfg: Res<RenderPipelineConfig>,
    mut sprites: Query<&mut Transform, With<BlitSprite>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale_x = window.width() / cfg.low_res_width as f32;
    let scale_y = window.height() / cfg.low_res_height as f32;
    let scale = scale_x.min(scale_y).max(1.0);
    for mut transform in &mut sprites {
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
}
