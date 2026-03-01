use bevy::prelude::*;

use crate::components::*;

pub fn manifold_system(
    mut groups: Query<&mut Manifold, With<Group>>,
    mut buildings: Query<(&GroupMember, &Recipe, &mut OutputBuffer, &mut InputBuffer)>,
) {
    // Pass 1: Collect — drain OutputBuffers into group Manifold
    for (member, _recipe, mut output_buf, _input_buf) in buildings.iter_mut() {
        if let Ok(mut manifold) = groups.get_mut(member.group_id) {
            for (res, amount) in output_buf.slots.drain() {
                if amount > 0.0 {
                    *manifold.resources.entry(res).or_default() += amount;
                }
            }
        }
    }

    // Pass 2: Distribute — fill InputBuffers from group Manifold
    for (member, recipe, _output_buf, mut input_buf) in buildings.iter_mut() {
        if let Ok(mut manifold) = groups.get_mut(member.group_id) {
            for (res, needed) in &recipe.inputs {
                let have = input_buf.slots.get(res).copied().unwrap_or(0.0);
                let deficit = (*needed - have).max(0.0);
                if deficit > 0.0 {
                    let available = manifold.resources.get(res).copied().unwrap_or(0.0);
                    let transfer = deficit.min(available);
                    if transfer > 0.0 {
                        *input_buf.slots.entry(*res).or_default() += transfer;
                        *manifold.resources.entry(*res).or_default() -= transfer;
                    }
                }
            }
        }
    }
}
