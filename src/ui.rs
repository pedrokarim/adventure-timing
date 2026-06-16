//! Menus interactifs, HUD et écrans de fin.
//!
//! Tous les écrans (Main, Settings, Credits, Pause, GameOver, Win)
//! utilisent une UI commune : entête (titre + sous-titre), bloc central
//! de boutons cliquables/navigables clavier, footer.
//!
//! Boutons : highlight au survol (souris), navigation clavier (Up/Down +
//! Enter), action déclenchée via `MenuAction` (composant attaché à
//! chaque bouton).

use crate::audio::CheckpointReached;
use crate::heroes::{Hero, SelectedHero};
use crate::items::{ActiveEffects, ItemKind, ItemPickedUp};
use crate::level::{Checkpoint, RespawnPoint};
use crate::physics::{Grounded, Velocity};
use crate::player::{Player, PlayerController, PlayerHp};
use crate::throwables::Inventory;
use crate::save::{save_data, save_settings, SaveData, Settings};
use crate::states::{GameState, RunStats};
use crate::world::{CurrentLevel, LevelId, PLAYER_SPAWN, TOTAL_LEVELS};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::text::Font;
use bevy::window::{PrimaryWindow, WindowMode};

// ============================================================ Style ===
// Direction "papyrus / bois bevel" — palette chaude rustique inspirée de
// Stardew Valley / Don't Starve, prototypée dans docs/ui-prototypes/.

/// Overlay des menus : sombre wine très semi-transparent pour laisser la
/// scène passer derrière tout en assurant la lisibilité du papyrus.
const OVERLAY_BG: Color = Color::srgba(0.18, 0.10, 0.06, 0.60);

const PAPYRUS: Color = Color::srgb(0.953, 0.878, 0.706); // #F3E0B4
const WOOD_LIGHT: Color = Color::srgb(0.847, 0.725, 0.478); // #D8B97A
const WOOD: Color = Color::srgb(0.776, 0.604, 0.365); // #C69A5D
const WOOD_DARK: Color = Color::srgb(0.420, 0.231, 0.094); // #6B3B18
const WOOD_DARKEST: Color = Color::srgb(0.290, 0.176, 0.102); // #4A2D1A
const CREAM: Color = Color::srgb(1.000, 0.973, 0.878); // #FFF8E0
const GOLD: Color = Color::srgb(1.000, 0.874, 0.522); // #FFDF85

/// Texte principal sur fond papyrus.
const TITLE_COLOR: Color = CREAM;
const SUBTITLE_COLOR: Color = Color::srgb(0.847, 0.725, 0.478);
const HINT_COLOR: Color = Color::srgb(0.788, 0.690, 0.502);
const ACCENT_CYAN: Color = GOLD; // l'accent "level" passe en or pour cohérence
const ACCENT_AMBER: Color = GOLD;

/// Bouton bevel rétro : bois clair par défaut, plus clair en hover,
/// plus sombre quand enfoncé. Bordure très sombre pour le contraste.
const BTN_NORMAL: Color = Color::srgb(0.776, 0.604, 0.365); // WOOD
const BTN_HOVER: Color = Color::srgb(0.847, 0.725, 0.478); // WOOD_LIGHT
const BTN_PRESSED: Color = Color::srgb(0.604, 0.451, 0.247);
const BTN_SELECTED: Color = Color::srgb(0.812, 0.671, 0.420);
const BTN_BORDER: Color = WOOD_DARKEST;
const BTN_BORDER_SELECTED: Color = GOLD;

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
struct HudHearts;

#[derive(Component)]
struct HeartIcon(u32);

/// Container des toasts (notifications). Spawné une fois au setup_hud.
#[derive(Component)]
struct ToastsContainer;

/// Une notification toast active. TTL puis fade out.
#[derive(Component)]
struct Toast {
    pub remaining: f32,
    pub initial: f32,
}

/// Container des slots d'inventaire (3 throwables) en bas-droite.
#[derive(Component)]
struct InventoryContainer;

#[derive(Component)]
struct InventorySlot(usize);

