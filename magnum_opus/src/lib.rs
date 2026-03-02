pub mod components;
pub mod resources;
pub mod events;
pub mod systems;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use crate::events::*;
use crate::resources::*;
use crate::systems::*;
use crate::systems::placement::PlacementCommands;
use crate::systems::terrain::{
    tick_advance_system, hazard_warning_system, hazard_trigger_system,
    element_interaction_system, weather_tick_system, fog_of_war_system,
    world_placement_system,
};
use crate::systems::progression::{
    milestone_check_system, opus_tree_sync_system, run_lifecycle_system,
    tier_gate_system, building_tier_upgrade_system, mini_opus_system,
};
use crate::systems::creatures::{
    creature_behavior_system, invasive_expansion_system, combat_group_system,
    nest_clearing_system, combat_pressure_system, creature_loot_system, minion_task_system,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    Groups,
    Power,
    Production,
    Manifold,
    Transport,
    Progression,
    Creatures,
    World,
}

/// Plugin for world & biomes simulation systems (used in world BDD tests).
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimTick>();
        app.init_resource::<CurrentWeather>();
        app.init_resource::<ActiveBiome>();
        app.init_resource::<BiomeQualityMap>();
        app.init_resource::<CurrentTierWorld>();
        app.init_resource::<WorldPlacementCommands>();
        app.init_resource::<FixedRng>();

        app.add_message::<BuildingDestroyed>();
        app.add_message::<SacrificeHit>();
        app.add_message::<SacrificeMiss>();
        app.add_message::<PlacementRejected>();
        app.add_message::<HazardTriggered>();

        app.add_systems(Update, (
            tick_advance_system,
            hazard_warning_system,
            hazard_trigger_system,
            element_interaction_system,
            weather_tick_system,
            fog_of_war_system,
            world_placement_system,
        ).chain());
    }
}

/// Plugin for creatures & combat simulation systems.
pub struct CreaturesPlugin;

impl Plugin for CreaturesPlugin {
    fn build(&self, app: &mut App) {
        // Ordering: Creatures runs after Transport
        app.configure_sets(
            Update,
            Phase::Transport.before(Phase::Creatures),
        );

        // Creature events
        app.add_message::<NestCleared>();
        app.add_message::<TierUnlockedProgression>();

        // Creature systems
        app.add_systems(
            Update,
            (
                combat_pressure_system.in_set(Phase::Creatures),
                combat_group_system.in_set(Phase::Creatures),
                creature_behavior_system.in_set(Phase::Creatures),
                invasive_expansion_system.in_set(Phase::Creatures),
                creature_loot_system.in_set(Phase::Creatures),
                nest_clearing_system.in_set(Phase::Creatures),
                minion_task_system.in_set(Phase::Creatures),
            ),
        );
    }
}

pub struct SimulationPlugin {
    pub grid_width: i32,
    pub grid_height: i32,
}

impl Default for SimulationPlugin {
    fn default() -> Self {
        Self { grid_width: 10, grid_height: 10 }
    }
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                Phase::Input.before(Phase::Groups),
                Phase::Groups.before(Phase::Power),
                Phase::Power.before(Phase::Production),
                Phase::Production.before(Phase::Manifold),
                Phase::Manifold.before(Phase::Transport),
                Phase::Transport.before(Phase::Progression),
            ),
        );

        app.insert_resource(Grid::new(self.grid_width, self.grid_height));
        app.init_resource::<EnergyPool>();
        app.init_resource::<PlacementCommands>();
        app.init_resource::<Inventory>();
        app.init_resource::<TierState>();
        app.init_resource::<FogMap>();
        app.init_resource::<PathOccupancy>();
        app.init_resource::<TransportCommands>();
        app.init_resource::<LastDrawPathResult>();
        app.init_resource::<TransportTierState>();
        // Progression resources
        app.init_resource::<OpusTreeResource>();
        app.init_resource::<ProductionRates>();
        app.init_resource::<RunConfig>();
        app.init_resource::<RunState>();

        app.add_message::<BuildingPlaced>();
        app.add_message::<BuildingRemoved>();
        app.add_message::<SetGroupPriority>();
        app.add_message::<PauseGroup>();
        app.add_message::<ResumeGroup>();
        app.add_message::<PathConnected>();
        app.add_message::<PathDisconnected>();
        app.add_message::<TierUnlocked>();
        // Progression events
        app.add_message::<MilestoneReached>();
        app.add_message::<MiniOpusCompleted>();
        app.add_message::<MiniOpusMissed>();
        app.add_message::<NestCleared>();
        app.add_message::<TierUnlockedProgression>();
        app.add_message::<RunWon>();
        app.add_message::<RunTimeUp>();
        app.add_message::<RunAbandoned>();

        app.add_systems(
            Update,
            (
                placement_system.in_set(Phase::Input),
                group_formation_system.in_set(Phase::Groups),
                group_priority_system.in_set(Phase::Groups),
                group_pause_system.in_set(Phase::Groups),
                energy_system.in_set(Phase::Power),
                production_system.in_set(Phase::Production),
                manifold_system.in_set(Phase::Manifold),
                // Transport: explicit order — destroy first, then place/upgrade, then move
                transport_destroy_system.in_set(Phase::Transport),
                transport_placement_system
                    .after(transport_destroy_system)
                    .in_set(Phase::Transport),
                transport_tier_upgrade_system
                    .after(transport_destroy_system)
                    .in_set(Phase::Transport),
                transport_movement_system
                    .after(transport_placement_system)
                    .after(transport_tier_upgrade_system)
                    .in_set(Phase::Transport),
                // Progression phase systems
                milestone_check_system.in_set(Phase::Progression),
                opus_tree_sync_system
                    .after(milestone_check_system)
                    .in_set(Phase::Progression),
                run_lifecycle_system
                    .after(opus_tree_sync_system)
                    .in_set(Phase::Progression),
                tier_gate_system.in_set(Phase::Progression),
                building_tier_upgrade_system
                    .after(tier_gate_system)
                    .in_set(Phase::Progression),
                mini_opus_system.in_set(Phase::Progression),
            ),
        );
    }
}
