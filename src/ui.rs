//! Menus interactifs, HUD et écrans de fin.
//!
//! Tous les écrans (Main, Settings, Credits, Pause, GameOver, Win)
//! utilisent une UI commune : entête (titre + sous-titre), bloc central
//! de boutons cliquables/navigables clavier, footer.
//!
//! Boutons : highlight au survol (souris), navigation clavier (Up/Down +
//! Enter), action déclenchée via `MenuAction` (composant attaché à
//! chaque bouton).

use crate::heroes::{Hero, SelectedHero};
use crate::level::{Checkpoint, RespawnPoint};
use crate::physics::{Grounded, Velocity};
use crate::player::{Player, PlayerController};
use crate::save::{save_data, save_settings, SaveData, Settings};
use crate::states::{GameState, RunStats};
use crate::world::{CurrentLevel, PLAYER_SPAWN, TOTAL_LEVELS};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::text::Font;
use bevy::window::{PrimaryWindow, WindowMode};

// ============================================================ Style ===

const OVERLAY_BG: Color = Color::srgba(0.04, 0.06, 0.10, 0.55);
const TITLE_COLOR: Color = Color::srgb(0.92, 0.94, 1.00);
const SUBTITLE_COLOR: Color = Color::srgb(0.66, 0.74, 0.86);
const HINT_COLOR: Color = Color::srgb(0.48, 0.56, 0.68);
const ACCENT_CYAN: Color = Color::srgb(0.42, 0.78, 0.92);
const ACCENT_AMBER: Color = Color::srgb(0.91, 0.66, 0.30);

const BTN_NORMAL: Color = Color::srgba(0.10, 0.14, 0.22, 0.65);
const BTN_HOVER: Color = Color::srgba(0.18, 0.26, 0.40, 0.85);
const BTN_PRESSED: Color = Color::srgba(0.30, 0.50, 0.66, 0.95);
const BTN_SELECTED: Color = Color::srgba(0.16, 0.30, 0.46, 0.90);
const BTN_BORDER: Color = Color::srgb(0.20, 0.30, 0.46);
const BTN_BORDER_SELECTED: Color = Color::srgb(0.42, 0.78, 0.92);

// ====================================================== Composants ===

/// Composant qui marque tout ce qui appartient à l'écran courant et
/// doit disparaître au changement d'état.
#[derive(Component)]
struct ScreenTag;

#[derive(Component)]
struct HudTag;

#[derive(Component)]
struct HudDeaths;

#[derive(Component)]
struct HudTime;

#[derive(Component)]
struct HudLevel;

#[derive(Component)]
struct TitleText;

/// Marque un bouton et son action associée.
#[derive(Component, Clone, Copy, Debug)]
pub enum MenuAction {
    StartNewGame,
    Continue,
    Resume,
    Restart,
    GotoHeroSelect,
    GotoSettings,
    GotoCredits,
    GotoMainMenu,
    SelectHero(Hero),
    ConfirmHero,
    ToggleFullscreen,
    AdjustMaster(f32),
    AdjustMusic(f32),
    AdjustSfx(f32),
    Quit,
}

/// Position du bouton dans son écran (sert à la navigation clavier).
#[derive(Component, Clone, Copy)]
struct ButtonIndex(usize);

/// Bouton actuellement sélectionné via clavier ou souris.
#[derive(Resource, Default)]
struct MenuSelection(usize);

/// Compteur de boutons spawné dans l'écran courant (réinitialisé OnEnter).
#[derive(Resource, Default)]
struct ButtonCount(usize);

/// Polices UI. `display` (Pixelify Sans) pour titres + boutons + tout
/// le texte hors HUD numérique. `mono` (DejaVu Sans Mono) pour les
/// chiffres du HUD qui changent souvent : la mono évite que le texte
/// "danse" à chaque mise à jour.
#[derive(Resource)]
struct UiFont {
    /// Pixelify Sans Regular — UI principale, lisible et game-y.
    regular: Handle<Font>,
    /// Idem (la police variable sert les deux poids à des tailles
    /// pixel-art ; on garde le nom pour le code existant).
    bold: Handle<Font>,
    /// DejaVu Sans Mono — pour le HUD numérique.
    mono: Handle<Font>,
}

