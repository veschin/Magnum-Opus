//! Pixel-art pipeline smoke test: runs the full scene (terrain + veins +
//! placed buildings) through `RenderPipelinePlugin`. The plugin owns the
//! low-res render target, iso orthographic scene camera, DepthPrepass +
//! NormalPrepass, and the fullscreen post-process blit (Sobel outline +
//! posterize + nearest-neighbour upscale). Scene meshes use the shared
//! `ToonMaterial` which bakes a fixed sun direction and ambient floor into
//! every fragment, producing flat banded shading.
//!
//! Interactive: `cargo run --example world_render_smoke`
//! Screenshot:  `SCREENSHOT=1 cargo run --example world_render_smoke`
//!
//! The `SCREENSHOT` env var routes the window into the Hyprland
//! `claude-dev-` special workspace and captures a single PNG at frame 45.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowPlugin;
use magnum_opus::building_render::BuildingRenderModule;
use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, CorePlugin};
use magnum_opus::grid::{GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule};
use magnum_opus::render_pipeline::{RenderPipelineConfigModule, RenderPipelinePlugin};
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

const SCREENSHOT_PATH: &str = "/tmp/claude-bevy-world_render_smoke.png";

#[derive(Resource)]
struct ScreenshotPath(&'static str);

fn main() {
    let screenshot_mode = std::env::var("SCREENSHOT").is_ok();

    let window_title = if screenshot_mode {
        "claude-dev-world_render_smoke".to_string()
    } else {
        "magnum-opus: world_render_smoke".to_string()
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
    app.add_data::<BuildingDbModule>();
    app.add_data::<RecipeDbModule>();
    app.add_data::<RenderPipelineConfigModule>();
    app.add_sim::<GridModule>();
    app.add_sim::<LandscapeModule>();
    app.add_sim::<ResourcesModule>();
    app.add_sim::<GroupFormationModule>();
    app.add_sim::<ProductionModule>();
    app.add_sim::<ManifoldModule>();
    app.add_view::<WorldRenderModule>();
    app.add_view::<BuildingRenderModule>();
    app.add_input::<PlacementInputModule>();
    app.finalize_modules();
    app.add_plugins(RenderPipelinePlugin);

    app.add_systems(Startup, seed_demo_placements);

    if screenshot_mode {
        app.insert_resource(ScreenshotPath(SCREENSHOT_PATH));
        app.add_systems(Update, capture_and_exit);
    }

    app.run();
}

/// Drop a handful of buildings onto the map so the screenshot shows the full
/// stack (terrain + veins + groups + meshes).
fn seed_demo_placements(mut bus: ResMut<CommandBus<PlaceTile>>) {
    let miner_cluster = [(12, 16), (12, 17), (13, 16), (13, 17)];
    for (x, y) in miner_cluster {
        bus.push(PlaceTile {
            x,
            y,
            building_type: Some(BuildingType::Miner),
        });
    }

    let smelter_row = [(14, 16), (14, 17), (15, 16)];
    for (x, y) in smelter_row {
        bus.push(PlaceTile {
            x,
            y,
            building_type: Some(BuildingType::Smelter),
        });
    }

    bus.push(PlaceTile {
        x: 30,
        y: 40,
        building_type: Some(BuildingType::Mall),
    });
    bus.push(PlaceTile {
        x: 40,
        y: 40,
        building_type: Some(BuildingType::EnergySource),
    });
    bus.push(PlaceTile {
        x: 40,
        y: 41,
        building_type: Some(BuildingType::EnergySource),
    });
}

fn capture_and_exit(
    mut commands: Commands,
    mut frame: Local<u32>,
    path: Res<ScreenshotPath>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    if *frame == 45 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0));
    }
    if *frame >= 75 {
        exit.write(AppExit::Success);
    }
}
