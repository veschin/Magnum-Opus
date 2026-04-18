use bevy::MinimalPlugins;
use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct R;

struct M;
impl SimDomain for M {
    const ID: &'static str = "m";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![R],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<R>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "finalize_modules() - cross-module invariants were never checked")]
fn first_tick_without_finalize_modules_panics() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CorePlugin);
    app.add_sim::<M>();
    // Deliberately skip app.finalize_modules() - first update must panic.
    app.update();
}
