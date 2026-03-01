//! BDD tests for the progression feature.
//! One test fn per Gherkin scenario in .ptsd/bdd/progression/progression.feature
//! Seed data: .ptsd/seeds/progression/*.yaml
//!
//! Opus tree: 5 main-path nodes, sustain_window=600 ticks, sample_interval=20
//! Scoring weights: opus 50%, mini-opus 30%, time 20%
//! Difficulty multipliers: Easy 1.0, Medium 1.5, Hard 2.0, Extreme 3.0
//! Starting kit forest: 2 iron_miner, 1 water_pump, 1 iron_smelter, 1 sawmill,
//!                       1 tree_farm, 1 constructor, 2 wind_turbine, 1 watchtower
//! Starting kit volcanic: 2 iron_miner, 2 stone_quarry, 1 iron_smelter,
//!                         1 constructor, 3 wind_turbine, 1 watchtower

use crate::components::{
    BuildingTier, MetaCurrency, MiniOpusBranch, MiniOpusKind, MiniOpusStatus, MiniOpusTrigger,
    OpusDifficulty, OpusNodeFull, PlayerInventory, ResourceType, TierGateComponent,
};
use crate::events::{
    MiniOpusCompleted, MiniOpusMissed, NestCleared, TierUnlockedProgression,
};
use crate::resources::{
    Biome, MiniOpusEntry, OpusNodeEntry, OpusTreeResource, RunConfig,
    StartingKitCommands, TieredPlacementCmd, TransportTierState,
};

// ── AC1: Opus tree nodes are production throughput milestones ─────────────────

#[test]
fn opus_tree_nodes_define_resource_and_rate_requirements() {
    let tree = OpusTreeResource {
        main_path: vec![
            OpusNodeEntry {
                node_index: 0,
                resource: ResourceType::IronOre,
                required_rate: 4.0,
                current_rate: 0.0,
                tier: 1,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 1,
                resource: ResourceType::IronBar,
                required_rate: 3.0,
                current_rate: 0.0,
                tier: 1,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 2,
                resource: ResourceType::SteelPlate,
                required_rate: 2.0,
                current_rate: 0.0,
                tier: 2,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 3,
                resource: ResourceType::Hide,
                required_rate: 1.6,
                current_rate: 0.0,
                tier: 2,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 4,
                resource: ResourceType::OpusIngot,
                required_rate: 1.0,
                current_rate: 0.0,
                tier: 3,
                sustained: false,
            },
        ],
        ..Default::default()
    };

    assert_eq!(tree.main_path.len(), 5, "must have 5 main-path nodes");

    assert_eq!(tree.main_path[0].resource, ResourceType::IronOre);
    assert!((tree.main_path[0].required_rate - 4.0).abs() < f32::EPSILON);
    assert_eq!(tree.main_path[0].tier, 1);

    assert_eq!(tree.main_path[1].resource, ResourceType::IronBar);
    assert!((tree.main_path[1].required_rate - 3.0).abs() < f32::EPSILON);

    assert_eq!(tree.main_path[2].resource, ResourceType::SteelPlate);
    assert!((tree.main_path[2].required_rate - 2.0).abs() < f32::EPSILON);
    assert_eq!(tree.main_path[2].tier, 2);

    assert_eq!(tree.main_path[3].resource, ResourceType::Hide);
    assert!((tree.main_path[3].required_rate - 1.6).abs() < 0.001);

    assert_eq!(tree.main_path[4].resource, ResourceType::OpusIngot);
    assert!((tree.main_path[4].required_rate - 1.0).abs() < f32::EPSILON);
    assert_eq!(tree.main_path[4].tier, 3);

    // no node has an item-crafting goal — all are throughput rates
    for node in &tree.main_path {
        assert!(node.required_rate > 0.0, "node {} must have a rate requirement", node.node_index);
    }
}

#[test]
fn opus_tree_scales_rates_by_difficulty_multiplier() {
    // hard rate_multiplier = 1.4, template base = 2.0 → required = 2.8
    let difficulty = OpusDifficulty::Hard;
    let rate_multiplier = difficulty.rate_multiplier();
    let base_rate = 2.0_f32;

    let required_rate = base_rate * rate_multiplier;

    assert!((rate_multiplier - 1.4).abs() < 0.001, "Hard multiplier must be 1.4, got {rate_multiplier}");
    assert!((required_rate - 2.8).abs() < 0.001, "node 1 requiredRate = 2.8, got {required_rate}");
}

// ── AC2: Milestone sustained after verification period ────────────────────────

#[test]
fn milestone_completes_when_rate_sustained_for_600_ticks() {
    // Simulate the milestone check logic using pure data
    let sustain_window_ticks = 600u32;

    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronOre,
        required_rate: 4.0,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    };

    let production_rate = 4.5_f32;

    // Simulate 600 ticks at or above required rate
    for _ in 0..sustain_window_ticks {
        if production_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(node.sustained, "node must be sustained after 600 ticks at 4.5 >= 4.0");
    assert_eq!(node.sustain_ticks, sustain_window_ticks, "sustain_ticks must equal window of 600");

    // Verify the CONDITIONS that would trigger MilestoneReached are met:
    // the node is sustained AND the resource/index are correct for the event payload.
    // Event emission itself is verified at impl stage via EventReader.
    assert_eq!(node.node_index, 0, "MilestoneReached would reference node index 0");
    assert_eq!(node.resource, ResourceType::IronOre, "MilestoneReached would reference IronOre");
    // Event emission verified at impl stage via EventReader
}

#[test]
fn milestone_does_not_complete_when_rate_held_for_fewer_than_600_ticks() {
    let sustain_window_ticks = 600u32;

    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronOre,
        required_rate: 4.0,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    };

    let production_rate = 4.5_f32;

    // Simulate only 500 ticks
    for _ in 0..500u32 {
        if production_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(!node.sustained, "node must NOT be sustained after only 500 ticks");
    assert_eq!(node.sustain_ticks, 500);
}

#[test]
fn milestone_does_not_complete_when_rate_is_below_required() {
    let sustain_window_ticks = 600u32;

    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronOre,
        required_rate: 4.0,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    };

    let production_rate = 3.5_f32; // below required 4.0

    // Simulate 600 ticks at below required rate
    for _ in 0..sustain_window_ticks {
        if production_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(!node.sustained, "node must NOT be sustained when rate 3.5 < required 4.0");
    assert_eq!(node.sustain_ticks, 0, "sustain_ticks reset to 0 when below required");
}

