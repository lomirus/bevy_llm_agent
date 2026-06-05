mod app_ext;
mod tool_adapter;

pub use app_ext::AppExt;
pub use rig::completion::ToolDefinition;
pub use rig::tool::Tool as RigTool;
pub use rig::tool::ToolError;
pub use schemars::JsonSchema;

pub(crate) use tool_adapter::ToolAdapter;

use bevy::{ecs::system::BoxedSystem, platform::collections::HashMap, prelude::*};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::{Any, TypeId, type_name},
    fmt::Debug,
    sync::{LazyLock, Mutex},
};

pub(crate) static TOOL_CALL_SENDERS: LazyLock<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
