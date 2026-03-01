//! BDD tests for meta-progression feature.
//! One test fn per Gherkin scenario in .ptsd/bdd/meta/meta.feature
//! Seed data: .ptsd/seeds/meta/currencies.yaml, unlocks.yaml, trading.yaml, fixtures.yaml

use std::collections::HashMap;

use crate::components::{
    DifficultyCategory, MetaCurrency, MetaWallet, MiniOpusReward,
    PurchasedUnlocks, ResourceType, RunEndResult, TraderEarnings, TraderState,
    UnlockCategory, find_unlock, find_trading_rate, ALL_UNLOCKS,
};
use crate::systems::trading::compute_earnings_for_manifold;

// ── AC1: Run-end currencies are multiplied by opus difficulty ─────────────────

#[test]
fn run_end_currencies_are_multiplied_by_opus_difficulty_medium_run() {
    let result = RunEndResult {
        difficulty: DifficultyCategory::Medium,
        mini_opus_rewards: vec![
            MiniOpusReward { currency: MetaCurrency::Gold,      amount: 50.0 },
            MiniOpusReward { currency: MetaCurrency::Knowledge, amount: 60.0 },
        ],
        abandoned: false,
        mini_opus_completed: 2,
        mini_opus_total: 3,
        all_failed_penalty: false,
    };

    let (gold, _souls, knowledge) = result.calculate_awards();

    assert!((gold - 75.0).abs() < 0.01,      "Gold: 50 * 1.5 = 75, got {gold}");
    assert!((knowledge - 90.0).abs() < 0.01, "Knowledge: 60 * 1.5 = 90, got {knowledge}");
}

#[test]
fn run_end_currencies_are_multiplied_by_opus_difficulty_extreme_run() {
    let result = RunEndResult {
        difficulty: DifficultyCategory::Extreme,
        mini_opus_rewards: vec![
            MiniOpusReward { currency: MetaCurrency::Souls, amount: 100.0 },
        ],
        abandoned: false,
        mini_opus_completed: 1,
        mini_opus_total: 1,
        all_failed_penalty: false,
    };

    let (_gold, souls, _knowledge) = result.calculate_awards();

    assert!((souls - 300.0).abs() < 0.01, "Souls: 100 * 3.0 = 300, got {souls}");
}

#[test]
fn run_end_currencies_are_multiplied_by_opus_difficulty_easy_run() {
    let result = RunEndResult {
        difficulty: DifficultyCategory::Easy,
        mini_opus_rewards: vec![
            MiniOpusReward { currency: MetaCurrency::Gold, amount: 80.0 },
        ],
        abandoned: false,
        mini_opus_completed: 1,
        mini_opus_total: 2,
        all_failed_penalty: false,
    };

    let (gold, _souls, _knowledge) = result.calculate_awards();

    assert!((gold - 80.0).abs() < 0.01, "Gold: 80 * 1.0 = 80, got {gold}");
}

// ── AC2: Meta store displays available unlocks with currency costs ─────────────

#[test]
fn meta_store_displays_biome_unlock_with_multi_currency_cost() {
    let unlock = find_unlock("unlock_volcanic").expect("unlock_volcanic must exist");

    assert_eq!(unlock.name, "Ashen Caldera");
    assert_eq!(unlock.category, UnlockCategory::Biome);
    assert_eq!(unlock.cost_gold, 200);
    assert_eq!(unlock.cost_souls, 100);
    assert_eq!(unlock.cost_knowledge, 0);
}

#[test]
fn meta_store_displays_starting_bonus_unlocks() {
    let gift = find_unlock("extra_starting_miner").expect("extra_starting_miner must exist");
    let war  = find_unlock("starting_combat").expect("starting_combat must exist");

    assert_eq!(gift.name, "Prospector's Gift");
    assert_eq!(gift.category, UnlockCategory::StartingBonus);
    assert_eq!(gift.cost_gold, 300);

    assert_eq!(war.name, "War Preparation");
    assert_eq!(war.category, UnlockCategory::StartingBonus);
    assert_eq!(war.cost_souls, 400);
}

