//! Lecture des SFX. Les sons sont générés par `examples/gen_audio.rs`
//! et stockés dans `assets/sfx/`. Le volume est multiplié par
//! `settings.master * settings.sfx`.

use crate::save::Settings;
use crate::states::{GameState, PlayerDied, PlayerWon};
use crate::world::{CurrentLevel, LevelId};
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

#[derive(Resource)]
struct Sfx {
    jump: Handle<AudioSource>,
    land: Handle<AudioSource>,
    death: Handle<AudioSource>,
    checkpoint: Handle<AudioSource>,
    win: Handle<AudioSource>,
}

#[derive(Resource)]
struct MusicTracks {
    level_1: Handle<AudioSource>,
    level_2: Handle<AudioSource>,
    level_3: Handle<AudioSource>,
    level_4: Handle<AudioSource>,
    level_5: Handle<AudioSource>,
}

#[derive(Component)]
struct MusicEntity {
    /// 1.0 = entrant (fade in), 0.0 = nominal, -1.0 = sortant (fade out).
    state: FadeState,
    fade_t: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum FadeState {
    FadingIn,
    Nominal,
    FadingOut,
}

/// Évènement émis par le contrôleur du joueur quand il décolle (sol ou
/// double saut). Le module effects écoute aussi `PlayerAirJumped` pour
/// distinguer les deux pour les particules.
#[derive(Event, Debug)]
pub struct PlayerJumped;

/// Émis uniquement quand le joueur déclenche son saut en l'air (double
/// saut). Sert au feedback visuel (burst circulaire).
#[derive(Event, Debug)]
pub struct PlayerAirJumped;

/// Évènement émis quand le joueur atterrit, avec le facteur d'impact
/// (0..1) pour moduler le volume.
#[derive(Event, Debug)]
pub struct PlayerLanded(pub f32);

/// Évènement émis quand un checkpoint est activé pour la première fois.
#[derive(Event, Debug)]
pub struct CheckpointReached;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // Chargement immédiat pour éviter le retard sur les premiers SFX.
        let asset_server = app.world().resource::<AssetServer>().clone();
        let sfx = Sfx {
            jump: asset_server.load("sfx/jump.wav"),
            land: asset_server.load("sfx/land.wav"),
            death: asset_server.load("sfx/death.wav"),
            checkpoint: asset_server.load("sfx/checkpoint.wav"),
            win: asset_server.load("sfx/win.wav"),
        };
        let music = MusicTracks {
            level_1: asset_server.load("music/level_1.wav"),
            level_2: asset_server.load("music/level_2.wav"),
            level_3: asset_server.load("music/level_3.wav"),
            level_4: asset_server.load("music/level_4.wav"),
            level_5: asset_server.load("music/level_5.wav"),
        };
        app.insert_resource(sfx)
            .insert_resource(music)
            .add_event::<PlayerJumped>()
            .add_event::<PlayerAirJumped>()
            .add_event::<PlayerLanded>()
            .add_event::<CheckpointReached>()
            .add_systems(OnEnter(GameState::Playing), start_music_for_level)
            .add_systems(OnEnter(GameState::MainMenu), stop_music)
            .add_systems(OnEnter(GameState::Win), stop_music)
            .add_systems(
                Update,
                (
                    play_jump,
                    play_land,
                    play_death,
                    play_checkpoint,
                    play_win,
                    update_music_on_level_change,
                    tick_music_fades,
                    update_music_volume,
                ),
            );
    }
}

/// Volume effectif pour les SFX, dérivé des deux curseurs.
fn sfx_volume(settings: &Settings) -> f32 {
    settings.master_volume * settings.sfx_volume
}

fn play_sound(commands: &mut Commands, handle: Handle<AudioSource>, volume: f32) {
    if volume <= 0.001 {
        return;
    }
    commands.spawn(AudioBundle {
        source: handle,
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(volume.clamp(0.0, 1.5)),
            ..default()
        },
    });
}

fn play_jump(
    mut events: EventReader<PlayerJumped>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    settings: Res<Settings>,
) {
    let volume = sfx_volume(&settings);
    for _ in events.read() {
        play_sound(&mut commands, sfx.jump.clone(), volume * 0.8);
    }
}

fn play_land(
    mut events: EventReader<PlayerLanded>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    settings: Res<Settings>,
) {
    let volume = sfx_volume(&settings);
    for ev in events.read() {
        // Petit impact n'a pas besoin de son (moins de 25% d'intensité).
        let intensity = ev.0.clamp(0.0, 1.0);
        if intensity < 0.25 {
            continue;
        }
        play_sound(&mut commands, sfx.land.clone(), volume * intensity);
    }
}

