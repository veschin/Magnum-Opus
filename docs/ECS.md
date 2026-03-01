# Magnum Opus — ECS Decomposition

Full breakdown of components, resources, systems, formulas, tick order, and interconnections.
Open this file to understand WHAT we build.

See [ARCH.md](ARCH.md) for HOW we build and test.

---

## Components

### Spatial

| Component | Fields | Used by |
|-----------|--------|---------|
| Position | x: int, y: int | Everything placed on the grid |
| Footprint | w: int, h: int | Buildings with size > 1x1 |
| Occupies | cells: Vec2[] | All cells an entity occupies (derived from Position + Footprint) |

### Terrain

| Component | Fields | Used by |
|-----------|--------|---------|
| Terrain | type: TerrainType | Every tile (grass, rock, water, lava, sand, ice...) |
| ResourceVein | resource: ResourceID, quality: Quality, remaining: float | Mineable deposits |
| HazardZone | hazardType: HazardType, intensity: float, nextEventTick: int | Recurring danger areas |
| TileEnhancement | type: EnhancementType, magnitude: float | Post-hazard tile bonus |
| ElementalState | fire: float, water: float, cold: float, wind: float | Elemental levels per tile |
| Buildable | value: bool | Whether buildings can be placed |
| Visibility | state: Hidden/Revealed/Visible, lastSeenTick: int | Fog of war per tile |

### Building

| Component | Fields | Used by |
|-----------|--------|---------|
| Building | type: BuildingTypeID, tier: int | Every placed building |
| GroupMember | groupID: EntityID | Links building to its group |
| Recipe | recipeID: RecipeID | Active production recipe |
| InputBuffer | slots: map[ResourceID → float] | Resources waiting to be consumed |
| OutputBuffer | slots: map[ResourceID → float] | Produced resources awaiting pickup |
| ProductionState | progress: float (0→1), active: bool | Current recipe progress |
| TerrainRequirement | requires: TerrainType | Placement constraint (miner → ore vein) |
| SacrificeBuilding | inHazardZone: bool, successChance: float | Sacrifice mechanic state |
| EnergySource | output: float | Energy generation per tick |
| FogRevealer | radius: int | Watchtower fog reveal radius |

### Group

| Component | Fields | Used by |
|-----------|--------|---------|
| Group | members: EntityID[], groupType: GroupType | Group entity |
| Manifold | resources: map[ResourceID → float] | Shared resource pool inside group |
| GroupIO | inputs: PortDef[], outputs: PortDef[] | Boundary receivers/senders |
| GroupEnergy | demand: float, allocated: float, priority: Priority | Energy state |
| GroupStats | productionRates: map[ResourceID → float] | Aggregate throughput |

### Transport

| Component | Fields | Used by |
|-----------|--------|---------|
| Path | segments: Vec2[], tier: int, resourceClass: Solid/Liquid | Rune path or pipe |
| PathConnection | fromGroup: EntityID, toGroup: EntityID, fromPort: int, toPort: int | Group-to-group link |
| Cargo | resource: ResourceID, amount: float, positionOnPath: float | Resource in transit |

### Creature

| Component | Fields | Used by |
|-----------|--------|---------|
| Creature | species: SpeciesID, archetype: Archetype | Identity |
| Territory | center: Vec2, radius: int | Owned area |
| Health | current: float, max: float | HP |
| CreatureAI | state: AIState, target: EntityID | Behavior state machine |
| Loot | drops: map[ResourceID → float] | Resources on death |
| CreatureNest | tier: int, hostility: Hostile/Neutral, cleared: bool, extracting: bool | Nest entity |

### Progression

| Component | Fields | Used by |
|-----------|--------|---------|
| OpusNode | resource: ResourceID, requiredRate: float, sustained: bool | Main path milestone |
| MiniOpus | triggerType: TriggerType, condition: Condition, reward: CurrencyReward, deadline: int | Branch challenge |
| TierGate | tier: int, unlocked: bool, nestID: EntityID | Tier unlock state |

### Meta

| Component | Fields | Used by |
|-----------|--------|---------|
| MetaUnlock | type: UnlockType, cost: CurrencyCost, purchased: bool | Persistent unlock |
| Trader | exchangeRates: map[ResourceID → float], inflation: map[ResourceID → float] | Trading building |

