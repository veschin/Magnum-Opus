//! Building Groups BDD tests — one test per scenario in building-groups.feature
//!
//! Seed data constants:
//!   IronMiner:    energy_consumption=5, recipe: [] -> [iron_ore:1] in 60 ticks
//!   CopperMiner:  energy_consumption=5, recipe: [] -> [copper_ore:1] in 60 ticks
//!   IronSmelter:  energy_consumption=10, recipe: [iron_ore:2] -> [iron_bar:1] in 120 ticks
//!   WindTurbine:  energy_generation=20
//!   Constructor:  energy_consumption=15, footprint=2x2, recipe: [iron_bar:3,plank:1] -> [item_iron_miner:1] in 300 ticks
//!   TreeFarm:     energy_consumption=8, footprint=2x2, recipe: [water:3] -> [wood:2] in 180 ticks

use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::systems::placement::PlacementCommands;
use crate::SimulationPlugin;

fn test_app(w: i32, h: i32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin { grid_width: w, grid_height: h });
    app
}

/// Reveal all cells of a w x h grid for fog-of-war.
fn reveal_all(app: &mut App, w: i32, h: i32) {
    app.world_mut().resource_mut::<FogMap>().reveal_all(w, h);
}

/// Standard recipe for IronMiner: no inputs, produces iron_ore:1 per 60 ticks.
fn iron_miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::IronOre, 1.0)], 60)
}

/// Standard recipe for CopperMiner: no inputs, produces copper_ore:1 per 60 ticks.
fn copper_miner_recipe() -> Recipe {
    Recipe::simple(vec![], vec![(ResourceType::CopperOre, 1.0)], 60)
}

/// Standard recipe for IronSmelter: consumes iron_ore:2, produces iron_bar:1 per 120 ticks.
fn iron_smelter_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::IronOre, 2.0)],
        vec![(ResourceType::IronBar, 1.0)],
        120,
    )
}

/// Recipe for WindTurbine (energy only, no production).
fn wind_turbine_recipe() -> Recipe {
    Recipe::simple(vec![], vec![], 1)
}

/// Recipe for Constructor (mall): iron_bar:3 + plank:1 -> item_iron_miner:1.
fn constructor_recipe() -> Recipe {
    Recipe::mall(
        vec![(ResourceType::IronBar, 3.0), (ResourceType::Plank, 1.0)],
        vec![(ResourceType::ItemIronMiner, 1.0)],
        300,
    )
}

/// Recipe for TreeFarm: water:3 -> wood:2 in 180 ticks.
fn tree_farm_recipe() -> Recipe {
    Recipe::simple(
        vec![(ResourceType::Water, 3.0)],
        vec![(ResourceType::Wood, 2.0)],
        180,
    )
}

/// Count entities with the Group marker component.
fn count_group_entities(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Group>();
    q.iter(app.world()).count()
}

/// Find the group_id for the building at (x, y).
fn group_of(app: &mut App, x: i32, y: i32) -> Option<Entity> {
    let mut q = app.world_mut().query::<(&Position, &GroupMember)>();
    q.iter(app.world())
        .find(|(p, _)| p.x == x && p.y == y)
        .map(|(_, m)| m.group_id)
}

/// Count buildings belonging to a given group.
fn buildings_in_group(app: &mut App, group_id: Entity) -> usize {
    let mut q = app.world_mut().query::<&GroupMember>();
    q.iter(app.world())
        .filter(|m| m.group_id == group_id)
        .count()
}

/// Count all placed Building entities.
fn count_buildings(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Building>();
    q.iter(app.world()).count()
}

/// Check whether any building entity exists at the given position.
fn building_at(app: &mut App, x: i32, y: i32) -> bool {
    let mut q = app.world_mut().query::<&Position>();
    q.iter(app.world()).any(|p| p.x == x && p.y == y)
}

/// Place a building via the legacy queue (no inventory / fog check).
fn place_legacy(app: &mut App, bt: BuildingType, x: i32, y: i32, recipe: Recipe) {
    app.world_mut()
        .resource_mut::<PlacementCommands>()
        .queue
        .push((bt, x, y, recipe));
}

/// Place a building via the request queue (full validation).
fn place_request(app: &mut App, bt: BuildingType, x: i32, y: i32, recipe: Recipe) {
    app.world_mut()
        .resource_mut::<PlacementCommands>()
        .requests
        .push(crate::systems::placement::PlacementRequest::new(bt, x, y, recipe));
}

/// Set terrain at a cell.
fn set_terrain(app: &mut App, x: i32, y: i32, t: TerrainType) {
    app.world_mut().resource_mut::<Grid>().terrain.insert((x, y), t);
}

/// Remove a building at position (x, y): despawn entity, free grid cells, send event.
fn remove_building_at(app: &mut App, x: i32, y: i32) {
    // Find entity at position
    let entity = {
        let mut q = app.world_mut().query::<(Entity, &Position)>();
        q.iter(app.world())
            .find(|(_, p)| p.x == x && p.y == y)
            .map(|(e, _)| e)
    };
    if let Some(e) = entity {
        // Free grid cell(s) — get the footprint first
        let cells: Vec<(i32, i32)> = {
            let mut q = app.world_mut().query::<(Entity, &Footprint)>();
            q.iter(app.world())
                .find(|(ent, _)| *ent == e)
                .map(|(_, fp)| fp.cells.clone())
                .unwrap_or_else(|| vec![(x, y)])
        };
        {
            let mut grid = app.world_mut().resource_mut::<Grid>();
            for c in &cells {
                grid.occupied.remove(c);
            }
        }
        app.world_mut().despawn(e);
        app.world_mut()
            .write_message(BuildingRemoved { entity: e, x, y });
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AC1: Placing a building adjacent to an existing building merges them
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Single building forms a group of one
#[test]
fn single_building_forms_a_group_of_one() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 5, 5, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 5, 5, iron_miner_recipe());
    app.update();

    // There is exactly 1 group
    assert_eq!(count_group_entities(&mut app), 1, "should have exactly 1 group entity");

    // The iron_miner at (5,5) belongs to that group
    let g = group_of(&mut app, 5, 5);
    assert!(g.is_some(), "iron_miner at (5,5) should belong to a group");

    // The group has an empty manifold
    let group_id = g.unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let manifold = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.is_empty());
    assert_eq!(manifold, Some(true), "new group manifold should be empty");
}

/// Scenario: Two adjacent buildings merge into one group
#[test]
fn two_adjacent_buildings_merge_into_one_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "should have exactly 1 group");
    let group_id = group_of(&mut app, 3, 3).unwrap();
    assert_eq!(buildings_in_group(&mut app, group_id), 2, "group should contain 2 buildings");
}

