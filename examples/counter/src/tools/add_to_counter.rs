use bevy::prelude::*;
use bevy_llm_agent::tool::{ToolTrait, ToolInvocation};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

#[derive(JsonSchema)]
/// Add an unsigned integer value to counter.
pub(crate) struct AddToCounter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    /// The unsigned integer value added to counter.
    increment: usize,
}

impl ToolTrait for AddToCounter {
    type Args = Args;
    type Output = ();
}

pub(crate) fn add_to_counter(
    mut calls: MessageReader<ToolInvocation<AddToCounter>>,
    mut counter: ResMut<Counter>,
) {
    for call in calls.read() {
        counter.0 += call.args.increment;
        call.respond(());
    }
}
