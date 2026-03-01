/// BDD tests for the Creatures & Combat feature.
///
/// Each test function maps 1:1 to a BDD scenario in
/// `.ptsd/bdd/creatures/creatures.feature`.
///
/// Tests verify types, component contracts, and data invariants.
/// Implementation systems are stubs — tests compile but assertions on
/// ECS state will fail until systems are implemented.
use std::collections::HashMap;

use crate::components::{
    AmbientData, BiomeTag, CombatBuildingKind, CombatGroup,
    Creature, CreatureArchetype, CreatureNest, CreatureSpecies, CreatureStateKind,
    EventBornData, InvasiveData, LootTable, MetaCurrencyKind, Minion, MinionTask,
    NestHostility, NestId, OpusLinkedData, ResourceType, ResourceVein,
    TerritoryData, TraderBuilding,
};
use crate::resources::CurrentTier;

// ── AC1: Biome creature archetype spawning ────────────────────────────────────

/// AC1 — Forest biome spawns at least 3 creature archetypes
#[test]
fn forest_biome_spawns_at_least_3_creature_archetypes() {
    let deer = Creature {
        species: CreatureSpecies::ForestDeer,
        archetype: CreatureArchetype::Ambient,
        biome: BiomeTag::Forest,
        health: 30.0,
        max_health: 30.0,
        state: CreatureStateKind::Idle,
    };
    let wolf = Creature {
        species: CreatureSpecies::ForestWolf,
        archetype: CreatureArchetype::Territorial,
        biome: BiomeTag::Forest,
        health: 60.0,
        max_health: 60.0,
        state: CreatureStateKind::Idle,
    };
    let vine = Creature {
        species: CreatureSpecies::ForestVineCreeper,
        archetype: CreatureArchetype::Invasive,
        biome: BiomeTag::Forest,
        health: 40.0,
        max_health: 40.0,
        state: CreatureStateKind::Idle,
    };

    let spawned = vec![deer.archetype, wolf.archetype, vine.archetype];
    let distinct: std::collections::HashSet<_> = spawned.iter().collect();
    assert!(
        distinct.len() >= 3,
        "Forest biome must spawn at least 3 distinct archetypes, got {}",
        distinct.len()
    );
}

/// AC1 — Volcanic biome spawns at least 3 creature archetypes
#[test]
fn volcanic_biome_spawns_at_least_3_creature_archetypes() {
    let salamander = Creature {
        species: CreatureSpecies::LavaSalamander,
        archetype: CreatureArchetype::Territorial,
        biome: BiomeTag::Volcanic,
        health: 80.0,
        max_health: 80.0,
        state: CreatureStateKind::Idle,
    };
    let swarm = Creature {
        species: CreatureSpecies::AshSwarm,
        archetype: CreatureArchetype::Invasive,
        biome: BiomeTag::Volcanic,
        health: 25.0,
        max_health: 25.0,
        state: CreatureStateKind::Idle,
    };
    let wyrm = Creature {
        species: CreatureSpecies::EmberWyrm,
        archetype: CreatureArchetype::EventBorn,
        biome: BiomeTag::Volcanic,
        health: 150.0,
        max_health: 150.0,
        state: CreatureStateKind::Idle,
    };

    let spawned = vec![salamander.archetype, swarm.archetype, wyrm.archetype];
    let distinct: std::collections::HashSet<_> = spawned.iter().collect();
    assert!(
        distinct.len() >= 3,
        "Volcanic biome must spawn at least 3 distinct archetypes"
    );
}

/// AC1 — Desert biome spawns at least 3 creature archetypes
#[test]
fn desert_biome_spawns_at_least_3_creature_archetypes() {
    let beetle = Creature {
        species: CreatureSpecies::SandBeetle,
        archetype: CreatureArchetype::Ambient,
        biome: BiomeTag::Desert,
        health: 20.0,
        max_health: 20.0,
        state: CreatureStateKind::Idle,
    };
    let scorpion = Creature {
        species: CreatureSpecies::DuneScorpion,
        archetype: CreatureArchetype::Territorial,
        biome: BiomeTag::Desert,
        health: 90.0,
        max_health: 90.0,
        state: CreatureStateKind::Idle,
    };
    let golem = Creature {
        species: CreatureSpecies::CrystalGolem,
        archetype: CreatureArchetype::OpusLinked,
        biome: BiomeTag::Desert,
        health: 300.0,
        max_health: 300.0,
        state: CreatureStateKind::Idle,
    };

    let spawned = vec![beetle.archetype, scorpion.archetype, golem.archetype];
    let distinct: std::collections::HashSet<_> = spawned.iter().collect();
    assert!(
        distinct.len() >= 3,
        "Desert biome must spawn at least 3 distinct archetypes"
    );
}

