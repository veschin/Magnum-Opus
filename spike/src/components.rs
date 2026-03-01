use bevy::prelude::*;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Building Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    // Legacy generic (existing tests)
    Miner,
    Smelter,
    EnergySource,

    // Extraction
    IronMiner,
    CopperMiner,
    StoneQuarry,
    WaterPump,
    ObsidianDrill,
    ManaExtractor,
    LavaSiphon,

    // Synthesis
    IronSmelter,
    CopperSmelter,
    TreeFarm,
    Sawmill,
    SteelForge,
    SteelSmelter,
    Tannery,
    CrystalRefinery,
    AlchemistLab,
    RunicForge,
    ArcaneDistillery,

    // Mall
    Constructor,
    Toolmaker,
    Assembler,

    // Combat
    ImpCamp,
    BreedingPen,
    WarLodge,

    // Energy
    WindTurbine,
    WaterWheel,
    LavaGenerator,
    ManaReactor,

    // Opus
    OpusForge,

    // Utility
    Watchtower,
    Trader,
    SacrificeAltar,
}

impl BuildingType {
    pub fn terrain_req(self) -> Option<TerrainType> {
        match self {
            BuildingType::IronMiner | BuildingType::Miner => Some(TerrainType::IronVein),
            BuildingType::CopperMiner => Some(TerrainType::CopperVein),
            BuildingType::StoneQuarry => Some(TerrainType::StoneDeposit),
            BuildingType::WaterPump => Some(TerrainType::WaterSource),
            BuildingType::ObsidianDrill => Some(TerrainType::ObsidianVein),
            BuildingType::ManaExtractor => Some(TerrainType::ManaNode),
            BuildingType::LavaSiphon => Some(TerrainType::LavaSource),
            BuildingType::WaterWheel => Some(TerrainType::WaterSource),
            BuildingType::LavaGenerator => Some(TerrainType::LavaSource),
            _ => None,
        }
    }

    pub fn tier(self) -> u8 {
        match self {
            BuildingType::ObsidianDrill
            | BuildingType::ManaExtractor
            | BuildingType::LavaSiphon
            | BuildingType::SteelForge
            | BuildingType::SteelSmelter
            | BuildingType::Tannery
            | BuildingType::CrystalRefinery
            | BuildingType::AlchemistLab
            | BuildingType::Toolmaker
            | BuildingType::Assembler
            | BuildingType::WarLodge
            | BuildingType::LavaGenerator
            | BuildingType::Trader => 2,

            BuildingType::RunicForge
            | BuildingType::ArcaneDistillery
            | BuildingType::OpusForge
            | BuildingType::ManaReactor => 3,

            _ => 1,
        }
    }

    /// Footprint (width, height).
    pub fn footprint(self) -> (i32, i32) {
        match self {
            BuildingType::TreeFarm
            | BuildingType::AlchemistLab
            | BuildingType::RunicForge
            | BuildingType::ArcaneDistillery
            | BuildingType::Constructor
            | BuildingType::Assembler
            | BuildingType::ImpCamp
            | BuildingType::BreedingPen
            | BuildingType::WarLodge
            | BuildingType::ManaReactor
            | BuildingType::Trader => (2, 2),

            BuildingType::SteelForge | BuildingType::CrystalRefinery => (2, 1),

            BuildingType::OpusForge => (3, 3),

            _ => (1, 1),
        }
    }

    pub fn energy_consumption(self) -> f32 {
        match self {
            BuildingType::Miner => 1.0,
            BuildingType::Smelter => 1.0,
            BuildingType::EnergySource => 0.0,
            BuildingType::IronMiner => 5.0,
            BuildingType::CopperMiner => 5.0,
            BuildingType::StoneQuarry => 4.0,
            BuildingType::WaterPump => 3.0,
            BuildingType::ObsidianDrill => 12.0,
            BuildingType::ManaExtractor => 15.0,
            BuildingType::LavaSiphon => 8.0,
            BuildingType::IronSmelter => 10.0,
            BuildingType::CopperSmelter => 10.0,
            BuildingType::TreeFarm => 8.0,
            BuildingType::Sawmill => 6.0,
            BuildingType::SteelForge | BuildingType::SteelSmelter => 18.0,
            BuildingType::Tannery => 12.0,
            BuildingType::CrystalRefinery => 20.0,
            BuildingType::AlchemistLab => 16.0,
            BuildingType::RunicForge => 30.0,
            BuildingType::ArcaneDistillery => 28.0,
            BuildingType::Constructor => 15.0,
            BuildingType::Toolmaker => 12.0,
            BuildingType::Assembler => 20.0,
            BuildingType::ImpCamp => 10.0,
            BuildingType::BreedingPen => 8.0,
            BuildingType::WarLodge => 18.0,
            BuildingType::WindTurbine => 0.0,
            BuildingType::WaterWheel => 0.0,
            BuildingType::LavaGenerator => 0.0,
            BuildingType::ManaReactor => 0.0,
            BuildingType::OpusForge => 40.0,
            BuildingType::Watchtower => 2.0,
            BuildingType::Trader => 5.0,
            BuildingType::SacrificeAltar => 3.0,
        }
    }

    pub fn energy_generation(self) -> f32 {
        match self {
            BuildingType::EnergySource => 1.0,
            BuildingType::WindTurbine => 20.0,
            BuildingType::WaterWheel => 25.0,
            BuildingType::LavaGenerator => 50.0,
            BuildingType::ManaReactor => 80.0,
            _ => 0.0,
        }
    }

    pub fn is_mall(self) -> bool {
        matches!(
            self,
            BuildingType::Constructor | BuildingType::Toolmaker | BuildingType::Assembler
        )
    }

