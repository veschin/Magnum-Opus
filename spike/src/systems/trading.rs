use bevy::prelude::*;

use crate::components::{
    Building, BuildingType, Group, GroupMember, Manifold,
    MetaCurrency, TraderEarnings, TraderState, find_trading_rate,
};

/// Trading system: Trader buildings convert all surplus resources in their group
/// manifold to meta-currency each tick. Applies logarithmic inflation per resource.
pub fn trading_system(
    mut traders: Query<(Entity, &Building, &GroupMember, &mut TraderState, &mut TraderEarnings)>,
    mut manifolds: Query<&mut Manifold, With<Group>>,
) {
    for (_entity, building, member, mut trader_state, mut earnings) in traders.iter_mut() {
        if building.building_type != BuildingType::Trader {
            continue;
        }

        let Ok(mut manifold) = manifolds.get_mut(member.group_id) else {
            continue;
        };

        // Collect resources to trade (we drain them all in one batch)
        let resources_to_trade: Vec<(crate::components::ResourceType, f32)> = manifold
            .resources
            .iter()
            .filter(|(_, amount)| **amount > 0.0)
            .map(|(&res, &amount)| (res, amount))
            .collect();

        for (resource, amount) in resources_to_trade {
            let Some(rate_def) = find_trading_rate(resource) else {
                continue; // resource has no trading rate — skip
            };

            let effective_rate = trader_state.effective_rate(resource, rate_def.rate);
            let earned = amount * effective_rate;

            earnings.add(rate_def.currency, earned);
            trader_state.record_trade(resource, amount);

            // Remove traded resources from manifold
            manifold.resources.remove(&resource);
        }
    }
}

/// Helper: compute earnings for a given manifold snapshot without mutation.
/// Used in tests to assert exact values.
pub fn compute_earnings_for_manifold(
    manifold_resources: &std::collections::HashMap<crate::components::ResourceType, f32>,
    trader_state: &TraderState,
) -> (f32, f32, f32) {
    let mut gold = 0.0_f32;
    let mut souls = 0.0_f32;
    let mut knowledge = 0.0_f32;

    for (&resource, &amount) in manifold_resources {
        let Some(rate_def) = find_trading_rate(resource) else {
            continue;
        };
        let effective_rate = trader_state.effective_rate(resource, rate_def.rate);
        let earned = amount * effective_rate;
        match rate_def.currency {
            MetaCurrency::Gold => gold += earned,
            MetaCurrency::Souls => souls += earned,
            MetaCurrency::Knowledge => knowledge += earned,
        }
    }

    (gold, souls, knowledge)
}
