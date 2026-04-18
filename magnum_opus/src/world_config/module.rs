use super::resource::WorldConfig;
use crate::core::*;
use crate::names;

pub struct WorldConfigModule;

impl StaticData for WorldConfigModule {
    const ID: &'static str = "world_config";

    fn writes() -> &'static [TypeKey] {
        names![WorldConfig]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(WorldConfig {
            width: 64,
            height: 64,
            seed: 0x9E37_79B9_7F4A_7C15,
        });
    }
}