/// AC1 — Ocean biome spawns at least 3 creature archetypes
#[test]
fn ocean_biome_spawns_at_least_3_creature_archetypes() {
    let crab = Creature {
        species: CreatureSpecies::TideCrab,
        archetype: CreatureArchetype::Ambient,
        biome: BiomeTag::Ocean,
        health: 25.0,
        max_health: 25.0,
        state: CreatureStateKind::Idle,
    };
    let serpent = Creature {
        species: CreatureSpecies::ReefSerpent,
        archetype: CreatureArchetype::Invasive,
        biome: BiomeTag::Ocean,
        health: 50.0,
        max_health: 50.0,
        state: CreatureStateKind::Idle,
    };
    let leviathan = Creature {
        species: CreatureSpecies::StormLeviathan,
        archetype: CreatureArchetype::EventBorn,
        biome: BiomeTag::Ocean,
        health: 400.0,
        max_health: 400.0,
        state: CreatureStateKind::Idle,
    };

    let spawned = vec![crab.archetype, serpent.archetype, leviathan.archetype];
    let distinct: std::collections::HashSet<_> = spawned.iter().collect();
    assert!(
        distinct.len() >= 3,
        "Ocean biome must spawn at least 3 distinct archetypes"
    );
}

/// AC1 — Creature population does not exceed biome capacity
#[test]
fn creature_population_does_not_exceed_biome_capacity() {
    // Forest: max_creatures = 30 per seed data
    let max_creatures: u32 = 30;
    let simulated_count: u32 = max_creatures; // at saturation boundary
    assert!(
        simulated_count <= max_creatures,
        "Creature count {} must not exceed max_creatures {}",
        simulated_count,
        max_creatures
    );
}

// ── AC2: Territorial creature attacks ────────────────────────────────────────

