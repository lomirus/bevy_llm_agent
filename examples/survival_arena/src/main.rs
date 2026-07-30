mod game;
mod tools;
mod ui;

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{AppExt, LlmAgentPlugin};

use game::{AwaitingAgentOutputs, ElapsedTime, GamePhase};
use tools::{DevourCharacter, GetCurrentInfo, MoveTo};
use ui::{DialogueView, SelectedCharacter};

fn main() {
    App::new()
        .init_resource::<SelectedCharacter>()
        .init_resource::<DialogueView>()
        .init_resource::<GamePhase>()
        .init_resource::<AwaitingAgentOutputs>()
        .insert_resource(ElapsedTime(0.0))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: DEFAULT_FILTER.to_owned(),
                ..default()
            }),
            FeathersPlugins,
            MeshPickingPlugin,
            LlmAgentPlugin,
            ui::AutoScrollToBottomPlugin,
        ))
        .add_agent_tool::<GetCurrentInfo>()
        .add_agent_tool::<MoveTo>()
        .add_agent_tool::<DevourCharacter>()
        .add_systems(
            Startup,
            (
                ui::setup,
                ApplyDeferred,
                ui::attach_map_viewport,
                ui::select_strongest_character,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                ui::update_dialogue_panel,
                ui::update_character_detail,
                ui::update_character_sprites,
                (
                    game::track_awaiting_agent_outputs,
                    game::check_awaiting_moves,
                )
                    .chain(),
                game::advance_world,
                ui::sync_continue_button,
            ),
        )
        .run();
}
