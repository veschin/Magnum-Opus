use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::components::*;
use crate::events::{BuildingPlaced, BuildingRemoved, BuildingDestroyed, PauseGroup, ResumeGroup, SetGroupPriority};

pub fn group_formation_system(
    mut commands: Commands,
    mut ev_placed: MessageReader<BuildingPlaced>,
    mut ev_removed: MessageReader<BuildingRemoved>,
    mut ev_destroyed: MessageReader<BuildingDestroyed>,
    buildings: Query<(Entity, &Position, &Building, &Footprint), With<Building>>,
    members: Query<(Entity, &GroupMember), With<Building>>,
    existing_groups: Query<(Entity, &Manifold), With<Group>>,
    ungrouped: Query<Entity, (With<Building>, With<Footprint>, Without<GroupMember>)>,
) {
    let has_ungrouped = !ungrouped.is_empty();
    if ev_placed.is_empty() && ev_removed.is_empty() && ev_destroyed.is_empty() && !has_ungrouped {
        return;
    }
    ev_placed.read().count();
    ev_removed.read().count();

    // Collect destroyed entity IDs so we can exclude them from the flood-fill even when
    // their despawn command is still pending (i.e., hazard_trigger_system ran in the same
    // tick before group_formation_system flushed commands).
    let destroyed_entities: HashSet<Entity> = ev_destroyed.read().map(|e| e.entity).collect();

    // Snapshot old manifold contents keyed by group entity before despawn.
    let mut old_manifolds: HashMap<Entity, HashMap<ResourceType, f32>> = HashMap::new();
    for (group_entity, manifold) in existing_groups.iter() {
        if !manifold.resources.is_empty() {
            old_manifolds.insert(group_entity, manifold.resources.clone());
        }
    }

    // Map each building entity to its old group entity.
    let mut building_to_old_group: HashMap<Entity, Entity> = HashMap::new();
    for (building_entity, member) in members.iter() {
        building_to_old_group.insert(building_entity, member.group_id);
    }

    // Despawn old group entities
    for (group_entity, _) in existing_groups.iter() {
        commands.entity(group_entity).despawn();
    }

    // Build a map: cell -> entity (for adjacency checks including multi-cell footprints).
    // Exclude entities that were just destroyed (their despawn may be deferred).
    let mut cell_to_entity: HashMap<(i32, i32), Entity> = HashMap::new();
    for (entity, _pos, _building, footprint) in buildings.iter() {
        if destroyed_entities.contains(&entity) {
            continue;
        }
        for &cell in &footprint.cells {
            cell_to_entity.insert(cell, entity);
        }
    }

    // Flood-fill connected components by cardinal adjacency
    let mut visited: HashSet<Entity> = HashSet::new();

    for (entity, _pos, building, _footprint) in buildings.iter() {
        if visited.contains(&entity) {
            continue;
        }
        // Skip entities destroyed this tick (despawn may be pending).
        if destroyed_entities.contains(&entity) {
            visited.insert(entity);
            continue;
        }

        let mut group_entities = Vec::new();
        let mut queue = VecDeque::new();
        visited.insert(entity);
        queue.push_back(entity);

        while let Some(cur_entity) = queue.pop_front() {
            group_entities.push(cur_entity);

            // Get all cells of cur_entity
            if let Ok((_, _, _, fp)) = buildings.get(cur_entity) {
                for &(cx, cy) in &fp.cells {
                    for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                        let neighbor_cell = (cx + dx, cy + dy);
                        if let Some(&neighbor_entity) = cell_to_entity.get(&neighbor_cell) {
                            if neighbor_entity != cur_entity && !visited.contains(&neighbor_entity) {
                                visited.insert(neighbor_entity);
                                queue.push_back(neighbor_entity);
                            }
                        }
                    }
                }
            }
        }

        // Determine group class from majority of building types
        let group_class = determine_group_class(&group_entities, &buildings);

        // Merge old manifold contents from all old groups that contributed buildings to this new group.
        let mut merged_resources: HashMap<ResourceType, f32> = HashMap::new();
        let mut seen_old_groups: HashSet<Entity> = HashSet::new();
        for &be in &group_entities {
            if let Some(&old_group) = building_to_old_group.get(&be) {
                if seen_old_groups.insert(old_group) {
                    if let Some(old_res) = old_manifolds.get(&old_group) {
                        for (res, amt) in old_res {
                            *merged_resources.entry(*res).or_default() += amt;
                        }
                    }
                }
            }
        }

        let initial_manifold = Manifold { resources: merged_resources };

        // Compute centroid of all building positions in this group.
        let (sum_x, sum_y, count) = group_entities.iter()
            .filter_map(|&e| buildings.get(e).ok())
            .fold((0i32, 0i32, 0usize), |(sx, sy, c), (_, pos, _, _)| {
                (sx + pos.x, sy + pos.y, c + 1)
            });
        let group_position = if count > 0 {
            GroupPosition { x: sum_x / count as i32, y: sum_y / count as i32 }
        } else {
            GroupPosition { x: 0, y: 0 }
        };

        let group_id = commands
            .spawn((
                Group,
                initial_manifold,
                GroupEnergy::default(),
                GroupControl::default(),
                GroupStats::default(),
                GroupType { class: group_class },
                group_position,
            ))
            .id();

        for &e in &group_entities {
            commands.entity(e).try_insert(GroupMember { group_id });
        }

        // Determine and store boundary cells as OutputSender / InputReceiver availability
        // (spawning boundary port entities is future work — here we just tag the group)
        let _ = building; // used in outer loop header
    }
}

