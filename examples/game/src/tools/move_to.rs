use bevy::prelude::*;
use bevy_llm_agent::tool::{Tool, ToolCall, ToolTrait};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Character, Coordinate};

/// Move to a specified location. After invoking this tool, the character sets the location as the movement target and gradually moves towards it over time.
///
/// - Requirement: Distance between your current position and target position >= 1. The target's horizontal and vertical coordinates must be within (-10, 10).
/// - Note: Subsequent tool calls can only be issued after the movement is completed. During the movement, the character is in an uncontrolled state. While moving, other characters can still act normally. Therefore, the character may be devoured by enemies during movement.
#[derive(SceneComponent, JsonSchema, Clone, Default)]
pub(crate) struct MoveTo;

impl MoveTo {
    fn scene() -> impl Scene {
        let tool = Tool::of::<Self>();
        bsn! {
            template_value(tool)
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    coordinate: Coordinate,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Output {
    is_success: bool,
    err_msg: Option<String>,
}

impl ToolTrait for MoveTo {
    type Args = Args;
    type Output = Output;

    fn boxed_system() -> bevy::ecs::system::BoxedSystem {
        Box::new(IntoSystem::into_system(system))
    }
}

fn system(mut calls: MessageMutator<ToolCall<MoveTo>>, mut character_query: Query<&mut Character>) {
    for call in calls.read() {
        let mut character = character_query.get_mut(call.entity).unwrap();

        if call.args.coordinate.x <= -10.0 || call.args.coordinate.x >= 10.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some("The target's horizontal coordinate must be within (-10, 10)".to_string()),
            }).unwrap();
            continue;
        }

        if call.args.coordinate.y <= -10.0 || call.args.coordinate.y >= 10.0 {
            call.respond(Output {
                is_success: false,
                err_msg: Some("The target's vertical coordinate must be within (-10, 10)".to_string()),
            }).unwrap();
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
