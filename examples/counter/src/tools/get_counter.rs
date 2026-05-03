use bevy::{ecs::system::BoxedSystem, prelude::*};
use bevy_llm_agent::tool::{JsonSchema, Tool, ToolRequest};
use serde::Deserialize;

use crate::Counter;

pub(crate) struct GetCounter;

/// Get the current value of counter.
#[derive(Deserialize, JsonSchema)]
pub(crate) struct GetCounterArgs {}

impl Tool for GetCounter {
    const NAME: &str = "get_counter";

    type Args = GetCounterArgs;
    type Output = usize;

    fn boxed_system() -> BoxedSystem {
        Box::new(IntoSystem::into_system(get_counter))
    }
}

fn get_counter(mut requests: MessageReader<ToolRequest<GetCounter>>, counter: Res<Counter>) {
    for request in requests.read() {
        request.send_back(counter.0);
    }
}