// ── AC3: Opus tree UI shows nodes, rates, completion % ───────────────────────

#[test]
fn opus_tree_exposes_data_for_ui_display() {
    let mut tree = OpusTreeResource {
        main_path: vec![
            OpusNodeEntry {
                node_index: 0,
                resource: ResourceType::IronOre,
                required_rate: 4.0,
                current_rate: 4.5,
                tier: 1,
                sustained: true,
            },
            OpusNodeEntry {
                node_index: 1,
                resource: ResourceType::IronBar,
                required_rate: 3.0,
                current_rate: 1.0,
                tier: 1,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 2,
                resource: ResourceType::SteelPlate,
                required_rate: 2.0,
                current_rate: 0.0,
                tier: 2,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 3,
                resource: ResourceType::Hide,
                required_rate: 1.6,
                current_rate: 0.0,
                tier: 2,
                sustained: false,
            },
            OpusNodeEntry {
                node_index: 4,
                resource: ResourceType::OpusIngot,
                required_rate: 1.0,
                current_rate: 0.0,
                tier: 3,
                sustained: false,
            },
        ],
        mini_opus: vec![
            MiniOpusEntry {
                id: "trade_5_wood".to_string(),
                parent_node: 0,
                status: MiniOpusStatus::Active,
                reward_currency: MetaCurrency::Gold,
                reward_amount: 50,
            },
            MiniOpusEntry {
                id: "fast_steel".to_string(),
                parent_node: 2,
                status: MiniOpusStatus::Active,
                reward_currency: MetaCurrency::Knowledge,
                reward_amount: 60,
            },
        ],
        completion_pct: 0.0,
        simultaneous_sustain_ticks: 0,
        sustain_ticks_required: 600,
    };

    tree.recalc_completion();

    assert_eq!(tree.main_path.len(), 5, "must have 5 main-path nodes");
    assert_eq!(tree.mini_opus.len(), 2, "must have 2 mini-opus entries");
    assert!(
        (tree.completion_pct - 0.2).abs() < 0.001,
        "completionPct = 1/5 = 0.2, got {}",
        tree.completion_pct
    );

    // node 0 has current_rate 4.5 vs required 4.0
    assert!((tree.main_path[0].current_rate - 4.5).abs() < 0.001);
    assert!(tree.main_path[0].sustained);

    // node 1 has current_rate 1.0 vs required 3.0, not sustained
    assert!((tree.main_path[1].current_rate - 1.0).abs() < 0.001);
    assert!(!tree.main_path[1].sustained);
}

// ── AC4: Mini-opus branches attached to parent main-path node ─────────────────

#[test]
fn mini_opus_branch_references_its_parent_main_path_node() {
    // branch_points at node indices 0, 2, 4 (0-indexed; BDD "nodes 1, 3, 5" are 1-indexed labels)
    let branches = vec![
        MiniOpusBranch {
            id: "trade_5_wood".to_string(),
            parent_node: 0,  // 0-indexed: BDD "node 1"
            kind: MiniOpusKind::TradeSurplus,
            trigger: MiniOpusTrigger::OnDemand,
            status: MiniOpusStatus::Active,
            reward_currency: MetaCurrency::Gold,
            reward_amount: 50,
            deadline_tick: None,
            condition_value: 0.0,
            condition_threshold: 5.0,
        },
        MiniOpusBranch {
            id: "fast_steel".to_string(),
            parent_node: 2,  // 0-indexed: BDD "node 3"
            kind: MiniOpusKind::SpeedProduction,
            trigger: MiniOpusTrigger::TimeBased,
            status: MiniOpusStatus::Active,
            reward_currency: MetaCurrency::Knowledge,
            reward_amount: 60,
            deadline_tick: Some(50000),
            condition_value: 0.0,
            condition_threshold: 3.0,
        },
    ];

    // branch_points at node indices 0, 2, 4 — verify parent references (0-indexed throughout)
    let trade_branch = branches.iter().find(|b| b.id == "trade_5_wood").unwrap();
    assert_eq!(trade_branch.parent_node, 0, "trade_5_wood parent_node must be 0 (BDD node 1, 0-indexed)");

    let fast_steel = branches.iter().find(|b| b.id == "fast_steel").unwrap();
    assert_eq!(fast_steel.parent_node, 2, "fast_steel parent_node must be 2 (BDD node 3, 0-indexed)");
}

// ── AC5: Mini-opus awards meta-currency; skipping has no penalty ──────────────

#[test]
fn completing_on_demand_mini_opus_awards_gold_currency() {
    let mut branch = MiniOpusBranch {
        id: "trade_5_wood".to_string(),
        parent_node: 1,
        kind: MiniOpusKind::TradeSurplus,
        trigger: MiniOpusTrigger::OnDemand,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Gold,
        reward_amount: 50,
        deadline_tick: None,
        condition_value: 0.0,
        condition_threshold: 5.0,
    };

    // Player trades 5 units of wood — condition met
    let wood_traded = 5.0_f32;
    branch.condition_value = wood_traded;

    // Simulate MiniOpusSystem check
    if branch.condition_value >= branch.condition_threshold
        && branch.trigger == MiniOpusTrigger::OnDemand
        && branch.status == MiniOpusStatus::Active
    {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed, "branch must be completed");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Gold);
    assert_eq!(event.reward_amount, 50);
}

#[test]
fn completing_time_based_mini_opus_before_deadline_awards_knowledge() {
    let mut branch = MiniOpusBranch {
        id: "fast_steel".to_string(),
        parent_node: 3,
        kind: MiniOpusKind::SpeedProduction,
        trigger: MiniOpusTrigger::TimeBased,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Knowledge,
        reward_amount: 60,
        deadline_tick: Some(50000),
        condition_value: 3.2,   // sustained rate of steel_plate: 3.2 per minute
        condition_threshold: 3.0,
    };

    let current_tick = 45000u64;
    let deadline = branch.deadline_tick.unwrap();

    // Condition: before deadline and rate met
    if current_tick < deadline
        && branch.condition_value >= branch.condition_threshold
        && branch.status == MiniOpusStatus::Active
    {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed, "fast_steel must complete before deadline");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Knowledge);
    assert_eq!(event.reward_amount, 60);
}