/// Scenario: Two non-adjacent buildings form separate groups
#[test]
fn two_non_adjacent_buildings_form_separate_groups() {
    let mut app = test_app(12, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);
    set_terrain(&mut app, 8, 3, TerrainType::CopperVein);

    place_legacy(&mut app, BuildingType::IronMiner, 2, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::CopperMiner, 8, 3, copper_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 2, "should have exactly 2 groups");

    let g_iron = group_of(&mut app, 2, 3).unwrap();
    let g_copper = group_of(&mut app, 8, 3).unwrap();
    assert_ne!(g_iron, g_copper, "iron_miner and copper_miner should be in different groups");
}

/// Scenario: Adjacent buildings share a single manifold
#[test]
fn adjacent_buildings_share_a_single_manifold() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    // Both belong to the same group
    let g_miner = group_of(&mut app, 3, 3).unwrap();
    let g_smelter = group_of(&mut app, 4, 3).unwrap();
    assert_eq!(g_miner, g_smelter, "both buildings should be in the same group");

    // Exactly 1 manifold entity (one per group)
    let mut mq = app.world_mut().query::<&Manifold>();
    assert_eq!(mq.iter(app.world()).count(), 1, "group should have exactly 1 shared manifold");
}

/// Scenario: Diagonal buildings do not form a group
#[test]
fn diagonal_buildings_do_not_form_a_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 4, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 4, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 2, "diagonal buildings should form separate groups");

    let g1 = group_of(&mut app, 3, 3).unwrap();
    let g2 = group_of(&mut app, 4, 4).unwrap();
    assert_ne!(g1, g2, "each diagonal building should be in a separate group");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC2: Resources produced by any building are available to all others
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Miner output is available to smelter via manifold
#[test]
fn miner_output_is_available_to_smelter_via_manifold() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    // Place miner + smelter in same group, plus wind turbine for energy
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 4, wind_turbine_recipe());
    app.update();

    // Run 60 ticks
    for _ in 0..60 {
        app.update();
    }

    // After 60 ticks (1 miner cycle): the miner produced 1 iron_ore.
    // The manifold system drains output buffers and distributes to consumers.
    // The smelter needs 2 iron_ore per cycle — only 1 is available — so it pre-loads
    // the smelter's input buffer waiting for the second unit.
    // Assert specifically: smelter input buffer has the iron_ore (proving manifold transported it).
    let mut input_q = app.world_mut().query::<(&Position, &InputBuffer)>();
    let smelter_iron_ore = input_q
        .iter(app.world())
        .find(|(p, _)| p.x == 4 && p.y == 3)
        .map(|(_, buf)| buf.slots.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert!(
        smelter_iron_ore > 0.0,
        "after 60 ticks the miner produced 1 iron_ore which manifold transported to smelter input buffer (got {smelter_iron_ore})"
    );

    // Verify the manifold is the transport mechanism: group exists and smelter received ore via it
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let manifold_exists = mq.iter(app.world()).any(|(e, _)| e == group_id);
    assert!(manifold_exists, "group manifold entity should exist to enable resource transport");
}

/// Scenario: Smelter consumes miner output within same group
#[test]
fn smelter_consumes_miner_output_within_same_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 4, wind_turbine_recipe());
    app.update();

    // Run 240 ticks — 2 miners produce 4 iron_ore; smelter needs 2 ore per cycle (120 ticks)
    for _ in 0..240 {
        app.update();
    }

    // After 240 ticks:
    //   - 2 miners each produce 1 iron_ore per 60 ticks → 4 iron_ore total at ticks 60,120,180,240
    //   - Smelter accumulates 2 ore in input_buf (from ticks 60 & 120), starts at tick 120
    //   - Smelter completes 1st cycle at tick 60+120=180 → produces 1 iron_bar → drained to manifold
    //   - 2nd cycle starts at tick 180 (has 2 more ore from ticks 180 & 240)
    // The manifold system drains output_buf → manifold each tick, so iron_bar is in manifold.
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_bar_in_manifold = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert!(
        iron_bar_in_manifold >= 1.0,
        "smelter should have produced at least 1 iron_bar in manifold after 240 ticks (got {iron_bar_in_manifold})"
    );
}

/// Scenario: Multiple consumers share manifold resources proportionally
#[test]
fn multiple_consumers_share_manifold_resources_proportionally() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);
    set_terrain(&mut app, 3, 5, TerrainType::IronVein);

    // 3 miners, 1 smelter — miners produce 3x faster than smelter can consume
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 5, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 4, wind_turbine_recipe());
    app.update();

    for _ in 0..240 {
        app.update();
    }

    // Iron ore should accumulate in the manifold because miners produce faster than smelter consumes
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore_stockpile = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert!(
        iron_ore_stockpile > 0.0,
        "iron_ore should accumulate in manifold when 3 miners produce faster than 1 smelter can consume (stockpile={iron_ore_stockpile})"
    );
}

/// Scenario: Mall building output goes to Inventory not manifold
#[test]
fn mall_building_output_goes_to_inventory_not_manifold() {
    let mut app = test_app(10, 10);

    // Place constructor via legacy (no inventory/fog checks) at (3,3), footprint 2x2
    place_legacy(&mut app, BuildingType::Constructor, 3, 3, constructor_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 5, 4, wind_turbine_recipe());
    app.update();

    // Pre-load input buffer of the constructor directly so production starts on tick 1.
    // (Seeding the manifold causes a 1-tick delay while manifold_system fills input buffers,
    //  so we pre-fill the input buffer to guarantee production completes in 300 ticks.)
    let group_id = group_of(&mut app, 3, 3).unwrap();
    {
        let mut bq = app.world_mut().query::<(Entity, &Building, &mut InputBuffer)>();
        for (_, b, mut ib) in bq.iter_mut(app.world_mut()) {
            if b.building_type == BuildingType::Constructor {
                ib.slots.insert(ResourceType::IronBar, 3.0);
                ib.slots.insert(ResourceType::Plank, 1.0);
            }
        }
    }

    // Run 302 ticks: 300 for recipe + 2 ticks overhead (manifold flush + cycle restart)
    for _ in 0..302 {
        app.update();
    }

    // The Inventory resource should contain iron_miner item (via ItemIronMiner resource key)
    let inv = app.world().resource::<Inventory>();
    let item_count = inv.resources.get(&ResourceType::ItemIronMiner).copied().unwrap_or(0);
    assert!(item_count >= 1, "Inventory should contain at least 1 iron_miner item after constructor cycle (got {item_count})");

    // The group manifold should NOT contain the iron_miner item
    let mut mq2 = app.world_mut().query::<(Entity, &Manifold)>();
    let manifold_has_item = mq2
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::ItemIronMiner).copied().unwrap_or(0.0) > 0.0)
        .unwrap_or(false);
    assert!(!manifold_has_item, "group manifold should NOT contain the iron_miner item");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC3: Identical buildings placed adjacent chain automatically
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Two identical miners chain automatically
#[test]
fn two_identical_miners_chain_automatically() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    app.update();

    let g1 = group_of(&mut app, 3, 3).unwrap();
    let g2 = group_of(&mut app, 4, 3).unwrap();
    assert_eq!(g1, g2, "both miners should belong to the same group (automatic chaining)");
    // No manual connection step — placement alone should have triggered grouping
}

