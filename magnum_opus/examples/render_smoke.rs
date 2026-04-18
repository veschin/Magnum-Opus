//! F18 render-pipeline / AC4 manual validation + Claude-dev screenshot harness.
//!
//! Run interactively:   `cargo run --example render_smoke`
//! Run for screenshot:  `SCREENSHOT=1 cargo run --example render_smoke`
//!
//! When `SCREENSHOT=1` is set, the example spawns a hidden window (title prefix
//! `claude-dev-` routes it to the `claude` special workspace via Hyprland rule),
//! captures a PNG at frame 30, and exits at frame 60. This lets Claude inspect
//! the rendered output without stealing user focus.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowPlugin;
use magnum_opus::core::{AppExt, CorePlugin};
use magnum_opus::render_pipeline::{RenderPipelineConfigModule, RenderPipelinePlugin};

const SCREENSHOT_PATH: &str = "/tmp/claude-bevy-render_smoke.png";

fn main() {
    let screenshot_mode = std::env::var("SCREENSHOT").is_ok();
    let window_title = if screenshot_mode {
        "claude-dev-render_smoke".to_string()
    } else {
        "render_smoke".to_string()
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
    app.add_data::<RenderPipelineConfigModule>();
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
