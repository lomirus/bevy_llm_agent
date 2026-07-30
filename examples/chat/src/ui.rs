use bevy::{
    feathers::{
        controls::{
            ButtonVariant, FeathersButton, FeathersScrollbar, FeathersTextInput,
            FeathersTextInputContainer,
        },
        rounded_corners::RoundedCorners,
        theme::{ThemeBackgroundColor, ThemeBorderColor, ThemedText},
        tokens,
    },
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::FocusedInput,
    prelude::*,
    text::{EditableText, FontSource, FontSourceTemplate},
    ui::{InteractionDisabled, Overflow},
    ui_widgets::{Activate, ControlOrientation, ScrollArea},
};
use bevy_llm_agent::{
    agent::{Agent, DialogMessage},
    messages::{
        agent_message::{AgentMessage, AgentMessageDelta},
        user_message::UserMessage,
    },
};

use crate::auto_scroll::AutoScrollToBottom;

const UI_FONT_SIZE: f32 = 14.0;

#[derive(Component, Clone, Default)]
pub(crate) struct ChatMessageList;

#[derive(Component, Clone, Default)]
pub(crate) struct ChatInput;

#[derive(Component, Clone, Default)]
pub(crate) struct SendButton;

fn dialog_entries(dialog: &[DialogMessage]) -> Vec<(&str, AlignSelf)> {
    let mut entries = Vec::new();
    for message in dialog {
        match message {
            DialogMessage::System { content } => {
                entries.push((content.as_str(), AlignSelf::FlexStart));
            }
            DialogMessage::User { content } => {
                entries.push((content.as_str(), AlignSelf::FlexEnd));
            }
            DialogMessage::Assistant {
                content,
                reasoning_content,
                ..
            } => {
                for body in [reasoning_content, content] {
                    if !body.is_empty() {
                        entries.push((body.as_str(), AlignSelf::FlexStart));
                    }
                }
            }
            DialogMessage::Tool { .. } => {}
        }
    }
    entries
}

fn message_bubble(body: &str, align: AlignSelf) -> impl Scene {
    let body = body.to_string();
    bsn! {
        Node {
            width: percent(80),
            align_self: {align},
            padding: px(8),
            border: UiRect::all(px(1)),
            border_radius: {RoundedCorners::All.to_border_radius(8.0)},
        }
        ThemeBackgroundColor(tokens::SUBPANE_BODY_BG)
        ThemeBorderColor(tokens::SUBPANE_BODY_BORDER)
        Text({body})
        ThemedText
        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter)
        TextFont {
            font: FontSourceTemplate::SystemUi,
            font_size: FontSize::Px(UI_FONT_SIZE),
        }
    }
}

fn chat_messages_panel() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect {
                right: px(10),
            },
        }
        Children [
            (
                #chat_scroll
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    overflow: Overflow::scroll_y(),
                    padding: px(8),
                }
                ScrollArea
                AutoScrollToBottom
                Children [(
                    Node {
                        width: percent(100),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                    }
                    ChatMessageList
                )]
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    width: px(6),
                }
                @FeathersScrollbar {
                    @target: #chat_scroll,
                    @orientation: {ControlOrientation::Vertical}
                }
            )
        ]
    }
}

fn input_row() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            flex_shrink: 0.0,
        }
        Children [
            (
                @FeathersTextInputContainer
                Children [(
                    @FeathersTextInput
                    ChatInput
                    on(on_chat_input_enter)
                )]
            ),
            (
                @FeathersButton {
                    @caption: bsn! {
                        Text("Send")
                        ThemedText
                    }
                    @variant: ButtonVariant::Primary,
                }
                SendButton
                on(on_send_clicked)
            ),
        ]
    }
}

pub(crate) fn chat_ui() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: px(16),
            row_gap: px(12),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            ({chat_messages_panel()}),
            ({input_row()}),
        ]
    }
}

/// `FeathersTextInput` hardcodes FiraSans, which has no CJK coverage. It supplies the font through a
/// `template(..)`, and those are never merged by component type, so patching `TextFont` in the scene
/// would insert it twice. The font has to be replaced once the scene is spawned instead.
pub(crate) fn use_system_font(mut font: Single<&mut TextFont, With<ChatInput>>) {
    font.font = FontSource::SystemUi;
    font.font_size = FontSize::Px(UI_FONT_SIZE);
}

fn on_send_clicked(
    _activate: On<Activate>,
    agent: Single<Entity, With<Agent>>,
    button: Single<Entity, With<SendButton>>,
    mut input: Single<&mut EditableText, With<ChatInput>>,
    mut sender: MessageWriter<UserMessage>,
    mut commands: Commands,
) {
    let prompt = input.value().to_string().trim().to_owned();
    if prompt.is_empty() {
        return;
    }
    input.clear();
    commands.entity(*button).insert(InteractionDisabled);
    sender.write(UserMessage::new(*agent, prompt));
}

fn on_chat_input_enter(
    key_input: On<FocusedInput<KeyboardInput>>,
    button: Single<(Entity, Has<InteractionDisabled>), With<SendButton>>,
    input: Single<&EditableText, With<ChatInput>>,
    mut commands: Commands,
) {
    let (button, is_disabled) = *button;
    if key_input.input.key_code != KeyCode::Enter
        || key_input.input.state != ButtonState::Pressed
        || is_disabled
        // Enter picks an IME candidate while composing, so it must not submit.
        || input.is_composing()
    {
        return;
    }
    commands.trigger(Activate { entity: button });
}

pub(crate) fn update_chat_messages(
    agent: Single<Ref<Agent>>,
    list: Single<(Entity, Option<&Children>), With<ChatMessageList>>,
    mut bodies: Query<&mut Text>,
    mut commands: Commands,
) {
    if !agent.is_changed() {
        return;
    }

    let entries = dialog_entries(&agent.dialog);
    let (list_entity, children) = *list;
    let rendered = children.map_or(0, Children::len);

    if rendered < entries.len() {
        for &(body, align) in &entries[rendered..] {
            commands
                .spawn_scene(message_bubble(body, align))
                .insert(ChildOf(list_entity));
        }
    } else if let Some(&bubble) = children.and_then(|children| children.last())
        && let Some(&(body, _)) = entries.last()
    {
        bodies.get_mut(bubble).unwrap().0 = body.to_owned();
    }
}

pub(crate) fn enable_send_button(
    button: Single<Entity, (With<SendButton>, With<InteractionDisabled>)>,
    mut messages: MessageReader<AgentMessage>,
    mut commands: Commands,
) {
    for AgentMessage { delta, .. } in messages.read() {
        if let Ok(AgentMessageDelta::Finish) | Err(_) = delta {
            commands.entity(*button).remove::<InteractionDisabled>();
        }
    }
}
