use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct Forgotten;

struct Lazy;
impl SimDomain for Lazy {
    const ID: &'static str = "lazy";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Forgotten],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "install never performed the matching installer call")]
fn forgotten_write_panics() {
    let _ = Harness::new().with_sim::<Lazy>().build();
}

#[derive(Message)]
struct ForgottenMsg;

struct LazyEmitter;
impl SimDomain for LazyEmitter {
    const ID: &'static str = "lazy_emitter";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            messages_out: names![ForgottenMsg],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "install never performed the matching installer call")]
fn forgotten_messages_out_panics() {
    let _ = Harness::new().with_sim::<LazyEmitter>().build();
}

struct ForgottenCmd;

struct LazyConsumer;
impl SimDomain for LazyConsumer {
    const ID: &'static str = "lazy_consumer";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            commands_in: names![ForgottenCmd],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

struct ForgottenCmdProducer;
impl InputUI for ForgottenCmdProducer {
    const ID: &'static str = "forgotten_cmd_producer";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn commands_out() -> &'static [TypeKey] {
        names![ForgottenCmd]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.emit_command::<ForgottenCmd>();
    }
}

#[test]
#[should_panic(expected = "install never performed the matching installer call")]
fn forgotten_commands_in_panics() {
    let _ = Harness::new()
        .with_input::<ForgottenCmdProducer>()
        .with_sim::<LazyConsumer>()
        .build();
}
