use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Message)]
struct NobodyEmits;
#[derive(Message)]
struct TilePlaced;
struct PlaceTile;

struct Reader;
impl SimDomain for Reader {
    const ID: &'static str = "reader";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            messages_in: names![NobodyEmits],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_message::<NobodyEmits>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "closed-messages")]
fn message_without_producer_panics() {
    let _ = Harness::new().with_sim::<Reader>().build();
}

struct Emitter;
impl SimDomain for Emitter {
    const ID: &'static str = "emitter";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            messages_out: names![TilePlaced],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.emit_message::<TilePlaced>();
        ctx.add_system(|| {});
    }
}

struct ValidReader;
impl SimDomain for ValidReader {
    const ID: &'static str = "valid_reader";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            messages_in: names![TilePlaced],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_message::<TilePlaced>();
        ctx.add_system(|| {});
    }
}

#[test]
fn matched_message_passes() {
    let _ = Harness::new()
        .with_sim::<Emitter>()
        .with_sim::<ValidReader>()
        .build();
}

struct CommandConsumer;
impl SimDomain for CommandConsumer {
    const ID: &'static str = "cmd_consumer";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            commands_in: names![PlaceTile],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.consume_command::<PlaceTile>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "closed-commands")]
fn command_without_producer_panics() {
    let _ = Harness::new().with_sim::<CommandConsumer>().build();
}

struct CommandInput;
impl InputUI for CommandInput {
    const ID: &'static str = "cmd_input";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn commands_out() -> &'static [TypeKey] {
        names![PlaceTile]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.emit_command::<PlaceTile>();
    }
}

#[test]
fn matched_command_passes() {
    let _ = Harness::new()
        .with_sim::<CommandConsumer>()
        .with_input::<CommandInput>()
        .build();
}