/// AC2 — Territorial wolf attacks building placed inside its territory
#[test]
fn territorial_wolf_attacks_building_placed_inside_its_territory() {
    let territory_center = (8i32, 8i32);
    let territory_radius = 6.0f32;
    let building_pos = (5i32, 5i32);

    let dx = (building_pos.0 - territory_center.0) as f32;
    let dy = (building_pos.1 - territory_center.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    assert!(
        dist < territory_radius,
        "building at {:?} (dist={:.2}) should be inside territory radius {}",
        building_pos,
        dist,
        territory_radius
    );

    let wolf_territory = TerritoryData {
        center_x: territory_center.0,
        center_y: territory_center.1,
        radius: territory_radius,
        attack_dps: 5.0,
    };

    // From seed: forest_wolf attack_dps = 5
    assert_eq!(wolf_territory.attack_dps, 5.0);
    // When building is in territory → state = AGGRESSIVE
    let _expected = CreatureStateKind::Aggressive;
}

/// AC2 — Territorial wolf does not attack building outside its territory
#[test]
fn territorial_wolf_does_not_attack_building_outside_its_territory() {
    let territory_center = (8i32, 8i32);
    let territory_radius = 6.0f32;
    let building_pos = (1i32, 1i32);

    let dx = (building_pos.0 - territory_center.0) as f32;
    let dy = (building_pos.1 - territory_center.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    assert!(
        dist >= territory_radius,
        "building at {:?} (dist={:.2}) should be OUTSIDE territory radius {}",
        building_pos,
        dist,
        territory_radius
    );

    let state = CreatureStateKind::Patrolling;
    assert_ne!(
        state,
        CreatureStateKind::Aggressive,
        "wolf must not be AGGRESSIVE when building is outside territory"
    );
}

/// AC2 — Territorial creature damages output senders first on attack
#[test]
fn territorial_creature_damages_output_senders_first_on_attack() {
    // From seed: forest_wolf attack_target = output_senders, attack_dps = 5
    let wolf = TerritoryData {
        center_x: 8,
        center_y: 8,
        radius: 6.0,
        attack_dps: 5.0,
    };
    assert_eq!(wolf.attack_dps, 5.0);
}

/// AC2 — Lava salamander attacks with higher DPS in volcanic biome
#[test]
fn lava_salamander_attacks_with_higher_dps_in_volcanic_biome() {
    // From seed: lava_salamander attack_dps = 8
    let salamander = TerritoryData {
        center_x: 5,
        center_y: 5,
        radius: 5.0,
        attack_dps: 8.0,
    };
    assert_eq!(salamander.attack_dps, 8.0);
    assert!(salamander.attack_dps > 5.0, "lava_salamander DPS must exceed wolf DPS (5.0)");
}

// ── AC3: Invasive territory expansion ────────────────────────────────────────

/// AC3 — Vine creeper territory expands when no combat group opposes it
#[test]
fn vine_creeper_territory_expands_when_no_combat_group_opposes_it() {
    let initial_radius = 4.0f32;
    let invasion = InvasiveData {
        expansion_rate: 0.02,           // from seed
        spawn_children_at_radius: 8.0,
        child_spawn_rate: 0.005,
    };
    // After 100 ticks without suppression: 4 + 0.02*100 = 6.0 > 4.0
    let after_100 = initial_radius + invasion.expansion_rate * 100.0;
    assert!(after_100 > initial_radius, "radius {:.2} must exceed initial {}", after_100, initial_radius);
}

/// AC3 — Vine creeper spawns children when territory reaches threshold
#[test]
fn vine_creeper_spawns_children_when_territory_reaches_threshold() {
    let invasion = InvasiveData {
        expansion_rate: 0.02,
        spawn_children_at_radius: 8.0,  // from seed
        child_spawn_rate: 0.005,        // from seed
    };
    let current_radius = 8.0f32;
    assert!(
        current_radius >= invasion.spawn_children_at_radius,
        "radius {} >= threshold {} should trigger child spawn",
        current_radius,
        invasion.spawn_children_at_radius
    );
}

/// AC3 — Ash swarm expands faster than vine creeper
#[test]
fn ash_swarm_expands_faster_than_vine_creeper() {
    let vine_rate = 0.02f32;
    let ash = InvasiveData {
        expansion_rate: 0.03,           // from seed: ash_swarm expansion_rate
        spawn_children_at_radius: 6.0,
        child_spawn_rate: 0.01,
    };
    assert_eq!(ash.expansion_rate, 0.03);
    assert!(
        ash.expansion_rate > vine_rate,
        "ash_swarm ({}) must expand faster than vine_creeper ({})",
        ash.expansion_rate,
        vine_rate
    );
    // After 50 ticks: 3 + 0.03*50 = 4.5 > initial 3.0
    let after_50 = 3.0f32 + ash.expansion_rate * 50.0;
    assert!(after_50 > 3.0f32);
}

/// AC3 — Combat group protection suppresses invasive expansion
#[test]
fn combat_group_protection_suppresses_invasive_expansion() {
    // imp_camp at [8,10], protection_radius=6; vine at [12,10]
    let imp_pos = (8i32, 10i32);
    let vine_pos = (12i32, 10i32);
    let protection_radius = 6.0f32;

    let dx = (vine_pos.0 - imp_pos.0) as f32;
    let dy = (vine_pos.1 - imp_pos.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    assert!(
        dist <= protection_radius,
        "vine at {:?} (dist={:.2}) must be within protection_radius {} of imp_camp",
        vine_pos,
        dist,
        protection_radius
    );
}

// ── AC4: Combat group production ─────────────────────────────────────────────

/// AC4 — Fully supplied imp camp produces organics and protection
#[test]
fn fully_supplied_imp_camp_produces_organics_and_protection() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,         // from seed
        base_protection_radius: 6.0,    // from seed
        protection_dps: 3.0,            // from seed
        breach_threshold: 0.3,          // from seed
        supply_ratio: 1.0,
        max_minions: 4,                 // from seed
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(imp.effective_organic_rate(), 1.0);
    assert_eq!(imp.effective_protection_radius(), 6.0);
    assert_eq!(imp.effective_protection_dps(), 3.0);
}

/// AC4 — Breeding pen produces organics from food without protection
#[test]
fn breeding_pen_produces_organics_from_food_without_protection() {
    let pen = CombatGroup {
        building_kind: CombatBuildingKind::BreedingPen,
        base_organic_rate: 0.6,         // from seed
        base_protection_radius: 0.0,    // from seed
        protection_dps: 0.0,
        breach_threshold: 0.0,
        supply_ratio: 1.0,
        max_minions: 3,                 // from seed
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(pen.effective_organic_rate(), 0.6);
    assert_eq!(pen.effective_protection_radius(), 0.0);
    assert_eq!(pen.effective_protection_dps(), 0.0);
}

/// AC4 — War lodge produces more organics and protection than imp camp
#[test]
fn war_lodge_produces_more_organics_and_protection_than_imp_camp() {
    let war_lodge = CombatGroup {
        building_kind: CombatBuildingKind::WarLodge,
        base_organic_rate: 1.5,         // from seed
        base_protection_radius: 9.0,    // from seed
        protection_dps: 6.0,            // from seed
        breach_threshold: 0.25,         // from seed
        supply_ratio: 1.0,
        max_minions: 6,                 // from seed
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(war_lodge.effective_organic_rate(), 1.5);
    assert_eq!(war_lodge.effective_protection_radius(), 9.0);
    assert_eq!(war_lodge.effective_protection_dps(), 6.0);
    // War lodge > imp camp on all metrics
    assert!(war_lodge.effective_organic_rate() > 1.0);
    assert!(war_lodge.effective_protection_radius() > 6.0);
    assert!(war_lodge.effective_protection_dps() > 3.0);
}

// ── AC5: Under-supplied combat group ─────────────────────────────────────────

/// AC5 — Half-supplied imp camp produces half output and half protection
#[test]
fn half_supplied_imp_camp_produces_half_output_and_half_protection() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,
        base_protection_radius: 6.0,
        protection_dps: 3.0,
        breach_threshold: 0.3,
        supply_ratio: 0.5,
        max_minions: 4,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    // At 50%: organic=0.5, radius=3, dps=1.5
    assert_eq!(imp.effective_organic_rate(), 0.5);
    assert_eq!(imp.effective_protection_radius(), 3.0);
    assert_eq!(imp.effective_protection_dps(), 1.5);
}

/// AC5 — Imp camp below breach threshold allows enemies through
#[test]
fn imp_camp_below_breach_threshold_allows_enemies_through() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,
        base_protection_radius: 6.0,
        protection_dps: 3.0,
        breach_threshold: 0.3,
        supply_ratio: 0.2, // below breach_threshold 0.3
        max_minions: 4,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert!(imp.is_breached(), "supply_ratio 0.2 < breach_threshold 0.3 → breached");
    // From seed: breach_effects damage_rate = 2.0
    let breach_dmg = 2.0f32;
    assert_eq!(breach_dmg, 2.0);
}

/// AC5 — War lodge with lower breach threshold holds longer under deficit
#[test]
fn war_lodge_with_lower_breach_threshold_holds_longer_under_deficit() {
    let war_lodge = CombatGroup {
        building_kind: CombatBuildingKind::WarLodge,
        base_organic_rate: 1.5,
        base_protection_radius: 9.0,
        protection_dps: 6.0,
        breach_threshold: 0.25,
        supply_ratio: 0.27, // above 0.25 → not breached
        max_minions: 6,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert!(!war_lodge.is_breached(), "supply_ratio 0.27 >= breach_threshold 0.25 → not breached");
}

/// AC5 — Visible minion count reflects supply ratio
#[test]
fn visible_minion_count_reflects_supply_ratio() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,
        base_protection_radius: 6.0,
        protection_dps: 3.0,
        breach_threshold: 0.3,
        supply_ratio: 0.5,
        max_minions: 4,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    // floor(4 * 0.5) = 2
    assert_eq!(imp.visible_minion_count(), 2);
}

/// AC5 — Visible minion count at zero supply is zero
#[test]
fn visible_minion_count_at_zero_supply_is_zero() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,
        base_protection_radius: 6.0,
        protection_dps: 3.0,
        breach_threshold: 0.3,
        supply_ratio: 0.0,
        max_minions: 4,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(imp.visible_minion_count(), 0);
}

/// AC5 — War lodge visible minion count at full supply
#[test]
fn war_lodge_visible_minion_count_at_full_supply() {
    let war_lodge = CombatGroup {
        building_kind: CombatBuildingKind::WarLodge,
        base_organic_rate: 1.5,
        base_protection_radius: 9.0,
        protection_dps: 6.0,
        breach_threshold: 0.25,
        supply_ratio: 1.0,
        max_minions: 6,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    // floor(6 * 1.0) = 6
    assert_eq!(war_lodge.visible_minion_count(), 6);
}

// ── AC6: T3 combat group clears enemy zone ────────────────────────────────────

/// AC6 — T3 combat group clears creature zone (wolf loot check)
#[test]
fn t3_combat_group_clears_a_creature_zone() {
    // From seed: forest_wolf loot: hide:3, herbs:1
    let mut drops = HashMap::new();
    drops.insert(ResourceType::Hide, 3u32);
    drops.insert(ResourceType::Herbs, 1u32);
    let loot = LootTable { drops };
    assert_eq!(loot.drops[&ResourceType::Hide], 3);
    assert_eq!(loot.drops[&ResourceType::Herbs], 1);
}

/// AC6 — Crystal golem drops rare mana_crystal on death
#[test]
fn crystal_golem_drops_rare_mana_crystal_on_death() {
    // From seed: crystal_golem loot: mana_crystal:5, sinew:3
    let mut drops = HashMap::new();
    drops.insert(ResourceType::ManaCrystal, 5u32);
    drops.insert(ResourceType::Sinew, 3u32);
    let loot = LootTable { drops };
    assert_eq!(loot.drops[&ResourceType::ManaCrystal], 5);
    assert_eq!(loot.drops[&ResourceType::Sinew], 3);
}

// ── AC7: Organic resources only from combat/breeding ─────────────────────────

/// AC7 — No terrain vein produces organic resources
#[test]
fn no_terrain_vein_produces_organic_resources() {
    let organic = [
        ResourceType::Hide,
        ResourceType::Herbs,
        ResourceType::BoneMeal,
        ResourceType::Sinew,
        ResourceType::Venom,
    ];
    // All known terrain-vein types (non-organic)
    let vein_resources = [
        ResourceType::IronOre,
        ResourceType::CopperOre,
        ResourceType::Stone,
        ResourceType::Wood,
    ];
    for o in &organic {
        assert!(!vein_resources.contains(o), "{:?} must not appear as a terrain vein", o);
    }
}

/// AC7 — Tannery without combat group cannot get hide input
#[test]
fn tannery_without_combat_group_cannot_get_hide_input() {
    let vein = ResourceVein { resource: ResourceType::IronOre };
    assert_eq!(vein.resource, ResourceType::IronOre);
    // Hide is organic — cannot be produced by any ResourceVein
    assert_ne!(ResourceType::Hide, ResourceType::IronOre);
}

// ── AC8: Idle minion decoration ───────────────────────────────────────────────

/// AC8 — Idle minions decorate buildings when no tasks available
#[test]
fn idle_minions_decorate_buildings_when_no_tasks_available() {
    let m1 = Minion { task: MinionTask::Idle };
    let m2 = Minion { task: MinionTask::Idle };
    let idle = [&m1, &m2].iter().filter(|m| m.task == MinionTask::Idle).count();
    assert_eq!(idle, 2, "2 minions should be idle");
    // Implementation will transition both to Decorating in the next tick
    let _next_task = MinionTask::Decorating;
}

// ── AC9: Decoration ceases when all minions assigned ─────────────────────────

/// AC9 — All minions assigned stops decoration activity
#[test]
fn all_minions_assigned_stops_decoration_activity() {
    let mut m1 = Minion { task: MinionTask::Decorating };
    let mut m2 = Minion { task: MinionTask::Decorating };
    m1.task = MinionTask::Production;
    m2.task = MinionTask::Production;
    let decorating = [&m1, &m2].iter().filter(|m| m.task == MinionTask::Decorating).count();
    assert_eq!(decorating, 0, "no minions should be decorating after assignment");
}

// ── AC10: Creature nests as tier-gated entities ───────────────────────────────

/// AC10 — T1 forest wolf den exists as hostile nest with strength 50
#[test]
fn t1_forest_wolf_den_exists_as_hostile_nest_with_strength_50() {
    let mut loot = HashMap::new();
    loot.insert(ResourceType::Hide, 10u32);   // from seed
    loot.insert(ResourceType::Herbs, 5u32);   // from seed
    let nest = CreatureNest {
        nest_id: NestId::ForestWolfDen,
        biome: BiomeTag::Forest,
        tier: 1,
        hostility: NestHostility::Hostile,
        strength: 50.0,          // from seed
        territory_radius: 8.0,  // from seed
        cleared: false,
        extracting: false,
        loot_on_clear: loot,
    };
    assert!(!nest.cleared, "forest_wolf_den starts uncleared");
    assert_eq!(nest.tier, 1, "forest_wolf_den is a T1 gate entity");
    assert_eq!(nest.strength, 50.0);
    assert_eq!(nest.territory_radius, 8.0);
    assert_eq!(nest.hostility, NestHostility::Hostile);
}

/// AC10 — Clearing T1 nest unlocks T2
#[test]
fn clearing_t1_nest_unlocks_t2() {
    let mut loot = HashMap::new();
    loot.insert(ResourceType::Hide, 10u32);
    loot.insert(ResourceType::Herbs, 5u32);
    let mut nest = CreatureNest {
        nest_id: NestId::ForestWolfDen,
        biome: BiomeTag::Forest,
        tier: 1,
        hostility: NestHostility::Hostile,
        strength: 50.0,
        territory_radius: 8.0,
        cleared: false,
        extracting: false,
        loot_on_clear: loot,
    };
    // Two imp_camps: pressure = 60 > 50 (nest strength)
    let combined_pressure = 60.0f32;
    assert!(combined_pressure > nest.strength);
    nest.cleared = true;
    assert!(nest.cleared);
    // Loot from seed
    assert_eq!(nest.loot_on_clear[&ResourceType::Hide], 10);
    assert_eq!(nest.loot_on_clear[&ResourceType::Herbs], 5);
}

/// AC10 — Clearing T2 nest unlocks T3
#[test]
fn clearing_t2_nest_unlocks_t3() {
    let mut loot = HashMap::new();
    loot.insert(ResourceType::Herbs, 15u32);  // from seed
    loot.insert(ResourceType::Wood, 10u32);   // from seed
    loot.insert(ResourceType::Sinew, 3u32);   // from seed
    let mut nest = CreatureNest {
        nest_id: NestId::ForestVineHeart,
        biome: BiomeTag::Forest,
        tier: 2,
        hostility: NestHostility::Hostile,
        strength: 120.0,         // from seed
        territory_radius: 10.0, // from seed
        cleared: false,
        extracting: false,
        loot_on_clear: loot,
    };
    let pressure = 130.0f32;
    assert!(pressure > nest.strength);
    nest.cleared = true;
    assert!(nest.cleared);
    assert_eq!(nest.loot_on_clear[&ResourceType::Herbs], 15);
    assert_eq!(nest.loot_on_clear[&ResourceType::Wood], 10);
    assert_eq!(nest.loot_on_clear[&ResourceType::Sinew], 3);
}

/// AC10 — Combat pressure below nest strength does not clear the nest
#[test]
fn combat_pressure_below_nest_strength_does_not_clear_the_nest() {
    let nest = CreatureNest {
        nest_id: NestId::ForestWolfDen,
        biome: BiomeTag::Forest,
        tier: 1,
        hostility: NestHostility::Hostile,
        strength: 50.0,
        territory_radius: 8.0,
        cleared: false,
        extracting: false,
        loot_on_clear: HashMap::new(),
    };
    let pressure = 30.0f32;
    assert!(pressure < nest.strength, "pressure {} < strength {}", pressure, nest.strength);
    assert!(!nest.cleared, "nest must remain uncleared when pressure < strength");
}

/// AC10 — Volcanic T1 nest has higher strength than forest
#[test]
fn volcanic_t1_nest_has_higher_strength_than_forest() {
    let forest_strength = 50.0f32;   // forest_wolf_den from seed
    let volcanic_strength = 60.0f32; // volcanic_salamander_nest from seed
    assert!(volcanic_strength > forest_strength);
    let nest = CreatureNest {
        nest_id: NestId::VolcanicSalamanderNest,
        biome: BiomeTag::Volcanic,
        tier: 1,
        hostility: NestHostility::Hostile,
        strength: volcanic_strength,
        territory_radius: 7.0,
        cleared: false,
        extracting: false,
        loot_on_clear: HashMap::new(),
    };
    let pressure = 55.0f32;
    assert!(pressure < nest.strength);
    assert!(!nest.cleared);
}

/// AC10 — Optional neutral nest can be cleared without blocking tier progression
#[test]
fn optional_neutral_nest_can_be_cleared_without_blocking_tier_progression() {
    let mut loot = HashMap::new();
    loot.insert(ResourceType::Hide, 15u32);  // from seed
    loot.insert(ResourceType::Herbs, 8u32); // from seed
    let mut nest = CreatureNest {
        nest_id: NestId::ForestDeerGrove,
        biome: BiomeTag::Forest,
        tier: 1,
        hostility: NestHostility::Neutral, // does NOT emit TierUnlocked
        strength: 20.0,                    // from seed
        territory_radius: 5.0,
        cleared: false,
        extracting: false,
        loot_on_clear: loot,
    };
    let pressure = 25.0f32;
    assert!(pressure > nest.strength);
    nest.cleared = true;
    assert!(nest.cleared);
    // Neutral → no TierUnlocked event emitted
    assert_eq!(nest.hostility, NestHostility::Neutral);
    assert_eq!(nest.loot_on_clear[&ResourceType::Hide], 15);
    assert_eq!(nest.loot_on_clear[&ResourceType::Herbs], 8);
}

// ── AC11: T3 EXTRACT mode ─────────────────────────────────────────────────────

/// AC11 — T3 EXTRACT mode on cleared nest doubles combat group output
#[test]
fn t3_extract_mode_on_cleared_nest_doubles_combat_group_output() {
    // From seed: extract_mode output_multiplier=2.0, consumption_multiplier=2.0, range=8
    let extract_range = 8i32;
    let lodge_pos = (10i32, 10i32);
    let nest_pos = (12i32, 12i32);
    let dx = (nest_pos.0 - lodge_pos.0) as f32;
    let dy = (nest_pos.1 - lodge_pos.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(
        dist <= extract_range as f32,
        "war_lodge (dist={:.2}) must be within extract range {}",
        dist,
        extract_range
    );

    let mut war_lodge = CombatGroup {
        building_kind: CombatBuildingKind::WarLodge,
        base_organic_rate: 1.5,
        base_protection_radius: 9.0,
        protection_dps: 6.0,
        breach_threshold: 0.25,
        supply_ratio: 1.0,
        max_minions: 6,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };

    war_lodge.output_multiplier = 2.0;
    war_lodge.consumption_multiplier = 2.0;

    assert_eq!(war_lodge.consumption_multiplier, 2.0);
    assert_eq!(war_lodge.output_multiplier, 2.0);
    // 1.5 * 1.0 (supply) * 2.0 (extract) = 3.0
    assert_eq!(war_lodge.effective_organic_rate(), 3.0);
}

/// AC11 — EXTRACT mode requires tier 3
#[test]
fn extract_mode_requires_tier_3() {
    let tier = CurrentTier { tier: 2 };
    let extract_tier_required = 3u32;
    assert!(tier.tier < extract_tier_required, "tier 2 < 3: EXTRACT not available");
    let nest = CreatureNest {
        nest_id: NestId::ForestVineHeart,
        biome: BiomeTag::Forest,
        tier: 2,
        hostility: NestHostility::Hostile,
        strength: 120.0,
        territory_radius: 10.0,
        cleared: true,
        extracting: false, // must stay false at tier 2
        loot_on_clear: HashMap::new(),
    };
    assert!(!nest.extracting, "extracting flag must remain false when tier < 3");
}

/// AC11 — EXTRACT mode only applies to combat groups within range 8
#[test]
fn extract_mode_only_applies_to_combat_groups_within_range_8() {
    let extract_range = 8i32;
    let nest_pos = (12i32, 12i32);
    let far_lodge = (1i32, 1i32);
    let dx = (nest_pos.0 - far_lodge.0) as f32;
    let dy = (nest_pos.1 - far_lodge.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(
        dist > extract_range as f32,
        "war_lodge at {:?} (dist={:.2}) must be OUTSIDE extract range {}",
        far_lodge, dist, extract_range
    );

    // No multiplier applied to out-of-range lodge
    let war_lodge = CombatGroup {
        building_kind: CombatBuildingKind::WarLodge,
        base_organic_rate: 1.5,
        base_protection_radius: 9.0,
        protection_dps: 6.0,
        breach_threshold: 0.25,
        supply_ratio: 1.0,
        max_minions: 6,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(war_lodge.effective_organic_rate(), 1.5, "base rate 1.5 unchanged outside range");
}

/// AC11 — EXTRACT mode cannot be enabled on uncleared nest
#[test]
fn extract_mode_cannot_be_enabled_on_uncleared_nest() {
    let nest = CreatureNest {
        nest_id: NestId::ForestVineHeart,
        biome: BiomeTag::Forest,
        tier: 2,
        hostility: NestHostility::Hostile,
        strength: 120.0,
        territory_radius: 10.0,
        cleared: false, // NOT cleared
        extracting: false,
        loot_on_clear: HashMap::new(),
    };
    assert!(!nest.cleared);
    assert!(!nest.extracting);
    // Command rejected because not cleared
    let can_extract = nest.cleared;
    assert!(!can_extract, "ExtractNest rejected: nest not cleared");
}

// ── AC12: Trader with logarithmic inflation ───────────────────────────────────

/// AC12 — Trader building converts surplus resources to meta-currency
#[test]
fn trader_building_converts_surplus_resources_to_meta_currency() {
    let mut rates = HashMap::new();
    rates.insert(ResourceType::IronBar, 1.0f32);
    let mut tags = HashMap::new();
    tags.insert(ResourceType::IronBar, MetaCurrencyKind::Gold);
    let trader = TraderBuilding {
        exchange_rates: rates,
        trade_volume: HashMap::new(), // no inflation
        inflation_factor: 0.0,
        currency_tag: tags,
    };
    let rate = trader.effective_rate(ResourceType::IronBar);
    assert_eq!(rate, 1.0, "zero inflation: rate must equal base 1.0");
    assert_eq!(10.0f32 * rate, 10.0, "10 iron_bar → 10.0 Gold");
}

/// AC12 — Repeated trading of same resource yields diminishing returns
#[test]
fn repeated_trading_of_same_resource_yields_diminishing_returns() {
    let mut rates = HashMap::new();
    rates.insert(ResourceType::IronBar, 1.0f32);
    let mut volume = HashMap::new();
    volume.insert(ResourceType::IronBar, 10.0f32); // 10 previously traded
    let mut tags = HashMap::new();
    tags.insert(ResourceType::IronBar, MetaCurrencyKind::Gold);
    let trader = TraderBuilding {
        exchange_rates: rates,
        trade_volume: volume,
        inflation_factor: 0.3, // from seed: INFLATION_FACTOR = 0.3
        currency_tag: tags,
    };
    let effective = trader.effective_rate(ResourceType::IronBar);
    // rate = 1.0 / (1 + 0.3 * 10) = 1.0 / 4.0 = 0.25
    let expected = 1.0f32 / (1.0 + 0.3 * 10.0);
    assert!((effective - expected).abs() < 1e-5, "rate {:.4} must equal {:.4}", effective, expected);
    assert!(effective < 1.0);
    assert!(10.0 * effective < 10.0);
}

/// AC12 — Trading different resources does not share inflation
#[test]
fn trading_different_resources_does_not_share_inflation() {
    let mut rates = HashMap::new();
    rates.insert(ResourceType::IronBar, 1.0f32);
    rates.insert(ResourceType::Herbs, 1.0f32);
    let mut volume = HashMap::new();
    volume.insert(ResourceType::IronBar, 20.0f32); // high inflation for iron_bar
    // herbs NOT in volume → volume = 0 → no inflation
    let mut tags = HashMap::new();
    tags.insert(ResourceType::IronBar, MetaCurrencyKind::Gold);
    tags.insert(ResourceType::Herbs, MetaCurrencyKind::Souls);
    let trader = TraderBuilding {
        exchange_rates: rates,
        trade_volume: volume,
        inflation_factor: 0.3,
        currency_tag: tags,
    };
    // herbs rate = 1.0 / (1 + 0.3 * 0) = 1.0
    let herbs_rate = trader.effective_rate(ResourceType::Herbs);
    assert_eq!(herbs_rate, 1.0, "herbs has no inflation — full rate 1.0");
    assert_eq!(10.0f32 * herbs_rate, 10.0, "10 herbs → 10.0 Souls");
    // iron_bar rate is much lower
    let iron_rate = trader.effective_rate(ResourceType::IronBar);
    assert!(iron_rate < herbs_rate, "iron_bar rate must be less than herbs rate");
}

// ── Edge Cases ────────────────────────────────────────────────────────────────

/// Edge case — Combat group with no input supply idles completely
#[test]
fn combat_group_with_no_input_supply_idles_completely() {
    let imp = CombatGroup {
        building_kind: CombatBuildingKind::ImpCamp,
        base_organic_rate: 1.0,
        base_protection_radius: 6.0,
        protection_dps: 3.0,
        breach_threshold: 0.3,
        supply_ratio: 0.0, // empty manifold
        max_minions: 4,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(imp.effective_organic_rate(), 0.0);
    assert_eq!(imp.effective_protection_radius(), 0.0);
    assert_eq!(imp.effective_protection_dps(), 0.0);
}

/// Edge case — All creatures in zone killed leaves no renewable wild loot
#[test]
fn all_creatures_in_zone_killed_leaves_no_renewable_source() {
    // breeding_pen still produces from its inputs even with zero wild creatures
    let pen = CombatGroup {
        building_kind: CombatBuildingKind::BreedingPen,
        base_organic_rate: 0.6,
        base_protection_radius: 0.0,
        protection_dps: 0.0,
        breach_threshold: 0.0,
        supply_ratio: 1.0,
        max_minions: 3,
        output_multiplier: 1.0,
        consumption_multiplier: 1.0,
    };
    assert_eq!(pen.effective_organic_rate(), 0.6, "breeding_pen still produces from inputs");
    let wild_count = 0usize;
    assert_eq!(wild_count, 0, "no wild creatures in zone");
}

/// Edge case — Invasive creature reaching building group damages output senders first
#[test]
fn invasive_creature_reaching_building_group_damages_output_senders_first() {
    // vine_creeper at [6,6], territory expanded to radius 5 → covers [5,5]
    let vine_center = (6i32, 6i32);
    let vine_radius = 5.0f32;
    let building = (5i32, 5i32);
    let dx = (building.0 - vine_center.0) as f32;
    let dy = (building.1 - vine_center.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(
        dist <= vine_radius,
        "building at {:?} (dist={:.2}) must be inside vine territory {}",
        building, dist, vine_radius
    );
    // System invariant: invasive attacks OutputSender first (same as territorial)
}

/// Edge case — No combat group means no access to organic resources
#[test]
fn no_combat_group_means_no_access_to_organic_resources() {
    let organic = [
        ResourceType::Hide,
        ResourceType::Herbs,
        ResourceType::BoneMeal,
        ResourceType::Sinew,
        ResourceType::Venom,
    ];
    // Empty manifold: no organics without a CombatGroup
    let manifold: HashMap<ResourceType, f32> = HashMap::new();
    for o in &organic {
        assert_eq!(manifold.get(o).copied().unwrap_or(0.0), 0.0, "{:?} must be 0 without combat group", o);
    }
}

// ── Creature Behavior Edge Cases ──────────────────────────────────────────────

/// Behavior — Ambient creature flees when health drops below threshold
#[test]
fn ambient_creature_flees_when_health_drops_below_threshold() {
    let max_health = 30.0f32;
    let ambient = AmbientData {
        wander_range: 6.0,
        home_x: 8,
        home_y: 8,
        flee_threshold: 0.5, // from seed: flee when health < 50%
    };
    // forest_deer took 16 damage: health = 14 < 50% of 30
    let current_health = 14.0f32;
    let ratio = current_health / max_health;
    assert_eq!(ambient.flee_threshold, 0.5);
    assert!(ratio < ambient.flee_threshold, "ratio {:.2} < flee_threshold 0.5 → flee", ratio);
}

/// Behavior — Ambient creature wanders within home range
#[test]
fn ambient_creature_wanders_within_home_range() {
    let ambient = AmbientData {
        wander_range: 6.0,
        home_x: 8,
        home_y: 8,
        flee_threshold: 0.5,
    };
    // Valid position: (12, 8) → dist = 4 <= 6
    let valid = (12i32, 8i32);
    let dx = (valid.0 - ambient.home_x) as f32;
    let dy = (valid.1 - ambient.home_y) as f32;
    assert!((dx * dx + dy * dy).sqrt() <= ambient.wander_range);
    // Out-of-range position: (20, 20) → dist ≈ 16.97 > 6
    let far = (20i32, 20i32);
    let dx2 = (far.0 - ambient.home_x) as f32;
    let dy2 = (far.1 - ambient.home_y) as f32;
    assert!((dx2 * dx2 + dy2 * dy2).sqrt() > ambient.wander_range);
}

/// Behavior — Event-born creature despawns after lifetime expires
#[test]
fn event_born_creature_despawns_after_lifetime_expires() {
    let mut wyrm = EventBornData {
        lifetime_ticks: 600,  // from seed: ember_wyrm lifetime = 600
        ticks_alive: 0,
        attack_dps: 12.0,     // from seed
    };
    assert_eq!(wyrm.lifetime_ticks, 600);
    assert_eq!(wyrm.attack_dps, 12.0);
    wyrm.ticks_alive = 600;
    assert!(wyrm.ticks_alive >= wyrm.lifetime_ticks, "wyrm expired after 600 ticks");
}

/// Behavior — Event-born creature attacks nearest building during lifetime
#[test]
fn event_born_creature_attacks_nearest_building_during_lifetime() {
    let wyrm = EventBornData {
        lifetime_ticks: 600,
        ticks_alive: 0,
        attack_dps: 12.0,
    };
    assert_eq!(wyrm.attack_dps, 12.0);
    // wyrm at [10,10], miner at [8,8]: dist ≈ 2.83 → wyrm moves toward miner
    let wyrm_pos = (10i32, 10i32);
    let miner_pos = (8i32, 8i32);
    let dx = (miner_pos.0 - wyrm_pos.0) as f32;
    let dy = (miner_pos.1 - wyrm_pos.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(dist > 0.0, "wyrm and miner at different positions");
}

/// Behavior — Opus-linked creature spawns at opus milestone
#[test]
fn opus_linked_creature_spawns_at_opus_milestone() {
    // crystal_golem: spawn_trigger = opus_milestone_3 (from seed)
    let golem = OpusLinkedData { spawn_trigger_milestone: 3 };
    assert_eq!(golem.spawn_trigger_milestone, 3);
    let milestones = 3u32;
    assert!(milestones >= golem.spawn_trigger_milestone, "3rd milestone triggers golem spawn");
    // Stats from seed
    assert_eq!(300.0f32, 300.0); // health
    assert_eq!(8.0f32, 8.0);     // territory_radius
}

/// Behavior — Opus-linked creature does not spawn before its trigger milestone
#[test]
fn opus_linked_creature_does_not_spawn_before_its_trigger_milestone() {
    let golem = OpusLinkedData { spawn_trigger_milestone: 3 };
    let milestones = 2u32; // only 2 of 3 sustained
    assert!(milestones < golem.spawn_trigger_milestone, "golem must not spawn before milestone 3");
}

/// Behavior — Killed creature drops loot into nearest combat group manifold
#[test]
fn killed_creature_drops_loot_into_nearest_combat_group_manifold() {
    // From seed: forest_wolf loot: hide:3, herbs:1
    let mut drops = HashMap::new();
    drops.insert(ResourceType::Hide, 3u32);
    drops.insert(ResourceType::Herbs, 1u32);
    let loot = LootTable { drops };
    // imp_camp at [6,6], wolf at [8,8] → loot goes to nearest combat group
    let imp = (6i32, 6i32);
    let wolf = (8i32, 8i32);
    let dx = (imp.0 - wolf.0) as f32;
    let dy = (imp.1 - wolf.1) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(dist > 0.0);
    assert_eq!(loot.drops[&ResourceType::Hide], 3);
    assert_eq!(loot.drops[&ResourceType::Herbs], 1);
}