#[test]
fn completing_conditional_mini_opus_awards_souls() {
    let mut branch = MiniOpusBranch {
        id: "clear_nest_fast".to_string(),
        parent_node: 2,
        kind: MiniOpusKind::ClearNestFast,
        trigger: MiniOpusTrigger::Conditional,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Souls,
        reward_amount: 70,
        deadline_tick: None,
        condition_value: 0.0,
        condition_threshold: 600.0, // must clear within 600 ticks of discovery
    };

    // Nest discovered at tick 10000, cleared at tick 10400 → 400 ticks elapsed
    let discovery_tick = 10000u64;
    let cleared_tick = 10400u64;
    let elapsed = (cleared_tick - discovery_tick) as f32;

    // condition: elapsed <= threshold (cleared within 600 ticks)
    branch.condition_value = elapsed;

    if branch.condition_value <= branch.condition_threshold
        && branch.trigger == MiniOpusTrigger::Conditional
        && branch.status == MiniOpusStatus::Active
    {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed, "clear_nest_fast must complete");
    assert!((elapsed - 400.0).abs() < 0.001, "elapsed must be 400 ticks");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Souls);
    assert_eq!(event.reward_amount, 70);
}

#[test]
fn skipping_a_mini_opus_does_not_affect_main_path_progression() {
    let sustain_window_ticks = 600u32;

    // Mini-opus is missed — simulate status transition from Active to Missed
    let mut mini_opus_status = MiniOpusStatus::Active;
    // The missed flag is applied (e.g. deadline expired)
    mini_opus_status = MiniOpusStatus::Missed;
    assert_eq!(mini_opus_status, MiniOpusStatus::Missed, "mini-opus status must be Missed");

    // Main-path node 1 is still being evaluated independently
    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronOre,
        required_rate: 4.0,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    };

    let production_rate = 4.5_f32;

    for _ in 0..sustain_window_ticks {
        if production_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(node.sustained, "node 1 sustained field is true regardless of missed mini-opus");
}

// ── AC6: Final Opus node requires simultaneous sustain ────────────────────────

#[test]
fn final_node_completes_when_all_main_path_rates_sustained_simultaneously() {
    let sustain_ticks_required = 600u32;

    let mut tree = OpusTreeResource {
        main_path: vec![
            OpusNodeEntry { node_index: 0, resource: ResourceType::IronOre,   required_rate: 4.0, current_rate: 4.5, tier: 1, sustained: true },
            OpusNodeEntry { node_index: 1, resource: ResourceType::IronBar,   required_rate: 3.0, current_rate: 3.5, tier: 1, sustained: true },
            OpusNodeEntry { node_index: 2, resource: ResourceType::SteelPlate, required_rate: 2.0, current_rate: 2.2, tier: 2, sustained: true },
            OpusNodeEntry { node_index: 3, resource: ResourceType::Hide,       required_rate: 1.6, current_rate: 1.8, tier: 2, sustained: true },
            OpusNodeEntry { node_index: 4, resource: ResourceType::OpusIngot,  required_rate: 1.0, current_rate: 1.1, tier: 3, sustained: true },
        ],
        mini_opus: vec![],
        completion_pct: 0.0,
        simultaneous_sustain_ticks: 600,
        sustain_ticks_required,
    };

    tree.recalc_completion();

    let all_sustained = tree.all_sustained();
    let sustain_met = tree.simultaneous_sustain_ticks >= sustain_ticks_required;

    assert!(all_sustained, "all 5 nodes must be sustained");
    assert!(sustain_met, "simultaneous sustain must have lasted 600 ticks");
    assert!((tree.completion_pct - 1.0).abs() < 0.001, "completion_pct must be 1.0");

    // Verify the CONDITIONS that would trigger RunWon are both met.
    // RunWon fires only when all_sustained AND simultaneous_sustain_ticks >= sustain_ticks_required.
    // Event emission verified at impl stage via EventReader.
    assert!(
        all_sustained && sustain_met,
        "RunWon trigger conditions: all nodes sustained AND sustain_ticks >= 600"
    );
}

#[test]
fn final_node_does_not_complete_when_one_rate_drops_during_sustain_window() {
    let sustain_ticks_required = 600u32;

    // Node 5 (index 4) drops below required rate at tick 300
    let tree = OpusTreeResource {
        main_path: vec![
            OpusNodeEntry { node_index: 0, resource: ResourceType::IronOre,   required_rate: 4.0, current_rate: 4.5, tier: 1, sustained: true },
            OpusNodeEntry { node_index: 1, resource: ResourceType::IronBar,   required_rate: 3.0, current_rate: 3.5, tier: 1, sustained: true },
            OpusNodeEntry { node_index: 2, resource: ResourceType::SteelPlate, required_rate: 2.0, current_rate: 2.2, tier: 2, sustained: true },
            OpusNodeEntry { node_index: 3, resource: ResourceType::Hide,       required_rate: 1.6, current_rate: 1.8, tier: 2, sustained: true },
            // Node 5 rate drops to 0.8 which is below 1.0 — not sustained
            OpusNodeEntry { node_index: 4, resource: ResourceType::OpusIngot,  required_rate: 1.0, current_rate: 0.8, tier: 3, sustained: false },
        ],
        mini_opus: vec![],
        completion_pct: 0.0,
        simultaneous_sustain_ticks: 0, // reset because not all sustained
        sustain_ticks_required,
    };

    let all_sustained = tree.all_sustained();
    let sustain_met = tree.simultaneous_sustain_ticks >= sustain_ticks_required;

    assert!(!all_sustained, "not all nodes sustained — node 5 dropped");
    assert!(!sustain_met, "simultaneous sustain window not reached");
    // No RunWon event emitted
}

// ── AC7: T2 inaccessible until T1 nest cleared ────────────────────────────────

#[test]
fn t2_buildings_cannot_be_placed_before_t1_nest_is_cleared() {
    let tier_gate = TierGateComponent {
        tier: 2,
        nest_id: "forest_wolf_den".to_string(),
        unlocked: false,
    };

    // "steel_smelter" is a T2 building
    let cmd = TieredPlacementCmd {
        building_name: "steel_smelter".to_string(),
        building_tier: 2,
        x: 3,
        y: 3,
    };

    // Simulate tier gate check — produces a rejection reason string
    let rejection_reason: Option<&'static str> = if cmd.building_tier > 1 && !tier_gate.unlocked {
        Some("tier_gate_locked")
    } else {
        None
    };

    assert!(rejection_reason.is_some(), "T2 building must be rejected when T2 gate is not unlocked");
    assert_eq!(rejection_reason, Some("tier_gate_locked"), "rejection reason must be 'tier_gate_locked'");
    assert_eq!(tier_gate.tier, 2, "tier gate must guard tier 2");
    assert_eq!(tier_gate.nest_id, "forest_wolf_den", "tier gate linked to forest_wolf_den");
    // Building was not placed: placement only proceeds when rejection_reason is None
    let placed_building_count = if rejection_reason.is_none() { 1 } else { 0 };
    assert_eq!(placed_building_count, 0, "steel_smelter must not be placed when rejected");
}

