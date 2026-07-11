use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AssistantContent, LlmAgentPlugin, MultiTurnItem, agent::AgentBuilder, prelude::*,
};

fn setup(mut commands: Commands) {
    let mut agent = AgentBuilder::new(DEEPSEEK_V4_FLASH).build();
    agent.streaming_chat("Hello!");
    commands.spawn(Camera2d);
    commands.spawn((Text::default(), agent, TextFont::from(FontSource::SystemUi)));
}

fn update_text(
    mut texts: Query<&mut Text>,
    mut stream_messages: MessageReader<bevy_llm_agent::AgentMessage>,
) {
    for stream_message in stream_messages.read() {
        let mut text = texts.get_mut(stream_message.entity).unwrap();
        if let MultiTurnItem::StreamAssistantItem(AssistantContent::Text(delta)) =
            &stream_message.delta
        {
            text.push_str(delta.text())
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
