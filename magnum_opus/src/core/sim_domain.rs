use super::contract::SimContract;
use super::install_ctx::SimInstaller;
use super::phase::Phase;

/// A module that owns a slice of simulation state and mutates it once per tick.
///
/// `PRIMARY_PHASE` names the module's owning phase. `install(ctx)` receives a
/// scoped `SimInstaller` - it cannot obtain `&mut App`, and every call it makes
/// is checked against the declared contract.
pub trait SimDomain: 'static + Send + Sync {
    const ID: &'static str;
    const PRIMARY_PHASE: Phase;

    fn contract() -> SimContract;

    /// Install resources, messages, and systems through the scoped installer.
    /// Systems attached via `ctx.add_system(...)` land in `Update` under the
    /// module's primary phase.
    fn install(ctx: &mut SimInstaller);
}