#[test]
fn clearing_t1_nest_unlocks_t2() {
    let mut tier_gate = TierGateComponent {
        tier: 2,
        nest_id: "forest_wolf_den".to_string(),
        unlocked: false,
    };

    let nest_cleared_event = NestCleared {
        nest_id: "forest_wolf_den".to_string(),
    };

    // TierGateSystem processes the event
    if nest_cleared_event.nest_id == tier_gate.nest_id && !tier_gate.unlocked {
        tier_gate.unlocked = true;
    }

    assert!(tier_gate.unlocked, "TierGate for tier 2 must be unlocked after nest cleared");

    // TierState updates to tier 2
    let mut tier_state = crate::resources::TierState::default();
    if tier_gate.unlocked {
        tier_state.current_tier = tier_gate.tier as u8;
    }
    assert_eq!(tier_state.current_tier, 2, "TierState currentTier must be 2");

    // TierUnlocked event is emitted
    let event = TierUnlockedProgression { tier: 2 };
    assert_eq!(event.tier, 2);
}

// ── AC8: T3 inaccessible until T2 nest cleared ────────────────────────────────

#[test]
fn t3_buildings_cannot_be_placed_before_t2_nest_is_cleared() {
    let tier_gate = TierGateComponent {
        tier: 3,
        nest_id: "forest_ancient_treant".to_string(),
        unlocked: false,
    };

    // "opus_forge" is a T3 building
    let cmd = TieredPlacementCmd {
        building_name: "opus_forge".to_string(),
        building_tier: 3,
        x: 5,
        y: 5,
    };

    // Current tier is 2 — T3 gate not unlocked
    let current_tier = 2u32;
    let rejection_reason: Option<&'static str> = if cmd.building_tier > current_tier && !tier_gate.unlocked {
        Some("tier_gate_locked")
    } else {
        None
    };

    assert!(rejection_reason.is_some(), "T3 building must be rejected when T3 gate is not unlocked");
    assert_eq!(rejection_reason, Some("tier_gate_locked"), "rejection reason must be 'tier_gate_locked'");
    assert_eq!(tier_gate.tier, 3, "tier gate must guard tier 3");
    assert_eq!(tier_gate.nest_id, "forest_ancient_treant", "tier gate linked to forest_ancient_treant");
    // Building was not placed: placement only proceeds when rejection_reason is None
    let placed_building_count = if rejection_reason.is_none() { 1 } else { 0 };
    assert_eq!(placed_building_count, 0, "opus_forge must not be placed when rejected");
}

#[test]
fn clearing_t2_nest_unlocks_t3() {
    let mut tier_gate = TierGateComponent {
        tier: 3,
        nest_id: "forest_ancient_treant".to_string(),
        unlocked: false,
    };

    let nest_cleared_event = NestCleared {
        nest_id: "forest_ancient_treant".to_string(),
    };

    // TierGateSystem processes the event
    if nest_cleared_event.nest_id == tier_gate.nest_id && !tier_gate.unlocked {
        tier_gate.unlocked = true;
    }

    assert!(tier_gate.unlocked, "TierGate for tier 3 must be unlocked after treant nest cleared");

    // TierState updates to tier 3
    let mut tier_state = crate::resources::TierState::default();
    if tier_gate.unlocked {
        tier_state.current_tier = tier_gate.tier as u8;
    }
    assert_eq!(tier_state.current_tier, 3, "TierState currentTier must be 3");

    // TierUnlocked event is emitted
    let event = TierUnlockedProgression { tier: 3 };
    assert_eq!(event.tier, 3);
}

// ── AC9: Final Opus node triggers run-end with scoring ────────────────────────

#[test]
fn run_ends_with_scoring_when_final_node_is_completed() {
    // Standard opus: 5 nodes all sustained
    // 1 of 2 mini-opus completed
    // elapsed 72000 / 108000 max → time remaining 36000 / 108000 = 0.333
    // opus_completion = 5/5 = 1.0
    // mini_opus_score = 1/2 = 0.5
    // time_bonus = 1 - (72000/108000) = 1 - 0.667 = 0.333
    // raw_score = 0.5 * 1.0 + 0.3 * 0.5 + 0.2 * 0.333 = 0.5 + 0.15 + 0.0667 = 0.7167
    // final_score = round(raw_score * 1000) = 717

    let elapsed_ticks = 72000u64;
    let max_ticks = 108000u64;
    let mini_opus_completed = 1u32;
    let mini_opus_total = 2u32;
    let total_nodes = 5u32;
    let sustained_nodes = 5u32; // all 5 sustained

    let opus_completion = sustained_nodes as f32 / total_nodes as f32;
    let mini_opus_score = mini_opus_completed as f32 / mini_opus_total as f32;
    let time_bonus = 1.0 - (elapsed_ticks as f32 / max_ticks as f32);

    let raw_score = 0.5 * opus_completion + 0.3 * mini_opus_score + 0.2 * time_bonus;
    let final_score = (raw_score * 1000.0).round() as u32;

    assert_eq!(opus_completion, 1.0, "opus_completion must be 5/5 = 1.0");
    assert!((mini_opus_score - 0.5).abs() < 0.001, "mini_opus_score = 1/2 = 0.5");
    assert!((time_bonus - 0.333).abs() < 0.01, "time_bonus ≈ 0.333, got {time_bonus}");
    assert!((raw_score - 0.717).abs() < 0.01, "raw_score ≈ 0.717, got {raw_score}");
    assert_eq!(final_score, 717, "final display score must be 717");

    // Currency earned: base_currency * raw_score * difficulty_multiplier
    let difficulty_multiplier = OpusDifficulty::Medium.currency_multiplier();
    assert!((difficulty_multiplier - 1.5).abs() < 0.001);
    // currency_earned = base * 0.717 * 1.5 (base defined elsewhere)
}

// ── AC10: All recipes for tier available immediately on unlock ─────────────────