**Total: ~37 component types**

---

## Resources (Global Singletons)

| Resource | Fields | Purpose |
|----------|--------|---------|
| SimClock | tick: int, runElapsed: float | Simulation time |
| EnergyPool | totalGen: float, totalConsumption: float, ratio: float | Global energy balance |
| TierState | currentTier: int, transportTier: int | Current progression tier |
| RunState | phase: RunPhase, biome: BiomeID, seed: int64 | Run metadata |
| RunConfig | startingBuildings: map[BuildingTypeID → int], startingResources: map[ResourceID → float] | Initial conditions |
| Weather | current: WeatherType, intensity: float, nextChangeTick: int | Active weather |
| OpusTree | nodes: EntityID[], completionPct: float | Tree structure |
| Inventory | buildings: map[BuildingTypeID → int] | Player's building stock |
| CommandBuffer | commands: Command[] | Player commands this tick |
| EventQueue | events: Event[] | Cross-system events this tick |
| RecipeDB | recipes: map[RecipeID → RecipeDef] | Static — all recipes |
| BuildingDB | buildings: map[BuildingTypeID → BuildingDef] | Static — all building types |
| CreatureDB | species: map[SpeciesID → SpeciesDef] | Static — all creature species |
| BiomeDB | biomes: map[BiomeID → BiomeDef] | Static — all biomes + quality maps |
| MetaState | currencies: [Gold, Souls, Knowledge], unlocks: UnlockID[] | Persistent between runs |

**Total: 15 resources**

---

## Systems (40 total)

### Phase 0 — Input

**CommandProcessSystem**
- Reads: CommandBuffer, Inventory, BuildingDB, TierState, spatial index, Visibility
- Writes: creates/destroys entities, updates Inventory
- Logic:
  ```
  for cmd in CommandBuffer:
      match cmd:
          PlaceBuilding{pos, type}:
              validate: tile is Buildable, Visible, meets TerrainRequirement
              validate: Inventory[type] > 0
              validate: BuildingDB[type].tier <= TierState.currentTier
              → create entity with Position, Building, Recipe, etc.
              → Inventory[type] -= 1
              → emit BuildingPlaced{entity}

          RemoveBuilding{id}:
              → destroy entity
              → emit BuildingDestroyed{id}

          DrawPath{from, to, waypoints}:
              validate: all tiles Buildable, no existing path/pipe on tiles
              → create Path entity with segments
              → create PathConnection linking groups

          SetGroupPriority{groupID, priority}:
              → update GroupEnergy.priority

          ExtractNest{nestID}:
              validate: nest.cleared == true, TierState >= 3
              → nest.extracting = true
  ```

---

### Phase 1 — World

**WeatherTickSystem**
- Reads: SimClock, Weather, BiomeDB
- Writes: Weather, ElementalState on affected tiles
- Formula:
  ```
  if SimClock.tick >= Weather.nextChangeTick:
      Weather.current = BiomeDB[biome].weatherTable.roll(rng)
      Weather.nextChangeTick = SimClock.tick + WEATHER_DURATION[Weather.current]
  apply weather effects to ElementalState:
      rain → tile.water += RAIN_RATE * Weather.intensity
      heat → tile.fire += HEAT_RATE * Weather.intensity
      cold → tile.cold += COLD_RATE * Weather.intensity
      wind → tile.wind = Weather.intensity
  ```

**ElementInteractionSystem**
- Reads: ElementalState on all tiles, Weather
- Writes: ElementalState, Terrain (state changes)
- Formula:
  ```
  for each tile:
      // fire + wind → spread
      if tile.fire > FIRE_THRESHOLD and tile.wind > 0:
          for neighbor in cardinal(tile):
              spread_chance = tile.fire * tile.wind * FIRE_SPREAD_FACTOR
              if rng.float() < spread_chance:
                  neighbor.fire += SPREAD_AMOUNT

      // rain fills water
      if tile.water > 0 and tile.terrain == dry_soil:
          tile.terrain = wet_soil

      // cold freezes water
      if tile.cold > FREEZE_THRESHOLD and tile.water > 0:
          tile.water -= FREEZE_RATE
          tile.terrain = ice

      // natural decay
      tile.fire *= FIRE_DECAY
      tile.cold *= COLD_DECAY
  ```

