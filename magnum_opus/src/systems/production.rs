use bevy::prelude::*;

use crate::components::*;
use crate::resources::{Inventory, ProductionRates};

pub fn production_system(
    mut buildings: Query<(
        &Building,
        &Recipe,
        &mut ProductionState,
        &GroupMember,
        &mut InputBuffer,
        &mut OutputBuffer,
    )>,
    groups: Query<(&GroupEnergy, &GroupControl), With<Group>>,
    mut inventory: Option<ResMut<Inventory>>,
) {
    for (_building, recipe, mut state, member, mut input_buf, mut output_buf) in
        buildings.iter_mut()
    {
        let (ratio, paused) = groups
            .get(member.group_id)
            .map(|(ge, ctrl)| (ge.ratio(), ctrl.status == GroupStatus::Paused))
            .unwrap_or((0.0, false));

        // Paused groups do not produce
        if paused {
            state.idle_reason = Some(IdleReason::GroupPaused);
            state.active = false;
            state.progress = 0.0;
            continue;
        }

        if !state.active {
            let can_start = recipe.inputs.iter().all(|(res, amount)| {
                input_buf.slots.get(res).copied().unwrap_or(0.0) >= *amount
            });

            if can_start {
                for (res, amount) in &recipe.inputs {
                    *input_buf.slots.entry(*res).or_default() -= amount;
                }
                state.active = true;
                state.progress = 0.0;
                state.idle_reason = None;
            } else if ratio <= 0.0 {
                state.idle_reason = Some(IdleReason::NoEnergy);
            } else {
                state.idle_reason = Some(IdleReason::NoInputs);
            }
        }

        if state.active {
            state.progress += ratio / recipe.duration_ticks as f32;
            if state.progress >= 1.0 {
                if recipe.output_to_inventory {
                    // Mall buildings: outputs go to Inventory resource
                    if let Some(ref mut inv) = inventory {
                        for (res, amount) in &recipe.outputs {
                            // Attempt to map ResourceType to BuildingType for inventory
                            // For now deposit into inventory resources
                            *inv.resources.entry(*res).or_default() += *amount as u32;
                        }
                    } else {
                        // No inventory resource — fall back to output buffer
                        for (res, amount) in &recipe.outputs {
                            *output_buf.slots.entry(*res).or_default() += amount;
                        }
                    }
                } else {
                    for (res, amount) in &recipe.outputs {
                        *output_buf.slots.entry(*res).or_default() += amount;
                    }
                }
                state.active = false;
                state.progress = 0.0;
            }
        }
    }
}

/// Updates ProductionRates with capacity-based throughput estimates.
///
/// For each building, computes `output_amount / duration_ticks * energy_ratio` as the
/// theoretical production rate per tick. This reflects current capacity (powered buildings
/// contribute their full throughput; unpowered buildings contribute 0) without waiting for
/// a full production cycle to complete.
///
/// Rationale: OpusTree milestones track sustained *throughput*, not accumulated stock.
/// Using capacity-based rates means a well-powered group registers its output rate from
/// tick 1, and losing power immediately zeros the rate — matching both the S7 (win) and
/// S3 (energy crisis) scenarios.
pub fn production_rates_system(
    buildings: Query<(&Recipe, &GroupMember), With<Building>>,
    groups: Query<(&GroupEnergy, &GroupControl), With<Group>>,
    mut rates: ResMut<ProductionRates>,
) {
    rates.rates.clear();
    for (recipe, member) in buildings.iter() {
        let (ratio, paused) = groups
            .get(member.group_id)
            .map(|(ge, ctrl)| (ge.ratio(), ctrl.status == GroupStatus::Paused))
            .unwrap_or((0.0, false));

        if paused || recipe.duration_ticks == 0 {
            continue;
        }

        let rate_per_tick = ratio / recipe.duration_ticks as f32;
        for (res, amount) in &recipe.outputs {
            *rates.rates.entry(*res).or_default() += amount * rate_per_tick;
        }
    }
}
