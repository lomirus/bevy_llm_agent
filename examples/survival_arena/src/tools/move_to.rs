use bevy::prelude::*;
use bevy_llm_agent::{agent_tool, tool::ToolInput};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::game::{Character, Coordinate};

/// Set a movement destination. After this turn ends, you gradually move there over time and will not get another turn until you arrive. Other characters can still act while you move, so you may be devourable mid-path.
///
/// - Requirements: you are alive; distance to target >= 1; target coordinates within (-10, 10) on both axes.
/// - Note: other tools may still be called in the same turn after this succeeds. The "no further turns until arrival" rule applies after the turn finishes.
#[agent_tool(system = move_to)]
pub(crate) struct MoveTo;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    coordinate: Coordinate,
}

fn move_to(
    In(ToolInput { agent, args }): In<ToolInput<Args>>,
    mut characters: Query<&mut Character>,
) -> Result<(), String> {
    let mut character = characters.get_mut(agent).unwrap();
    let target = args.coordinate;

    if character.is_dead {
        return Err("Dead characters cannot move".to_owned());
    }
    if target.x <= -10.0 || target.x >= 10.0 {
        return Err("The target's horizontal coordinate must be within (-10, 10)".to_owned());
    }
    if target.y <= -10.0 || target.y >= 10.0 {
        return Err("The target's vertical coordinate must be within (-10, 10)".to_owned());
    }
    let dist = character.coordinate.dist(&target);
    if dist < 1.0 {
        return Err(format!(
            "The distance between the current coordinates and the moving target ({dist}) is less than 1"
        ));
    }

    character.moving_target = Some(target);
    Ok(())
}