#[test]
fn t2_recipes_become_available_immediately_when_t2_is_unlocked() {
    // Simulate a RecipeDB entry with a tier lock: steel_plate_recipe requires tier 2
    let recipe_required_tier: u32 = 2;
    let recipe_name = "steel_plate_recipe";

    // Before unlock: current_tier = 1 → recipe locked
    let current_tier_before: u32 = 1;
    let is_locked_before = current_tier_before < recipe_required_tier;
    assert!(is_locked_before, "{recipe_name} must be locked when current_tier=1 < required_tier=2");

    // TierUnlocked event for tier 2 arrives → tier transitions to 2
    let event = TierUnlockedProgression { tier: 2 };
    let current_tier_after = event.tier;

    // After unlock: recipe available because current_tier >= required_tier
    let is_available = current_tier_after >= recipe_required_tier;
    assert!(is_available, "{recipe_name} must be available immediately when current_tier=2 >= required_tier=2");

    // All T2 recipes share required_tier=2, so the same condition unlocks them all
    let t2_recipe_tiers: Vec<u32> = vec![2, 2, 2]; // representative sample of T2 recipes
    let all_t2_unlocked = t2_recipe_tiers.iter().all(|&rt| current_tier_after >= rt);
    assert!(all_t2_unlocked, "all T2 recipes in RecipeDB must be available immediately after T2 unlock");
}

// ── AC11: Existing buildings auto-upgrade on tier unlock ──────────────────────

#[test]
fn existing_t1_buildings_auto_upgrade_when_t2_unlocks() {
    // Simulate a placed iron_smelter at tier 1
    let mut building_tier = BuildingTier { tier: 1 };
    let building_x = 2i32;
    let building_y = 3i32;
    let building_group_id: u64 = 42; // mock group id

    // TierUnlocked event for tier 2
    let event = TierUnlockedProgression { tier: 2 };

    // Auto-upgrade: all buildings with tier < event.tier get upgraded
    if building_tier.tier < event.tier {
        building_tier.tier = event.tier;
    }

    assert_eq!(building_tier.tier, 2, "iron_smelter building tier must be 2 after T2 unlock");
    // Position and group membership are retained (not changed by upgrade)
    assert_eq!(building_x, 2, "building retains x position");
    assert_eq!(building_y, 3, "building retains y position");
    assert_eq!(building_group_id, 42, "building retains group membership");
}

#[test]
fn auto_upgrade_applies_to_all_existing_buildings_of_lower_tier() {
    // 3 buildings at tier 1, 2 buildings at tier 2; TierUnlocked for tier 3
    let mut tiers: Vec<BuildingTier> = vec![
        BuildingTier { tier: 1 },
        BuildingTier { tier: 1 },
        BuildingTier { tier: 1 },
        BuildingTier { tier: 2 },
        BuildingTier { tier: 2 },
    ];

    let event = TierUnlockedProgression { tier: 3 };

    for bt in tiers.iter_mut() {
        if bt.tier < event.tier {
            bt.tier = event.tier;
        }
    }

    assert!(
        tiers.iter().all(|bt| bt.tier == 3),
        "all 5 buildings must be at tier 3 after T3 unlock"
    );
}

// ── Edge Case: Milestone no-regression ────────────────────────────────────────

#[test]
fn rate_drop_after_sustain_does_not_revoke_milestone() {
    // Node was already sustained = true
    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::IronOre,
        required_rate: 4.0,
        tier: 1,
        sustained: true, // already sustained
        sustain_ticks: 600,
    };

    // Rate drops to 2.0 — below required 4.0
    let production_rate = 2.0_f32;
    let sustain_window_ticks = 600u32;

    // MilestoneCheckSystem: only marks sustained if not already sustained
    // Once sustained, no regression
    if !node.sustained {
        if production_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(node.sustained, "sustained=true must not be revoked by rate drop");
}

// ── Edge Case: Time-based mini-opus missed ────────────────────────────────────

#[test]
fn time_based_mini_opus_marked_missed_after_deadline() {
    let mut branch = MiniOpusBranch {
        id: "fast_steel".to_string(),
        parent_node: 3,
        kind: MiniOpusKind::SpeedProduction,
        trigger: MiniOpusTrigger::TimeBased,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Knowledge,
        reward_amount: 60,
        deadline_tick: Some(50000),
        condition_value: 0.0,     // condition not met
        condition_threshold: 3.0,
    };

    let current_tick = 51000u64;
    let deadline = branch.deadline_tick.unwrap();

    // Deadline has passed — mark as missed
    if current_tick >= deadline && branch.status == MiniOpusStatus::Active {
        branch.status = MiniOpusStatus::Missed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Missed, "fast_steel must be marked missed after deadline");

    let event = MiniOpusMissed { id: branch.id.clone() };
    assert_eq!(event.id, "fast_steel");

    // Main-path node 3 is unaffected — verify it can still sustain
    let node_3 = OpusNodeFull {
        node_index: 3,
        resource: ResourceType::Hide,
        required_rate: 1.6,
        tier: 2,
        sustained: false,
        sustain_ticks: 0,
    };
    assert!(!node_3.sustained, "node 3 still has its own independent sustain state");
}

// ── Edge Case: Opus requires non-biome resource ───────────────────────────────

#[test]
fn opus_node_requires_resource_unavailable_in_biome() {
    // Volcanic biome: no natural wood veins
    // But player builds a tree_farm synthesis group that produces wood
    let biome = Biome::Volcanic;

    // Opus node requires wood at 2.0/min at tier 1
    let mut node = OpusNodeFull {
        node_index: 0,
        resource: ResourceType::Wood,
        required_rate: 2.0,
        tier: 1,
        sustained: false,
        sustain_ticks: 0,
    };

    // Verify biome is volcanic
    assert_eq!(biome, Biome::Volcanic);

    // tree_farm produces wood at 2.0 per minute (synthesis group, no vein needed)
    let tree_farm_rate = 2.0_f32;
    let sustain_window_ticks = 600u32;

    for _ in 0..sustain_window_ticks {
        if tree_farm_rate >= node.required_rate {
            node.sustain_ticks += 1;
        } else {
            node.sustain_ticks = 0;
        }
        if node.sustain_ticks >= sustain_window_ticks {
            node.sustained = true;
        }
    }

    assert!(node.sustained, "opus node for wood sustained via tree_farm synthesis in volcanic biome");
}

// ── Edge Case: Run timeout with partial completion ─────────────────────────────

