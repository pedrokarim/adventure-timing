//! Lecture des SFX. Les sons sont générés par `examples/gen_audio.rs`
//! et stockés dans `assets/sfx/`. Le volume est multiplié par
//! `settings.master * settings.sfx`.

use crate::save::Settings;
use crate::states::{PlayerDied, PlayerWon};
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

/// Évènement émis par le contrôleur du joueur quand il décolle du sol.
#[derive(Event, Debug)]
pub struct PlayerJumped;

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
        app.insert_resource(sfx)
            .add_event::<PlayerJumped>()
            .add_event::<PlayerLanded>()
            .add_event::<CheckpointReached>()
            .add_systems(
                Update,
                (
                    play_jump,
                    play_land,
                    play_death,
                    play_checkpoint,
                    play_win,
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
