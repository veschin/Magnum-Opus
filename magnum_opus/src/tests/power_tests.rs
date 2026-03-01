use bevy::prelude::*;

use crate::components::*;
use crate::resources::EnergyPool;
use crate::systems::placement::PlacementCommands;
use crate::SimulationPlugin;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin::default());
    app
}

fn energy_recipe() -> Recipe {
    Recipe::simple(vec![], vec![], 1)
}

fn production_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::IronOre, 1.0)],
        vec![(ResourceType::IronBar, 1.0)],
        2,
    )
}

#[test]
fn test_energy_surplus_speeds_production() {
    let mut app = test_app();

    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        // 2 energy buildings
        cmds.queue.push((BuildingType::EnergySource, 0, 0, energy_recipe()));
        cmds.queue.push((BuildingType::EnergySource, 1, 0, energy_recipe()));
        // 1 production building
        cmds.queue.push((BuildingType::Smelter, 2, 0, production_recipe()));
    }

    app.update();

    let pool = app.world().resource::<EnergyPool>();
    // 2 EnergySource (gen=1.0 each) vs 1 Smelter (cons=1.0 legacy unit).
    // Legacy types use 1.0 unit counting so ratio = 2/1 = 2.0 (surplus).
    assert!(pool.ratio > 1.0, "energy ratio should be > 1.0 with surplus, got {}", pool.ratio);
    assert_eq!(pool.total_generation, 2.0);
    assert_eq!(pool.total_consumption, 1.0);
}

#[test]
fn test_energy_deficit_slows_production() {
    let mut app = test_app();

    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        // 0 energy buildings, 3 production buildings (Smelter = 10 each)
        cmds.queue.push((BuildingType::Smelter, 0, 0, production_recipe()));
        cmds.queue.push((BuildingType::Smelter, 1, 0, production_recipe()));
        cmds.queue.push((BuildingType::Smelter, 2, 0, production_recipe()));
    }

    app.update();

    let pool = app.world().resource::<EnergyPool>();
    assert_eq!(pool.ratio, 0.0, "energy ratio should be 0.0 with no generation");
    assert_eq!(pool.total_generation, 0.0);
    assert!(pool.total_consumption > 0.0, "should have non-zero consumption from smelters");
}
