//! Manifold collect pass: attach Manifold to new Groups, drain OutputBuffers.

use super::component::Manifold;
use crate::group_formation::{Group, GroupMember};
use crate::recipes_production::OutputBuffer;
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
