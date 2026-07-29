mod tools;

use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    feathers::{
        FeathersPlugins,
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            ButtonVariant, FeathersButton, FeathersListRow, FeathersListView, FeathersScrollbar,
        },
        dark_theme::create_dark_theme,
        palette,
        theme::{ThemeBackgroundColor, ThemeBorderColor, ThemedText, UiTheme},
        tokens,
    },
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    ui::{InteractionDisabled, Overflow, Selected, widget::ViewportNode},
    ui_widgets::{Activate, ControlOrientation, ScrollArea, ValueChange},
};
use bevy_llm_agent::{
    AppExt, DEEPSEEK_V4_FLASH, LlmAgentPlugin,
    agent::{Agent, DialogMessage, Thinking},
    messages::{
        agent_message::{AgentMessage, AgentMessageDelta},
        user_message::UserMessage,
    },
    tool::AgentTools,
};
use rand::random_range;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tools::{DevourCharacter, GetCurrentInfo, MoveTo};

const UI_FONT_SIZE: f32 = 12.0;
/// Map side length in meters (matches `Coordinate::random` range of ±10).
const MAP_SIZE_M: f32 = 20.0;
/// Character collision / render radius in meters.
const CHARACTER_RADIUS_M: f32 = 0.5;
/// Render layer for the map camera and world entities (UI stays on layer 0).
const MAP_RENDER_LAYER: usize = 1;
const CONTINUE_ACTION_PROMPT: &str =
    "Take your next action now. You must call tools to act.";

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
    /// Whether the character has been eliminated.
    is_dead: bool,
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

/// World clock only advances in [`GamePhase::Moving`].
#[derive(Resource, Clone, PartialEq, Eq, Default)]
enum GamePhase {
    /// Waiting for the player to click Continue Dialogue.
    #[default]
    AwaitingContinue,
    /// Idle characters were prompted; wait until every character has a move target
    /// (or is dead) and all prompted agent outputs have finished.
    AwaitingMoves,
    /// Targets are set; coordinates advance with real time.
    Moving,
}

/// Characters prompted this round whose agent output has not finished yet.
#[derive(Resource, Default)]
struct AwaitingAgentOutputs(Vec<Entity>);

#[derive(Resource, Default)]
struct SelectedCharacter(Option<Entity>);

#[derive(Component, Clone)]
struct CharacterListItem(Entity);

impl Default for CharacterListItem {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Clone, Default)]
struct CharacterDetailBody;

/// Marks the UI node that displays the map camera render target.
#[derive(Component, Clone, Default)]
struct MapViewport;

#[derive(Component, Clone, Default)]
struct MapCamera;

#[derive(Component, Clone, Default)]
struct ContinueDialogueButton;

#[derive(Component, Clone, Default)]
struct DialogueHeader;

#[derive(Component, Clone, Default)]
struct DialogueMessageList;

#[derive(Component, Clone, Default)]
struct DialogueMessageTitle;

#[derive(Component, Clone, Default)]
struct DialogueBody;

#[derive(Component, Clone, Default)]
struct DialogueMessageIndex(usize);

#[derive(Component, Clone, Default)]
struct DialogueScrollArea;

/// Which character's dialog is currently mirrored into the message list.
#[derive(Resource, Default)]
struct DialogueView {
    character: Option<Entity>,
    message_count: usize,
}

fn character_scene(character: Character) -> impl Scene {
    let transform = Transform::from_xyz(character.coordinate.x, character.coordinate.y, 0.0);
    let render_layers = RenderLayers::layer(MAP_RENDER_LAYER);
    bsn! {
        template_value(character)
        Agent::new(DEEPSEEK_V4_FLASH, Thinking::Off)
        AgentTools [
            @GetCurrentInfo,
            @MoveTo,
            @DevourCharacter,
        ]
        Mesh2d(asset_value(Circle::new(CHARACTER_RADIUS_M)))
        MeshMaterial2d::<ColorMaterial>(asset_value(palette::LIGHT_GRAY_1))
        template_value(transform)
        template_value(render_layers)
        on(on_character_map_click)
    }
}

fn map_panel() -> impl Scene {
    bsn! {
        Node {
            height: percent(100),
            aspect_ratio: {Some(1.0)},
            flex_shrink: 0.0,
            border: UiRect::all(px(1)),
            padding: px(8),
        }
        ThemeBackgroundColor(tokens::PANE_BODY_BG)
        ThemeBorderColor(tokens::PANE_HEADER_BORDER)
        Children [(
            Node {
                width: percent(100),
                height: percent(100),
            }
            MapViewport
        )]
    }
}

