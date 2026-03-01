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

fn miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 1)
}

#[test]
fn test_three_adjacent_form_one_group() {
    let mut app = test_app();

    // Place 3 adjacent buildings in a row
    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        cmds.queue.push((BuildingType::Miner, 0, 0, miner_recipe()));
        cmds.queue.push((BuildingType::Miner, 1, 0, miner_recipe()));
        cmds.queue.push((BuildingType::Miner, 2, 0, miner_recipe()));
    }

    app.update();

    let mut query = app.world_mut().query::<&GroupMember>();
    let group_ids: Vec<Entity> = query.iter(app.world()).map(|g| g.group_id).collect();

    assert_eq!(group_ids.len(), 3, "all 3 buildings should have GroupMember");
    assert!(
        group_ids.iter().all(|&id| id == group_ids[0]),
        "all 3 should share the same group_id"
    );
}

#[test]
fn test_remove_middle_splits_group() {
    let mut app = test_app();

    // Place 3 adjacent buildings
    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        cmds.queue.push((BuildingType::Miner, 0, 0, miner_recipe()));
        cmds.queue.push((BuildingType::Miner, 1, 0, miner_recipe()));
        cmds.queue.push((BuildingType::Miner, 2, 0, miner_recipe()));
    }
    app.update();

    // Find the middle building (at position 1,0) and despawn it
    let middle_entity = {
        let mut query = app.world_mut().query::<(Entity, &Position)>();
        query.iter(app.world())
            .find(|(_, p)| p.x == 1 && p.y == 0)
            .map(|(e, _)| e)
            .expect("middle building should exist")
    };

    // Despawn middle, update grid, send removal event
    app.world_mut().resource_mut::<Grid>().occupied.remove(&(1, 0));
    app.world_mut().despawn(middle_entity);
    app.world_mut().write_message(crate::events::BuildingRemoved {
        entity: middle_entity, x: 1, y: 0,
    });

    app.update();

    // Remaining buildings at (0,0) and (2,0) should be in different groups
    let mut query = app.world_mut().query::<(&Position, &GroupMember)>();
    let groups: Vec<(i32, Entity)> = query.iter(app.world())
        .map(|(p, g)| (p.x, g.group_id))
        .collect();

    assert_eq!(groups.len(), 2, "should have 2 remaining buildings");
    let g0 = groups.iter().find(|(x, _)| *x == 0).unwrap().1;
    let g2 = groups.iter().find(|(x, _)| *x == 2).unwrap().1;
    assert_ne!(g0, g2, "buildings at (0,0) and (2,0) should be in different groups");
}
