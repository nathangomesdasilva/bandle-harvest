use bevy::prelude::*;

const WINDOW_W: u32 = 1120;
const WINDOW_H: u32 = 720;
const PLAYER_SPEED: f32 = 220.0;
const INTERACT_RADIUS: f32 = 62.0;

#[bevy_main]
fn main() {
    run_app();
}

pub fn run_app() {
    #[cfg(target_os = "android")]
    let primary_window = Window {
        title: "Bandle Harvest".into(),
        resizable: true,
        ..default()
    };

    #[cfg(not(target_os = "android"))]
    let primary_window = Window {
        title: "Bandle Harvest — protótipo Bevy".into(),
        resolution: (WINDOW_W, WINDOW_H).into(),
        resizable: false,
        ..default()
    };

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.12, 0.23, 0.20)))
        .insert_resource(GameState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                handle_interaction,
                handle_sleep,
                update_plot_appearance,
                update_hud,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Lulu;

#[derive(Component)]
struct Bed;

#[derive(Component)]
struct CropPlot {
    crop: Option<Crop>,
}

#[derive(Clone, Copy)]
struct Crop {
    days_grown: u8,
    watered: bool,
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct InteractButton;

#[derive(Component)]
struct SleepButton;

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuestState {
    NotStarted,
    Active,
    Complete,
}

#[derive(Resource)]
struct GameState {
    day: u32,
    seeds: u32,
    berries: u32,
    gold: u32,
    quest: QuestState,
    message: String,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            day: 1,
            seeds: 6,
            berries: 0,
            gold: 0,
            quest: QuestState::NotStarted,
            message: "Bem-vindo à clareira! Fale com Lulu e prepare uma colheita.".into(),
        }
    }
}

#[derive(Clone, Copy)]
enum Target {
    Plot(Entity),
    Lulu,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Clareira, trilha e canteiros
    commands.spawn((
        Sprite::from_color(Color::srgb(0.29, 0.48, 0.30), Vec2::new(1040.0, 640.0)),
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.69, 0.55, 0.35), Vec2::new(1040.0, 92.0)),
        Transform::from_xyz(0.0, -135.0, -8.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.76, 0.64, 0.43), Vec2::new(620.0, 50.0)),
        Transform::from_xyz(20.0, 78.0, -7.0),
    ));

    for (x, y, color) in [
        (-440.0, 250.0, Color::srgb(0.19, 0.36, 0.25)),
        (-370.0, 230.0, Color::srgb(0.22, 0.40, 0.26)),
        (420.0, 230.0, Color::srgb(0.18, 0.34, 0.24)),
        (455.0, 190.0, Color::srgb(0.23, 0.39, 0.25)),
        (-450.0, -245.0, Color::srgb(0.20, 0.38, 0.24)),
        (425.0, -260.0, Color::srgb(0.18, 0.34, 0.23)),
    ] {
        commands.spawn((
            Sprite::from_color(color, Vec2::new(76.0, 46.0)),
            Transform::from_xyz(x, y, -5.0),
        ));
    }

    for x in [-330.0, -220.0, -110.0, 0.0] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.39, 0.24, 0.16), Vec2::new(82.0, 70.0)),
            Transform::from_xyz(x, -55.0, 0.0),
            CropPlot { crop: None },
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.55, 0.37, 0.22), Vec2::new(88.0, 6.0)),
            Transform::from_xyz(x, -91.0, 0.1),
        ));
    }

    // Cama
    commands.spawn((
        Sprite::from_color(Color::srgb(0.45, 0.28, 0.58), Vec2::new(74.0, 46.0)),
        Transform::from_xyz(330.0, -205.0, 1.0),
        Bed,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.88, 0.78, 0.88), Vec2::new(32.0, 16.0)),
        Transform::from_xyz(315.0, -195.0, 1.2),
    ));

    // Lulu
    commands.spawn((
        Sprite::from_color(Color::srgb(0.58, 0.37, 0.79), Vec2::new(34.0, 42.0)),
        Transform::from_xyz(145.0, 125.0, 2.0),
        Lulu,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.34, 0.22, 0.53), Vec2::new(48.0, 12.0)),
        Transform::from_xyz(145.0, 153.0, 2.1),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.95, 0.78, 0.49), Vec2::new(24.0, 22.0)),
        Transform::from_xyz(145.0, 129.0, 2.2),
    ));

    // Jogador
    commands.spawn((
        Sprite::from_color(Color::srgb(0.27, 0.64, 0.77), Vec2::new(30.0, 38.0)),
        Transform::from_xyz(180.0, -195.0, 3.0),
        Player,
    ));

    // HUD
    commands.spawn((
        Text::new(""),
        TextFont::from_font_size(15.0),
        TextColor(Color::srgb(0.96, 0.95, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            width: px(330),
            padding: UiRect::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.07, 0.10, 0.13, 0.90)),
        Hud,
    ));

    // Botão Touch: Interagir (E)
    commands
        .spawn((
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(20),
                right: px(140),
                width: px(110),
                height: px(54),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.20, 0.55, 0.45, 0.92)),
            InteractButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Interagir"),
                TextFont::from_font_size(16.0),
                TextColor(Color::WHITE),
            ));
        });

    // Botão Touch: Dormir (N)
    commands
        .spawn((
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(20),
                right: px(20),
                width: px(105),
                height: px(54),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.42, 0.28, 0.58, 0.92)),
            SleepButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Dormir"),
                TextFont::from_font_size(16.0),
                TextColor(Color::WHITE),
            ));
        });
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Controle por toque (arrastar na metade esquerda da tela)
    if let Ok(window) = windows.single() {
        let half_w = window.width() * 0.5;
        for touch in touches.iter() {
            if touch.start_position().x < half_w {
                let delta = touch.position() - touch.start_position();
                if delta.length() > 8.0 {
                    direction += Vec2::new(delta.x, -delta.y).normalize();
                }
            }
        }
    }

    if direction.length_squared() > 0.0 {
        let delta = direction.normalize() * PLAYER_SPEED * time.delta_secs();
        transform.translation.x = (transform.translation.x + delta.x).clamp(-495.0, 495.0);
        transform.translation.y = (transform.translation.y + delta.y).clamp(-295.0, 295.0);
    }
}