/// Texte dynamique affichant l'état d'un settings (ex: "Fullscreen : ON").
#[derive(Component, Clone, Copy)]
enum DynamicLabel {
    Fullscreen,
    MasterVolume,
    MusicVolume,
    SfxVolume,
}

// =========================================================== Plugin ===

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Charger les polices en build-time pour qu'elles soient dispo
        // dès les premiers Startup systems.
        let asset_server = app.world().resource::<AssetServer>().clone();
        let pixelify = asset_server.load::<Font>("fonts/PixelifySans-Regular.ttf");
        let ui_font = UiFont {
            regular: pixelify.clone(),
            bold: pixelify,
            mono: asset_server.load("fonts/DejaVuSansMono.ttf"),
        };
        app.insert_resource(ui_font)
            .init_resource::<MenuSelection>()
            .init_resource::<ButtonCount>()
            .add_systems(Startup, setup_hud)
            // Écrans
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_screen)
            .add_systems(OnEnter(GameState::HeroSelect), spawn_hero_select)
            .add_systems(OnExit(GameState::HeroSelect), despawn_screen)
            .add_systems(OnEnter(GameState::Settings), spawn_settings)
            .add_systems(OnExit(GameState::Settings), exit_settings)
            .add_systems(OnEnter(GameState::Credits), spawn_credits)
            .add_systems(OnExit(GameState::Credits), despawn_screen)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_screen)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_screen)
            .add_systems(OnEnter(GameState::Win), spawn_win_screen)
            .add_systems(OnExit(GameState::Win), exit_win)
            .add_systems(OnEnter(GameState::Playing), reset_player_for_run)
            // Update
            .add_systems(
                Update,
                (
                    button_interaction,
                    keyboard_navigation
                        .run_if(not(in_state(GameState::Playing))),
                    sync_dynamic_labels,
                    pulse_title,
                    toggle_pause_in_game.run_if(in_state(GameState::Playing)),
                    update_hud.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

// ============================================================ HUD ===

fn setup_hud(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn((
            HudTag,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(16.0),
                    left: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                HudLevel,
                TextBundle::from_section(
                    "Niveau 1",
                    TextStyle {
                        font: font.regular.clone(),
                        font_size: 22.0,
                        color: ACCENT_CYAN,
                    },
                ),
            ));
            p.spawn((
                HudDeaths,
                TextBundle::from_section(
                    "Morts : 0",
                    TextStyle {
                        font: font.mono.clone(),
                        font_size: 20.0,
                        color: HINT_COLOR,
                    },
                ),
            ));
            p.spawn((
                HudTime,
                TextBundle::from_section(
                    "Temps : 0.0 s",
                    TextStyle {
                        font: font.mono.clone(),
                        font_size: 20.0,
                        color: HINT_COLOR,
                    },
                ),
            ));
        });
}

fn update_hud(
    stats: Res<RunStats>,
    current_level: Res<CurrentLevel>,
    mut deaths_q: Query<&mut Text, (With<HudDeaths>, Without<HudTime>, Without<HudLevel>)>,
    mut time_q: Query<&mut Text, (With<HudTime>, Without<HudDeaths>, Without<HudLevel>)>,
    mut level_q: Query<&mut Text, (With<HudLevel>, Without<HudTime>, Without<HudDeaths>)>,
) {
    if let Ok(mut t) = deaths_q.get_single_mut() {
        t.sections[0].value = format!("Morts : {}", stats.deaths);
    }
    if let Ok(mut t) = time_q.get_single_mut() {
        t.sections[0].value = format!("Temps : {:.1} s", stats.time_seconds);
    }
    if let Ok(mut t) = level_q.get_single_mut() {
        t.sections[0].value = format!(
            "Niveau {} / {}  -  {}",
            current_level.0.number(),
            TOTAL_LEVELS,
            current_level.0.label(),
        );
    }
}

