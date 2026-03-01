use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::components::*;
use crate::events::{BuildingPlaced, BuildingRemoved, PauseGroup, ResumeGroup, SetGroupPriority};

pub fn group_formation_system(
    mut commands: Commands,
    mut ev_placed: MessageReader<BuildingPlaced>,
    mut ev_removed: MessageReader<BuildingRemoved>,
    buildings: Query<(Entity, &Position, &Building, &Footprint), With<Building>>,
    existing_groups: Query<Entity, With<Group>>,
) {
    if ev_placed.is_empty() && ev_removed.is_empty() {
        return;
    }
    ev_placed.read().count();
    ev_removed.read().count();

    // Despawn old group entities
    for group_entity in existing_groups.iter() {
        commands.entity(group_entity).despawn();
    }

    // Build a map: cell -> entity (for adjacency checks including multi-cell footprints)
    let mut cell_to_entity: HashMap<(i32, i32), Entity> = HashMap::new();
    for (entity, _pos, _building, footprint) in buildings.iter() {
        for &cell in &footprint.cells {
            cell_to_entity.insert(cell, entity);
        }
    }

    // Flood-fill connected components by cardinal adjacency
    let mut visited: HashSet<Entity> = HashSet::new();

    for (entity, _pos, building, footprint) in buildings.iter() {
        if visited.contains(&entity) {
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

        let group_id = commands
            .spawn((
                Group,
                Manifold::default(),
                GroupEnergy::default(),
                GroupControl::default(),
                GroupStats::default(),
                GroupType { class: group_class },
            ))
            .id();

        for &e in &group_entities {
            commands.entity(e).insert(GroupMember { group_id });
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
pub fn group_priority_system(
    mut ev: MessageReader<SetGroupPriority>,
    mut controls: Query<&mut GroupControl, With<Group>>,
) {
    for cmd in ev.read() {
        if let Ok(mut ctrl) = controls.get_mut(cmd.group_id) {
            ctrl.priority = cmd.priority;
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
