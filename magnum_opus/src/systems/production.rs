use bevy::prelude::*;

use crate::components::*;
use crate::resources::Inventory;

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
    for (building, recipe, mut state, member, mut input_buf, mut output_buf) in
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