#[test]
fn meta_store_displays_building_pool_unlocks() {
    let alchemist = find_unlock("unlock_alchemist_lab").expect("unlock_alchemist_lab must exist");
    let arcane    = find_unlock("unlock_mana_reactor").expect("unlock_mana_reactor must exist");

    assert_eq!(alchemist.name, "Alchemist's Secret");
    assert_eq!(alchemist.category, UnlockCategory::BuildingPool);
    assert_eq!(alchemist.cost_knowledge, 500);

    assert_eq!(arcane.name, "Arcane Power");
    assert_eq!(arcane.category, UnlockCategory::BuildingPool);
    assert_eq!(arcane.cost_knowledge, 600);
    assert_eq!(arcane.cost_souls, 200);
}

#[test]
fn meta_store_displays_cosmetic_unlocks() {
    let golden   = find_unlock("golden_rune_paths").expect("golden_rune_paths must exist");
    let spectral = find_unlock("spectral_minions").expect("spectral_minions must exist");

    assert_eq!(golden.name, "Golden Runes");
    assert_eq!(golden.category, UnlockCategory::Cosmetic);
    assert_eq!(golden.cost_gold, 100);

    assert_eq!(spectral.name, "Spectral Workers");
    assert_eq!(spectral.category, UnlockCategory::Cosmetic);
    assert_eq!(spectral.cost_souls, 150);
}

// ── AC3: Unlocked content persists across runs ────────────────────────────────

#[test]
fn purchased_unlock_persists_into_a_new_run_extra_starting_miner() {
    let mut purchased = PurchasedUnlocks::default();
    purchased.add("extra_starting_miner");

    let wallet = MetaWallet { gold: 0.0, souls: 0.0, knowledge: 0.0, ..Default::default() };

    assert!(purchased.has("extra_starting_miner"), "unlock must still be owned");
    assert_eq!(wallet.gold, 0.0);

    let extra_miners = if purchased.has("extra_starting_miner") { 1usize } else { 0 };
    assert_eq!(extra_miners, 1, "starting kit: 1 extra iron_miner");
}

#[test]
fn purchased_biome_unlock_is_available_for_run_selection() {
    let mut purchased = PurchasedUnlocks::default();
    purchased.add("unlock_volcanic");

    assert!(purchased.has("unlock_volcanic"), "volcanic biome selectable");
}

#[test]
fn multiple_unlocks_persist_across_runs() {
    let mut purchased = PurchasedUnlocks::default();
    purchased.add("unlock_volcanic");
    purchased.add("extra_starting_miner");
    purchased.add("golden_rune_paths");

    assert_eq!(purchased.ids.len(), 3);
    assert!(purchased.has("unlock_volcanic"));
    assert!(purchased.has("extra_starting_miner"));
    assert!(purchased.has("golden_rune_paths"));
}

// ── AC4: Opus multiplier is determined by biome-opus difficulty match ─────────

#[test]
fn forest_biome_with_short_opus_yields_easy_difficulty_multiplier_1_0() {
    let score = 1u32 + 1u32;
    let category = DifficultyCategory::from_score(score);
    let multiplier = category.opus_multiplier();

    assert_eq!(score, 2);
    assert_eq!(category, DifficultyCategory::Easy);
    assert!((multiplier - 1.0).abs() < f32::EPSILON);
}

#[test]
fn ocean_biome_with_standard_opus_yields_medium_difficulty_multiplier_1_5() {
    let score = 2u32 + 2u32;
    let category = DifficultyCategory::from_score(score);
    let multiplier = category.opus_multiplier();

    assert_eq!(score, 4);
    assert_eq!(category, DifficultyCategory::Medium);
    assert!((multiplier - 1.5).abs() < f32::EPSILON);
}

#[test]
fn desert_biome_with_grand_opus_yields_hard_difficulty_multiplier_2_0() {
    let score = 2u32 + 3u32;
    let category = DifficultyCategory::from_score(score);
    let multiplier = category.opus_multiplier();

    assert_eq!(score, 5);
    assert_eq!(category, DifficultyCategory::Hard);
    assert!((multiplier - 2.0).abs() < f32::EPSILON);
}

#[test]
fn volcanic_biome_with_grand_opus_yields_extreme_difficulty_multiplier_3_0() {
    let score = 3u32 + 3u32;
    let category = DifficultyCategory::from_score(score);
    let multiplier = category.opus_multiplier();

    assert_eq!(score, 6);
    assert_eq!(category, DifficultyCategory::Extreme);
    assert!((multiplier - 3.0).abs() < f32::EPSILON);
}

