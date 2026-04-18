use bevy::ecs::message::Message;

#[derive(Message, Debug, Clone, Copy)]
pub struct VeinsGenerated {
    pub count: u32,
}
