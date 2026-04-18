use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct UnwrittenResource;
#[derive(Resource, Default)]
struct NobodyWrites;
#[derive(Resource, Default)]
struct Ghost;
#[derive(Resource, Default)]
struct Resource1;
#[derive(Resource, Default)]
struct RecipeDB;

struct SimReader;
impl SimDomain for SimReader {
    const ID: &'static str = "sim_reader";
    const PRIMARY_PHASE: Phase = Phase::Production;
    fn contract() -> SimContract {
        SimContract {
            reads: names![UnwrittenResource],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<UnwrittenResource>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "closed-reads")]
fn sim_reading_unwritten_resource_panics() {
    let _ = Harness::new().with_sim::<SimReader>().build();
}

struct ViewReader;
impl View for ViewReader {
    const ID: &'static str = "view_reader";
    fn reads() -> &'static [TypeKey] {
        names![NobodyWrites]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.read_resource::<NobodyWrites>();
    }
}

#[test]
#[should_panic(expected = "closed-reads")]
fn view_reading_unwritten_resource_panics() {
    let _ = Harness::new().with_view::<ViewReader>().build();
}

struct InputReader;
impl InputUI for InputReader {
    const ID: &'static str = "input_reader";
    fn reads() -> &'static [TypeKey] {
        names![Ghost]
    }
    fn writes() -> &'static [TypeKey] {
        &[]
    }
    fn commands_out() -> &'static [TypeKey] {
        &[]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.read_resource::<Ghost>();
    }
}

#[test]
#[should_panic(expected = "closed-reads")]
fn input_reading_unwritten_resource_panics() {
    let _ = Harness::new().with_input::<InputReader>().build();
}

struct Writer;
impl SimDomain for Writer {
    const ID: &'static str = "writer";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Resource1],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Resource1>();
        ctx.add_system(|| {});
    }
}

struct Reader;
impl SimDomain for Reader {
    const ID: &'static str = "reader";
    const PRIMARY_PHASE: Phase = Phase::Production;
    fn contract() -> SimContract {
        SimContract {
            reads: names![Resource1],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<Resource1>();
        ctx.add_system(|| {});
    }
}

#[test]
fn matched_write_and_read_passes() {
    let _ = Harness::new()
        .with_sim::<Writer>()
        .with_sim::<Reader>()
        .build();
}

struct DataProvider;
impl StaticData for DataProvider {
    const ID: &'static str = "data_provider";
    fn writes() -> &'static [TypeKey] {
        names![RecipeDB]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.write_resource::<RecipeDB>();
    }
}

struct DataConsumer;
impl SimDomain for DataConsumer {
    const ID: &'static str = "data_consumer";
    const PRIMARY_PHASE: Phase = Phase::Production;
    fn contract() -> SimContract {
        SimContract {
            reads: names![RecipeDB],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<RecipeDB>();
        ctx.add_system(|| {});
    }
}

#[test]
fn sim_reads_data_written_by_static_data() {
    let _ = Harness::new()
        .with_data::<DataProvider>()
        .with_sim::<DataConsumer>()
        .build();
}
