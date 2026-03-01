use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::components::{BuildingType, ResourceType, TerrainType, EnergyPriority};

#[derive(Resource)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    /// Cells currently occupied (one entry per cell, even for multi-cell footprints).
    pub occupied: HashSet<(i32, i32)>,
    /// Terrain type at each cell (default = Grass when absent).
    pub terrain: HashMap<(i32, i32), TerrainType>,
}

impl Grid {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            occupied: HashSet::new(),
            terrain: HashMap::new(),
        }
    }

    /// Returns terrain at (x, y), defaulting to Grass.
    pub fn terrain_at(&self, x: i32, y: i32) -> TerrainType {
        self.terrain.get(&(x, y)).copied().unwrap_or_default()
    }

    /// True if (x, y) is inside the grid bounds.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }
}

#[derive(Resource)]
pub struct EnergyPool {
    pub total_generation: f32,
    pub total_consumption: f32,
    pub ratio: f32,
}

impl Default for EnergyPool {
    fn default() -> Self {
        Self { total_generation: 0.0, total_consumption: 0.0, ratio: 1.0 }
    }
}

/// Player's building inventory: building_type -> count.
#[derive(Resource, Default)]
pub struct Inventory {
    pub buildings: HashMap<BuildingType, u32>,
    /// Non-building resources (future use).
    pub resources: HashMap<ResourceType, u32>,
}

impl Inventory {
    /// Add count of a building.
    pub fn add_building(&mut self, bt: BuildingType, count: u32) {
        *self.buildings.entry(bt).or_default() += count;
    }

    /// Returns how many of this building type are available.
    pub fn count_building(&self, bt: BuildingType) -> u32 {
        self.buildings.get(&bt).copied().unwrap_or(0)
    }

    /// Consume one building from inventory. Returns false if not available.
    pub fn consume_building(&mut self, bt: BuildingType) -> bool {
        let entry = self.buildings.entry(bt).or_default();
        if *entry > 0 {
            *entry -= 1;
            true
        } else {
            false
        }
    }
}

/// Current progression tier (1 / 2 / 3).
#[derive(Resource)]
pub struct TierState {
    pub current_tier: u8,
}

impl Default for TierState {
    fn default() -> Self {
        Self { current_tier: 1 }
    }
}

/// Fog-of-war: set of revealed cell positions.
#[derive(Resource, Default)]
pub struct FogMap {
    pub revealed: HashSet<(i32, i32)>,
}

impl FogMap {
    /// Returns true if position is visible (revealed).
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.revealed.contains(&(x, y))
    }

    /// Reveal a cell.
    pub fn reveal(&mut self, x: i32, y: i32) {
        self.revealed.insert((x, y));
    }

    /// Reveal all cells in a rect (grid-wide reveal for tests).
    pub fn reveal_all(&mut self, width: i32, height: i32) {
        for y in 0..height {
            for x in 0..width {
                self.revealed.insert((x, y));
            }
        }
    }
}

// ── Energy BDD Resources ──────────────────────────────────────────────────────

/// Active biome for the current run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Resource)]
pub enum Biome {
    #[default]
    Forest,
    Desert,
    Volcanic,
    Ocean,
}

/// Current game tier (1, 2, or 3). Controls which buildings can be placed.
#[derive(Resource)]
pub struct CurrentTier {
    pub tier: u32,
}

impl Default for CurrentTier {
    fn default() -> Self { Self { tier: 1 } }
}

/// Commands for setting group energy priority.
#[derive(Resource, Default)]
pub struct SetGroupPriorityCommands {
    pub queue: Vec<SetGroupPriorityCmd>,
}

pub struct SetGroupPriorityCmd {
    pub group_id: Entity,
    pub priority: crate::components::EnergyPriority,
}

/// Commands for removing buildings (by position).
#[derive(Resource, Default)]
pub struct RemoveBuildingCommands {
    pub queue: Vec<(i32, i32)>,
}

// ── Progression BDD Resources ─────────────────────────────────────────────────

use crate::components::{
    ResourceType as RT, OpusDifficulty, MiniOpusStatus, MetaCurrency,
};

/// Snapshot of a mini-opus branch for the OpusTreeResource view.
#[derive(Debug, Clone)]
pub struct MiniOpusEntry {
    pub id: String,
    pub parent_node: u32,
    pub status: MiniOpusStatus,
    pub reward_currency: MetaCurrency,
    pub reward_amount: u32,
}

/// Snapshot of a main-path node for the OpusTreeResource view.
#[derive(Debug, Clone)]
pub struct OpusNodeEntry {
    pub node_index: u32,
    pub resource: RT,
    pub required_rate: f32,
    pub current_rate: f32,
    pub tier: u32,
    pub sustained: bool,
}