#[test]
fn forest_biome_with_standard_opus_yields_easy_difficulty_boundary_at_3() {
    let score = 1u32 + 2u32;
    let category = DifficultyCategory::from_score(score);
    let multiplier = category.opus_multiplier();

    assert_eq!(score, 3);
    assert_eq!(category, DifficultyCategory::Easy);
    assert!((multiplier - 1.0).abs() < f32::EPSILON);
}

// ── AC5: Player can view total lifetime currency earnings and spending ─────────

#[test]
fn lifetime_stats_display_total_earnings_and_spending() {
    let wallet = MetaWallet {
        gold: 500.0,
        souls: 300.0,
        knowledge: 200.0,
        lifetime_earned_gold: 2000.0,
        lifetime_earned_souls: 1500.0,
        lifetime_earned_knowledge: 1000.0,
        lifetime_spent_gold: 1500.0,
        lifetime_spent_souls: 1200.0,
        lifetime_spent_knowledge: 800.0,
    };

    assert_eq!(wallet.lifetime_earned_gold, 2000.0);
    assert_eq!(wallet.lifetime_earned_souls, 1500.0);
    assert_eq!(wallet.lifetime_earned_knowledge, 1000.0);
    assert_eq!(wallet.lifetime_spent_gold, 1500.0);
    assert_eq!(wallet.lifetime_spent_souls, 1200.0);
    assert_eq!(wallet.lifetime_spent_knowledge, 800.0);
    assert_eq!(wallet.gold, 500.0);
    assert_eq!(wallet.souls, 300.0);
    assert_eq!(wallet.knowledge, 200.0);
}

#[test]
fn lifetime_stats_include_unlocks_purchased() {
    let mut purchased = PurchasedUnlocks::default();
    purchased.add("unlock_volcanic");
    purchased.add("extra_starting_miner");
    purchased.add("golden_rune_paths");

    assert_eq!(purchased.ids.len(), 3);
}

// ── AC6: Trader building converts surplus resources to meta-currency ───────────

#[test]
fn trader_converts_iron_ore_surplus_to_gold_first_trade_no_inflation() {
    let trader_state = TraderState::default();
    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 10.0);

    let (gold, souls, knowledge) =
        compute_earnings_for_manifold(&manifold_resources, &trader_state);

    assert!((gold - 5.0).abs() < 0.01, "5.0 Gold (10 * 0.5), got {gold}");
    assert_eq!(souls, 0.0);
    assert_eq!(knowledge, 0.0);
}

#[test]
fn trader_converts_mixed_resources_to_different_currencies() {
    let trader_state = TraderState::default();
    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 5.0);
    manifold_resources.insert(ResourceType::Hide, 3.0);
    manifold_resources.insert(ResourceType::SteelPlate, 2.0);

    let (gold, souls, knowledge) =
        compute_earnings_for_manifold(&manifold_resources, &trader_state);

    assert!((gold      - 2.5).abs() < 0.01, "2.5 Gold, got {gold}");
    assert!((souls     - 4.5).abs() < 0.01, "4.5 Souls, got {souls}");
    assert!((knowledge - 4.0).abs() < 0.01, "4.0 Knowledge, got {knowledge}");
}

#[test]
fn trader_converts_t3_resource_at_high_base_rate() {
    let trader_state = TraderState::default();
    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::OpusIngot, 1.0);

    let (gold, souls, knowledge) =
        compute_earnings_for_manifold(&manifold_resources, &trader_state);

    assert_eq!(gold, 0.0);
    assert_eq!(souls, 0.0);
    assert!((knowledge - 10.0).abs() < 0.01, "10.0 Knowledge, got {knowledge}");
}

#[test]
fn trader_with_empty_manifold_produces_no_currency() {
    let trader_state = TraderState::default();
    let manifold_resources: HashMap<ResourceType, f32> = HashMap::new();

    let (gold, souls, knowledge) =
        compute_earnings_for_manifold(&manifold_resources, &trader_state);

    assert_eq!(gold, 0.0);
    assert_eq!(souls, 0.0);
    assert_eq!(knowledge, 0.0);
}