/// Determines the dominant group class for a set of buildings.
fn determine_group_class(
    entities: &[Entity],
    buildings: &Query<(Entity, &Position, &Building, &Footprint), With<Building>>,
) -> GroupClass {
    let mut class_counts: HashMap<GroupClass, usize> = HashMap::new();
    for &e in entities {
        if let Ok((_, _, b, _)) = buildings.get(e) {
            *class_counts.entry(b.building_type.group_class()).or_default() += 1;
        }
    }

    // Priority: Combat > Mall > Extraction > Synthesis > Energy > Opus > Utility
    for class in [
        GroupClass::Combat,
        GroupClass::Mall,
        GroupClass::Extraction,
        GroupClass::Synthesis,
        GroupClass::Energy,
        GroupClass::Opus,
        GroupClass::Utility,
    ] {
        if class_counts.get(&class).copied().unwrap_or(0) > 0 {
            return class;
        }
    }

    GroupClass::Synthesis
}

/// Handles SetGroupPriority commands.
/// Updates both GroupControl.priority (management priority) and GroupEnergy.priority
/// (energy allocation priority) so the energy system reflects the change immediately.
pub fn group_priority_system(
    mut ev: MessageReader<SetGroupPriority>,
    mut groups: Query<(&mut GroupControl, &mut GroupEnergy), With<Group>>,
) {
    for cmd in ev.read() {
        if let Ok((mut ctrl, mut ge)) = groups.get_mut(cmd.group_id) {
            ctrl.priority = cmd.priority;
            // Sync GroupEnergy.priority from GroupPriority command
            ge.priority = match cmd.priority {
                crate::components::GroupPriority::High => EnergyPriority::High,
                crate::components::GroupPriority::Medium => EnergyPriority::Medium,
                crate::components::GroupPriority::Low => EnergyPriority::Low,
            };
        }
    }
}

/// Handles PauseGroup / ResumeGroup commands.
pub fn group_pause_system(
    mut ev_pause: MessageReader<PauseGroup>,
    mut ev_resume: MessageReader<ResumeGroup>,
    mut controls: Query<&mut GroupControl, With<Group>>,
) {
    for cmd in ev_pause.read() {
        if let Ok(mut ctrl) = controls.get_mut(cmd.group_id) {
            ctrl.status = GroupStatus::Paused;
        }
    }
    for cmd in ev_resume.read() {
        if let Ok(mut ctrl) = controls.get_mut(cmd.group_id) {
            ctrl.status = GroupStatus::Active;
        }
    }
}