// =================================================== Layout helpers ===

fn spawn_overlay(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ScreenTag,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(28.0),
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                background_color: OVERLAY_BG.into(),
                ..default()
            },
        ))
        .id()
}

fn spawn_text(
    parent: &mut ChildBuilder,
    font: &UiFont,
    text: &str,
    size: f32,
    color: Color,
) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font: font.regular.clone(),
            font_size: size,
            color,
        },
    ));
}

fn spawn_title(parent: &mut ChildBuilder, font: &UiFont, text: &str) {
    parent.spawn((
        TitleText,
        TextBundle::from_section(
            text,
            TextStyle {
                font: font.bold.clone(),
                font_size: 72.0,
                color: TITLE_COLOR,
            },
        ),
    ));
}

fn spawn_subtitle(parent: &mut ChildBuilder, font: &UiFont, text: &str) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font: font.regular.clone(),
            font_size: 22.0,
            color: SUBTITLE_COLOR,
        },
    ));
}

fn spawn_button(
    parent: &mut ChildBuilder,
    counter: &mut ResMut<ButtonCount>,
    font: &UiFont,
    text: &str,
    action: MenuAction,
) {
    let font_handle = font.bold.clone();
    spawn_button_with_label(parent, counter, action, move |p| {
        p.spawn(TextBundle::from_section(
            text.to_string(),
            TextStyle {
                font: font_handle,
                font_size: 28.0,
                color: TITLE_COLOR,
            },
        ));
    });
}

fn spawn_button_with_label(
    parent: &mut ChildBuilder,
    counter: &mut ResMut<ButtonCount>,
    action: MenuAction,
    content: impl FnOnce(&mut ChildBuilder),
) {
    let index = counter.0;
    counter.0 += 1;
    parent
        .spawn((
            action,
            ButtonIndex(index),
            ButtonBundle {
                style: Style {
                    min_width: Val::Px(340.0),
                    height: Val::Px(54.0),
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    column_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: BTN_NORMAL.into(),
                border_color: BTN_BORDER.into(),
                ..default()
            },
        ))
        .with_children(content);
}

fn spawn_dynamic_label(parent: &mut ChildBuilder, font: &UiFont, kind: DynamicLabel) {
    parent.spawn((
        kind,
        TextBundle::from_section(
            "...",
            TextStyle {
                font: font.bold.clone(),
                font_size: 28.0,
                color: ACCENT_CYAN,
            },
        ),
    ));
}

/// Reset le compteur + la sélection avant de spawner un écran.
fn begin_screen(counter: &mut ResMut<ButtonCount>, selection: &mut ResMut<MenuSelection>) {
    counter.0 = 0;
    selection.0 = 0;
}

// ====================================================== Main menu ===

fn spawn_main_menu(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    save: Res<SaveData>,
    font: Res<UiFont>,
    asset_server: Res<AssetServer>,
) {
    begin_screen(&mut counter, &mut selection);
    let font = font.into_inner();

    // Conteneur racine plein écran : pas de couleur de fond, on va y
    // empiler l'image et l'overlay de texte.
    let root = commands
        .spawn((
            ScreenTag,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.entity(root).with_children(|p| {
        // Background : la scène rose à la Camille (camille-unknown-home).
        p.spawn(ImageBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            image: UiImage::new(asset_server.load("sprites/menu_background.png")),
            ..default()
        });

        // Overlay très léger juste pour assurer la lisibilité du texte
        // par-dessus le fond rose saturé.
        p.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(28.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            background_color: Color::srgba(0.10, 0.04, 0.12, 0.25).into(),
            ..default()
        })
        .with_children(|p| {
            spawn_title(p, font, "Adventure Timing");
            spawn_subtitle(p, font, "Une nuit sans étoiles à traverser");
            spawn_text(
                p,
                font,
                &format!("{} niveau{}  ·  double saut", TOTAL_LEVELS, if TOTAL_LEVELS > 1 { "x" } else { "" }),
                18.0,
                SUBTITLE_COLOR,
            );

        if save.runs_completed > 0 {
            let best_time = save
                .best_time
                .map(|t| format!("{t:.2} s"))
                .unwrap_or_else(|| "—".into());
            let best_deaths = save
                .fewest_deaths
                .map(|d| d.to_string())
                .unwrap_or_else(|| "—".into());
            spawn_text(
                p,
                font,
                &format!(
                    "Meilleur temps : {best_time}  ·  Moins de morts : {best_deaths}  ·  Runs : {}",
                    save.runs_completed
                ),
                18.0,
                ACCENT_CYAN,
            );
        }

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            if save.runs_completed > 0 {
                spawn_button(p, &mut counter, font, "Continuer", MenuAction::Continue);
            }
            spawn_button(p, &mut counter, font, "Nouvelle partie", MenuAction::GotoHeroSelect);
            spawn_button(p, &mut counter, font, "Paramètres", MenuAction::GotoSettings);
            spawn_button(p, &mut counter, font, "Crédits", MenuAction::GotoCredits);
            spawn_button(p, &mut counter, font, "Quitter", MenuAction::Quit);
        });

        spawn_text(
            p,
            font,
            "Z S pour naviguer  ·  Entree pour valider  ·  Souris OK",
            16.0,
            HINT_COLOR,
        );
        }); // ferme le with_children de l'overlay
    }); // ferme le with_children du root
}

