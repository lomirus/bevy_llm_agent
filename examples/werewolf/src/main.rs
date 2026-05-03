mod auto_scroll;

use bevy::{
    feathers::{
        FeathersPlugins,
        containers::{
            flex_spacer, pane, pane_body, pane_header, pane_header_divider, subpane, subpane_body,
            subpane_header,
        },
        controls::{ButtonProps, ButtonVariant, button, tool_button},
        dark_theme::create_dark_theme,
        display::{label, label_dim},
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    input::mouse::MouseScrollUnit,
    input_focus::tab_navigation::TabGroup,
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Activate,
};
use bevy_llm::{
    Client, LlmPlugin, MultiTurnStreamItem, StreamMessage, StreamedAssistantContent, agent::Agent,
    prelude::*,
};
use rand::seq::SliceRandom;

use crate::auto_scroll::{AutoScrollToBottom, AutoScrollToBottomPlugin};

const NAMES: [&str; 12] = [
    "Alice", "Bob", "Charlie", "David", "Emma", "Frank", "Grace", "Henry", "Ivy", "Jack", "Kate",
    "Leo",
];

#[derive(Clone)]
enum RoleType {
    Werewolf,
    Villager,
    Seer,
    Witch,
    Hunter,
    Idiot,
}

impl RoleType {
    fn emoji(&self) -> &str {
        match self {
            Self::Werewolf => "🐺",
            Self::Villager => "🧑‍🌾",
            Self::Seer => "🔮",
            Self::Witch => "🧙‍♀️",
            Self::Hunter => "🏹",
            Self::Idiot => "🤪",
        }
    }
}

impl ToString for RoleType {
    fn to_string(&self) -> String {
        match self {
            Self::Werewolf => "狼人",
            Self::Villager => "村民",
            Self::Seer => "预言家",
            Self::Witch => "女巫",
            Self::Hunter => "猎人",
            Self::Idiot => "白痴",
        }
        .to_string()
    }
}

enum RoleState {
    Alive,
    Dead,
}

impl RoleState {
    fn emoji(&self) -> &str {
        match self {
            Self::Alive => "❤️",
            Self::Dead => "💀",
        }
    }
}

#[derive(Component)]
struct Character {
    name: String,
    role_type: RoleType,
    role_state: RoleState,
}

#[derive(Component, Clone, Default)]
struct RoleTypeText;

#[derive(Component, Clone, Default)]
struct RoleStateText;

#[derive(Component, Clone, Default)]
struct ThinkingContent;

#[derive(Component, Clone, Default)]
struct SpeakingContent;

#[derive(Resource)]
struct TurnQueue {
    speaker_order: Vec<Entity>,
    current_speaker: Option<Entity>,
    speech_record: Vec<Vec<String>>,
    round: usize,
}

#[derive(Component, Clone, Default)]
struct StartNextRoundButton;

#[derive(Component, FromTemplate)]
struct AgentReference(Entity);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                // See issue: https://github.com/bevyengine/bevy/issues/22733
                filter: format!("{DEFAULT_FILTER},wgpu_hal::vulkan::instance=off"),
                ..default()
            }),
            FeathersPlugins,
            LlmPlugin,
            AutoScrollToBottomPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, scene.spawn())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            handle_turn_queue_change.run_if(resource_changed::<TurnQueue>),
        )
        .add_systems(FixedUpdate, handle_agent_stream_mesaage)
        .run();
}

fn scene() -> impl SceneList {
    bsn_list![Camera2d]
}

fn setup(mut commands: Commands) {
    let mut role_types = [
        RoleType::Werewolf,
        RoleType::Werewolf,
        RoleType::Werewolf,
        RoleType::Werewolf,
        RoleType::Villager,
        RoleType::Villager,
        RoleType::Villager,
        RoleType::Villager,
        RoleType::Seer,
        RoleType::Witch,
        RoleType::Hunter,
        RoleType::Idiot,
    ];
    let mut rng = rand::rng();
    role_types.shuffle(&mut rng);

    let mut role_cards = Vec::with_capacity(NAMES.len());
    let mut speaker_order = Vec::with_capacity(NAMES.len());
    for (index, (name, role_type)) in NAMES.into_iter().zip(role_types).enumerate() {
        let client = Client::from_env().unwrap();
        let system_prompt = format!(
            include_str!("prompts/system.md"),
            name,
            role_type.to_string(),
            index + 1
        );
        let agent = client
            .agent(DEEPSEEK_V4_FLASH)
            .preamble(&system_prompt)
            .build();
        let agent_id = commands
            .spawn((
                Character {
                    name: name.to_string(),
                    role_type: role_type.clone(),
                    role_state: RoleState::Alive,
                },
                Agent::new(agent),
            ))
            .id();
        role_cards.push(bsn! { :role_card(name, role_type, agent_id) });
        speaker_order.push(agent_id);
    }

    commands.insert_resource(TurnQueue {
        speaker_order,
        current_speaker: None,
        speech_record: Vec::new(),
        round: 1,
    });

    commands.spawn_scene(ui_root(role_cards));
}

