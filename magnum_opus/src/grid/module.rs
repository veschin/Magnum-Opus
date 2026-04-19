use super::resource::Grid;
use super::systems::{grid_bootstrap_system, grid_metrics_system};
use crate::core::*;
use crate::names;
use crate::world_config::WorldConfig;

pub struct GridModule;

impl SimDomain for GridModule {
    const ID: &'static str = "grid";
    const PRIMARY_PHASE: Phase = Phase::World;

    fn contract() -> SimContract {
        SimContract {
            writes: names![Grid],
            reads: names![WorldConfig],
            metrics: &[MetricDesc {
                name: "grid.occupancy_count",
                kind: MetricKind::Gauge,
            }],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Grid>();
        ctx.read_resource::<WorldConfig>();
        ctx.add_system(grid_bootstrap_system);
        ctx.add_metric_publish(grid_metrics_system);
    }
}
