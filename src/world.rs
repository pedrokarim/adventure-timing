//! Construction d'un niveau de test en dur. À remplacer par LDtk plus
//! tard. Sert uniquement à valider le contrôleur du joueur.

use crate::physics::{Collider, Solid};
use bevy::prelude::*;

const GROUND_COLOR: Color = Color::srgb(0.18, 0.42, 0.22);
const PLATFORM_COLOR: Color = Color::srgb(0.55, 0.38, 0.22);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_level);
    }
}

fn spawn_test_level(mut commands: Commands) {
    // Sol principal, large et fin
    spawn_solid(
        &mut commands,
        Vec2::new(0.0, -320.0),
        Vec2::new(4000.0, 80.0),
        GROUND_COLOR,
    );

    // Plateformes à différentes hauteurs pour tester le saut
    let platforms = [
        (Vec2::new(-400.0, -200.0), Vec2::new(180.0, 24.0)),
        (Vec2::new(-150.0, -100.0), Vec2::new(180.0, 24.0)),
        (Vec2::new(150.0, -40.0), Vec2::new(180.0, 24.0)),
        (Vec2::new(450.0, 40.0), Vec2::new(180.0, 24.0)),
        (Vec2::new(750.0, 120.0), Vec2::new(180.0, 24.0)),
        (Vec2::new(1050.0, 60.0), Vec2::new(180.0, 24.0)),
        // un petit mur pour tester le blocage horizontal
        (Vec2::new(1300.0, -240.0), Vec2::new(40.0, 120.0)),
    ];

    for (pos, size) in platforms {
        spawn_solid(&mut commands, pos, size, PLATFORM_COLOR);
    }
}

fn spawn_solid(commands: &mut Commands, pos: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        Collider::new(size),
        Solid,
    ));
}
