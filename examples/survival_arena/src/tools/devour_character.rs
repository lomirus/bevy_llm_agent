use bevy::prelude::*;
use bevy_llm_agent::prelude::*;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::game::Character;

/// Devour the target character by id.
///
/// - Requirements: you are alive; target exists, is alive, and is not yourself; your combat power >= target's; distance < 1.
/// - Success: target is eliminated; your combat power becomes the sum of both.
#[agent_tool]
pub(crate) struct DevourCharacter;

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Args {
    target_id: u8,
}

fn devour_character(
    In(ToolInput { agent, args }): In<ToolInput<Args>>,
    mut characters: Query<(Entity, &mut Character)>,
) -> Result<(), String> {
    let (_, self_character) = characters.get(agent).unwrap();
    if self_character.is_dead {
        return Err("Dead characters cannot act".to_owned());
    }

    let Some((target_entity, target_character)) = characters
        .iter()
        .find(|(_, character)| character.id == args.target_id)
    else {
        return Err("Non-existent target ID".to_owned());
    };
    if target_entity == agent {
        return Err("Cannot devour yourself".to_owned());
    }
    if target_character.is_dead {
        return Err("Target is already dead".to_owned());
    }

    let self_combat_power = self_character.combat_power;
    let target_combat_power = target_character.combat_power;
    if self_combat_power < target_combat_power {
        return Err(format!(
            "Your combat power ({self_combat_power}) should be higher than that of the target ({target_combat_power})"
        ));
    }

    let distance = self_character.coordinate.dist(&target_character.coordinate);
    if distance >= 1.0 {
        return Err(format!(
            "The distance to the target should be less than 1, but the current distance is {distance}"
        ));
    }

    let [(_, mut self_character), (_, mut target_character)] =
        characters.get_many_mut([agent, target_entity]).unwrap();
    target_character.is_dead = true;
    target_character.moving_target = None;
    self_character.combat_power += target_combat_power;
    Ok(())
}
