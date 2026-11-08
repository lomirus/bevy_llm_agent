use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolCall};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game::{Character, Coordinate};

/// Set a movement destination. After this turn ends, you gradually move there over time and will not get another turn until you arrive. Other characters can still act while you move, so you may be devourable mid-path.
///
/// - Requirements: you are alive; distance to target >= 1; target coordinates within (-10, 10) on both axes.
/// - Note: other tools may still be called in the same turn after this succeeds. The "no further turns until arrival" rule applies after the turn finishes.
#[agent_tool(args = Args, output = Output, system = system)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct MoveTo;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    coordinate: Coordinate,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Output {
    is_success: bool,
    err_msg: Option<String>,
}

fn system(mut calls: MessageMutator<ToolCall<MoveTo>>, mut character_query: Query<&mut Character>) {
    for call in calls.read() {
        let mut character = character_query.get_mut(call.entity).unwrap();

        if character.is_dead {
            call.respond(Output {
                is_success: false,
                err_msg: Some("Dead characters cannot move".to_string()),
            })
            .unwrap();
            continue;
        }

        if call.args.coordinate.x <= -10.0 || call.args.coordinate.x >= 10.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some(
                    "The target's horizontal coordinate must be within (-10, 10)".to_string(),
                ),
            })
            .unwrap();
            continue;
        }

        if call.args.coordinate.y <= -10.0 || call.args.coordinate.y >= 10.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some(
                    "The target's vertical coordinate must be within (-10, 10)".to_string(),
                ),
            })
            .unwrap();
            continue;
        }

        let dist = character.coordinate.dist(&call.args.coordinate);
        if dist < 1.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some(format!("The distance between the current coordinates and the moving target ({dist}) is less than 1")),
            }).unwrap();
            continue;
        }

        character.moving_target = Some(call.args.coordinate.clone());
        call.respond(Output {
            is_success: true,
            err_msg: None,
        })
        .unwrap();
    }
}
