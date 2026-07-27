mod tools;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AppExt, DEEPSEEK_V4_FLASH, LlmAgentPlugin,
    agent::{Agent, Thinking},
    tool::AgentTools,
};
use rand::random_range;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tools::{DevourCharacter, GetCurrentInfo, MoveTo};

#[derive(Component, JsonSchema, Serialize, Clone, Default)]
struct Character {
    /// Character ID
    id: u8,
    /// Combat power.
    combat_power: u32,
    /// Moving speed.
    ///
    /// Unit: m/s.
    moving_speed: f32,
    /// Current coordinate of the character
    coordinate: Coordinate,
    /// Coordinate of the moving target
    moving_target: Option<Coordinate>,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default)]
struct Coordinate {
    /// Unit: m
    x: f32,
    /// Unit: m
    y: f32,
}

impl Coordinate {
    fn random() -> Self {
        Self {
            x: random_range(-10.0..10.0),
            y: random_range(-10.0..10.0),
        }
    }

    fn dist(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Resource)]
struct ElapsedTime(f32);

fn setup(mut commands: Commands) {
    for id in 1..=10 {
        commands.spawn_scene(bsn! {
            Character {
                id,
                combat_power: random_range::<u32, _>(1..=10000),
                moving_speed: random_range::<f32, _>(0.5..2.0),
                coordinate: Coordinate::random(),
                moving_target: None,
            }
            Agent::new(DEEPSEEK_V4_FLASH, Thinking::Off)
            AgentTools [
                @GetCurrentInfo,
                @MoveTo,
                @DevourCharacter,
            ]
        });
    }
}

fn main() {
    App::new()
        .insert_resource(ElapsedTime(0.0))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: DEFAULT_FILTER.to_owned(),
                ..default()
            }),
            LlmAgentPlugin,
        ))
        .add_agent_tool::<GetCurrentInfo>()
        .add_agent_tool::<MoveTo>()
        .add_agent_tool::<DevourCharacter>()
        .add_systems(Startup, setup)
        .run();
}
