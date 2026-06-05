use bevy::prelude::*;
use rig::{completion::ToolDefinition, tool::ToolError};
use std::{any::TypeId, sync::Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::tool::{TOOL_CALL_SENDERS, Tool, ToolRequest};

pub(crate) struct ToolAdapter<T: Tool> {
    sender: UnboundedSender<ToolRequest<T>>,
}

impl<T: Tool> ToolAdapter<T> {
    pub(crate) fn new() -> Self {
        let tool_call_senders = TOOL_CALL_SENDERS.lock().unwrap();
        let sender = tool_call_senders.get(&TypeId::of::<T>()).unwrap();
        let sender = sender
            .downcast_ref::<UnboundedSender<ToolRequest<T>>>()
            .unwrap()
            .clone();

        Self { sender }
    }
}

impl<T: Tool> rig::tool::Tool for ToolAdapter<T> {
    const NAME: &str = T::NAME;

    type Error = ToolError;
    type Args = T::Args;
    type Output = T::Output;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        T::definition()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (output_sender, output_receiver) = tokio::sync::oneshot::channel();

        self.sender
            .send(ToolRequest {
                args,
                output_sender: Mutex::new(Some(output_sender)),
            })
            .unwrap();

        let output = output_receiver.await.unwrap();

        Ok(output)
    }
}