    pub fn group_class(self) -> GroupClass {
        match self {
            BuildingType::IronMiner
            | BuildingType::CopperMiner
            | BuildingType::StoneQuarry
            | BuildingType::WaterPump
            | BuildingType::ObsidianDrill
            | BuildingType::ManaExtractor
            | BuildingType::LavaSiphon
            | BuildingType::Miner => GroupClass::Extraction,

            BuildingType::IronSmelter
            | BuildingType::CopperSmelter
            | BuildingType::TreeFarm
            | BuildingType::Sawmill
            | BuildingType::SteelForge
            | BuildingType::SteelSmelter
            | BuildingType::Tannery
            | BuildingType::CrystalRefinery
            | BuildingType::AlchemistLab
            | BuildingType::RunicForge
            | BuildingType::ArcaneDistillery
            | BuildingType::Smelter => GroupClass::Synthesis,

            BuildingType::Constructor
            | BuildingType::Toolmaker
            | BuildingType::Assembler => GroupClass::Mall,

            BuildingType::ImpCamp | BuildingType::BreedingPen | BuildingType::WarLodge => {
                GroupClass::Combat
            }

            BuildingType::WindTurbine
            | BuildingType::WaterWheel
            | BuildingType::LavaGenerator
            | BuildingType::ManaReactor
            | BuildingType::EnergySource => GroupClass::Energy,

            BuildingType::OpusForge => GroupClass::Opus,

            BuildingType::Watchtower | BuildingType::Trader | BuildingType::SacrificeAltar => {
                GroupClass::Utility
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    // Legacy
    IronOre,
    IronBar,

    // T1 Raw
    CopperOre,
    Stone,
    Water,

    // T1 Processed
    CopperBar,
    Plank,
    Wood,

    // T1 Organic
    Hide,
    Herbs,
    BoneMeal,

    // T2 Raw
    ObsidianShard,
    ManaCrystal,
    Lava,

    // T2 Processed
    SteelPlate,
    TreatedLeather,
    RefinedCrystal,
    PotionBase,

    // T2 Organic
    Venom,
    Sinew,

    // T3
    RunicAlloy,
    ArcaneEssence,
    OpusIngot,

    // Building items (mall / Inventory output)
    ItemIronMiner,
    ItemCopperMiner,
    ItemStoneQuarry,
    ItemWindTurbine,
    ItemWatchtower,
    ItemImpCamp,
    ItemBreedingPen,
    ItemWaterPump,
    ItemIronSmelter,
    ItemCopperSmelter,
    ItemTreeFarm,
    ItemSawmill,
    ItemWaterWheel,
    ItemSacrificeAltar,
    ItemToolmaker,
    ItemAssembler,
    ItemObsidianDrill,
    ItemManaExtractor,
    ItemWarLodge,
    ItemTrader,
    ItemSteelForge,
    ItemTannery,
    ItemCrystalRefinery,
    ItemAlchemistLab,
    ItemLavaSiphon,
    ItemLavaGenerator,
    ItemRunicForge,
    ItemArcaneDistillery,
    ItemOpusForge,
    ItemManaReactor,
}

// ─────────────────────────────────────────────────────────────────────────────
// Terrain
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainType {
    Grass,
    IronVein,
    CopperVein,
    StoneDeposit,
    WaterSource,
    ObsidianVein,
    ManaNode,
    LavaSource,
}

impl Default for TerrainType {
    fn default() -> Self {
        TerrainType::Grass
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Group classification
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupClass {
    Extraction,
    Synthesis,
    Mall,
    Combat,
    Energy,
    Opus,
    Utility,
}

// ─────────────────────────────────────────────────────────────────────────────
// Priority / Status enums
// ─────────────────────────────────────────────────────────────────────────────

/// Energy-distribution priority (legacy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EnergyPriority {
    High,
    #[default]
    Medium,
    Low,
}

/// Group management priority (for player control).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GroupPriority {
    Low,
    #[default]
    Medium,
    High,
}

/// Whether a group is running or paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GroupStatus {
    #[default]
    Active,
    Paused,
}

/// Reason a building is not producing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdleReason {
    NoInputs,
    NoEnergy,
    GroupPaused,
}

// ─────────────────────────────────────────────────────────────────────────────
// ECS Components
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}

#[derive(Component)]
pub struct GroupMember {
    pub group_id: Entity,
}

#[derive(Component, Clone)]
pub struct Recipe {
    pub inputs: Vec<(ResourceType, f32)>,
    pub outputs: Vec<(ResourceType, f32)>,
    pub duration_ticks: u32,
    /// If true, outputs go to Inventory instead of group manifold (mall buildings).
    pub output_to_inventory: bool,
}

impl Recipe {
    pub fn simple(
        inputs: Vec<(ResourceType, f32)>,
        outputs: Vec<(ResourceType, f32)>,
        duration_ticks: u32,
    ) -> Self {
        Self { inputs, outputs, duration_ticks, output_to_inventory: false }
    }

    pub fn mall(
        inputs: Vec<(ResourceType, f32)>,
        outputs: Vec<(ResourceType, f32)>,
        duration_ticks: u32,
    ) -> Self {
        Self { inputs, outputs, duration_ticks, output_to_inventory: true }
    }
}

#[derive(Component)]
pub struct ProductionState {
    pub progress: f32,
    pub active: bool,
    pub idle_reason: Option<IdleReason>,
}

impl Default for ProductionState {
    fn default() -> Self {
        Self { progress: 0.0, active: false, idle_reason: None }
    }
}