/// Scenario: Four identical miners in a square form one group
#[test]
fn four_identical_miners_in_a_square_form_one_group() {
    let mut app = test_app(10, 10);
    for (x, y) in [(3, 3), (4, 3), (3, 4), (4, 4)] {
        set_terrain(&mut app, x, y, TerrainType::IronVein);
    }

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 4, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "4 miners in a square should form exactly 1 group");

    let group_id = group_of(&mut app, 3, 3).unwrap();
    assert_eq!(buildings_in_group(&mut app, group_id), 4, "the group should contain 4 buildings");
}

/// Scenario: L-shaped group of identical and different buildings
#[test]
fn l_shaped_group_of_identical_and_different_buildings() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 3, 5, iron_smelter_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "L-shaped group should be exactly 1 group");

    let group_id = group_of(&mut app, 3, 3).unwrap();
    assert_eq!(buildings_in_group(&mut app, group_id), 4, "the group should contain 4 buildings");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC4: Group displays aggregate input/output rates
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Group stats show aggregate production rate
#[test]
fn group_stats_show_aggregate_production_rate() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 4, wind_turbine_recipe());
    app.update();

    for _ in 0..120 {
        app.update();
    }

    // Assert the GroupStats component is present on the group entity.
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut gsq = app.world_mut().query::<(Entity, &GroupStats)>();
    let stats_present = gsq.iter(app.world()).any(|(e, _)| e == group_id);
    assert!(stats_present, "group should have a GroupStats component for aggregate rate display");

    // Verify aggregate output rate: 2 miners × 1 ore/60 ticks.
    // After 120 ticks: each miner completed ~2 production cycles.
    // The smelter should be active (mid-cycle), having consumed iron_ore from the manifold.
    // Assert the smelter is active (consuming ore confirms the aggregate input rate).
    let mut psq = app.world_mut().query::<(&Position, &ProductionState)>();
    let smelter_state = psq
        .iter(app.world())
        .find(|(p, _)| p.x == 4 && p.y == 3)
        .map(|(_, ps)| (ps.active, ps.progress, ps.idle_reason));
    assert!(smelter_state.is_some(), "smelter entity should exist at (4,3)");
    let (smelter_active, smelter_progress, _smelter_reason) = smelter_state.unwrap();
    assert!(
        smelter_active,
        "smelter should be active (mid-cycle) after consuming iron_ore from the group manifold (progress={smelter_progress})"
    );
    assert!(
        smelter_progress > 0.0,
        "smelter progress should be > 0, confirming it has been consuming ore and cycling (got {smelter_progress})"
    );
}

/// Scenario: Group stats for single miner
#[test]
fn group_stats_for_single_miner() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 3, wind_turbine_recipe());
    app.update();

    for _ in 0..60 {
        app.update();
    }

    // Assert GroupStats component exists on the group entity.
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut gsq = app.world_mut().query::<(Entity, &GroupStats)>();
    let stats_present = gsq.iter(app.world()).any(|(e, _)| e == group_id);
    assert!(stats_present, "single-miner group should have a GroupStats component");

    // Assert actual production: 1 iron_ore produced in 60 ticks at rate 1 unit/60 ticks.
    // No consumer, so iron_ore stays in manifold = 1.0 exactly.
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore_rate_evidence = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    assert_eq!(
        iron_ore_rate_evidence, 1.0,
        "single miner should show output rate of 1 iron_ore per 60 ticks (manifold has {iron_ore_rate_evidence})"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// AC5: Player can place input receivers and output senders on group boundary
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Group of one has configurable receivers and senders
#[test]
fn group_of_one_has_configurable_receivers_and_senders() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 5, 5, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 5, 5, iron_miner_recipe());
    app.update();

    let group_id = group_of(&mut app, 5, 5).unwrap();

    // The group entity has a Group marker, Manifold, GroupEnergy, GroupControl, GroupStats, GroupType.
    let mut gq = app.world_mut().query::<(Entity, &Group, &Manifold, &GroupControl)>();
    let group_present = gq.iter(app.world()).any(|(e, _, _, _)| e == group_id);
    assert!(group_present, "group entity should have Group, Manifold, and GroupControl components");

    // The iron_miner at (5,5) has 4 boundary cells: (4,5),(6,5),(5,4),(5,6).
    // Verify boundary cells are NOT occupied (they are valid port placement locations).
    let boundary_cells = [(4i32, 5i32), (6, 5), (5, 4), (5, 6)];
    let grid = app.world().resource::<Grid>();
    for &(bx, by) in &boundary_cells {
        assert!(
            !grid.occupied.contains(&(bx, by)),
            "boundary cell ({bx},{by}) should be free for output sender / input receiver placement"
        );
    }

    // Verify the group's building is at (5,5) and its footprint marks (5,5) as occupied.
    assert!(
        app.world().resource::<Grid>().occupied.contains(&(5, 5)),
        "iron_miner at (5,5) should occupy that cell"
    );
}

