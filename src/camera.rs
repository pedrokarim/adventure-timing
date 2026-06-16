//! Caméra 2D cinématique : suivi du joueur avec deadzone, lookahead
//! horizontal lissé, anticipation verticale (regarde plus haut en saut,
//! plus bas en chute), et lerp séparé par axe pour un feel naturel.

use crate::physics::{PhysicsSet, Velocity};
use crate::player::Player;
use bevy::prelude::*;

/// Lerp horizontal. Plus haut = plus rigide.
const FOLLOW_LERP_X: f32 = 7.0;
/// Lerp vertical. Plus bas que l'horizontal pour absorber les sauts
/// sans faire trembler la caméra.
const FOLLOW_LERP_Y: f32 = 4.5;

/// Demi-largeur de la zone morte horizontale (px monde). Tant que le
/// joueur reste dans cette zone autour de la cible courante, la caméra
/// ne suit pas.
const DEADZONE_X: f32 = 24.0;
/// Demi-hauteur de la zone morte verticale.
const DEADZONE_Y: f32 = 16.0;

/// Lookahead horizontal max (px). Lissé via son propre lerp pour
/// éviter le snap quand on change brusquement de direction.
const LOOKAHEAD_MAX_X: f32 = 110.0;
const LOOKAHEAD_LERP: f32 = 3.5;

/// Anticipation verticale basée sur la vélocité Y du joueur (px).
const LOOKAHEAD_MAX_UP: f32 = 60.0;
const LOOKAHEAD_MAX_DOWN: f32 = 90.0;

/// Élévation constante de la caméra par rapport au joueur. Légèrement
/// au-dessus pour donner plus de vue sur ce qui arrive devant.
const BASE_OFFSET_Y: f32 = 32.0;

#[derive(Component, Default)]
struct CameraState {
    /// Lookahead courant lissé (X et Y).
    smoothed_lookahead: Vec2,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            follow_player.after(PhysicsSet),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut bundle = Camera2dBundle::default();
    bundle.projection.scale = 0.5;
    commands.spawn((bundle, CameraState::default()));
}

fn follow_player(
    time: Res<Time>,
    player: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
    mut camera: Query<(&mut Transform, &mut CameraState), With<Camera>>,
) {
    let Ok((player_t, player_v)) = player.get_single() else {
        return;
    };
    let Ok((mut cam_t, mut state)) = camera.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();

    // ----- Lookahead cible -----
    let lookahead_x = (player_v.0.x / 320.0).clamp(-1.0, 1.0) * LOOKAHEAD_MAX_X;
    // Lookahead vertical asymétrique : on regarde plus loin en chute
    // qu'en montée (donne le temps de réagir aux gaps).
    let lookahead_y = if player_v.0.y > 0.0 {
        (player_v.0.y / 760.0).min(1.0) * LOOKAHEAD_MAX_UP
    } else {
        (player_v.0.y / 900.0).max(-1.0) * LOOKAHEAD_MAX_DOWN
    };
    let target_lookahead = Vec2::new(lookahead_x, lookahead_y);

    // Lissage du lookahead lui-même : framerate-indépendant.
    let look_alpha = 1.0 - (-LOOKAHEAD_LERP * dt).exp();
    state.smoothed_lookahead = state.smoothed_lookahead.lerp(target_lookahead, look_alpha);

    // ----- Cible caméra avec deadzone -----
    let mut target_x = cam_t.translation.x;
    let mut target_y = cam_t.translation.y;

    let desired_x = player_t.translation.x + state.smoothed_lookahead.x;
    let desired_y = player_t.translation.y + BASE_OFFSET_Y + state.smoothed_lookahead.y;

    let dx = desired_x - cam_t.translation.x;
    if dx.abs() > DEADZONE_X {
        target_x = desired_x - DEADZONE_X.copysign(dx);
    }
    let dy = desired_y - cam_t.translation.y;
    if dy.abs() > DEADZONE_Y {
        target_y = desired_y - DEADZONE_Y.copysign(dy);
    }

    // ----- Lerp séparé X/Y vers la cible -----
    let alpha_x = 1.0 - (-FOLLOW_LERP_X * dt).exp();
    let alpha_y = 1.0 - (-FOLLOW_LERP_Y * dt).exp();
    cam_t.translation.x = lerp(cam_t.translation.x, target_x, alpha_x);
    cam_t.translation.y = lerp(cam_t.translation.y, target_y, alpha_y);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