/// Resource-level view of the entire opus tree (queried by UI / scoring).
#[derive(Resource, Default)]
pub struct OpusTreeResource {
    pub main_path: Vec<OpusNodeEntry>,
    pub mini_opus: Vec<MiniOpusEntry>,
    /// Fraction 0.0..1.0: sustained nodes / total nodes.
    pub completion_pct: f32,
    /// True when ALL main-path nodes are simultaneously sustained for sustain_ticks_required.
    pub simultaneous_sustain_ticks: u32,
    /// Threshold (from template final_node.sustain_ticks).
    pub sustain_ticks_required: u32,
}

impl OpusTreeResource {
    pub fn recalc_completion(&mut self) {
        let total = self.main_path.len();
        if total == 0 {
            self.completion_pct = 0.0;
        } else {
            let sustained = self.main_path.iter().filter(|n| n.sustained).count();
            self.completion_pct = sustained as f32 / total as f32;
        }
    }

    pub fn all_sustained(&self) -> bool {
        !self.main_path.is_empty() && self.main_path.iter().all(|n| n.sustained)
    }
}

/// Per-resource measured production rate (items / minute).
#[derive(Resource, Default)]
pub struct ProductionRates {
    pub rates: std::collections::HashMap<RT, f32>,
}

impl ProductionRates {
    pub fn get(&self, resource: RT) -> f32 {
        self.rates.get(&resource).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, resource: RT, rate: f32) {
        self.rates.insert(resource, rate);
    }
}

/// Configuration for the current run.
#[derive(Resource)]
pub struct RunConfig {
    pub biome: Biome,
    pub template_id: String,
    pub difficulty: OpusDifficulty,
    pub current_tick: u64,
    pub max_ticks: u64,
    pub tps: u32,
    pub sustain_window_ticks: u32,
    pub sample_interval_ticks: u32,
    pub abandoned: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            biome: Biome::Forest,
            template_id: "standard".to_string(),
            difficulty: OpusDifficulty::Medium,
            current_tick: 0,
            max_ticks: 108000,
            tps: 20,
            sustain_window_ticks: 600,
            sample_interval_ticks: 20,
            abandoned: false,
        }
    }
}

/// Run lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    InProgress,
    Won,
    TimedOut,
    Abandoned,
}

/// Current run status.
#[derive(Resource)]
pub struct RunState {
    pub status: RunStatus,
    pub opus_completion: f32,
    pub mini_opus_score: f32,
    pub time_bonus: f32,
    pub raw_score: f32,
    pub final_score: u32,
    pub currency_earned: f32,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            status: RunStatus::InProgress,
            opus_completion: 0.0,
            mini_opus_score: 0.0,
            time_bonus: 0.0,
            raw_score: 0.0,
            final_score: 0,
            currency_earned: 0.0,
        }
    }
}

/// Placement commands for tier-gated scenarios (with tier check).
#[derive(Resource, Default)]
pub struct TieredPlacementCommands {
    pub queue: Vec<TieredPlacementCmd>,
    pub last_rejection: Option<String>,
}

pub struct TieredPlacementCmd {
    pub building_name: String,
    pub building_tier: u32,
    pub x: i32,
    pub y: i32,
}

/// Starting kit commands.
#[derive(Resource, Default)]
pub struct StartingKitCommands {
    pub biome: Biome,
    pub meta_unlocks: Vec<String>,
    pub applied: bool,
}

/// Transport tier (tracks current transport unlock level).
#[derive(Resource, Default)]
pub struct TransportTierState {
    pub transport_tier: u32,
}

// ── Transport feature resources ────────────────────────────────────────────────

/// Maps tile position → the transport path entity that occupies it.
#[derive(Resource, Default)]
pub struct PathOccupancy {
    pub tiles: HashMap<(i32, i32), Entity>,
}

/// Result codes for draw-path validation (stored for test inspection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPathResult {
    Ok,
    RejectedImpassable,
    RejectedTooLong,
    RejectedOccupied,
    RejectedNoSender,
    RejectedNoReceiver,
}

/// Stores the outcome of the last DrawPath / DrawPipe attempt.
#[derive(Resource, Default)]
pub struct LastDrawPathResult {
    pub result: Option<DrawPathResult>,
}

/// A pending draw-path command.
pub struct DrawPathCmd {
    pub source_group: Entity,
    pub target_group: Entity,
    pub waypoints: Vec<(i32, i32)>,
    pub is_pipe: bool,
}

/// Queue of transport commands to process each tick.
#[derive(Resource, Default)]
pub struct TransportCommands {
    pub draw_path: Vec<DrawPathCmd>,
    pub destroy_path: Vec<Entity>,
    pub destroy_segment: Vec<(i32, i32)>,
}

