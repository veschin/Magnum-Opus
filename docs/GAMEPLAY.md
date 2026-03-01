# Magnum Opus — Gameplay Flow

Schematic walkthrough of a single run, from meta-hub to scoring screen.
Purpose: verify that every player action maps to an ECS system.

---

## Pre-Run: Meta Hub

1. Player sees meta-currencies (Gold / Souls / Knowledge) earned from previous runs
2. Player browses unlock shop — buys permanent unlocks (new biomes, starting bonuses, building pools)
3. Player starts new run → biome assigned (random or choice from unlocked set)
4. Opus tree generated for this biome — player sees all main-path milestones + side branches
5. Map generated from biome + seed → fog of war hides most of map, starting area revealed
6. Player receives starting kit: a handful of free buildings + small resource stockpile

**Systems involved:** MetaState, BiomeDB, WorldGenSystem, OpusTree generation

Starting kit = free buildings for first extraction + Mall, defined by RunConfig (biome + meta unlocks).

---

## T1 — Setup (~25 min)

### First minutes: bootstrap

7. Player places first miners on a resource vein (stone, wood, basic ore)
8. Adjacent miners auto-form a group → manifold distributes internally
9. Player places a Mall group nearby (constructor + toolmaker + assembler)
10. Before paths: minions auto-carry resources from miners to Mall (slow, short-range)
11. Mall starts producing buildings → buildings go to Inventory. Player draws first rune path when ready.
12. Player places first energy building (biome-specific: wind turbine, water wheel, lava siphon)
13. Energy pool goes positive → all groups get speed bonus

### Expanding production

14. Player places more extraction groups on different veins
15. Connects them via rune paths to Mall and to each other
16. Ambient creatures wander the map — player notes them as future organic source
17. Player builds first combat group (imp camp) — feeds it weapons from Mall + food
18. Combat group produces organic resources (hides, herbs, wood) + protection radius
19. Player draws paths from combat group → groups that need organics

### T1 milestone + gate

20. Opus tree T1 nodes: simple rates like "produce 2 stone/min", "produce 1 iron bar/min"
21. Player sustains rates → milestones complete
22. Mini-opus branch appears: "trade 5 wood to Trader" → bonus Gold
23. T1 creature nest on map — player builds combat group nearby, overpowers nest
24. Nest cleared → T2 unlocked, all T2 recipes/buildings immediately available

**Systems involved:** BuildingPlacement, GroupFormation, ManifoldSystem, TransportFlow, EnergyGeneration/Distribution, ProductionTick, CombatGroupSystem, RateMonitor, MilestoneCheck, TierGate

Buildings come from Inventory (produced by Mall). Tier gates = creature nest clearing. Trading = Trader building with inflation.

---

## T2 — Expansion (~35 min)

### New tools

25. T2 buildings unlocked: biome-specific extractors, refineries, synthesis groups
26. T2 rune paths auto-upgrade globally — ALL existing paths become faster
27. Pipes unlocked for liquid resources (water, lava, potions)
28. Synthesis groups placed anywhere — convert base resources into others (tree farm: water → wood)

### Complexity rises

29. Multi-step recipes: ore → ingot → plate → component
30. Player needs multiple connected groups: extraction → refining → assembly
31. Chain visualizer becomes essential — shows bottlenecks in red
32. Energy becomes scarce — player sets priorities: HIGH for critical groups, LOW for stockpiling
33. Calculator: "I need 3 steel plates/min" → shows required chain of groups

### Threats

34. Territorial creatures attack — player expanded into their zones
35. Combat groups must be well-fed or enemies breach and damage output senders
36. First hazard event: storm/eruption announced N seconds in advance, zone highlighted
37. Player decides: evacuate buildings from zone, or place sacrifice building (see odds: 70% bonus / 30% loss)
38. Hazard hits — destroys unprotected buildings, enhances tiles, sacrifice resolves
39. Invasive creatures start expanding — uncontested areas get reclaimed

### T2 milestones + gate

