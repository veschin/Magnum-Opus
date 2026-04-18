use magnum_opus::core::*;
use magnum_opus::names;

struct PlaceCmd;

struct ConsumerA;
impl SimDomain for ConsumerA {
    const ID: &'static str = "consumer_a";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            commands_in: names![PlaceCmd],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.consume_command::<PlaceCmd>();
        ctx.add_system(|| {});
    }
}

struct ConsumerB;
impl SimDomain for ConsumerB {
    const ID: &'static str = "consumer_b";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            commands_in: names![PlaceCmd],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.consume_command::<PlaceCmd>();
        ctx.add_system(|| {});
    }
}

struct Producer;
impl InputUI for Producer {
    const ID: &'static str = "producer";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn commands_out() -> &'static [TypeKey] {
        names![PlaceCmd]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.emit_command::<PlaceCmd>();
    }
}

#[test]
#[should_panic(expected = "single-consumer-commands")]
fn two_consumers_same_command_panic() {
    let _ = Harness::new()
        .with_input::<Producer>()
        .with_sim::<ConsumerA>()
        .with_sim::<ConsumerB>()
        .build();
}
