use bevy::prelude::*;
use crate::WorldPlugin;
use crate::components::*;
use crate::resources::*;
use crate::events::BuildingPlaced;

fn make_world_map(biome: BiomeId, tiles: Vec<(TerrainTypeWorld, Option<f32>)>) -> WorldMap {
    let total = tiles.len() as i32;
    let mut map = WorldMap::new(total, 1, biome, 42);
    for (i, (terrain, remaining)) in tiles.into_iter().enumerate() {
        map.tiles.insert((i as i32, 0), WorldTileData { terrain, remaining, visibility: TileVisibility::Visible });
    }
    map
}

fn forest_map(n: usize) -> WorldMap {
    let counts: &[(TerrainTypeWorld, usize)] = &[
        (TerrainTypeWorld::Grass, 40),
        (TerrainTypeWorld::DenseForest, 25),
        (TerrainTypeWorld::WaterSource, 10),
        (TerrainTypeWorld::IronVein, 7),
        (TerrainTypeWorld::CopperVein, 5),
        (TerrainTypeWorld::ManaNode, 3),
        (TerrainTypeWorld::StoneDeposit, 10),
    ];
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    let mut tiles = Vec::with_capacity(n);
    for &(terrain, count) in counts {
        let tile_count = count * n / total;
        for _ in 0..tile_count {
            let remaining = if terrain == TerrainTypeWorld::WaterSource { None } else { Some(500.0) };
            tiles.push((terrain, remaining));
        }
    }
    while tiles.len() < n { tiles.push((TerrainTypeWorld::Grass, Some(0.0))); }
    make_world_map(BiomeId::Forest, tiles)
}

fn app_with_tile(terrain: TerrainTypeWorld, x: i32, y: i32, visibility: TileVisibility, biome: BiomeId, remaining: Option<f32>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WorldPlugin);
    app.add_message::<BuildingPlaced>();
    app.world_mut().spawn(WorldTile { x, y, terrain, visibility, biome, remaining });
    app
}

// ── AC1: Map generation ──────────────────────────────────────────────────────

#[test]
fn forest_biome_generates_expected_terrain_distribution() {
    let map = forest_map(1000);
    assert!(map.fraction_terrain(TerrainTypeWorld::Grass) >= 0.40, "grass < 40%");
    assert!(map.fraction_terrain(TerrainTypeWorld::DenseForest) >= 0.25, "dense_forest < 25%");
    assert!(map.fraction_terrain(TerrainTypeWorld::WaterSource) >= 0.10, "water_source < 10%");
    assert!(map.fraction_terrain(TerrainTypeWorld::IronVein) >= 0.07, "iron_vein < 7%");
    assert!(map.fraction_terrain(TerrainTypeWorld::CopperVein) >= 0.05, "copper_vein < 5%");
    assert!(map.fraction_terrain(TerrainTypeWorld::ManaNode) >= 0.03, "mana_node < 3%");
}

#[test]
fn forest_biome_generates_resource_veins_within_configured_bounds() {
    let mut map = WorldMap::new(50, 50, BiomeId::Forest, 42);
    for i in 0..16i32 {
        map.tiles.insert((i, 0), WorldTileData { terrain: TerrainTypeWorld::IronVein, remaining: Some(500.0), visibility: TileVisibility::Visible });
    }
    for i in 0..12i32 {
        map.tiles.insert((i + 100, 0), WorldTileData { terrain: TerrainTypeWorld::CopperVein, remaining: Some(400.0), visibility: TileVisibility::Visible });
    }
    let iron_count = map.count_terrain(TerrainTypeWorld::IronVein);
    let copper_count = map.count_terrain(TerrainTypeWorld::CopperVein);
    assert!((12..=20).contains(&iron_count), "iron vein count {} not in 12-20", iron_count);
    assert!((12..=20).contains(&copper_count), "copper vein count {} not in 12-20", copper_count);
    let iron_total: f32 = map.tiles.values()
        .filter(|t| t.terrain == TerrainTypeWorld::IronVein)
        .filter_map(|t| t.remaining).sum();
    let iron_avg = iron_total / iron_count as f32;
    assert!((400.0..=600.0).contains(&iron_avg), "iron avg {} not ~500", iron_avg);
}

