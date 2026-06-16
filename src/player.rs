//! Contrôleur du joueur : input, accélération/freinage horizontal,
//! saut avec coyote time, jump buffer, saut variable, plus le rig
//! d'animation par texture atlas (7 frames, voir examples/gen_assets.rs).

use crate::effects::{ScreenShake, SquashStretch};
use crate::level::RespawnPoint;
use crate::physics::{Collider, Grounded, PhysicsSet, Velocity};
use crate::states::{GameState, PlayerDied, RunStats};
use crate::world::PLAYER_SPAWN;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Hitbox de gameplay. Plus petite que le sprite (32x48) pour
/// permettre des passages serrés et une lecture précise des collisions.
const PLAYER_SIZE: Vec2 = Vec2::new(28.0, 44.0);

/// Ratio pour aligner le bas du sprite avec le bas du collider.
/// (sprite 48 px, collider 44 px → anchor 2 px sous le centre)
const SPRITE_ANCHOR_Y: f32 = -2.0 / 48.0;

const SPRITE_FRAME_SIZE: UVec2 = UVec2::new(32, 48);
const SPRITE_FRAME_COUNT: u32 = 7;

const MOVE_SPEED: f32 = 280.0;
const ACCEL: f32 = 2400.0;
const GROUND_FRICTION: f32 = 2200.0;
const AIR_ACCEL: f32 = 1400.0;

const JUMP_VELOCITY: f32 = 760.0;
const JUMP_CUT_FACTOR: f32 = 0.45;

const COYOTE_TIME: f32 = 0.10;
const JUMP_BUFFER: f32 = 0.12;

const RUN_FRAME_TIME: f32 = 0.09;

#[derive(Component)]
pub struct Player {
    /// Dernière direction non nulle (1.0 droite, -1.0 gauche).
    pub facing: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { facing: 1.0 }
    }
}

#[derive(Component, Default)]
pub struct PlayerController {
    pub coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub is_jumping: bool,
}

/// État d'animation. Sépare la logique "quelle pose afficher" de l'index
/// concret dans l'atlas, ce qui permet de réorganiser le sprite sheet
/// sans toucher au reste.
#[derive(Component)]
pub struct PlayerAnimation {
    pub state: AnimState,
    pub timer: Timer,
    pub run_step: usize,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            state: AnimState::Idle,
            timer: Timer::from_seconds(RUN_FRAME_TIME, TimerMode::Repeating),
            run_step: 0,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AnimState {
    Idle,
    Run,
    Jump,
    Fall,
}

impl AnimState {
    /// Index dans le sprite sheet. Pour Run on retourne le premier frame
    /// du cycle ; le système d'animation gère ensuite le step.
    fn base_index(self) -> usize {
        match self {
            AnimState::Idle => 0,
            AnimState::Run => 1,
            AnimState::Jump => 5,
            AnimState::Fall => 6,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    tick_player_timers,
                    handle_horizontal_input,
                    handle_jump_input,
                )
                    .chain()
                    .before(PhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            )
            // L'animation tourne aussi en pause/menu (lisibilité visuelle).
            .add_systems(Update, (handle_death, animate_player));
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sprites/player.png");
    let layout = TextureAtlasLayout::from_grid(
        SPRITE_FRAME_SIZE,
        SPRITE_FRAME_COUNT,
        1,
        None,
        None,
    );
    let layout_handle = layouts.add(layout);

    commands.spawn((
        Player::default(),
        PlayerController::default(),
        PlayerAnimation::default(),
        Velocity::default(),
        Collider::new(PLAYER_SIZE),
        Grounded::default(),
        SquashStretch::default(),
        SpriteBundle {
            texture,
            sprite: Sprite {
                anchor: Anchor::Custom(Vec2::new(0.0, SPRITE_ANCHOR_Y)),
                ..default()
            },
            transform: Transform::from_translation(PLAYER_SPAWN.extend(1.0)),
            ..default()
        },
        TextureAtlas {
            layout: layout_handle,
            index: 0,
        },
    ));
}

fn handle_death(
    mut deaths: EventReader<PlayerDied>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut PlayerController), With<Player>>,
    respawn: Res<RespawnPoint>,
    mut stats: ResMut<RunStats>,
    mut shake: ResMut<ScreenShake>,
) {
    if deaths.is_empty() {
        return;
    }
    deaths.clear();

    let Ok((mut transform, mut velocity, mut ctrl)) = player.get_single_mut() else {
        return;
    };

    transform.translation = respawn.0.extend(transform.translation.z);
    velocity.0 = Vec2::ZERO;
    *ctrl = PlayerController::default();
    stats.deaths += 1;
    shake.add(0.65);
}

fn tick_player_timers(time: Res<Time>, mut q: Query<(&mut PlayerController, &Grounded)>) {
    let dt = time.delta_seconds();
    for (mut ctrl, grounded) in &mut q {
        if grounded.0 {
            ctrl.coyote_timer = 0.0;
        } else {
            ctrl.coyote_timer += dt;
        }
        ctrl.jump_buffer_timer += dt;
    }
}

fn handle_horizontal_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Velocity, &Grounded, &mut Player, &mut Sprite)>,
) {
    let dt = time.delta_seconds();
    let mut dir = 0.0;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir += 1.0;
    }

    for (mut velocity, grounded, mut player, mut sprite) in &mut q {
        let target = dir * MOVE_SPEED;
        let accel = if grounded.0 { ACCEL } else { AIR_ACCEL };

        if dir != 0.0 {
            let delta = (target - velocity.0.x).clamp(-accel * dt, accel * dt);
            velocity.0.x += delta;
            player.facing = dir;
            sprite.flip_x = dir < 0.0;
        } else if grounded.0 {
            let friction = GROUND_FRICTION * dt;
            if velocity.0.x.abs() <= friction {
                velocity.0.x = 0.0;
            } else {
                velocity.0.x -= friction.copysign(velocity.0.x);
            }
        }
    }
}

