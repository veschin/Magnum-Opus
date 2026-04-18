use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

// Tick, ModuleRegistry, MetricsRegistry, and CoreSeal are claimed by "core"
// in CorePlugin::build. User modules trying to claim any of them trigger
// single-writer violation.

struct TickHijack;
impl SimDomain for TickHijack {
    const ID: &'static str = "tick_hijack";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Tick],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Tick>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn user_cannot_claim_tick() {
    let _ = Harness::new().with_sim::<TickHijack>().build();
}

struct MetricsHijack;
impl SimDomain for MetricsHijack {
    const ID: &'static str = "metrics_hijack";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![MetricsRegistry],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<MetricsRegistry>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn user_cannot_claim_metrics_registry() {
    let _ = Harness::new().with_sim::<MetricsHijack>().build();
}

struct ModuleRegistryHijack;
impl SimDomain for ModuleRegistryHijack {
    const ID: &'static str = "reg_hijack";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![ModuleRegistry],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ModuleRegistry>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-writer violation")]
fn user_cannot_claim_module_registry() {
    let _ = Harness::new().with_sim::<ModuleRegistryHijack>().build();
}
