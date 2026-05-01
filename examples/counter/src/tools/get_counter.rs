use bevy::{ecs::system::BoxedSystem, prelude::*};
use bevy_llm::tool::{Tool, ToolDefinition, ToolRequest};
use serde::Deserialize;
use serde_json::json;

use crate::Counter;

pub(crate) struct GetCounter;

#[derive(Deserialize)]
pub(crate) struct GetCounterArgs {}

impl Tool for GetCounter {
    const NAME: &str = "get_counter";

    type Args = GetCounterArgs;
    type Output = usize;

    fn definition() -> ToolDefinition {
        ToolDefinition {
            name: "get_counter".to_string(),
            description: "Get the current value of counter".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn boxed_system() -> BoxedSystem {
        Box::new(IntoSystem::into_system(get_counter))
    }
}

fn get_counter(mut requests: MessageReader<ToolRequest<GetCounter>>, counter: Res<Counter>) {
    for request in requests.read() {
        request.send_back(counter.0);
    }
}
