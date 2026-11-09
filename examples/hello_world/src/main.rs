use bevy::prelude::*;
use bevy_llm_agent::{prelude::*, provider::deepseek};

fn setup(mut commands: Commands) {
    let agent = Agent::builder(deepseek::FLASH)
        .thinking(deepseek::Thinking::Off)
        .build()
        .unwrap();
    commands.spawn(Camera2d);
    commands
        .spawn((
            Text::default(),
            agent,
            TextFont::from(FontSource::system_ui()),
        ))
        .prompt("Hello!");
}

fn update_text(mut texts: Query<&mut Text>, mut agent_messages: MessageReader<AgentMessage>) {
    for AgentMessage { entity, delta } in agent_messages.read() {
        let mut text = texts.get_mut(*entity).unwrap();
        if let AgentMessageDelta::Content(content) = delta {
            text.push_str(content)
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LlmAgentPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update_text)
        .run();
}
