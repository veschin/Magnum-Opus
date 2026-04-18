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
fn replacing_registry_does_not_bypass_seal() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CorePlugin);
    app.add_sim::<M>();

    // Attacker: replace the registry with a freshly finalized empty one.
    // Before round-4-round-2 this suppressed the first-tick panic. Now the
    // finalize decision is tracked in CoreSeal (not in ModuleRegistry), so the
    // swap does not touch the seal.
    let mut fake = ModuleRegistry::default();
    let _ = fake.finalize_checks();
    app.world_mut().insert_resource(fake);

    // First tick must still panic because CoreSeal.is_finalized() == false.
    app.update();
}