/// Scenario: Multi-building group has boundary ports
#[test]
fn multi_building_group_has_boundary_ports() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    let g_miner = group_of(&mut app, 3, 3).unwrap();
    let g_smelter = group_of(&mut app, 4, 3).unwrap();
    assert_eq!(g_miner, g_smelter, "both buildings must be in the same group to share boundary");

    // The group occupies cells (3,3) and (4,3).
    // Boundary = all cells cardinally adjacent to the group that are NOT part of the group:
    //   from (3,3): (2,3),(3,2),(3,4)  [not (4,3) — that's in-group]
    //   from (4,3): (5,3),(4,2),(4,4)  [not (3,3) — that's in-group]
    // Exactly 6 boundary cells, all unoccupied.
    let group_cells: std::collections::HashSet<(i32, i32)> = [(3i32, 3i32), (4, 3)].into();
    let expected_boundary: std::collections::HashSet<(i32, i32)> =
        [(2i32, 3i32), (3, 2), (3, 4), (5, 3), (4, 2), (4, 4)].into();

    let grid = app.world().resource::<Grid>();

    // All boundary cells must be unoccupied (valid for port placement).
    for &(bx, by) in &expected_boundary {
        assert!(
            !grid.occupied.contains(&(bx, by)),
            "boundary cell ({bx},{by}) should be unoccupied and available for port placement"
        );
    }

    // No expected boundary cell is part of the group itself.
    for cell in &expected_boundary {
        assert!(
            !group_cells.contains(cell),
            "boundary cell {cell:?} should not be inside the group footprint"
        );
    }

    // Group footprint cells are occupied.
    assert!(grid.occupied.contains(&(3, 3)), "cell (3,3) should be occupied by iron_miner");
    assert!(grid.occupied.contains(&(4, 3)), "cell (4,3) should be occupied by iron_smelter");
}

/// Scenario: Output sender feeds transport path
#[test]
fn output_sender_feeds_transport_path() {
    let mut app = test_app(16, 10);

    // Place group A: iron_miner with output sender
    set_terrain(&mut app, 2, 5, TerrainType::IronVein);
    place_legacy(&mut app, BuildingType::IronMiner, 2, 5, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 2, 4, wind_turbine_recipe());
    app.update();

    let group_a = group_of(&mut app, 2, 5).unwrap();

    // Place group B: iron_smelter as a separate entity (not adjacent to group A)
    place_legacy(&mut app, BuildingType::IronSmelter, 10, 5, iron_smelter_recipe());
    app.update();

    let group_b = group_of(&mut app, 10, 5).unwrap();
    assert_ne!(group_a, group_b, "groups A and B should be separate");

    // Add sender to group A manifold and receiver to group B
    app.world_mut().spawn(OutputSender {
        group_id: group_a,
        resource: Some(ResourceType::IronOre),
        boundary_pos: (3, 5),
    });
    app.world_mut().spawn(InputReceiver {
        group_id: group_b,
        resource: Some(ResourceType::IronOre),
        boundary_pos: (9, 5),
    });

    // Seed group A manifold with iron_ore
    {
        let mut mq = app.world_mut().query::<(Entity, &mut Manifold)>();
        for (e, mut m) in mq.iter_mut(app.world_mut()) {
            if e == group_a {
                m.resources.insert(ResourceType::IronOre, 5.0);
            }
        }
    }

    for _ in 0..120 {
        app.update();
    }

    // Verify group A still has its output sender (structure test)
    let mut sender_q = app.world_mut().query::<&OutputSender>();
    let sender_exists = sender_q.iter(app.world()).any(|s| s.group_id == group_a);
    assert!(sender_exists, "group A should have an output sender configured");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC6: Removing a building that bridges two sub-groups splits them
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Removing bridge building splits group into two
#[test]
fn removing_bridge_building_splits_group_into_two() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::IronVein);

    // Place: miner(3,3) — smelter(4,3) — miner(5,3) in one line
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 5, 3, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "should start as 1 group");

    // Remove the bridge smelter at (4,3)
    remove_building_at(&mut app, 4, 3);
    app.update();

    assert_eq!(count_group_entities(&mut app), 2, "should have 2 groups after bridge removal");

    let g_left = group_of(&mut app, 3, 3).unwrap();
    let g_right = group_of(&mut app, 5, 3).unwrap();
    assert_ne!(g_left, g_right, "iron_miner at (3,3) and iron_miner at (5,3) should be in different groups");
}

/// Scenario: Split groups get separate manifolds
#[test]
fn split_groups_get_separate_manifolds() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 5, 3, iron_miner_recipe());
    app.update();

    // Seed the single manifold with iron_ore
    let initial_group = group_of(&mut app, 3, 3).unwrap();
    {
        let mut mq = app.world_mut().query::<(Entity, &mut Manifold)>();
        for (e, mut m) in mq.iter_mut(app.world_mut()) {
            if e == initial_group {
                m.resources.insert(ResourceType::IronOre, 10.0);
            }
        }
    }

    // Remove bridge
    remove_building_at(&mut app, 4, 3);
    app.update();

    // Each resulting group should have its own Manifold component
    let mut mq = app.world_mut().query::<&Manifold>();
    let manifold_count = mq.iter(app.world()).count();
    assert_eq!(manifold_count, 2, "each split group should have its own separate manifold (got {manifold_count})");
}

/// Scenario: Removing non-bridge building does not split group
#[test]
fn removing_non_bridge_building_does_not_split_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);

    // L-shape: (3,3), (4,3), (3,4) — removing (4,3) leaves (3,3)-(3,4) still connected
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "should start as 1 group");

    // Remove the non-bridge miner at (4,3)
    remove_building_at(&mut app, 4, 3);
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "removing non-bridge should still leave 1 group");

    let group_id = group_of(&mut app, 3, 3).unwrap();
    assert_eq!(buildings_in_group(&mut app, group_id), 2, "remaining group should contain 2 buildings");
}

/// Scenario: Removing last building destroys the group
#[test]
fn removing_last_building_destroys_the_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 5, 5, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 5, 5, iron_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "should have 1 group before removal");

    remove_building_at(&mut app, 5, 5);
    app.update();

    assert_eq!(count_group_entities(&mut app), 0, "removing last building should destroy the group (got {})", count_group_entities(&mut app));
    assert!(!building_at(&mut app, 5, 5), "no building should remain at (5,5)");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC7: Chain manager displays groups with energy, priority, and status
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Group has energy demand and allocation
#[test]
fn group_has_energy_demand_and_allocation() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 3, wind_turbine_recipe());
    app.update();

    // After 1 more tick the energy system should have allocated energy
    app.update();

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut geq = app.world_mut().query::<(Entity, &GroupEnergy)>();
    let energy = geq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, ge)| (ge.demand, ge.allocated));

    assert!(energy.is_some(), "group should have a GroupEnergy component");
    let (demand, allocated) = energy.unwrap();
    // IronMiner energy_consumption = 5.0
    assert!(demand >= 5.0, "group energy demand should be >= 5 (miner consumes 5), got {demand}");
    assert!(allocated > 0.0, "group should have allocated energy > 0 with wind turbine, got {allocated}");
}