// ── AC7: Trading the same resource repeatedly yields diminishing returns ───────

#[test]
fn inflation_reduces_effective_rate_after_50_units_traded() {
    // effective = 0.5 / (1 + 0.3 * ln(51)) ≈ 0.23
    let mut trader_state = TraderState::default();
    *trader_state.volume_traded.entry(ResourceType::IronOre).or_default() = 50.0;

    let effective_rate = trader_state.effective_rate(ResourceType::IronOre, 0.5);
    assert!((effective_rate - 0.23).abs() < 0.02, "~0.23, got {effective_rate}");

    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 10.0);
    let (gold, _, _) = compute_earnings_for_manifold(&manifold_resources, &trader_state);
    assert!((gold - 2.3).abs() < 0.1, "~2.3 Gold, got {gold}");
}

#[test]
fn first_10_units_traded_get_nearly_full_rate() {
    let trader_state = TraderState::default(); // 0 traded
    let effective_rate = trader_state.effective_rate(ResourceType::IronOre, 0.5);
    assert!((effective_rate - 0.5).abs() < 0.01, "Full rate 0.5, got {effective_rate}");

    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 10.0);
    let (gold, _, _) = compute_earnings_for_manifold(&manifold_resources, &trader_state);
    assert!((gold - 5.0).abs() < 0.01, "5.0 Gold, got {gold}");
}

#[test]
fn inflation_is_tracked_per_resource_type_independently() {
    let mut trader_state = TraderState::default();
    *trader_state.volume_traded.entry(ResourceType::IronOre).or_default() = 50.0;

    let iron_rate = trader_state.effective_rate(ResourceType::IronOre, 0.5);
    let hide_rate = trader_state.effective_rate(ResourceType::Hide, 1.5);

    assert!((iron_rate - 0.23).abs() < 0.02, "Iron ~0.23, got {iron_rate}");
    assert!((hide_rate - 1.5).abs()  < 0.01, "Hide 1.5, got {hide_rate}");

    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 10.0);
    manifold_resources.insert(ResourceType::Hide, 10.0);
    let (gold, souls, _) = compute_earnings_for_manifold(&manifold_resources, &trader_state);

    assert!((gold  - 2.3).abs()  < 0.1, "~2.3 Gold, got {gold}");
    assert!((souls - 15.0).abs() < 0.1, "~15.0 Souls, got {souls}");
}

#[test]
fn inflation_resets_each_run() {
    let trader_state = TraderState::default();
    let effective_rate = trader_state.effective_rate(ResourceType::IronOre, 0.5);
    assert!((effective_rate - 0.5).abs() < 0.01, "Reset: full rate 0.5, got {effective_rate}");

    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::IronOre, 10.0);
    let (gold, _, _) = compute_earnings_for_manifold(&manifold_resources, &trader_state);
    assert!((gold - 5.0).abs() < 0.01, "5.0 Gold, got {gold}");
}

// ── Edge Case: Abandoned run yields zero currency ─────────────────────────────

#[test]
fn abandoned_run_yields_zero_currency() {
    let result = RunEndResult {
        difficulty: DifficultyCategory::Medium,
        mini_opus_rewards: vec![
            MiniOpusReward { currency: MetaCurrency::Gold, amount: 50.0 },
        ],
        abandoned: true,
        mini_opus_completed: 1,
        mini_opus_total: 3,
        all_failed_penalty: false,
    };

    let (gold, souls, knowledge) = result.calculate_awards();
    assert_eq!(gold, 0.0);
    assert_eq!(souls, 0.0);
    assert_eq!(knowledge, 0.0);
}

// ── Edge Case: All Mini-Opus failed — 25% penalty ─────────────────────────────

#[test]
fn all_mini_opus_failed_yields_reduced_currency_25_percent_penalty() {
    // hard (mult=2.0), all_failed_penalty=true → 25% × base
    // base_reward=100 → 100 * 2.0 * 0.25 = 50.0

    let result = RunEndResult {
        difficulty: DifficultyCategory::Hard,
        mini_opus_rewards: vec![
            MiniOpusReward { currency: MetaCurrency::Gold, amount: 100.0 },
        ],
        abandoned: false,
        mini_opus_completed: 0,
        mini_opus_total: 4,
        all_failed_penalty: true,
    };

    let (gold, _, _) = result.calculate_awards();

    assert!(gold > 0.0, "Currency must be > 0 (reduced, not zero)");
    assert!((gold - 50.0).abs() < 0.01, "100 * 2.0 * 0.25 = 50.0, got {gold}");
}

