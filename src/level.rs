//! Éléments interactifs du niveau : pics, checkpoints, drapeau de fin.
//! Tout est testé par AABB contre la hitbox du joueur dans son propre
//! système, séparé de la physique de collision solide.

use crate::audio::CheckpointReached;
use crate::items::ActiveEffects;
use crate::physics::Collider;
use crate::player::{Player, PlayerHit};
use crate::states::{GameState, PlayerDied, PlayerWon};
use crate::world::LevelEntity;
use bevy::prelude::*;

const CHECKPOINT_ACTIVE: Color = Color::srgb(0.45, 1.0, 0.55);
pub const KILL_FLOOR_Y: f32 = -800.0;

#[derive(Component)]
pub struct Spike;

#[derive(Component)]
pub struct Goal;

#[derive(Component)]
pub struct Checkpoint {
    pub spawn_point: Vec2,
    pub triggered: bool,
}

#[derive(Resource, Debug)]
pub struct RespawnPoint(pub Vec2);

impl Default for RespawnPoint {
    fn default() -> Self {
        Self(Vec2::new(-600.0, -100.0))
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RespawnPoint>().add_systems(
            Update,
            (
                check_spike_collision,
                check_kill_floor,
                check_checkpoints,
                check_goal,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn aabb_overlap(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let ah = a_size * 0.5;
    let bh = b_size * 0.5;
    (a_pos.x - b_pos.x).abs() < ah.x + bh.x && (a_pos.y - b_pos.y).abs() < ah.y + bh.y
}

fn check_spike_collision(
    player: Query<(&Transform, &Collider), With<Player>>,
    spikes: Query<(&Transform, &Collider), With<Spike>>,
    effects: Res<ActiveEffects>,
    mut hit: EventWriter<PlayerHit>,
) {
    if effects.invincible > 0.0 {
        return;
    }
    let Ok((p_t, p_c)) = player.get_single() else {
        return;
    };
    for (s_t, s_c) in &spikes {
        if aabb_overlap(p_t.translation, p_c.size, s_t.translation, s_c.size) {
            hit.send(PlayerHit { damage: 1 });
            return;
        }
    }
}

fn check_kill_floor(player: Query<&Transform, With<Player>>, mut death: EventWriter<PlayerDied>) {
    let Ok(p_t) = player.get_single() else {
        return;
    };
    // Chute dans le vide : pas d'invincibilité ici, c'est définitif.
    if p_t.translation.y < KILL_FLOOR_Y {
        death.send(PlayerDied);
    }
}

fn check_checkpoints(
    player: Query<(&Transform, &Collider), With<Player>>,
    mut checkpoints: Query<(&Transform, &Collider, &mut Checkpoint, &mut Sprite)>,
    mut respawn: ResMut<RespawnPoint>,
    mut reached: EventWriter<CheckpointReached>,
) {
    let Ok((p_t, p_c)) = player.get_single() else {
        return;
    };
    for (c_t, c_c, mut chk, mut sprite) in &mut checkpoints {
        if !chk.triggered
            && aabb_overlap(p_t.translation, p_c.size, c_t.translation, c_c.size)
        {
            chk.triggered = true;
            sprite.color = CHECKPOINT_ACTIVE;
            respawn.0 = chk.spawn_point;
            reached.send(CheckpointReached);
        }
    }
}

fn check_goal(
    player: Query<(&Transform, &Collider), With<Player>>,
    goals: Query<(&Transform, &Collider), With<Goal>>,
    mut won: EventWriter<PlayerWon>,
) {
    let Ok((p_t, p_c)) = player.get_single() else {
        return;
    };
    for (g_t, g_c) in &goals {
        if aabb_overlap(p_t.translation, p_c.size, g_t.translation, g_c.size) {
            won.send(PlayerWon);
            return;
        }
    }
}

// =========================================================== Spawners ===

/// Spawne plusieurs pics côte à côte pour couvrir la zone `(pos, size)`.
/// Chaque pic utilise sa taille de texture native (32x24) pour préserver
/// la lecture pixel-art.
pub fn spawn_spike_field(
    commands: &mut Commands,
    asset_server: &AssetServer,
    center: Vec2,
    width: f32,
) {
    let tex = asset_server.load("sprites/spike.png");
    let tile = 32.0;
    let count = ((width / tile).round() as i32).max(1);
    let total = count as f32 * tile;
    let start_x = center.x - total * 0.5 + tile * 0.5;
    for i in 0..count {
        let x = start_x + i as f32 * tile;
        commands.spawn((
            LevelEntity,
            Spike,
            Collider::new(Vec2::new(tile, 24.0)),
            SpriteBundle {
                texture: tex.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(tile, 24.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, center.y, 0.5)),
                ..default()
            },
        ));
    }
}

pub fn spawn_checkpoint(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    spawn: Vec2,
) {
    let size = Vec2::new(32.0, 64.0);
    commands.spawn((
        LevelEntity,
        Checkpoint {
            spawn_point: spawn,
            triggered: false,
        },
        Collider::new(Vec2::new(20.0, 56.0)),
        SpriteBundle {
            texture: asset_server.load("sprites/checkpoint.png"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}

pub fn spawn_goal(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(48.0, 80.0);
    commands.spawn((
        LevelEntity,
        Goal,
        Collider::new(Vec2::new(28.0, 70.0)),
        SpriteBundle {
            texture: asset_server.load("sprites/goal.png"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}
