//! F19 world-render / AC7 and F22 render-outline / AC5 manual validation.
//!
//! Shows 64x64 terrain tiles rendered into F18's low-res target. Run with
//! `SCREENSHOT=1 cargo run --example world_render_smoke` to capture a PNG
//! to /tmp/claude-bevy-world_render_smoke.png without stealing focus.
//!
//! Set `OUTLINE=1` to enable the Sobel outline shader. The PNG path switches
//! to /tmp/claude-bevy-world_render_smoke_outline.png so the two variants can
//! be inspected side-by-side.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowPlugin;
use magnum_opus::core::{AppExt, CorePlugin};
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::render_pipeline::{
    RenderPipelineConfig, RenderPipelineConfigModule, RenderPipelinePlugin,
};
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

const SCREENSHOT_PATH_PLAIN: &str = "/tmp/claude-bevy-world_render_smoke.png";
const SCREENSHOT_PATH_OUTLINE: &str = "/tmp/claude-bevy-world_render_smoke_outline.png";

#[derive(Resource)]
struct ScreenshotPath(&'static str);

fn main() {
    let screenshot_mode = std::env::var("SCREENSHOT").is_ok();
    let outline_mode = std::env::var("OUTLINE").is_ok();

    let window_title = match (screenshot_mode, outline_mode) {
        (true, true) => "claude-dev-world_render_smoke_outline".to_string(),
        (true, false) => "claude-dev-world_render_smoke".to_string(),
        (false, true) => "world_render_smoke (outline)".to_string(),
        (false, false) => "world_render_smoke".to_string(),
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

    if outline_mode {
        app.insert_resource(RenderPipelineConfig {
            low_res_width: 480,
            low_res_height: 270,
            outline_enabled: true,
            toon_bands: 0,
            posterize_levels: 0,
        });
    }

    app.add_plugins(RenderPipelinePlugin);

    if screenshot_mode {
        let path = if outline_mode {
            SCREENSHOT_PATH_OUTLINE
        } else {
            SCREENSHOT_PATH_PLAIN
        };
        app.insert_resource(ScreenshotPath(path));
        app.add_systems(Update, capture_and_exit);
    }

    app.run();
}

fn capture_and_exit(
    mut commands: Commands,
    mut frame: Local<u32>,
    path: Res<ScreenshotPath>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 30 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0));
    }
    if *frame >= 60 {
        exit.write(AppExit::Success);
    }
}
