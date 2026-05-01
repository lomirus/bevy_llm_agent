use bevy::{ecs::system::BoxedSystem, prelude::*};
use bevy_llm::tool::{Tool, ToolDefinition, ToolRequest};
use serde::Deserialize;
use serde_json::json;

use crate::Counter;

pub(crate) struct AddToCounter;

#[derive(Deserialize)]
pub(crate) struct AddToCounterArgs {
    increment: usize,
}

impl Tool for AddToCounter {
    const NAME: &str = "add_to_counter";

    type Args = AddToCounterArgs;
    type Output = ();

    fn definition() -> ToolDefinition {
        ToolDefinition {
            name: "add_to_counter".to_string(),
            description: "Add an unsigned integer value to counter".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "increment": {
                        "type": "integer",
                        "description": "The unsigned integer value added to counter",
                    }
                },
                "required": ["increment"],
                "additionalProperties": false
            }),
        }
    }

    fn boxed_system() -> BoxedSystem {
        Box::new(IntoSystem::into_system(add_to_counter))
    }
}

fn add_to_counter(
    mut requests: MessageReader<ToolRequest<AddToCounter>>,
    mut counter: ResMut<Counter>,
) {
    for request in requests.read() {
        counter.0 += request.increment;
        request.send_back(());
    }
}
