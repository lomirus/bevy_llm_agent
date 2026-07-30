use bevy::prelude::*;

use crate::tool::ToolTrait;

#[derive(Message)]
pub struct ToolCall<T: ToolTrait> {
    pub args: T::Args,
    pub entity: Entity,
    pub(crate) responder: Option<oneshot::Sender<String>>,
}

impl<T: ToolTrait> ToolCall<T> {
    pub fn respond(&mut self, output: T::Output) -> Result<(), BevyError> {
        let output = serde_json::to_string(&output)?;
        let sender = self
            .responder
            .take()
            .ok_or("tool call has already been responded")?;
        sender.send(output)?;
        Ok(())
    }
}
