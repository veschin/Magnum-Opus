//! Bevy plugin that installs the low-res render target and the upscale pass.
//!
//! This plugin is deliberately NOT a core SimDomain/View module. It reaches into
//! Bevy's render graph via `&mut App` which the core installers do not expose.
//! The configuration resource `RenderPipelineConfig` is the only PTSD-tracked
//! surface; everything the plugin spawns is render-private.
//!
//! MVP (v0): create an off-screen `Image`, render the scene camera into it, and
//! blit it to the window with a nearest-neighbor sprite. No shaders yet.

use super::resource::RenderPipelineConfig;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_low_res_target);
    }
}

fn setup_low_res_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
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

    // Scene layer: everything game-visible lives here.
    let scene_layer = RenderLayers::layer(1);
    // Window layer: only the blit sprite.
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
        scene_layer.clone(),
    ));

    // Camera 1: renders the window-layer blit sprite to the actual window.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        window_layer.clone(),
    ));

    // Sprite showing the low-res image upscaled by nearest-neighbor.
    commands.spawn((Sprite::from_image(handle), window_layer));
}
