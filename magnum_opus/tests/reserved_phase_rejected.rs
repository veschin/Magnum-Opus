use magnum_opus::core::*;

struct ReservedCommands;
impl SimDomain for ReservedCommands {
    const ID: &'static str = "reserved_commands";
    const PRIMARY_PHASE: Phase = Phase::Commands;
    fn contract() -> SimContract {
        SimContract::EMPTY
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

struct ReservedMetrics;
impl SimDomain for ReservedMetrics {
    const ID: &'static str = "reserved_metrics";
    const PRIMARY_PHASE: Phase = Phase::Metrics;
    fn contract() -> SimContract {
        SimContract::EMPTY
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

struct ReservedEnd;
impl SimDomain for ReservedEnd {
    const ID: &'static str = "reserved_end";
    const PRIMARY_PHASE: Phase = Phase::End;
    fn contract() -> SimContract {
        SimContract::EMPTY
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "reserved by the core runtime")]
fn primary_phase_commands_panics() {
    let _ = Harness::new().with_sim::<ReservedCommands>().build();
}

#[test]
#[should_panic(expected = "reserved by the core runtime")]
fn primary_phase_metrics_panics() {
    let _ = Harness::new().with_sim::<ReservedMetrics>().build();
}

#[test]
#[should_panic(expected = "reserved by the core runtime")]
fn primary_phase_end_panics() {
    let _ = Harness::new().with_sim::<ReservedEnd>().build();
}