fn ui_root(role_cards: Vec<impl Scene>) -> impl Scene {
    bsn!((
        Node {
            width: percent(100),
            height: percent(100),
            padding: px(8),
            row_gap: px(8),
            flex_direction: FlexDirection::Column,
        }
        TabGroup
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            (
                Node {
                    flex_grow: 1.0,
                    display: Display::Grid,
                    row_gap: px(8),
                    column_gap: px(8),
                    grid_template_columns: { vec![
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                    ] },
                    grid_template_rows: { vec![
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                    ] },
                }
                Children [{role_cards}]
            ),
            Node {
                justify_content: JustifyContent::Center
            }
            Children [
                (
                    button(ButtonProps{
                        caption: Box::new(bsn_list!(
                            (Text("Start Next Round") ThemedText),
                        )),
                        ..default()
                    })
                    StartNextRoundButton
                    on(|_activate: On<Activate>, mut turn_queue: ResMut<TurnQueue>, mut agents: Query<&mut Agent>| {
                        let agent_id = turn_queue.speaker_order.first().unwrap();
                        let mut agent = agents.get_mut(*agent_id).unwrap();
                        turn_queue.current_speaker = Some(*agent_id);
                        turn_queue.speech_record.push(Vec::with_capacity(12));
                        agent.streaming_chat(format!(include_str!("prompts/user.md"), turn_queue.round, "无", "无"));
                    })
                )
            ]
        ]
    ))
}

fn role_card(role_name: &str, role_type: RoleType, agent_entity: Entity) -> impl Scene {
    bsn!(
        :pane
        Node {
            justify_content: JustifyContent::Stretch
        }
        Children [
            :pane_header Children [
                Text({role_type.emoji()}) ThemedText RoleTypeText,
                Text({RoleState::Alive.emoji()}) ThemedText RoleStateText,
                :pane_header_divider,
                Text(role_name) ThemedText,
                :flex_spacer,
                (
                    :tool_button(ButtonProps{
                        variant: ButtonVariant::Plain,
                        ..default()
                    })
                    on(|_activate: On<Activate>| {
                        info!("TODO: Switch to the message in the previous round")
                    })
                    Children [
                        (Text("◀️") ThemedText)
                    ]
                ),
                (
                    :tool_button(ButtonProps{
                        variant: ButtonVariant::Plain,
                        ..default()
                    })
                    on(|_activate: On<Activate>| {
                        info!("TODO: Switch to the message in the next round")
                    })
                    Children [
                        (Text("▶️") ThemedText)
                    ]
                ),
            ],
            :pane_body
            Node {
                flex_grow: 1.0,
                overflow: Overflow::scroll_y()
            }
            AutoScrollToBottom
            on(
                |scroll: On<Pointer<Scroll>>,
                mut query: Query<(&mut ScrollPosition, &ComputedNode)>| {
                    if let Ok((mut scroll_position, node)) = query.get_mut(scroll.entity) {
                        let dy = match scroll.unit {
                            MouseScrollUnit::Line => scroll.y * 20.0,
                            MouseScrollUnit::Pixel => scroll.y,
                        };
                        let max_y =
                            (node.content_size.y - node.size.y).max(0.0) * node.inverse_scale_factor;
                        scroll_position.y = (scroll_position.y - dy).clamp(0.0, max_y);
                    }
                },
            )
            Children [
                :subpane Children [
                    :subpane_header Children [
                        Text("Thinking") ThemedText,
                    ],
                    :subpane_body Children [
                        :label_dim("") ThinkingContent AgentReference(agent_entity),
                    ],
                ],
                Node { height: px(8) },
                :label("") SpeakingContent AgentReference(agent_entity),
            ]
        ]
    )
}

