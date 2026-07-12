use bevy::prelude::*;
use bevy_llm_agent::tool::ToolInvocation;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Get the current value of counter.
#[derive(Deserialize, JsonSchema)]
pub(crate) struct GetCounter {}

pub(crate) fn get_counter(
    mut requests: MessageReader<ToolInvocation<GetCounter, usize>>,
    counter: Res<Counter>,
) {
    for request in requests.read() {
        request.send_back(counter.0);
    }
}
