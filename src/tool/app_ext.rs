use bevy::prelude::*;
use std::any::TypeId;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::tool::TOOL_CALL_SENDERS;
use crate::tool::Tool;
use crate::tool::ToolRequest;

pub trait AppExt {
    fn register_llm_tool<T: Tool>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn register_llm_tool<T: Tool>(&mut self) -> &mut Self {
        // Validate the generated schema on the main thread during app setup.
        T::definition();

        let (sender, receiver) = unbounded_channel();

        TOOL_CALL_SENDERS
            .lock()
            .unwrap()
            .insert(TypeId::of::<T>(), Box::new(sender));

        self.add_message::<ToolRequest<T>>()
            .insert_resource(ToolRequestInbox::<T> { receiver })
            .add_systems(FixedUpdate, T::boxed_system())
            .add_systems(FixedUpdate, poll_tool_requests::<T>)
    }
}

#[derive(Resource)]
struct ToolRequestInbox<T: Tool> {
    receiver: UnboundedReceiver<ToolRequest<T>>,
}

fn poll_tool_requests<T: Tool>(
    mut tool_request: ResMut<ToolRequestInbox<T>>,
    mut message_writer: MessageWriter<ToolRequest<T>>,
) {
    while let Ok(tool_request) = tool_request.receiver.try_recv() {
        message_writer.write(tool_request);
    }
}