// =================================================== Hero select ===

fn spawn_hero_select(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    font: Res<UiFont>,
    asset_server: Res<AssetServer>,
    selected: Res<SelectedHero>,
) {
    begin_screen(&mut counter, &mut selection);
    let font = font.into_inner();
    // Préselectionne le slot du héros courant
    selection.0 = Hero::all().iter().position(|h| *h == selected.0).unwrap_or(0);

    let root = spawn_overlay(&mut commands);
    commands.entity(root).with_children(|p| {
        spawn_title(p, font, "Choisis ton heros");
        spawn_subtitle(p, font, "Fleches gauche/droite, Entree pour valider");

        // Rangée de 3 cartes
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                margin: UiRect::vertical(Val::Px(28.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            for &hero in Hero::all() {
                spawn_hero_card(p, &mut counter, font, &asset_server, hero);
            }
        });

        spawn_text(
            p,
            font,
            "Selectionne une carte pour lancer la partie",
            16.0,
            HINT_COLOR,
        );
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(12.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Retour au menu", MenuAction::GotoMainMenu);
        });
    });
}

fn spawn_hero_card(
    parent: &mut ChildBuilder,
    counter: &mut ResMut<ButtonCount>,
    font: &UiFont,
    asset_server: &AssetServer,
    hero: Hero,
) {
    let index = counter.0;
    counter.0 += 1;
    let label = hero.label();
    let tagline = hero.tagline();
    let description = hero.description();
    let sprite = asset_server.load(hero.preview_path());
    parent
        .spawn((
            MenuAction::SelectHero(hero),
            ButtonIndex(index),
            ButtonBundle {
                style: Style {
                    width: Val::Px(240.0),
                    height: Val::Px(320.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BTN_NORMAL.into(),
                border_color: BTN_BORDER.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            // Sprite preview : on prend le frame 0 du sprite sheet via
            // Sprite::rect. Sprite sheet 168x36, frame 0 = (0,0,24,36).
            p.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(72.0),
                    height: Val::Px(108.0),
                    ..default()
                },
                image: UiImage::new(sprite),
                ..default()
            });
            p.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font: font.bold.clone(),
                    font_size: 26.0,
                    color: TITLE_COLOR,
                },
            ));
            p.spawn(TextBundle::from_section(
                tagline,
                TextStyle {
                    font: font.regular.clone(),
                    font_size: 16.0,
                    color: ACCENT_CYAN,
                },
            ));
            p.spawn(TextBundle::from_section(
                description,
                TextStyle {
                    font: font.regular.clone(),
                    font_size: 14.0,
                    color: SUBTITLE_COLOR,
                },
            ));
        });
}

