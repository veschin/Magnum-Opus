//! InputUI module that declares `PlaceTile` as an emitted command.
//!
//! Producer counterpart for the grid module's `PlaceTile` drain. Required
//! so the core registry's `closed-commands` invariant has a registered
//! producer for every sink. F3 ships a no-op system; F21 fills the real
//! cursor-to-command translation.

use super::systems::placement_input_noop_system;
use crate::core::*;
use crate::grid::PlaceTile;
use crate::names;

pub struct PlacementInputModule;

impl InputUI for PlacementInputModule {
    const ID: &'static str = "placement_input";

    fn reads() -> &'static [TypeKey] {
        &[]
    }

    fn writes() -> &'static [TypeKey] {
        &[]
    }

    fn commands_out() -> &'static [TypeKey] {
        names![PlaceTile]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut InputInstaller) {
        ctx.emit_command::<PlaceTile>();
        ctx.add_system(placement_input_noop_system);
    }
}
