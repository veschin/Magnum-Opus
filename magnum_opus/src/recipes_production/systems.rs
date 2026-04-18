//! Production system: attach components and advance ProductionState per tick.

use super::component::{InputBuffer, OutputBuffer, ProductionState, Recipe};
use super::resource::RecipeDB;
use crate::buildings::Building;
use bevy::prelude::*;

pub fn production_attach_system(
    mut commands: Commands,
    db: Res<RecipeDB>,
    buildings_q: Query<(Entity, &Building), Without<Recipe>>,
) {
    for (entity, building) in buildings_q.iter() {
        if db.recipes.contains_key(&building.building_type) {
            commands.entity(entity).insert((
                Recipe {
                    building_type: building.building_type,
                },
                ProductionState::default(),
                OutputBuffer::default(),
                InputBuffer::default(),
            ));
        }
    }
}

pub fn production_advance_system(
    db: Res<RecipeDB>,
    mut q: Query<(
        &Recipe,
        &mut ProductionState,
        &mut InputBuffer,
        &mut OutputBuffer,
    )>,
) {
    for (recipe, mut state, mut inputs, mut outputs) in q.iter_mut() {
        let Some(def) = db.recipes.get(&recipe.building_type) else {
            continue;
        };
        if def.duration_ticks == 0 {
            continue;
        }

        if !state.active {
            let inputs_ok = def.inputs.iter().all(|(r, amount)| {
                inputs.slots.get(r).copied().unwrap_or(0.0) >= *amount
            });
            if inputs_ok {
                for (r, amount) in &def.inputs {
                    *inputs.slots.entry(*r).or_default() -= amount;
                }
                state.active = true;
                state.progress = 0.0;
            }
        }

        if state.active {
            state.progress += 1.0 / def.duration_ticks as f32;
            if state.progress >= 1.0 {
                for (r, amount) in &def.outputs {
                    *outputs.slots.entry(*r).or_default() += amount;
                }
                state.active = false;
                state.progress = 0.0;
            }
        }
    }
}
