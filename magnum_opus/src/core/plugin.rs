use bevy::prelude::*;

use super::metrics::MetricsRegistry;
use super::phase::Phase;
use super::registry::ModuleRegistry;
use super::seal::CoreSeal;
use super::tick::{Tick, tick_increment_system};
use super::type_key::TypeKey;

/// Core plugin. Registers shared resources, configures `Phase` ordering in
/// `Update`, claims core-owned resources under single-writer, installs the
/// tick-increment system in `Phase::End`, and adds a startup-time guard that
/// panics if `finalize_modules()` was never called.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tick>();
        app.init_resource::<ModuleRegistry>();
        app.init_resource::<MetricsRegistry>();
        app.insert_resource(CoreSeal::new());

        {
            let mut reg = app.world_mut().resource_mut::<ModuleRegistry>();
            reg.register_core_writer(TypeKey::new::<Tick>("core::Tick"));
            reg.register_core_writer(TypeKey::new::<ModuleRegistry>("core::ModuleRegistry"));
            reg.register_core_writer(TypeKey::new::<MetricsRegistry>("core::MetricsRegistry"));
            reg.register_core_writer(TypeKey::new::<CoreSeal>("core::CoreSeal"));
        }

        app.configure_sets(
            Update,
            (
                Phase::Commands,
                Phase::World,
                Phase::Placement,
                Phase::Groups,
                Phase::Power,
                Phase::Production,
                Phase::Manifold,
                Phase::Transport,
                Phase::Progression,
                Phase::Metrics,
                Phase::End,
            )
                .chain(),
        );

        app.add_systems(Update, tick_increment_system.in_set(Phase::End));
        app.add_systems(Startup, assert_registry_finalized);
    }
}

/// Panics at first tick if `finalize_modules()` was not called.
/// Reads `CoreSeal` (not `ModuleRegistry`) so that swapping the registry via
/// `world.insert_resource(fake_finalized_registry)` does not bypass the guard.
fn assert_registry_finalized(seal: Res<CoreSeal>) {
    if !seal.is_finalized() {
        panic!(
            "core: app.update() reached without app.finalize_modules() - cross-module invariants were never checked. Call finalize_modules() or use Harness::build()."
        );
    }
}
