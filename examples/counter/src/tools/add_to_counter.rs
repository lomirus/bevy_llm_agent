use bevy::prelude::*;
use bevy_llm_agent::tool::ToolInvocation;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

#[derive(Deserialize, JsonSchema)]
/// Add an unsigned integer value to counter.
pub(crate) struct AddToCounter {
    /// The unsigned integer value added to counter.
    increment: usize,
}

pub(crate) fn add_to_counter(
    mut requests: MessageReader<ToolInvocation<AddToCounter, ()>>,
    mut counter: ResMut<Counter>,
) {
    for request in requests.read() {
        counter.0 += request.args.increment;
        request.send_back(());
    }
}
