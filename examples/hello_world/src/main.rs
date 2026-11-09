use bevy::prelude::*;
use bevy_llm_agent::{prelude::*, provider::deepseek};

fn setup(mut commands: Commands, mut sender: MessageWriter<UserMessage>) {
    let agent = Agent::builder(deepseek::FLASH)
        .thinking(deepseek::Thinking::Off)
        .build()
        .unwrap();
    commands.spawn(Camera2d);
    let entity = commands
        .spawn((
            Text::default(),
            agent,
            TextFont::from(FontSource::system_ui()),
        ))
        .id();
    sender.write(UserMessage::new(entity, "Hello!"));
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
        .add_systems(FixedUpdate, update_text)
        .run();
}