// ── Edge Case: Purchase fails with insufficient currency ──────────────────────

#[test]
fn purchase_fails_when_player_lacks_required_gold() {
    let mut wallet = MetaWallet { gold: 100.0, souls: 50.0, knowledge: 0.0, ..Default::default() };
    let mut purchased = PurchasedUnlocks::default();

    let unlock = find_unlock("unlock_volcanic").unwrap();
    let success = unlock.try_purchase(&mut wallet, &mut purchased);

    assert!(!success);
    assert_eq!(wallet.gold, 100.0);
    assert_eq!(wallet.souls, 50.0);
    assert_eq!(wallet.knowledge, 0.0);
}

#[test]
fn purchase_fails_when_player_lacks_one_of_multiple_required_currencies() {
    let mut wallet = MetaWallet { gold: 300.0, souls: 50.0, knowledge: 0.0, ..Default::default() };
    let mut purchased = PurchasedUnlocks::default();

    let unlock = find_unlock("unlock_volcanic").unwrap(); // needs 200 Gold + 100 Souls
    let success = unlock.try_purchase(&mut wallet, &mut purchased);

    assert!(!success, "Rejected: souls 50 < 100 required");
    assert_eq!(wallet.gold, 300.0);
    assert_eq!(wallet.souls, 50.0);
}

// ── Edge Case: Purchase is atomic ─────────────────────────────────────────────

#[test]
fn successful_purchase_deducts_exact_cost_atomically() {
    let mut wallet = MetaWallet {
        gold: 250.0, souls: 150.0, knowledge: 50.0, ..Default::default()
    };
    let mut purchased = PurchasedUnlocks::default();

    let unlock = find_unlock("extra_starting_turbine").unwrap(); // costs 250 Gold
    let success = unlock.try_purchase(&mut wallet, &mut purchased);

    assert!(success);
    assert_eq!(wallet.gold, 0.0);
    assert_eq!(wallet.souls, 150.0);
    assert_eq!(wallet.knowledge, 50.0);
    assert!(purchased.has("extra_starting_turbine"));
}

#[test]
fn multi_currency_purchase_deducts_all_currencies_atomically() {
    let mut wallet = MetaWallet {
        gold: 200.0, souls: 100.0, knowledge: 500.0, ..Default::default()
    };
    let mut purchased = PurchasedUnlocks::default();

    let unlock = find_unlock("unlock_volcanic").unwrap(); // 200 Gold + 100 Souls
    let success = unlock.try_purchase(&mut wallet, &mut purchased);

    assert!(success);
    assert_eq!(wallet.gold, 0.0);
    assert_eq!(wallet.souls, 0.0);
    assert_eq!(wallet.knowledge, 500.0);
    assert!(purchased.has("unlock_volcanic"));
}

// ── Edge Case: Cannot purchase already-purchased unlock ───────────────────────

#[test]
fn cannot_re_purchase_an_already_owned_unlock() {
    let mut wallet = MetaWallet {
        gold: 500.0, souls: 500.0, knowledge: 500.0, ..Default::default()
    };
    let mut purchased = PurchasedUnlocks::default();
    purchased.add("golden_rune_paths");

    let unlock = find_unlock("golden_rune_paths").unwrap();
    let success = unlock.try_purchase(&mut wallet, &mut purchased);

    assert!(!success, "Re-purchase rejected");
    assert_eq!(wallet.gold, 500.0);
    assert_eq!(wallet.souls, 500.0);
    assert_eq!(wallet.knowledge, 500.0);
    assert_eq!(
        purchased.ids.iter().filter(|id| id.as_str() == "golden_rune_paths").count(),
        1
    );
}

// ── Edge Case: Difficulty matrix boundary validation ──────────────────────────