**HazardSystem**
- Reads: HazardZone, SimClock, spatial index
- Writes: buildings (destroy), TileEnhancement, SacrificeBuilding
- Formula:
  ```
  for each zone where SimClock.tick >= zone.nextEventTick:
      // announce N ticks before (UI reads this)
      if SimClock.tick >= zone.nextEventTick:
          for building in buildings_in(zone.cells):
              if building has SacrificeBuilding:
                  if rng.float() < building.successChance:
                      emit SacrificeHit{building, reward: BASE_REWARD * zone.intensity}
                  else:
                      emit BuildingDestroyed{building}
              else:
                  emit BuildingDestroyed{building}

          for tile in zone.cells:
              tile.enhancement = {zone.hazardType, zone.intensity * ENHANCE_FACTOR}

          zone.nextEventTick += HAZARD_INTERVAL[zone.hazardType]
  ```

---

### Phase 2 — Creatures

**CreatureSpawnSystem**
- Reads: RunState, BiomeDB, SimClock, creature count per species
- Writes: new Creature entities
- Formula:
  ```
  for each species in BiomeDB[biome].creatures:
      if count[species] < biome.capacity[species]:
          if rng.float() < SPAWN_RATE[species]:
              spawn creature at valid position near species.spawnZone
  ```

**CreatureBehaviorSystem**
- Reads: Creature, CreatureAI, Territory, Position, spatial index, combat group positions
- Writes: CreatureAI, Position, Territory, Health
- Formula per archetype:
  ```
  ambient:
      wander randomly within home range
      flee from combat groups within FLEE_RADIUS
      if health < FLEE_THRESHOLD: move away from danger

  territorial:
      if any player building in Territory:
          state = AGGRESSIVE
          move toward nearest player building
          on arrival: damage building output senders
      else:
          patrol territory border

  invasive:
      if no combat group protection overlaps territory:
          territory.radius += EXPANSION_RATE per tick
      else:
          expansion suppressed (see TerritoryControlSystem)
      spawn child creatures when territory large enough

  event_born:
      spawn when associated event fires
      aggressive for LIFETIME ticks, then despawn

  opus_linked:
      spawn when opus_node[triggerIndex].sustained becomes true
      behave like territorial but stronger
  ```

---

### Phase 3 — Energy

**EnergyGenerationSystem**
- Reads: all entities with EnergySource + Building (active)
- Writes: EnergyPool.totalGen
- Formula: `totalGen = sum(source.output for all active energy buildings)`

**EnergyConsumptionSystem**
- Reads: all GroupEnergy
- Writes: EnergyPool.totalConsumption
- Formula: `totalConsumption = sum(group.demand for all active groups)`

**EnergyDistributionSystem**
- Reads: EnergyPool, all GroupEnergy
- Writes: GroupEnergy.allocated per group
- Formula:
  ```
  ratio = totalGen / totalConsumption   (handle div-by-zero: if consumption=0, ratio=1)

  if ratio >= 1.0:
      // surplus: everyone gets proportional bonus
      for each group:
          group.allocated = group.demand * ratio

  else:
      // deficit: distribute by priority
      remaining = totalGen
      for priority in [HIGH, MEDIUM, LOW]:
          groups = groups_at_priority(priority)
          total_demand = sum(g.demand for g in groups)
          share = min(remaining, total_demand)
          for g in groups:
              g.allocated = share * (g.demand / total_demand)
          remaining -= share
  ```

---

### Phase 4 — Production

