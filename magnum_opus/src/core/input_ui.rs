use super::contract::MetricDesc;
use super::install_ctx::InputInstaller;
use super::type_key::TypeKey;

/// A module that reads input and sim state, then pushes commands into the bus.
///
/// Owns its own UI state (cursor, camera, input mode) - declared via `writes`.
/// The `InputInstaller` enforces that input systems land in `PreUpdate` and
/// that all `commands_out` and `writes` are claimed.
pub trait InputUI: 'static + Send + Sync {
    const ID: &'static str;

    fn reads() -> &'static [TypeKey];
    fn writes() -> &'static [TypeKey];
    fn commands_out() -> &'static [TypeKey];
    fn metrics() -> &'static [MetricDesc];

    fn install(ctx: &mut InputInstaller);
}
