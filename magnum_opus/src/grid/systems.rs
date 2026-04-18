use super::commands::PlaceTile;
use super::position::Position;
use super::resource::Grid;
use crate::buildings::Building;
use crate::core::{CommandBus, MetricsRegistry};
use crate::world_config::WorldConfig;
use bevy::prelude::{Commands, Local, Res, ResMut};

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

pub fn grid_placement_drain_system(
    mut commands: Commands,
    mut bus: ResMut<CommandBus<PlaceTile>>,
    mut grid: ResMut<Grid>,
) {
    if !grid.dims_set {
        return;
    }
    for cmd in bus.drain() {
        if cmd.x >= grid.width || cmd.y >= grid.height {
            continue;
        }
        if grid.occupancy.contains_key(&(cmd.x, cmd.y)) {
            continue;
        }
        let position = Position {
            x: cmd.x,
            y: cmd.y,
        };
        let entity = match cmd.building_type {
            Some(building_type) => commands
                .spawn((position, Building { building_type }))
                .id(),
            None => commands.spawn(position).id(),
        };
        grid.occupancy.insert((cmd.x, cmd.y), entity);
    }
}

pub fn grid_metrics_system(grid: Res<Grid>, mut metrics: ResMut<MetricsRegistry>) {
    metrics.set("grid.occupancy_count", grid.occupancy.len() as f64);
}
