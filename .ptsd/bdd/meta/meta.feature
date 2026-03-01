@feature:meta
Feature: Meta-Progression
  Between-run progression: currencies earned from runs, permanent unlocks, and difficulty scaling.
  Players earn 3 meta-currencies (Gold, Souls, Knowledge) from Mini-Opus completions,
  multiplied by the main Opus difficulty. Currencies buy permanent unlocks that expand
  options for future runs.

  # ──────────────────────────────────────────────
  # AC1: Run-end screen shows earned currencies with Opus multiplier applied
  # ──────────────────────────────────────────────

  Scenario: Run-end currencies are multiplied by opus difficulty — medium run
    Given a completed run with difficulty "medium"
    Given the opus multiplier for "medium" is 1.5
    Given the player completed 2 of 3 mini-opus challenges
    Given mini-opus rewards are 50 Gold and 60 Knowledge
    When the run ends
    Then the run-end screen shows Gold earned as 75
    Then the run-end screen shows Knowledge earned as 90

  Scenario: Run-end currencies are multiplied by opus difficulty — extreme run
    Given a completed run with difficulty "extreme"
    Given the opus multiplier for "extreme" is 3.0
    Given the player completed 1 of 1 mini-opus challenges
    Given mini-opus rewards are 100 Souls
    When the run ends
    Then the run-end screen shows Souls earned as 300

  Scenario: Run-end currencies are multiplied by opus difficulty — easy run
    Given a completed run with difficulty "easy"
    Given the opus multiplier for "easy" is 1.0
    Given the player completed 1 of 2 mini-opus challenges
    Given mini-opus rewards are 80 Gold
    When the run ends
    Then the run-end screen shows Gold earned as 80

  # ──────────────────────────────────────────────
  # AC2: Meta store displays available unlocks with currency costs
  # ──────────────────────────────────────────────

  Scenario: Meta store displays biome unlock with multi-currency cost
    Given the meta store is open
    Given the unlock "Ashen Caldera" exists with cost 200 Gold and 100 Souls
    When the player views available unlocks
    Then "Ashen Caldera" is listed under "biome" category
    Then "Ashen Caldera" shows cost 200 Gold and 100 Souls

  Scenario: Meta store displays starting bonus unlocks
    Given the meta store is open
    Given the unlock "Prospector's Gift" exists with cost 300 Gold
    Given the unlock "War Preparation" exists with cost 400 Souls
    When the player views available unlocks
    Then "Prospector's Gift" is listed under "starting_bonus" category
    Then "War Preparation" is listed under "starting_bonus" category

  Scenario: Meta store displays building pool unlocks
    Given the meta store is open
    Given the unlock "Alchemist's Secret" exists with cost 500 Knowledge
    Given the unlock "Arcane Power" exists with cost 600 Knowledge and 200 Souls
    When the player views available unlocks
    Then "Alchemist's Secret" is listed under "building_pool" category
    Then "Arcane Power" is listed under "building_pool" category

  Scenario: Meta store displays cosmetic unlocks
    Given the meta store is open
    Given the unlock "Golden Runes" exists with cost 100 Gold
    Given the unlock "Spectral Workers" exists with cost 150 Souls
    When the player views available unlocks
    Then "Golden Runes" is listed under "cosmetic" category
    Then "Spectral Workers" is listed under "cosmetic" category

  # ──────────────────────────────────────────────
  # AC3: Unlocked content persists across runs
  # ──────────────────────────────────────────────

  Scenario: Purchased unlock persists into a new run — extra starting miner
    Given the player has purchased "extra_starting_miner"
    Given the player has 0 Gold, 0 Souls, 0 Knowledge
    When the player starts a new run in the "forest" biome
    Then the starting kit includes 1 extra iron_miner
    Then the "extra_starting_miner" unlock is still marked as purchased

  Scenario: Purchased biome unlock is available for run selection
    Given the player has purchased "unlock_volcanic"
    When the player views the biome selection screen
    Then the "volcanic" biome is selectable

  Scenario: Multiple unlocks persist across runs
    Given the player has purchased "unlock_volcanic"
    Given the player has purchased "extra_starting_miner"
    Given the player has purchased "golden_rune_paths"
    When the player starts a new run
    Then all 3 unlocks are active in the new run

  # ──────────────────────────────────────────────
  # AC4: Opus multiplier is determined by biome-opus difficulty match
  # ──────────────────────────────────────────────

  Scenario: Forest biome with short opus yields easy difficulty — multiplier 1.0
    Given the biome is "forest" with biome_difficulty 1
    Given the opus template is "short" with opus_difficulty 1
    When the difficulty is calculated
    Then the combined difficulty score is 2
    Then the difficulty category is "easy"
    Then the opus multiplier is 1.0

  Scenario: Ocean biome with standard opus yields medium difficulty — multiplier 1.5
    Given the biome is "ocean" with biome_difficulty 2
    Given the opus template is "standard" with opus_difficulty 2
    When the difficulty is calculated
    Then the combined difficulty score is 4
    Then the difficulty category is "medium"
    Then the opus multiplier is 1.5

  Scenario: Desert biome with grand opus yields hard difficulty — multiplier 2.0
    Given the biome is "desert" with biome_difficulty 2
    Given the opus template is "grand" with opus_difficulty 3
    When the difficulty is calculated
    Then the combined difficulty score is 5
    Then the difficulty category is "hard"
    Then the opus multiplier is 2.0

  Scenario: Volcanic biome with grand opus yields extreme difficulty — multiplier 3.0
    Given the biome is "volcanic" with biome_difficulty 3
    Given the opus template is "grand" with opus_difficulty 3
    When the difficulty is calculated
    Then the combined difficulty score is 6
    Then the difficulty category is "extreme"
    Then the opus multiplier is 3.0

  Scenario: Forest biome with standard opus yields easy difficulty — boundary at 3
    Given the biome is "forest" with biome_difficulty 1
    Given the opus template is "standard" with opus_difficulty 2
    When the difficulty is calculated
    Then the combined difficulty score is 3
    Then the difficulty category is "easy"
    Then the opus multiplier is 1.0

  # ──────────────────────────────────────────────
  # AC5: Player can view total lifetime currency earnings and spending
  # ──────────────────────────────────────────────

  Scenario: Lifetime stats display total earnings and spending
    Given the player has lifetime_earned of 2000 Gold, 1500 Souls, 1000 Knowledge
    Given the player has lifetime_spent of 1500 Gold, 1200 Souls, 800 Knowledge
    Given the player has current balance of 500 Gold, 300 Souls, 200 Knowledge
    When the player views lifetime statistics
    Then lifetime earnings show 2000 Gold, 1500 Souls, 1000 Knowledge
    Then lifetime spending shows 1500 Gold, 1200 Souls, 800 Knowledge
    Then current balance shows 500 Gold, 300 Souls, 200 Knowledge

  Scenario: Lifetime stats include unlocks purchased
    Given the player has purchased "unlock_volcanic", "extra_starting_miner", "golden_rune_paths"
    When the player views lifetime statistics
    Then the unlocks list shows 3 purchased unlocks

  # ──────────────────────────────────────────────
  # AC6: Trader building converts surplus resources to meta-currency during a run
  # ──────────────────────────────────────────────

  Scenario: Trader converts iron_ore surplus to Gold — first trade no inflation
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 10 iron_ore
    Given the base rate for iron_ore is 0.5 Gold per unit
    Given the trader has 0 iron_ore previously traded
    When the trading system processes one tick
    Then 5.0 Gold is added to the player meta-currency
    Then the trader manifold iron_ore is 0

  Scenario: Trader converts mixed resources to different currencies
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 5 iron_ore, 3 hide, 2 steel_plate
    Given the base rate for iron_ore is 0.5 Gold per unit
    Given the base rate for hide is 1.5 Souls per unit
    Given the base rate for steel_plate is 2.0 Knowledge per unit
    Given the trader has no previous trades
    When the trading system processes one tick
    Then 2.5 Gold is added to the player meta-currency
    Then 4.5 Souls is added to the player meta-currency
    Then 4.0 Knowledge is added to the player meta-currency

  Scenario: Trader converts T3 resource at high base rate
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 1 opus_ingot
    Given the base rate for opus_ingot is 10.0 Knowledge per unit
    Given the trader has no previous trades
    When the trading system processes one tick
    Then 10.0 Knowledge is added to the player meta-currency

  Scenario: Trader with empty manifold produces no currency
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold is empty
    When the trading system processes one tick
    Then 0 Gold is added to the player meta-currency
    Then 0 Souls is added to the player meta-currency
    Then 0 Knowledge is added to the player meta-currency

  # ──────────────────────────────────────────────
  # AC7: Trading the same resource repeatedly yields diminishing returns (inflation)
  # ──────────────────────────────────────────────

  Scenario: Inflation reduces effective rate after 50 units traded
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 10 iron_ore
    Given the base rate for iron_ore is 0.5 Gold per unit
    Given the trader has already traded 50 iron_ore
    Given the inflation formula is rate / (1 + 0.3 * ln(1 + total_traded))
    When the trading system processes one tick
    Then the effective rate per unit is approximately 0.23 Gold
    Then approximately 2.3 Gold is added to the player meta-currency

  Scenario: First 10 units traded get nearly full rate
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 10 iron_ore
    Given the base rate for iron_ore is 0.5 Gold per unit
    Given the trader has already traded 0 iron_ore
    When the trading system processes one tick
    Then the effective rate per unit is approximately 0.5 Gold
    Then approximately 5.0 Gold is added to the player meta-currency

  Scenario: Inflation is tracked per resource type independently
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 10 iron_ore and 10 hide
    Given the trader has already traded 50 iron_ore and 0 hide
    Given the base rate for iron_ore is 0.5 Gold per unit
    Given the base rate for hide is 1.5 Souls per unit
    When the trading system processes one tick
    Then the effective iron_ore rate is approximately 0.23 Gold per unit
    Then the effective hide rate is approximately 1.5 Souls per unit

  Scenario: Inflation resets each run
    Given a new run has started
    Given the trader has 0 units traded for all resources
    Given the trader group manifold contains 10 iron_ore
    Given the base rate for iron_ore is 0.5 Gold per unit
    When the trading system processes one tick
    Then the effective rate per unit is 0.5 Gold
    Then 5.0 Gold is added to the player meta-currency

  # ──────────────────────────────────────────────
  # Edge Case: Run abandoned before any Mini-Opus — 0 currencies earned
  # ──────────────────────────────────────────────

  Scenario: Abandoned run yields zero currency
    Given a run in progress with difficulty "medium"
    Given the player completed 1 of 3 mini-opus challenges
    Given opus completion is at 40%
    When the player abandons the run
    Then 0 Gold is earned
    Then 0 Souls is earned
    Then 0 Knowledge is earned

  # ──────────────────────────────────────────────
  # Edge Case: All Mini-Opus failed — reduced but not zero currency
  # ──────────────────────────────────────────────

  Scenario: All mini-opus failed yields reduced currency — 25% penalty
    Given a completed run with difficulty "hard"
    Given the opus multiplier for "hard" is 2.0
    Given the player completed 0 of 4 mini-opus challenges
    Given opus completion is at 80%
    Given the failed_run_penalty multiplier is 0.25
    When the run ends
    Then currencies are awarded at 25% of the base amount
    Then the currency amount is greater than 0

  # ──────────────────────────────────────────────
  # Edge Case: Purchase fails with insufficient currency
  # ──────────────────────────────────────────────

  Scenario: Purchase fails when player lacks required Gold
    Given the player has 100 Gold, 50 Souls, 0 Knowledge
    Given the unlock "unlock_volcanic" costs 200 Gold and 100 Souls
    When the player attempts to purchase "unlock_volcanic"
    Then the purchase is rejected
    Then the player still has 100 Gold, 50 Souls, 0 Knowledge

  Scenario: Purchase fails when player lacks one of multiple required currencies
    Given the player has 300 Gold, 50 Souls, 0 Knowledge
    Given the unlock "unlock_volcanic" costs 200 Gold and 100 Souls
    When the player attempts to purchase "unlock_volcanic"
    Then the purchase is rejected
    Then the player still has 300 Gold, 50 Souls, 0 Knowledge

  # ──────────────────────────────────────────────
  # Edge Case: Purchase is atomic — no partial spending
  # ──────────────────────────────────────────────

  Scenario: Successful purchase deducts exact cost atomically
    Given the player has 250 Gold, 150 Souls, 50 Knowledge
    Given the unlock "extra_starting_turbine" costs 250 Gold
    When the player attempts to purchase "extra_starting_turbine"
    Then the purchase succeeds
    Then the player has 0 Gold, 150 Souls, 50 Knowledge
    Then "extra_starting_turbine" is marked as purchased

  Scenario: Multi-currency purchase deducts all currencies atomically
    Given the player has 200 Gold, 100 Souls, 500 Knowledge
    Given the unlock "unlock_volcanic" costs 200 Gold and 100 Souls
    When the player attempts to purchase "unlock_volcanic"
    Then the purchase succeeds
    Then the player has 0 Gold, 0 Souls, 500 Knowledge
    Then "unlock_volcanic" is marked as purchased

  # ──────────────────────────────────────────────
  # Edge Case: Cannot purchase already-purchased unlock
  # ──────────────────────────────────────────────

  Scenario: Cannot re-purchase an already owned unlock
    Given the player has 500 Gold, 500 Souls, 500 Knowledge
    Given the player has already purchased "golden_rune_paths"
    When the player attempts to purchase "golden_rune_paths"
    Then the purchase is rejected
    Then the player still has 500 Gold, 500 Souls, 500 Knowledge

  # ──────────────────────────────────────────────
  # Edge Case: Difficulty matrix boundary validation
  # ──────────────────────────────────────────────

  Scenario: Minimum possible difficulty score is 2 — forest + short
    Given the biome is "forest" with biome_difficulty 1
    Given the opus template is "short" with opus_difficulty 1
    When the difficulty is calculated
    Then the combined difficulty score is 2
    Then the difficulty category is "easy"

  Scenario: Maximum possible difficulty score is 6 — volcanic + grand
    Given the biome is "volcanic" with biome_difficulty 3
    Given the opus template is "grand" with opus_difficulty 3
    When the difficulty is calculated
    Then the combined difficulty score is 6
    Then the difficulty category is "extreme"

  # ──────────────────────────────────────────────
  # Edge Case: Trading organic resources yields Souls
  # ──────────────────────────────────────────────

  Scenario: Trading organic resources produces Souls currency
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 10 venom
    Given the base rate for venom is 3.0 Souls per unit
    Given the trader has no previous trades
    When the trading system processes one tick
    Then 30.0 Souls is added to the player meta-currency

  # ──────────────────────────────────────────────
  # Edge Case: Trading T2 knowledge resources
  # ──────────────────────────────────────────────

  Scenario: Trading refined_crystal produces Knowledge currency
    Given a 10x10 grid
    Given a trader building at position [5, 5]
    Given a wind_turbine at position [5, 7]
    Given the trader group manifold contains 4 refined_crystal
    Given the base rate for refined_crystal is 2.5 Knowledge per unit
    Given the trader has no previous trades
    When the trading system processes one tick
    Then 10.0 Knowledge is added to the player meta-currency
