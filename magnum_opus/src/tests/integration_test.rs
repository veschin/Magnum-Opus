use bevy::prelude::*;

use crate::components::*;
use crate::systems::placement::PlacementCommands;
use crate::SimulationPlugin;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin::default());
    app
}

#[test]
fn test_miner_smelter_produces_bars() {
    let mut app = test_app();

    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        cmds.queue.push((
            BuildingType::Miner, 0, 0,
            Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 1),
        ));
        cmds.queue.push((
            BuildingType::Smelter, 1, 0,
            Recipe::simple(vec![(ResourceType::IronOre, 2.0)], vec![(ResourceType::IronBar, 1.0)], 2),
        ));
        cmds.queue.push((
            BuildingType::EnergySource, 0, 1,
            Recipe::simple(vec![], vec![], 1),
        ));
    }

    // Buffer-mediated flow is slower — need more ticks
    for _ in 0..15 {
        app.update();
    }

    // Check Manifold components on Group entities
    let mut manifold_query = app.world_mut().query::<&Manifold>();
    let has_bars = manifold_query.iter(app.world()).any(|m| {
        m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0) > 0.0
    });
    assert!(has_bars, "should have produced iron bars after 15 ticks");

    let first_run_bars: f32 = manifold_query.iter(app.world())
        .map(|m| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
        .sum();

    // Determinism check
    let mut app2 = test_app();
    {
        let mut cmds = app2.world_mut().resource_mut::<PlacementCommands>();
        cmds.queue.push((
            BuildingType::Miner, 0, 0,
            Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 1),
        ));
        cmds.queue.push((
            BuildingType::Smelter, 1, 0,
            Recipe::simple(vec![(ResourceType::IronOre, 2.0)], vec![(ResourceType::IronBar, 1.0)], 2),
        ));
        cmds.queue.push((
            BuildingType::EnergySource, 0, 1,
            Recipe::simple(vec![], vec![], 1),
        ));
    }
    for _ in 0..15 {
        app2.update();
    }

    let mut manifold_query2 = app2.world_mut().query::<&Manifold>();
    let second_run_bars: f32 = manifold_query2.iter(app2.world())
        .map(|m| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
        .sum();

    assert_eq!(
        first_run_bars, second_run_bars,
        "deterministic: same setup should produce same results"
    );

    println!("Integration test: Iron bars produced = {first_run_bars}");
}
