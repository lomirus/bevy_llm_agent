use bevy::{prelude::*, ui_widgets::Activate};
use bevy_llm_agent::messages::{
    agent_message::{AgentMessage, AgentMessageDelta},
    user_message::UserMessage,
};
use rand::random_range;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONTINUE_ACTION_PROMPT: &str = "Take your next action now. You must call tools to act.";

#[derive(Component, JsonSchema, Serialize, Clone, Default)]
pub(crate) struct Character {
    /// Character ID
    pub(crate) id: u8,
    /// Combat power.
    pub(crate) combat_power: u32,
    /// Moving speed.
    ///
    /// Unit: m/s.
    pub(crate) moving_speed: f32,
    /// Current coordinate of the character
    pub(crate) coordinate: Coordinate,
    /// Coordinate of the moving target
    pub(crate) moving_target: Option<Coordinate>,
    /// Whether the character has been eliminated.
    pub(crate) is_dead: bool,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Coordinate {
    /// Unit: m
    pub(crate) x: f32,
    /// Unit: m
    pub(crate) y: f32,
}

impl Coordinate {
    pub(crate) fn random() -> Self {
        Self {
            x: random_range(-10.0..10.0),
            y: random_range(-10.0..10.0),
        }
    }

    pub(crate) fn dist(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Resource)]
pub(crate) struct ElapsedTime(pub(crate) f32);

/// World clock only advances in [`GamePhase::Moving`].
#[derive(Resource, Clone, PartialEq, Eq, Default)]
pub(crate) enum GamePhase {
    /// Waiting for the player to click Continue Dialogue.
    #[default]
    AwaitingContinue,
    /// Idle characters were prompted; wait until every character has a move target
    /// (or is dead) and all prompted agent outputs have finished.
    AwaitingMoves,
    /// Targets are set; coordinates advance with real time.
    Moving,
}

/// Characters prompted this round whose agent output has not finished yet.
#[derive(Resource, Default)]
pub(crate) struct AwaitingAgentOutputs(pub(crate) Vec<Entity>);

pub(crate) fn on_continue_dialogue(
    _activate: On<Activate>,
    mut phase: ResMut<GamePhase>,
    mut awaiting_outputs: ResMut<AwaitingAgentOutputs>,
    characters: Query<(Entity, &Character)>,
    mut sender: MessageWriter<UserMessage>,
) {
    if *phase != GamePhase::AwaitingContinue {
        return;
    }

    let mut idle: Vec<_> = characters
        .iter()
        .filter(|(_, character)| !character.is_dead && character.moving_target.is_none())
        .collect();
    if idle.is_empty() {
        return;
    }
    idle.sort_by_key(|(_, character)| character.id);

    awaiting_outputs.0 = idle.iter().map(|(entity, _)| *entity).collect();
    for (entity, _) in &idle {
        sender.write(UserMessage::new(*entity, CONTINUE_ACTION_PROMPT));
    }
    *phase = GamePhase::AwaitingMoves;
}

pub(crate) fn track_awaiting_agent_outputs(
    mut awaiting_outputs: ResMut<AwaitingAgentOutputs>,
    mut messages: MessageReader<AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for AgentMessage { entity, delta } in messages.read() {
        match delta {
            Ok(AgentMessageDelta::Finish) => {
                awaiting_outputs.0.retain(|&e| e != *entity);
            }
            Ok(_) => {}
            Err(err) => {
                error!("agent {entity} failed: {err:?}");
                app_exit.write(AppExit::error());
            }
        }
    }
}

pub(crate) fn check_awaiting_moves(
    mut phase: ResMut<GamePhase>,
    awaiting_outputs: Res<AwaitingAgentOutputs>,
    characters: Query<&Character>,
) {
    if *phase != GamePhase::AwaitingMoves {
        return;
    }
    let targets_ready = characters
        .iter()
        .all(|character| character.is_dead || character.moving_target.is_some());
    if targets_ready && awaiting_outputs.0.is_empty() {
        *phase = GamePhase::Moving;
    }
}

pub(crate) fn advance_world(
    time: Res<Time>,
    mut phase: ResMut<GamePhase>,
    mut elapsed: ResMut<ElapsedTime>,
    mut characters: Query<&mut Character>,
) {
    if *phase != GamePhase::Moving {
        return;
    }

    let dt = time.delta_secs();
    elapsed.0 += dt;

    let mut anyone_arrived = false;
    for mut character in &mut characters {
        if character.is_dead {
            continue;
        }
        let target = character.moving_target.clone().unwrap();
        let dist = character.coordinate.dist(&target);
        let step = character.moving_speed * dt;
        if step >= dist {
            character.coordinate = target;
            character.moving_target = None;
            anyone_arrived = true;
        } else {
            let t = step / dist;
            character.coordinate.x += (target.x - character.coordinate.x) * t;
            character.coordinate.y += (target.y - character.coordinate.y) * t;
        }
    }

    if anyone_arrived {
        *phase = GamePhase::AwaitingContinue;
    }
}