fn handle_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    interact_btn: Query<&Interaction, (Changed<Interaction>, With<InteractButton>)>,
    mut state: ResMut<GameState>,
    player: Query<&Transform, With<Player>>,
    lulu: Query<&Transform, (With<Lulu>, Without<Player>)>,
    mut plots: Query<(Entity, &Transform, &mut CropPlot)>,
) {
    let btn_pressed = interact_btn.iter().any(|i| *i == Interaction::Pressed);
    if !keys.just_pressed(KeyCode::KeyE) && !btn_pressed {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    let mut nearest: Option<(f32, Target)> = None;
    if let Ok(npc_transform) = lulu.single() {
        let distance = player_pos.distance(npc_transform.translation.truncate());
        if distance <= INTERACT_RADIUS {
            nearest = Some((distance, Target::Lulu));
        }
    }
    for (entity, transform, _) in &plots {
        let distance = player_pos.distance(transform.translation.truncate());
        if distance <= INTERACT_RADIUS && nearest.is_none_or(|(best, _)| distance < best) {
            nearest = Some((distance, Target::Plot(entity)));
        }
    }

    let Some((_, target)) = nearest else {
        state.message = "Chegue mais perto de um canteiro ou da Lulu para interagir.".into();
        return;
    };

    match target {
        Target::Lulu => match state.quest {
            QuestState::NotStarted => {
                state.quest = QuestState::Active;
                state.message = "Lulu: Estou preparando uma poção para estabilizar um portal de Bandle. Traga 3 Frutas Estelares! (Missão iniciada)".into();
            }
            QuestState::Active if state.berries >= 3 => {
                state.berries -= 3;
                state.gold += 45;
                state.quest = QuestState::Complete;
                state.message = "Lulu: Perfeito! A poção está pronta. O portal brilha de novo — e aqui vai sua recompensa: 45 moedas!".into();
            }
            QuestState::Active => {
                state.message = format!(
                    "Lulu: Ainda preciso de 3 Frutas Estelares. Você tem {}/3. Plante, regue e espere algumas manhãs!",
                    state.berries
                );
            }
            QuestState::Complete => {
                state.message =
                    "Lulu: O portal está estável. Obrigada pela ajuda, fazendeiro!".into();
            }
        },
        Target::Plot(entity) => {
            let Ok((_, _, mut plot)) = plots.get_mut(entity) else {
                return;
            };
            match &mut plot.crop {
                None if state.seeds > 0 => {
                    state.seeds -= 1;
                    plot.crop = Some(Crop {
                        days_grown: 0,
                        watered: false,
                    });
                    state.message =
                        "Você plantou uma semente de Fruta Estelar. Regue o canteiro.".into();
                }
                None => {
                    state.message =
                        "Você está sem sementes. (Protótipo: sementes iniciais limitadas.)".into()
                }
                Some(crop) if crop.days_grown >= 2 => {
                    state.berries += 1;
                    plot.crop = None;
                    state.message = format!("Colheita feita! Frutas Estelares: {}.", state.berries);
                }
                Some(crop) if crop.watered => {
                    state.message = "Esse canteiro já foi regado hoje. Descanse perto da cama para avançar o dia.".into();
                }
                Some(crop) => {
                    crop.watered = true;
                    state.message =
                        "Canteiro regado. Agora descanse perto da cama para a planta crescer."
                            .into();
                }
            }
        }
    }
}

fn handle_sleep(
    keys: Res<ButtonInput<KeyCode>>,
    sleep_btn: Query<&Interaction, (Changed<Interaction>, With<SleepButton>)>,
    mut state: ResMut<GameState>,
    player: Query<&Transform, With<Player>>,
    bed: Query<&Transform, (With<Bed>, Without<Player>)>,
    mut plots: Query<&mut CropPlot>,
) {
    let btn_pressed = sleep_btn.iter().any(|i| *i == Interaction::Pressed);
    if !keys.just_pressed(KeyCode::KeyN) && !btn_pressed {
        return;
    }
    let (Ok(player_transform), Ok(bed_transform)) = (player.single(), bed.single()) else {
        return;
    };
    if player_transform
        .translation
        .truncate()
        .distance(bed_transform.translation.truncate())
        > INTERACT_RADIUS + 12.0
    {
        state.message = "A cama fica no canto inferior direito da clareira. Chegue perto para dormir.".into();
        return;
    }

    state.day += 1;
    let mut grew = 0;
    for mut plot in &mut plots {
        if let Some(crop) = &mut plot.crop {
            if crop.watered && crop.days_grown < 2 {
                crop.days_grown += 1;
                crop.watered = false;
                grew += 1;
            }
        }
    }
    state.message = if grew > 0 {
        format!(
            "Dia {}! Suas plantas cresceram. Regue-as novamente para avançar mais uma etapa.",
            state.day
        )
    } else {
        format!(
            "Dia {}! As plantas só crescem quando regadas antes de dormir.",
            state.day
        )
    };
}

fn update_plot_appearance(mut plots: Query<(&CropPlot, &mut Sprite)>) {
    for (plot, mut sprite) in &mut plots {
        sprite.color = match plot.crop {
            None => Color::srgb(0.39, 0.24, 0.16),
            Some(Crop {
                days_grown: 0,
                watered: false,
            }) => Color::srgb(0.42, 0.31, 0.20),
            Some(Crop {
                days_grown: 0,
                watered: true,
            }) => Color::srgb(0.30, 0.35, 0.21),
            Some(Crop {
                days_grown: 1,
                watered: false,
            }) => Color::srgb(0.31, 0.53, 0.25),
            Some(Crop {
                days_grown: 1,
                watered: true,
            }) => Color::srgb(0.29, 0.62, 0.27),
            Some(Crop {
                days_grown: 2.., ..
            }) => Color::srgb(0.88, 0.48, 0.67),
        };
    }
}

fn update_hud(state: Res<GameState>, mut hud: Query<&mut Text, With<Hud>>) {
    if !state.is_changed() {
        return;
    }
    let quest_line = match state.quest {
        QuestState::NotStarted => "Missão: fale com Lulu".to_string(),
        QuestState::Active => format!(
            "Missão: entregue 3 Frutas Estelares ({}/3)",
            state.berries.min(3)
        ),
        QuestState::Complete => "Missão: portal de Bandle estabilizado".to_string(),
    };
    let text = format!(
        "BANDLE HARVEST\nDia {}  |  Sementes {}  |  Frutas {}  |  Moedas {}\n\n{}\n\nArraste na esquerda (ou WASD): mover\nBotões na direita (ou E / N): agir e dormir\n\n{}",
        state.day, state.seeds, state.berries, state.gold, quest_line, state.message
    );
    for mut ui_text in &mut hud {
        *ui_text = Text::new(text.clone());
    }
}
