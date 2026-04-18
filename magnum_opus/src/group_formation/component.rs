//! Group marker on the group entity itself, GroupMember on each Building.

use bevy::prelude::{Component, Entity};

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Group;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupMember {
    pub group: Entity,
}
