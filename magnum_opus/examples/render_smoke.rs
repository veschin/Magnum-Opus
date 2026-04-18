//! F18 render-pipeline / AC4: manual validation binary.
//!
//! Opens a window with a 480x270 low-res render target upscaled to window size
//! via nearest-neighbor. The window shows a black framebuffer - visible content
//! arrives with F19 (world-render). Run: `cargo run --example render_smoke`.

use bevy::prelude::*;
use magnum_opus::core::{AppExt, CorePlugin};
use magnum_opus::render_pipeline::{RenderPipelineConfigModule, RenderPipelinePlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(CorePlugin);
    app.add_data::<RenderPipelineConfigModule>();
    app.finalize_modules();
    app.add_plugins(RenderPipelinePlugin);
    app.run();
}