fn character_detail_values(character: &Character) -> [String; 6] {
    [
        character.id.to_string(),
        character.combat_power.to_string(),
        format!("{:.2} m/s", character.moving_speed),
        format!(
            "({:.1}, {:.1})",
            character.coordinate.x, character.coordinate.y
        ),
        match &character.moving_target {
            Some(t) => format!("({:.1}, {:.1})", t.x, t.y),
            None => "None".into(),
        },
        if character.is_dead {
            "Yes".into()
        } else {
            "No".into()
        },
    ]
}

fn detail_row(label: &str, value: String) -> impl Scene {
    let label = label.to_string();
    bsn! {
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
        }
        Children [
            (
                Text({label})
                ThemedText
                TextLayout::new(Justify::Left, LineBreak::NoWrap)
                TextFont {
                    font_size: FontSize::Px(UI_FONT_SIZE),
                }
            ),
            (
                Text({value})
                ThemedText
                TextLayout::new(Justify::Right, LineBreak::NoWrap)
                TextFont {
                    font_size: FontSize::Px(UI_FONT_SIZE),
                }
            ),
        ]
    }
}

fn character_detail_panel() -> impl Scene {
    bsn! {
        subpane()
        Children [
            subpane_header() Children [
                (Text("Selected Character") ThemedText)
            ],
            (
                subpane_body()
                CharacterDetailBody
                Children [
                    ({detail_row("ID", String::new())}),
                    ({detail_row("Combat Power", String::new())}),
                    ({detail_row("Moving Speed", String::new())}),
                    ({detail_row("Coordinate", String::new())}),
                    ({detail_row("Moving Target", String::new())}),
                    ({detail_row("Dead", String::new())}),
                ]
            )
        ]
    }
}

fn character_list_row(entity: Entity, character: &Character) -> impl Scene {
    let label = format!("ID {}: {}", character.id, character.combat_power);
    bsn! {
        @FeathersListRow
        CharacterListItem(entity)
        Children [(
            Text({label})
            ThemedText
        )]
    }
}

fn character_list_panel(rows: impl SceneList) -> impl Scene {
    bsn! {
        subpane()
        Node {
            flex_grow: 1.0,
            min_height: px(0),
        }
        Children [
            subpane_header() Children [
                (Text("Characters") ThemedText)
            ],
            (
                subpane_body()
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                }
                Children [(
                    @FeathersListView {
                        @rows: {Box::new(rows) as Box<dyn SceneList>}
                    }
                    Node {
                        flex_grow: 1.0,
                        min_height: px(0),
                    }
                    on(on_character_list_select)
                )]
            )
        ]
    }
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
                let call = tool_calls
                    .iter()
                    .find(|call| call.id == *id)
                    .unwrap();
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
        subpane()
        Node {
            width: percent(100),
        }
        Children [
            subpane_header() Children [
                (
                    Text({title})
                    ThemedText
                    DialogueMessageTitle
                    template_value(index.clone())
                )
            ],
            (
                subpane_body()
                Children [(
                    Node {
                        width: percent(100),
                    }
                    Text({body})
                    ThemedText
                    DialogueBody
                    template_value(index)
                    TextLayout::new(Justify::Left, LineBreak::AnyCharacter)
                    TextFont {
                        font_size: FontSize::Px(UI_FONT_SIZE),
                    }
                )]
            )
        ]
    }
}

