use super::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    /// Monotonic counter (only increments).
    Counter,
    /// Instantaneous value (overwritten each tick).
    Gauge,
    /// Per-tick rate (overwritten each tick).
    Rate,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricDesc {
    pub name: &'static str,
    pub kind: MetricKind,
}

/// Declarative surface of a `SimDomain`.
///
/// Sim modules consume commands (from Input), read and write sim resources,
/// and emit/receive messages. They do NOT produce commands - that's the Input
/// archetype's job. The command flow is strictly Input -> Sim.
///
/// Every slot lists `TypeKey`s of types the module interacts with.
/// The registry enforces closure across all registered modules at
/// `finalize_modules()` time: every consumed name must have a producer.
#[derive(Debug, Clone, Copy)]
pub struct SimContract {
    pub reads: &'static [TypeKey],
    pub writes: &'static [TypeKey],
    pub commands_in: &'static [TypeKey],
    pub messages_in: &'static [TypeKey],
    pub messages_out: &'static [TypeKey],
    pub metrics: &'static [MetricDesc],
}

impl SimContract {
    pub const EMPTY: SimContract = SimContract {
        reads: &[],
        writes: &[],
        commands_in: &[],
        messages_in: &[],
        messages_out: &[],
        metrics: &[],
    };
}