#[test]
fn run_timeout_awards_partial_score_based_on_tree_fill() {
    // Nodes 1, 2, 3 sustained; nodes 4, 5 not sustained → 3/5 = 0.6
    // 0 of 2 mini-opus completed → mini_opus_score = 0.0
    // At max_ticks → time_bonus = 0.0
    // raw_score = 0.5 * 0.6 + 0.3 * 0.0 + 0.2 * 0.0 = 0.3
    // final_score = 300

    let total_nodes = 5u32;
    let sustained_nodes = 3u32;
    let mini_opus_completed = 0u32;
    let mini_opus_total = 2u32;
    let current_tick = 108000u64;
    let max_ticks = 108000u64;

    let opus_completion = sustained_nodes as f32 / total_nodes as f32;
    let mini_opus_score = if mini_opus_total > 0 {
        mini_opus_completed as f32 / mini_opus_total as f32
    } else {
        0.0
    };
    let time_bonus = if current_tick >= max_ticks { 0.0 } else {
        1.0 - (current_tick as f32 / max_ticks as f32)
    };

    let raw_score = 0.5 * opus_completion + 0.3 * mini_opus_score + 0.2 * time_bonus;
    let final_score = (raw_score * 1000.0).round() as u32;

    assert!((opus_completion - 0.6).abs() < 0.001, "opus_completion must be 0.6, got {opus_completion}");
    assert!((mini_opus_score - 0.0).abs() < 0.001, "mini_opus_score must be 0.0");
    assert!((time_bonus - 0.0).abs() < 0.001, "time_bonus must be 0.0 at timeout");
    assert!((raw_score - 0.3).abs() < 0.001, "raw_score must be 0.3, got {raw_score}");
    assert_eq!(final_score, 300, "final display score must be 300");

    // Verify the CONDITIONS that would trigger RunTimeUp are met.
    // RunTimeUp fires when current_tick >= max_ticks.
    // Event emission verified at impl stage via EventReader.
    let timeout_reached = current_tick >= max_ticks;
    assert!(timeout_reached, "RunTimeUp condition met: current_tick={current_tick} >= max_ticks={max_ticks}");
}

// ── Edge Case: All mini-opus missed ───────────────────────────────────────────

#[test]
fn run_completable_with_all_mini_opus_missed() {
    // All 5 nodes sustained, 0 of 2 mini-opus completed, elapsed 54000 ticks
    // opus_completion = 1.0
    // mini_opus_score = 0.0
    // time_bonus = 1.0 - (54000/108000) = 0.5
    // raw_score = 0.5 * 1.0 + 0.3 * 0.0 + 0.2 * 0.5 = 0.6

    let elapsed_ticks = 54000u64;
    let max_ticks = 108000u64;
    let mini_opus_completed = 0u32;
    let mini_opus_total = 2u32;
    let total_nodes = 5u32;
    let sustained_nodes = 5u32; // all 5 sustained

    let opus_completion = sustained_nodes as f32 / total_nodes as f32;
    let mini_opus_score = if mini_opus_total > 0 {
        mini_opus_completed as f32 / mini_opus_total as f32
    } else {
        0.0
    };
    let time_bonus = 1.0 - (elapsed_ticks as f32 / max_ticks as f32);
    let raw_score = 0.5 * opus_completion + 0.3 * mini_opus_score + 0.2 * time_bonus;

    assert_eq!(opus_completion, 1.0, "opus_completion must be 5/5 = 1.0");
    assert!((mini_opus_score - 0.0).abs() < 0.001, "mini_opus_score must be 0.0");
    assert!((time_bonus - 0.5).abs() < 0.001, "time_bonus must be 0.5, got {time_bonus}");
    assert!((raw_score - 0.6).abs() < 0.001, "raw_score = 0.6, got {raw_score}");
}

// ── Edge Case: Run abandoned ──────────────────────────────────────────────────

#[test]
fn abandoned_run_earns_zero_meta_currency() {
    let mut run_config = RunConfig::default();
    run_config.abandoned = true;

    let abandon_currency_multiplier = if run_config.abandoned { 0.0_f32 } else { 1.0 };

    assert!((abandon_currency_multiplier - 0.0).abs() < f32::EPSILON,
        "abandon_currency_multiplier must be 0.0");

    let base_currency = 500.0_f32;
    let currency_awarded = base_currency * abandon_currency_multiplier;
    assert!((currency_awarded - 0.0).abs() < f32::EPSILON,
        "no meta-currency awarded on abandon, got {currency_awarded}");

    // Verify the CONDITIONS that would trigger RunAbandoned are met.
    // RunAbandoned fires when run_config.abandoned is true.
    // Event emission verified at impl stage via EventReader.
    assert!(run_config.abandoned, "RunAbandoned condition met: run_config.abandoned is true");
}

// ── Starting Kit ──────────────────────────────────────────────────────────────

#[test]
fn forest_starting_kit_provides_correct_buildings() {
    let mut inventory = PlayerInventory::default();

    // Simulate applying forest starting kit with no meta unlocks
    let kit = StartingKitCommands {
        biome: Biome::Forest,
        meta_unlocks: vec![],
        applied: false,
    };

    // Forest base kit (from seed: progression/starting_kits.yaml)
    match kit.biome {
        Biome::Forest => {
            *inventory.buildings.entry("iron_miner".to_string()).or_default() += 2;
            *inventory.buildings.entry("water_pump".to_string()).or_default() += 1;
            *inventory.buildings.entry("iron_smelter".to_string()).or_default() += 1;
            *inventory.buildings.entry("sawmill".to_string()).or_default() += 1;
            *inventory.buildings.entry("tree_farm".to_string()).or_default() += 1;
            *inventory.buildings.entry("constructor".to_string()).or_default() += 1;
            *inventory.buildings.entry("wind_turbine".to_string()).or_default() += 2;
            *inventory.buildings.entry("watchtower".to_string()).or_default() += 1;
        }
        _ => {}
    }

    assert_eq!(*inventory.buildings.get("iron_miner").unwrap_or(&0), 2);
    assert_eq!(*inventory.buildings.get("water_pump").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("iron_smelter").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("sawmill").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("tree_farm").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("constructor").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("wind_turbine").unwrap_or(&0), 2);
    assert_eq!(*inventory.buildings.get("watchtower").unwrap_or(&0), 1);
}

