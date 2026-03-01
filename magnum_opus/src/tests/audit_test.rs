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

fn print_tick_state(app: &mut App, tick: u32) {
    println!("\n═══ TICK {tick} ═══");

    // Groups
    let mut group_query = app.world_mut().query::<(Entity, &GroupEnergy, &Manifold)>();
    let group_data: Vec<_> = group_query.iter(app.world())
        .map(|(e, ge, m)| (e, ge.allocated, ge.demand, ge.ratio(), m.resources.clone()))
        .collect();

    for (entity, allocated, demand, ratio, resources) in &group_data {
        println!("  [group]      {entity:?}: gen={allocated:.1}, demand={demand:.1}, ratio={ratio:.2}");
        if !resources.is_empty() {
            let items: Vec<String> = resources.iter()
                .map(|(r, a)| format!("{r:?}:{a:.1}"))
                .collect();
            println!("               manifold = {{{}}}", items.join(", "));
        }
    }

    // Buildings
    let mut building_query = app.world_mut().query::<(
        &Position, &Building, &ProductionState, &Recipe,
        &InputBuffer, &OutputBuffer, &GroupMember,
    )>();
    let building_data: Vec<_> = building_query.iter(app.world())
        .map(|(pos, b, ps, r, ib, ob, gm)| {
            (pos.x, pos.y, b.building_type, ps.active, ps.progress,
             r.inputs.clone(), r.outputs.clone(), r.duration_ticks,
             ib.slots.clone(), ob.slots.clone(), gm.group_id)
        })
        .collect();

    for (x, y, btype, active, progress, inputs, outputs, dur, in_buf, out_buf, _gid) in &building_data {
        let name = match btype {
            BuildingType::Miner          => "Miner",
            BuildingType::Smelter        => "Smelter",
            BuildingType::EnergySource   => "EnergySource",
            BuildingType::IronMiner      => "IronMiner",
            BuildingType::IronSmelter    => "IronSmelter",
            BuildingType::CopperMiner    => "CopperMiner",
            BuildingType::CopperSmelter  => "CopperSmelter",
            BuildingType::Constructor    => "Constructor",
            BuildingType::StoneQuarry    => "StoneQuarry",
            BuildingType::Sawmill        => "Sawmill",
            BuildingType::WindTurbine    => "WindTurbine",
            BuildingType::WaterWheel     => "WaterWheel",
            BuildingType::LavaGenerator  => "LavaGenerator",
            BuildingType::ManaReactor    => "ManaReactor",
            BuildingType::WaterPump      => "WaterPump",
            BuildingType::TreeFarm       => "TreeFarm",
            BuildingType::Watchtower     => "Watchtower",
            BuildingType::SteelSmelter   => "SteelSmelter",
            BuildingType::OpusForge      => "OpusForge",
            BuildingType::ImpCamp        => "ImpCamp",
            BuildingType::BreedingPen    => "BreedingPen",
            _ => "Unknown",
        };

        let status = if *active {
            format!("ACTIVE progress={progress:.2}/{dur}")
        } else if inputs.is_empty() && !outputs.is_empty() {
            "IDLE (no inputs needed)".to_string()
        } else if !inputs.is_empty() {
            let needs: Vec<String> = inputs.iter()
                .map(|(r, a)| {
                    let have = in_buf.get(r).copied().unwrap_or(0.0);
                    format!("{r:?}: need {a:.1}, have {have:.1}")
                })
                .collect();
            format!("WAIT ({})", needs.join(", "))
        } else {
            "IDLE".to_string()
        };

        print!("  [production] {name}@({x},{y}): {status}");

        if !in_buf.is_empty() {
            let items: Vec<String> = in_buf.iter()
                .filter(|(_, a)| **a > 0.001)
                .map(|(r, a)| format!("{r:?}:{a:.1}"))
                .collect();
            if !items.is_empty() {
                print!(" | in={{{}}}", items.join(", "));
            }
        }
        if !out_buf.is_empty() {
            let items: Vec<String> = out_buf.iter()
                .filter(|(_, a)| **a > 0.001)
                .map(|(r, a)| format!("{r:?}:{a:.1}"))
                .collect();
            if !items.is_empty() {
                print!(" | out={{{}}}", items.join(", "));
            }
        }
        println!();
    }
}

#[test]
fn test_audit_trail() {
    let mut app = test_app();

    println!("\n══════════════════════════════════════════");
    println!("  AUDIT TRAIL: Miner → Smelter → IronBar");
    println!("══════════════════════════════════════════");
    println!("\nSetup:");
    println!("  Miner@(0,0)        recipe: [] → [IronOre:1.0], duration=1");
    println!("  Smelter@(1,0)      recipe: [IronOre:2.0] → [IronBar:1.0], duration=2");
    println!("  EnergySource@(0,1) recipe: [] → [], duration=1");
    println!("  Energy: 1 source, 2 consumers → ratio=0.50");

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

    let mut bar_produced = false;
    for tick in 1..=12 {
        app.update();
        print_tick_state(&mut app, tick);

        // Check if bars appeared
        let mut mq = app.world_mut().query::<&Manifold>();
        let bars: f32 = mq.iter(app.world())
            .map(|m| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
            .sum();
        if bars > 0.0 && !bar_produced {
            println!("\n  *** IRON BAR PRODUCED at tick {tick}! ***");
            bar_produced = true;
        }
    }

    println!("\n══════════════════════════════════════════");
    assert!(bar_produced, "should have produced iron bars within 12 ticks");
}