#[derive(Component, Default)]
pub struct InputBuffer {
    pub slots: HashMap<ResourceType, f32>,
}

impl InputBuffer {
    pub fn get(&self, r: &ResourceType) -> Option<&f32> {
        self.slots.get(r)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceType, &f32)> {
        self.slots.iter()
    }
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[derive(Component, Default)]
pub struct OutputBuffer {
    pub slots: HashMap<ResourceType, f32>,
}

impl OutputBuffer {
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceType, &f32)> {
        self.slots.iter()
    }
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[derive(Component)]
pub struct Group;

#[derive(Component)]
pub struct GroupEnergy {
    pub demand: f32,
    pub allocated: f32,
    pub priority: EnergyPriority,
}

impl Default for GroupEnergy {
    fn default() -> Self {
        Self { demand: 0.0, allocated: 0.0, priority: EnergyPriority::Medium }
    }
}

impl GroupEnergy {
    pub fn ratio(&self) -> f32 {
        if self.demand > 0.0 {
            (self.allocated / self.demand).clamp(0.0, 1.5)
        } else {
            1.0
        }
    }
}

#[derive(Component, Default)]
pub struct Manifold {
    pub resources: HashMap<ResourceType, f32>,
}

/// Group management control: priority + pause/resume.
#[derive(Component, Default)]
pub struct GroupControl {
    pub priority: GroupPriority,
    pub status: GroupStatus,
}

/// Aggregate production/consumption stats tracked per group.
#[derive(Component, Default)]
pub struct GroupStats {
    pub produced: HashMap<ResourceType, f32>,
    pub consumed: HashMap<ResourceType, f32>,
    pub ticks: u32,
}

/// Determined group class (Extraction / Synthesis / Mall / Combat …).
#[derive(Component)]
pub struct GroupType {
    pub class: GroupClass,
}

/// Multi-cell footprint of a building (the origin entity owns this).
#[derive(Component)]
pub struct Footprint {
    pub cells: Vec<(i32, i32)>,
}

impl Footprint {
    pub fn single(x: i32, y: i32) -> Self {
        Self { cells: vec![(x, y)] }
    }

    pub fn rect(ox: i32, oy: i32, w: i32, h: i32) -> Self {
        let mut cells = Vec::with_capacity((w * h) as usize);
        for dy in 0..h {
            for dx in 0..w {
                cells.push((ox + dx, oy + dy));
            }
        }
        Self { cells }
    }
}

/// Output sender port on a group boundary.
#[derive(Component)]
pub struct OutputSender {
    pub group_id: Entity,
    pub resource: Option<ResourceType>,
    pub boundary_pos: (i32, i32),
}

/// Input receiver port on a group boundary.
#[derive(Component)]
pub struct InputReceiver {
    pub group_id: Entity,
    pub resource: Option<ResourceType>,
    pub boundary_pos: (i32, i32),
}

/// Transport link between a sender and a receiver.
#[derive(Component)]
pub struct TransportLink {
    pub from_sender: Entity,
    pub to_receiver: Entity,
}

// ─────────────────────────────────────────────────────────────────────────────
// Energy generation / consumption components (legacy ECS approach)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct EnergyGeneration {
    pub base: f32,
    pub effective: f32,
}

impl EnergyGeneration {
    pub fn new(base: f32) -> Self {
        Self { base, effective: base }
    }
}

#[derive(Component)]
pub struct EnergyConsumption {
    pub amount: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Placement validation components
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TierRequirement {
    pub min_tier: u32,
}

#[derive(Component)]
pub struct TerrainRequirement {
    pub terrain: TerrainType,
}

// ─────────────────────────────────────────────────────────────────────────────
// Opus / Progression
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct GroupLabel {
    pub name: String,
}

#[derive(Component)]
pub struct OpusNode {
    pub resource: ResourceType,
    pub required_rate: f32,
    pub sustained: bool,
}

#[derive(Component)]
pub struct OpusTree {
    pub total_nodes: usize,
}

// ── Progression BDD — Additional Types ───────────────────────────────────────

/// Opus difficulty level (for progression/scoring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpusDifficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

impl OpusDifficulty {
    pub fn rate_multiplier(self) -> f32 {
        match self {
            OpusDifficulty::Easy => 0.7,
            OpusDifficulty::Medium => 1.0,
            OpusDifficulty::Hard => 1.4,
            OpusDifficulty::Extreme => 1.8,
        }
    }

    pub fn currency_multiplier(self) -> f32 {
        match self {
            OpusDifficulty::Easy => 1.0,
            OpusDifficulty::Medium => 1.5,
            OpusDifficulty::Hard => 2.0,
            OpusDifficulty::Extreme => 3.0,
        }
    }
}

/// Mini-opus trigger type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiniOpusTrigger {
    OnDemand,
    TimeBased,
    Conditional,
}

/// Mini-opus type identifier (maps to mini_opus_types.yaml).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiniOpusKind {
    TradeSurplus,
    BuildMonument,
    SpeedProduction,
    SurviveHazardProducing,
    ClearNestFast,
    ZeroWaste,
    OrganicSurplus,
}

/// Status of a mini-opus branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiniOpusStatus {
    Active,
    Completed,
    Missed,
}

/// Mini-opus side branch attached to a main-path node.
#[derive(Component, Debug, Clone)]
pub struct MiniOpusBranch {
    pub id: String,
    pub parent_node: u32,
    pub kind: MiniOpusKind,
    pub trigger: MiniOpusTrigger,
    pub status: MiniOpusStatus,
    pub reward_currency: MetaCurrency,
    pub reward_amount: u32,
    /// For time-based: deadline tick.
    pub deadline_tick: Option<u64>,
    /// For conditional: current progress value.
    pub condition_value: f32,
    /// Required condition threshold.
    pub condition_threshold: f32,
}

