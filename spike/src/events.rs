use bevy::prelude::*;

use crate::components::GroupPriority;

/// Fired after a building is successfully placed.
#[derive(Message)]
pub struct BuildingPlaced {
    pub entity: Entity,
    pub x: i32,
    pub y: i32,
}

/// Fired after a building is removed.
#[derive(Message)]
pub struct BuildingRemoved {
    pub entity: Entity,
    pub x: i32,
    pub y: i32,
}

/// Command: set the priority of a group.
#[derive(Message)]
pub struct SetGroupPriority {
    pub group_id: Entity,
    pub priority: GroupPriority,
}

/// Command: pause a group (stops all production inside it).
#[derive(Message)]
pub struct PauseGroup {
    pub group_id: Entity,
}

/// Command: resume a previously paused group.
#[derive(Message)]
pub struct ResumeGroup {
    pub group_id: Entity,
}

// ── Progression feature events ──────────────────────────────────────────────

use crate::components::{MetaCurrency, ResourceType};

/// Fired when a milestone opus node is sustained for the full window.
#[derive(Message, Clone)]
pub struct MilestoneReached {
    pub node_index: u32,
    pub resource: ResourceType,
}

/// Fired when a mini-opus branch is completed.
#[derive(Message, Clone)]
pub struct MiniOpusCompleted {
    pub id: String,
    pub reward_currency: MetaCurrency,
    pub reward_amount: u32,
}

/// Fired when a mini-opus branch is missed (deadline passed / condition failed).
#[derive(Message, Clone)]
pub struct MiniOpusMissed {
    pub id: String,
}

/// Fired when a creature nest is cleared.
#[derive(Message, Clone)]
pub struct NestCleared {
    pub nest_id: String,
}

/// Fired when a new game tier is unlocked.
#[derive(Message, Clone)]
pub struct TierUnlockedProgression {
    pub tier: u32,
}

/// Fired when the final opus node completes (run won).
#[derive(Message, Clone)]
pub struct RunWon;

/// Fired when the run timer expires.
#[derive(Message, Clone)]
pub struct RunTimeUp;

/// Fired when the player abandons the run.
#[derive(Message, Clone)]
pub struct RunAbandoned;

// ── Transport feature events ─────────────────────────────────────────────────

/// Fired when a path or pipe is successfully created.
#[derive(Message, Clone)]
pub struct PathConnected {
    pub path_entity: Entity,
    pub source_group: Entity,
    pub target_group: Entity,
}

/// Fired when a path or pipe segment is destroyed (connectivity broken).
#[derive(Message, Clone)]
pub struct PathDisconnected {
    pub path_entity: Entity,
    pub source_group: Entity,
    pub target_group: Entity,
}

/// Fired when a new transport tier is unlocked globally.
#[derive(Message, Clone)]
pub struct TierUnlocked {
    pub tier: u8,
}

// ═══════════════════════════════════════════════════════════════════════════
// World feature events
// ═══════════════════════════════════════════════════════════════════════════

use crate::components::{HazardKind, EnhancementType};

/// Fired when a building is destroyed by a hazard event.
#[derive(Message, Clone)]
pub struct BuildingDestroyed {
    pub entity: Entity,
    pub x: i32,
    pub y: i32,
}

/// Fired when a sacrifice building survives the hazard (RNG roll passes).
#[derive(Message, Clone)]
pub struct SacrificeHit {
    pub sacrifice_entity: Entity,
}

/// Fired when a sacrifice building is destroyed by the hazard (RNG roll fails).
#[derive(Message, Clone)]
pub struct SacrificeMiss {
    pub sacrifice_entity: Entity,
}

/// Fired when a building placement is rejected due to terrain/visibility.
#[derive(Message, Clone)]
pub struct PlacementRejected {
    pub x: i32,
    pub y: i32,
    pub reason: &'static str,
}

/// Fired when a hazard event triggers (for test inspection).
#[derive(Message, Clone)]
pub struct HazardTriggered {
    pub hazard_kind: HazardKind,
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub enhancement_type: EnhancementType,
    pub enhancement_magnitude: f32,
}
