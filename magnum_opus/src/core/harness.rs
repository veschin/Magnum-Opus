use bevy::MinimalPlugins;
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
