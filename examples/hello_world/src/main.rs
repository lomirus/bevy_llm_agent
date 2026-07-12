use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AgentMessage, AgentMessageDelta, DEEPSEEK_V4_FLASH, LlmAgentPlugin, UserMessage,
    agent::{Agent, Thinking},
};

fn setup(mut commands: Commands, mut sender: MessageWriter<UserMessage>) {
    let agent = Agent::new(DEEPSEEK_V4_FLASH, Thinking::Off);
    commands.spawn(Camera2d);
    let entity = commands
        .spawn((Text::default(), agent, TextFont::from(FontSource::SystemUi)))
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
    // See issue: https://github.com/bevyengine/bevy/issues/22733
    const FILTER_WGPU_HAL: &str = "wgpu_hal::vulkan::instance=off";
    // See issue: https://github.com/bevyengine/bevy/issues/24094
    const FILTER_ICU_PROVIDER: &str = "icu_provider::error=off";

    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},{FILTER_WGPU_HAL},{FILTER_ICU_PROVIDER}"),
                ..default()
            }),
            LlmAgentPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_text)
        .run();
}
