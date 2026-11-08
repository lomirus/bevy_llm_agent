use bevy::prelude::*;
use bevy_llm_agent::prelude::*;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Add an unsigned integer value to counter.
#[agent_tool]
pub(crate) struct AddToCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    /// The unsigned integer value added to counter.
    increment: usize,
}

fn add_to_counter(In(input): In<ToolInput<Args>>, mut counter: ResMut<Counter>) {
    counter.0 += input.args.increment;
}
