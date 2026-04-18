use super::generator::generate_terrain;
use super::messages::LandscapeGenerated;
use super::resource::{Landscape, TerrainKind};
use crate::core::MetricsRegistry;
use crate::world_config::WorldConfig;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::{Local, Res, ResMut};
use std::collections::HashSet;

pub fn landscape_bootstrap_system(
    mut done: Local<bool>,
    cfg: Res<WorldConfig>,
    mut ls: ResMut<Landscape>,
    mut events: MessageWriter<LandscapeGenerated>,
) {
    if *done {
        return;
    }
    ls.width = cfg.width;
    ls.height = cfg.height;
    ls.cells = generate_terrain(cfg.seed, cfg.width, cfg.height);
    ls.ready = true;
    events.write(LandscapeGenerated);
    *done = true;
}

pub fn landscape_metrics_system(ls: Res<Landscape>, mut reg: ResMut<MetricsRegistry>) {
    reg.set("landscape.cells", ls.cells.len() as f64);
    let distinct: HashSet<TerrainKind> = ls.cells.iter().map(|c| c.kind).collect();
    reg.set("landscape.kinds_present", distinct.len() as f64);
}