// ======================================================= Settings ===

fn spawn_settings(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    font: Res<UiFont>,
) {
    begin_screen(&mut counter, &mut selection);
    let root = spawn_overlay(&mut commands);
    let font = font.into_inner();
    commands.entity(root).with_children(|p| {
        spawn_title(p, font, "Paramètres");
        spawn_subtitle(p, font, "Affichage et son");

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            let label_style = TextStyle {
                font: font.bold.clone(),
                font_size: 26.0,
                color: TITLE_COLOR,
            };
            // Fullscreen
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::ToggleFullscreen, |p| {
                p.spawn(TextBundle::from_section("Plein écran", s));
                spawn_dynamic_label(p, font, DynamicLabel::Fullscreen);
            });
            // Master volume
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustMaster(0.1), |p| {
                p.spawn(TextBundle::from_section("Volume général", s));
                spawn_dynamic_label(p, font, DynamicLabel::MasterVolume);
            });
            // Music volume
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustMusic(0.1), |p| {
                p.spawn(TextBundle::from_section("Musique", s));
                spawn_dynamic_label(p, font, DynamicLabel::MusicVolume);
            });
            // SFX volume
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustSfx(0.1), |p| {
                p.spawn(TextBundle::from_section("Effets", s));
                spawn_dynamic_label(p, font, DynamicLabel::SfxVolume);
            });

            spawn_button(p, &mut counter, font, "Retour", MenuAction::GotoMainMenu);
        });

        spawn_text(
            p,
            font,
            "Volumes : Entrée pour +10 %, Maj+Entrée pour -10 %",
            16.0,
            HINT_COLOR,
        );
    });
}

fn exit_settings(commands: Commands, q: Query<Entity, With<ScreenTag>>, settings: Res<Settings>) {
    save_settings(&settings);
    despawn_screen(commands, q);
}

// ======================================================== Credits ===

fn spawn_credits(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    font: Res<UiFont>,
) {
    begin_screen(&mut counter, &mut selection);
    let root = spawn_overlay(&mut commands);
    let font = font.into_inner();
    commands.entity(root).with_children(|p| {
        spawn_title(p, font, "Crédits");

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                margin: UiRect::vertical(Val::Px(16.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_text(p, font, "Direction artistique inspirée par", 22.0, SUBTITLE_COLOR);
            spawn_text(p, font, "Camille Unknown", 38.0, ACCENT_CYAN);
            spawn_text(p, font, "artstation.com/camilleunknown", 18.0, HINT_COLOR);
            spawn_text(p, font, "", 12.0, HINT_COLOR);
            spawn_text(p, font, "Moteur", 22.0, SUBTITLE_COLOR);
            spawn_text(p, font, "Bevy 0.14 — bevyengine.org", 22.0, ACCENT_AMBER);
            spawn_text(p, font, "", 12.0, HINT_COLOR);
            spawn_text(p, font, "Code & assets procéduraux", 22.0, SUBTITLE_COLOR);
            spawn_text(p, font, "Karim — avec Claude Code", 22.0, TITLE_COLOR);
        });

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Retour", MenuAction::GotoMainMenu);
        });
    });
}

// ========================================================== Pause ===

