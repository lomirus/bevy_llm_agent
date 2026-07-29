use bevy::prelude::*;
use bevy_llm_agent::tool::{Tool, ToolCall, ToolTrait};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game::{Character, ElapsedTime};

#[derive(SceneComponent, JsonSchema, Clone, Default)]
/// Get current character and environment information.
pub(crate) struct GetCurrentInfo;

impl GetCurrentInfo {
    fn scene() -> impl Scene {
        let tool = Tool::of::<Self>();
        bsn! {
            template_value(tool)
        }
    }
}

#[derive(JsonSchema, Deserialize)]
pub(crate) struct Args {}

#[derive(JsonSchema, Serialize)]
pub(crate) struct Output {
    self_info: Character,
    characters_info: Vec<Character>,
    /// Unit: s
    elapsed_time: f32,
}

impl ToolTrait for GetCurrentInfo {
    type Args = Args;
    type Output = Output;

    fn boxed_system() -> bevy::ecs::system::BoxedSystem {
        Box::new(IntoSystem::into_system(system))
    }
}

fn system(
    mut calls: MessageMutator<ToolCall<GetCurrentInfo>>,
    character_query: Query<&Character>,
    elapsed_time: Res<ElapsedTime>,
) {
    for call in calls.read() {
        let self_info = (*character_query.get(call.entity).unwrap()).clone();
        let characters_info: Vec<_> = character_query
            .iter()
            .filter(|char| char.id != self_info.id && !char.is_dead)
            .map(|char| (*char).clone())
            .collect();
        let elapsed_time = elapsed_time.0;
        call.respond(Output {
            self_info,
            characters_info,
            elapsed_time,
        })
        .unwrap();
    }
}
