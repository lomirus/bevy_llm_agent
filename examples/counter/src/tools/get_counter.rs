use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolInput};

use crate::Counter;

/// Get the current value of counter.
#[agent_tool(system = get_counter)]
pub(crate) struct GetCounter;

fn get_counter(_: In<ToolInput>, counter: Res<Counter>) -> usize {
    counter.0
}