// ── UX BDD Resources ──────────────────────────────────────────────────────────

use std::collections::VecDeque;

/// Simulation tick counter — incremented each app.update().
#[derive(Resource, Default)]
pub struct SimulationTick {
    pub tick: u64,
}

/// Result of a calculator query.
#[derive(Debug, Clone)]
pub enum CalculatorResult {
    Success {
        buildings_needed: std::collections::HashMap<BuildingType, u32>,
        energy_needed: f32,
        energy_buildings: std::collections::HashMap<BuildingType, u32>,
        notes: Vec<String>,
    },
    Error {
        kind: CalculatorErrorKind,
        message: String,
        required_tier: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculatorErrorKind {
    TierLocked,
    BiomeUnavailable,
    UnknownResource,
}

/// Input to the production calculator.
#[derive(Debug, Clone)]
pub struct CalculatorQuery {
    pub target_resource: ResourceType,
    pub target_rate_per_min: f32,
    pub current_tier: u32,
    pub biome: Biome,
}

/// Resource holding calculator state.
#[derive(Resource, Default)]
pub struct CalculatorState {
    pub last_result: Option<CalculatorResult>,
    pub is_open: bool,
}

/// Bottleneck highlight level for chain visualizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BottleneckLevel {
    None,
    Yellow, // production < 80% of potential
    Red,    // production < 50% of potential
}

/// Per-group info for chain visualizer.
#[derive(Debug, Clone)]
pub struct GroupVisualizerInfo {
    pub group_entity: Entity,
    pub name: Option<String>,
    pub bottleneck: BottleneckLevel,
    pub efficiency: f32, // 0.0..=1.0+
}

/// Path connection in chain visualizer.
#[derive(Debug, Clone)]
pub struct PathVisualizerInfo {
    pub from_group: Entity,
    pub to_group: Entity,
    pub throughput: f32,
    pub resource: ResourceType,
}

/// Chain visualizer overlay state.
#[derive(Resource)]
pub struct ChainVisualizerState {
    pub is_active: bool,
    pub groups: Vec<GroupVisualizerInfo>,
    pub paths: Vec<PathVisualizerInfo>,
    pub empty_message: Option<String>,
    pub threshold_yellow: f32,
    pub threshold_red: f32,
}

impl Default for ChainVisualizerState {
    fn default() -> Self {
        Self {
            is_active: false,
            groups: Vec::new(),
            paths: Vec::new(),
            empty_message: None,
            threshold_yellow: 0.8,
            threshold_red: 0.5,
        }
    }
}

/// Energy gauge display color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeColor {
    Green,
    Yellow,
    Red,
}

impl GaugeColor {
    pub fn from_balance(balance: f32) -> Self {
        if balance > 0.0 {
            GaugeColor::Green
        } else if balance < 0.0 {
            GaugeColor::Red
        } else {
            GaugeColor::Yellow
        }
    }
}

/// Target-vs-current comparison style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateStyle {
    AboveTarget,
    BelowTarget,
}

/// Rate comparison entry for the dashboard.
#[derive(Debug, Clone)]
pub struct RateComparison {
    pub resource: ResourceType,
    pub current_rate: f32,
    pub required_rate: f32,
}

impl RateComparison {
    pub fn style(&self) -> RateStyle {
        if self.current_rate >= self.required_rate {
            RateStyle::AboveTarget
        } else {
            RateStyle::BelowTarget
        }
    }
}

/// Per-group stockpile entry.
#[derive(Debug, Clone)]
pub struct GroupStockpile {
    pub group_name: String,
    pub resources: std::collections::HashMap<ResourceType, f32>,
}

/// Per-group energy allocation entry.
#[derive(Debug, Clone)]
pub struct GroupEnergyAlloc {
    pub group_name: String,
    pub allocated_energy: f32,
    pub priority: EnergyPriority,
}

/// Time-series data point.
#[derive(Debug, Clone)]
pub struct TimeSeriesPoint {
    pub tick: u64,
    pub value: f32,
}

/// Dashboard state computed from ECS each frame.
#[derive(Resource, Default)]
pub struct DashboardState {
    pub is_open: bool,
    pub energy_balance: f32,
    pub energy_color: Option<GaugeColor>,
    pub opus_progress: f32,       // 0.0..1.0
    pub current_tier: u32,
    pub production_rates: std::collections::HashMap<ResourceType, f32>,
    pub rate_comparisons: Vec<RateComparison>,
    pub group_stockpiles: Vec<GroupStockpile>,
    pub inventory: std::collections::HashMap<BuildingType, u32>,
    pub energy_per_group: Vec<GroupEnergyAlloc>,
    pub energy_history_gen: VecDeque<TimeSeriesPoint>,
    pub energy_history_cons: VecDeque<TimeSeriesPoint>,
    pub production_history: std::collections::HashMap<ResourceType, VecDeque<TimeSeriesPoint>>,
    pub messages: Vec<String>,
}

