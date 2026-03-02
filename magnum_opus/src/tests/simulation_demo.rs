//! Multi-group supply chain demo — 3 production groups feed a mall
//! that crafts buildings into inventory.
//!
//! Run with: cargo test simulation_demo -- --nocapture
//!
//! Diagnostic tool to observe the full ECS pipeline working together:
//! Input → Groups → Power → Production → Manifold → Transport

use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::*;
use crate::resources::*;
use crate::systems::placement::PlacementCommands;
use crate::SimulationPlugin;

// ═════════════════════════════════════════════════════════════════════════════
// Recipes (3× faster than seed data for more observable cycles)
// ═════════════════════════════════════════════════════════════════════════════

fn wt_recipe() -> Recipe {
    Recipe::simple(vec![], vec![], 1)
}

fn iron_miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 20)
}

fn copper_miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::CopperOre, 1.0)], 20)
}

fn iron_smelter_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::IronOre, 2.0)],
        vec![(ResourceType::IronBar, 1.0)],
        40,
    )
}

fn copper_smelter_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::CopperOre, 2.0)],
        vec![(ResourceType::CopperBar, 1.0)],
        40,
    )
}

fn constructor_recipe() -> Recipe {
    Recipe::mall(
        vec![
            (ResourceType::IronBar, 3.0),
            (ResourceType::CopperBar, 1.0),
        ],
        vec![(ResourceType::ItemIronMiner, 1.0)],
        80,
    )
}

// ═════════════════════════════════════════════════════════════════════════════
// State tracking
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct BuildingSnap {
    building_type: BuildingType,
    active: bool,
    progress: f32,
    idle_reason: Option<IdleReason>,
    outputs: Vec<(ResourceType, f32)>,
    is_mall: bool,
}

#[derive(Clone)]
struct CargoSnap {
    resource: ResourceType,
    amount: f32,
}

struct Tracker {
    buildings: HashMap<(i32, i32), BuildingSnap>,
    cargos: HashMap<Entity, CargoSnap>,
    inventory: HashMap<ResourceType, u32>,
    total_produced: HashMap<ResourceType, f32>,
    total_transported: HashMap<ResourceType, f32>,
    items_crafted: u32,
}

