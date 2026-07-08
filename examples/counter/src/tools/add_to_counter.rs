use bevy::{ecs::system::BoxedSystem, prelude::*};
use bevy_llm_agent::tool::{Tool, ToolRequest};
use serde::Deserialize;
use schemars::JsonSchema;

use crate::Counter;

/// Add an unsigned integer value to counter.
#[derive(JsonSchema)]
pub(crate) struct AddToCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct AddToCounterArgs {
    /// The unsigned integer value added to counter.
    increment: usize,
}

impl Tool for AddToCounter {
    const NAME: &str = "add_to_counter";

    type Args = AddToCounterArgs;
    type Output = ();

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