impl DashboardState {
    pub fn gauge_color(balance: f32) -> GaugeColor {
        GaugeColor::from_balance(balance)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// World feature resources
// ═══════════════════════════════════════════════════════════════════════════

use crate::components::{
    TerrainTypeWorld, TileVisibility, WeatherType, BiomeId,
    ResourceQuality, HazardKind, Tier,
};

/// The world tilemap: a flat hash map from (x,y) → tile data.
#[derive(Resource)]
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    pub biome: BiomeId,
    pub seed: u64,
    pub tiles: std::collections::HashMap<(i32, i32), WorldTileData>,
}

#[derive(Clone)]
pub struct WorldTileData {
    pub terrain: TerrainTypeWorld,
    pub remaining: Option<f32>,
    pub visibility: TileVisibility,
}

impl WorldMap {
    pub fn new(width: i32, height: i32, biome: BiomeId, seed: u64) -> Self {
        Self { width, height, biome, seed, tiles: std::collections::HashMap::new() }
    }

    pub fn count_terrain(&self, terrain: TerrainTypeWorld) -> usize {
        self.tiles.values().filter(|t| t.terrain == terrain).count()
    }

    pub fn fraction_terrain(&self, terrain: TerrainTypeWorld) -> f64 {
        let total = self.tiles.len();
        if total == 0 { return 0.0; }
        self.count_terrain(terrain) as f64 / total as f64
    }

    pub fn terrain_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        let mut pairs: Vec<((i32, i32), u8)> = self.tiles.iter()
            .map(|(pos, data)| (*pos, data.terrain as u8))
            .collect();
        pairs.sort_by_key(|(pos, _)| *pos);
        pairs.hash(&mut hasher);
        hasher.finish()
    }
}

/// Global simulation tick counter.
#[derive(Resource, Default)]
pub struct SimTick {
    pub current: u64,
}

/// Current active weather and its per-tick elemental effects.
#[derive(Resource)]
pub struct CurrentWeather {
    pub weather_type: WeatherType,
    pub fire_effect: f32,
    pub water_effect: f32,
    pub cold_effect: f32,
    pub wind_effect: f32,
    /// For fog weather: reduces watchtower radius by this fraction.
    pub fog_penalty: f32,
    pub ticks_remaining: u32,
}

impl Default for CurrentWeather {
    fn default() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            fire_effect: 0.0,
            water_effect: 0.0,
            cold_effect: 0.0,
            wind_effect: 0.0,
            fog_penalty: 0.0,
            ticks_remaining: 600,
        }
    }
}

/// The active biome for the current run.
#[derive(Resource)]
pub struct ActiveBiome {
    pub id: BiomeId,
}

impl Default for ActiveBiome {
    fn default() -> Self { Self { id: BiomeId::Forest } }
}


/// Quality map for the current biome (resource → High/Normal/unavailable).
#[derive(Resource, Default)]
pub struct BiomeQualityMap {
    /// None value = unavailable in this biome.
    pub entries: std::collections::HashMap<&'static str, Option<ResourceQuality>>,
}

impl BiomeQualityMap {
    /// Returns Some(Some(quality)) if mapped, Some(None) if unavailable, None if unknown.
    pub fn quality(&self, resource: &str) -> Option<Option<ResourceQuality>> {
        self.entries.get(resource).copied()
    }
}

/// Current player tier for the world feature.
#[derive(Resource)]
pub struct CurrentTierWorld {
    pub tier: Tier,
}

impl Default for CurrentTierWorld {
    fn default() -> Self { Self { tier: Tier::T1 } }
}


/// Pending world placement commands (with terrain/visibility checks).
#[derive(Resource, Default)]
pub struct WorldPlacementCommands {
    pub queue: Vec<WorldPlacementCmd>,
    /// Set after each tick if a placement was rejected.
    pub last_rejection: Option<&'static str>,
}

pub struct WorldPlacementCmd {
    pub building_type: crate::components::BuildingType,
    pub x: i32,
    pub y: i32,
    /// None = any buildable terrain; Some(t) = must match exactly.
    pub required_terrain: Option<TerrainTypeWorld>,
}

/// Fixed RNG roll for deterministic test scenarios.
#[derive(Resource, Default)]
pub struct FixedRng {
    pub roll: Option<f32>,
}
