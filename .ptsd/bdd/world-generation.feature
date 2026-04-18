@feature:world-generation
Feature: world-generation

  Deterministic terrain generation with clustered resource veins.
  Scenarios map 1:1 to PRD F2 acceptance criteria.

  Scenario: AC1 - landscape generates full grid after two ticks
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and LandscapeModule
    When app.update() runs twice
    Then Landscape.ready is true and Landscape.cells length equals 4096

  Scenario: AC2 - landscape is deterministic across two independent runs
    Given two Harness builds with identical WorldConfig with seed 0x9E3779B97F4A7C15
    When each runs app.update() twice
    Then both Landscape.cells vectors compare equal by bit

  Scenario: AC3 - landscape shows variety of terrain kinds
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and LandscapeModule
    When app.update() runs twice
    Then Landscape.cells contains at least 4 distinct TerrainKind values and metric landscape.kinds_present equals that count

  Scenario: AC4 - resources generate after landscape emits LandscapeGenerated
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and LandscapeModule and ResourcesModule
    When app.update() runs twice
    Then ResourceVeins.ready is true and ResourceVeins.veins length is greater than zero

  Scenario: AC5 - every vein sits on terrain that matches its resource rule
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and LandscapeModule and ResourcesModule after two app.update() calls
    When each vein is inspected at its position
    Then every vein's resource kind matches the terrain rule at that cell

  Scenario: AC6 - at least one cluster contains five or more veins in radius 3
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and LandscapeModule and ResourcesModule after two app.update() calls
    When cluster centers are inspected
    Then at least one cluster position has five or more veins within Manhattan-3

  Scenario: AC7 - landscape without world_config panics closed-reads
    Given a Harness with LandscapeModule only
    When Harness.build runs
    Then the panic message contains the substring closed-reads

  Scenario: AC8 - resources without landscape panics on both missing sides
    Given a Harness with WorldConfigModule with seed 0x9E3779B97F4A7C15 and ResourcesModule but no LandscapeModule
    When Harness.build runs
    Then the panic message contains both substrings closed-messages and closed-reads

  Scenario: AC9 - resource veins are deterministic across two independent runs
    Given two Harness builds with identical WorldConfig with seed 0x9E3779B97F4A7C15
    When each runs app.update() twice
    Then both ResourceVeins.veins maps compare equal by BTreeMap equality

  Scenario: Edge - seed zero still produces valid generation
    Given a WorldConfig with seed 0
    When Harness runs app.update() twice
    Then Landscape.ready is true and ResourceVeins.ready is true and no panic occurs
