use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{ButtonVariant, FeathersButton, FeathersScrollbar},
        theme::ThemedText,
    },
    prelude::*,
    ui::{InteractionDisabled, Overflow},
    ui_widgets::{ControlOrientation, ScrollArea},
};
use bevy_llm_agent::prelude::*;

use crate::game::{Character, GamePhase, on_continue_dialogue};

use super::{UI_FONT_SIZE, auto_scroll::AutoScrollToBottom, character::SelectedCharacter};

#[derive(Component, Clone, Default)]
pub(crate) struct ContinueDialogueButton;

#[derive(Component, Clone, Default)]
pub(crate) struct DialogueHeader;

#[derive(Component, Clone, Default)]
pub(crate) struct DialogueMessageList;

#[derive(Component, Clone, Default)]
pub(crate) struct DialogueMessageTitle;

#[derive(Component, Clone, Default)]
pub(crate) struct DialogueBody;

#[derive(Component, Clone, Copy, Default)]
pub(crate) struct DialogueMessageIndex(usize);

#[derive(Component, Clone, Default)]
pub(crate) struct DialogueScrollArea;

/// Which character's dialog is currently mirrored into the message list.
#[derive(Resource, Default)]
pub(crate) struct DialogueView {
    character: Option<Entity>,
    message_count: usize,
}

fn dialog_entries(dialog: &[DialogMessage]) -> Vec<(&'static str, String)> {
    let mut entries = Vec::new();
    let mut tool_calls = Vec::new();
    for message in dialog {
        match message {
            DialogMessage::System { content } => {
                entries.push(("System", content.clone()));
            }
            DialogMessage::User { content } => {
                entries.push(("User", content.clone()));
            }
            DialogMessage::Assistant {
                content,
                reasoning_content,
                tool_calls: calls,
            } => {
                if !reasoning_content.is_empty() {
                    entries.push(("Reasoning", reasoning_content.clone()));
                }
                if !content.is_empty() {
                    entries.push(("Assistant", content.clone()));
                }
                tool_calls.extend(calls.iter());
            }
            DialogMessage::Tool { id, result } => {
                let call = tool_calls.iter().find(|call| call.id == *id).unwrap();
                let mut body = format!("{}({})", call.name, call.arguments);
                if !result.is_empty() {
                    body.push_str("\n\n");
                    body.push_str(result);
                }
                entries.push(("Tool", body));
            }
        }
    }
    entries
}

fn message_subpane(title: &str, body: &str, index: usize) -> impl Scene {
    let title = title.to_string();
    let body = body.to_string();
    let index = DialogueMessageIndex(index);
    bsn! {
        @subpane()
        Node {
            width: percent(100),
        }
        Children [
            @subpane_header() Children [
                Text({title})
                ThemedText
                DialogueMessageTitle
                index
            ]
            --
            @subpane_body()
            Children [
                Node {
                    width: percent(100),
                }
                Text({body})
                ThemedText
                DialogueBody
                index
                TextLayout::new(Justify::Left, LineBreak::AnyCharacter)
                TextFont {
                    font_size: FontSize::Px(UI_FONT_SIZE),
                }
            ]
        ]
    }
}

fn dialogue_panel() -> impl Scene {
    bsn! {
        @subpane()
        Node {
            flex_grow: 1.0,
            min_height: px(0),
        }
        Children [
            @subpane_header() Children [
                Text("Player -")
                ThemedText
                DialogueHeader
            ]
            --
            @subpane_body()
            Node {
                flex_grow: 1.0,
                min_height: px(0),
            }
            Children [
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    padding: UiRect {
                        right: px(10),
                    },
                }
                Children [
                    #dialogue_scroll
                    Node {
                        flex_grow: 1.0,
                        min_height: px(0),
                        overflow: Overflow::scroll_y(),
                    }
                    ScrollArea
                    DialogueScrollArea
                    AutoScrollToBottom
                    Children [
                        Node {
                            width: percent(100),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8),
                        }
                        DialogueMessageList
                    ]
                    --
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
                        width: px(6),
                    }
                    @FeathersScrollbar {
                        @target: #dialogue_scroll,
                        @orientation: {ControlOrientation::Vertical}
                    }
                ]
            ]
        ]
    }
}

pub(super) fn dialogue_section() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.0,
            min_width: px(0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
        }
        Children [
            @dialogue_panel()
            --
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
            }
            Children [
                @FeathersButton {
                    @caption: bsn! {
                        Text("Continue Dialogue")
                        ThemedText
                    }
                    @variant: ButtonVariant::Primary,
                }
                ContinueDialogueButton
                @on(on_continue_dialogue)
            ]
        ]
    }
}

fn rebuild_dialogue_messages(commands: &mut Commands, list: Entity, dialog: &[DialogMessage]) {
    commands.entity(list).despawn_children();
    for (index, (title, body)) in dialog_entries(dialog).into_iter().enumerate() {
        let message_entity = commands
            .spawn_scene(message_subpane(title, &body, index))
            .id();
        commands.entity(list).add_child(message_entity);
    }
}

pub(crate) fn update_dialogue_panel(
    selected: Res<SelectedCharacter>,
    mut view: ResMut<DialogueView>,
    agents: Query<(Ref<Agent>, &Character)>,
    mut header: Single<
        &mut Text,
        (
            With<DialogueHeader>,
            Without<DialogueMessageTitle>,
            Without<DialogueBody>,
        ),
    >,
    list: Single<Entity, With<DialogueMessageList>>,
    mut titles: Query<
        (&DialogueMessageIndex, &mut Text),
        (
            With<DialogueMessageTitle>,
            Without<DialogueHeader>,
            Without<DialogueBody>,
        ),
    >,
    mut bodies: Query<
        (&DialogueMessageIndex, &mut Text),
        (
            With<DialogueBody>,
            Without<DialogueHeader>,
            Without<DialogueMessageTitle>,
        ),
    >,
    mut commands: Commands,
) {
    let Some(entity) = selected.0 else {
        if view.character.is_some() || selected.is_changed() {
            header.0 = "Player -".into();
            rebuild_dialogue_messages(&mut commands, *list, &[]);
            *view = DialogueView::default();
        }
        return;
    };
    let Ok((agent, character)) = agents.get(entity) else {
        return;
    };

    let selection_changed = view.character != Some(entity);
    let entries = dialog_entries(&agent.dialog);
    let count_changed = view.message_count != entries.len();
    if !selection_changed && !count_changed && !agent.is_changed() && !selected.is_changed() {
        return;
    }

    header.0 = format!("Player {}", character.id);

    if selection_changed || count_changed {
        rebuild_dialogue_messages(&mut commands, *list, &agent.dialog);
        view.character = Some(entity);
        view.message_count = entries.len();
        return;
    }

    for (DialogueMessageIndex(index), mut text) in &mut titles {
        text.0 = entries[*index].0.into();
    }
    for (DialogueMessageIndex(index), mut text) in &mut bodies {
        text.0 = entries[*index].1.clone();
    }
}

pub(crate) fn sync_continue_button(
    phase: Res<GamePhase>,
    button: Single<(Entity, Has<InteractionDisabled>), With<ContinueDialogueButton>>,
    mut commands: Commands,
) {
    if !phase.is_changed() {
        return;
    }
    let (entity, is_disabled) = *button;
    let should_disable = *phase != GamePhase::AwaitingContinue;
    if should_disable && !is_disabled {
        commands.entity(entity).insert(InteractionDisabled);
    } else if !should_disable && is_disabled {
        commands.entity(entity).remove::<InteractionDisabled>();
    }
}
