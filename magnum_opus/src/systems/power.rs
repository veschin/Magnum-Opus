use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::*;
use crate::resources::EnergyPool;

const MAX_MODIFIER: f32 = 1.5;

pub fn energy_system(
    buildings: Query<(&Building, &GroupMember)>,
    mut groups: Query<(Entity, &mut GroupEnergy), With<Group>>,
    mut pool: ResMut<EnergyPool>,
) {
    // Reset all group energy
    for (_, mut ge) in groups.iter_mut() {
        ge.demand = 0.0;
        ge.allocated = 0.0;
    }

    // Aggregate per-group generation and consumption
    let mut per_group: HashMap<Entity, (f32, f32)> = HashMap::new();
    for (building, member) in buildings.iter() {
        let entry = per_group.entry(member.group_id).or_default();
        let energy_gen_val = building.building_type.energy_generation();
        let cons = building.building_type.energy_consumption();
        entry.0 += energy_gen_val;
        entry.1 += cons;
    }

    // Accumulate global totals and write per-group demand
    let mut total_gen = 0.0f32;
    let mut total_cons = 0.0f32;
    for (gid, (energy_gen_val, cons)) in &per_group {
        total_gen += energy_gen_val;
        total_cons += cons;
        if let Ok((_, mut ge)) = groups.get_mut(*gid) {
            ge.demand = *cons;
        }
    }

    // Priority-based allocation: HIGH → MEDIUM → LOW
    let mut remaining = total_gen;
    for priority in [EnergyPriority::High, EnergyPriority::Medium, EnergyPriority::Low] {
        let tier_groups: Vec<(Entity, f32)> = groups.iter()
            .filter(|(_, ge)| ge.priority == priority && ge.demand > 0.001)
            .map(|(e, ge)| (e, ge.demand))
            .collect();

        if tier_groups.is_empty() { continue; }

        let tier_demand: f32 = tier_groups.iter().map(|(_, d)| d).sum();

        if remaining >= tier_demand {
            // Full allocation for this tier
            for (group_id, demand) in &tier_groups {
                if let Ok((_, mut ge)) = groups.get_mut(*group_id) {
                    ge.allocated = *demand;
                }
            }
            remaining -= tier_demand;
        } else {
            // Proportional allocation
            for (group_id, demand) in &tier_groups {
                let alloc = if tier_demand > 0.0 { (demand / tier_demand) * remaining } else { 0.0 };
                if let Ok((_, mut ge)) = groups.get_mut(*group_id) {
                    ge.allocated = alloc;
                }
            }
            remaining = 0.0;
        }
    }

    let global_ratio = if total_cons > 0.001 {
        (total_gen / total_cons).clamp(0.0, MAX_MODIFIER)
    } else {
        1.0
    };

    pool.total_generation = total_gen;
    pool.total_consumption = total_cons;
    pool.ratio = global_ratio;
}
