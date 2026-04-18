//! Manifold collect + distribute passes in Phase::Manifold.
//!
//! Order-sensitive: `.chain()` ensures collect runs before distribute so that
//! outputs produced this tick are eligible to be drawn by consumers in the
//! same tick.

use super::component::Manifold;
use crate::group_formation::{Group, GroupMember};
use crate::recipes_production::{InputBuffer, OutputBuffer, ProductionState, Recipe, RecipeDB};
use bevy::prelude::*;

pub fn manifold_collect_system(
    mut commands: Commands,
    new_groups_q: Query<Entity, (With<Group>, Without<Manifold>)>,
    mut group_manifold_q: Query<&mut Manifold, With<Group>>,
    mut buildings_q: Query<(&GroupMember, &mut OutputBuffer)>,
) {
    for entity in new_groups_q.iter() {
        commands.entity(entity).insert(Manifold::default());
    }

    for (member, mut output) in buildings_q.iter_mut() {
        let Ok(mut manifold) = group_manifold_q.get_mut(member.group) else {
            continue;
        };
        let taken = std::mem::take(&mut output.slots);
        for (resource, amount) in taken {
            *manifold.slots.entry(resource).or_default() += amount;
        }
    }
}

pub fn manifold_distribute_system(
    db: Res<RecipeDB>,
    mut groups_q: Query<&mut Manifold, With<Group>>,
    mut buildings_q: Query<(&Recipe, &ProductionState, &GroupMember, &mut InputBuffer)>,
) {
    for (recipe, state, member, mut inputs) in buildings_q.iter_mut() {
        if state.active {
            continue;
        }
        let Some(def) = db.recipes.get(&recipe.building_type) else {
            continue;
        };
        if def.inputs.is_empty() {
            continue;
        }
        let Ok(mut manifold) = groups_q.get_mut(member.group) else {
            continue;
        };
        let mut shortfalls: Vec<(_, f32)> = Vec::with_capacity(def.inputs.len());
        let mut any_needed = false;
        for (resource, amount) in &def.inputs {
            let have = inputs.slots.get(resource).copied().unwrap_or(0.0);
            let need = amount - have;
            if need > 0.0 {
                any_needed = true;
                shortfalls.push((*resource, need));
            } else {
                shortfalls.push((*resource, 0.0));
            }
        }
        if !any_needed {
            continue;
        }
        let can_pull = shortfalls.iter().all(|(resource, need)| {
            *need <= 0.0 || manifold.slots.get(resource).copied().unwrap_or(0.0) >= *need
        });
        if !can_pull {
            continue;
        }
        for (resource, need) in &shortfalls {
            if *need > 0.0 {
                *manifold.slots.entry(*resource).or_default() -= need;
                *inputs.slots.entry(*resource).or_default() += need;
            }
        }
    }
}