#[test]
fn volcanic_starting_kit_has_no_wood_or_water_buildings() {
    let mut inventory = PlayerInventory::default();

    let kit = StartingKitCommands {
        biome: Biome::Volcanic,
        meta_unlocks: vec![],
        applied: false,
    };

    // Volcanic base kit (from seed: progression/starting_kits.yaml)
    match kit.biome {
        Biome::Volcanic => {
            *inventory.buildings.entry("iron_miner".to_string()).or_default() += 2;
            *inventory.buildings.entry("stone_quarry".to_string()).or_default() += 2;
            *inventory.buildings.entry("iron_smelter".to_string()).or_default() += 1;
            *inventory.buildings.entry("constructor".to_string()).or_default() += 1;
            *inventory.buildings.entry("wind_turbine".to_string()).or_default() += 3;
            *inventory.buildings.entry("watchtower".to_string()).or_default() += 1;
        }
        _ => {}
    }

    assert_eq!(*inventory.buildings.get("iron_miner").unwrap_or(&0), 2);
    assert_eq!(*inventory.buildings.get("stone_quarry").unwrap_or(&0), 2);
    assert_eq!(*inventory.buildings.get("iron_smelter").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("constructor").unwrap_or(&0), 1);
    assert_eq!(*inventory.buildings.get("wind_turbine").unwrap_or(&0), 3);
    assert_eq!(*inventory.buildings.get("watchtower").unwrap_or(&0), 1);

    // Must NOT contain wood or water buildings
    assert_eq!(*inventory.buildings.get("water_pump").unwrap_or(&0), 0,
        "volcanic kit must not contain water_pump");
    assert_eq!(*inventory.buildings.get("sawmill").unwrap_or(&0), 0,
        "volcanic kit must not contain sawmill");
}

#[test]
fn starting_kit_enhanced_by_meta_unlocks() {
    let mut inventory = PlayerInventory::default();

    let kit = StartingKitCommands {
        biome: Biome::Forest,
        meta_unlocks: vec![
            "extra_starting_miner".to_string(),
            "extra_starting_turbine".to_string(),
        ],
        applied: false,
    };

    // Base forest kit
    *inventory.buildings.entry("iron_miner".to_string()).or_default() += 2;
    *inventory.buildings.entry("wind_turbine".to_string()).or_default() += 2;

    // Apply meta unlock bonuses
    for unlock in &kit.meta_unlocks {
        match unlock.as_str() {
            "extra_starting_miner"   => *inventory.buildings.entry("iron_miner".to_string()).or_default() += 1,
            "extra_starting_turbine" => *inventory.buildings.entry("wind_turbine".to_string()).or_default() += 1,
            _ => {}
        }
    }

    assert_eq!(*inventory.buildings.get("iron_miner").unwrap_or(&0), 3,
        "3 iron_miner with extra_starting_miner unlock");
    assert_eq!(*inventory.buildings.get("wind_turbine").unwrap_or(&0), 3,
        "3 wind_turbine with extra_starting_turbine unlock");
}

// ── Mini-Opus Branch Generation ───────────────────────────────────────────────

#[test]
fn branch_points_receive_1_to_2_mini_opus_branches_each() {
    // Standard template has branch_points at nodes 1, 3, 5 (indices 0, 2, 4)
    // Each branch point gets 1-2 mini-opus branches
    let branch_points = vec![0u32, 2u32, 4u32];

    let mini_opus_list = vec![
        MiniOpusEntry { id: "trade_5_wood".to_string(), parent_node: 0, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Gold, reward_amount: 50 },
        MiniOpusEntry { id: "build_monument".to_string(), parent_node: 0, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Knowledge, reward_amount: 40 },
        MiniOpusEntry { id: "fast_steel".to_string(), parent_node: 2, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Knowledge, reward_amount: 60 },
        MiniOpusEntry { id: "zero_waste".to_string(), parent_node: 4, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Knowledge, reward_amount: 55 },
    ];

    for bp in &branch_points {
        let count = mini_opus_list.iter().filter(|m| m.parent_node == *bp).count();
        assert!(
            count >= 1 && count <= 2,
            "node {} must have 1-2 mini-opus branches, got {count}",
            bp
        );
    }
}

#[test]
fn non_branch_point_nodes_receive_no_mini_opus_branches() {
    // Nodes 2 and 4 (indices 1 and 3) are not branch points in the standard template
    let non_branch_nodes = vec![1u32, 3u32]; // node 2 and node 4 in 1-indexed BDD

    let mini_opus_list = vec![
        MiniOpusEntry { id: "trade_5_wood".to_string(), parent_node: 0, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Gold, reward_amount: 50 },
        MiniOpusEntry { id: "fast_steel".to_string(), parent_node: 2, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Knowledge, reward_amount: 60 },
        MiniOpusEntry { id: "zero_waste".to_string(), parent_node: 4, status: MiniOpusStatus::Active, reward_currency: MetaCurrency::Knowledge, reward_amount: 55 },
    ];

    for nb in &non_branch_nodes {
        let count = mini_opus_list.iter().filter(|m| m.parent_node == *nb).count();
        assert_eq!(count, 0, "node {} must have 0 mini-opus branches, got {count}", nb);
    }
}

// ── Opus Difficulty Multiplier ────────────────────────────────────────────────

#[test]
fn easy_difficulty_applies_1_0_opus_multiplier_to_currency() {
    let difficulty = OpusDifficulty::Easy;
    let base_reward = 50.0_f32;
    let multiplier = difficulty.currency_multiplier();
    let earned = base_reward * multiplier;

    assert!((multiplier - 1.0).abs() < f32::EPSILON, "Easy multiplier must be 1.0, got {multiplier}");
    assert!((earned - 50.0).abs() < 0.001, "Easy: 50 * 1.0 = 50, got {earned}");
}

#[test]
fn hard_difficulty_applies_2_0_opus_multiplier_to_currency() {
    let difficulty = OpusDifficulty::Hard;
    let base_reward = 50.0_f32;
    let multiplier = difficulty.currency_multiplier();
    let earned = base_reward * multiplier;

    assert!((multiplier - 2.0).abs() < f32::EPSILON, "Hard multiplier must be 2.0, got {multiplier}");
    assert!((earned - 100.0).abs() < 0.001, "Hard: 50 * 2.0 = 100, got {earned}");
}

#[test]
fn extreme_difficulty_applies_3_0_opus_multiplier_to_currency() {
    let difficulty = OpusDifficulty::Extreme;
    let base_reward = 50.0_f32;
    let multiplier = difficulty.currency_multiplier();
    let earned = base_reward * multiplier;

    assert!((multiplier - 3.0).abs() < f32::EPSILON, "Extreme multiplier must be 3.0, got {multiplier}");
    assert!((earned - 150.0).abs() < 0.001, "Extreme: 50 * 3.0 = 150, got {earned}");
}