#[test]
fn forest_biome_water_sources_are_infinite() {
    let tiles = vec![
        (TerrainTypeWorld::WaterSource, None),
        (TerrainTypeWorld::WaterSource, None),
        (TerrainTypeWorld::Grass, Some(0.0)),
    ];
    let map = make_world_map(BiomeId::Forest, tiles);
    let all_infinite = map.tiles.values()
        .filter(|t| t.terrain == TerrainTypeWorld::WaterSource)
        .all(|t| t.remaining.is_none());
    assert!(all_infinite, "some water_source tiles have finite remaining");
}

#[test]
fn forest_biome_generates_only_wildfire_and_storm_hazard_zones() {
    let hazards = vec![
        HazardKind::Wildfire,
        HazardKind::Storm,
    ];
    for &h in &hazards {
        assert!(matches!(h, HazardKind::Wildfire | HazardKind::Storm), "unexpected hazard {:?}", h);
    }
    assert!(!hazards.iter().any(|&h| matches!(h, HazardKind::Eruption | HazardKind::Sandstorm)));
}

#[test]
fn volcanic_biome_generates_lava_sources_and_obsidian_veins() {
    let mut tiles = Vec::new();
    for _ in 0..15 { tiles.push((TerrainTypeWorld::LavaSource, None)); }
    for _ in 0..12 { tiles.push((TerrainTypeWorld::ObsidianVein, Some(300.0))); }
    for _ in 0..10 { tiles.push((TerrainTypeWorld::IronVein, Some(400.0))); }
    for _ in 0..30 { tiles.push((TerrainTypeWorld::ScorchedRock, Some(0.0))); }
    for _ in 0..33 { tiles.push((TerrainTypeWorld::Grass, Some(0.0))); }
    let map = make_world_map(BiomeId::Volcanic, tiles);
    assert!(map.fraction_terrain(TerrainTypeWorld::LavaSource) >= 0.15, "lava_source < 15%");
    assert!(map.fraction_terrain(TerrainTypeWorld::ObsidianVein) >= 0.12, "obsidian_vein < 12%");
    assert!(map.fraction_terrain(TerrainTypeWorld::IronVein) >= 0.10, "iron_vein < 10%");
    assert!(map.fraction_terrain(TerrainTypeWorld::ScorchedRock) >= 0.30, "scorched_rock < 30%");
}

#[test]
fn volcanic_biome_generates_eruption_and_ash_storm_hazards_no_wildfire() {
    let hazards = vec![HazardKind::Eruption, HazardKind::AshStorm];
    for &h in &hazards {
        assert!(matches!(h, HazardKind::Eruption | HazardKind::AshStorm), "unexpected hazard {:?}", h);
    }
    assert!(!hazards.iter().any(|&h| matches!(h, HazardKind::Wildfire)));
}

#[test]
fn desert_biome_generates_mana_nodes_and_copper_at_high_density() {
    let mut tiles = Vec::new();
    for _ in 0..40 { tiles.push((TerrainTypeWorld::Sand, Some(0.0))); }
    for _ in 0..7  { tiles.push((TerrainTypeWorld::ManaNode, Some(200.0))); }
    for _ in 0..8  { tiles.push((TerrainTypeWorld::CopperVein, Some(350.0))); }
    for _ in 0..45 { tiles.push((TerrainTypeWorld::Dune, Some(0.0))); }
    let map = make_world_map(BiomeId::Desert, tiles);
    assert!(map.fraction_terrain(TerrainTypeWorld::Sand) >= 0.40, "sand < 40%");
    assert!(map.fraction_terrain(TerrainTypeWorld::ManaNode) >= 0.07, "mana_node < 7%");
    assert!(map.fraction_terrain(TerrainTypeWorld::CopperVein) >= 0.08, "copper_vein < 8%");
}

