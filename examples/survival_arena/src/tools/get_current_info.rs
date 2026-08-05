use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolCall};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game::{Character, ElapsedTime};

/// Get current information: yourself, all other living characters, and elapsed time (seconds).
#[agent_tool(args = Args, output = Output, system = system)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct GetCurrentInfo;

#[derive(JsonSchema, Deserialize)]
pub(crate) struct Args {}

#[derive(JsonSchema, Serialize)]
pub(crate) struct Output {
    self_info: Character,
    characters_info: Vec<Character>,
    /// Unit: s
    elapsed_time: f32,
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