fn spawn_pause_menu(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    stats: Res<RunStats>,
    font: Res<UiFont>,
) {
    begin_screen(&mut counter, &mut selection);
    let root = spawn_overlay(&mut commands);
    let font = font.into_inner();
    let time = stats.time_seconds;
    let deaths = stats.deaths;
    commands.entity(root).with_children(|p| {
        spawn_title(p, font, "Pause");
        spawn_text(
            p,
            font,
            &format!("Temps écoulé : {time:.1} s · Morts : {deaths}"),
            20.0,
            SUBTITLE_COLOR,
        );

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Reprendre", MenuAction::Resume);
            spawn_button(p, &mut counter, font, "Recommencer", MenuAction::Restart);
            spawn_button(p, &mut counter, font, "Quitter au menu", MenuAction::GotoMainMenu);
        });

        spawn_text(p, font, "Échap pour reprendre", 16.0, HINT_COLOR);
    });
}

// ====================================================== Game Over ===

fn spawn_game_over(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    stats: Res<RunStats>,
    font: Res<UiFont>,
) {
    begin_screen(&mut counter, &mut selection);
    let root = spawn_overlay(&mut commands);
    let font = font.into_inner();
    let deaths = stats.deaths;
    commands.entity(root).with_children(|p| {
        spawn_text(p, font, "Game Over", 72.0, Color::srgb(0.95, 0.45, 0.45));
        spawn_text(
            p,
            font,
            &format!("Morts cumulées : {deaths}"),
            22.0,
            SUBTITLE_COLOR,
        );

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Recommencer", MenuAction::Restart);
            spawn_button(p, &mut counter, font, "Menu principal", MenuAction::GotoMainMenu);
        });
    });
}

// =========================================================== Win ===

fn spawn_win_screen(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    stats: Res<RunStats>,
    save: Res<SaveData>,
    font: Res<UiFont>,
) {
    begin_screen(&mut counter, &mut selection);
    let root = spawn_overlay(&mut commands);
    let font = font.into_inner();
    let deaths = stats.deaths;
    let secs = stats.time_seconds;
    let is_new_best_time = save.best_time.map_or(true, |b| secs < b);
    let is_new_best_deaths = save.fewest_deaths.map_or(true, |d| deaths < d);

    commands.entity(root).with_children(|p| {
        spawn_text(p, font, "Bravo !", 80.0, ACCENT_CYAN);
        spawn_text(
            p,
            font,
            &format!("Les {TOTAL_LEVELS} niveaux sont terminés"),
            22.0,
            ACCENT_AMBER,
        );
        spawn_text(
            p,
            font,
            &format!("Temps : {secs:.2} s   ·   Morts : {deaths}"),
            26.0,
            SUBTITLE_COLOR,
        );

        if is_new_best_time || is_new_best_deaths {
            let mut record = String::from("Nouveau record :");
            if is_new_best_time {
                record.push_str(" meilleur temps");
            }
            if is_new_best_deaths {
                if is_new_best_time {
                    record.push_str(" et");
                }
                record.push_str(" moins de morts");
            }
            spawn_text(p, font, &record, 22.0, ACCENT_AMBER);
        }

        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Rejouer", MenuAction::Restart);
            spawn_button(p, &mut counter, font, "Menu principal", MenuAction::GotoMainMenu);
        });
    });
}

/// Enregistre les stats du run dans la save AVANT de despawn l'écran.
fn exit_win(
    commands: Commands,
    q: Query<Entity, With<ScreenTag>>,
    stats: Res<RunStats>,
    mut save: ResMut<SaveData>,
) {
    save.record_run(stats.time_seconds, stats.deaths);
    save_data(&save);
    despawn_screen(commands, q);
}

// ================================================ Despawn helper ===