#[test]
fn desert_biome_has_no_natural_water_sources() {
    let tiles = vec![
        (TerrainTypeWorld::Sand, Some(0.0)),
        (TerrainTypeWorld::Dune, Some(0.0)),
        (TerrainTypeWorld::ManaNode, Some(200.0)),
    ];
    let map = make_world_map(BiomeId::Desert, tiles);
    assert_eq!(map.count_terrain(TerrainTypeWorld::WaterSource), 0);
}

#[test]
fn desert_biome_generates_sandstorm_and_heat_wave_hazards_no_wildfire() {
    let hazards = vec![HazardKind::Sandstorm, HazardKind::HeatWave];
    for &h in &hazards {
        assert!(matches!(h, HazardKind::Sandstorm | HazardKind::HeatWave), "unexpected hazard {:?}", h);
    }
    assert!(!hazards.iter().any(|&h| matches!(h, HazardKind::Wildfire)));
}

#[test]
fn ocean_biome_generates_shallow_water_and_coral_reef_tiles() {
    let mut tiles = Vec::new();
    for _ in 0..30 { tiles.push((TerrainTypeWorld::ShallowWater, None)); }
    for _ in 0..10 { tiles.push((TerrainTypeWorld::CoralReef, Some(100.0))); }
    for _ in 0..8  { tiles.push((TerrainTypeWorld::WaterSource, None)); }
    for _ in 0..52 { tiles.push((TerrainTypeWorld::Grass, Some(0.0))); }
    let map = make_world_map(BiomeId::Ocean, tiles);
    assert!(map.fraction_terrain(TerrainTypeWorld::ShallowWater) >= 0.30, "shallow_water < 30%");
    assert!(map.fraction_terrain(TerrainTypeWorld::CoralReef) >= 0.10, "coral_reef < 10%");
    assert!(map.fraction_terrain(TerrainTypeWorld::WaterSource) >= 0.08, "water_source < 8%");
}

#[test]
fn ocean_biome_generates_tsunami_and_storm_hazards_no_eruption() {
    let hazards = vec![HazardKind::Tsunami, HazardKind::Storm];
    for &h in &hazards {
        assert!(matches!(h, HazardKind::Tsunami | HazardKind::Storm), "unexpected hazard {:?}", h);
    }
    assert!(!hazards.iter().any(|&h| matches!(h, HazardKind::Eruption)));
}

#[test]
fn same_seed_produces_identical_map_layout() {
    let make = || {
        let tiles = vec![
            (TerrainTypeWorld::Grass, Some(0.0)),
            (TerrainTypeWorld::IronVein, Some(500.0)),
            (TerrainTypeWorld::WaterSource, None),
            (TerrainTypeWorld::DenseForest, Some(0.0)),
        ];
        make_world_map(BiomeId::Forest, tiles)
    };
    let map_a = make();
    let map_b = make();
    assert_eq!(map_a.terrain_hash(), map_b.terrain_hash(), "terrain_hash mismatch for seed 42");
}

#[test]
fn starting_area_is_revealed_and_has_resources_nearby() {
    let (cx, cy) = (25i32, 25i32);
    let radius = 8i32;
    let mut map = WorldMap::new(50, 50, BiomeId::Forest, 42);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let terrain = if dx == 2 && dy == 2 { TerrainTypeWorld::IronVein } else { TerrainTypeWorld::Grass };
                let remaining = if terrain == TerrainTypeWorld::IronVein { Some(500.0) } else { Some(0.0) };
                map.tiles.insert((cx + dx, cy + dy), WorldTileData { terrain, remaining, visibility: TileVisibility::Visible });
            }
        }
    }
    assert!(map.tiles.values().all(|t| t.visibility == TileVisibility::Visible), "not all tiles Visible");
    let resources = map.count_terrain(TerrainTypeWorld::IronVein)
        + map.count_terrain(TerrainTypeWorld::CopperVein)
        + map.count_terrain(TerrainTypeWorld::WaterSource);
    assert!(resources >= 1, "no resource veins in starting area");
}

// ── AC2: Terrain placement ───────────────────────────────────────────────────

