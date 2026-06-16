//! Caméra 2D qui suit le joueur avec un smoothing exponentiel et un
//! léger lookahead horizontal dans la direction du mouvement.

use crate::physics::{PhysicsSet, Velocity};
use crate::player::Player;
use bevy::prelude::*;

/// Vitesse de convergence de la caméra vers la cible. Plus haut = plus
/// rigide. Ordre de grandeur 6.0 pour un suivi confortable.
const FOLLOW_LERP_RATE: f32 = 6.0;
/// Décalage horizontal max en fonction de la vélocité du joueur (px).
const LOOKAHEAD_MAX: f32 = 90.0;
/// Vélocité du joueur à laquelle on atteint le lookahead max (px/s).
const LOOKAHEAD_REF_SPEED: f32 = 280.0;
/// Légère élévation verticale pour mieux voir devant.
const VERTICAL_OFFSET: f32 = 40.0;

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
    // Zoom : on divise par 2 la zone visible pour que les sprites
    // pixel-art (32x48) occupent une part lisible de l'écran.
    bundle.projection.scale = 0.5;
    commands.spawn(bundle);
}

fn follow_player(
    time: Res<Time>,
    player: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let Ok((player_t, player_v)) = player.get_single() else {
        return;
    };
    let Ok(mut cam_t) = camera.get_single_mut() else {
        return;
    };

    let lookahead = (player_v.0.x / LOOKAHEAD_REF_SPEED).clamp(-1.0, 1.0) * LOOKAHEAD_MAX;
    let target = Vec3::new(
        player_t.translation.x + lookahead,
        player_t.translation.y + VERTICAL_OFFSET,
        cam_t.translation.z,
    );

    // Smoothing indépendant du framerate : facteur exponentiel.
    let alpha = 1.0 - (-FOLLOW_LERP_RATE * time.delta_seconds()).exp();
    cam_t.translation = cam_t.translation.lerp(target, alpha);
}
