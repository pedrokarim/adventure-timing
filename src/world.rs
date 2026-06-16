//! Construction du niveau. Le sol et les plateformes utilisent des
//! textures tilées (ImageScaleMode::Tiled) afin que les motifs se
//! répètent à l'identique quelle que soit la taille du solide.

use crate::level::{spawn_checkpoint, spawn_goal, spawn_spike_field};
use crate::physics::{Collider, Solid};
use bevy::prelude::*;

pub const PLAYER_SPAWN: Vec2 = Vec2::new(-600.0, -100.0);

/// Numéro du niveau courant (1-indexed). Le jeu n'embarque qu'un seul
/// niveau pour l'instant ; à mettre à jour quand on en ajoute.
pub const CURRENT_LEVEL: u32 = 1;
pub const TOTAL_LEVELS: u32 = 1;

/// Asset utilisé pour rendre un solide tilé.
#[derive(Clone, Copy)]
enum Tile {
    /// Sol naturel (terre + bande d'herbe au sommet).
    Ground,
    /// Plateforme en bois flottante.
    Platform,
    /// Mur de pierre (pas de bande d'herbe).
    Wall,
}

const GRASS_STRIP_HEIGHT: f32 = 12.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // === Section 0 : zone d'échauffement ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(-700.0, -320.0), Vec2::new(800.0, 80.0), Tile::Ground);

    // === Section 1 : escaliers de plateformes ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(-180.0, -240.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(50.0, -160.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(280.0, -80.0), Vec2::new(160.0, 32.0), Tile::Platform);

    spawn_checkpoint(
        &mut commands,
        &asset_server,
        Vec2::new(280.0, -32.0),
        Vec2::new(280.0, -32.0),
    );

    // === Section 2 : passage avec pics ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(620.0, -320.0), Vec2::new(700.0, 80.0), Tile::Ground);
    spawn_spike_field(&mut commands, &asset_server, Vec2::new(620.0, -268.0), 448.0);

    // Plateformes au-dessus pour traverser
    spawn_solid(&mut commands, &asset_server, Vec2::new(450.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(640.0, -80.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(840.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);

    // === Section 3 : ascension verticale ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(1020.0, -120.0), Vec2::new(64.0, 256.0), Tile::Wall);

    spawn_solid(&mut commands, &asset_server, Vec2::new(880.0, 20.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(700.0, 120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(880.0, 220.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(1080.0, 320.0), Vec2::new(192.0, 32.0), Tile::Platform);

    spawn_checkpoint(
        &mut commands,
        &asset_server,
        Vec2::new(1080.0, 368.0),
        Vec2::new(1080.0, 368.0),
    );

    // === Section 4 : gap périlleux ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(1320.0, 300.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(1560.0, 280.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, Vec2::new(1800.0, 260.0), Vec2::new(96.0, 32.0), Tile::Platform);

    // === Section 5 : arrivée ===
    spawn_solid(&mut commands, &asset_server, Vec2::new(2100.0, 220.0), Vec2::new(384.0, 32.0), Tile::Ground);
    spawn_goal(&mut commands, &asset_server, Vec2::new(2200.0, 276.0));
    spawn_solid(&mut commands, &asset_server, Vec2::new(2300.0, 270.0), Vec2::new(64.0, 128.0), Tile::Wall);
}

fn spawn_solid(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    size: Vec2,
    tile: Tile,
) {
    let (texture_path, tile_y) = match tile {
        Tile::Ground => ("sprites/tile_ground.png", true),
        Tile::Platform => ("sprites/tile_platform.png", true),
        Tile::Wall => ("sprites/tile_wall.png", true),
    };

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(texture_path),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y,
            stretch_value: 1.0,
        },
        Collider::new(size),
        Solid,
    ));

    // Pour les sols, on superpose une bande d'herbe non-collidable au
    // sommet du solide.
    if matches!(tile, Tile::Ground) {
        let grass_y = pos.y + size.y * 0.5 + GRASS_STRIP_HEIGHT * 0.5 - 4.0;
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/tile_grass.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(size.x, GRASS_STRIP_HEIGHT)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(pos.x, grass_y, 0.1)),
                ..default()
            },
            ImageScaleMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1.0,
            },
        ));
    }
}
