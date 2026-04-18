use super::messages::VeinsGenerated;
use super::resource::ResourceVeins;
use super::systems::{resources_bootstrap_system, resources_metrics_system};
use crate::core::*;
use crate::landscape::{Landscape, LandscapeGenerated};
use crate::names;
use crate::world_config::WorldConfig;

pub struct ResourcesModule;

impl SimDomain for ResourcesModule {
    const ID: &'static str = "resources";
    const PRIMARY_PHASE: Phase = Phase::World;

    fn contract() -> SimContract {
        SimContract {
            writes: names![ResourceVeins],
            reads: names![WorldConfig, Landscape],
            messages_in: names![LandscapeGenerated],
            messages_out: names![VeinsGenerated],
            metrics: &[
                MetricDesc {
                    name: "resources.vein_count",
                    kind: MetricKind::Gauge,
                },
                MetricDesc {
                    name: "resources.cluster_count",
                    kind: MetricKind::Gauge,
                },
                MetricDesc {
                    name: "resources.total_amount",
                    kind: MetricKind::Gauge,
                },
            ],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ResourceVeins>();
        ctx.read_resource::<WorldConfig>();
        ctx.read_resource::<Landscape>();
        ctx.read_message::<LandscapeGenerated>();
        ctx.emit_message::<VeinsGenerated>();
        ctx.add_system(resources_bootstrap_system);
        ctx.add_metric_publish(resources_metrics_system);
    }
}
