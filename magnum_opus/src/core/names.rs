/// Binds a list of Rust types to a `&'static [TypeKey]` with stable identity
/// via `TypeId`.
///
/// Two types with the same simple name (`a::Grid` vs `b::Grid`) produce
/// distinct `TypeKey`s - killing the name-collision class of v1 drift bugs.
/// The diagnostic `name` field carries whatever the caller wrote (via
/// `stringify!`), while equality/hashing use `TypeId`.
///
/// Usage:
/// ```ignore
/// use magnum_opus::{names, core::{SimContract, TypeKey}};
/// # struct Grid; struct PlaceTile; struct TilePlaced;
///
/// const C: SimContract = SimContract {
///     writes:       names![Grid],
///     commands_in:  names![PlaceTile],
///     messages_out: names![TilePlaced],
///     ..SimContract::EMPTY
/// };
/// ```
#[macro_export]
macro_rules! names {
    [] => { &[] as &'static [$crate::core::TypeKey] };
    [$($t:ty),+ $(,)?] => {{
        const __NAMES: &[$crate::core::TypeKey] = &[
            $( $crate::core::TypeKey::new::<$t>(::core::stringify!($t)) ),+
        ];
        __NAMES
    }};
}
