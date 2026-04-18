//! Placement input system stubs.
//!
//! F3 ships a no-op placeholder because nothing can translate mouse/cursor
//! events into `PlaceTile` commands yet (F21 adds camera-ui). The InputUI
//! contract requires at least the `emit_command` declaration; the actual
//! push logic will replace this stub when F21 lands.

pub fn placement_input_noop_system() {}
