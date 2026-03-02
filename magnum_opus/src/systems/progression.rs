//! Progression systems — milestone tracking, tier gates, mini-opus, run lifecycle.
//!
//! Phase: Progression (after Transport)
//!   1. milestone_check_system      — update OpusNodeFull sustain_ticks, emit MilestoneReached
//!   2. opus_tree_sync_system       — sync OpusNodeFull entities → OpusTreeResource
//!   3. run_lifecycle_system        — emit RunWon / RunTimeUp / RunAbandoned
//!   4. tier_gate_system            — process NestCleared → TierGateComponent → TierUnlockedProgression
//!   5. building_tier_upgrade_system — auto-upgrade BuildingTier on TierUnlockedProgression
//!   6. mini_opus_system            — evaluate MiniOpusBranch conditions

use bevy::prelude::*;

use crate::components::{
    MiniOpusBranch, MiniOpusStatus, MiniOpusTrigger, OpusNodeFull,
    TierGateComponent, BuildingTier,
};
use crate::events::{
    MilestoneReached, MiniOpusCompleted, MiniOpusMissed, NestCleared,
    TierUnlockedProgression, RunWon, RunTimeUp, RunAbandoned,
};
use crate::resources::{
    OpusTreeResource, OpusNodeEntry, ProductionRates, RunConfig, RunState, RunStatus, TierState,
};

// ─────────────────────────────────────────────────────────────────────────────
// 1. Milestone check system
// ─────────────────────────────────────────────────────────────────────────────

