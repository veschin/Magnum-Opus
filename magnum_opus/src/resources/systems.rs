use super::generator::generate_veins;
use super::messages::VeinsGenerated;
use super::resource::ResourceVeins;
use crate::core::MetricsRegistry;
use crate::landscape::{Landscape, LandscapeGenerated};
use crate::world_config::WorldConfig;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::{Local, Res, ResMut};

pub fn resources_bootstrap_system(
    mut done: Local<bool>,
    cfg: Res<WorldConfig>,
    landscape: Res<Landscape>,
    mut veins: ResMut<ResourceVeins>,
    mut reader: MessageReader<LandscapeGenerated>,
    mut writer: MessageWriter<VeinsGenerated>,
) {
    if *done {
        return;
    }
    if !landscape.ready {
        let _ = reader.read();
        return;
    }
    let _ = reader.read();
    let (map, clusters) = generate_veins(cfg.seed, &landscape);
    let count = map.len() as u32;
    veins.veins = map;
    veins.clusters = clusters;
    veins.ready = true;
    writer.write(VeinsGenerated { count });
    *done = true;
}

pub fn resources_metrics_system(
    veins: Res<ResourceVeins>,
    mut reg: ResMut<MetricsRegistry>,
) {
    reg.set("resources.vein_count", veins.veins.len() as f64);
    reg.set("resources.cluster_count", veins.clusters as f64);
    let total: f32 = veins.veins.values().map(|v| v.remaining).sum();
    reg.set("resources.total_amount", total as f64);
}