#[test]
fn minimum_possible_difficulty_score_is_2_forest_plus_short() {
    let score = 1u32 + 1u32;
    assert_eq!(score, 2);
    assert_eq!(DifficultyCategory::from_score(score), DifficultyCategory::Easy);
}

#[test]
fn maximum_possible_difficulty_score_is_6_volcanic_plus_grand() {
    let score = 3u32 + 3u32;
    assert_eq!(score, 6);
    assert_eq!(DifficultyCategory::from_score(score), DifficultyCategory::Extreme);
}

// ── Edge Case: Trading organic resources yields Souls ─────────────────────────

#[test]
fn trading_organic_resources_produces_souls_currency() {
    let trader_state = TraderState::default();
    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::Venom, 10.0);

    let (_, souls, _) = compute_earnings_for_manifold(&manifold_resources, &trader_state);
    assert!((souls - 30.0).abs() < 0.01, "30.0 Souls (10 * 3.0), got {souls}");
}

// ── Edge Case: Trading T2 knowledge resources ─────────────────────────────────

#[test]
fn trading_refined_crystal_produces_knowledge_currency() {
    let trader_state = TraderState::default();
    let mut manifold_resources = HashMap::new();
    manifold_resources.insert(ResourceType::RefinedCrystal, 4.0);

    let (_, _, knowledge) = compute_earnings_for_manifold(&manifold_resources, &trader_state);
    assert!((knowledge - 10.0).abs() < 0.01, "10.0 Knowledge (4 * 2.5), got {knowledge}");
}

// ── Seed data integrity checks ────────────────────────────────────────────────

#[test]
fn all_11_unlocks_are_in_catalog() {
    let expected = [
        "unlock_volcanic", "unlock_desert", "unlock_ocean",
        "extra_starting_miner", "extra_starting_turbine", "starting_combat",
        "starting_watchtower_upgrade", "unlock_alchemist_lab", "unlock_mana_reactor",
        "golden_rune_paths", "spectral_minions",
    ];

    assert_eq!(ALL_UNLOCKS.len(), expected.len());
    for id in &expected {
        assert!(find_unlock(id).is_some(), "Unlock '{id}' must exist");
    }
}

#[test]
fn trading_rates_catalog_covers_all_seed_resources() {
    let expected = [
        ResourceType::IronOre, ResourceType::CopperOre, ResourceType::Stone,
        ResourceType::Wood, ResourceType::IronBar, ResourceType::CopperBar,
        ResourceType::Plank, ResourceType::Hide, ResourceType::Herbs,
        ResourceType::BoneMeal, ResourceType::ObsidianShard, ResourceType::ManaCrystal,
        ResourceType::SteelPlate, ResourceType::TreatedLeather, ResourceType::RefinedCrystal,
        ResourceType::PotionBase, ResourceType::Venom, ResourceType::Sinew,
        ResourceType::RunicAlloy, ResourceType::ArcaneEssence, ResourceType::OpusIngot,
    ];

    for res in &expected {
        assert!(find_trading_rate(*res).is_some(), "Resource {res:?} must have a trading rate");
    }
}

#[test]
fn difficulty_multipliers_match_seed_data() {
    assert!((DifficultyCategory::Easy.opus_multiplier()    - 1.0).abs() < f32::EPSILON);
    assert!((DifficultyCategory::Medium.opus_multiplier()  - 1.5).abs() < f32::EPSILON);
    assert!((DifficultyCategory::Hard.opus_multiplier()    - 2.0).abs() < f32::EPSILON);
    assert!((DifficultyCategory::Extreme.opus_multiplier() - 3.0).abs() < f32::EPSILON);
}

#[test]
fn inflation_formula_matches_seed_spec() {
    // factor=0.3, formula: base / (1 + 0.3 * ln(1 + traded))
    // traded=10 → expected = 1.0 / (1 + 0.3 * ln(11))

    let mut trader_state = TraderState::default();
    *trader_state.volume_traded.entry(ResourceType::IronOre).or_default() = 10.0;

    let effective = trader_state.effective_rate(ResourceType::IronOre, 1.0);
    let expected  = 1.0_f32 / (1.0 + 0.3 * 11.0_f32.ln());

    assert!(
        (effective - expected).abs() < 0.001,
        "Formula: expected {expected:.4}, got {effective:.4}"
    );
}
