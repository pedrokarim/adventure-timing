//! Décor de fond parallaxé en 3 couches : pics lointains avec étoiles,
//! pics intermédiaires, forêt en silhouette au premier plan. Toutes les
//! textures sont générées dans examples/gen_assets.rs.

use bevy::prelude::*;

struct ParallaxLayer {
    /// 0 = fixe au monde, 1 = collé à la caméra. Plus petit = plus loin.
    factor: f32,
    /// Path de la texture relative à `assets/`.
    texture: &'static str,
    /// Y en coordonnées monde où l'image est centrée.
    y: f32,
    /// Taille rendue de la tuile en monde. La largeur sert à tiler le long
    /// de la caméra ; la hauteur correspond à la hauteur native du PNG.
    size: Vec2,
    /// Profondeur Z (toujours négative pour rester derrière le gameplay).
    z: f32,
}

const LAYERS: &[ParallaxLayer] = &[
    // Pics lointains avec étoiles
    ParallaxLayer {
        factor: 0.10,
        texture: "sprites/parallax_back.png",
        y: 60.0,
        size: Vec2::new(512.0, 360.0),
        z: -12.0,
    },
    // Pics moyens
    ParallaxLayer {
        factor: 0.30,
        texture: "sprites/parallax_mid.png",
        y: -30.0,
        size: Vec2::new(512.0, 280.0),
        z: -11.0,
    },
    // Forêt en silhouette devant
    ParallaxLayer {
        factor: 0.55,
        texture: "sprites/parallax_front.png",
        y: -160.0,
        size: Vec2::new(512.0, 200.0),
        z: -10.0,
    },
];

/// Nombre de tuiles spawnées par couche pour couvrir le niveau.
const TILE_COUNT: i32 = 10;
const START_X: f32 = -1400.0;

#[derive(Component)]
struct Parallax {
    factor: f32,
    base_x: f32,
}

pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_layers)
            .add_systems(Update, update_parallax);
    }
}

fn spawn_layers(mut commands: Commands, asset_server: Res<AssetServer>) {
    for layer in LAYERS {
        for i in 0..TILE_COUNT {
            let x = START_X + i as f32 * layer.size.x;
            commands.spawn((
                Parallax {
                    factor: layer.factor,
                    base_x: x,
                },
                SpriteBundle {
                    texture: asset_server.load(layer.texture),
                    sprite: Sprite {
                        custom_size: Some(layer.size),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, layer.y, layer.z),
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
