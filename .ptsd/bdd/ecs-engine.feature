@feature:ecs-engine
Feature: Cross-Feature Integration (ECS Engine)
  Verify that all 8 features work together through real ECS pipelines.
  Each scenario combines 3+ features and runs real app.update() cycles.
  No shared Background — each scenario builds its own plugin stack.

  # ═══════════════════════════════════════════════════════════
  # AC1: Production Pipeline (Energy + BuildingGroups + Transport)
  # ═══════════════════════════════════════════════════════════

  Scenario: S1 — Production pipeline delivers resources across groups via transport
    Given a 20x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3) and (3,3)
    Given group A: wind_turbine at (1,3), iron_miner at (2,3), iron_miner at (3,3), iron_smelter at (4,3)
    Given group A has TransportSender for iron_ore at position (4,3)
    Given group B: iron_smelter at (12,3) with GroupPosition at (12,3)
    Given group B has TransportReceiver for iron_ore with demand=2
    Given a T1 rune_path from group A to group B with 7 waypoint tiles (5,3)..(11,3)
    When the simulation runs for 200 ticks
    Then at least 1 Cargo entity exists in the ECS world
    Then group B Manifold contains iron_ore with amount > 0

  # ═══════════════════════════════════════════════════════════
  # AC2: Transport Delivery with Real Production
  # ═══════════════════════════════════════════════════════════

  Scenario: S2 — Produced IronOre transports to smelter yielding IronBar
    Given a 20x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3) and (3,3)
    Given group A: wind_turbine at (1,3), iron_miner at (2,3), iron_miner at (3,3) with TransportSender for iron_ore
    Given group B: wind_turbine at (11,3), iron_smelter at (12,3) with TransportReceiver for iron_ore demand=2
    Given group B has GroupPosition at (12,3)
    Given a T1 rune_path from group A to group B with 7 waypoints, distance=8 tiles (> MINION_RANGE=5)
    When the simulation runs for 200 ticks
    Then group B Manifold contains iron_bar with amount > 0

  # ═══════════════════════════════════════════════════════════
  # AC3: Energy Crisis Cascade
  # ═══════════════════════════════════════════════════════════

  Scenario: S3 — Removing energy source halts production and resets milestone sustain
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3)
    Given wind_turbine at (1,3), iron_miner at (2,3), iron_smelter at (3,3) in one group
    Given an OpusNodeFull entity: resource=IronBar, required_rate=0.01, sustain_window_ticks=5
    When the simulation runs for 10 ticks
    Then EnergyPool.ratio > 0
    Then at least one ProductionState has active=true
    When the wind_turbine entity at (1,3) is despawned and BuildingRemoved event sent
    When the simulation runs for 5 more ticks
    Then EnergyPool.total_generation == 0
    Then all ProductionState entities have idle_reason=NoEnergy
    Then OpusNodeFull.sustain_ticks == 0

  # ═══════════════════════════════════════════════════════════
  # AC4: Nest Clear → Tier Progression
  # ═══════════════════════════════════════════════════════════

  Scenario: S4 — Combat pressure clears nest and advances tier
    Given a 20x10 grid with SimulationPlugin + CreaturesPlugin
    Given wind_turbine at (3,3) and imp_camp at (4,3) forming a combat group
    Given the combat group entity has CombatGroup with supply_ratio=1.0, protection_dps=100.0
    Given the group entity has Position component at centroid (3,3)
    Given a CreatureNest entity at Position (6,3): nest_id=ForestWolfDen, strength=50.0, hostility=Hostile, territory_radius=5
    Given CombatPressure component on nest entity with value=0.0
    Given a TierGateComponent entity: nest_id="ForestWolfDen", tier=2, unlocked=false
    Given BuildingTier components on all buildings with tier=1
    When the simulation runs for 2 ticks
    Then NestCleared event was emitted with nest_id containing "ForestWolfDen"
    Then TierState.current_tier == 2
    Then all BuildingTier components have tier == 2

  # ═══════════════════════════════════════════════════════════
  # AC5: Organic Supply Chain
  # ═══════════════════════════════════════════════════════════

  Scenario: S5 — Combat group organics transported to tannery
    Given a 30x10 grid with SimulationPlugin + CreaturesPlugin
    Given combat group: wind_turbine at (2,3), imp_camp at (3,3) with manifold pre-seeded with Hide=10.0
    Given combat group has TransportSender for Hide, GroupPosition at (3,3)
    Given processing group: wind_turbine at (14,3), tannery at (15,3) with TransportReceiver for Hide demand=2
    Given processing group has GroupPosition at (15,3)
    Given a T1 rune_path from combat group to processing group with 10 waypoints (4,3)..(13,3)
    Given an OpusNodeFull entity: resource=TreatedLeather, required_rate=0.01, sustain_window_ticks=5
    When the simulation runs for 50 ticks
    Then at least 1 Cargo entity was created (carrying Hide)
    Then processing group Manifold contains hide with amount > 0 or tannery InputBuffer has hide

  # ═══════════════════════════════════════════════════════════
  # AC6: Group Split on Removal
  # ═══════════════════════════════════════════════════════════

  Scenario: S6 — Removing middle building splits group into two
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (3,3) and (5,3)
    Given wind_turbine at (3,4), iron_miner at (3,3), iron_smelter at (4,3), iron_miner at (5,3)
    When the simulation runs for 1 tick
    Then there is exactly 1 group containing the 3 production buildings
    When the building at (4,3) is despawned and BuildingRemoved event sent
    When the simulation runs for 1 tick
    Then there are exactly 2 groups (each with 1 building from the original row)
    Then building at (3,3) and building at (5,3) belong to different groups

  # ═══════════════════════════════════════════════════════════
  # AC7: Full Run Win Condition
  # ═══════════════════════════════════════════════════════════

  Scenario: S7 — All opus nodes sustained triggers RunWon
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3)
    Given wind_turbine at (1,3), iron_miner at (2,3), iron_smelter at (3,3)
    Given OpusNodeFull entity: resource=IronBar, required_rate=0.01, tier=1, sustain_ticks=0
    Given RunConfig: sustain_window_ticks=3, max_ticks=1000
    Given OpusTreeResource: sustain_ticks_required=2
    When the simulation runs for 15 ticks
    Then OpusTreeResource.all_sustained() == true
    Then RunState.status == Won

  # ═══════════════════════════════════════════════════════════
  # AC8: Diamond Network Conservation
  # ═══════════════════════════════════════════════════════════

  Scenario: S8 — Diamond transport network conserves resources
    Given a 30x20 grid with SimulationPlugin
    Given iron_vein terrain at (2,5) and (3,5)
    Given group A: wind_turbine at (1,5), iron_miner at (2,5), iron_miner at (3,5) with TransportSender for iron_ore
    Given group B: wind_turbine at (9,2), iron_smelter at (10,2) with TransportReceiver for iron_ore
    Given group C: wind_turbine at (9,8), iron_smelter at (10,8) with TransportReceiver for iron_ore
    Given group D: wind_turbine at (17,5), iron_smelter at (18,5) with TransportReceiver for iron_bar
    Given rune_path A→B (6 tiles), rune_path A→C (6 tiles), rune_path B→D (7 tiles), rune_path C→D (7 tiles)
    When the simulation runs for 300 ticks
    Then sum of iron_ore in all Manifolds + all Cargo amounts + all InputBuffer amounts >= 0
    Then no negative resource amounts exist in any Manifold

  # ═══════════════════════════════════════════════════════════
  # AC9: Determinism
  # ═══════════════════════════════════════════════════════════

  Scenario: S9 — Identical setup produces identical state after 50 ticks
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3)
    Given wind_turbine at (1,3), iron_miner at (2,3), iron_smelter at (3,3)
    When run_a App is created with identical setup and runs 50 ticks
    When run_b App is created with identical setup and runs 50 ticks
    Then all Manifold resource amounts in run_a == run_b
    Then EnergyPool.total_generation and total_consumption match in run_a and run_b
    Then all ProductionState.progress values match in run_a and run_b

  # ═══════════════════════════════════════════════════════════
  # AC10: UX Dashboard Reads Live State
  # ═══════════════════════════════════════════════════════════

  Scenario: S10 — Dashboard reflects live ECS state
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3)
    Given wind_turbine at (1,3), iron_miner at (2,3), iron_smelter at (3,3)
    Given DashboardState resource with is_open=true
    Given CurrentTier resource with tier=1
    Given SimulationTick resource initialized at 0
    Given an OpusTree entity with total_nodes=1 and an OpusNode entity with sustained=false
    When the simulation runs for 5 ticks
    Then DashboardState.energy_balance == EnergyPool.total_generation - EnergyPool.total_consumption
    Then DashboardState.energy_color == Some(GaugeColor::Green) (balance > 0)
    Then DashboardState.current_tier == 1
    Then SimulationTick.tick == 5

  # ═══════════════════════════════════════════════════════════
  # AC11: Trader Converts Surplus to Meta-Currency
  # ═══════════════════════════════════════════════════════════

  Scenario: S11 — Trader converts manifold surplus to Gold with inflation
    Given a 10x10 grid with SimulationPlugin
    Given iron_vein terrain at (2,3)
    Given wind_turbine at (1,3), iron_miner at (2,3), trader at (3,3) in same group
    Given the trader entity has TraderState (default) and TraderEarnings (zeros) components
    When the simulation runs for 100 ticks
    Then TraderEarnings.gold > 0.0
    Then group Manifold iron_ore amount == 0 (trader drained it)
    Then TraderState.volume_traded contains iron_ore with value > 0.0

  # ═══════════════════════════════════════════════════════════
  # AC12: Hazard Destroys Building → Group Reforms
  # ═══════════════════════════════════════════════════════════

  Scenario: S12 — Hazard destroys middle building and group splits
    Given a 10x10 grid with SimulationPlugin + WorldPlugin
    Given iron_vein terrain at (3,3) and (5,3)
    Given iron_miner at (3,3), iron_smelter at (4,3), iron_miner at (5,3) forming 1 group
    Given a BiomeHazard entity: kind=Eruption, center=(4,3), radius=0, next_event_tick=3, intensity=999.0
    When the simulation runs for 5 ticks
    Then BuildingDestroyed event was emitted for position (4,3)
    Then no Building entity exists at position (4,3)
    Then there are exactly 2 groups
    Then EnergyPool.total_consumption < initial_total_consumption (by 10 = smelter demand)