/// For each `OpusNodeFull` entity:
///   - If not yet sustained: check `ProductionRates` for the node's resource.
///   - If rate >= required_rate: increment sustain_ticks.
///   - Else: reset sustain_ticks to 0.
///   - If sustain_ticks >= RunConfig.sustain_window_ticks: mark sustained, emit MilestoneReached.
pub fn milestone_check_system(
    run_config: Res<RunConfig>,
    rates: Res<ProductionRates>,
    mut nodes: Query<&mut OpusNodeFull>,
    mut milestone_writer: MessageWriter<MilestoneReached>,
) {
    let window = run_config.sustain_window_ticks;

    for mut node in nodes.iter_mut() {
        // Once sustained, never regress.
        if node.sustained {
            continue;
        }

        let current_rate = rates.get(node.resource);
        if current_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }

        if node.sustain_ticks >= window {
            node.sustained = true;
            milestone_writer.write(MilestoneReached {
                node_index: node.node_index,
                resource: node.resource,
            });
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. OpusTree sync system
// ─────────────────────────────────────────────────────────────────────────────

/// Rebuilds `OpusTreeResource.main_path` from `OpusNodeFull` entities,
/// then calls `recalc_completion()` and updates `simultaneous_sustain_ticks`.
pub fn opus_tree_sync_system(
    nodes: Query<&OpusNodeFull>,
    mut tree: ResMut<OpusTreeResource>,
) {
    // Collect nodes sorted by node_index.
    let mut node_vec: Vec<&OpusNodeFull> = nodes.iter().collect();
    node_vec.sort_by_key(|n| n.node_index);

    tree.main_path = node_vec
        .iter()
        .map(|n| OpusNodeEntry {
            node_index: n.node_index,
            resource: n.resource,
            required_rate: n.required_rate,
            current_rate: 0.0,
            tier: n.tier,
            sustained: n.sustained,
        })
        .collect();

    tree.recalc_completion();

    // Update simultaneous sustain tracking: increment if all sustained, else reset.
    if tree.all_sustained() {
        tree.simultaneous_sustain_ticks += 1;
    } else {
        tree.simultaneous_sustain_ticks = 0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Run lifecycle system
// ─────────────────────────────────────────────────────────────────────────────

/// Checks run-end conditions and emits RunWon / RunTimeUp / RunAbandoned.
pub fn run_lifecycle_system(
    run_config: Res<RunConfig>,
    tree: Res<OpusTreeResource>,
    mut run_state: ResMut<RunState>,
    mut won_writer: MessageWriter<RunWon>,
    mut timeout_writer: MessageWriter<RunTimeUp>,
    mut abandoned_writer: MessageWriter<RunAbandoned>,
) {
    if run_state.status != RunStatus::InProgress {
        return;
    }

    if run_config.abandoned {
        run_state.status = RunStatus::Abandoned;
        run_state.currency_earned = 0.0;
        abandoned_writer.write(RunAbandoned);
        return;
    }

    let all_won = tree.all_sustained()
        && tree.simultaneous_sustain_ticks >= tree.sustain_ticks_required;

    if all_won {
        run_state.status = RunStatus::Won;
        won_writer.write(RunWon);
        return;
    }

    if run_config.current_tick >= run_config.max_ticks {
        run_state.status = RunStatus::TimedOut;
        timeout_writer.write(RunTimeUp);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Tier gate system
// ─────────────────────────────────────────────────────────────────────────────

/// Reads `NestCleared` events, finds the matching `TierGateComponent`,
/// sets `unlocked = true`, updates `TierState`, emits `TierUnlockedProgression`.
pub fn tier_gate_system(
    mut nest_reader: MessageReader<NestCleared>,
    mut gates: Query<&mut TierGateComponent>,
    mut tier_state: ResMut<TierState>,
    mut tier_writer: MessageWriter<TierUnlockedProgression>,
) {
    let events: Vec<NestCleared> = nest_reader.read().cloned().collect();
    for event in events {
        for mut gate in gates.iter_mut() {
            if gate.nest_id == event.nest_id && !gate.unlocked {
                gate.unlocked = true;
                if gate.tier as u8 > tier_state.current_tier {
                    tier_state.current_tier = gate.tier as u8;
                }
                tier_writer.write(TierUnlockedProgression { tier: gate.tier });
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Building tier upgrade system
// ─────────────────────────────────────────────────────────────────────────────

/// On `TierUnlockedProgression`, upgrades all `BuildingTier` components
/// that are below the new tier.
pub fn building_tier_upgrade_system(
    mut tier_reader: MessageReader<TierUnlockedProgression>,
    mut buildings: Query<&mut BuildingTier>,
) {
    let events: Vec<TierUnlockedProgression> = tier_reader.read().cloned().collect();
    for event in events {
        for mut bt in buildings.iter_mut() {
            if bt.tier < event.tier {
                bt.tier = event.tier;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Mini-opus check system
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluates `MiniOpusBranch` components each tick.
pub fn mini_opus_system(
    run_config: Res<RunConfig>,
    mut branches: Query<&mut MiniOpusBranch>,
    mut completed_writer: MessageWriter<MiniOpusCompleted>,
    mut missed_writer: MessageWriter<MiniOpusMissed>,
) {
    let tick = run_config.current_tick;

    for mut branch in branches.iter_mut() {
        if branch.status != MiniOpusStatus::Active {
            continue;
        }

        match branch.trigger {
            MiniOpusTrigger::OnDemand => {
                if branch.condition_value >= branch.condition_threshold {
                    branch.status = MiniOpusStatus::Completed;
                    completed_writer.write(MiniOpusCompleted {
                        id: branch.id.clone(),
                        reward_currency: branch.reward_currency,
                        reward_amount: branch.reward_amount,
                    });
                }
            }
            MiniOpusTrigger::TimeBased => {
                if let Some(deadline) = branch.deadline_tick {
                    if tick >= deadline {
                        branch.status = MiniOpusStatus::Missed;
                        missed_writer.write(MiniOpusMissed { id: branch.id.clone() });
                    } else if branch.condition_value >= branch.condition_threshold {
                        branch.status = MiniOpusStatus::Completed;
                        completed_writer.write(MiniOpusCompleted {
                            id: branch.id.clone(),
                            reward_currency: branch.reward_currency,
                            reward_amount: branch.reward_amount,
                        });
                    }
                }
            }
            MiniOpusTrigger::Conditional => {
                if branch.condition_value >= branch.condition_threshold {
                    branch.status = MiniOpusStatus::Completed;
                    completed_writer.write(MiniOpusCompleted {
                        id: branch.id.clone(),
                        reward_currency: branch.reward_currency,
                        reward_amount: branch.reward_amount,
                    });
                }
            }
        }
    }
}
