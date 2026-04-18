use bevy::prelude::Resource;

/// Crate-sealed token inserted by `CorePlugin::build` and transitioned by
/// `AppExt::finalize_modules`. The startup-time guard reads it to decide
/// whether cross-module invariants were checked.
///
/// User code CANNOT construct `CoreSeal` because `_private: ()` is private.
/// Swapping `ModuleRegistry` via `world.insert_resource(fake_finalized)` does
/// not touch `CoreSeal`, so the finalize guard cannot be bypassed by replacing
/// the registry with a pre-finalized empty copy.
#[derive(Resource)]
pub struct CoreSeal {
    finalized: bool,
    _private: (),
}

impl CoreSeal {
    pub(crate) fn new() -> Self {
        Self {
            finalized: false,
            _private: (),
        }
    }

    pub(crate) fn set_finalized(&mut self) {
        self.finalized = true;
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }
}
