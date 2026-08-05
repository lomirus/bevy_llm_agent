use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolCall};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Add an unsigned integer value to counter.
#[agent_tool(args = Args, output = (), system = add_to_counter)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct AddToCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    /// The unsigned integer value added to counter.
    increment: usize,
}

fn add_to_counter(mut calls: MessageMutator<ToolCall<AddToCounter>>, mut counter: ResMut<Counter>) {
    for call in calls.read() {
        counter.0 += call.args.increment;
        call.respond(()).unwrap();
    }
}
