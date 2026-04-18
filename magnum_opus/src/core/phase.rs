use bevy::prelude::SystemSet;

/// Simulation tick phases within the `Update` schedule.
/// `CorePlugin` chains them in the order declared here.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Drain `CommandBus<T>` queues into sim state.
    Commands,
    /// Terrain, environment, weather.
    World,
    /// Entity placement (buildings, structures).
    Placement,
    /// Group formation, splits, merges.
    Groups,
    /// Energy generation and distribution.
    Power,
    /// Production tick: inputs to outputs.
    Production,
    /// Resource flow within a group.
    Manifold,
    /// Resource flow between groups.
    Transport,
    /// Milestones, tier gates, run lifecycle.
    Progression,
    /// Modules publish metrics for the tick.
    Metrics,
    /// Tick counter increments, cleanup.
    End,
}

pub const PHASE_ORDER: [Phase; 11] = [
    Phase::Commands,
    Phase::World,
    Phase::Placement,
    Phase::Groups,
    Phase::Power,
    Phase::Production,
    Phase::Manifold,
    Phase::Transport,
    Phase::Progression,
    Phase::Metrics,
    Phase::End,
];
