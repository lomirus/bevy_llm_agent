use bevy::prelude::*;

use crate::tool::ToolTrait;

#[derive(Message)]
pub struct ToolInvocation<T: ToolTrait> {
    pub args: T::Args,
    pub(crate) responder: Option<oneshot::Sender<String>>,
}

impl<T: ToolTrait> ToolInvocation<T> {
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
