use std::any::TypeId;

/// Stable, type-unique identifier for a Rust type.
///
/// `id` is the authoritative key (compared for equality, hashed in maps).
/// `name` is diagnostic only: whatever the caller wrote at the construction
/// site. Two places writing `TypeKey::new::<Grid>("Grid")` and
/// `TypeKey::new::<Grid>("a::Grid")` produce equal keys - the string is a
/// hint for error messages, never a correctness key.
///
/// Build via the `names!` macro, which wires `stringify!` for `name`.
/// Do NOT call `TypeKey::new` directly from user code - use `names!`.
#[derive(Debug, Clone, Copy)]
pub struct TypeKey {
    pub id: TypeId,
    pub name: &'static str,
}

impl TypeKey {
    /// Construct a TypeKey. Intended to be called by the `names!` macro only.
    pub const fn new<T: 'static>(name: &'static str) -> TypeKey {
        TypeKey {
            id: TypeId::of::<T>(),
            name,
        }
    }

    /// Test whether this key identifies `T`.
    pub fn is<T: 'static>(&self) -> bool {
        self.id == TypeId::of::<T>()
    }
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeKey {}

impl std::hash::Hash for TypeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialOrd for TypeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
