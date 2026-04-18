use bevy::prelude::{ResMut, Resource};

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tick(pub u64);

pub fn tick_increment_system(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}
