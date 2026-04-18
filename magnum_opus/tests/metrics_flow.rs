use bevy::prelude::*;
use magnum_opus::core::*;

const COUNTER: MetricDesc = MetricDesc {
    name: "metrics_mod.ticks",
    kind: MetricKind::Counter,
};

const GAUGE: MetricDesc = MetricDesc {
    name: "metrics_mod.latest",
    kind: MetricKind::Gauge,
};

struct M;
impl SimDomain for M {
    const ID: &'static str = "metrics_mod";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            metrics: &[COUNTER, GAUGE],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
        ctx.add_metric_publish(|mut reg: ResMut<MetricsRegistry>| {
            reg.inc("metrics_mod.ticks", 1.0);
            let t = reg.get("metrics_mod.ticks").unwrap();
            reg.set("metrics_mod.latest", t * 10.0);
        });
    }
}

#[test]
fn counter_increments_per_tick() {
    let mut app = Harness::new().with_sim::<M>().build();
    app.update();
    app.update();
    app.update();
    let reg = app.world().resource::<MetricsRegistry>();
    assert_eq!(reg.get("metrics_mod.ticks"), Some(3.0));
    assert_eq!(reg.get("metrics_mod.latest"), Some(30.0));
    assert_eq!(reg.owner("metrics_mod.ticks"), Some("metrics_mod"));
}

struct DupA;
impl SimDomain for DupA {
    const ID: &'static str = "dup_a";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            metrics: &[MetricDesc {
                name: "shared.metric",
                kind: MetricKind::Counter,
            }],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

struct DupB;
impl SimDomain for DupB {
    const ID: &'static str = "dup_b";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            metrics: &[MetricDesc {
                name: "shared.metric",
                kind: MetricKind::Counter,
            }],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "duplicate metric name")]
fn duplicate_metric_name_panics() {
    let _ = Harness::new().with_sim::<DupA>().with_sim::<DupB>().build();
}

const NOT_A_COUNTER: MetricDesc = MetricDesc {
    name: "gauge_only.val",
    kind: MetricKind::Gauge,
};

struct GaugeOnly;
impl SimDomain for GaugeOnly {
    const ID: &'static str = "gauge_only";
    const PRIMARY_PHASE: Phase = Phase::Progression;
    fn contract() -> SimContract {
        SimContract {
            metrics: &[NOT_A_COUNTER],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.add_system(|| {});
        ctx.add_metric_publish(|mut reg: ResMut<MetricsRegistry>| {
            reg.inc("gauge_only.val", 1.0);
        });
    }
}

#[test]
#[should_panic(expected = "non-counter")]
fn inc_on_gauge_panics() {
    let mut app = Harness::new().with_sim::<GaugeOnly>().build();
    app.update();
}
