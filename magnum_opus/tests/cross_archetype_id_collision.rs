use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct A;
#[derive(Resource, Default)]
struct B;

struct SimMod;
impl SimDomain for SimMod {
    const ID: &'static str = "shared_id";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![A],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<A>();
        ctx.add_system(|| {});
    }
}

struct DataMod;
impl StaticData for DataMod {
    const ID: &'static str = "shared_id";
    fn writes() -> &'static [TypeKey] {
        names![B]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.write_resource::<B>();
    }
}

#[test]
#[should_panic(expected = "duplicate module id")]
fn sim_and_data_same_id_panic() {
    let _ = Harness::new()
        .with_sim::<SimMod>()
        .with_data::<DataMod>()
        .build();
}

#[test]
#[should_panic(expected = "duplicate module id \"core\"")]
fn user_claiming_core_id_panics() {
    struct Impostor;
    impl SimDomain for Impostor {
        const ID: &'static str = "core";
        const PRIMARY_PHASE: Phase = Phase::World;
        fn contract() -> SimContract {
            SimContract::EMPTY
        }
        fn install(ctx: &mut SimInstaller) {
            ctx.add_system(|| {});
        }
    }
    let _ = Harness::new().with_sim::<Impostor>().build();
}
