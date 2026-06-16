//! Éléments interactifs du niveau : pics, checkpoints, drapeau de fin.
//! Tout est testé par AABB contre la hitbox du joueur dans son propre
//! système, séparé de la physique de collision solide.

use crate::physics::Collider;
use crate::player::Player;
use crate::states::{GameState, PlayerDied, PlayerWon};
use bevy::prelude::*;

const SPIKE_COLOR: Color = Color::srgb(0.85, 0.85, 0.95);
const CHECKPOINT_INACTIVE: Color = Color::srgb(0.85, 0.75, 0.25);
const CHECKPOINT_ACTIVE: Color = Color::srgb(0.20, 0.85, 0.35);
const GOAL_COLOR: Color = Color::srgb(0.95, 0.25, 0.70);
/// Altitude sous laquelle on considère que le joueur est tombé hors du monde.
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

/// Position de respawn courante. Mise à jour au passage d'un checkpoint
/// et au démarrage de la partie.
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
    mut death: EventWriter<PlayerDied>,
) {
    let Ok((p_t, p_c)) = player.get_single() else {
        return;
    };
    for (s_t, s_c) in &spikes {
        if aabb_overlap(p_t.translation, p_c.size, s_t.translation, s_c.size) {
            death.send(PlayerDied);
            return;
        }
    }
}

fn check_kill_floor(
    player: Query<&Transform, With<Player>>,
    mut death: EventWriter<PlayerDied>,
) {
    let Ok(p_t) = player.get_single() else {
        return;
    };
    if p_t.translation.y < KILL_FLOOR_Y {
        death.send(PlayerDied);
    }
}

fn check_checkpoints(
    player: Query<(&Transform, &Collider), With<Player>>,
    mut checkpoints: Query<(&Transform, &Collider, &mut Checkpoint, &mut Sprite)>,
    mut respawn: ResMut<RespawnPoint>,
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

/// Helpers de spawn appelés depuis le module world.
pub fn spawn_spike(commands: &mut Commands, pos: Vec2, size: Vec2) {
    commands.spawn((
        Spike,
        Collider::new(size),
        SpriteBundle {
            sprite: Sprite {
                color: SPIKE_COLOR,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}

pub fn spawn_checkpoint(commands: &mut Commands, pos: Vec2, spawn: Vec2) {
    let size = Vec2::new(28.0, 60.0);
    commands.spawn((
        Checkpoint {
            spawn_point: spawn,
            triggered: false,
        },
        Collider::new(size),
        SpriteBundle {
            sprite: Sprite {
                color: CHECKPOINT_INACTIVE,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}

pub fn spawn_goal(commands: &mut Commands, pos: Vec2) {
    let size = Vec2::new(40.0, 80.0);
    commands.spawn((
        Goal,
        Collider::new(size),
        SpriteBundle {
            sprite: Sprite {
                color: GOAL_COLOR,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}

