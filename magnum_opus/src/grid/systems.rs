use super::resource::Grid;
use crate::core::MetricsRegistry;
use crate::world_config::WorldConfig;
use bevy::prelude::{Local, Res, ResMut};

pub fn grid_bootstrap_system(
    mut done: Local<bool>,
    cfg: Res<WorldConfig>,
    mut grid: ResMut<Grid>,
) {
    if *done {
        return;
    }
    grid.width = cfg.width;
    grid.height = cfg.height;
    grid.dims_set = true;
    *done = true;
}

pub fn grid_metrics_system(grid: Res<Grid>, mut metrics: ResMut<MetricsRegistry>) {
    metrics.set("grid.occupancy_count", grid.occupancy.len() as f64);
}
