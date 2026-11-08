use bevy::prelude::*;
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::tool::ToolInput;

/// Runs a tool against the world and returns the result sent back to the model.
pub(crate) type ToolFn = Arc<dyn Fn(&mut World, Entity, &str) -> String + Send + Sync>;

/// A tool an agent can call, attached to the agent through [`AgentTools`](crate::tool::AgentTools).
///
/// Usually created by [`agent_tool`](crate::agent_tool).
#[derive(Clone, Component)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub(crate) call: ToolFn,
}

impl Default for Tool {
    // See issue: https://github.com/bevyengine/bevy/issues/24739
    fn default() -> Self {
        Tool {
            name: String::new(),
            description: String::new(),
            parameters: serde_json::Value::Null,
            call: Arc::new(|_, _, _| String::new()),
        }
    }
}

impl Tool {
    /// Create a tool backed by `system`.
    ///
    /// The system receives the model's arguments through [`ToolInput`], and its output is
    /// serialized to JSON and sent back to the model.
    pub fn new<A, O, M, S>(
        name: impl Into<String>,
        description: impl Into<String>,
        system: S,
    ) -> Self
    where
        A: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static,
        M: 'static,
        S: IntoSystem<In<ToolInput<A>>, O, M> + Copy + Send + Sync + 'static,
    {
        Tool {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::to_value(schemars::schema_for!(A)).unwrap(),
            call: Arc::new(move |world, agent, raw_args| run(world, system, agent, raw_args)),
        }
    }
}

fn run<A, O, M, S>(world: &mut World, system: S, agent: Entity, raw_args: &str) -> String
where
    A: DeserializeOwned + 'static,
    O: Serialize + 'static,
    S: IntoSystem<In<ToolInput<A>>, O, M> + 'static,
{
    // Some models send empty arguments for tools without parameters.
    let raw_args = if raw_args.trim().is_empty() {
        "{}"
    } else {
        raw_args
    };
    let args = match serde_json::from_str(raw_args) {
        Ok(args) => args,
        Err(error) => return format!("Error: invalid arguments: {error}"),
    };
    match world.run_system_cached_with(system, ToolInput { agent, args }) {
        Ok(output) => serde_json::to_string(&output)
            .unwrap_or_else(|error| format!("Error: failed to serialize the result: {error}")),
        Err(error) => format!("Error: {error}"),
    }
}