fn play_death(
    mut events: EventReader<PlayerDied>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    settings: Res<Settings>,
) {
    let volume = sfx_volume(&settings);
    for _ in events.read() {
        play_sound(&mut commands, sfx.death.clone(), volume);
    }
}

fn play_checkpoint(
    mut events: EventReader<CheckpointReached>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    settings: Res<Settings>,
) {
    let volume = sfx_volume(&settings);
    for _ in events.read() {
        play_sound(&mut commands, sfx.checkpoint.clone(), volume * 0.7);
    }
}

fn play_win(
    mut events: EventReader<PlayerWon>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    settings: Res<Settings>,
) {
    let volume = sfx_volume(&settings);
    for _ in events.read() {
        play_sound(&mut commands, sfx.win.clone(), volume);
    }
}

// ============================================================ Music ===

fn music_handle(tracks: &MusicTracks, level: LevelId) -> Handle<AudioSource> {
    match level {
        LevelId::PinkSunset => tracks.level_1.clone(),
        LevelId::NightForest => tracks.level_2.clone(),
        LevelId::AmberRuins => tracks.level_3.clone(),
        LevelId::Sanctuary => tracks.level_4.clone(),
        LevelId::Dawn => tracks.level_5.clone(),
    }
}

fn music_volume(settings: &Settings) -> f32 {
    (settings.master_volume * settings.music_volume).clamp(0.0, 1.0)
}

fn start_music_for_level(
    mut commands: Commands,
    tracks: Res<MusicTracks>,
    current_level: Res<CurrentLevel>,
    settings: Res<Settings>,
    existing: Query<Entity, With<MusicEntity>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
    commands.spawn((
        MusicEntity {
            state: FadeState::Nominal,
            fade_t: 0.0,
        },
        AudioBundle {
            source: music_handle(&tracks, current_level.0),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(music_volume(&settings)),
                ..default()
            },
        },
    ));
}

fn stop_music(mut commands: Commands, existing: Query<Entity, With<MusicEntity>>) {
    for e in &existing {
        commands.entity(e).despawn();
    }
}

const FADE_DURATION: f32 = 1.5;

/// Quand le niveau change en cours de partie, fade out l'ancienne piste
/// et fade in la nouvelle (crossfade 1.5 s).
fn update_music_on_level_change(
    mut commands: Commands,
    tracks: Res<MusicTracks>,
    current_level: Res<CurrentLevel>,
    mut existing: Query<&mut MusicEntity>,
) {
    if !current_level.is_changed() {
        return;
    }
    // Marque les pistes en cours comme sortantes
    for mut music in &mut existing {
        if music.state != FadeState::FadingOut {
            music.state = FadeState::FadingOut;
            music.fade_t = 0.0;
        }
    }
    // Spawn la nouvelle en fade in
    commands.spawn((
        MusicEntity {
            state: FadeState::FadingIn,
            fade_t: 0.0,
        },
        AudioBundle {
            source: music_handle(&tracks, current_level.0),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.0),
                ..default()
            },
        },
    ));
}

fn tick_music_fades(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<Settings>,
    mut q: Query<(Entity, &mut MusicEntity, &AudioSink)>,
) {
    let dt = time.delta_seconds();
    let target_volume = music_volume(&settings);
    for (entity, mut music, sink) in &mut q {
        match music.state {
            FadeState::FadingIn => {
                music.fade_t += dt;
                let t = (music.fade_t / FADE_DURATION).clamp(0.0, 1.0);
                sink.set_volume(target_volume * t);
                if t >= 1.0 {
                    music.state = FadeState::Nominal;
                }
            }
            FadeState::FadingOut => {
                music.fade_t += dt;
                let t = (1.0 - music.fade_t / FADE_DURATION).clamp(0.0, 1.0);
                sink.set_volume(target_volume * t);
                if music.fade_t >= FADE_DURATION {
                    commands.entity(entity).despawn();
                }
            }
            FadeState::Nominal => {}
        }
    }
}

/// Suit les changements de volume dans Settings (menu paramètres) et
/// les applique en live sur la piste qui joue (sauf pendant fade).
fn update_music_volume(
    settings: Res<Settings>,
    music_q: Query<(&AudioSink, &MusicEntity)>,
) {
    if !settings.is_changed() {
        return;
    }
    let v = music_volume(&settings);
    for (sink, music) in &music_q {
        if music.state == FadeState::Nominal {
            sink.set_volume(v);
        }
    }
}