fn despawn_screen(mut commands: Commands, q: Query<Entity, With<ScreenTag>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

// ============================================== Interactions ===

#[allow(clippy::too_many_arguments)]
fn button_interaction(
    mut q: Query<
        (
            &Interaction,
            &MenuAction,
            &ButtonIndex,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut selection: ResMut<MenuSelection>,
    mut next: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut settings: ResMut<Settings>,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut exit_event: EventWriter<AppExit>,
    mut selected_hero: ResMut<SelectedHero>,
    mut save_data: ResMut<SaveData>,
    mut current_level: ResMut<crate::world::CurrentLevel>,
) {
    for (interaction, action, index, mut bg, mut border) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                *bg = BTN_PRESSED.into();
                *border = BTN_BORDER_SELECTED.into();
                selection.0 = index.0;
                trigger_action(
                    *action,
                    &mut next,
                    &state,
                    &mut settings,
                    &mut window_q,
                    &mut exit_event,
                    &mut selected_hero,
                    &mut save_data,
                    &mut current_level,
                );
            }
            Interaction::Hovered => {
                selection.0 = index.0;
                *bg = BTN_HOVER.into();
                *border = BTN_BORDER_SELECTED.into();
            }
            Interaction::None => {
                if index.0 == selection.0 {
                    *bg = BTN_SELECTED.into();
                    *border = BTN_BORDER_SELECTED.into();
                } else {
                    *bg = BTN_NORMAL.into();
                    *border = BTN_BORDER.into();
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn keyboard_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    counter: Res<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    mut q: Query<(
        &MenuAction,
        &ButtonIndex,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut next: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut settings: ResMut<Settings>,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut exit_event: EventWriter<AppExit>,
    mut selected_hero: ResMut<SelectedHero>,
    mut save_data: ResMut<SaveData>,
    mut current_level: ResMut<crate::world::CurrentLevel>,
) {
    if counter.0 == 0 {
        return;
    }

    // Échap : retour rapide depuis sous-menus
    if keys.just_pressed(KeyCode::Escape)
        && matches!(
            *state.get(),
            GameState::Settings | GameState::Credits
        )
    {
        next.set(GameState::MainMenu);
        return;
    }

    let prev = selection.0;
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.0 = (selection.0 + 1) % counter.0;
    } else if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        selection.0 = (selection.0 + counter.0 - 1) % counter.0;
    }

    if prev != selection.0 {
        for (_, index, mut bg, mut border) in &mut q {
            if index.0 == selection.0 {
                *bg = BTN_SELECTED.into();
                *border = BTN_BORDER_SELECTED.into();
            } else {
                *bg = BTN_NORMAL.into();
                *border = BTN_BORDER.into();
            }
        }
    }

    let activate = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if activate {
        for (action, index, _, _) in &mut q {
            if index.0 == selection.0 {
                let action = match (*action, shift_held) {
                    (MenuAction::AdjustMaster(v), true) => MenuAction::AdjustMaster(-v),
                    (MenuAction::AdjustMusic(v), true) => MenuAction::AdjustMusic(-v),
                    (MenuAction::AdjustSfx(v), true) => MenuAction::AdjustSfx(-v),
                    (other, _) => other,
                };
                trigger_action(
                    action,
                    &mut next,
                    &state,
                    &mut settings,
                    &mut window_q,
                    &mut exit_event,
                    &mut selected_hero,
                    &mut save_data,
                    &mut current_level,
                );
                return;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn trigger_action(
    action: MenuAction,
    next: &mut ResMut<NextState<GameState>>,
    state: &Res<State<GameState>>,
    settings: &mut ResMut<Settings>,
    window_q: &mut Query<&mut Window, With<PrimaryWindow>>,
    exit_event: &mut EventWriter<AppExit>,
    selected_hero: &mut ResMut<SelectedHero>,
    save_data: &mut ResMut<SaveData>,
    current_level: &mut ResMut<crate::world::CurrentLevel>,
) {
    match action {
        MenuAction::StartNewGame | MenuAction::Continue | MenuAction::ConfirmHero => {
            // Démarrer une partie remet le niveau à 1.
            current_level.0 = crate::world::LevelId::default();
            save_data.selected_hero = selected_hero.0;
            crate::save::save_data(save_data);
            next.set(GameState::Playing);
        }
        MenuAction::Resume => {
            if matches!(state.get(), GameState::Paused) {
                next.set(GameState::Playing);
            }
        }
        MenuAction::Restart => next.set(GameState::Playing),
        MenuAction::GotoHeroSelect => next.set(GameState::HeroSelect),
        MenuAction::GotoSettings => next.set(GameState::Settings),
        MenuAction::GotoCredits => next.set(GameState::Credits),
        MenuAction::GotoMainMenu => next.set(GameState::MainMenu),
        MenuAction::SelectHero(hero) => {
            selected_hero.0 = hero;
            // Activer une carte démarre directement la partie au niveau 1.
            current_level.0 = crate::world::LevelId::default();
            save_data.selected_hero = hero;
            crate::save::save_data(save_data);
            next.set(GameState::Playing);
        }
        MenuAction::ToggleFullscreen => {
            settings.fullscreen = !settings.fullscreen;
            if let Ok(mut window) = window_q.get_single_mut() {
                window.mode = if settings.fullscreen {
                    WindowMode::BorderlessFullscreen
                } else {
                    WindowMode::Windowed
                };
            }
        }
        MenuAction::AdjustMaster(delta) => {
            settings.master_volume = (settings.master_volume + delta).clamp(0.0, 1.0);
        }
        MenuAction::AdjustMusic(delta) => {
            settings.music_volume = (settings.music_volume + delta).clamp(0.0, 1.0);
        }
        MenuAction::AdjustSfx(delta) => {
            settings.sfx_volume = (settings.sfx_volume + delta).clamp(0.0, 1.0);
        }
        MenuAction::Quit => {
            exit_event.send(AppExit::Success);
        }
    }
}

// =============================================== Dynamic labels ===

fn sync_dynamic_labels(
    settings: Res<Settings>,
    mut q: Query<(&DynamicLabel, &mut Text)>,
) {
    for (kind, mut text) in &mut q {
        let value = match *kind {
            DynamicLabel::Fullscreen => {
                if settings.fullscreen {
                    "Activé"
                } else {
                    "Désactivé"
                }
                .to_string()
            }
            DynamicLabel::MasterVolume => format!("{}%", (settings.master_volume * 100.0) as u32),
            DynamicLabel::MusicVolume => format!("{}%", (settings.music_volume * 100.0) as u32),
            DynamicLabel::SfxVolume => format!("{}%", (settings.sfx_volume * 100.0) as u32),
        };
        text.sections[0].value = value;
    }
}

// ============================================== Title pulse fx ===

fn pulse_title(time: Res<Time>, mut q: Query<&mut Text, With<TitleText>>) {
    let t = time.elapsed_seconds();
    // Une respiration douce de la luminosité du titre
    let alpha = 0.85 + 0.15 * (t * 1.6).sin();
    for mut text in &mut q {
        let color = TITLE_COLOR.to_srgba();
        text.sections[0].style.color =
            Color::srgba(color.red, color.green, color.blue, alpha);
    }
}

// ============================================== Misc gameplay ===

fn toggle_pause_in_game(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::Paused);
    }
}


#[allow(clippy::too_many_arguments)]
fn reset_player_for_run(
    mut player: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PlayerController,
            &mut Grounded,
        ),
        (With<Player>, Without<Checkpoint>),
    >,
    mut respawn: ResMut<RespawnPoint>,
    mut stats: ResMut<RunStats>,
    mut checkpoints: Query<(&mut Checkpoint, &mut Sprite), Without<Player>>,
) {
    if let Ok((mut transform, mut velocity, mut ctrl, mut grounded)) = player.get_single_mut() {
        transform.translation = PLAYER_SPAWN.extend(transform.translation.z);
        velocity.0 = Vec2::ZERO;
        *ctrl = PlayerController::default();
        grounded.0 = false;
    }
    respawn.0 = PLAYER_SPAWN;
    *stats = RunStats::default();

    for (mut chk, mut sprite) in &mut checkpoints {
        chk.triggered = false;
        sprite.color = Color::WHITE;
    }
}