**ProductionTickSystem**
- Reads: Building, Recipe, InputBuffer, GroupMember, GroupEnergy, RecipeDB, Manifold
- Writes: InputBuffer, OutputBuffer, ProductionState, Manifold
- Formula:
  ```
  for each building with Recipe:
      group = groups[building.GroupMember.groupID]
      energy_modifier = group.allocated / group.demand   // 0.0 to ~1.5+
      speed = energy_modifier                            // 1.0 = normal speed

      recipe = RecipeDB[building.recipeID]

      if not building.ProductionState.active:
          // try to grab inputs from manifold
          can_start = true
          for (resource, amount) in recipe.inputs:
              if group.manifold[resource] < amount:
                  can_start = false
                  break
          if can_start:
              for (resource, amount) in recipe.inputs:
                  group.manifold[resource] -= amount
              building.ProductionState.active = true
              building.ProductionState.progress = 0

      if building.ProductionState.active:
          building.ProductionState.progress += speed / recipe.durationTicks
          if building.ProductionState.progress >= 1.0:
              for (resource, amount) in recipe.outputs:
                  quality_mult = if BiomeDB[biome].qualityMap[resource] == HIGH: 1 + QUALITY_BONUS else: 1
                  building.OutputBuffer[resource] += amount * quality_mult
              building.ProductionState.active = false
              building.ProductionState.progress = 0
  ```

**ManifoldSystem**
- Reads: Group, OutputBuffer of all members
- Writes: Manifold, OutputBuffer (clear)
- Formula:
  ```
  for each group:
      // collect outputs into shared pool
      for building in group.members:
          for (resource, amount) in building.OutputBuffer:
              group.manifold[resource] += amount
              building.OutputBuffer[resource] = 0
  ```

**GroupStatsSystem**
- Reads: Group, member ProductionState, RecipeDB
- Writes: GroupStats
- Formula:
  ```
  for each group:
      rates = {}
      for building in group.members:
          recipe = RecipeDB[building.recipeID]
          if building is actively producing:
              for (resource, amount) in recipe.outputs:
                  rates[resource] += amount / recipe.durationTicks * energy_modifier
      group.stats.productionRates = rates
  ```

---

### Phase 5 — Transport

**MinionCarrySystem**
- Reads: Groups without path connections, Manifold, spatial index
- Writes: Manifold of destination group
- Formula:
  ```
  for each group with surplus output (manifold has resources no internal building needs):
      nearby_groups = groups_within(MINION_CARRY_RANGE)
      for nearby in nearby_groups:
          for resource that nearby needs but group has:
              transfer = min(available, MINION_CARRY_RATE, nearby_demand)
              source.manifold[resource] -= transfer
              dest.manifold[resource] += transfer
  ```
  - Slow: MINION_CARRY_RATE << path throughput
  - Short range: MINION_CARRY_RANGE = few cells
  - No explicit minion entities — this is an abstract system

**TransportFlowSystem**
- Reads: PathConnection, Path, GroupIO, Manifold
- Writes: Cargo entities, source/dest Manifold
- Formula:
  ```
  TIER_CAPACITY = {T1: cap1, T2: cap2, T3: cap3}   // items per tick
  TIER_SPEED    = {T1: spd1, T2: spd2, T3: spd3}   // cells per tick

  // launch new cargo
  for each path_connection:
      available = source.manifold[path.resource]
      capacity = TIER_CAPACITY[TierState.transportTier]
      // demand = what dest group needs
      demand = dest_group_input_demand(path.resource)

      flow = min(available, capacity, demand)
      if flow > 0:
          source.manifold[path.resource] -= flow
          create Cargo{resource, amount: flow, positionOnPath: 0}

  // move existing cargo
  for each cargo:
      cargo.positionOnPath += TIER_SPEED[TierState.transportTier]
      if cargo.positionOnPath >= path.length:
          dest.manifold[cargo.resource] += cargo.amount
          destroy cargo
  ```

---

### Phase 6 — Combat

**CombatGroupSystem**
- Reads: Group (combat type), Manifold, Building (imp camp, breeding pen)
- Writes: Manifold (organic output), protection state
- Formula:
  ```
  for each combat group:
      supply_ratio = min_input_supply / required_input   // 0.0 to 1.0
      combat_efficiency = clamp(supply_ratio, 0, 1)

      organic_output = BASE_ORGANIC_RATE * combat_efficiency * energy_modifier
      protection_radius = BASE_PROTECTION_RADIUS * combat_efficiency

      for (resource, amount) in organic_recipe.outputs:
          group.manifold[resource] += organic_output * amount

      if combat_efficiency < BREACH_THRESHOLD:
          emit TerritoryBreach{group}
  ```