fn handle_turn_queue_change(
    mut commands: Commands,
    turn_queue: Res<TurnQueue>,
    start_next_round_button: Single<(Entity, Has<InteractionDisabled>), With<StartNextRoundButton>>,
) {
    let (entity, disabled) = *start_next_round_button;
    let mut entity_commands = commands.entity(entity.entity());
    if turn_queue.current_speaker.is_some() && !disabled {
        entity_commands.insert(InteractionDisabled);
    } else if turn_queue.current_speaker.is_none() && disabled {
        entity_commands.remove::<InteractionDisabled>();
    }
}

fn handle_agent_stream_mesaage(
    mut messages: MessageReader<StreamMessage>,
    mut turn_queue: ResMut<TurnQueue>,
    mut agents: Query<&mut Agent>,
    characters: Query<&Character>,
    thinking_content: Query<
        (&mut Text, &AgentReference),
        (With<ThinkingContent>, Without<SpeakingContent>),
    >,
    speaking_content: Query<
        (&mut Text, &AgentReference),
        (With<SpeakingContent>, Without<ThinkingContent>),
    >,
) {
    let Some(agent_id) = turn_queue.current_speaker else {
        return;
    };
    let (mut thinking_content, _) = thinking_content
        .into_iter()
        .find(|(_, agent_ref)| agent_ref.0 == agent_id)
        .unwrap();
    let (mut speaking_content, _) = speaking_content
        .into_iter()
        .find(|(_, agent_ref)| agent_ref.0 == agent_id)
        .unwrap();

    for StreamMessage {
        entity: _entity,
        delta,
    } in messages.read()
    {
        use MultiTurnStreamItem::*;
        match delta {
            StreamAssistantItem(content) => match content {
                StreamedAssistantContent::Text(text) => {
                    speaking_content.push_str(text.text());
                }
                StreamedAssistantContent::ToolCall { .. } => todo!("输出 info 日志"),
                StreamedAssistantContent::ToolCallDelta { .. } => todo!("输出 info 日志"),
                StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                    thinking_content.push_str(reasoning);
                }
                StreamedAssistantContent::Final(..) => {}
                _ => unreachable!(),
            },
            StreamUserItem(..) => todo!("输出 info 日志"),
            FinalResponse(final_response) => {
                let round_speech_record = turn_queue.speech_record.last_mut().unwrap();
                round_speech_record.push(final_response.response().to_string());

                let current_speaker_index = turn_queue
                    .speaker_order
                    .iter()
                    .position(|speaker| *speaker == agent_id)
                    .unwrap();
                turn_queue.current_speaker = if current_speaker_index
                    == turn_queue.speaker_order.len() - 1
                {
                    None
                } else {
                    let next_speaker_index = current_speaker_index + 1;
                    let next_agent_id = turn_queue.speaker_order[next_speaker_index];

                    let previous_round_speech_record: Vec<_> = turn_queue
                        .round
                        .checked_sub(2)
                        .and_then(|round_index| turn_queue.speech_record.get(round_index))
                        .map(|speech_record| {
                            speech_record
                                .iter()
                                .enumerate()
                                .filter(|(index, _)| *index > next_speaker_index)
                                .map(|(index, msg)| {
                                    let entity = turn_queue.speaker_order[index];
                                    let character = characters.get(entity).unwrap();
                                    format!("- ({}/12) {}: {}", index + 1, character.name, msg)
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let previous_round_speech_record = if previous_round_speech_record.is_empty() {
                        "无".to_string()
                    } else {
                        previous_round_speech_record.join("\n")
                    };

                    let current_round_speech_record: Vec<_> = turn_queue.speech_record
                        [turn_queue.round - 1]
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index < next_speaker_index)
                        .map(|(index, msg)| {
                            let entity = turn_queue.speaker_order[index];
                            let character = characters.get(entity).unwrap();
                            format!("- ({}/12) {}: {}", index + 1, character.name, msg)
                        })
                        .collect();
                    let current_round_speech_record = if current_round_speech_record.is_empty() {
                        "无".to_string()
                    } else {
                        current_round_speech_record.join("\n")
                    };

                    let mut next_agent = agents.get_mut(next_agent_id).unwrap();
                    next_agent.streaming_chat(format!(
                        include_str!("prompts/user.md"),
                        turn_queue.round, previous_round_speech_record, current_round_speech_record
                    ));
                    Some(next_agent_id)
                };
            }
            _ => unreachable!(),
        }
    }
}
