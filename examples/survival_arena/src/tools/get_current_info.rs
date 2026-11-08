use bevy::prelude::*;
use bevy_llm_agent::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

use crate::game::{Character, ElapsedTime};

/// Get current information: yourself, all other living characters, and elapsed time (seconds).
#[agent_tool]
pub(crate) struct GetCurrentInfo;

#[derive(JsonSchema, Serialize)]
pub(crate) struct Output {
    self_info: Character,
    characters_info: Vec<Character>,
    /// Unit: s
    elapsed_time: f32,
}

fn get_current_info(
    In(input): In<ToolInput>,
    characters: Query<&Character>,
    elapsed_time: Res<ElapsedTime>,
) -> Output {
    let self_info = characters.get(input.agent).unwrap().clone();
    let characters_info = characters
        .iter()
        .filter(|character| character.id != self_info.id && !character.is_dead)
        .cloned()
        .collect();
    Output {
        self_info,
        characters_info,
        elapsed_time: elapsed_time.0,
    }
}
