pub use rig::completion::ToolDefinition;
pub use rig::tool::Tool as RigTool;
pub use rig::tool::ToolError;
pub use schemars::JsonSchema;

use bevy::{ecs::system::BoxedSystem, prelude::*};
use serde::{Serialize, de::DeserializeOwned};
use std::{any::type_name, fmt::Debug, sync::Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub trait AppExt {
    fn register_llm_tool<T: Tool>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn register_llm_tool<T: Tool>(&mut self) -> &mut Self {
        // Validate the generated schema on the main thread during app setup.
        T::definition();

        let (sender, receiver) = unbounded_channel();

        self.add_message::<ToolRequest<T>>()
            .insert_resource(ToolAdapter::<T> { sender })
            .insert_resource(ToolRequestInbox::<T> { receiver })
            .add_systems(FixedUpdate, T::boxed_system())
            .add_systems(FixedUpdate, poll_tool_requests::<T>)
    }
}

pub trait Tool: 'static + Sync + Send {
    const NAME: &'static str;

    type Args: Send + Sync + 'static + DeserializeOwned + JsonSchema;
    type Output: Send + Serialize + Debug;

    fn definition() -> ToolDefinition {
        let parameters = serde_json::json!(schemars::schema_for!(Self::Args));
        let description = parameters
            .get("description")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "tool `{}` args type `{}` is missing a schema description; add a doc comment to the args struct",
                    Self::NAME,
                    type_name::<Self::Args>()
                )
            })
            .to_string();

        ToolDefinition {
            name: Self::NAME.to_string(),
            description,
            parameters,
        }
    }

    fn boxed_system() -> BoxedSystem;
}

#[derive(Resource)]
pub struct ToolAdapter<T: Tool> {
    sender: UnboundedSender<ToolRequest<T>>,
}

impl<T: Tool> Clone for ToolAdapter<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
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

#[derive(Message, Deref)]
pub struct ToolRequest<T: Tool> {
    #[deref]
    args: T::Args,
    output_sender: Mutex<Option<tokio::sync::oneshot::Sender<T::Output>>>,
}

impl<T: Tool> ToolRequest<T> {
    pub fn send_back(&self, output: T::Output) {
        self.output_sender
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(output)
            .unwrap();
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
