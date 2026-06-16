//! Contrôleur du joueur : input, accélération/freinage horizontal,
//! saut avec coyote time, jump buffer, et saut variable (hauteur
//! contrôlée par la durée d'appui sur le bouton).

use crate::physics::{Collider, Grounded, PhysicsSet, Velocity};
use bevy::prelude::*;

const PLAYER_COLOR: Color = Color::srgb(0.85, 0.30, 0.30);
const PLAYER_SIZE: Vec2 = Vec2::new(28.0, 44.0);
const SPAWN_POS: Vec2 = Vec2::new(-600.0, -100.0);

/// Vitesse horizontale max (px/s).
const MOVE_SPEED: f32 = 280.0;
/// Combien on accélère par seconde quand on pousse une direction.
const ACCEL: f32 = 2400.0;
/// Freinage actif quand aucune direction n'est pressée et qu'on est au sol.
const GROUND_FRICTION: f32 = 2200.0;
/// Contrôle aérien réduit pour éviter le glissement infini.
const AIR_ACCEL: f32 = 1400.0;

/// Vitesse verticale appliquée au moment du saut (px/s).
const JUMP_VELOCITY: f32 = 760.0;
/// Si le joueur relâche le bouton tôt, on coupe la vélocité Y à ce
/// facteur pour créer un saut court — cœur du saut variable.
const JUMP_CUT_FACTOR: f32 = 0.45;

/// Durée pendant laquelle on peut encore sauter après avoir quitté le sol.
const COYOTE_TIME: f32 = 0.10;
/// Durée pendant laquelle un appui saut anticipé reste mémorisé.
const JUMP_BUFFER: f32 = 0.12;

#[derive(Component)]
pub struct Player;

/// État du contrôleur. Regroupé pour éviter d'éparpiller la logique
/// temporelle (timers) dans plusieurs composants.
#[derive(Component, Default)]
pub struct PlayerController {
    /// Temps depuis qu'on a quitté le sol. Si < COYOTE_TIME, saut autorisé.
    pub coyote_timer: f32,
    /// Temps depuis le dernier appui saut. Si < JUMP_BUFFER au moment
    /// d'atterrir, saut auto.
    pub jump_buffer_timer: f32,
    /// Le joueur est-il en train de monter dans un saut (utile pour le
    /// saut variable : le cut ne s'applique qu'en phase ascendante).
    pub is_jumping: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                tick_player_timers,
                handle_horizontal_input,
                handle_jump_input,
            )
                .chain()
                .before(PhysicsSet),
        );
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerController::default(),
        Velocity::default(),
        Collider::new(PLAYER_SIZE),
        Grounded::default(),
        SpriteBundle {
            sprite: Sprite {
                color: PLAYER_COLOR,
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            transform: Transform::from_translation(SPAWN_POS.extend(1.0)),
            ..default()
        },
    ));
}

fn tick_player_timers(time: Res<Time>, mut q: Query<(&mut PlayerController, &Grounded)>) {
    let dt = time.delta_seconds();
    for (mut ctrl, grounded) in &mut q {
        if grounded.0 {
            ctrl.coyote_timer = 0.0;
        } else {
            ctrl.coyote_timer += dt;
        }
        // Le jump buffer compte toujours, peu importe l'état au sol.
        ctrl.jump_buffer_timer += dt;
    }
}

fn handle_horizontal_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Velocity, &Grounded), With<Player>>,
) {
    let dt = time.delta_seconds();
    let mut dir = 0.0;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir += 1.0;
    }

    for (mut velocity, grounded) in &mut q {
        let target = dir * MOVE_SPEED;
        let accel = if grounded.0 { ACCEL } else { AIR_ACCEL };

        if dir != 0.0 {
            // Approche linéaire de la vitesse cible
            let delta = (target - velocity.0.x).clamp(-accel * dt, accel * dt);
            velocity.0.x += delta;
        } else if grounded.0 {
            // Friction uniquement au sol pour préserver l'élan en l'air
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
    let jump_pressed =
        keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::ArrowUp)
            || keys.just_pressed(KeyCode::KeyW);
    let jump_released =
        keys.just_released(KeyCode::Space) || keys.just_released(KeyCode::ArrowUp)
            || keys.just_released(KeyCode::KeyW);

    for (mut velocity, mut ctrl, grounded) in &mut q {
        if jump_pressed {
            ctrl.jump_buffer_timer = 0.0;
        }

        // Saut effectif : buffer récent ET (au sol OU encore dans la
        // fenêtre coyote time). Le coyote_timer est à 0 quand grounded,
        // donc la condition couvre les deux cas.
        let can_jump = ctrl.coyote_timer < COYOTE_TIME || grounded.0;
        let buffered = ctrl.jump_buffer_timer < JUMP_BUFFER;

        if can_jump && buffered {
            velocity.0.y = JUMP_VELOCITY;
            ctrl.is_jumping = true;
            // On consomme buffer et coyote pour éviter un re-déclenchement
            ctrl.jump_buffer_timer = JUMP_BUFFER;
            ctrl.coyote_timer = COYOTE_TIME;
        }

        // Saut variable : si on relâche tôt en montant, on coupe.
        if jump_released && ctrl.is_jumping && velocity.0.y > 0.0 {
            velocity.0.y *= JUMP_CUT_FACTOR;
            ctrl.is_jumping = false;
        }

        // On finit le "is_jumping" dès qu'on amorce la descente.
        if velocity.0.y <= 0.0 {
            ctrl.is_jumping = false;
        }
    }
}