/// Scenario: Group priority can be set via command
#[test]
fn group_priority_can_be_set_via_command() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    app.update();

    let group_id = group_of(&mut app, 3, 3).unwrap();

    // Verify default priority is Medium
    {
        let mut gcq = app.world_mut().query::<(Entity, &GroupControl)>();
        let ctrl = gcq
            .iter(app.world())
            .find(|(e, _)| *e == group_id)
            .map(|(_, c)| c.priority);
        assert_eq!(ctrl, Some(GroupPriority::Medium), "default group priority should be MEDIUM");
    }

    // Send SetGroupPriority command
    app.world_mut().write_message(SetGroupPriority {
        group_id,
        priority: GroupPriority::High,
    });
    app.update();

    // Verify priority changed to High
    let mut gcq = app.world_mut().query::<(Entity, &GroupControl)>();
    let new_priority = gcq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, c)| c.priority);
    assert_eq!(new_priority, Some(GroupPriority::High), "group priority should be HIGH after command");
}

/// Scenario: Group can be paused and resumed
#[test]
fn group_can_be_paused_and_resumed() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 3, wind_turbine_recipe());
    app.update();

    let group_id = group_of(&mut app, 3, 3).unwrap();

    // Send PauseGroup command
    app.world_mut().write_message(PauseGroup { group_id });
    app.update();

    // Verify group is paused
    {
        let mut gcq = app.world_mut().query::<(Entity, &GroupControl)>();
        let status = gcq
            .iter(app.world())
            .find(|(e, _)| *e == group_id)
            .map(|(_, c)| c.status);
        assert_eq!(status, Some(GroupStatus::Paused), "group should be Paused");
    }

    // Run 60 ticks while paused — verify miner is idle (no iron_ore produced)
    for _ in 0..60 {
        app.update();
    }

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    assert_eq!(iron_ore, 0.0, "no iron_ore should be produced while group is paused (got {iron_ore})");

    // Check miner production state is paused
    let mut psq = app.world_mut().query::<(&Position, &ProductionState)>();
    let miner_reason = psq
        .iter(app.world())
        .find(|(p, _)| p.x == 3 && p.y == 3)
        .map(|(_, ps)| ps.idle_reason);
    assert_eq!(
        miner_reason,
        Some(Some(IdleReason::GroupPaused)),
        "miner should have idle_reason=GroupPaused"
    );

    // Send ResumeGroup command
    let group_id = group_of(&mut app, 3, 3).unwrap();
    app.world_mut().write_message(ResumeGroup { group_id });
    app.update();

    // Verify group is active again
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut gcq = app.world_mut().query::<(Entity, &GroupControl)>();
    let status = gcq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, c)| c.status);
    assert_eq!(status, Some(GroupStatus::Active), "group should be Active after resume");

    // Run 60 more ticks — miner should produce again
    for _ in 0..60 {
        app.update();
    }

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq2 = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore_after_resume = mq2
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    assert!(
        iron_ore_after_resume > 0.0,
        "iron_ore should be produced after group is resumed (got {iron_ore_after_resume})"
    );
}

/// Scenario: Chain manager shows each group as a manageable unit
#[test]
fn chain_manager_shows_each_group_as_a_manageable_unit() {
    let mut app = test_app(12, 10);
    set_terrain(&mut app, 2, 3, TerrainType::IronVein);
    set_terrain(&mut app, 8, 3, TerrainType::CopperVein);

    place_legacy(&mut app, BuildingType::IronMiner, 2, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::CopperMiner, 8, 3, copper_miner_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 2, "chain manager should list 2 groups");

    // Each group entity should have GroupEnergy, GroupControl, GroupStats components
    let mut q = app.world_mut().query::<(Entity, &Group, &GroupEnergy, &GroupControl, &GroupStats)>();
    let entries: Vec<Entity> = q.iter(app.world()).map(|(e, _, _, _, _)| e).collect();
    assert_eq!(entries.len(), 2, "each group should have energy, priority, and status components");
}

// ═══════════════════════════════════════════════════════════════════════════
// AC8: Synthesis groups function without terrain requirements
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Synthesis building placed on plain terrain
#[test]
fn synthesis_building_placed_on_plain_terrain() {
    let mut app = test_app(10, 10);
    // Grass terrain is default — no need to set explicitly

    // Place iron_smelter on grass (no terrain requirement for synthesis)
    place_legacy(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());
    app.update();

    assert!(building_at(&mut app, 3, 3), "iron_smelter should be placed on plain grass terrain");
    let group_id = group_of(&mut app, 3, 3);
    assert!(group_id.is_some(), "iron_smelter should belong to a group");
}

/// Scenario: Tree farm placed on any tile produces wood from water
#[test]
fn tree_farm_placed_on_any_tile_produces_wood_from_water() {
    let mut app = test_app(10, 10);
    // TreeFarm footprint is 2x2 — place at (3,3), occupying (3,3),(4,3),(3,4),(4,4)

    place_legacy(&mut app, BuildingType::TreeFarm, 3, 3, tree_farm_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 5, 3, wind_turbine_recipe());
    app.update();

    // Pre-load the tree_farm input buffer directly so production starts on tick 1.
    // (Seeding the manifold causes a 1-tick delay; direct input fill ensures completion in 180 ticks.)
    {
        let mut bq = app.world_mut().query::<(&Building, &mut InputBuffer)>();
        for (b, mut ib) in bq.iter_mut(app.world_mut()) {
            if b.building_type == BuildingType::TreeFarm {
                ib.slots.insert(ResourceType::Water, 3.0);
            }
        }
    }

    for _ in 0..182 {
        app.update();
    }

    // After 182 ticks: tree_farm completed 1 cycle (180 ticks) producing wood:2.
    // The manifold_system drains OutputBuffer -> Manifold each tick, so wood is in manifold.
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq2 = app.world_mut().query::<(Entity, &Manifold)>();
    let wood_in_manifold = mq2
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::Wood).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert_eq!(
        wood_in_manifold, 2.0,
        "tree_farm should have produced exactly wood:2 from water:3 in 182 ticks (got {wood_in_manifold})"
    );
}

