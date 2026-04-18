use bevy::prelude::Resource;

/// Typed command queue.
///
/// `InputUI` modules push commands; a `SimDomain` drains them in `Phase::Commands`.
/// One `CommandBus<T>` per command type.
#[derive(Resource)]
pub struct CommandBus<T: Send + Sync + 'static> {
    queue: Vec<T>,
}

impl<T: Send + Sync + 'static> Default for CommandBus<T> {
    fn default() -> Self {
        Self { queue: Vec::new() }
    }
}

impl<T: Send + Sync + 'static> CommandBus<T> {
    pub fn push(&mut self, cmd: T) {
        self.queue.push(cmd);
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, T> {
        self.queue.drain(..)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
