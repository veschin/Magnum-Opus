use super::contract::MetricDesc;
use super::install_ctx::ViewInstaller;
use super::type_key::TypeKey;

/// A read-only projection of sim state.
///
/// Reads sim resources, may own view-private resources (declared via `writes`),
/// and spawns scene entities in `PostUpdate`. The `ViewInstaller` enforces that
/// view systems land in `PostUpdate` and that all `writes` are claimed.
pub trait View: 'static + Send + Sync {
    const ID: &'static str;

    fn reads() -> &'static [TypeKey];

    /// View-private resources this module exclusively writes (e.g. scene caches).
    fn writes() -> &'static [TypeKey];

    fn metrics() -> &'static [MetricDesc];

    fn install(ctx: &mut ViewInstaller);
}
