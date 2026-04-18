use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct ResA;
#[derive(Resource, Default)]
struct ResB;
#[derive(Resource, Default)]
struct ResC;
struct Cmd1;
#[derive(Message)]
struct Msg1;
#[derive(Message)]
struct Msg2;

struct ResWriter;
impl SimDomain for ResWriter {
    const ID: &'static str = "res_writer";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![ResA, ResB],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ResA>();
        ctx.write_resource::<ResB>();
        ctx.add_system(|| {});
    }
}

struct DeclaresAll;
impl SimDomain for DeclaresAll {
    const ID: &'static str = "drift_mod";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            reads: names![ResA, ResB],
            writes: names![ResC],
            commands_in: names![Cmd1],
            messages_in: names![Msg1],
            messages_out: names![Msg2],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<ResA>();
        ctx.read_resource::<ResB>();
        ctx.write_resource::<ResC>();
        ctx.consume_command::<Cmd1>();
        ctx.read_message::<Msg1>();
        ctx.emit_message::<Msg2>();
        ctx.add_system(|| {});
    }
}

struct Cmd1Producer;
impl InputUI for Cmd1Producer {
    const ID: &'static str = "cmd1_producer";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn commands_out() -> &'static [TypeKey] {
        names![Cmd1]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.emit_command::<Cmd1>();
    }
}

struct Msg1Producer;
impl SimDomain for Msg1Producer {
    const ID: &'static str = "msg1_producer";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            messages_out: names![Msg1],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.emit_message::<Msg1>();
        ctx.add_system(|| {});
    }
}

#[test]
fn registry_records_declared_surface() {
    let app = Harness::new()
        .with_sim::<ResWriter>()
        .with_sim::<DeclaresAll>()
        .with_input::<Cmd1Producer>()
        .with_sim::<Msg1Producer>()
        .build();

    let reg = app.world().resource::<ModuleRegistry>();
    let rec = reg.get("drift_mod").unwrap();
    assert_eq!(rec.archetype, Archetype::Sim);
    assert_eq!(rec.phase, Some(Phase::World));

    assert_eq!(rec.reads.len(), 2);
    assert!(rec.reads.iter().any(|k| k.is::<ResA>()));
    assert!(rec.reads.iter().any(|k| k.is::<ResB>()));

    assert_eq!(rec.writes.len(), 1);
    assert!(rec.writes.iter().any(|k| k.is::<ResC>()));

    assert_eq!(rec.commands_in.len(), 1);
    assert!(rec.commands_in.iter().any(|k| k.is::<Cmd1>()));

    assert_eq!(rec.messages_in.len(), 1);
    assert!(rec.messages_in.iter().any(|k| k.is::<Msg1>()));

    assert_eq!(rec.messages_out.len(), 1);
    assert!(rec.messages_out.iter().any(|k| k.is::<Msg2>()));

    assert_eq!(reg.writer_of_type::<ResC>(), Some("drift_mod"));
    assert_eq!(reg.writer_of_type::<ResA>(), Some("res_writer"));

    let input_rec = reg.get("cmd1_producer").unwrap();
    assert_eq!(input_rec.archetype, Archetype::Input);
    assert_eq!(input_rec.commands_out.len(), 1);
    assert!(input_rec.commands_out.iter().any(|k| k.is::<Cmd1>()));
}