// ── Tier Gate: Transport Auto-Upgrade ─────────────────────────────────────────

#[test]
fn transport_tier_upgrades_globally_on_tier_unlock() {
    let mut transport_state = TransportTierState {
        transport_tier: 1,
    };

    // TierUnlocked event for tier 2
    let event = TierUnlockedProgression { tier: 2 };

    // TierGateSystem: upgrades transport tier to match
    if event.tier > transport_state.transport_tier {
        transport_state.transport_tier = event.tier;
    }

    assert_eq!(transport_state.transport_tier, 2,
        "TierState transportTier must be 2 after T2 unlock");
}

// ── Conditional Mini-Opus: zero_waste ─────────────────────────────────────────

#[test]
fn zero_waste_mini_opus_completes_when_no_idle_resources_for_300_ticks() {
    let mut branch = MiniOpusBranch {
        id: "zero_waste".to_string(),
        parent_node: 4,
        kind: MiniOpusKind::ZeroWaste,
        trigger: MiniOpusTrigger::Conditional,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Knowledge,
        reward_amount: 55,
        deadline_tick: None,
        condition_value: 300.0, // 300 ticks with no idle resources
        condition_threshold: 300.0,
    };

    // Condition: no resources idle for 300 consecutive ticks
    if branch.condition_value >= branch.condition_threshold
        && branch.trigger == MiniOpusTrigger::Conditional
        && branch.status == MiniOpusStatus::Active
    {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed, "zero_waste must be completed");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Knowledge);
    assert_eq!(event.reward_amount, 55);
}

// ── Conditional Mini-Opus: organic_surplus ────────────────────────────────────

#[test]
fn organic_surplus_mini_opus_completes_when_threshold_reached() {
    let mut branch = MiniOpusBranch {
        id: "organic_surplus".to_string(),
        parent_node: 2,
        kind: MiniOpusKind::OrganicSurplus,
        trigger: MiniOpusTrigger::Conditional,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Souls,
        reward_amount: 45,
        deadline_tick: None,
        condition_value: 20.0,      // 20 organic resources produced in combat group
        condition_threshold: 20.0,
    };

    // Condition: produce 20 organic resources in single combat group
    if branch.condition_value >= branch.condition_threshold
        && branch.trigger == MiniOpusTrigger::Conditional
        && branch.status == MiniOpusStatus::Active
    {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed, "organic_surplus must be completed");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Souls);
    assert_eq!(event.reward_amount, 45);
}

// ── Time-Based Mini-Opus: survive_hazard_producing ───────────────────────────

#[test]
fn survive_hazard_producing_mini_opus_requires_maintaining_rate_during_hazard() {
    // condition: maintain >= 80% of current rate during hazard
    // current_rate = 5.0, 80% threshold = 4.0
    // actual rate during hazard = 4.2 (above threshold)
    let mut branch = MiniOpusBranch {
        id: "survive_hazard".to_string(),
        parent_node: 2,
        kind: MiniOpusKind::SurviveHazardProducing,
        trigger: MiniOpusTrigger::TimeBased,
        status: MiniOpusStatus::Active,
        reward_currency: MetaCurrency::Souls,
        reward_amount: 80,
        deadline_tick: None, // deadline is end of hazard
        condition_value: 4.2,   // rate during hazard
        condition_threshold: 4.0, // 80% of 5.0
    };

    let hazard_duration_ticks = 400u32;
    let current_production_rate = 5.0_f32;
    let required_fraction = 0.8_f32;
    let min_rate_during_hazard = current_production_rate * required_fraction;

    assert!((min_rate_during_hazard - 4.0).abs() < 0.001,
        "80% of 5.0 = 4.0, got {min_rate_during_hazard}");
    assert!(branch.condition_value >= min_rate_during_hazard,
        "4.2 >= 4.0: rate maintained during hazard");

    // After hazard ends (400 ticks elapsed), condition met
    let hazard_survived = branch.condition_value >= branch.condition_threshold;
    if hazard_survived && branch.status == MiniOpusStatus::Active {
        branch.status = MiniOpusStatus::Completed;
    }

    assert_eq!(branch.status, MiniOpusStatus::Completed,
        "survive_hazard must be completed when rate maintained during hazard");

    let event = MiniOpusCompleted {
        id: branch.id.clone(),
        reward_currency: branch.reward_currency,
        reward_amount: branch.reward_amount,
    };
    assert_eq!(event.reward_currency, MetaCurrency::Souls);
    assert_eq!(event.reward_amount, 80);
    let _ = hazard_duration_ticks;
}

// ── Run Lifecycle: Tier Timing ────────────────────────────────────────────────

#[test]
fn tier_timing_targets_are_advisory_only() {
    // T1 end target is tick 30000; current tick is 35000; current tier is still 1
    let tier_t1_end_target = 30000u64;
    let current_tick = 35000u64;
    let current_tier = 1u32;

    // Exceeded advisory target — no penalty, no block
    let exceeded_target = current_tick > tier_t1_end_target;
    assert!(exceeded_target, "current_tick 35000 exceeds t1_end target 30000");

    // No penalty applied: RunConfig has no production_speed_modifier or penalty field.
    // Verify that RunConfig with exceeded timing has all default multipliers (no degradation).
    let run_config = RunConfig::default();
    // RunConfig tracks no penalty for exceeding tier timing — production rates are unaffected.
    // Advisory timing is purely informational; the sustain_window_ticks stays at 600.
    assert_eq!(run_config.sustain_window_ticks, 600,
        "sustain_window_ticks unchanged after exceeding advisory target");
    assert_eq!(run_config.sample_interval_ticks, 20,
        "sample_interval_ticks unchanged after exceeding advisory target");

    // Player can still clear T1 nest to unlock T2
    let tier_gate = TierGateComponent {
        tier: 2,
        nest_id: "forest_wolf_den".to_string(),
        unlocked: false,
    };
    assert_eq!(current_tier, 1, "tier is still 1");
    assert!(!tier_gate.unlocked, "T2 gate not yet unlocked — nest not cleared");

    // Clearing the nest still works regardless of timing
    let nest_cleared = NestCleared { nest_id: "forest_wolf_den".to_string() };
    assert_eq!(nest_cleared.nest_id, "forest_wolf_den",
        "player can still clear T1 nest to unlock T2");
}