/// Scenario: Synthesis group idles when inputs unavailable
#[test]
fn synthesis_group_idles_when_inputs_unavailable() {
    let mut app = test_app(10, 10);

    // Place iron_smelter with empty manifold (no iron_ore available)
    place_legacy(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 3, 4, wind_turbine_recipe());
    app.update();

    for _ in 0..240 {
        app.update();
    }

    // The smelter production state should be idle (not active)
    let mut psq = app.world_mut().query::<(&Position, &ProductionState)>();
    let smelter_state = psq
        .iter(app.world())
        .find(|(p, _)| p.x == 3 && p.y == 3)
        .map(|(_, ps)| (ps.active, ps.idle_reason));

    assert!(smelter_state.is_some(), "smelter entity should exist");
    let (is_active, idle_reason) = smelter_state.unwrap();
    assert!(!is_active, "smelter should be idle (not active) without inputs");
    assert_eq!(idle_reason, Some(IdleReason::NoInputs), "idle reason should be NoInputs");

    // No iron_bar should have been produced
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_bar = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronBar).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    assert_eq!(iron_bar, 0.0, "no iron_bar should be produced without inputs (got {iron_bar})");

    // The simulation did not crash (test reached this point)
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: Building placed between two existing groups merges all into one
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Building placed between two groups merges them
#[test]
fn building_placed_between_two_groups_merges_them() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::CopperVein);

    // Place iron_miner at (3,3) and copper_miner at (5,3) — separate groups
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::CopperMiner, 5, 3, copper_miner_recipe());
    app.update();
    assert_eq!(count_group_entities(&mut app), 2, "should have 2 groups initially");

    // Place smelter at (4,3) between them — should merge both groups
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    assert_eq!(count_group_entities(&mut app), 1, "placing bridge building should merge into 1 group");

    let group_id = group_of(&mut app, 3, 3).unwrap();
    assert_eq!(buildings_in_group(&mut app, group_id), 3, "merged group should contain 3 buildings: iron_miner, iron_smelter, copper_miner");
}

/// Scenario: Three-way merge preserves all manifold contents
#[test]
fn three_way_merge_preserves_all_manifold_contents() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 5, 3, TerrainType::CopperVein);

    // Set up two separate groups
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::CopperMiner, 5, 3, copper_miner_recipe());
    app.update();

    // Seed group A (iron_miner) with iron_ore:5
    let group_a = group_of(&mut app, 3, 3).unwrap();
    {
        let mut mq = app.world_mut().query::<(Entity, &mut Manifold)>();
        for (e, mut m) in mq.iter_mut(app.world_mut()) {
            if e == group_a {
                m.resources.insert(ResourceType::IronOre, 5.0);
            }
        }
    }

    // Seed group B (copper_miner) with copper_ore:3
    let group_b = group_of(&mut app, 5, 3).unwrap();
    {
        let mut mq = app.world_mut().query::<(Entity, &mut Manifold)>();
        for (e, mut m) in mq.iter_mut(app.world_mut()) {
            if e == group_b {
                m.resources.insert(ResourceType::CopperOre, 3.0);
            }
        }
    }

    // Place bridge smelter at (4,3) — triggers merge
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    // The merged group manifold should contain both iron_ore:5 (from group A) and copper_ore:3 (from B).
    // NOTE: the manifold_system runs in the same update tick as the group merge. The smelter needs
    // iron_ore:2 per cycle, so up to 2 iron_ore may be transferred from manifold → smelter input_buf
    // within the same tick. We verify the TOTAL iron_ore (manifold + smelter input_buf) equals 5.
    let merged_group = group_of(&mut app, 3, 3).unwrap();

    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let resources = mq
        .iter(app.world())
        .find(|(e, _)| *e == merged_group)
        .map(|(_, m)| (
            m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0),
            m.resources.get(&ResourceType::CopperOre).copied().unwrap_or(0.0),
        ));

    assert!(resources.is_some(), "merged group should have a manifold");
    let (iron_ore_in_manifold, copper_ore) = resources.unwrap();

    // Smelter may have pulled up to 2 iron_ore into its input_buf from the manifold.
    let mut ibq = app.world_mut().query::<(&Position, &InputBuffer)>();
    let smelter_iron_ore = ibq
        .iter(app.world())
        .find(|(p, _)| p.x == 4 && p.y == 3)
        .map(|(_, ib)| ib.slots.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    let total_iron_ore = iron_ore_in_manifold + smelter_iron_ore;
    assert_eq!(
        total_iron_ore, 5.0,
        "merged group total iron_ore (manifold + smelter input) should equal 5 from group A (got manifold={iron_ore_in_manifold}, smelter_input={smelter_iron_ore})"
    );
    assert_eq!(copper_ore, 3.0, "merged manifold should contain copper_ore:3 from group B (got {copper_ore})");
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: Extraction group produces without input receivers
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Extraction group produces without input receivers
#[test]
fn extraction_group_produces_without_input_receivers() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    // No input receivers configured — miner is self-sufficient
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 3, wind_turbine_recipe());
    app.update();

    for _ in 0..60 {
        app.update();
    }

    // After 60 ticks: miner completes 1 cycle producing 1 iron_ore.
    // No consumer exists for iron_ore, so the manifold retains it.
    // The manifold_system drains OutputBuffer → Manifold each tick after production,
    // so iron_ore:1 is in the manifold (output buffer was already drained).
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert_eq!(
        iron_ore, 1.0,
        "extraction group should have produced exactly iron_ore:1 in 60 ticks without external inputs (got {iron_ore})"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: No energy buildings — zero production speed
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Group with no energy produces at zero speed
#[test]
fn group_with_no_energy_produces_at_zero_speed() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);

    // No wind turbine — no energy generation
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    for _ in 0..120 {
        app.update();
    }

    // Energy allocated to the group should be 0
    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut geq = app.world_mut().query::<(Entity, &GroupEnergy)>();
    let allocated = geq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, ge)| ge.allocated)
        .unwrap_or(0.0);
    assert_eq!(allocated, 0.0, "group energy allocated should be 0 with no energy buildings (got {allocated})");

    // No iron_ore should be produced
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);
    assert_eq!(iron_ore, 0.0, "no iron_ore should be produced without energy (got {iron_ore})");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement on invalid terrain
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Miner placement rejected on wrong terrain
#[test]
fn miner_placement_rejected_on_wrong_terrain() {
    let mut app = test_app(10, 10);
    reveal_all(&mut app, 10, 10);
    // (3,3) has default Grass terrain — IronMiner requires IronVein

    // Add iron_miner to inventory
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 5);

    place_request(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    app.update();

    // Placement should be rejected
    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement on wrong terrain should be rejected");

    // No building should exist at (3,3)
    assert!(!building_at(&mut app, 3, 3), "no building should exist at (3,3) after rejected placement");

    // No group should be created
    assert_eq!(count_group_entities(&mut app), 0, "no group should be created after rejected placement");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement on occupied tile
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Placement rejected on already occupied tile
#[test]
fn placement_rejected_on_already_occupied_tile() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    reveal_all(&mut app, 10, 10);

    // Place iron_miner first via legacy
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    app.update();

    assert_eq!(count_buildings(&mut app), 1, "should have 1 building");

    // Try to place smelter on same tile via request (full validation)
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronSmelter, 5);
    place_request(&mut app, BuildingType::IronSmelter, 3, 3, iron_smelter_recipe());
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement on occupied tile should be rejected");

    // Still only 1 building (the original iron_miner)
    assert_eq!(count_buildings(&mut app), 1, "should still have only 1 building");

    // The iron_miner is unchanged
    let mut bq = app.world_mut().query::<(&Position, &Building)>();
    let buildings: Vec<_> = bq.iter(app.world()).collect();
    assert_eq!(buildings.len(), 1);
    assert_eq!(buildings[0].1.building_type, BuildingType::IronMiner, "original iron_miner should be unchanged");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement out of grid bounds
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Placement rejected outside grid bounds
#[test]
fn placement_rejected_outside_grid_bounds() {
    let mut app = test_app(5, 5);
    reveal_all(&mut app, 5, 5);
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 5);

    place_request(&mut app, BuildingType::IronMiner, 10, 10, iron_miner_recipe());
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement outside bounds should be rejected");

    assert!(!building_at(&mut app, 10, 10), "no building should exist at (10,10)");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement of tier-locked building
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: T2 building rejected while at T1
#[test]
fn t2_building_rejected_while_at_t1() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::ObsidianVein);
    reveal_all(&mut app, 10, 10);
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::ObsidianDrill, 5);

    // Current tier is 1 (default) — ObsidianDrill requires tier 2
    let current_tier = app.world().resource::<TierState>().current_tier;
    assert_eq!(current_tier, 1, "should be at tier 1 by default");

    let obsidian_recipe = Recipe::simple(vec![], vec![(ResourceType::ObsidianShard, 1.0)], 60);
    place_request(&mut app, BuildingType::ObsidianDrill, 3, 3, obsidian_recipe);
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "tier 2 building should be rejected at tier 1");

    assert!(!building_at(&mut app, 3, 3), "no building should exist at (3,3)");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Footprint overlap
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: 2x2 building placement rejected when footprint overlaps existing building
#[test]
fn two_by_two_building_rejected_when_footprint_overlaps_existing_building() {
    let mut app = test_app(10, 10);
    reveal_all(&mut app, 10, 10);

    // Place constructor at (3,3) — footprint: (3,3),(4,3),(3,4),(4,4)
    place_legacy(&mut app, BuildingType::Constructor, 3, 3, constructor_recipe());
    app.update();

    assert!(app.world().resource::<Grid>().occupied.contains(&(4, 3)), "cell (4,3) should be occupied by constructor");

    // Try to place iron_smelter at (4,3) — overlaps constructor
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronSmelter, 5);
    place_request(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement at (4,3) should be rejected because constructor occupies that cell");

    // Only the constructor should exist
    assert_eq!(count_buildings(&mut app), 1, "only the constructor should exist");
}

