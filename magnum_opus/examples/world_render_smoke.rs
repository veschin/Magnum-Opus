//! Visual smoke test: terrain, veins, and placed Buildings on the low-res
//! pixel-art target. `SCREENSHOT=1` captures a PNG; `OUTLINE=1` switches on
//! the Sobel outline shader and writes to the outline-tagged path.

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
    app.add_systems(Startup, seed_demo_placements);

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

/// Drop a handful of buildings onto the map so the screenshot shows the full
/// stack (terrain + veins + groups + sprites).
fn seed_demo_placements(mut bus: ResMut<CommandBus<PlaceTile>>) {
    // A small Miner cluster.
    let miner_cluster = [(12, 16), (12, 17), (13, 16), (13, 17)];
    for (x, y) in miner_cluster {
        bus.push(PlaceTile {
            x,
            y,
            building_type: Some(BuildingType::Miner),
        });
    }

    // Smelter row next to the miners.
    let smelter_row = [(14, 16), (14, 17), (15, 16)];
    for (x, y) in smelter_row {
        bus.push(PlaceTile {
            x,
            y,
            building_type: Some(BuildingType::Smelter),
        });
    }

    // Mall + EnergySource singleton elsewhere.
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
