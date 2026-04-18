use super::contract::MetricDesc;
use super::install_ctx::DataInstaller;
use super::type_key::TypeKey;

/// A module that loads read-only reference data once at startup.
pub trait StaticData: 'static + Send + Sync {
    const ID: &'static str;

    fn writes() -> &'static [TypeKey];
    fn metrics() -> &'static [MetricDesc];

    /// Install resources and startup systems through the scoped installer.
    fn install(ctx: &mut DataInstaller);
}
