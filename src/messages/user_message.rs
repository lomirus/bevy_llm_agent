use bevy::prelude::*;

#[derive(Message)]
pub struct UserMessage {
    pub entity: Entity,
    pub prompt: String,
}

impl UserMessage {
    pub fn new(entity: Entity, prompt: impl Into<String>) -> Self {
        UserMessage {
            entity,
            prompt: prompt.into(),
        }
    }
}