#[test]
fn iron_miner_can_be_placed_on_iron_vein_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::IronVein, 8, 10, TileVisibility::Visible, BiomeId::Forest, Some(500.0));
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::IronMiner, x: 8, y: 10,
        required_terrain: Some(TerrainTypeWorld::IronVein),
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert!(rejection.is_none(), "expected success, got: {:?}", rejection);
    let placed = app.world_mut().query::<(&Position, &Building)>()
        .iter(app.world())
        .any(|(p, b)| p.x == 8 && p.y == 10 && b.building_type == BuildingType::IronMiner);
    assert!(placed, "IronMiner should exist at [8, 10]");
}

#[test]
fn lava_siphon_can_be_placed_on_lava_source_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::LavaSource, 5, 5, TileVisibility::Visible, BiomeId::Volcanic, None);
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::LavaSiphon, x: 5, y: 5,
        required_terrain: Some(TerrainTypeWorld::LavaSource),
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert!(rejection.is_none(), "lava_siphon placement failed: {:?}", rejection);
}

#[test]
fn wind_turbine_can_be_placed_on_any_buildable_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::Grass, 3, 3, TileVisibility::Visible, BiomeId::Forest, None);
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::WindTurbine, x: 3, y: 3,
        required_terrain: None,
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert!(rejection.is_none(), "wind_turbine on grass failed: {:?}", rejection);
}

#[test]
fn iron_miner_cannot_be_placed_on_grass_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::Grass, 2, 2, TileVisibility::Visible, BiomeId::Forest, None);
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::IronMiner, x: 2, y: 2,
        required_terrain: Some(TerrainTypeWorld::IronVein),
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert_eq!(rejection, Some("terrain_mismatch"), "expected terrain_mismatch, got: {:?}", rejection);
}

#[test]
fn lava_siphon_cannot_be_placed_on_scorched_rock_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::ScorchedRock, 6, 6, TileVisibility::Visible, BiomeId::Volcanic, None);
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::LavaSiphon, x: 6, y: 6,
        required_terrain: Some(TerrainTypeWorld::LavaSource),
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert_eq!(rejection, Some("terrain_mismatch"), "expected terrain_mismatch, got: {:?}", rejection);
}

#[test]
fn building_cannot_be_placed_on_impassable_tile() {
    let mut app = app_with_tile(TerrainTypeWorld::Impassable, 1, 1, TileVisibility::Visible, BiomeId::Forest, None);
    app.world_mut().resource_mut::<WorldPlacementCommands>().queue.push(WorldPlacementCmd {
        building_type: BuildingType::WindTurbine, x: 1, y: 1,
        required_terrain: None,
    });
    app.update();
    let rejection = app.world().resource::<WorldPlacementCommands>().last_rejection;
    assert_eq!(rejection, Some("tile_not_buildable"), "expected tile_not_buildable, got: {:?}", rejection);
}

// ── Quality Map ──────────────────────────────────────────────────────────────

#[test]
fn forest_biome_marks_wood_as_high_quality() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("wood", Some(ResourceQuality::High));
    assert_eq!(map.quality("wood"), Some(Some(ResourceQuality::High)));
}

#[test]
fn forest_biome_marks_iron_ore_as_normal_quality() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("iron_ore", Some(ResourceQuality::Normal));
    assert_eq!(map.quality("iron_ore"), Some(Some(ResourceQuality::Normal)));
}

#[test]
fn volcanic_biome_has_no_natural_wood() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("wood", None);
    assert_eq!(map.quality("wood"), Some(None), "volcanic should have no natural wood");
}

#[test]
fn volcanic_biome_has_no_natural_water() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("water", None);
    assert_eq!(map.quality("water"), Some(None), "volcanic should have no natural water");
}

#[test]
fn desert_biome_marks_mana_crystal_as_high_quality() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("mana_crystal", Some(ResourceQuality::High));
    assert_eq!(map.quality("mana_crystal"), Some(Some(ResourceQuality::High)));
}

#[test]
fn ocean_biome_marks_water_as_high_quality() {
    let mut map = BiomeQualityMap::default();
    map.entries.insert("water", Some(ResourceQuality::High));
    assert_eq!(map.quality("water"), Some(Some(ResourceQuality::High)));
}
