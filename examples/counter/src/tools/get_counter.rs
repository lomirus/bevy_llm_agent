use bevy::prelude::*;
use bevy_llm_agent::prelude::*;

use crate::Counter;

/// Get the current value of counter.
#[agent_tool]
pub(crate) struct GetCounter;

fn get_counter(_: In<ToolInput>, counter: Res<Counter>) -> usize {
    counter.0
}
