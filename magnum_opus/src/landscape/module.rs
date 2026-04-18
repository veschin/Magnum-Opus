use super::messages::LandscapeGenerated;
use super::resource::Landscape;
use super::systems::{landscape_bootstrap_system, landscape_metrics_system};
use crate::core::*;
use crate::names;
use crate::world_config::WorldConfig;

pub struct LandscapeModule;

impl SimDomain for LandscapeModule {
    const ID: &'static str = "landscape";
    const PRIMARY_PHASE: Phase = Phase::World;

    fn contract() -> SimContract {
        SimContract {
            writes: names![Landscape],
            reads: names![WorldConfig],
            messages_out: names![LandscapeGenerated],
            metrics: &[
                MetricDesc {
                    name: "landscape.cells",
                    kind: MetricKind::Gauge,
                },
                MetricDesc {
                    name: "landscape.kinds_present",
                    kind: MetricKind::Gauge,
                },
            ],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Landscape>();
        ctx.read_resource::<WorldConfig>();
        ctx.emit_message::<LandscapeGenerated>();
        ctx.add_system(landscape_bootstrap_system);
        ctx.add_metric_publish(landscape_metrics_system);
    }
}
