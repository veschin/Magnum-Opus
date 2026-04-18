use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct Grid;
struct PlaceTile;
#[derive(Message)]
struct TilePlaced;
#[derive(Message)]
struct PlaceRejected;

struct GridModule;
impl SimDomain for GridModule {
    const ID: &'static str = "grid_macro_test";
    const PRIMARY_PHASE: Phase = Phase::Placement;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Grid],
            commands_in: names![PlaceTile],
            messages_out: names![TilePlaced, PlaceRejected],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.consume_command::<PlaceTile>();
        ctx.emit_message::<TilePlaced>();
        ctx.emit_message::<PlaceRejected>();
        ctx.add_system(|| {});
    }
}

struct PlaceTileProducer;
impl InputUI for PlaceTileProducer {
    const ID: &'static str = "place_tile_producer";
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
fn names_macro_produces_correct_types() {
    let keys: &[TypeKey] = names![Grid, PlaceTile, TilePlaced];
    assert_eq!(keys.len(), 3);
    assert!(keys[0].is::<Grid>());
    assert!(keys[1].is::<PlaceTile>());
    assert!(keys[2].is::<TilePlaced>());
    assert_eq!(keys[0].name, "Grid");
    assert_eq!(keys[1].name, "PlaceTile");
    assert_eq!(keys[2].name, "TilePlaced");
}

#[test]
fn names_macro_empty_compiles() {
    let empty: &[TypeKey] = names![];
    assert!(empty.is_empty());
}

#[test]
fn contract_built_via_macro_registers_correctly() {
    let app = Harness::new()
        .with_input::<PlaceTileProducer>()
        .with_sim::<GridModule>()
        .build();

    let reg = app.world().resource::<ModuleRegistry>();
    let rec = reg.get("grid_macro_test").unwrap();

    assert_eq!(rec.writes.len(), 1);
    assert!(rec.writes.iter().any(|k| k.is::<Grid>()));

    assert_eq!(rec.commands_in.len(), 1);
    assert!(rec.commands_in.iter().any(|k| k.is::<PlaceTile>()));

    assert_eq!(rec.messages_out.len(), 2);
    assert!(rec.messages_out.iter().any(|k| k.is::<TilePlaced>()));
    assert!(rec.messages_out.iter().any(|k| k.is::<PlaceRejected>()));
}
