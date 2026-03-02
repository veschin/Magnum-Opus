//! Creature systems for the Creatures & Combat feature.
//!
//! Implements:
//! - `creature_behavior_system`  — state transitions for Ambient/Territorial/Invasive/EventBorn
//! - `combat_group_system`       — CombatGroup supply ratio computation and organic output
//! - `nest_clearing_system`      — CombatPressure vs nest strength, tier unlocks, loot drops
//! - `minion_task_system`        — idle minions auto-transition to Decorating

use bevy::prelude::*;

use crate::components::{
    AmbientData, Building, BuildingType, CombatGroup, CombatPressure, Creature,
    CreatureArchetype, CreatureNest, CreatureStateKind, EventBornData, Group, GroupMember,
    InvasiveData, LootTable, Manifold, NestHostility, Position, ResourceType,
    TerritoryData, Minion, MinionTask,
};
use crate::events::{NestCleared, TierUnlockedProgression};
use crate::resources::TierState;

// ─────────────────────────────────────────────────────────────────────────────
// Creature Behavior System
//
// Per-tick state machine for all creature archetypes.
// ─────────────────────────────────────────────────────────────────────────────

/// Updates creature states each tick based on their archetype data and world context.
///
/// - **Ambient** (`AmbientData`): flee if health < flee_threshold fraction of max_health.
/// - **Territorial** (`TerritoryData`): become Aggressive when any building is inside radius.
/// - **Invasive** (`InvasiveData`): expand territory radius each tick; spawn child at threshold.
/// - **EventBorn** (`EventBornData`): despawn when ticks_alive >= lifetime_ticks.
/// - **OpusLinked** (`OpusLinkedData`): no per-tick state change — spawned by nest_clearing_system.
pub fn creature_behavior_system(
    mut commands: Commands,
    mut creatures: Query<(Entity, &mut Creature, Option<&AmbientData>, Option<&TerritoryData>, Option<&mut EventBornData>)>,
    buildings: Query<&Position, With<Building>>,
) {
    for (entity, mut creature, ambient_opt, territory_opt, event_opt) in creatures.iter_mut() {
        match creature.archetype {
            // ── Ambient: flee when health fraction < flee_threshold ──────────
            CreatureArchetype::Ambient => {
                if let Some(ambient) = ambient_opt {
                    let health_ratio = creature.health / creature.max_health;
                    if health_ratio < ambient.flee_threshold {
                        creature.state = CreatureStateKind::Fleeing;
                    } else if creature.state == CreatureStateKind::Fleeing {
                        // Recovered above threshold — resume wandering
                        creature.state = CreatureStateKind::Wandering;
                    } else if creature.state == CreatureStateKind::Idle {
                        creature.state = CreatureStateKind::Wandering;
                    }
                }
            }

            // ── Territorial: Aggressive when building within radius ──────────
            CreatureArchetype::Territorial => {
                if let Some(territory) = territory_opt {
                    let center = (territory.center_x, territory.center_y);
                    let mut triggered = false;
                    for pos in buildings.iter() {
                        let dx = (pos.x - center.0) as f32;
                        let dy = (pos.y - center.1) as f32;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < territory.radius {
                            triggered = true;
                            break;
                        }
                    }
                    creature.state = if triggered {
                        CreatureStateKind::Aggressive
                    } else {
                        CreatureStateKind::Patrolling
                    };
                }
            }

            // ── Invasive: territory expansion handled by invasive_expansion_system ─
            CreatureArchetype::Invasive => {
                // State reflects current territory state.
                // Expansion radius is updated by invasive_expansion_system each tick.
                // If any combat group covers this creature, state becomes Patrolling (suppressed).
                if creature.state == CreatureStateKind::Idle {
                    creature.state = CreatureStateKind::Patrolling;
                }
            }

            // ── EventBorn: despawn on lifetime expiry ────────────────────────
            CreatureArchetype::EventBorn => {
                if let Some(mut event) = event_opt {
                    event.ticks_alive += 1;
                    if event.ticks_alive >= event.lifetime_ticks {
                        creature.state = CreatureStateKind::Despawned;
                        commands.entity(entity).despawn();
                    } else {
                        creature.state = CreatureStateKind::Aggressive;
                    }
                }
            }

            // ── OpusLinked: passive until milestone triggers spawn ────────────
            CreatureArchetype::OpusLinked => {
                // Spawned externally when opus milestone is reached.
                // Once spawned: always Aggressive (highest-tier threat).
                if creature.state == CreatureStateKind::Idle {
                    creature.state = CreatureStateKind::Aggressive;
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Invasive Territory Expansion System
//
// Separate system to update TerritoryData.radius on Invasive creatures each tick.
// ─────────────────────────────────────────────────────────────────────────────

/// Expands territory radius of invasive creatures each tick unless suppressed by a combat group.
pub fn invasive_expansion_system(
    mut invasive_creatures: Query<(&Creature, &mut TerritoryData, &InvasiveData, Option<&Position>)>,
    combat_groups: Query<(&Position, &CombatGroup)>,
) {
    for (creature, mut territory, invasive, pos_opt) in invasive_creatures.iter_mut() {
        if creature.archetype != CreatureArchetype::Invasive {
            continue;
        }

        // Check if any combat group suppresses this creature's territory
        let creature_pos = pos_opt.map(|p| (p.x, p.y)).unwrap_or((territory.center_x, territory.center_y));

        let suppressed = combat_groups.iter().any(|(cg_pos, cg)| {
            let dx = (cg_pos.x - creature_pos.0) as f32;
            let dy = (cg_pos.y - creature_pos.1) as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            dist <= cg.effective_protection_radius() && cg.effective_protection_dps() > 0.0
        });

        if !suppressed {
            territory.radius += invasive.expansion_rate;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Combat Group System
//
// Computes supply_ratio from manifold resources vs. consumption needs.
// Outputs: organic rate, protection radius, protection DPS — all stored in CombatGroup.
// ─────────────────────────────────────────────────────────────────────────────

/// Updates each CombatGroup's supply_ratio based on available manifold resources,
/// then deposits effective_organic_rate into the group manifold as organic output.
pub fn combat_group_system(
    mut combat_buildings: Query<(&Building, &GroupMember, &mut CombatGroup)>,
    mut manifolds: Query<&mut Manifold, With<Group>>,
) {
    // Compute supply ratio per group: min(available_food / required_food, 1.0)
    // Food requirement: max_minions * 0.5 herbs per cycle (simplified seed model)
    const HERBS_PER_MINION: f32 = 0.5;

    for (building, member, mut combat) in combat_buildings.iter_mut() {
        // Only process combat building types
        match building.building_type {
            BuildingType::ImpCamp | BuildingType::BreedingPen | BuildingType::WarLodge => {}
            _ => continue,
        }

        let Ok(mut manifold) = manifolds.get_mut(member.group_id) else {
            continue;
        };

        // Compute supply ratio from available herbs
        let required = combat.max_minions as f32 * HERBS_PER_MINION;
        let available = manifold.resources.get(&ResourceType::Herbs).copied().unwrap_or(0.0);
        let ratio = if required > 0.0 {
            (available / required).clamp(0.0, 1.0)
        } else {
            1.0
        };
        combat.supply_ratio = ratio;

        // Consume herbs proportional to supply
        let consumed = required * ratio * combat.consumption_multiplier;
        if consumed > 0.0 {
            let herbs = manifold.resources.entry(ResourceType::Herbs).or_default();
            *herbs = (*herbs - consumed).max(0.0);
        }

        // Deposit organic output (hide as the representative organic resource)
        let organic_out = combat.effective_organic_rate();
        if organic_out > 0.0 {
            *manifold.resources.entry(ResourceType::Hide).or_default() += organic_out;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Nest Clearing System
//
// Checks accumulated CombatPressure against nest strength.
// On clear: marks nest cleared, emits NestCleared event, advances TierState for hostile nests.
// ─────────────────────────────────────────────────────────────────────────────

/// Clears nests when accumulated CombatPressure exceeds their strength.
/// Hostile nests advance TierState on clear; neutral nests only drop loot.
pub fn nest_clearing_system(
    mut nests: Query<(&mut CreatureNest, &CombatPressure)>,
    mut tier_state: ResMut<TierState>,
    mut ev_nest_cleared: MessageWriter<NestCleared>,
    mut ev_tier_unlocked: MessageWriter<TierUnlockedProgression>,
) {
    for (mut nest, pressure) in nests.iter_mut() {
        if nest.cleared {
            continue; // already cleared — skip
        }

        if pressure.value > nest.strength {
            nest.cleared = true;

            // Emit nest cleared event
            ev_nest_cleared.write(NestCleared {
                nest_id: format!("{:?}", nest.nest_id),
            });

            // Only hostile nests advance the tier gate
            if nest.hostility == NestHostility::Hostile {
                let new_tier = tier_state.current_tier + 1;
                tier_state.current_tier = new_tier;

                ev_tier_unlocked.write(TierUnlockedProgression {
                    tier: new_tier as u32,
                });
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Combat Pressure Accumulation System
//
// Each tick: accumulate DPS from all combat groups within territory radius of each nest.
// ─────────────────────────────────────────────────────────────────────────────

/// Accumulates CombatPressure on nests from nearby combat groups each tick.
/// Pressure = sum of effective_protection_dps() for groups within nest territory_radius.
///
/// CombatGroup source: Buildings (ImpCamp / BreedingPen / WarLodge) with a GroupMember —
/// group position is resolved via the group entity's Position component.
pub fn combat_pressure_system(
    mut nests: Query<(&CreatureNest, &mut CombatPressure, Option<&Position>)>,
    combat_buildings: Query<(&Building, &GroupMember, &CombatGroup)>,
    group_positions: Query<&Position, With<Group>>,
) {
    for (nest, mut pressure, nest_pos_opt) in nests.iter_mut() {
        if nest.cleared {
            pressure.value = 0.0;
            continue;
        }

        let nest_pos = match nest_pos_opt {
            Some(p) => (p.x, p.y),
            None => continue,
        };

        let mut total_pressure = 0.0f32;

        // CombatGroup on Building entities with a GroupMember (normal in-game path).
        // Group position is resolved from the group entity for range calculation.
        for (building, member, combat) in combat_buildings.iter() {
            match building.building_type {
                BuildingType::ImpCamp | BuildingType::BreedingPen | BuildingType::WarLodge => {}
                _ => continue,
            }

            // Get the group's position for range calculation
            let group_pos = match group_positions.get(member.group_id) {
                Ok(p) => (p.x, p.y),
                Err(_) => continue,
            };

            let dx = (group_pos.0 - nest_pos.0) as f32;
            let dy = (group_pos.1 - nest_pos.1) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= nest.territory_radius {
                total_pressure += combat.effective_protection_dps();
            }
        }

        pressure.value += total_pressure;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Loot Drop System
//
// When a creature's health reaches zero, deposit its LootTable into the nearest
// combat group manifold and despawn the creature entity.
// ─────────────────────────────────────────────────────────────────────────────

/// Detects dead creatures (health <= 0), deposits loot into the nearest combat group
/// manifold, and despawns the creature entity.
pub fn creature_loot_system(
    mut commands: Commands,
    dead_creatures: Query<(Entity, &Creature, &LootTable, Option<&Position>)>,
    combat_buildings: Query<(&Building, &GroupMember, Option<&Position>)>,
    mut manifolds: Query<&mut Manifold, With<Group>>,
) {
    for (entity, creature, loot, creature_pos_opt) in dead_creatures.iter() {
        if creature.health > 0.0 {
            continue;
        }

        let creature_pos = match creature_pos_opt {
            Some(p) => (p.x, p.y),
            None => {
                commands.entity(entity).despawn();
                continue;
            }
        };

        // Find the nearest combat group
        let mut nearest_group: Option<(Entity, f32)> = None;

        for (building, member, cg_pos_opt) in combat_buildings.iter() {
            match building.building_type {
                BuildingType::ImpCamp | BuildingType::BreedingPen | BuildingType::WarLodge => {}
                _ => continue,
            }

            let cg_pos = match cg_pos_opt {
                Some(p) => (p.x, p.y),
                None => continue,
            };

            let dx = (cg_pos.0 - creature_pos.0) as f32;
            let dy = (cg_pos.1 - creature_pos.1) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            match nearest_group {
                None => nearest_group = Some((member.group_id, dist)),
                Some((_, best_dist)) if dist < best_dist => {
                    nearest_group = Some((member.group_id, dist));
                }
                _ => {}
            }
        }

        // Deposit loot into nearest manifold
        if let Some((group_id, _)) = nearest_group {
            if let Ok(mut manifold) = manifolds.get_mut(group_id) {
                for (&resource, &amount) in &loot.drops {
                    *manifold.resources.entry(resource).or_default() += amount as f32;
                }
            }
        }

        commands.entity(entity).despawn();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Minion Task System
//
// Idle minions auto-transition to Decorating; remove Decorating when assigned.
// ─────────────────────────────────────────────────────────────────────────────

/// Transitions idle minions to Decorating state when no tasks are available.
/// When a production task is assigned (task != Idle), clears the Decorating flag.
pub fn minion_task_system(mut minions: Query<&mut Minion>) {
    for mut minion in minions.iter_mut() {
        match minion.task {
            MinionTask::Idle => {
                minion.task = MinionTask::Decorating;
            }
            MinionTask::Decorating => {
                // Remain decorating until assigned a production task
            }
            MinionTask::Production => {
                // Producing — no change needed
            }
        }
    }
}
