use bevy::MinimalPlugins;
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use super::app_ext::AppExt;
use super::input_ui::InputUI;
use super::plugin::CorePlugin;
use super::sim_domain::SimDomain;
use super::static_data::StaticData;
use super::view::View;

/// Canonical test harness. All module tests go through this.
///
/// ```ignore
/// let mut app = Harness::new()
///     .with_sim::<MyDomain>()
///     .with_input::<MyInput>()
///     .build();
/// app.update();
/// ```
pub struct Harness {
    app: App,
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

impl Harness {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // View-archetype modules spawn `Mesh3d` + `MeshMaterial3d<StandardMaterial>`.
        // Their deferred commands request `Assets<Mesh>` / `Assets<StandardMaterial>`
        // at apply-time; the headless harness must register both asset types so
        // the tests don't panic with "resource does not exist".
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<Image>();
        app.add_plugins(CorePlugin);
        Self { app }
    }

    pub fn with_sim<M: SimDomain>(mut self) -> Self {
        self.app.add_sim::<M>();
        self
    }

    pub fn with_data<M: StaticData>(mut self) -> Self {
        self.app.add_data::<M>();
        self
    }

    pub fn with_view<M: View>(mut self) -> Self {
        self.app.add_view::<M>();
        self
    }

    pub fn with_input<M: InputUI>(mut self) -> Self {
        self.app.add_input::<M>();
        self
    }

    pub fn build(mut self) -> App {
        self.app.finalize_modules();
        self.app
    }
}
