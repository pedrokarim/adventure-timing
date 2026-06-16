//! Décor de fond avec parallaxe. Plusieurs couches de rectangles
//! colorés défilent à un fraction de la vitesse de la caméra pour créer
//! l'illusion de profondeur. Pas d'assets externes : couleurs unies +
//! géométrie simple répétée le long de l'axe X.

use bevy::prelude::*;

/// Couches arrière → avant. Plus le facteur est petit, plus c'est loin.
const LAYERS: &[ParallaxLayer] = &[
    ParallaxLayer {
        factor: 0.15,
        color: Color::srgb(0.30, 0.45, 0.65),
        y: -120.0,
        height: 700.0,
    },
    ParallaxLayer {
        factor: 0.35,
        color: Color::srgb(0.22, 0.38, 0.55),
        y: -180.0,
        height: 500.0,
    },
    ParallaxLayer {
        factor: 0.60,
        color: Color::srgb(0.18, 0.32, 0.45),
        y: -240.0,
        height: 320.0,
    },
];

/// Largeur d'une tuile de fond. On en spawne assez pour couvrir le
/// niveau + une marge confortable.
const TILE_WIDTH: f32 = 480.0;
const TILE_COUNT: i32 = 14;
const START_X: f32 = -1400.0;

#[derive(Clone, Copy)]
struct ParallaxLayer {
    factor: f32,
    color: Color,
    y: f32,
    height: f32,
}

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

fn spawn_layers(mut commands: Commands) {
    for (depth, layer) in LAYERS.iter().enumerate() {
        // Le Z est négatif pour passer derrière les sprites du jeu.
        let z = -10.0 - depth as f32;
        // Légère variation de hauteur pour casser l'effet "rectangle".
        for i in 0..TILE_COUNT {
            let x = START_X + i as f32 * TILE_WIDTH;
            let bump = ((i as f32) * 1.7 + depth as f32 * 0.3).sin() * 30.0;
            commands.spawn((
                Parallax {
                    factor: layer.factor,
                    base_x: x,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: layer.color,
                        custom_size: Some(Vec2::new(TILE_WIDTH + 4.0, layer.height + bump)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, layer.y + bump * 0.5, z),
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
        // (1 - factor) revient à dire : si factor=1 le tile reste collé à
        // la caméra (pas de parallaxe), si factor=0 il reste fixe au monde.
        transform.translation.x = parallax.base_x + cam.translation.x * (1.0 - parallax.factor);
    }
}