40. Opus T2 nodes: "produce 3 refined crystal/min", "produce 2 organic compound/min"
41. Mini-opus: "survive ash storm while maintaining 5 stone/min" → bonus Souls
42. Time-based mini-opus: "reach 4 wood/min before tick X" → miss = lost bonus, no penalty
43. T2 creature nest cleared → T3 unlocked, T3 EXTRACT mode available on cleared nests

**Systems involved:** all T1 systems + WeatherTick, ElementInteraction, HazardSystem, SacrificeOdds, CreatureBehavior (territorial + invasive), TerritoryControl, BottleneckDetector, ProductionCalculator

All T2 recipes available immediately. Buildings auto-upgrade on tier unlock.

---

## T3 — Opus Push (~30 min)

### Final stretch

44. T3 buildings: opus-specific production (opus forge, arcane assembler, etc.)
45. Player builds opus groups targeting final milestones
46. T3 paths auto-upgrade — massive throughput boost to entire network
47. Resource quality matters: some opus recipes need HIGH quality inputs
48. Player optimizes for biome-contextual quality (rotten wood = HIGH in undead biome, etc.)

### Pressure peaks

49. Opus-linked creatures spawn at milestone thresholds — new threat type
50. Invasive creatures are serious — large territories if unchecked since T1
51. Energy is the main bottleneck — every group wants more
52. Player constantly adjusts priorities, pauses low-value groups, overclocks critical ones
53. Hazard events more frequent in T3 — sacrifice-or-flee decisions every few minutes

### Closing the Opus

54. Main-path milestones: "sustain 5 obsidian plates/min", "sustain 3 arcane essence/min"
55. Each requires complex multi-group production chains connected by paths+pipes
56. Sustained = rate held for 30 seconds (600 ticks) minimum
57. Once sustained, milestone locks — doesn't regress even if rate drops
58. Mini-opus branches: high-risk high-reward challenges for meta-currency
59. **Final node: sustain ALL main-path rates simultaneously**
60. Player must keep everything running at once — the production symphony

**Systems involved:** all previous + MiniOpusSystem (complex conditions), VeinDepletion (running out of veins), GroupStats (monitoring everything)

Quality = biome-contextual. BiomeDB defines which resources are HIGH quality per biome.

---

## Run End

### Win

61. All main-path rates sustained simultaneously for verification period
62. Run-end sequence triggers — cinematic moment
63. Scoring: opus completion %, mini-opus count, time remaining, efficiency metrics

### Timeout

64. 90-minute timer expires before final node
65. Partial scoring based on how far through the tree

### Abandoned

66. Player quits mid-run
67. Zero or minimal currency earned

### Scoring Screen

68. Currencies earned: Gold (economy mini-opus), Souls (combat mini-opus), Knowledge (tech mini-opus)
69. Opus multiplier applied: x1.5 / x2 / x3 based on biome-opus difficulty mismatch
70. Total currencies added to MetaState
71. New unlocks now affordable? → back to meta hub

**Systems involved:** RunLifecycle, RunScoring, CurrencyAward, MetaState

---

## Resolved Design Decisions

| # | Question | Resolution |
|---|----------|-----------|
| 1 | Starting kit | RunConfig: free buildings for first extraction + Mall, defined by biome + meta unlocks |
| 2 | Building costs | Buildings produced by Mall → Inventory. Placed from Inventory. Resources stay in manifolds. |
| 3 | Tier gates | Creature nests on map. Clear T1 nest → T2. Clear T2 nest → T3. T3 gives EXTRACT mode (2x cost → bonus). |
| 4 | Trading | Trader building: surplus resources → meta-currency with inflation curve |
| 5 | Recipe unlock | All recipes available immediately at tier transition |
| 6 | Building upgrades | Auto-upgrade on tier unlock, no demolish+rebuild |
| 7 | Resource source | Inventory for buildings, manifolds for resources. Two separate logistics. |
| 8 | Quality | Biome-contextual: same resource has different quality per biome via BiomeDB.qualityMap |
| 9 | Map visibility | Fog of war. Watchtowers reveal cells in radius. Cannot build on hidden tiles. |
| 10 | Minions | Decorative, 1 per building. Visual-only. Combat building minions reflect supply ratio. Pre-path: minions auto-carry. |
