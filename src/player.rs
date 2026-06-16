//! Contrôleur du joueur : input, accélération/freinage horizontal,
//! saut avec coyote time, jump buffer, saut variable, plus le rig
//! d'animation par texture atlas (7 frames, voir examples/gen_assets.rs).

use crate::audio::{PlayerAirJumped, PlayerJumped};
use crate::heroes::SelectedHero;
use crate::items::ActiveEffects;
use crate::effects::{ScreenShake, SquashStretch};
use crate::level::RespawnPoint;
use crate::physics::{Collider, Grounded, PhysicsSet, Velocity};
use crate::states::{GameState, PlayerDied, RunStats};
use crate::world::PLAYER_SPAWN;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Hitbox de gameplay. Plus petite que le sprite (24x36) car la cape
/// déborde sur les côtés sans participer aux collisions.
const PLAYER_SIZE: Vec2 = Vec2::new(14.0, 30.0);

/// Ratio pour aligner le bas du sprite avec le bas du collider.
/// (sprite 36 px, collider 30 px → anchor 3 px sous le centre)
const SPRITE_ANCHOR_Y: f32 = -3.0 / 36.0;

const SPRITE_FRAME_SIZE: UVec2 = UVec2::new(24, 36);
const SPRITE_FRAME_COUNT: u32 = 7;

/// Accélération au sol. Volontairement plus basse pour donner du poids.
const ACCEL: f32 = 1900.0;
/// Décélération naturelle. Plus basse → on garde plus d'inertie en
/// relâchant la touche.
const GROUND_FRICTION: f32 = 1400.0;
/// Contrôle aérien renforcé pour matcher le double saut et les
/// distances allongées.
const AIR_ACCEL: f32 = 1700.0;
/// Multiplicateur d'accélération quand on pousse dans le sens opposé
/// au mouvement courant — pour répondre vite aux demi-tours.
const TURNAROUND_BOOST: f32 = 1.7;

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

/// PV du joueur + cooldown d'invincibilité post-coup (i-frames).
#[derive(Component, Debug)]
pub struct PlayerHp {
    pub current: u32,
    pub max: u32,
    pub hit_invuln: f32,
}

impl PlayerHp {
    pub fn full(max: u32) -> Self {
        Self {
            current: max,
            max,
            hit_invuln: 0.0,
        }
    }
}

/// Évènement émis quand le joueur prend un coup (mais ne meurt pas
/// forcément). Si HP tombe à 0, on émet aussi PlayerDied.
#[derive(Event, Debug)]
pub struct PlayerHit {
    pub damage: u32,
}

/// Durée d'invincibilité post-coup pour éviter de chain-tap les dégâts.
pub const HIT_INVULN: f32 = 0.7;

#[derive(Component)]
pub struct PlayerController {
    pub coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub is_jumping: bool,
    /// Sauts encore disponibles en l'air. Rechargé au contact du sol.
    pub air_jumps_remaining: u8,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
            is_jumping: false,
            // air_jumps_remaining sera reset par tick_player_timers selon
            // le héros sélectionné dès la prochaine frame.
            air_jumps_remaining: 1,
        }
    }
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
        app.add_event::<PlayerHit>()
            .add_systems(Startup, spawn_player)
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
            .add_systems(
                Update,
                (
                    handle_player_hit,
                    handle_death,
                    animate_player,
                    update_hero_sprite,
                    tick_hp_invuln,
                ),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_hero: Res<SelectedHero>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load(selected_hero.0.sprite_path());
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
        PlayerHp::full(selected_hero.0.max_hp()),
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

#[allow(clippy::too_many_arguments)]
fn handle_death(
    mut deaths: EventReader<PlayerDied>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut PlayerController, &mut PlayerHp), With<Player>>,
    respawn: Res<RespawnPoint>,
    selected_hero: Res<SelectedHero>,
    mut effects: ResMut<ActiveEffects>,
    mut stats: ResMut<RunStats>,
    mut shake: ResMut<ScreenShake>,
) {
    if deaths.is_empty() {
        return;
    }
    deaths.clear();

    let Ok((mut transform, mut velocity, mut ctrl, mut hp)) = player.get_single_mut() else {
        return;
    };

    // Pétale mémoire : ignore cette mort
    if effects.skip_next_death {
        effects.skip_next_death = false;
        hp.current = hp.max;
        hp.hit_invuln = 1.0; // grace period
        effects.invincible = effects.invincible.max(1.0);
        return;
    }

    transform.translation = respawn.0.extend(transform.translation.z);
    velocity.0 = Vec2::ZERO;
    *ctrl = PlayerController::default();
    hp.current = hp.max;
    hp.hit_invuln = 0.0;
    stats.deaths += 1;
    shake.add(0.65);

    let respawn_invuln = selected_hero.0.respawn_invincibility();
    if respawn_invuln > 0.0 {
        effects.invincible = effects.invincible.max(respawn_invuln);
    }
}