/// Tier gate — clears when a specific nest is destroyed.
#[derive(Component, Debug, Clone)]
pub struct TierGateComponent {
    pub tier: u32,
    pub nest_id: String,
    pub unlocked: bool,
}

/// Tier marker on a building entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct BuildingTier {
    pub tier: u32,
}

/// Player inventory — maps building name to count (starting kit + mall output).
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInventory {
    pub buildings: HashMap<String, u32>,
}

/// Extended opus node with full milestone data (replaces the basic OpusNode above).
/// We use a separate name to avoid conflict with the existing minimal OpusNode.
#[derive(Component, Debug, Clone)]
pub struct OpusNodeFull {
    pub node_index: u32,
    pub resource: ResourceType,
    /// Required production rate in units per minute.
    pub required_rate: f32,
    pub tier: u32,
    /// True once the rate has been sustained for the full sustain_window_ticks.
    pub sustained: bool,
    /// Consecutive ticks at or above required_rate.
    pub sustain_ticks: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Meta / Trading types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaCurrency {
    Gold,
    Souls,
    Knowledge,
}

/// Rate definition for a resource in the trading system.
pub struct TradingRateDef {
    pub currency: MetaCurrency,
    /// Base rate (resource units → meta-currency per unit).
    pub rate: f32,
}

/// Returns the trading rate for a given resource, or None if not tradeable.
/// Rates derived from seed data (meta/trading.yaml).
///
/// Currency assignment:
///   Gold      — T1 raw + T1 processed (non-organic)
///   Souls     — all organic resources
///   Knowledge — T2+ processed, T3, and mana/obsidian materials
pub fn find_trading_rate(resource: ResourceType) -> Option<TradingRateDef> {
    match resource {
        // ── T1 Raw → Gold ───────────────────────────────────────────────
        ResourceType::IronOre   => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 0.5  }),
        ResourceType::CopperOre => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 0.5  }),
        ResourceType::Stone     => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 0.3  }),
        ResourceType::Wood      => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 0.4  }),

        // ── T1 Processed → Gold ─────────────────────────────────────────
        ResourceType::IronBar   => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 1.0  }),
        ResourceType::CopperBar => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 1.0  }),
        ResourceType::Plank     => Some(TradingRateDef { currency: MetaCurrency::Gold,      rate: 0.6  }),

        // ── T1 Organic → Souls ──────────────────────────────────────────
        ResourceType::Hide      => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 1.5  }),
        ResourceType::Herbs     => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 1.0  }),
        ResourceType::BoneMeal  => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 1.2  }),

        // ── T2 Raw (obsidian/mana) → Knowledge ─────────────────────────
        ResourceType::ObsidianShard => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 1.5 }),
        ResourceType::ManaCrystal   => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 2.0 }),

        // ── T2 Processed → Knowledge / mixed ───────────────────────────
        ResourceType::SteelPlate      => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 2.0  }),
        ResourceType::TreatedLeather  => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 2.5  }),
        ResourceType::RefinedCrystal  => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 2.5  }),
        ResourceType::PotionBase      => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 3.0  }),

        // ── T2 Organic → Souls ──────────────────────────────────────────
        ResourceType::Venom     => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 3.0  }),
        ResourceType::Sinew     => Some(TradingRateDef { currency: MetaCurrency::Souls,     rate: 2.0  }),

        // ── T3 → Knowledge ──────────────────────────────────────────────
        ResourceType::RunicAlloy    => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 5.0  }),
        ResourceType::ArcaneEssence => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 7.0  }),
        ResourceType::OpusIngot     => Some(TradingRateDef { currency: MetaCurrency::Knowledge, rate: 10.0 }),

        // Building items / water / lava are not tradeable
        _ => None,
    }
}

/// Per-trader inflation state (tracks volume traded per resource for log inflation).
#[derive(Component)]
pub struct TraderState {
    pub volume_traded: std::collections::HashMap<ResourceType, f32>,
    /// Inflation factor (0.3 from seed). Lower = faster inflation.
    pub inflation_factor: f32,
}

impl Default for TraderState {
    fn default() -> Self {
        Self { volume_traded: Default::default(), inflation_factor: 0.3 }
    }
}

impl TraderState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute effective rate with logarithmic inflation.
    /// effective_rate = base_rate / (1 + inflation_factor * ln(1 + volume))
    pub fn effective_rate(&self, resource: ResourceType, base_rate: f32) -> f32 {
        let volume = self.volume_traded.get(&resource).copied().unwrap_or(0.0);
        base_rate / (1.0 + self.inflation_factor * (1.0 + volume).ln())
    }

    pub fn record_trade(&mut self, resource: ResourceType, amount: f32) {
        *self.volume_traded.entry(resource).or_default() += amount;
    }
}

/// Per-trader accumulated earnings.
#[derive(Component, Default)]
pub struct TraderEarnings {
    pub gold: f32,
    pub souls: f32,
    pub knowledge: f32,
}