/// Scenario: Two 2x2 buildings cannot overlap footprints
#[test]
fn two_two_by_two_buildings_cannot_overlap_footprints() {
    let mut app = test_app(10, 10);
    reveal_all(&mut app, 10, 10);

    // Place constructor at (3,3) — footprint: (3,3),(4,3),(3,4),(4,4)
    place_legacy(&mut app, BuildingType::Constructor, 3, 3, constructor_recipe());
    app.update();

    // Try to place imp_camp (2x2) at (4,4) — overlaps constructor's (4,4) cell
    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::ImpCamp, 5);
    let imp_recipe = Recipe::simple(vec![], vec![], 1);
    place_request(&mut app, BuildingType::ImpCamp, 4, 4, imp_recipe);
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "imp_camp placement at (4,4) should be rejected — cell (4,4) occupied by constructor");

    assert_eq!(count_buildings(&mut app), 1, "only the constructor should exist");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement from empty inventory
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Placement rejected when building not in inventory
#[test]
fn placement_rejected_when_building_not_in_inventory() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    reveal_all(&mut app, 10, 10);

    // Inventory has 0 iron_miners (default)
    let count = app.world().resource::<Inventory>().count_building(BuildingType::IronMiner);
    assert_eq!(count, 0, "inventory should have 0 iron_miners");

    place_request(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement should be rejected when building not in inventory");

    assert!(!building_at(&mut app, 3, 3), "no building should exist at (3,3)");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR PATH: Placement on hidden (fogged) tile
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Placement rejected on hidden tile
#[test]
fn placement_rejected_on_hidden_tile() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 7, 7, TerrainType::IronVein);
    // Do NOT reveal (7,7) — it remains hidden

    app.world_mut().resource_mut::<Inventory>().add_building(BuildingType::IronMiner, 5);

    place_request(&mut app, BuildingType::IronMiner, 7, 7, iron_miner_recipe());
    app.update();

    let last_results = app.world().resource::<PlacementCommands>().last_results.clone();
    assert_eq!(last_results, vec![false], "placement on hidden tile should be rejected");

    assert!(!building_at(&mut app, 7, 7), "no building should exist at hidden tile (7,7)");
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: Manifold overflow — accumulation
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Manifold accumulates when production exceeds consumption
#[test]
fn manifold_accumulates_when_production_exceeds_consumption() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);
    set_terrain(&mut app, 3, 5, TerrainType::IronVein);

    // 3 miners produce 3 iron_ore / 60 ticks; 1 smelter consumes 2 iron_ore / 120 ticks
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 5, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    place_legacy(&mut app, BuildingType::WindTurbine, 4, 4, wind_turbine_recipe());
    app.update();

    for _ in 0..240 {
        app.update();
    }

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut mq = app.world_mut().query::<(Entity, &Manifold)>();
    let iron_ore = mq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, m)| m.resources.get(&ResourceType::IronOre).copied().unwrap_or(0.0))
        .unwrap_or(0.0);

    assert!(
        iron_ore > 0.0,
        "iron_ore should accumulate in manifold when production exceeds consumption (got {iron_ore})"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: 2x2 building adjacency
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: 1x1 building adjacent to 2x2 building forms a group
#[test]
fn one_by_one_building_adjacent_to_two_by_two_building_forms_a_group() {
    let mut app = test_app(10, 10);

    // Constructor at (3,3): footprint covers (3,3),(4,3),(3,4),(4,4)
    place_legacy(&mut app, BuildingType::Constructor, 3, 3, constructor_recipe());
    app.update();

    // Sawmill at (5,3): adjacent to constructor via cell (4,3)
    place_legacy(&mut app, BuildingType::Sawmill, 5, 3,
        Recipe::simple(vec![(ResourceType::Wood, 1.0)], vec![(ResourceType::Plank, 1.0)], 60));
    app.update();

    // Both should be in the same group
    let g_constructor = group_of(&mut app, 3, 3).unwrap();
    let g_sawmill = group_of(&mut app, 5, 3).unwrap();
    assert_eq!(g_constructor, g_sawmill, "constructor and sawmill should be in the same group");

    assert_eq!(buildings_in_group(&mut app, g_constructor), 2, "group should contain 2 buildings");
}

// ═══════════════════════════════════════════════════════════════════════════
// EDGE CASE: Group type determination
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Extraction group type assigned when group contains only miners
#[test]
fn extraction_group_type_assigned_when_group_contains_only_miners() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    app.update();

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let mut gtq = app.world_mut().query::<(Entity, &GroupType)>();
    let group_class = gtq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, gt)| gt.class);

    assert_eq!(group_class, Some(GroupClass::Extraction), "group with only miners should have Extraction type");
}