/// Switche la texture du joueur quand `SelectedHero` change (depuis
/// l'écran de sélection). Met aussi à jour les PV max.
fn update_hero_sprite(
    selected_hero: Res<SelectedHero>,
    asset_server: Res<AssetServer>,
    mut q: Query<(&mut Handle<Image>, &mut PlayerHp), With<Player>>,
) {
    if !selected_hero.is_changed() {
        return;
    }
    let new_handle = asset_server.load(selected_hero.0.sprite_path());
    let new_max = selected_hero.0.max_hp();
    for (mut h, mut hp) in &mut q {
        *h = new_handle.clone();
        hp.max = new_max;
        hp.current = new_max;
    }
}

/// Consomme PlayerHit : décrémente les PV, applique invul, si HP=0 émet
/// PlayerDied.
fn handle_player_hit(
    mut events: EventReader<PlayerHit>,
    mut q: Query<&mut PlayerHp, With<Player>>,
    mut died: EventWriter<PlayerDied>,
    mut shake: ResMut<ScreenShake>,
) {
    let Ok(mut hp) = q.get_single_mut() else {
        return;
    };
    for ev in events.read() {
        if hp.hit_invuln > 0.0 {
            continue;
        }
        hp.current = hp.current.saturating_sub(ev.damage);
        hp.hit_invuln = HIT_INVULN;
        shake.add(0.35);
        if hp.current == 0 {
            died.send(PlayerDied);
        }
    }
}

fn tick_hp_invuln(time: Res<Time>, mut q: Query<&mut PlayerHp>) {
    let dt = time.delta_seconds();
    for mut hp in &mut q {
        hp.hit_invuln = (hp.hit_invuln - dt).max(0.0);
    }
}

fn tick_player_timers(
    time: Res<Time>,
    selected_hero: Res<SelectedHero>,
    mut q: Query<(&mut PlayerController, &Grounded)>,
) {
    let dt = time.delta_seconds();
    let max_air_jumps = selected_hero.0.max_air_jumps();
    for (mut ctrl, grounded) in &mut q {
        if grounded.0 {
            ctrl.coyote_timer = 0.0;
            ctrl.air_jumps_remaining = max_air_jumps;
        } else {
            ctrl.coyote_timer += dt;
        }
        ctrl.jump_buffer_timer += dt;
    }
}

fn handle_horizontal_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    selected_hero: Res<SelectedHero>,
    mut q: Query<(&mut Velocity, &Grounded, &mut Player, &mut Sprite)>,
) {
    let dt = time.delta_seconds();
    let move_speed = selected_hero.0.move_speed();
    let mut dir = 0.0;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir += 1.0;
    }

    for (mut velocity, grounded, mut player, mut sprite) in &mut q {
        let target = dir * move_speed;
        let base_accel = if grounded.0 { ACCEL } else { AIR_ACCEL };

        if dir != 0.0 {
            // Boost en demi-tour pour répondre vite quand on change de
            // sens, sans pour autant casser le poids du mouvement normal.
            let opposite = velocity.0.x * dir < 0.0;
            let accel = if opposite { base_accel * TURNAROUND_BOOST } else { base_accel };
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
    selected_hero: Res<SelectedHero>,
    mut q: Query<(&mut Velocity, &mut PlayerController, &Grounded), With<Player>>,
    effects: Res<ActiveEffects>,
    mut jumped: EventWriter<PlayerJumped>,
    mut air_jumped: EventWriter<PlayerAirJumped>,
) {
    let hero = selected_hero.0;
    let jump_velocity = hero.jump_velocity();
    let air_jump_velocity = hero.air_jump_velocity();
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

        let can_jump_ground = ctrl.coyote_timer < COYOTE_TIME || grounded.0;
        let can_jump_air = !grounded.0
            && ctrl.coyote_timer >= COYOTE_TIME
            && ctrl.air_jumps_remaining > 0;
        let buffered = ctrl.jump_buffer_timer < JUMP_BUFFER;

        // Plume blanche : saut +30 %
        let boost = if effects.jump_boost > 0.0 { 1.3 } else { 1.0 };

        if buffered && can_jump_ground {
            velocity.0.y = jump_velocity * boost;
            ctrl.is_jumping = true;
            ctrl.jump_buffer_timer = JUMP_BUFFER;
            ctrl.coyote_timer = COYOTE_TIME;
            jumped.send(PlayerJumped);
        } else if buffered && can_jump_air {
            velocity.0.y = air_jump_velocity * boost;
            ctrl.is_jumping = true;
            ctrl.air_jumps_remaining -= 1;
            ctrl.jump_buffer_timer = JUMP_BUFFER;
            jumped.send(PlayerJumped);
            air_jumped.send(PlayerAirJumped);
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