/// Container des effets actifs en haut-droite.
#[derive(Component)]
struct EffectsContainer;

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
    GotoLevelMap,
    GotoSettings,
    GotoCredits,
    GotoMainMenu,
    SelectHero(Hero),
    SelectLevel(LevelId),
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

/// Marqueur qui empêche button_interaction de toucher la couleur de
/// fond. Utile pour les cartes héros (fond papyrus statique, seule
/// la bordure réagit au hover/selected).
#[derive(Component)]
struct LockedBackground;

/// Bouton actuellement sélectionné via clavier ou souris.
#[derive(Resource, Default)]
struct MenuSelection(usize);

/// Compteur de boutons spawné dans l'écran courant (réinitialisé OnEnter).
#[derive(Resource, Default)]
struct ButtonCount(usize);

/// Polices UI. Direction papyrus :
/// - `display` (Cinzel Bold) : titres + HUD + cartes héros (gothic serif)
/// - `regular` (Crimson Pro) : sous-titres, body, descriptions
/// - `bold` = display pour compat avec le code existant
/// - `mono` (DejaVu Sans Mono) : pour fallback / chiffres si besoin
#[derive(Resource)]
struct UiFont {
    /// Crimson Pro Regular — texte courant, élégant et lisible.
    regular: Handle<Font>,
    /// Cinzel Bold — titres + boutons + HUD (gothic serif).
    bold: Handle<Font>,
    /// Idem que bold, alias sémantique.
    display: Handle<Font>,
    /// DejaVu Mono — fallback technique.
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

/// Composant attaché à un slider bevel papyrus. Stocke quel volume il
/// représente. Affiché comme bar fill dans une frame bois.
#[derive(Component, Clone, Copy)]
enum SliderKind {
    Master,
    Music,
    Sfx,
}

#[derive(Component)]
struct SliderFill;

/// Composant attaché à un toggle tick papyrus.
#[derive(Component)]
struct FullscreenToggle;

// =========================================================== Plugin ===

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Charger les polices en build-time pour qu'elles soient dispo
        // dès les premiers Startup systems.
        let asset_server = app.world().resource::<AssetServer>().clone();
        let cinzel = asset_server.load::<Font>("fonts/Cinzel-Bold.ttf");
        let ui_font = UiFont {
            regular: asset_server.load("fonts/CrimsonPro-Regular.ttf"),
            bold: cinzel.clone(),
            display: cinzel,
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
            .add_systems(OnEnter(GameState::LevelMap), spawn_level_map)
            .add_systems(OnExit(GameState::LevelMap), despawn_screen)
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
                    spawn_pickup_toasts.run_if(in_state(GameState::Playing)),
                    spawn_checkpoint_toast.run_if(in_state(GameState::Playing)),
                    tick_toasts,
                    update_inventory_hud.run_if(in_state(GameState::Playing)),
                    update_effects_hud.run_if(in_state(GameState::Playing)),
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
            // H4 Gothic serif : Cinzel, accent or, espacements lettres
            p.spawn((
                HudLevel,
                TextBundle::from_section(
                    "Niveau I · II",
                    TextStyle {
                        font: font.display.clone(),
                        font_size: 22.0,
                        color: GOLD,
                    },
                ),
            ));
            p.spawn((
                HudDeaths,
                TextBundle::from_section(
                    "Mortes — 0",
                    TextStyle {
                        font: font.display.clone(),
                        font_size: 18.0,
                        color: CREAM,
                    },
                ),
            ));
            p.spawn((
                HudTime,
                TextBundle::from_section(
                    "Temps — 0.0 s",
                    TextStyle {
                        font: font.display.clone(),
                        font_size: 18.0,
                        color: CREAM,
                    },
                ),
            ));

            // Ligne de cœurs (max 5 affichés, plus = juste +N)
            p.spawn((
                HudHearts,
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                for i in 0..5 {
                    p.spawn((
                        HeartIcon(i),
                        TextBundle::from_section(
                            "♥",
                            TextStyle {
                                font: font.display.clone(),
                                font_size: 22.0,
                                color: Color::srgb(0.30, 0.12, 0.16),
                            },
                        ),
                    ));
                }
            });
        });

    // Container des toasts (top-right)
    commands.spawn((
        HudTag,
        ToastsContainer,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        },
    ));

    // Inventaire (bottom-right) — 3 slots bevel rétro
    commands
        .spawn((
            HudTag,
            InventoryContainer,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    right: Val::Px(20.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            for i in 0..3 {
                p.spawn((
                    InventorySlot(i),
                    NodeBundle {
                        style: Style {
                            width: Val::Px(44.0),
                            height: Val::Px(44.0),
                            border: UiRect::all(Val::Px(3.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: WOOD_LIGHT.into(),
                        border_color: WOOD_DARK.into(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "",
                        TextStyle {
                            font: font.display.clone(),
                            font_size: 22.0,
                            color: WOOD_DARKEST,
                        },
                    ));
                });
            }
        });

    // Container des effets actifs (sous les toasts, à droite)
    commands.spawn((
        HudTag,
        EffectsContainer,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(120.0),
                right: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        },
    ));
}

#[allow(clippy::type_complexity)]
fn update_hud(
    stats: Res<RunStats>,
    current_level: Res<CurrentLevel>,
    player_hp: Query<&PlayerHp, With<Player>>,
    mut deaths_q: Query<&mut Text, (With<HudDeaths>, Without<HudTime>, Without<HudLevel>, Without<HeartIcon>)>,
    mut time_q: Query<&mut Text, (With<HudTime>, Without<HudDeaths>, Without<HudLevel>, Without<HeartIcon>)>,
    mut level_q: Query<&mut Text, (With<HudLevel>, Without<HudTime>, Without<HudDeaths>, Without<HeartIcon>)>,
    mut hearts_q: Query<(&HeartIcon, &mut Text), (Without<HudLevel>, Without<HudDeaths>, Without<HudTime>)>,
) {
    if let Ok(mut t) = deaths_q.get_single_mut() {
        t.sections[0].value = format!("Mortes — {}", stats.deaths);
    }
    if let Ok(mut t) = time_q.get_single_mut() {
        t.sections[0].value = format!("Temps — {:.1} s", stats.time_seconds);
    }
    if let Ok(mut t) = level_q.get_single_mut() {
        t.sections[0].value = format!(
            "Niveau {} · {}   {}",
            current_level.0.number(),
            TOTAL_LEVELS,
            current_level.0.label(),
        );
    }
    if let Ok(hp) = player_hp.get_single() {
        for (icon, mut text) in &mut hearts_q {
            if icon.0 < hp.max {
                if icon.0 < hp.current {
                    text.sections[0].value = "♥".to_string();
                    text.sections[0].style.color = Color::srgb(0.93, 0.30, 0.36);
                } else {
                    text.sections[0].value = "♡".to_string();
                    text.sections[0].style.color = Color::srgb(0.40, 0.16, 0.20);
                }
            } else {
                text.sections[0].value = "".to_string();
            }
        }
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
                font: font.display.clone(),
                font_size: 76.0,
                color: GOLD,
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
            color: PAPYRUS,
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
    let font_handle = font.display.clone();
    spawn_button_with_label(parent, counter, action, move |p| {
        p.spawn(TextBundle::from_section(
            text.to_string(),
            TextStyle {
                font: font_handle,
                font_size: 24.0,
                color: WOOD_DARKEST,
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
                    height: Val::Px(56.0),
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(3.0)),
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
                font: font.display.clone(),
                font_size: 22.0,
                color: GOLD,
            },
        ),
    ));
}

/// T4 — Tick papyrus : carré crème avec ✓ noir quand activé.
fn spawn_tick_papyrus(parent: &mut ChildBuilder, font: &UiFont) {
    parent
        .spawn((
            FullscreenToggle,
            NodeBundle {
                style: Style {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(3.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: PAPYRUS.into(),
                border_color: WOOD_DARKEST.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font: font.display.clone(),
                    font_size: 26.0,
                    color: WOOD_DARKEST,
                },
            ));
        });
}

/// S4 — Bevel papyrus slider : cadre bois sombre avec fill bois clair
/// graduel.
fn spawn_bevel_slider(parent: &mut ChildBuilder, kind: SliderKind) {
    parent
        .spawn((
            kind,
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(3.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: WOOD_DARKEST.into(),
                border_color: WOOD_DARK.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                SliderFill,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(60.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: WOOD_LIGHT.into(),
                    ..default()
                },
            ));
        });
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
        // Background : la scène rose du menu principal.
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
            spawn_button(p, &mut counter, font, "Carte du voyage", MenuAction::GotoLevelMap);
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

// =================================================== Level map ===

fn spawn_level_map(
    mut commands: Commands,
    mut counter: ResMut<ButtonCount>,
    mut selection: ResMut<MenuSelection>,
    font: Res<UiFont>,
    save: Res<SaveData>,
) {
    begin_screen(&mut counter, &mut selection);
    let font = font.into_inner();
    let highest = save.highest_level;

    let root = spawn_overlay(&mut commands);
    commands.entity(root).with_children(|p| {
        spawn_title(p, font, "Carte du voyage");
        spawn_subtitle(p, font, "Cinq etapes a traverser");

        // Row de 5 medallions
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                margin: UiRect::vertical(Val::Px(28.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            for (i, &level) in LevelId::all().iter().enumerate() {
                let unlocked = level.number() <= highest;
                spawn_level_medallion(p, &mut counter, font, level, unlocked);
                // Pointillés entre médaillons (sauf après le dernier)
                if i < LevelId::all().len() - 1 {
                    spawn_path_dots(p);
                }
            }
        });

        spawn_text(p, font, "Clique sur un niveau debloque pour y aller", 16.0, HINT_COLOR);

        p.spawn(NodeBundle {
            style: Style {
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            spawn_button(p, &mut counter, font, "Retour au menu", MenuAction::GotoMainMenu);
        });
    });
}

fn spawn_level_medallion(
    parent: &mut ChildBuilder,
    counter: &mut ResMut<ButtonCount>,
    font: &UiFont,
    level: LevelId,
    unlocked: bool,
) {
    let index = counter.0;
    counter.0 += 1;

    let (bg_color, border_color, text_color, num_color) = if unlocked {
        (PAPYRUS, WOOD_DARK, WOOD_DARKEST, WOOD_DARKEST)
    } else {
        (
            Color::srgba(0.50, 0.40, 0.30, 0.5),
            Color::srgba(0.20, 0.13, 0.06, 0.8),
            Color::srgba(0.40, 0.30, 0.20, 0.8),
            Color::srgba(0.30, 0.20, 0.10, 0.8),
        )
    };
    let action = if unlocked {
        MenuAction::SelectLevel(level)
    } else {
        MenuAction::GotoLevelMap // no-op, juste un placeholder
    };

    parent
        .spawn((
            action,
            ButtonIndex(index),
            LockedBackground,
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(210.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: bg_color.into(),
                border_color: border_color.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            // 4 nails dans les coins (carte bois clouté)
            for (top, left, right, bottom) in [
                (Some(2.0), Some(2.0), None, None),
                (Some(2.0), None, Some(2.0), None),
                (None, Some(2.0), None, Some(2.0)),
                (None, None, Some(2.0), Some(2.0)),
            ] {
                p.spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: top.map(Val::Px).unwrap_or(Val::Auto),
                        left: left.map(Val::Px).unwrap_or(Val::Auto),
                        right: right.map(Val::Px).unwrap_or(Val::Auto),
                        bottom: bottom.map(Val::Px).unwrap_or(Val::Auto),
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                    background_color: border_color.into(),
                    ..default()
                });
            }

            // Numéro romain (gros)
            p.spawn(TextBundle::from_section(
                level.roman(),
                TextStyle {
                    font: font.display.clone(),
                    font_size: 42.0,
                    color: num_color,
                },
            ));

            // Badge couleur (32x32 carré papyrus contenant la badge_color)
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: level.badge_color().into(),
                border_color: border_color.into(),
                ..default()
            });

            // Label du niveau
            p.spawn(TextBundle::from_section(
                level.label(),
                TextStyle {
                    font: font.display.clone(),
                    font_size: 14.0,
                    color: text_color,
                },
            ));

            // Tagline
            p.spawn(TextBundle::from_section(
                level.tagline(),
                TextStyle {
                    font: font.regular.clone(),
                    font_size: 11.0,
                    color: text_color,
                },
            ));

            // Status (étoile / cadenas)
            p.spawn(TextBundle::from_section(
                if unlocked { "✓" } else { "verr." },
                TextStyle {
                    font: font.display.clone(),
                    font_size: 16.0,
                    color: if unlocked { GOLD } else { text_color },
                },
            ));
        });
}

fn spawn_path_dots(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(20.0),
                height: Val::Px(210.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            for _ in 0..3 {
                p.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        ..default()
                    },
                    background_color: WOOD_DARK.into(),
                    ..default()
                });
            }
        });
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
    // HC5 Cadre bois clouté : carte papyrus avec bordure bois épaisse +
    // 4 nails simulés par des dots en absolu dans les coins.
    parent
        .spawn((
            MenuAction::SelectHero(hero),
            ButtonIndex(index),
            LockedBackground,
            ButtonBundle {
                style: Style {
                    width: Val::Px(240.0),
                    height: Val::Px(320.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(18.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: PAPYRUS.into(),
                border_color: WOOD_DARK.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            // 4 nails dans les coins (mini cercles d'ombre)
            for (top, left, right, bottom) in [
                (Some(2.0), Some(2.0), None, None),
                (Some(2.0), None, Some(2.0), None),
                (None, Some(2.0), None, Some(2.0)),
                (None, None, Some(2.0), Some(2.0)),
            ] {
                p.spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: top.map(Val::Px).unwrap_or(Val::Auto),
                        left: left.map(Val::Px).unwrap_or(Val::Auto),
                        right: right.map(Val::Px).unwrap_or(Val::Auto),
                        bottom: bottom.map(Val::Px).unwrap_or(Val::Auto),
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: WOOD_DARKEST.into(),
                    border_color: WOOD_LIGHT.into(),
                    ..default()
                });
            }

            // Sprite preview encadré
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(76.0),
                    height: Val::Px(112.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::srgba(0.290, 0.176, 0.102, 0.08).into(),
                border_color: WOOD_DARK.into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(90.0),
                        ..default()
                    },
                    image: UiImage::new(sprite),
                    ..default()
                });
            });
            p.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font: font.display.clone(),
                    font_size: 22.0,
                    color: WOOD_DARKEST,
                },
            ));
            p.spawn(TextBundle::from_section(
                tagline,
                TextStyle {
                    font: font.regular.clone(),
                    font_size: 15.0,
                    color: WOOD_DARK,
                },
            ));
            p.spawn(TextBundle::from_section(
                description,
                TextStyle {
                    font: font.regular.clone(),
                    font_size: 13.0,
                    color: WOOD_DARK,
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
                font: font.display.clone(),
                font_size: 22.0,
                color: WOOD_DARKEST,
            };
            // T4 Tick papyrus pour Plein écran
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::ToggleFullscreen, |p| {
                p.spawn(TextBundle::from_section("Plein écran", s));
                spawn_tick_papyrus(p, font);
            });
            // S4 Bevel papyrus pour les 3 volumes
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustMaster(0.1), |p| {
                p.spawn(TextBundle::from_section("Volume général", s));
                spawn_bevel_slider(p, SliderKind::Master);
            });
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustMusic(0.1), |p| {
                p.spawn(TextBundle::from_section("Musique", s));
                spawn_bevel_slider(p, SliderKind::Music);
            });
            let s = label_style.clone();
            spawn_button_with_label(p, &mut counter, MenuAction::AdjustSfx(0.1), |p| {
                p.spawn(TextBundle::from_section("Effets", s));
                spawn_bevel_slider(p, SliderKind::Sfx);
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
            Has<LockedBackground>,
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
    for (interaction, action, index, mut bg, mut border, locked_bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                if !locked_bg {
                    *bg = BTN_PRESSED.into();
                }
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
                if !locked_bg {
                    *bg = BTN_HOVER.into();
                }
                *border = BTN_BORDER_SELECTED.into();
            }
            Interaction::None => {
                if index.0 == selection.0 {
                    if !locked_bg {
                        *bg = BTN_SELECTED.into();
                    }
                    *border = BTN_BORDER_SELECTED.into();
                } else {
                    if !locked_bg {
                        *bg = BTN_NORMAL.into();
                    }
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
        Has<LockedBackground>,
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
        for (_, index, mut bg, mut border, locked_bg) in &mut q {
            if index.0 == selection.0 {
                if !locked_bg {
                    *bg = BTN_SELECTED.into();
                }
                *border = BTN_BORDER_SELECTED.into();
            } else {
                if !locked_bg {
                    *bg = BTN_NORMAL.into();
                }
                *border = BTN_BORDER.into();
            }
        }
    }

    let activate = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if activate {
        for (action, index, _, _, _) in &mut q {
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
        MenuAction::GotoLevelMap => next.set(GameState::LevelMap),
        MenuAction::GotoSettings => next.set(GameState::Settings),
        MenuAction::GotoCredits => next.set(GameState::Credits),
        MenuAction::GotoMainMenu => next.set(GameState::MainMenu),
        MenuAction::SelectLevel(level) => {
            current_level.0 = level;
            next.set(GameState::Playing);
        }
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
    mut toggles: Query<(&FullscreenToggle, &Children), Without<DynamicLabel>>,
    mut toggle_texts: Query<&mut Text, (Without<DynamicLabel>, Without<SliderKind>)>,
    mut sliders: Query<(&SliderKind, &Children), Without<DynamicLabel>>,
    mut fills: Query<&mut Style, (With<SliderFill>, Without<DynamicLabel>)>,
) {
    // Texte (fallback ancien système)
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

    // Toggle papyrus : ✓ si activé
    for (_, children) in &mut toggles {
        for child in children {
            if let Ok(mut text) = toggle_texts.get_mut(*child) {
                text.sections[0].value =
                    if settings.fullscreen { "✓".to_string() } else { "".to_string() };
            }
        }
    }

    // Sliders : fill % selon volume
    for (kind, children) in &mut sliders {
        let value = match *kind {
            SliderKind::Master => settings.master_volume,
            SliderKind::Music => settings.music_volume,
            SliderKind::Sfx => settings.sfx_volume,
        };
        for child in children {
            if let Ok(mut style) = fills.get_mut(*child) {
                style.width = Val::Percent((value * 100.0).clamp(0.0, 100.0));
            }
        }
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

// ====================================================== Toasts ===

const TOAST_DURATION: f32 = 2.5;

fn spawn_toast(
    commands: &mut Commands,
    container: Entity,
    font: &UiFont,
    text: &str,
    style: ToastStyle,
) {
    let (bg, border, text_color) = match style {
        ToastStyle::Pickup => (CREAM, GOLD, WOOD_DARKEST),
        ToastStyle::Papyrus => (PAPYRUS, WOOD_DARKEST, WOOD_DARKEST),
    };
    commands.entity(container).with_children(|p| {
        p.spawn((
            Toast {
                remaining: TOAST_DURATION,
                initial: TOAST_DURATION,
            },
            NodeBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: bg.into(),
                border_color: border.into(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font: font.display.clone(),
                    font_size: 18.0,
                    color: text_color,
                },
            ));
        });
    });
}

#[derive(Clone, Copy)]
enum ToastStyle {
    /// N2 — Filled accent (pickup d'item)
    Pickup,
    /// N3 — Papyrus bois (checkpoint, record)
    Papyrus,
}

fn spawn_pickup_toasts(
    mut commands: Commands,
    mut events: EventReader<ItemPickedUp>,
    container: Query<Entity, With<ToastsContainer>>,
    font: Res<UiFont>,
) {
    let Ok(container) = container.get_single() else {
        return;
    };
    let font = font.into_inner();
    for ev in events.read() {
        let text = match ev.kind {
            ItemKind::AirJumpCrystal => "💎 + Cristal cyan",
            ItemKind::AmberPetal => "🟠 Petale d'ambre",
            ItemKind::WhiteFeather => "🪶 Plume",
            ItemKind::Hourglass => "⏳ Sablier",
            ItemKind::Heart => "❤ + 1 PV",
            ItemKind::MemoryPetal => "🌸 Petale memoire",
        };
        spawn_toast(&mut commands, container, font, text, ToastStyle::Pickup);
    }
}

fn spawn_checkpoint_toast(
    mut commands: Commands,
    mut events: EventReader<CheckpointReached>,
    container: Query<Entity, With<ToastsContainer>>,
    font: Res<UiFont>,
) {
    let Ok(container) = container.get_single() else {
        return;
    };
    let font = font.into_inner();
    for _ in events.read() {
        spawn_toast(&mut commands, container, font, "Checkpoint atteint", ToastStyle::Papyrus);
    }
}

fn tick_toasts(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Toast, &mut BackgroundColor, &mut BorderColor)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut toast, mut bg, mut border) in &mut q {
        toast.remaining -= dt;
        if toast.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        // Fade out sur les derniers 0.5 s
        let alpha = (toast.remaining / 0.5).min(1.0);
        let c = bg.0.to_srgba();
        bg.0 = Color::srgba(c.red, c.green, c.blue, alpha);
        let c = border.0.to_srgba();
        border.0 = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ===================================================== Inventaire ===

fn update_inventory_hud(
    inventory: Res<Inventory>,
    mut slots: Query<(&InventorySlot, &Children, &mut BackgroundColor, &mut BorderColor)>,
    mut texts: Query<&mut Text>,
) {
    for (slot, children, mut bg, mut border) in &mut slots {
        let kind = inventory.slots[slot.0];
        let is_selected = inventory.selected == slot.0;

        if is_selected {
            *bg = PAPYRUS.into();
            *border = GOLD.into();
        } else if kind.is_some() {
            *bg = WOOD_LIGHT.into();
            *border = WOOD_DARK.into();
        } else {
            *bg = Color::srgba(0.42, 0.23, 0.09, 0.5).into();
            *border = WOOD_DARKEST.into();
        }

        for child in children {
            if let Ok(mut text) = texts.get_mut(*child) {
                text.sections[0].value = match kind {
                    Some(crate::throwables::ThrowableKind::Bomb) => "B".into(),
                    Some(crate::throwables::ThrowableKind::IceBlock) => "G".into(),
                    Some(crate::throwables::ThrowableKind::MagicPlatform) => "P".into(),
                    Some(crate::throwables::ThrowableKind::Rock) => "C".into(),
                    Some(crate::throwables::ThrowableKind::Torch) => "T".into(),
                    None => "".into(),
                };
            }
        }
    }
}

// ====================================================== Effets actifs ===

fn update_effects_hud(
    mut commands: Commands,
    effects: Res<ActiveEffects>,
    container: Query<Entity, With<EffectsContainer>>,
    children: Query<&Children, With<EffectsContainer>>,
    font: Res<UiFont>,
) {
    let Ok(container) = container.get_single() else {
        return;
    };
    // Despawn tous les anciens enfants
    if let Ok(children) = children.get_single() {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }
    let font = font.into_inner();

    let mut effects_list: Vec<(&str, f32, Color)> = vec![];
    if effects.invincible > 0.0 {
        effects_list.push(("Invincible", effects.invincible / 3.0, GOLD));
    }
    if effects.jump_boost > 0.0 {
        effects_list.push(("Saut+", effects.jump_boost / 5.0, CREAM));
    }
    if effects.time_slow > 0.0 {
        effects_list.push(("Lent", effects.time_slow / 4.0, PAPYRUS));
    }

    commands.entity(container).with_children(|p| {
        for (label, t, color) in effects_list {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    width: Val::Px(120.0),
                    padding: UiRect::all(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: Color::srgba(0.29, 0.18, 0.10, 0.7).into(),
                border_color: WOOD_DARK.into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    label,
                    TextStyle {
                        font: font.display.clone(),
                        font_size: 14.0,
                        color,
                    },
                ));
                p.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0 * t.clamp(0.0, 1.0)),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    background_color: color.into(),
                    ..default()
                });
            });
        }
    });
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