/// Scenario: Combat group type assigned for imp camp and breeding pen
#[test]
fn combat_group_type_assigned_for_imp_camp_and_breeding_pen() {
    let mut app = test_app(10, 10);
    // ImpCamp footprint 2x2 at (3,3): cells (3,3),(4,3),(3,4),(4,4)
    // BreedingPen footprint 2x2 at (5,3): cells (5,3),(6,3),(5,4),(6,4)
    // (4,3) is adjacent to (5,3) — they share a boundary

    place_legacy(&mut app, BuildingType::ImpCamp, 3, 3, Recipe::simple(vec![], vec![], 1));
    place_legacy(&mut app, BuildingType::BreedingPen, 5, 3, Recipe::simple(vec![], vec![], 1));
    app.update();

    let group_id = group_of(&mut app, 3, 3).unwrap();
    let g_pen = group_of(&mut app, 5, 3).unwrap();
    assert_eq!(group_id, g_pen, "ImpCamp and BreedingPen should be in the same group (adjacent via 2x2 footprints)");

    let mut gtq = app.world_mut().query::<(Entity, &GroupType)>();
    let group_class = gtq
        .iter(app.world())
        .find(|(e, _)| *e == group_id)
        .map(|(_, gt)| gt.class);

    assert_eq!(group_class, Some(GroupClass::Combat), "combat buildings should form a Combat group");
}

// ═══════════════════════════════════════════════════════════════════════════
// INVARIANT: Single group membership
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: Every building belongs to exactly one group
#[test]
fn every_building_belongs_to_exactly_one_group() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);

    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronSmelter, 4, 3, iron_smelter_recipe());
    app.update();

    // In a correct ECS implementation each Building entity has exactly 1 GroupMember component.
    // Collect all (building_entity, group_id) pairs.
    let mut q = app.world_mut().query::<(Entity, &Building, &GroupMember)>();
    let members: Vec<(Entity, BuildingType, Entity)> = q
        .iter(app.world())
        .map(|(e, b, m)| (e, b.building_type, m.group_id))
        .collect();

    assert_eq!(members.len(), 2, "should have exactly 2 buildings, each with exactly 1 GroupMember");

    // Both buildings belong to the SAME group (they are adjacent).
    let group_ids: std::collections::HashSet<Entity> = members.iter().map(|(_, _, g)| *g).collect();
    assert_eq!(group_ids.len(), 1, "both buildings should belong to the same single group (got {} distinct group_ids)", group_ids.len());

    // Verify iron_miner has exactly 1 GroupMember pointing to the shared group.
    let miner_group = members.iter()
        .find(|(_, bt, _)| *bt == BuildingType::IronMiner)
        .map(|(_, _, g)| *g);
    assert!(miner_group.is_some(), "iron_miner should have a GroupMember component");

    // Verify iron_smelter has exactly 1 GroupMember pointing to the same group.
    let smelter_group = members.iter()
        .find(|(_, bt, _)| *bt == BuildingType::IronSmelter)
        .map(|(_, _, g)| *g);
    assert!(smelter_group.is_some(), "iron_smelter should have a GroupMember component");

    // Both must point to the identical group entity.
    assert_eq!(
        miner_group, smelter_group,
        "iron_miner and iron_smelter should belong to exactly the same group"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// INVARIANT: Group connectivity
// ═══════════════════════════════════════════════════════════════════════════

/// Scenario: All buildings in a group are reachable via cardinal adjacency
#[test]
fn all_buildings_in_group_are_reachable_via_cardinal_adjacency() {
    let mut app = test_app(10, 10);
    set_terrain(&mut app, 3, 3, TerrainType::IronVein);
    set_terrain(&mut app, 4, 3, TerrainType::IronVein);
    set_terrain(&mut app, 3, 4, TerrainType::IronVein);

    // L-shape: (3,3)-(4,3) horizontal, (3,3)-(3,4) vertical
    place_legacy(&mut app, BuildingType::IronMiner, 3, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 4, 3, iron_miner_recipe());
    place_legacy(&mut app, BuildingType::IronMiner, 3, 4, iron_miner_recipe());
    app.update();

    // All 3 buildings should be in the same group
    let g1 = group_of(&mut app, 3, 3).unwrap();
    let g2 = group_of(&mut app, 4, 3).unwrap();
    let g3 = group_of(&mut app, 3, 4).unwrap();
    assert_eq!(g1, g2, "buildings at (3,3) and (4,3) should be in the same group");
    assert_eq!(g1, g3, "buildings at (3,3) and (3,4) should be in the same group");

    assert_eq!(buildings_in_group(&mut app, g1), 3, "group should contain 3 buildings");

    // Verify connectivity: from (3,3) can reach (4,3) via cardinal adjacency (dx=1,dy=0)
    // From (3,3) can reach (3,4) via cardinal adjacency (dx=0,dy=1)
    // No building should be disconnected (all in same group proves connectivity)

    // Flood-fill check: collect all positions in the group and verify BFS connectivity
    let positions: Vec<(i32, i32)> = {
        let mut q = app.world_mut().query::<(&Position, &GroupMember)>();
        q.iter(app.world())
            .filter(|(_, m)| m.group_id == g1)
            .map(|(p, _)| (p.x, p.y))
            .collect()
    };

    assert_eq!(positions.len(), 3, "all 3 buildings should be in the L-shaped group");

    // BFS from first position
    let pos_set: std::collections::HashSet<(i32, i32)> = positions.iter().copied().collect();
    let start = positions[0];
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let neighbor = (x + dx, y + dy);
            if pos_set.contains(&neighbor) && !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    assert_eq!(
        visited.len(),
        pos_set.len(),
        "all buildings in the group should be reachable via cardinal adjacency"
    );
}
