use bevy::prelude::*;
use bevy_llm_agent::tool::{Tool, ToolCall, ToolTrait};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Counter;

/// Get the current value of counter.
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct GetCounter;

impl GetCounter {
    fn scene() -> impl Scene {
        let tool = Tool::of::<Self>();
        bsn! {
            template_value(tool)
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {}

impl ToolTrait for GetCounter {
    type Args = Args;
    type Output = usize;

    fn boxed_system() -> bevy::ecs::system::BoxedSystem {
        Box::new(IntoSystem::into_system(get_counter))
    }
}

fn get_counter(mut calls: MessageMutator<ToolCall<GetCounter>>, counter: Res<Counter>) {
    for call in calls.read() {
        call.respond(counter.0).unwrap();
    }
}
