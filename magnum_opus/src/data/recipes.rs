use crate::components::{BuildingType, Recipe, ResourceType};

/// Default recipe for a building type. Used when placing buildings from UI.
pub fn default_recipe(bt: BuildingType) -> Recipe {
    use ResourceType as RT;

    match bt {
        // Extractors — no inputs
        BuildingType::IronMiner => Recipe {
            inputs: vec![],
            outputs: vec![(RT::IronOre, 1.0)],
            duration_ticks: 60,
            output_to_inventory: false,
        },
        BuildingType::CopperMiner => Recipe {
            inputs: vec![],
            outputs: vec![(RT::CopperOre, 1.0)],
            duration_ticks: 60,
            output_to_inventory: false,
        },
        BuildingType::StoneQuarry => Recipe {
            inputs: vec![],
            outputs: vec![(RT::Stone, 2.0)],
            duration_ticks: 80,
            output_to_inventory: false,
        },
        BuildingType::WaterPump => Recipe {
            inputs: vec![],
            outputs: vec![(RT::Water, 2.0)],
            duration_ticks: 40,
            output_to_inventory: false,
        },
        BuildingType::ObsidianDrill => Recipe {
            inputs: vec![],
            outputs: vec![(RT::ObsidianShard, 1.0)],
            duration_ticks: 100,
            output_to_inventory: false,
        },
        BuildingType::ManaExtractor => Recipe {
            inputs: vec![],
            outputs: vec![(RT::ManaCrystal, 1.0)],
            duration_ticks: 120,
            output_to_inventory: false,
        },
        BuildingType::LavaSiphon => Recipe {
            inputs: vec![],
            outputs: vec![(RT::Lava, 3.0)],
            duration_ticks: 60,
            output_to_inventory: false,
        },
        // T1 Synthesis
        BuildingType::IronSmelter => Recipe {
            inputs: vec![(RT::IronOre, 2.0)],
            outputs: vec![(RT::IronBar, 1.0)],
            duration_ticks: 120,
            output_to_inventory: false,
        },
        BuildingType::CopperSmelter => Recipe {
            inputs: vec![(RT::CopperOre, 2.0)],
            outputs: vec![(RT::CopperBar, 1.0)],
            duration_ticks: 120,
            output_to_inventory: false,
        },
        BuildingType::TreeFarm => Recipe {
            inputs: vec![(RT::Water, 3.0)],
            outputs: vec![(RT::Wood, 2.0)],
            duration_ticks: 180,
            output_to_inventory: false,
        },
        BuildingType::Sawmill => Recipe {
            inputs: vec![(RT::Wood, 1.0)],
            outputs: vec![(RT::Plank, 2.0)],
            duration_ticks: 80,
            output_to_inventory: false,
        },
        // T2 Synthesis
        BuildingType::SteelForge | BuildingType::SteelSmelter => Recipe {
            inputs: vec![(RT::IronBar, 2.0), (RT::CopperBar, 1.0)],
            outputs: vec![(RT::SteelPlate, 1.0)],
            duration_ticks: 200,
            output_to_inventory: false,
        },
        BuildingType::Tannery => Recipe {
            inputs: vec![(RT::Hide, 3.0), (RT::Herbs, 1.0)],
            outputs: vec![(RT::TreatedLeather, 1.0)],
            duration_ticks: 160,
            output_to_inventory: false,
        },
        BuildingType::CrystalRefinery => Recipe {
            inputs: vec![(RT::ManaCrystal, 2.0), (RT::Water, 1.0)],
            outputs: vec![(RT::RefinedCrystal, 1.0)],
            duration_ticks: 240,
            output_to_inventory: false,
        },
        BuildingType::AlchemistLab => Recipe {
            inputs: vec![(RT::Water, 2.0), (RT::Herbs, 3.0)],
            outputs: vec![(RT::PotionBase, 2.0)],
            duration_ticks: 200,
            output_to_inventory: false,
        },
        // T3 Synthesis
        BuildingType::RunicForge => Recipe {
            inputs: vec![(RT::SteelPlate, 2.0), (RT::ManaCrystal, 1.0), (RT::ObsidianShard, 1.0)],
            outputs: vec![(RT::RunicAlloy, 1.0)],
            duration_ticks: 300,
            output_to_inventory: false,
        },
        BuildingType::ArcaneDistillery => Recipe {
            inputs: vec![(RT::RefinedCrystal, 2.0), (RT::Venom, 1.0), (RT::PotionBase, 1.0)],
            outputs: vec![(RT::ArcaneEssence, 1.0)],
            duration_ticks: 280,
            output_to_inventory: false,
        },
        BuildingType::OpusForge => Recipe {
            inputs: vec![(RT::RunicAlloy, 2.0), (RT::ArcaneEssence, 1.0)],
            outputs: vec![(RT::OpusIngot, 1.0)],
            duration_ticks: 400,
            output_to_inventory: false,
        },
        // Mall — constructor builds iron miners (default)
        BuildingType::Constructor => Recipe {
            inputs: vec![(RT::IronBar, 3.0), (RT::Plank, 1.0)],
            outputs: vec![(RT::ItemIronMiner, 1.0)],
            duration_ticks: 300,
            output_to_inventory: true,
        },
        BuildingType::Toolmaker => Recipe {
            inputs: vec![(RT::SteelPlate, 2.0), (RT::ObsidianShard, 1.0)],
            outputs: vec![(RT::ItemObsidianDrill, 1.0)],
            duration_ticks: 400,
            output_to_inventory: true,
        },
        BuildingType::Assembler => Recipe {
            inputs: vec![(RT::SteelPlate, 3.0), (RT::TreatedLeather, 2.0), (RT::Plank, 4.0)],
            outputs: vec![(RT::ItemWarLodge, 1.0)],
            duration_ticks: 500,
            output_to_inventory: true,
        },
        // Combat
        BuildingType::ImpCamp => Recipe {
            inputs: vec![(RT::IronBar, 1.0), (RT::Herbs, 2.0)],
            outputs: vec![(RT::Hide, 3.0), (RT::BoneMeal, 1.0)],
            duration_ticks: 120,
            output_to_inventory: false,
        },
        BuildingType::BreedingPen => Recipe {
            inputs: vec![(RT::Water, 2.0), (RT::Herbs, 1.0)],
            outputs: vec![(RT::Hide, 1.0), (RT::Herbs, 2.0)],
            duration_ticks: 180,
            output_to_inventory: false,
        },
        BuildingType::WarLodge => Recipe {
            inputs: vec![(RT::SteelPlate, 1.0), (RT::TreatedLeather, 1.0), (RT::Herbs, 2.0)],
            outputs: vec![(RT::Venom, 2.0), (RT::Sinew, 1.0)],
            duration_ticks: 160,
            output_to_inventory: false,
        },
        // Energy — no recipe (energy generation is passive)
        BuildingType::WindTurbine
        | BuildingType::WaterWheel
        | BuildingType::LavaGenerator
        | BuildingType::ManaReactor => Recipe {
            inputs: vec![],
            outputs: vec![],
            duration_ticks: 1,
            output_to_inventory: false,
        },
        // Utility — trader has no production recipe
        BuildingType::Trader | BuildingType::SacrificeAltar | BuildingType::Watchtower => Recipe {
            inputs: vec![],
            outputs: vec![],
            duration_ticks: 1,
            output_to_inventory: false,
        },
        // Legacy
        BuildingType::Miner => Recipe {
            inputs: vec![],
            outputs: vec![(RT::IronOre, 1.0)],
            duration_ticks: 60,
            output_to_inventory: false,
        },
        BuildingType::Smelter => Recipe {
            inputs: vec![(RT::IronOre, 2.0)],
            outputs: vec![(RT::IronBar, 1.0)],
            duration_ticks: 120,
            output_to_inventory: false,
        },
        BuildingType::EnergySource => Recipe {
            inputs: vec![],
            outputs: vec![],
            duration_ticks: 1,
            output_to_inventory: false,
        },
    }
}