impl TraderEarnings {
    pub fn add(&mut self, currency: MetaCurrency, amount: f32) {
        match currency {
            MetaCurrency::Gold      => self.gold += amount,
            MetaCurrency::Souls     => self.souls += amount,
            MetaCurrency::Knowledge => self.knowledge += amount,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Transport feature — components
// ═══════════════════════════════════════════════════════════════════════════

/// Whether a resource is a solid or a liquid (determines transport method).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceClass {
    Solid,
    Liquid,
}

impl ResourceType {
    pub fn class(self) -> ResourceClass {
        match self {
            ResourceType::Water | ResourceType::Lava => ResourceClass::Liquid,
            _ => ResourceClass::Solid,
        }
    }
}

/// Whether a transport channel carries solids (RunePath) or liquids (Pipe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportKind {
    RunePath,
    Pipe,
}

/// Stats for a path or pipe at a given tier (derived from seed data).
#[derive(Debug, Clone, Copy)]
pub struct TierStats {
    pub capacity: u32,
    pub speed: f32,
}

impl TierStats {
    pub fn for_path(tier: u8) -> Self {
        match tier {
            1 => TierStats { capacity: 2, speed: 1.0 },
            2 => TierStats { capacity: 5, speed: 2.0 },
            3 => TierStats { capacity: 10, speed: 3.0 },
            _ => TierStats { capacity: 10, speed: 3.0 },
        }
    }

    pub fn for_pipe(tier: u8) -> Self {
        match tier {
            1 => TierStats { capacity: 3, speed: 1.5 },
            2 => TierStats { capacity: 8, speed: 3.0 },
            3 => TierStats { capacity: 15, speed: 4.5 },
            _ => TierStats { capacity: 15, speed: 4.5 },
        }
    }
}

/// A rune path (solid) or pipe (liquid) transport entity between two groups.
#[derive(Component)]
pub struct TransportPath {
    pub kind: TransportKind,
    pub source_group: Entity,
    pub target_group: Entity,
    pub resource_filter: Option<ResourceType>,
    pub tier: u8,
    pub capacity: u32,
    pub speed: f32,
    pub connected: bool,
    /// Ordered waypoint tiles of the path.
    pub segments: Vec<(i32, i32)>,
}

/// One tile in a TransportPath — kept separately for occupancy tracking.
#[derive(Component)]
pub struct PathSegmentTile {
    pub path_entity: Entity,
    pub tile_pos: (i32, i32),
    pub segment_index: usize,
}

/// Logical connection record attached alongside a TransportPath.
#[derive(Component)]
pub struct PathConnection {
    pub source_group: Entity,
    pub target_group: Entity,
    pub path_entity: Entity,
}

/// A resource unit in transit on a transport path or pipe.
#[derive(Component)]
pub struct Cargo {
    pub path_entity: Entity,
    pub resource: ResourceType,
    pub amount: f32,
    pub position_on_path: f32,
}

/// A hazard zone that can destroy path segments when it fires.
#[derive(Component)]
pub struct HazardZone {
    pub center: (i32, i32),
    pub radius: i32,
    pub next_event_tick: u64,
}

/// Position marker placed on a Group entity for minion-range calculations.
#[derive(Component)]
pub struct GroupPosition {
    pub x: i32,
    pub y: i32,
}

/// Transport-specific sender port (simpler than the existing OutputSender).
/// Placed directly on a Group entity.
#[derive(Component, Default)]
pub struct TransportSender {
    pub resource: Option<ResourceType>,
    pub disconnected: bool,
}

/// Transport-specific receiver port.
/// Placed directly on a Group entity.
#[derive(Component, Default)]
pub struct TransportReceiver {
    pub resource: Option<ResourceType>,
    /// Units per tick the group demands (0 = saturated).
    pub demand: u32,
    pub disconnected: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Creatures feature — components
// ═══════════════════════════════════════════════════════════════════════════

/// Biome identifier for creature spawning and world logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum BiomeTag {
    Forest,
    Volcanic,
    Desert,
    Ocean,
}

/// Creature behavior archetype — 5 types across all biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureArchetype {
    Ambient,
    Territorial,
    Invasive,
    EventBorn,
    OpusLinked,
}

/// Creature state machine variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureStateKind {
    Idle,
    Patrolling,
    Aggressive,
    Fleeing,
    Wandering,
    Despawned,
    Decorating,
}

/// Species identifier for a creature entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureSpecies {
    ForestDeer,
    ForestWolf,
    ForestVineCreeper,
    LavaSalamander,
    AshSwarm,
    EmberWyrm,
    SandBeetle,
    DuneScorpion,
    CrystalGolem,
    TideCrab,
    ReefSerpent,
    StormLeviathan,
}

/// Main creature component — carries identity, health, and state.
#[derive(Component)]
pub struct Creature {
    pub species: CreatureSpecies,
    pub archetype: CreatureArchetype,
    pub biome: BiomeTag,
    pub health: f32,
    pub max_health: f32,
    pub state: CreatureStateKind,
}

/// Territorial behavior data — patrol + aggression trigger.
#[derive(Component)]
pub struct TerritoryData {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: f32,
    pub attack_dps: f32,
}

/// Invasive behavior data — territory expands per tick.
#[derive(Component)]
pub struct InvasiveData {
    pub expansion_rate: f32,           // radius growth per tick
    pub spawn_children_at_radius: f32, // spawn child when territory >= this
    pub child_spawn_rate: f32,
}

/// EventBorn behavior — exists for a limited number of ticks.
#[derive(Component)]
pub struct EventBornData {
    pub lifetime_ticks: u32,
    pub ticks_alive: u32,
    pub attack_dps: f32,
}

/// Ambient behavior — wanders within home range.
#[derive(Component)]
pub struct AmbientData {
    pub wander_range: f32,
    pub home_x: i32,
    pub home_y: i32,
    pub flee_threshold: f32, // flee when health < this fraction of max
}