**TerritoryControlSystem**
- Reads: combat group protection radii, Creature positions, Territory
- Writes: Creature Health, Territory radius
- Formula:
  ```
  for each creature near combat groups:
      dist = manhattan(creature.pos, combat_group.pos)
      if dist <= protection_radius:
          creature.health -= PROTECTION_DPS * combat_efficiency
          if creature.health <= 0:
              for (resource, amount) in creature.loot:
                  nearest_combat_group.manifold[resource] += amount
              emit CreatureKilled{creature}

  for each invasive creature:
      suppression = sum(protection_strength for combat groups in range)
      net_expansion = EXPANSION_RATE - suppression * SUPPRESSION_FACTOR
      territory.radius += max(0, net_expansion)
  ```

**NestClearingSystem**
- Reads: CreatureNest, nearby combat groups, TierState
- Writes: CreatureNest.cleared, CreatureNest.extracting, TierGate
- Formula:
  ```
  for each nest:
      if not nest.cleared:
          combat_pressure = sum(protection of combat groups within nest.radius)
          if combat_pressure > NEST_STRENGTH[nest.tier]:
              nest.cleared = true
              emit NestCleared{nest}

      if nest.extracting:  // T3 EXTRACT mode
          // nearby combat groups get 2x consumption, 2x output
          for group in combat_groups_near(nest):
              group.consumptionMultiplier = 2.0
              group.outputMultiplier = 2.0
  ```

---

### Phase 7 — Progression

**RateMonitorSystem**
- Reads: GroupStats (all groups)
- Writes: global rate tracking (rolling window)
- Formula:
  ```
  SUSTAIN_WINDOW = configurable ticks (e.g. 600 = 30 sec at 20 tps)

  for each resource tracked by opus:
      current_rate = sum(groupStats.rates[resource] for all groups)
      rate_history[resource].push(current_rate)
      if rate_history[resource].len > SUSTAIN_WINDOW:
          rate_history[resource].removeOldest()
      sustained_rate[resource] = min(rate_history[resource])
  ```

**MilestoneCheckSystem**
- Reads: OpusNode, sustained rates
- Writes: OpusNode.sustained, emits MilestoneReached
- Formula:
  ```
  for each opus_node:
      if not node.sustained:
          if sustained_rate[node.resource] >= node.requiredRate:
              node.sustained = true
              emit MilestoneReached{node}
  ```

**MiniOpusSystem**
- Reads: MiniOpus, various game state, SimClock
- Writes: MiniOpus completion state, emits MiniOpusCompleted/Missed
- Formula:
  ```
  for each mini_opus:
      match trigger_type:
          on_demand: check if condition met → complete
          time_based: if SimClock.tick > deadline → emit Missed
                      elif condition met → emit Completed
          conditional: if game state matches condition → emit Completed
  ```

**TierGateSystem**
- Reads: TierGate, EventQueue (NestCleared events)
- Writes: TierState, TierGate.unlocked, emits TierUnlocked
- Formula:
  ```
  for each tier_gate:
      if not gate.unlocked:
          if NestCleared event for gate.nestID exists:
              gate.unlocked = true
              TierState.currentTier = max(TierState.currentTier, gate.tier)
              TierState.transportTier = gate.tier  // global transport upgrade
              emit TierUnlocked{gate.tier}
              // auto-upgrade all buildings of lower tier
  ```

---

### Phase 8 — Cleanup

**VeinDepletionSystem**
- Reads: ResourceVein, extraction groups on top
- Writes: ResourceVein.remaining
- Formula:
  ```
  for each vein with extraction group:
      extraction_rate = group production rate for vein.resource
      vein.remaining -= extraction_rate
      if vein.remaining <= 0:
          vein.remaining = 0
          // buildings on vein stop producing (no input)
  ```

**BuildingDestructionSystem**
- Reads: EventQueue (BuildingDestroyed events)
- Writes: removes entities, emits GroupChanged
- Logic: destroy building entity, remove from group.members, emit GroupChanged

