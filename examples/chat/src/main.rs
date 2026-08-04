mod auto_scroll;
mod ui;

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    LlmAgentPlugin,
    agent::{Agent, Thinking},
    provider::deepseek,
};

use auto_scroll::AutoScrollToBottomPlugin;
use ui::{chat_ui, enable_send_button, update_chat_messages, use_system_font};

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn(Agent::new(deepseek::V4_FLASH, Thinking::Off).unwrap());
    commands.spawn_scene(chat_ui());
}

fn main() {
    // See issue: https://github.com/bevyengine/bevy/issues/24094
    const FILTER_ICU_PROVIDER: &str = "icu_provider::error=off";

    App::new()
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},{FILTER_ICU_PROVIDER}"),
                ..default()
            }),
            FeathersPlugins,
            LlmAgentPlugin,
            AutoScrollToBottomPlugin,
        ))
        .add_systems(Startup, (setup, use_system_font).chain())
        .add_systems(Update, (update_chat_messages, enable_send_button))
        .run();
}