/// OpusLinked — spawns when a specific main opus milestone is sustained.
#[derive(Component)]
pub struct OpusLinkedData {
    pub spawn_trigger_milestone: u32,
}

/// Loot table on a creature entity — resources dropped on kill.
#[derive(Component, Default, Clone)]
pub struct LootTable {
    pub drops: HashMap<ResourceType, u32>,
}

/// Nest entity — tier gate; clearing one unlocks the next tier.
#[derive(Component)]
pub struct CreatureNest {
    pub nest_id: NestId,
    pub biome: BiomeTag,
    pub tier: u8,
    pub hostility: NestHostility,
    pub strength: f32,
    pub territory_radius: f32,
    pub cleared: bool,
    pub extracting: bool,
    pub loot_on_clear: HashMap<ResourceType, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NestHostility {
    Hostile,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NestId {
    ForestWolfDen,
    VolcanicSalamanderNest,
    DesertBeetleBurrow,
    OceanReefNest,
    ForestVineHeart,
    VolcanicWyrmLair,
    DesertScorpionHive,
    OceanDeepRift,
    ForestDeerGrove,
}

/// Combat group building component — imp_camp, breeding_pen, war_lodge.
#[derive(Component)]
pub struct CombatGroup {
    pub building_kind: CombatBuildingKind,
    pub base_organic_rate: f32,
    pub base_protection_radius: f32,
    pub protection_dps: f32,
    pub breach_threshold: f32,
    pub supply_ratio: f32,  // 0.0–1.0
    pub max_minions: u32,
    pub output_multiplier: f32,
    pub consumption_multiplier: f32,
}

impl CombatGroup {
    /// Organic items produced per cycle = base * supply_ratio * output_multiplier
    pub fn effective_organic_rate(&self) -> f32 {
        self.base_organic_rate * self.supply_ratio * self.output_multiplier
    }

    /// Protection radius = base * supply_ratio
    pub fn effective_protection_radius(&self) -> f32 {
        self.base_protection_radius * self.supply_ratio
    }

    /// Protection DPS = protection_dps * supply_ratio
    pub fn effective_protection_dps(&self) -> f32 {
        self.protection_dps * self.supply_ratio
    }

    /// floor(max_minions * supply_ratio)
    pub fn visible_minion_count(&self) -> u32 {
        (self.max_minions as f32 * self.supply_ratio).floor() as u32
    }

    /// True when supply_ratio < breach_threshold (enemies break through)
    pub fn is_breached(&self) -> bool {
        self.supply_ratio < self.breach_threshold
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CombatBuildingKind {
    ImpCamp,
    BreedingPen,
    WarLodge,
}

/// Applied combat pressure value from a combat group toward a nest.
#[derive(Component)]
pub struct CombatPressure {
    pub value: f32,
}

/// Terrain vein entity — produces non-organic resources only.
#[derive(Component)]
pub struct ResourceVein {
    pub resource: ResourceType,
}

/// Trader building — converts manifold surplus to meta-currencies with inflation.
#[derive(Component)]
pub struct TraderBuilding {
    pub exchange_rates: HashMap<ResourceType, f32>,
    pub trade_volume: HashMap<ResourceType, f32>,
    pub inflation_factor: f32,
    pub currency_tag: HashMap<ResourceType, MetaCurrencyKind>,
}

impl TraderBuilding {
    /// Effective rate after linear inflation: base / (1 + inflation_factor * volume)
    pub fn effective_rate(&self, resource: ResourceType) -> f32 {
        let base = self.exchange_rates.get(&resource).copied().unwrap_or(0.0);
        let volume = self.trade_volume.get(&resource).copied().unwrap_or(0.0);
        base / (1.0 + self.inflation_factor * volume)
    }
}

/// Meta-currency kind as used in creature-related trading (AC12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaCurrencyKind {
    Gold,
    Souls,
    Knowledge,
}

/// Minion entity — idle minions can decorate nearby buildings.
#[derive(Component)]
pub struct Minion {
    pub task: MinionTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinionTask {
    Idle,
    Decorating,
    Production,
}


// ═══════════════════════════════════════════════════════════════════════════
// World / Terrain / Hazard feature stubs
// Required for compilation of terrain.rs, ux.rs, events.rs, resources.rs.
// Full types arrive in the world/ux impl stage.
// ═══════════════════════════════════════════════════════════════════════════

/// Extended terrain types for the world map (all biomes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub enum TerrainTypeWorld {
    #[default]
    Grass,
    DenseForest,
    WaterSource,
    StoneDeposit,
    IronVein,
    CopperVein,
    ManaNode,
    Impassable,
    // Volcanic
    ScorchedRock,
    LavaSource,
    ObsidianVein,
    // Desert
    Sand,
    Dune,
    Oasis,
    // Ocean
    ShallowWater,
    CoralReef,
    // Element-transformed
    DrySoil,
    WetSoil,
    Ice,
}

impl TerrainTypeWorld {
    pub fn is_buildable(&self) -> bool {
        !matches!(
            self,
            TerrainTypeWorld::Impassable | TerrainTypeWorld::ShallowWater
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileVisibility {
    #[default]
    Hidden,
    Revealed,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    Rain,
    HeavyRain,
    Wind,
    Heat,
    ColdSnap,
    Fog,
    Storm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BiomeId {
    #[default]
    Forest,
    Volcanic,
    Desert,
    Ocean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ResourceQuality {
    #[default]
    Normal,
    High,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HazardKind {
    Eruption,
    AshStorm,
    Wildfire,
    Storm,
    Sandstorm,
    HeatWave,
    Tsunami,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Default)]
pub enum Tier {
    #[default]
    T1,
    T2,
    T3,
}

/// A tile entity in the world map.
#[derive(Component)]
pub struct WorldTile {
    pub x: i32,
    pub y: i32,
    pub terrain: TerrainTypeWorld,
    pub visibility: TileVisibility,
    pub biome: BiomeId,
    /// None = infinite (water_source, lava_source).
    pub remaining: Option<f32>,
}

/// A recurring biome hazard zone with full scheduling data.
#[derive(Component)]
pub struct BiomeHazard {
    pub hazard_kind: HazardKind,
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub intensity: f32,
    pub next_event_tick: u32,
    pub warning_ticks: u32,
    pub interval_ticks: u32,
    pub interval_variance: u32,
    pub warning_issued: bool,
}

/// Active hazard warning — present N ticks before the event.
#[derive(Component)]
pub struct HazardWarning {
    pub hazard_kind: HazardKind,
    pub center_x: i32,
    pub center_y: i32,
    pub ticks_remaining: u32,
}

/// Tile enhancement buff applied after a hazard event.
#[derive(Component)]
pub struct TileEnhancement {
    pub enhancement_type: EnhancementType,
    /// Multiplier applied to extraction on enhanced tiles (e.g. 1.5 = +50%).
    pub magnitude: f32,
    pub remaining_ticks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnhancementType {
    Enriched,
    CharredFertile,
    UncoveredDeposit,
    TidalDeposit,
    FertileAsh,
    Waterlogged,
    GlassSand,
}

/// Elemental levels per tile; each 0.0–1.0. Wind is set by weather each tick.
#[derive(Component, Default)]
pub struct ElementalState {
    pub fire: f32,
    pub water: f32,
    pub cold: f32,
    pub wind: f32,
}

/// Reveals fog around a building within a configurable manhattan radius.
#[derive(Component)]
pub struct FogRevealer {
    pub radius: i32,
}

/// Sacrifice altar state — may receive hazard bonus or be destroyed.
#[derive(Component)]
pub struct SacrificeBuilding {
    pub in_hazard_zone: bool,
    /// None = not in a hazard zone; Some(p) = probability of success.
    pub success_chance: Option<f32>,
}

/// A rune-path segment tile (solid transport) on the world map.
#[derive(Component)]
pub struct RunePathSegment {
    pub x: i32,
    pub y: i32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Meta feature — types
// ═══════════════════════════════════════════════════════════════════════════

/// Opus + biome difficulty category, determines the meta-currency multiplier.
/// Score = biome_rank + opus_rank (each 1–3, so total 2–6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DifficultyCategory {
    Easy,    // score 2–3  → ×1.0
    Medium,  // score 4    → ×1.5
    Hard,    // score 5    → ×2.0
    Extreme, // score 6    → ×3.0
}

impl DifficultyCategory {
    /// Map a combined biome+opus difficulty score (2–6) to a DifficultyCategory.
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=3 => DifficultyCategory::Easy,
            4     => DifficultyCategory::Medium,
            5     => DifficultyCategory::Hard,
            _     => DifficultyCategory::Extreme,
        }
    }

    /// Reward multiplier applied to all meta-currencies at run end.
    pub fn opus_multiplier(self) -> f32 {
        match self {
            DifficultyCategory::Easy    => 1.0,
            DifficultyCategory::Medium  => 1.5,
            DifficultyCategory::Hard    => 2.0,
            DifficultyCategory::Extreme => 3.0,
        }
    }
}

/// Player's persistent meta-hub wallet (gold / souls / knowledge).
#[derive(Debug, Clone, Default)]
pub struct MetaWallet {
    pub gold: f32,
    pub souls: f32,
    pub knowledge: f32,
    pub lifetime_earned_gold: f32,
    pub lifetime_earned_souls: f32,
    pub lifetime_earned_knowledge: f32,
    pub lifetime_spent_gold: f32,
    pub lifetime_spent_souls: f32,
    pub lifetime_spent_knowledge: f32,
}

/// A single reward entry from a mini-opus branch.
#[derive(Debug, Clone)]
pub struct MiniOpusReward {
    pub currency: MetaCurrency,
    pub amount: f32,
}

/// Summary of a completed run — used to compute final meta-currency awards.
#[derive(Debug, Clone)]
pub struct RunEndResult {
    pub difficulty: DifficultyCategory,
    pub mini_opus_rewards: Vec<MiniOpusReward>,
    /// True if the player quit mid-run (yields zero currency).
    pub abandoned: bool,
    pub mini_opus_completed: u32,
    pub mini_opus_total: u32,
    /// True if ALL mini-opus branches failed → 25% penalty applied.
    pub all_failed_penalty: bool,
}

impl RunEndResult {
    /// Calculate (gold, souls, knowledge) earned at run end.
    ///
    /// Rules:
    /// - If abandoned → (0, 0, 0)
    /// - Apply `difficulty.opus_multiplier()` to each reward
    /// - If `all_failed_penalty` → further multiply by 0.25
    pub fn calculate_awards(&self) -> (f32, f32, f32) {
        if self.abandoned {
            return (0.0_f32, 0.0_f32, 0.0_f32);
        }

        let mult = self.difficulty.opus_multiplier()
            * if self.all_failed_penalty { 0.25_f32 } else { 1.0_f32 };

        let mut gold = 0.0_f32;
        let mut souls = 0.0_f32;
        let mut knowledge = 0.0_f32;

        for reward in &self.mini_opus_rewards {
            let earned = reward.amount * mult;
            match reward.currency {
                MetaCurrency::Gold      => gold += earned,
                MetaCurrency::Souls     => souls += earned,
                MetaCurrency::Knowledge => knowledge += earned,
            }
        }

        (gold, souls, knowledge)
    }
}

/// Set of purchased meta-store unlocks that persist across runs.
#[derive(Debug, Clone, Default)]
pub struct PurchasedUnlocks {
    pub ids: Vec<String>,
}

impl PurchasedUnlocks {
    pub fn add(&mut self, id: &str) {
        self.ids.push(id.to_string());
    }

    pub fn has(&self, id: &str) -> bool {
        self.ids.iter().any(|s| s.as_str() == id)
    }
}

/// Category of an unlock in the meta store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnlockCategory {
    Biome,
    StartingBonus,
    BuildingPool,
    Cosmetic,
}

/// Static definition of a meta-store unlock.
pub struct UnlockDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: UnlockCategory,
    pub cost_gold: u32,
    pub cost_souls: u32,
    pub cost_knowledge: u32,
}

impl UnlockDef {
    /// Attempt to purchase this unlock from the wallet.
    ///
    /// Fails atomically if:
    /// - already purchased, or
    /// - wallet lacks sufficient currency for any cost component.
    ///
    /// On success, deducts costs and records the unlock.
    pub fn try_purchase(
        &self,
        wallet: &mut MetaWallet,
        purchased: &mut PurchasedUnlocks,
    ) -> bool {
        // Already owned
        if purchased.has(self.id) {
            return false;
        }
        // Check all costs
        if wallet.gold      < self.cost_gold      as f32 { return false; }
        if wallet.souls     < self.cost_souls     as f32 { return false; }
        if wallet.knowledge < self.cost_knowledge as f32 { return false; }

        // Atomic deduction
        wallet.gold      -= self.cost_gold      as f32;
        wallet.souls     -= self.cost_souls     as f32;
        wallet.knowledge -= self.cost_knowledge as f32;
        wallet.lifetime_spent_gold      += self.cost_gold      as f32;
        wallet.lifetime_spent_souls     += self.cost_souls     as f32;
        wallet.lifetime_spent_knowledge += self.cost_knowledge as f32;

        purchased.add(self.id);
        true
    }
}

/// Full unlock catalog (11 unlocks from seed data).
static UNLOCK_CATALOG: &[UnlockDef] = &[
    // Biome unlocks
    UnlockDef {
        id: "unlock_volcanic",
        name: "Ashen Caldera",
        category: UnlockCategory::Biome,
        cost_gold: 200,
        cost_souls: 100,
        cost_knowledge: 0,
    },
    UnlockDef {
        id: "unlock_desert",
        name: "Sunscorched Wastes",
        category: UnlockCategory::Biome,
        cost_gold: 200,
        cost_souls: 0,
        cost_knowledge: 100,
    },
    UnlockDef {
        id: "unlock_ocean",
        name: "Abyssal Shore",
        category: UnlockCategory::Biome,
        cost_gold: 150,
        cost_souls: 150,
        cost_knowledge: 0,
    },
    // Starting bonus unlocks
    UnlockDef {
        id: "extra_starting_miner",
        name: "Prospector's Gift",
        category: UnlockCategory::StartingBonus,
        cost_gold: 300,
        cost_souls: 0,
        cost_knowledge: 0,
    },
    UnlockDef {
        id: "extra_starting_turbine",
        name: "Wind Blessing",
        category: UnlockCategory::StartingBonus,
        cost_gold: 250,
        cost_souls: 0,
        cost_knowledge: 0,
    },
    UnlockDef {
        id: "starting_combat",
        name: "War Preparation",
        category: UnlockCategory::StartingBonus,
        cost_gold: 0,
        cost_souls: 400,
        cost_knowledge: 0,
    },
    UnlockDef {
        id: "starting_watchtower_upgrade",
        name: "Sentinel's Eye",
        category: UnlockCategory::StartingBonus,
        cost_gold: 0,
        cost_souls: 200,
        cost_knowledge: 100,
    },
    // Building pool unlocks
    UnlockDef {
        id: "unlock_alchemist_lab",
        name: "Alchemist's Secret",
        category: UnlockCategory::BuildingPool,
        cost_gold: 0,
        cost_souls: 0,
        cost_knowledge: 500,
    },
    UnlockDef {
        id: "unlock_mana_reactor",
        name: "Arcane Power",
        category: UnlockCategory::BuildingPool,
        cost_gold: 0,
        cost_souls: 200,
        cost_knowledge: 600,
    },
    // Cosmetic unlocks
    UnlockDef {
        id: "golden_rune_paths",
        name: "Golden Runes",
        category: UnlockCategory::Cosmetic,
        cost_gold: 100,
        cost_souls: 0,
        cost_knowledge: 0,
    },
    UnlockDef {
        id: "spectral_minions",
        name: "Spectral Workers",
        category: UnlockCategory::Cosmetic,
        cost_gold: 0,
        cost_souls: 150,
        cost_knowledge: 0,
    },
];

/// All 11 unlock IDs in catalog order.
pub const ALL_UNLOCKS: &[&str] = &[
    "unlock_volcanic",
    "unlock_desert",
    "unlock_ocean",
    "extra_starting_miner",
    "extra_starting_turbine",
    "starting_combat",
    "starting_watchtower_upgrade",
    "unlock_alchemist_lab",
    "unlock_mana_reactor",
    "golden_rune_paths",
    "spectral_minions",
];

/// Look up a static unlock definition by ID, or None if not found.
pub fn find_unlock(id: &str) -> Option<&'static UnlockDef> {
    UNLOCK_CATALOG.iter().find(|u| u.id == id)
}
