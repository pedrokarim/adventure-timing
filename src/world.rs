//! Niveau de test dessiné en dur. La structure progresse de gauche à
//! droite : zone d'apprentissage du saut, première section avec pics,
//! checkpoint, traversée verticale, descente, gros gap, drapeau final.

use crate::level::{spawn_checkpoint, spawn_goal, spawn_spike};
use crate::physics::{Collider, Solid};
use bevy::prelude::*;

const GROUND_COLOR: Color = Color::srgb(0.18, 0.42, 0.22);
const PLATFORM_COLOR: Color = Color::srgb(0.55, 0.38, 0.22);
const WALL_COLOR: Color = Color::srgb(0.40, 0.28, 0.18);

/// Position de spawn initiale du joueur. Doit correspondre au respawn
/// par défaut dans level.rs.
pub const PLAYER_SPAWN: Vec2 = Vec2::new(-600.0, -100.0);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands) {
    // === Section 0 : zone d'échauffement ===
    // Sol large où on apprend à marcher / sauter sans risque.
    spawn_solid(&mut commands, Vec2::new(-700.0, -320.0), Vec2::new(800.0, 80.0), GROUND_COLOR);

    // === Section 1 : escaliers de plateformes ===
    spawn_solid(&mut commands, Vec2::new(-180.0, -240.0), Vec2::new(160.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(50.0, -160.0), Vec2::new(160.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(280.0, -80.0), Vec2::new(160.0, 24.0), PLATFORM_COLOR);

    // Premier checkpoint en haut des escaliers.
    spawn_checkpoint(
        &mut commands,
        Vec2::new(280.0, -38.0),
        Vec2::new(280.0, -60.0),
    );

    // === Section 2 : passage avec pics ===
    // Sol bas avec une rangée de pics, oblige à sauter de plateforme en plateforme.
    spawn_solid(&mut commands, Vec2::new(620.0, -320.0), Vec2::new(700.0, 80.0), GROUND_COLOR);
    for i in 0..7 {
        let x = 380.0 + (i as f32) * 80.0;
        spawn_spike(&mut commands, Vec2::new(x, -268.0), Vec2::new(50.0, 26.0));
    }
    // Plateformes au-dessus pour traverser.
    spawn_solid(&mut commands, Vec2::new(450.0, -120.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(640.0, -80.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(840.0, -120.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);

    // === Section 3 : ascension verticale ===
    // Mur à droite force à monter.
    spawn_solid(&mut commands, Vec2::new(1020.0, -120.0), Vec2::new(40.0, 240.0), WALL_COLOR);

    spawn_solid(&mut commands, Vec2::new(880.0, 20.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(700.0, 120.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(880.0, 220.0), Vec2::new(120.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(1080.0, 320.0), Vec2::new(180.0, 24.0), PLATFORM_COLOR);

    // Deuxième checkpoint au sommet, juste avant la traversée finale.
    spawn_checkpoint(
        &mut commands,
        Vec2::new(1080.0, 362.0),
        Vec2::new(1080.0, 340.0),
    );

    // === Section 4 : gap périlleux ===
    // Trois piliers étroits espacés au-dessus du vide.
    spawn_solid(&mut commands, Vec2::new(1320.0, 300.0), Vec2::new(80.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(1560.0, 280.0), Vec2::new(80.0, 24.0), PLATFORM_COLOR);
    spawn_solid(&mut commands, Vec2::new(1800.0, 260.0), Vec2::new(80.0, 24.0), PLATFORM_COLOR);

    // === Section 5 : arrivée ===
    spawn_solid(&mut commands, Vec2::new(2100.0, 220.0), Vec2::new(360.0, 24.0), GROUND_COLOR);
    spawn_goal(&mut commands, Vec2::new(2200.0, 272.0));

    // Mur final pour borner le niveau.
    spawn_solid(&mut commands, Vec2::new(2300.0, 270.0), Vec2::new(40.0, 120.0), WALL_COLOR);
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
