use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::*;
use crate::resources::{Biome, EnergyPool};

const MAX_MODIFIER: f32 = 1.5;

/// Applies biome-specific generation bonus to a building type.
fn biome_bonus(bt: BuildingType, biome: Biome) -> f32 {
    match (bt, biome) {
        (BuildingType::WindTurbine, Biome::Desert) => 1.3,
        (BuildingType::WindTurbine, Biome::Ocean) => 1.1,
        (BuildingType::WaterWheel, Biome::Ocean) => 1.4,
        _ => 1.0,
    }
}

/// Returns true if the ManaReactor has fuel (ManaCrystal > 0 in its group manifold).
/// `group_manifolds`: map from group_id -> &Manifold.
fn mana_reactor_has_fuel(group_id: Entity, group_manifolds: &HashMap<Entity, f32>) -> bool {
    group_manifolds.get(&group_id).copied().unwrap_or(0.0) > 0.0
}

pub fn energy_system(
    buildings: Query<(&Building, &GroupMember)>,
    mut groups: Query<(Entity, &mut GroupEnergy, Option<&Manifold>), With<Group>>,
    mut pool: ResMut<EnergyPool>,
    biome: Option<Res<Biome>>,
) {
    let active_biome = biome.map(|b| *b).unwrap_or(Biome::Forest);

    // Build map of group_id -> mana_crystal amount (for ManaReactor fuel check)
    let mut mana_crystal_by_group: HashMap<Entity, f32> = HashMap::new();
    for (gid, _, manifold_opt) in groups.iter() {
        if let Some(manifold) = manifold_opt {
            let amount = manifold.resources.get(&ResourceType::ManaCrystal).copied().unwrap_or(0.0);
            mana_crystal_by_group.insert(gid, amount);
        }
    }

    // Reset all group energy
    for (_, mut ge, _) in groups.iter_mut() {
        ge.demand = 0.0;
        ge.allocated = 0.0;
    }

    // Aggregate per-group generation and consumption
    let mut per_group: HashMap<Entity, (f32, f32)> = HashMap::new();
    for (building, member) in buildings.iter() {
        let entry = per_group.entry(member.group_id).or_default();
        let bt = building.building_type;
        let mut generation = bt.energy_generation();

        // ManaReactor requires fuel to generate
        if bt == BuildingType::ManaReactor {
            let has_fuel = mana_reactor_has_fuel(member.group_id, &mana_crystal_by_group);
            if !has_fuel {
                generation = 0.0;
            }
        }

        // Apply biome bonus
        if generation > 0.0 {
            generation *= biome_bonus(bt, active_biome);
        }

        let cons = bt.energy_consumption();
        entry.0 += generation;
        entry.1 += cons;
    }

    // Accumulate global totals and write per-group demand
    let mut total_gen = 0.0f32;
    let mut total_cons = 0.0f32;
    for (gid, (generation, cons)) in &per_group {
        total_gen += generation;
        total_cons += cons;
        if let Ok((_, mut ge, _)) = groups.get_mut(*gid) {
            ge.demand = *cons;
        }
    }

    // Compute global ratio (clamped)
    let global_ratio = if total_cons > 0.001 {
        (total_gen / total_cons).clamp(0.0, MAX_MODIFIER)
    } else {
        1.0
    };

    // Priority-based allocation: HIGH → MEDIUM → LOW
    //
    // In global deficit (global_ratio < 1.0): tiers are served in priority order.
    //   A tier whose total demand fits in remaining → all groups fully served (ratio=1.0).
    //   First tier that does not fit → proportional split (ratio<1.0).
    //   Remaining tiers get 0.
    //
    // In global surplus (global_ratio >= 1.0): all groups are fully served and
    //   allocated = demand * global_ratio so ge.ratio() reflects the speed boost.
    let mut remaining = total_gen;
    for priority in [EnergyPriority::High, EnergyPriority::Medium, EnergyPriority::Low] {
        let tier_groups: Vec<(Entity, f32)> = groups.iter()
            .filter(|(_, ge, _)| ge.priority == priority && ge.demand > 0.001)
            .map(|(e, ge, _)| (e, ge.demand))
            .collect();

        if tier_groups.is_empty() { continue; }

        let tier_demand: f32 = tier_groups.iter().map(|(_, d)| d).sum();

        if remaining >= tier_demand {
            // This tier is fully served.
            if global_ratio >= 1.0 {
                // Global surplus: encode speed boost via allocated = demand * global_ratio
                // so that ge.ratio() = global_ratio (clamped to MAX_MODIFIER).
                for (group_id, demand) in &tier_groups {
                    let alloc = demand * global_ratio.min(MAX_MODIFIER);
                    if let Ok((_, mut ge, _)) = groups.get_mut(*group_id) {
                        ge.allocated = alloc;
                    }
                }
            } else {
                // Deficit, but this priority tier still gets its full demand (ratio=1.0).
                for (group_id, demand) in &tier_groups {
                    if let Ok((_, mut ge, _)) = groups.get_mut(*group_id) {
                        ge.allocated = *demand;
                    }
                }
            }
            remaining -= tier_demand;
        } else {
            // Deficit: proportional allocation within this tier (ratio < 1.0).
            for (group_id, demand) in &tier_groups {
                let alloc = if tier_demand > 0.0 { (demand / tier_demand) * remaining } else { 0.0 };
                if let Ok((_, mut ge, _)) = groups.get_mut(*group_id) {
                    ge.allocated = alloc;
                }
            }
            remaining = 0.0;
        }
    }

    pool.total_generation = total_gen;
    pool.total_consumption = total_cons;
    pool.ratio = global_ratio;
}
