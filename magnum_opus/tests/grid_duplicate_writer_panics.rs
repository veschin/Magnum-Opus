//! F1 world-foundation / AC5: two modules claiming writes: names![Grid] panic single-writer.

use magnum_opus::core::*;
use magnum_opus::grid::{Grid, GridModule};
use magnum_opus::names;
use magnum_opus::world_config::WorldConfigModule;

struct RogueGridWriter;
impl StaticData for RogueGridWriter {
    const ID: &'static str = "rogue_grid_writer";
    fn writes() -> &'static [TypeKey] {
        names![Grid]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(Grid::default());
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_module_claiming_grid_writes_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_data::<RogueGridWriter>()
        .build();
}