fn handle_jump_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Velocity, &mut PlayerController, &Grounded), With<Player>>,
) {
    let jump_pressed = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::ArrowUp)
        || keys.just_pressed(KeyCode::KeyW);
    let jump_released = keys.just_released(KeyCode::Space)
        || keys.just_released(KeyCode::ArrowUp)
        || keys.just_released(KeyCode::KeyW);

    for (mut velocity, mut ctrl, grounded) in &mut q {
        if jump_pressed {
            ctrl.jump_buffer_timer = 0.0;
        }

        let can_jump = ctrl.coyote_timer < COYOTE_TIME || grounded.0;
        let buffered = ctrl.jump_buffer_timer < JUMP_BUFFER;

        if can_jump && buffered {
            velocity.0.y = JUMP_VELOCITY;
            ctrl.is_jumping = true;
            ctrl.jump_buffer_timer = JUMP_BUFFER;
            ctrl.coyote_timer = COYOTE_TIME;
        }

        if jump_released && ctrl.is_jumping && velocity.0.y > 0.0 {
            velocity.0.y *= JUMP_CUT_FACTOR;
            ctrl.is_jumping = false;
        }

        if velocity.0.y <= 0.0 {
            ctrl.is_jumping = false;
        }
    }
}

/// Détermine l'état d'animation à partir de la vélocité et du contact
/// sol, puis met à jour l'index de la TextureAtlas du joueur.
fn animate_player(
    time: Res<Time>,
    mut q: Query<(
        &Velocity,
        &Grounded,
        &mut PlayerAnimation,
        &mut TextureAtlas,
    )>,
) {
    for (velocity, grounded, mut anim, mut atlas) in &mut q {
        let new_state = if !grounded.0 {
            if velocity.0.y > 0.0 {
                AnimState::Jump
            } else {
                AnimState::Fall
            }
        } else if velocity.0.x.abs() > 30.0 {
            AnimState::Run
        } else {
            AnimState::Idle
        };

        // Changement d'état : reset du cycle.
        if new_state != anim.state {
            anim.state = new_state;
            anim.run_step = 0;
            anim.timer.reset();
        }

        match anim.state {
            AnimState::Run => {
                anim.timer.tick(time.delta());
                if anim.timer.just_finished() {
                    anim.run_step = (anim.run_step + 1) % 4;
                }
                atlas.index = AnimState::Run.base_index() + anim.run_step;
            }
            other => {
                atlas.index = other.base_index();
            }
        }
    }
}
