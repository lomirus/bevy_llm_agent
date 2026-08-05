use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolCall};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Get the current value of counter.
#[agent_tool(args = Args, output = usize, system = get_counter)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct GetCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {}

fn get_counter(mut calls: MessageMutator<ToolCall<GetCounter>>, counter: Res<Counter>) {
    for call in calls.read() {
        call.respond(counter.0).unwrap();
    }
}
