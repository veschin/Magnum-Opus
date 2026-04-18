//! Full-rebuild group-formation. See F7 PRD for rationale.
//!
//! Optimisation added for F6: the system captures a Local signature of the
//! current Building layout and skips the entire rebuild when the signature
//! matches the previous tick. This keeps `Group` entity ids stable across
//! ticks so that `Manifold` components attached by F6 persist accumulated
//! resources. When placement adds or removes a Building, the signature
//! changes and the full flood-fill rebuild runs.

use super::component::{Group, GroupMember};
use super::resource::GroupIndex;
use crate::buildings::Building;
use crate::grid::Position;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn group_formation_system(
    mut commands: Commands,
    mut index: ResMut<GroupIndex>,
    buildings_q: Query<(Entity, &Position), With<Building>>,
    existing_members_q: Query<Entity, With<GroupMember>>,
    existing_groups_q: Query<Entity, With<Group>>,
    mut last_signature: Local<Option<BTreeSet<(u32, u32, Entity)>>>,
) {
    let current_signature: BTreeSet<(u32, u32, Entity)> = buildings_q
        .iter()
        .map(|(e, p)| (p.x, p.y, e))
        .collect();

    if last_signature.as_ref() == Some(&current_signature) {
        return;
    }
    *last_signature = Some(current_signature.clone());

    for entity in existing_groups_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_members_q.iter() {
        commands.entity(entity).remove::<GroupMember>();
    }
    index.groups.clear();
    index.member_to_group.clear();

    let mut tile_to_entity: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
    for (entity, pos) in buildings_q.iter() {
        tile_to_entity.insert((pos.x, pos.y), entity);
    }

    let mut visited: BTreeSet<(u32, u32)> = BTreeSet::new();
    for &start in tile_to_entity.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut cluster: Vec<Entity> = Vec::new();
        let mut stack: Vec<(u32, u32)> = vec![start];
        while let Some(cur) = stack.pop() {
            if !visited.insert(cur) {
                continue;
            }
            let Some(&entity) = tile_to_entity.get(&cur) else {
                continue;
            };
            cluster.push(entity);
            let (x, y) = cur;
            let mut neighbors = Vec::with_capacity(4);
            neighbors.push((x + 1, y));
            neighbors.push((x, y + 1));
            if let Some(nx) = x.checked_sub(1) {
                neighbors.push((nx, y));
            }
            if let Some(ny) = y.checked_sub(1) {
                neighbors.push((x, ny));
            }
            for n in neighbors {
                if !visited.contains(&n) && tile_to_entity.contains_key(&n) {
                    stack.push(n);
                }
            }
        }
        let group_id = commands.spawn(Group).id();
        index.groups.insert(group_id);
        for member in &cluster {
            index.member_to_group.insert(*member, group_id);
            commands
                .entity(*member)
                .insert(GroupMember { group: group_id });
        }
    }
}