**GroupRecalculationSystem**
- Reads: GroupChanged events, Position of all buildings
- Writes: Group, GroupMember (reassignment)
- Algorithm:
  ```
  for each affected group:
      // flood-fill from first remaining member
      components = connected_components(group.members, by cardinal adjacency)
      if components.count == 1:
          // group intact, no change
      elif components.count > 1:
          // split: keep first component as original group
          // create new Group entities for other components
          emit GroupSplit{...}
      elif components.count == 0:
          // all buildings destroyed → destroy group
          disconnect all paths to/from this group

  // also check for merges when BuildingPlaced
  for each BuildingPlaced event:
      adjacent_groups = unique groups of cardinal neighbors
      if adjacent_groups.count == 0:
          create new Group with this building
      elif adjacent_groups.count == 1:
          add building to existing group
      else:
          merge all adjacent groups into one
          emit GroupMerged{...}
  ```

**FogOfWarSystem**
- Reads: all FogRevealer + Position
- Writes: Visibility on tiles
- Formula:
  ```
  // reset all to hidden (or revealed-but-not-visible)
  for each tile: if tile.visibility == Visible: tile.visibility = Revealed

  // reveal from watchtowers
  for each entity with FogRevealer:
      for tile in cells_within_radius(entity.pos, revealer.radius):
          tile.visibility = Visible
          tile.lastSeenTick = SimClock.tick
  ```

**TradingSystem**
- Reads: Trader buildings, Manifold of trader's group
- Writes: MetaState.currencies, Trader.inflation
- Formula:
  ```
  for each trader building:
      for resource in trader.group.manifold:
          if manifold[resource] > 0:
              rate = trader.exchangeRates[resource] / (1 + trader.inflation[resource])
              currency_earned = manifold[resource] * rate
              MetaState.currencies[resource.currencyType] += currency_earned
              trader.inflation[resource] += manifold[resource] * INFLATION_FACTOR
              manifold[resource] = 0
  ```

**RunLifecycleSystem**
- Reads: SimClock, OpusTree, RunState
- Writes: RunState, emits RunWon / RunTimeUp
- Formula:
  ```
  if all opus main nodes have sustained == true:
      emit RunWon
  if SimClock.tick >= RUN_DURATION_TICKS:
      emit RunTimeUp

  run_score = opus_completion_pct * OPUS_WEIGHT + mini_opus_score * MINI_WEIGHT + time_bonus
  ```

---

### Phase 9 — Meta (on run end only)

**CurrencyAwardSystem**
- Reads: completed MiniOpus, OpusTree, RunState
- Writes: MetaState.currencies
- Formula:
  ```
  OPUS_MULTIPLIER = {easy: 1.0, medium: 1.5, hard: 2.0, extreme: 3.0}
  difficulty = calculate_difficulty(RunState.biome, OpusTree)

  for each completed mini_opus:
      MetaState.currencies[mini_opus.reward.type] += mini_opus.reward.amount * OPUS_MULTIPLIER[difficulty]
  ```

---

## Tick Pipeline Summary

```
Phase 0: Input       │ CommandProcess
Phase 1: World       │ Weather → Elements → Hazards
Phase 2: Creatures   │ Spawn → Behavior
Phase 3: Energy      │ Generation → Consumption → Distribution
Phase 4: Production  │ ProductionTick → Manifold → GroupStats
Phase 5: Transport   │ MinionCarry → PathFlow
Phase 6: Combat      │ CombatGroup → TerritoryControl → NestClearing
Phase 7: Progression │ RateMonitor → MilestoneCheck → MiniOpus → TierGate
Phase 8: Cleanup     │ VeinDepletion → Destruction → GroupRecalc → FogOfWar → Trading → RunLifecycle
Phase 9: Meta        │ CurrencyAward (run end only)
```

---

## System Interconnection Graph

