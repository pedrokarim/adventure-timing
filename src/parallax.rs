//! Décor de fond parallaxé en 3 couches : pics lointains avec étoiles,
//! pics intermédiaires, forêt en silhouette au premier plan. Toutes les
//! textures sont générées dans examples/gen_assets.rs et choisies via
//! la `Resource CurrentLevel` (voir world.rs).

use crate::world::{CurrentLevel, LevelEntity, LevelId};
use bevy::prelude::*;

struct ParallaxSlot {
    /// 0 = fixe au monde, 1 = collé à la caméra. Plus petit = plus loin.
    factor: f32,
    /// Y en coordonnées monde où l'image est centrée.
    y: f32,
    /// Taille rendue de la tuile en monde.
    size: Vec2,
    /// Profondeur Z (toujours négative pour rester derrière le gameplay).
    z: f32,
}

const LAYERS: &[ParallaxSlot] = &[
    ParallaxSlot {
        factor: 0.10,
        y: -100.0,
        size: Vec2::new(512.0, 320.0),
        z: -12.0,
    },
    ParallaxSlot {
        factor: 0.30,
        y: -200.0,
        size: Vec2::new(512.0, 260.0),
        z: -11.0,
    },
    ParallaxSlot {
        factor: 0.55,
        y: -280.0,
        size: Vec2::new(512.0, 180.0),
        z: -10.0,
    },
];

const TILE_COUNT: i32 = 10;
const START_X: f32 = -1400.0;

#[derive(Component)]
pub struct Parallax {
    factor: f32,
    base_x: f32,
}

pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_default_layers)
            .add_systems(Update, update_parallax);
    }
}

fn spawn_default_layers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
) {
    spawn_parallax_layers(&mut commands, &asset_server, current_level.0);
}

/// Spawne les 3 couches de parallax pour le niveau donné. Public car
/// `world::handle_level_transition` l'appelle à la transition.
pub fn spawn_parallax_layers(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: LevelId,
) {
    let paths = [
        level.parallax_back(),
        level.parallax_mid(),
        level.parallax_front(),
    ];
    for (slot, path) in LAYERS.iter().zip(paths) {
        for i in 0..TILE_COUNT {
            let x = START_X + i as f32 * slot.size.x;
            commands.spawn((
                LevelEntity,
                Parallax {
                    factor: slot.factor,
                    base_x: x,
                },
                SpriteBundle {
                    texture: asset_server.load(path),
                    sprite: Sprite {
                        custom_size: Some(slot.size),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, slot.y, slot.z),
                    ..default()
                },
            ));
        }
    }
}

fn update_parallax(
    camera: Query<&Transform, (With<Camera>, Without<Parallax>)>,
    mut layers: Query<(&Parallax, &mut Transform), Without<Camera>>,
) {
    let Ok(cam) = camera.get_single() else {
        return;
    };
    for (parallax, mut transform) in &mut layers {
        transform.translation.x = parallax.base_x + cam.translation.x * (1.0 - parallax.factor);
    }
}