fn dialogue_panel() -> impl Scene {
    bsn! {
        subpane()
        Node {
            flex_grow: 1.0,
            min_height: px(0),
        }
        Children [
            subpane_header() Children [(
                Text("Player -")
                ThemedText
                DialogueHeader
            )],
            subpane_body()
            Node {
                flex_grow: 1.0,
                min_height: px(0),
            }
            Children [(
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
                    (
                        #dialogue_scroll
                        Node {
                            flex_grow: 1.0,
                            min_height: px(0),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        DialogueScrollArea
                        Children [(
                            Node {
                                width: percent(100),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(8),
                            }
                            DialogueMessageList
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
                            @target: #dialogue_scroll,
                            @orientation: {ControlOrientation::Vertical}
                        }
                    )
                ]
            )]
        ]
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // These camera components are not bsn-friendly (missing Clone/Default/FromTemplate).
    commands.spawn((Camera2d, IsDefaultUiCamera));

    let mut map_image = Image::new_uninit(
        default(),
        TextureDimension::D2,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
    map_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let map_image = images.add(map_image);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(palette::GRAY_1),
            ..default()
        },
        RenderTarget::Image(map_image.into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: MAP_SIZE_M,
                height: MAP_SIZE_M,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(MAP_RENDER_LAYER),
        MapCamera,
    ));

    let mut characters = Vec::new();
    for id in 1..=10 {
        let character = Character {
            id,
            combat_power: random_range::<u32, _>(1..=1000),
            moving_speed: random_range::<f32, _>(0.5..2.0),
            coordinate: Coordinate::random(),
            moving_target: None,
            is_dead: false,
        };
        let entity = commands
            .spawn_scene(character_scene(character.clone()))
            .id();
        characters.push((entity, character));
    }

    characters.sort_by_key(|(_, character)| character.id);

    commands.spawn_scene(game_ui(&characters));
}

fn attach_map_viewport(
    mut commands: Commands,
    map_viewport: Single<Entity, With<MapViewport>>,
    map_camera: Single<Entity, With<MapCamera>>,
) {
    commands
        .entity(*map_viewport)
        .insert(ViewportNode::new(*map_camera));
}

fn game_ui(characters: &[(Entity, Character)]) -> impl Scene {
    let rows: Vec<_> = characters
        .iter()
        .map(|(entity, character)| character_list_row(*entity, character))
        .collect();

    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            padding: px(16),
            column_gap: px(16),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            // Character inspector
            (
                Node {
                    min_width: px(240),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                }
                Children [
                    ({character_detail_panel()}),
                    ({character_list_panel(rows)}),
                ]
            ),
            // 20m x 20m square game map
            ({map_panel()}),
            // Right-hand dialogue panel
            (
                Node {
                    flex_grow: 1.0,
                    min_width: px(0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                }
                Children [
                    ({dialogue_panel()}),
                    (
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                        }
                        Children [
                            (
                                @FeathersButton {
                                    @caption: bsn! {
                                        Text("Continue Dialogue")
                                        ThemedText
                                    }
                                    @variant: ButtonVariant::Primary,
                                }
                                ContinueDialogueButton
                                on(on_continue_dialogue)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn apply_character_selection(
    character: Entity,
    selected: &mut SelectedCharacter,
    items: &Query<(Entity, &CharacterListItem, Has<Selected>)>,
    commands: &mut Commands,
) {
    selected.0 = Some(character);
    for (row, CharacterListItem(item), is_selected) in items {
        if *item == character {
            if !is_selected {
                commands.entity(row).insert(Selected);
            }
        } else if is_selected {
            commands.entity(row).remove::<Selected>();
        }
    }
}

fn on_character_list_select(
    change: On<ValueChange<Entity>>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut selected: ResMut<SelectedCharacter>,
    mut commands: Commands,
) {
    let Ok((_, CharacterListItem(character), _)) = items.get(change.event().value) else {
        return;
    };
    apply_character_selection(*character, &mut selected, &items, &mut commands);
}

fn on_character_map_click(
    click: On<Pointer<Click>>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut selected: ResMut<SelectedCharacter>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    apply_character_selection(click.entity, &mut selected, &items, &mut commands);
}

fn on_continue_dialogue(
    _activate: On<Activate>,
    mut phase: ResMut<GamePhase>,
    mut awaiting_outputs: ResMut<AwaitingAgentOutputs>,
    characters: Query<(Entity, &Character)>,
    mut sender: MessageWriter<UserMessage>,
) {
    if *phase != GamePhase::AwaitingContinue {
        return;
    }

    let mut idle: Vec<_> = characters
        .iter()
        .filter(|(_, character)| !character.is_dead && character.moving_target.is_none())
        .collect();
    if idle.is_empty() {
        return;
    }
    idle.sort_by_key(|(_, character)| character.id);

    awaiting_outputs.0 = idle.iter().map(|(entity, _)| *entity).collect();
    for (entity, _) in &idle {
        sender.write(UserMessage::new(*entity, CONTINUE_ACTION_PROMPT));
    }
    *phase = GamePhase::AwaitingMoves;
}

fn rebuild_dialogue_messages(
    commands: &mut Commands,
    list: Entity,
    dialog: &[DialogMessage],
) {
    commands.entity(list).despawn_children();
    for (index, (title, body)) in dialog_entries(dialog).into_iter().enumerate() {
        let message_entity = commands
            .spawn_scene(message_subpane(title, &body, index))
            .id();
        commands.entity(list).add_child(message_entity);
    }
}

fn update_dialogue_panel(
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
    mut scroll: Single<&mut ScrollPosition, With<DialogueScrollArea>>,
    mut commands: Commands,
) {
    let Some(entity) = selected.0 else {
        if view.character.is_some() || selected.is_changed() {
            header.0 = "Player -".into();
            rebuild_dialogue_messages(&mut commands, *list, &[]);
            **scroll = ScrollPosition::DEFAULT;
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
        if selection_changed {
            **scroll = ScrollPosition::DEFAULT;
        }
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

fn select_strongest_character(
    mut selected: ResMut<SelectedCharacter>,
    characters: Query<(Entity, &Character)>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut commands: Commands,
) {
    let Some(character) = characters
        .iter()
        .max_by_key(|(_, character)| character.combat_power)
        .map(|(entity, _)| entity)
    else {
        return;
    };
    apply_character_selection(character, &mut selected, &items, &mut commands);
}

fn update_character_detail(
    selected: Res<SelectedCharacter>,
    characters: Query<Ref<Character>>,
    body: Single<&Children, With<CharacterDetailBody>>,
    rows: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    let Some(entity) = selected.0 else {
        return;
    };
    let Ok(character) = characters.get(entity) else {
        return;
    };
    if !selected.is_changed() && !character.is_changed() {
        return;
    }
    let values = character_detail_values(&character);
    for (row, value) in body.iter().zip(values) {
        let Some(&value_entity) = rows.get(row).ok().and_then(|c| c.get(1)) else {
            continue;
        };
        if let Ok(mut text) = texts.get_mut(value_entity) {
            text.0 = value;
        }
    }
}

fn update_character_sprites(
    selected: Res<SelectedCharacter>,
    mut characters: Query<(
        Entity,
        Ref<Character>,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let selection_changed = selected.is_changed();
    for (entity, character, mut transform, mut visibility, material) in &mut characters {
        if character.is_changed() {
            transform.translation.x = character.coordinate.x;
            transform.translation.y = character.coordinate.y;
            *visibility = if character.is_dead {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
        if selection_changed {
            if let Some(mut material) = materials.get_mut(&material.0) {
                material.color = if selected.0 == Some(entity) {
                    palette::ACCENT
                } else {
                    palette::LIGHT_GRAY_1
                };
            }
        }
    }
}

fn track_awaiting_agent_outputs(
    mut awaiting_outputs: ResMut<AwaitingAgentOutputs>,
    mut messages: MessageReader<AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for AgentMessage { entity, delta } in messages.read() {
        match delta {
            Ok(AgentMessageDelta::Finish) => {
                awaiting_outputs.0.retain(|&e| e != *entity);
            }
            Ok(_) => {}
            Err(err) => {
                error!("agent {entity} failed: {err:?}");
                app_exit.write(AppExit::error());
            }
        }
    }
}

fn check_awaiting_moves(
    mut phase: ResMut<GamePhase>,
    awaiting_outputs: Res<AwaitingAgentOutputs>,
    characters: Query<&Character>,
) {
    if *phase != GamePhase::AwaitingMoves {
        return;
    }
    let targets_ready = characters
        .iter()
        .all(|character| character.is_dead || character.moving_target.is_some());
    if targets_ready && awaiting_outputs.0.is_empty() {
        *phase = GamePhase::Moving;
    }
}

fn advance_world(
    time: Res<Time>,
    mut phase: ResMut<GamePhase>,
    mut elapsed: ResMut<ElapsedTime>,
    mut characters: Query<&mut Character>,
) {
    if *phase != GamePhase::Moving {
        return;
    }

    let dt = time.delta_secs();
    elapsed.0 += dt;

    let mut anyone_arrived = false;
    for mut character in &mut characters {
        if character.is_dead {
            continue;
        }
        let target = character.moving_target.clone().unwrap();
        let dist = character.coordinate.dist(&target);
        let step = character.moving_speed * dt;
        if step >= dist {
            character.coordinate = target;
            character.moving_target = None;
            anyone_arrived = true;
        } else {
            let t = step / dist;
            character.coordinate.x += (target.x - character.coordinate.x) * t;
            character.coordinate.y += (target.y - character.coordinate.y) * t;
        }
    }

    if anyone_arrived {
        *phase = GamePhase::AwaitingContinue;
    }
}

fn sync_continue_button(
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
        ))
        .add_agent_tool::<GetCurrentInfo>()
        .add_agent_tool::<MoveTo>()
        .add_agent_tool::<DevourCharacter>()
        .add_systems(
            Startup,
            (
                setup,
                ApplyDeferred,
                attach_map_viewport,
                select_strongest_character,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_dialogue_panel,
                update_character_detail,
                update_character_sprites,
                (
                    track_awaiting_agent_outputs,
                    check_awaiting_moves,
                )
                    .chain(),
                advance_world,
                sync_continue_button,
            ),
        )
        .run();
}
