use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolCall};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game::Character;

/// Devour the target character by id.
///
/// - Requirements: you are alive; target exists, is alive, and is not yourself; your combat power >= target's; distance < 1.
/// - Success: target is eliminated; your combat power becomes the sum of both.
#[agent_tool(args = Args, output = Output, system = system)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct DevourCharacter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    target_id: u8,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Output {
    is_success: bool,
    err_msg: Option<String>,
}

fn system(
    mut calls: MessageMutator<ToolCall<DevourCharacter>>,
    mut characters: Query<(Entity, &mut Character)>,
) {
    for call in calls.read() {
        let self_entity = call.entity;
        let self_character = characters.get(self_entity).unwrap().1;

        if self_character.is_dead {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Dead characters cannot act".to_string()),
            })
            .unwrap();
            continue;
        }

        let Some(target_entity) = characters
            .iter()
            .find(|(_, character)| character.id == call.args.target_id)
            .map(|(entity, _)| entity)
        else {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Non-existent target ID".to_string()),
            })
            .unwrap();
            continue;
        };

        if target_entity == self_entity {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Cannot devour yourself".to_string()),
            })
            .unwrap();
            continue;
        }

        let target_character = characters.get(target_entity).unwrap().1;
        if target_character.is_dead {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Target is already dead".to_string()),
            })
            .unwrap();
            continue;
        }

        let self_combat_power = self_character.combat_power;
        let target_combat_power = target_character.combat_power;
        if self_combat_power < target_combat_power {
            call.respond(Output {
                is_success: false,
                err_msg: Some(format!(
                    "Your combat power ({self_combat_power}) should be higher than that of the target ({target_combat_power})"
                )),
            })
            .unwrap();
            continue;
        }

        let distance = self_character.coordinate.dist(&target_character.coordinate);
        if distance >= 1.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some(format!(
                    "The distance to the target should be less than 1, but the current distance is {distance}"
                )),
            })
            .unwrap();
            continue;
        }

        let [(_, mut self_character), (_, mut target_character)] = characters
            .get_many_mut([self_entity, target_entity])
            .unwrap();
        target_character.is_dead = true;
        target_character.moving_target = None;
        self_character.combat_power += target_combat_power;

        call.respond(Output {
            is_success: true,
            err_msg: None,
        })
        .unwrap();
    }
}
