use bevy::prelude::*;

use crate::components::*;
use crate::resources::Grid;
use crate::systems::placement::PlacementCommands;
use crate::SimulationPlugin;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin::default());
    app
}

#[test]
fn test_place_building_on_empty_grid() {
    let mut app = test_app();

    app.world_mut().resource_mut::<PlacementCommands>().queue.push((
        BuildingType::Miner,
        3, 4,
        Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 1),
    ));

    app.update();

    let grid = app.world().resource::<Grid>();
    assert!(grid.occupied.contains(&(3, 4)), "grid should mark (3,4) as occupied");

    let mut query = app.world_mut().query::<(&Position, &Building)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.x, 3);
    assert_eq!(results[0].0.y, 4);
    assert_eq!(results[0].1.building_type, BuildingType::Miner);
}

#[test]
fn test_place_building_on_occupied_tile() {
    let mut app = test_app();

    // Place first building
    app.world_mut().resource_mut::<PlacementCommands>().queue.push((
        BuildingType::Miner,
        3, 4,
        Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 1),
    ));
    app.update();

    // Try to place second building on same tile
    app.world_mut().resource_mut::<PlacementCommands>().queue.push((
        BuildingType::Smelter,
        3, 4,
        Recipe::simple(vec![(ResourceType::IronOre, 2.0)], vec![(ResourceType::IronBar, 1.0)], 2),
    ));
    app.update();

    let mut query = app.world_mut().query::<&Building>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "should still have only 1 building");
}