impl Tracker {
    fn new() -> Self {
        Self {
            buildings: HashMap::new(),
            cargos: HashMap::new(),
            inventory: HashMap::new(),
            total_produced: HashMap::new(),
            total_transported: HashMap::new(),
            items_crafted: 0,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Helpers
// ═════════════════════════════════════════════════════════════════════════════

fn bt_name(bt: BuildingType) -> &'static str {
    match bt {
        BuildingType::WindTurbine => "WT",
        BuildingType::IronMiner => "IronMiner",
        BuildingType::CopperMiner => "CopperMiner",
        BuildingType::IronSmelter => "IronSmelter",
        BuildingType::CopperSmelter => "CopperSmelter",
        BuildingType::Constructor => "Constructor",
        _ => "?",
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Event detection + logging
// ═════════════════════════════════════════════════════════════════════════════

fn detect_and_log(app: &mut App, tick: u32, tracker: &mut Tracker) {
    let mut events: Vec<String> = Vec::new();

    // ── Buildings ─────────────────────────────────────────────────────────
    let mut bq = app
        .world_mut()
        .query::<(&Position, &Building, &ProductionState, &Recipe)>();
    let current: Vec<_> = bq
        .iter(app.world())
        .map(|(p, b, ps, r)| {
            (
                (p.x, p.y),
                BuildingSnap {
                    building_type: b.building_type,
                    active: ps.active,
                    progress: ps.progress,
                    idle_reason: ps.idle_reason,
                    outputs: r.outputs.clone(),
                    is_mall: r.output_to_inventory,
                },
            )
        })
        .collect();

    for (pos, snap) in &current {
        // Skip buildings with no real outputs (WindTurbine)
        if snap.outputs.is_empty() {
            continue;
        }
        let name = bt_name(snap.building_type);

        if let Some(prev) = tracker.buildings.get(pos) {
            // Completed production cycle
            if prev.active && !snap.active {
                let out = snap
                    .outputs
                    .iter()
                    .map(|(r, a)| format!("{:.0} {:?}", a, r))
                    .collect::<Vec<_>>()
                    .join(" + ");
                if snap.is_mall {
                    events.push(format!(
                        "[MALL]  ★ {}@({},{}) → {} → Inventory!",
                        name, pos.0, pos.1, out
                    ));
                } else {
                    events.push(format!(
                        "[PROD]  {}@({},{}) → {}",
                        name, pos.0, pos.1, out
                    ));
                }
                for (r, a) in &snap.outputs {
                    *tracker.total_produced.entry(*r).or_default() += a;
                }
                if snap.is_mall {
                    tracker.items_crafted += 1;
                }
            }
            // Started production
            if !prev.active && snap.active {
                events.push(format!("[START] {}@({},{}) started", name, pos.0, pos.1));
            }
            // Went idle (new reason or changed reason)
            if snap.idle_reason.is_some() && snap.idle_reason != prev.idle_reason {
                events.push(format!(
                    "[IDLE]  {}@({},{}) {:?}",
                    name,
                    pos.0,
                    pos.1,
                    snap.idle_reason.unwrap()
                ));
            }
        } else {
            // First time seeing this building
            if snap.active {
                events.push(format!("[START] {}@({},{}) started", name, pos.0, pos.1));
            }
            if let Some(reason) = snap.idle_reason {
                events.push(format!(
                    "[IDLE]  {}@({},{}) {:?}",
                    name, pos.0, pos.1, reason
                ));
            }
        }
    }
    tracker.buildings = current.into_iter().collect();

    // ── Cargo ─────────────────────────────────────────────────────────────
    let mut cq = app.world_mut().query::<(Entity, &Cargo)>();
    let current_cargos: HashMap<Entity, CargoSnap> = cq
        .iter(app.world())
        .map(|(e, c)| {
            (
                e,
                CargoSnap {
                    resource: c.resource,
                    amount: c.amount,
                },
            )
        })
        .collect();

    for (e, c) in &current_cargos {
        if !tracker.cargos.contains_key(e) {
            events.push(format!(
                "[SEND]  ▶ cargo: {:.0} {:?}",
                c.amount, c.resource
            ));
            *tracker.total_transported.entry(c.resource).or_default() += c.amount;
        }
    }
    for (e, c) in &tracker.cargos {
        if !current_cargos.contains_key(e) {
            events.push(format!(
                "[RECV]  ✓ delivered: {:.0} {:?}",
                c.amount, c.resource
            ));
        }
    }
    tracker.cargos = current_cargos;

    // ── Inventory ─────────────────────────────────────────────────────────
    let inv = app.world().resource::<Inventory>();
    for (r, count) in &inv.resources {
        let prev = tracker.inventory.get(r).copied().unwrap_or(0);
        if *count > prev {
            events.push(format!(
                "[INV]   +{} {:?} in inventory!",
                count - prev,
                r
            ));
        }
    }
    tracker.inventory = inv.resources.clone();

    // ── Print ─────────────────────────────────────────────────────────────
    for msg in &events {
        println!("  {:>4}  {}", tick, msg);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Dashboard
// ═════════════════════════════════════════════════════════════════════════════

fn print_dashboard(
    app: &mut App,
    tick: u32,
    tracker: &Tracker,
    group_names: &HashMap<Entity, &str>,
) {
    let energy = app.world().resource::<EnergyPool>();

    println!();
    println!(
        "  ┌── DASHBOARD @ TICK {:>4} ─────────────────────────────────────┐",
        tick
    );
    println!(
        "  │  ENERGY: gen={:.0}  demand={:.0}  ratio={:.2}",
        energy.total_generation, energy.total_consumption, energy.ratio
    );

    // Groups
    let mut gq = app
        .world_mut()
        .query::<(Entity, &GroupEnergy, &Manifold, &GroupPosition)>();
    let mut groups: Vec<_> = gq
        .iter(app.world())
        .map(|(e, ge, m, gp)| {
            (
                e,
                ge.allocated,
                ge.demand,
                ge.ratio(),
                m.resources.clone(),
                gp.x,
                gp.y,
            )
        })
        .collect();
    groups.sort_by_key(|(_, _, _, _, _, _, y)| *y);

    for (entity, alloc, demand, ratio, resources, gx, gy) in &groups {
        let name = group_names.get(entity).copied().unwrap_or("???");
        println!("  │");
        println!(
            "  │  GROUP {:>6} @ ({},{})  energy={:.0}/{:.0} ratio={:.2}",
            name, gx, gy, alloc, demand, ratio
        );
        let res_str: Vec<String> = resources
            .iter()
            .filter(|(_, a)| **a > 0.001)
            .map(|(r, a)| format!("{:?}={:.1}", r, a))
            .collect();
        if res_str.is_empty() {
            println!("  │    manifold: (empty)");
        } else {
            println!("  │    manifold: {}", res_str.join(", "));
        }
    }

    // Buildings grouped by y for readability
    println!("  │");
    println!("  │  BUILDINGS:");
    let mut bq = app
        .world_mut()
        .query::<(&Position, &Building, &ProductionState, &Recipe)>();
    let mut blds: Vec<_> = bq
        .iter(app.world())
        .map(|(p, b, ps, r)| {
            (
                p.x,
                p.y,
                b.building_type,
                ps.active,
                ps.progress,
                r.duration_ticks,
                ps.idle_reason,
            )
        })
        .collect();
    blds.sort_by_key(|(x, y, _, _, _, _, _)| (*y, *x));

    for (x, y, bt, active, progress, dur, idle) in &blds {
        let name = bt_name(*bt);
        let status = if *active {
            let current = progress * *dur as f32;
            format!("active {:.0}/{}", current, dur)
        } else if let Some(reason) = idle {
            format!("idle({:?})", reason)
        } else if bt.energy_generation() > 0.0 {
            "gen".to_string()
        } else {
            "idle".to_string()
        };
        println!("  │    {}@({},{}) {}", name, x, y, status);
    }

    // Cargo
    if !tracker.cargos.is_empty() {
        println!("  │");
        println!("  │  CARGO IN TRANSIT: {}", tracker.cargos.len());
        let mut by_res: HashMap<ResourceType, (f32, usize)> = HashMap::new();
        for c in tracker.cargos.values() {
            let e = by_res.entry(c.resource).or_default();
            e.0 += c.amount;
            e.1 += 1;
        }
        for (r, (amt, n)) in &by_res {
            println!("  │    {:?}: {}× ({:.0} total)", r, n, amt);
        }
    }

    // Inventory
    let inv = app.world().resource::<Inventory>();
    let items: Vec<_> = inv.resources.iter().filter(|(_, c)| **c > 0).collect();
    println!("  │");
    if items.is_empty() {
        println!("  │  INVENTORY: (empty)");
    } else {
        println!("  │  INVENTORY:");
        for (r, c) in &items {
            println!("  │    {:?} × {}", r, c);
        }
    }

    // Totals
    println!("  │");
    println!("  │  PRODUCTION TOTALS:");
    let mut sorted: Vec<_> = tracker
        .total_produced
        .iter()
        .filter(|(_, a)| **a > 0.001)
        .collect();
    sorted.sort_by_key(|(r, _)| format!("{:?}", r));
    if sorted.is_empty() {
        println!("  │    (nothing yet)");
    } else {
        for (r, a) in &sorted {
            println!("  │    {:?}: {:.0}", r, a);
        }
    }
    println!("  │  Items crafted: {}", tracker.items_crafted);
    println!(
        "  └────────────────────────────────────────────────────────────┘"
    );
    println!();
}

// ═════════════════════════════════════════════════════════════════════════════
// Transport path helper
// ═════════════════════════════════════════════════════════════════════════════

fn spawn_path(
    world: &mut World,
    source_group: Entity,
    target_group: Entity,
    waypoints: Vec<(i32, i32)>,
) -> Entity {
    let stats = TierStats::for_path(1);
    let resource_filter = world
        .entity(source_group)
        .get::<TransportSender>()
        .and_then(|s| s.resource);

    let path_entity = world
        .spawn(TransportPath {
            kind: TransportKind::RunePath,
            source_group,
            target_group,
            resource_filter,
            tier: 1,
            capacity: stats.capacity,
            speed: stats.speed,
            connected: true,
            segments: waypoints.clone(),
        })
        .id();

    world.spawn(PathConnection {
        source_group,
        target_group,
        path_entity,
    });

    {
        let occupancy = &mut world.resource_mut::<PathOccupancy>();
        for pos in &waypoints {
            occupancy.tiles.insert(*pos, path_entity);
        }
    }

    for (idx, pos) in waypoints.iter().enumerate() {
        world.spawn(PathSegmentTile {
            path_entity,
            tile_pos: *pos,
            segment_index: idx,
        });
    }

    path_entity
}

// ═════════════════════════════════════════════════════════════════════════════
// THE SIMULATION
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn simulation_demo() {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  MAGNUM OPUS — MULTI-GROUP SUPPLY CHAIN DEMO");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  Scenario: Iron + Copper extraction → smelting → transport → mall");
    println!("  Goal: Constructor produces ItemIronMiner → Inventory");
    println!("  Recipes at 3× speed (20/40/80 ticks instead of 60/120/300)");
    println!();
    println!("  MAP (30×15):");
    println!("  y=2:  [WT][IM][IM][IS]  ════ IronBar path ═══════════╗");
    println!("        (1,2)(2,2)(3,2)(4,2)           12 tiles        ║");
    println!("                                                       ║");
    println!("  y=5:                      [WT][WT][Constructor 2×2] ◄╣");
    println!("                            (14)(15)(16,5─17,6)        ║");
    println!("                                                       ║");
    println!("  y=8:  [WT][CM][CS]  ════ CopperBar path ════════════╝");
    println!("        (1,8)(2,8)(3,8)             11 tiles");
    println!();
    println!("  Groups:");
    println!("    #1 Iron:   WT+2×IM+IS   gen=20 dem=20  IronOre → IronBar");
    println!("    #2 Copper: WT+CM+CS     gen=20 dem=15  CopperOre → CopperBar");
    println!("    #3 Mall:   2×WT+Constr  gen=40 dem=15  3 IronBar + 1 CopperBar → ItemIronMiner");
    println!();
    println!("  Recipes:");
    println!("    IronMiner:     [] → 1 IronOre          20 ticks  energy=5");
    println!("    CopperMiner:   [] → 1 CopperOre        20 ticks  energy=5");
    println!("    IronSmelter:   2 IronOre → 1 IronBar   40 ticks  energy=10");
    println!("    CopperSmelter: 2 CopperOre → 1 CopperBar  40 ticks  energy=10");
    println!("    Constructor:   3 IronBar + 1 CopperBar → 1 ItemIronMiner  80 ticks  energy=15");
    println!();

    // ─── Setup ───────────────────────────────────────────────────────────

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin {
        grid_width: 30,
        grid_height: 15,
    });

    // Terrain
    {
        let mut grid = app.world_mut().resource_mut::<Grid>();
        grid.terrain.insert((2, 2), TerrainType::IronVein);
        grid.terrain.insert((3, 2), TerrainType::IronVein);
        grid.terrain.insert((2, 8), TerrainType::CopperVein);
    }
    app.world_mut().resource_mut::<FogMap>().reveal_all(30, 15);

    // Place all buildings
    {
        let mut cmds = app.world_mut().resource_mut::<PlacementCommands>();
        // Iron group (y=2)
        cmds.queue
            .push((BuildingType::WindTurbine, 1, 2, wt_recipe()));
        cmds.queue
            .push((BuildingType::IronMiner, 2, 2, iron_miner_recipe()));
        cmds.queue
            .push((BuildingType::IronMiner, 3, 2, iron_miner_recipe()));
        cmds.queue
            .push((BuildingType::IronSmelter, 4, 2, iron_smelter_recipe()));
        // Copper group (y=8)
        cmds.queue
            .push((BuildingType::WindTurbine, 1, 8, wt_recipe()));
        cmds.queue
            .push((BuildingType::CopperMiner, 2, 8, copper_miner_recipe()));
        cmds.queue.push((
            BuildingType::CopperSmelter,
            3,
            8,
            copper_smelter_recipe(),
        ));
        // Mall group (y=5, x=14+)
        cmds.queue
            .push((BuildingType::WindTurbine, 14, 5, wt_recipe()));
        cmds.queue
            .push((BuildingType::WindTurbine, 15, 5, wt_recipe()));
        cmds.queue
            .push((BuildingType::Constructor, 16, 5, constructor_recipe()));
    }

    // Tick 1: placement + group formation + first production tick
    app.update();

    // ─── Find groups ─────────────────────────────────────────────────────

    let mut group_names: HashMap<Entity, &str> = HashMap::new();

    let iron_group = {
        let mut gq = app.world_mut().query::<(Entity, &GroupPosition)>();
        gq.iter(app.world())
            .find(|(_, gp)| gp.y == 2)
            .map(|(e, _)| e)
    };
    let copper_group = {
        let mut gq = app.world_mut().query::<(Entity, &GroupPosition)>();
        gq.iter(app.world())
            .find(|(_, gp)| gp.y == 8)
            .map(|(e, _)| e)
    };
    let mall_group = {
        let mut gq = app.world_mut().query::<(Entity, &GroupPosition)>();
        gq.iter(app.world())
            .find(|(_, gp)| gp.y >= 5 && gp.x >= 14)
            .map(|(e, _)| e)
    };

    if let Some(e) = iron_group {
        group_names.insert(e, "Iron");
    }
    if let Some(e) = copper_group {
        group_names.insert(e, "Copper");
    }
    if let Some(e) = mall_group {
        group_names.insert(e, "Mall");
    }

    println!("  Groups formed after tick 1:");
    for (e, name) in &group_names {
        println!("    {}: {:?}", name, e);
    }
    println!();

    // ─── Setup transport ─────────────────────────────────────────────────

    let transport_ok =
        if let (Some(iron_grp), Some(copper_grp), Some(mall_grp)) =
            (iron_group, copper_group, mall_group)
        {
            // Senders on production groups
            app.world_mut().entity_mut(iron_grp).insert(TransportSender {
                resource: Some(ResourceType::IronBar),
                disconnected: false,
            });
            app.world_mut()
                .entity_mut(copper_grp)
                .insert(TransportSender {
                    resource: Some(ResourceType::CopperBar),
                    disconnected: false,
                });
            // Receiver on mall (resource=None accepts anything)
            app.world_mut()
                .entity_mut(mall_grp)
                .insert(TransportReceiver {
                    resource: None,
                    demand: 5,
                    disconnected: false,
                });

            // Iron → Mall: L-shape right then down
            let iron_wp: Vec<(i32, i32)> = {
                let mut p: Vec<(i32, i32)> = (5..=12).map(|x| (x, 2)).collect();
                p.extend((3..=5).map(|y| (12, y)));
                p.push((13, 5));
                p
            };
            // Copper → Mall: L-shape right then up
            let copper_wp: Vec<(i32, i32)> = {
                let mut p: Vec<(i32, i32)> = (4..=10).map(|x| (x, 8)).collect();
                p.extend((5..=7).rev().map(|y| (10, y)));
                p.push((11, 5));
                p
            };

            println!("  Transport:");
            println!(
                "    Iron→Mall:   {} tiles, speed=1.0, cap=2",
                iron_wp.len()
            );
            println!(
                "    Copper→Mall: {} tiles, speed=1.0, cap=2",
                copper_wp.len()
            );

            spawn_path(app.world_mut(), iron_grp, mall_grp, iron_wp);
            spawn_path(app.world_mut(), copper_grp, mall_grp, copper_wp);
            true
        } else {
            println!("  WARN: Not all groups found — transport skipped");
            false
        };

    println!();
    println!("───────────────────────────────────────────────────────────────────");
    println!(
        "  {:>4}  {:<8} {}",
        "TICK", "TAG", "EVENT"
    );
    println!("───────────────────────────────────────────────────────────────────");

    // ─── Main simulation loop ────────────────────────────────────────────

    let mut tracker = Tracker::new();

    // Capture tick 1 state (detect initial starts/idles)
    detect_and_log(&mut app, 1, &mut tracker);

    for tick in 2..=500u32 {
        app.update();
        detect_and_log(&mut app, tick, &mut tracker);

        if tick % 100 == 0 {
            print_dashboard(&mut app, tick, &tracker, &group_names);
        }
    }

    // ─── Final summary ───────────────────────────────────────────────────

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  FINAL SUMMARY (500 ticks)");
    println!("═══════════════════════════════════════════════════════════════════");

    let (e_gen, e_dem, e_ratio) = {
        let e = app.world().resource::<EnergyPool>();
        (e.total_generation, e.total_consumption, e.ratio)
    };
    println!(
        "  Energy: gen={:.0}  demand={:.0}  ratio={:.2}",
        e_gen, e_dem, e_ratio
    );

    // Groups
    let mut gq = app
        .world_mut()
        .query::<(Entity, &GroupEnergy, &Manifold)>();
    let group_count = gq.iter(app.world()).count();
    println!("  Groups: {}", group_count);

    println!();
    println!("  Production totals:");
    let mut sorted: Vec<_> = tracker
        .total_produced
        .iter()
        .filter(|(_, a)| **a > 0.001)
        .collect();
    sorted.sort_by_key(|(r, _)| format!("{:?}", r));
    for (r, a) in &sorted {
        println!("    {:?}: {:.0}", r, a);
    }

    println!();
    println!("  Transported totals:");
    let mut sorted: Vec<_> = tracker
        .total_transported
        .iter()
        .filter(|(_, a)| **a > 0.001)
        .collect();
    sorted.sort_by_key(|(r, _)| format!("{:?}", r));
    if sorted.is_empty() {
        println!("    (none)");
    } else {
        for (r, a) in &sorted {
            println!("    {:?}: {:.0}", r, a);
        }
    }

    let inv = app.world().resource::<Inventory>();
    println!();
    let items: Vec<_> = inv.resources.iter().filter(|(_, c)| **c > 0).collect();
    if items.is_empty() {
        println!("  Inventory: (empty)");
    } else {
        println!("  Inventory (mall output):");
        for (r, c) in &items {
            println!("    {:?} × {}", r, c);
        }
    }

    println!();
    println!("  Items crafted: {}", tracker.items_crafted);

    // Health check
    println!();
    println!("  HEALTH CHECK:");
    let iron_ore = tracker
        .total_produced
        .get(&ResourceType::IronOre)
        .copied()
        .unwrap_or(0.0);
    let iron_bar = tracker
        .total_produced
        .get(&ResourceType::IronBar)
        .copied()
        .unwrap_or(0.0);
    let copper_ore = tracker
        .total_produced
        .get(&ResourceType::CopperOre)
        .copied()
        .unwrap_or(0.0);
    let copper_bar = tracker
        .total_produced
        .get(&ResourceType::CopperBar)
        .copied()
        .unwrap_or(0.0);

    let pass = |v: bool| if v { "YES ✓" } else { "NO  ✗" };
    println!(
        "    IronOre produced:     {}",
        pass(iron_ore > 0.0)
    );
    println!(
        "    IronBar smelted:      {}",
        pass(iron_bar > 0.0)
    );
    println!(
        "    CopperOre produced:   {}",
        pass(copper_ore > 0.0)
    );
    println!(
        "    CopperBar smelted:    {}",
        pass(copper_bar > 0.0)
    );
    println!(
        "    Transport working:    {}",
        pass(!tracker.total_transported.is_empty())
    );
    println!(
        "    Items crafted (≥1):   {}",
        pass(tracker.items_crafted >= 1)
    );
    println!(
        "    Energy stable:        {}",
        pass(e_ratio >= 1.0)
    );

    println!();
    println!("═══════════════════════════════════════════════════════════════════");

    // Hard assertions
    assert!(iron_ore > 0.0, "No iron ore produced!");
    assert!(iron_bar > 0.0, "No iron bars smelted!");
    assert!(copper_ore > 0.0, "No copper ore produced!");
    assert!(copper_bar > 0.0, "No copper bars smelted!");
    assert!(transport_ok, "Transport setup failed!");
}
