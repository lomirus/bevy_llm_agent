use bevy::prelude::*;
use bevy_llm_agent::tool::{ToolTrait, ToolInvocation};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Get the current value of counter.
#[derive(JsonSchema)]
pub(crate) struct GetCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {}

impl ToolTrait for GetCounter {
    type Args = Args;
    type Output = usize;
}

pub(crate) fn get_counter(
    mut calls: MessageReader<ToolInvocation<GetCounter>>,
    counter: Res<Counter>,
) {
    for call in calls.read() {
        call.respond(counter.0);
    }
}