```
                    ┌──── Player Input ────┐
                    ▼                      │
              CommandProcess               │
              ╱     │     ╲                │
    Building     Path      Priority    Command
    Placed    Connected    Changed     Buffer
       │          │           │
       ▼          ▼           ▼
   ┌─────────────────────────────────────────────┐
   │              WORLD SIMULATION               │
   │  Weather ──→ Elements ──→ Hazards           │
   │                              │               │
   │              BuildingDestroyed               │
   └──────────────────┬──────────────────────────┘
                      │
   ┌──────────────────┼──────────────────────────┐
   │              CREATURES                       │
   │  Spawn ──→ Behavior ──→ attacks/expands     │
   └──────────────────┬──────────────────────────┘
                      │
   ┌──────────────────┼──────────────────────────┐
   │              ENERGY                          │
   │  Generation ──→ Consumption ──→ Distribution │
   │                                    │         │
   │                          energy_modifier     │
   └──────────────────┬─────────────┬────────────┘
                      │             │
   ┌──────────────────┼─────────────┼────────────┐
   │              PRODUCTION        │             │
   │  ProductionTick ◄──────────────┘             │
   │       │                                      │
   │       ▼                                      │
   │  Manifold ──→ GroupStats                     │
   │       │              │                       │
   └───────┼──────────────┼──────────────────────┘
           │              │
   ┌───────┼──────────────┼──────────────────────┐
   │  TRANSPORT           │                       │
   │  MinionCarry ◄───────┘ (surplus detection)   │
   │  PathFlow ◄─── manifold outputs              │
   │       │                                      │
   └───────┼──────────────────────────────────────┘
           │ delivered resources
   ┌───────┼──────────────────────────────────────┐
   │  COMBAT                                      │
   │  CombatGroup ◄── delivered weapons/food      │
   │       │ organic output + protection           │
   │       ▼                                      │
   │  TerritoryControl ◄── creature positions     │
   │       │ kills → loot                          │
   │       ▼                                      │
   │  NestClearing ◄── combat pressure            │
   │       │ NestCleared event                     │
   └───────┼──────────────────────────────────────┘
           │
   ┌───────┼──────────────────────────────────────┐
   │  PROGRESSION                                 │
   │  RateMonitor ◄── GroupStats (all groups)     │
   │       │ sustained_rate                        │
   │       ▼                                      │
   │  MilestoneCheck ──→ MilestoneReached         │
   │       │                                      │
   │  MiniOpus ──→ Completed/Missed               │
   │       │                                      │
   │  TierGate ◄── NestCleared                    │
   │       │ TierUnlocked → auto-upgrade          │
   └───────┼──────────────────────────────────────┘
           │
   ┌───────┼──────────────────────────────────────┐
   │  CLEANUP                                     │
   │  VeinDepletion ── finite resources           │
   │  Destruction ── remove dead entities          │
   │  GroupRecalc ── split/merge                   │
   │  FogOfWar ── visibility update                │
   │  Trading ── surplus → meta-currency           │
   │  RunLifecycle ── win/timeout check            │
   └───────┼──────────────────────────────────────┘
           │ RunEnded
   ┌───────┼──────────────────────────────────────┐
   │  META                                        │
   │  CurrencyAward ── rewards × multiplier       │
   └──────────────────────────────────────────────┘
```

**Key data flows:**
1. Energy → Production: energy_modifier controls speed
2. Production → Manifold → Transport: resources flow between groups
3. Production (Mall) → Inventory → Placement: buildings produced then placed
4. Creatures → Combat Groups → Organics: only organic source
5. Combat pressure → Nest clearing → Tier unlock: progression through combat
6. Production rates → Rate monitor → Milestones → Opus completion
7. Mini-opus → Meta currencies: between-run progression
8. Surplus resources → Trader → Meta currencies: alternative meta income

---

## White Spots

Decisions deferred to seed data / implementation:

| # | Question | Options | Impact |
|---|----------|---------|--------|
| 1 | Manifold distribution priority | Round-robin / proportional demand / FIFO | Affects production fairness within groups |
| 2 | Minion carry pathfinding | Manhattan distance nearest / A* | Performance vs accuracy tradeoff |
| 3 | Sacrifice success formula | Flat chance per hazard / building bonus modifiers | Risk/reward tuning |
| 4 | Creature nest strength scaling | Linear per tier / exponential / biome-specific | Combat difficulty curve |
| 5 | Trading inflation curve | Linear / exponential / logarithmic | Meta economy balance |
| 6 | Fog reveal radius per tier | Fixed per building type / scales with tier | Exploration pacing |
| 7 | Element interaction rates | All tuning constants (spread, decay, threshold) | World feel and danger |
| 8 | Run scoring weights | Opus weight vs mini-opus weight vs time bonus | Meta reward balance |
| 9 | Starting kit composition | Per biome? Affected by meta unlocks? | Early game feel |
| 10 | Opus tree generation algorithm | Random from template / fully procedural / hand-crafted | Replayability vs balance |
