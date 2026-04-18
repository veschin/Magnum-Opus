use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct ResX;
#[derive(Resource, Default)]
struct DataX;
#[derive(Resource, Default)]
struct SceneCacheX;
#[derive(Resource, Default)]
struct CursorGridPos;
struct CmdX;
#[derive(Message)]
struct MsgX;

struct MySim;
impl SimDomain for MySim {
    const ID: &'static str = "sim_x";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![ResX],
            commands_in: names![CmdX],
            messages_out: names![MsgX],
            metrics: &[MetricDesc {
                name: "sim_x.events",
                kind: MetricKind::Counter,
            }],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ResX>();
        ctx.consume_command::<CmdX>();
        ctx.emit_message::<MsgX>();
        ctx.add_system(|| {});
    }
}

struct MyData;
impl StaticData for MyData {
    const ID: &'static str = "data_x";
    fn writes() -> &'static [TypeKey] {
        names![DataX]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[MetricDesc {
            name: "data_x.rows",
            kind: MetricKind::Gauge,
        }]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.write_resource::<DataX>();
    }
}

struct MyView;
impl View for MyView {
    const ID: &'static str = "view_x";
    fn reads() -> &'static [TypeKey] {
        names![ResX, DataX]
    }
    fn writes() -> &'static [TypeKey] {
        names![SceneCacheX]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.read_resource::<ResX>();
        ctx.read_resource::<DataX>();
        ctx.write_resource::<SceneCacheX>();
    }
}

struct MyInput;
impl InputUI for MyInput {
    const ID: &'static str = "input_x";
    fn reads() -> &'static [TypeKey] {
        names![ResX]
    }
    fn writes() -> &'static [TypeKey] {
        names![CursorGridPos]
    }
    fn commands_out() -> &'static [TypeKey] {
        names![CmdX]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.read_resource::<ResX>();
        ctx.write_resource::<CursorGridPos>();
        ctx.emit_command::<CmdX>();
    }
}

#[test]
fn all_four_archetypes_coexist() {
    let app = Harness::new()
        .with_sim::<MySim>()
        .with_data::<MyData>()
        .with_view::<MyView>()
        .with_input::<MyInput>()
        .build();

    let reg = app.world().resource::<ModuleRegistry>();
    assert_eq!(reg.len(), 5);
    assert_eq!(reg.get("sim_x").unwrap().archetype, Archetype::Sim);
    assert_eq!(reg.get("data_x").unwrap().archetype, Archetype::Data);
    assert_eq!(reg.get("view_x").unwrap().archetype, Archetype::View);
    assert_eq!(reg.get("input_x").unwrap().archetype, Archetype::Input);

    assert_eq!(reg.writer_of_type::<ResX>(), Some("sim_x"));
    assert_eq!(reg.writer_of_type::<DataX>(), Some("data_x"));
    assert_eq!(reg.writer_of_type::<SceneCacheX>(), Some("view_x"));
    assert_eq!(reg.writer_of_type::<CursorGridPos>(), Some("input_x"));

    let mreg = app.world().resource::<MetricsRegistry>();
    assert_eq!(mreg.len(), 2);
    assert!(mreg.get("sim_x.events").is_some());
    assert!(mreg.get("data_x.rows").is_some());
    assert_eq!(mreg.owner("data_x.rows"), Some("data_x"));
}

#[test]
fn duplicate_module_id_panics() {
    struct A;
    impl SimDomain for A {
        const ID: &'static str = "same_id";
        const PRIMARY_PHASE: Phase = Phase::World;
        fn contract() -> SimContract {
            SimContract::EMPTY
        }
        fn install(_ctx: &mut SimInstaller) {}
    }

    struct B;
    impl SimDomain for B {
        const ID: &'static str = "same_id";
        const PRIMARY_PHASE: Phase = Phase::World;
        fn contract() -> SimContract {
            SimContract::EMPTY
        }
        fn install(_ctx: &mut SimInstaller) {}
    }

    let result = std::panic::catch_unwind(|| {
        let _ = Harness::new().with_sim::<A>().with_sim::<B>().build();
    });
    assert!(result.is_err());
}
