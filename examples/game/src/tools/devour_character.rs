use bevy::prelude::*;
use bevy_llm_agent::tool::{Tool, ToolCall, ToolTrait};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Character;

/// Devour the target character.
/// 
/// - Requirements: The character's combat power must be greater than the target's combat power, and the distance between the character and the target must be less than 1.
/// - Success: The target is eliminated from the game. The character's new combat power becomes the sum of its original combat power and the target's original combat power.
/// - Failure: None.
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct DevourCharacter;

impl DevourCharacter {
    fn scene() -> impl Scene {
        let tool = Tool::of::<Self>();
        bsn! {
            template_value(tool)
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    target_id: u8,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Output {
    is_success: bool,
    err_msg: Option<String>,
}

impl ToolTrait for DevourCharacter {
    type Args = Args;
    type Output = Output;

    fn boxed_system() -> bevy::ecs::system::BoxedSystem {
        Box::new(IntoSystem::into_system(get_counter))
    }
}

fn get_counter(
    mut calls: MessageMutator<ToolCall<DevourCharacter>>,
    character_query: Query<&Character>,
) {
    for call in calls.read() {
        let self_character = character_query.get(call.entity).unwrap();

        let Some(target_character) = character_query
            .iter()
            .find(|char| char.id == call.args.target_id)
        else {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Non-existent target ID".to_string()),
            })
            .unwrap();
            continue;
        };
    
        let self_combat_power = self_character.combat_power;
        let target_combat_power = target_character.combat_power;
        if self_character.combat_power < target_character.combat_power {
            call.respond(Output {
                is_success: false,
                err_msg: Some(format!("Your combat power ({self_combat_power}) should be higher than that of the target ({target_combat_power})")),
            })
            .unwrap();
            continue;
        }

        let distance = self_character.coordinate.dist(&target_character.coordinate);
        if distance >= 1.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some(format!("The distance to the target should be less than 1, but the current distance is {distance}")),
            })
            .unwrap();
            continue;
        }

        call.respond(Output {
            is_success: true,
            err_msg: None,
        }).unwrap();
    }
}
