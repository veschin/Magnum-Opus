//! F19 world-render / AC7 manual validation.
//!
//! Shows 64x64 terrain tiles rendered into F18's low-res target. Run with
//! `SCREENSHOT=1 cargo run --example world_render_smoke` to capture a PNG
//! to /tmp/claude-bevy-world_render_smoke.png without stealing focus.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowPlugin;
use magnum_opus::core::{AppExt, CorePlugin};
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::render_pipeline::{RenderPipelineConfigModule, RenderPipelinePlugin};
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

const SCREENSHOT_PATH: &str = "/tmp/claude-bevy-world_render_smoke.png";

fn main() {
    let screenshot_mode = std::env::var("SCREENSHOT").is_ok();
    let window_title = if screenshot_mode {
        "claude-dev-world_render_smoke".to_string()
    } else {
        "world_render_smoke".to_string()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: window_title,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(CorePlugin);
    app.add_data::<WorldConfigModule>();
    app.add_data::<RenderPipelineConfigModule>();
    app.add_sim::<LandscapeModule>();
    app.add_sim::<ResourcesModule>();
    app.add_view::<WorldRenderModule>();
    app.finalize_modules();
    app.add_plugins(RenderPipelinePlugin);

    if screenshot_mode {
        app.add_systems(Update, capture_and_exit);
    }

    app.run();
}

fn capture_and_exit(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 30 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(SCREENSHOT_PATH));
    }
    if *frame >= 60 {
        exit.write(AppExit::Success);
    }
}
