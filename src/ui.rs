//! Menus et HUD. Tout en Bevy UI natif, sans assets externes (font par
//! défaut). Chaque écran est spawné dans son OnEnter et despawné dans
//! son OnExit pour éviter d'accumuler des entités fantômes.

use crate::level::{Checkpoint, RespawnPoint};
use crate::physics::{Grounded, Velocity};
use crate::player::{Player, PlayerController};
use crate::states::{GameState, PlayerWon, RunStats};
use crate::world::PLAYER_SPAWN;
use bevy::prelude::*;

const OVERLAY_BG: Color = Color::srgba(0.05, 0.08, 0.15, 0.85);
const TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
const SUBTITLE_COLOR: Color = Color::srgb(0.80, 0.85, 0.90);
const HINT_COLOR: Color = Color::srgb(0.70, 0.75, 0.80);

#[derive(Component)]
struct ScreenTag;

#[derive(Component)]
struct HudTag;

#[derive(Component)]
struct HudDeaths;

#[derive(Component)]
struct HudTime;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            // Écrans plein écran montés selon l'état.
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_screen)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_screen)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_screen)
            .add_systems(OnEnter(GameState::Win), spawn_win_screen)
            .add_systems(OnExit(GameState::Win), despawn_screen)
            // Transitions globales (input clavier).
            .add_systems(
                Update,
                (
                    start_from_menu.run_if(in_state(GameState::MainMenu)),
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    resume_or_quit.run_if(in_state(GameState::Paused)),
                    handle_win_event.run_if(in_state(GameState::Playing)),
                    restart_from_endgame
                        .run_if(in_state(GameState::GameOver).or_else(in_state(GameState::Win))),
                    update_hud.run_if(in_state(GameState::Playing)),
                ),
            )
            // Reset de la run sur (re)démarrage.
            .add_systems(OnEnter(GameState::Playing), reset_player_for_run);
    }
}

// ============================================================ HUD ===

fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            HudTag,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                HudDeaths,
                TextBundle::from_section(
                    "Morts : 0",
                    TextStyle {
                        font_size: 22.0,
                        color: HINT_COLOR,
                        ..default()
                    },
                ),
            ));
            p.spawn((
                HudTime,
                TextBundle::from_section(
                    "Temps : 0.0 s",
                    TextStyle {
                        font_size: 22.0,
                        color: HINT_COLOR,
                        ..default()
                    },
                ),
            ));
        });
}

fn update_hud(
    stats: Res<RunStats>,
    mut deaths_q: Query<&mut Text, (With<HudDeaths>, Without<HudTime>)>,
    mut time_q: Query<&mut Text, (With<HudTime>, Without<HudDeaths>)>,
) {
    if let Ok(mut t) = deaths_q.get_single_mut() {
        t.sections[0].value = format!("Morts : {}", stats.deaths);
    }
    if let Ok(mut t) = time_q.get_single_mut() {
        t.sections[0].value = format!("Temps : {:.1} s", stats.time_seconds);
    }
}

// ========================================================== Menus ===

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
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                background_color: OVERLAY_BG.into(),
                ..default()
            },
        ))
        .id()
}

fn label(text: &str, size: f32, color: Color) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: size,
            color,
            ..default()
        },
    )
}

fn spawn_main_menu(mut commands: Commands) {
    let root = spawn_overlay(&mut commands);
    commands.entity(root).with_children(|p| {
        p.spawn(label("Adventure Timing", 64.0, TITLE_COLOR));
        p.spawn(label(
            "Side-scroller platformer",
            22.0,
            SUBTITLE_COLOR,
        ));
        p.spawn(label("[ Espace ] Commencer", 28.0, HINT_COLOR));
    });
}

fn spawn_pause_menu(mut commands: Commands) {
    let root = spawn_overlay(&mut commands);
    commands.entity(root).with_children(|p| {
        p.spawn(label("Pause", 56.0, TITLE_COLOR));
        p.spawn(label("[ Échap ] Reprendre", 24.0, HINT_COLOR));
        p.spawn(label("[ Q ] Menu principal", 24.0, HINT_COLOR));
    });
}

fn spawn_game_over(mut commands: Commands, stats: Res<RunStats>) {
    let root = spawn_overlay(&mut commands);
    let deaths = stats.deaths;
    commands.entity(root).with_children(|p| {
        p.spawn(label("Game Over", 64.0, Color::srgb(0.95, 0.45, 0.45)));
        p.spawn(label(&format!("Morts cumulées : {deaths}"), 24.0, SUBTITLE_COLOR));
        p.spawn(label("[ Espace ] Recommencer", 26.0, HINT_COLOR));
        p.spawn(label("[ Q ] Menu principal", 22.0, HINT_COLOR));
    });
}

fn spawn_win_screen(mut commands: Commands, stats: Res<RunStats>) {
    let root = spawn_overlay(&mut commands);
    let deaths = stats.deaths;
    let secs = stats.time_seconds;
    commands.entity(root).with_children(|p| {
        p.spawn(label("Bravo !", 72.0, Color::srgb(0.40, 0.95, 0.55)));
        p.spawn(label(
            &format!("Temps : {secs:.2} s — Morts : {deaths}"),
            26.0,
            SUBTITLE_COLOR,
        ));
        p.spawn(label("[ Espace ] Rejouer", 26.0, HINT_COLOR));
        p.spawn(label("[ Q ] Menu principal", 22.0, HINT_COLOR));
    });
}

fn despawn_screen(mut commands: Commands, q: Query<Entity, With<ScreenTag>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

// =================================================== Transitions ===

fn start_from_menu(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next.set(GameState::Playing);
    }
}

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::Paused);
    }
}

fn resume_or_quit(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::KeyQ) {
        next.set(GameState::MainMenu);
    }
}

fn handle_win_event(mut events: EventReader<PlayerWon>, mut next: ResMut<NextState<GameState>>) {
    if events.read().next().is_some() {
        next.set(GameState::Win);
    }
}

fn restart_from_endgame(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::KeyQ) {
        next.set(GameState::MainMenu);
    }
}

/// Réinitialise le joueur, les checkpoints, le respawn et les stats
/// au début de chaque nouvelle partie.
fn reset_player_for_run(
    mut player: Query<
        (&mut Transform, &mut Velocity, &mut PlayerController, &mut Grounded),
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
        sprite.color = Color::srgb(0.85, 0.75, 0.25);
    }
}
