use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct Grid;
#[derive(Resource, Default)]
struct RecipeDB;
#[derive(Resource, Default)]
struct SharedThing;
#[derive(Resource, Default)]
struct SceneCache;
#[derive(Resource, Default)]
struct CursorGridPos;

struct SimA;
impl SimDomain for SimA {
    const ID: &'static str = "sim_a";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Grid],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.add_system(|| {});
    }
}

struct SimB;
impl SimDomain for SimB {
    const ID: &'static str = "sim_b";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Grid],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn two_sim_writers_same_resource_panic() {
    let _ = Harness::new().with_sim::<SimA>().with_sim::<SimB>().build();
}

struct Data1;
impl StaticData for Data1 {
    const ID: &'static str = "data_1";
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

struct Data2;
impl StaticData for Data2 {
    const ID: &'static str = "data_2";
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

#[test]
#[should_panic(expected = "single-writer violation")]
fn two_data_writers_same_resource_panic() {
    let _ = Harness::new()
        .with_data::<Data1>()
        .with_data::<Data2>()
        .build();
}

struct SimWriter;
impl SimDomain for SimWriter {
    const ID: &'static str = "sim_w";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![SharedThing],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<SharedThing>();
        ctx.add_system(|| {});
    }
}

struct DataWriter;
impl StaticData for DataWriter {
    const ID: &'static str = "data_w";
    fn writes() -> &'static [TypeKey] {
        names![SharedThing]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.write_resource::<SharedThing>();
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn sim_and_data_cannot_both_write_same_resource() {
    let _ = Harness::new()
        .with_sim::<SimWriter>()
        .with_data::<DataWriter>()
        .build();
}

struct ViewWriter;
impl View for ViewWriter {
    const ID: &'static str = "view_w";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![SceneCache]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<SceneCache>();
    }
}

struct ViewWriter2;
impl View for ViewWriter2 {
    const ID: &'static str = "view_w2";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![SceneCache]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<SceneCache>();
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn two_view_writers_same_resource_panic() {
    let _ = Harness::new()
        .with_view::<ViewWriter>()
        .with_view::<ViewWriter2>()
        .build();
}

struct InputWriter;
impl InputUI for InputWriter {
    const ID: &'static str = "input_w";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![CursorGridPos]
    }
    fn commands_out() -> &'static [TypeKey] {
        &[]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.write_resource::<CursorGridPos>();
    }
}

struct InputWriter2;
impl InputUI for InputWriter2 {
    const ID: &'static str = "input_w2";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![CursorGridPos]
    }
    fn commands_out() -> &'static [TypeKey] {
        &[]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut InputInstaller) {
        ctx.write_resource::<CursorGridPos>();
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn two_input_writers_same_resource_panic() {
    let _ = Harness::new()
        .with_input::<InputWriter>()
        .with_input::<InputWriter2>()
        .build();
}
