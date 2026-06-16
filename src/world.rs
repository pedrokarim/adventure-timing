//! Construction du niveau. Le sol et les plateformes utilisent des
//! textures tilées (ImageScaleMode::Tiled) afin que les motifs se
//! répètent à l'identique quelle que soit la taille du solide.

use crate::level::{spawn_checkpoint, spawn_goal, spawn_spike_field};
use crate::physics::{Collider, Solid};
use bevy::prelude::*;

pub const PLAYER_SPAWN: Vec2 = Vec2::new(-600.0, -100.0);

pub const TOTAL_LEVELS: u32 = 5;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LevelId {
    #[default]
    PinkSunset,
    NightForest,
    AmberRuins,
    Sanctuary,
    Dawn,
}

impl LevelId {
    pub fn number(self) -> u32 {
        match self {
            LevelId::PinkSunset => 1,
            LevelId::NightForest => 2,
            LevelId::AmberRuins => 3,
            LevelId::Sanctuary => 4,
            LevelId::Dawn => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LevelId::PinkSunset => "Au commencement",
            LevelId::NightForest => "Foret silencieuse",
            LevelId::AmberRuins => "Ruines d'ambre",
            LevelId::Sanctuary => "Sanctuaire",
            LevelId::Dawn => "Aurore",
        }
    }

    pub fn next(self) -> Option<LevelId> {
        match self {
            LevelId::PinkSunset => Some(LevelId::NightForest),
            LevelId::NightForest => Some(LevelId::AmberRuins),
            LevelId::AmberRuins => Some(LevelId::Sanctuary),
            LevelId::Sanctuary => Some(LevelId::Dawn),
            LevelId::Dawn => None,
        }
    }

    pub fn sky(self) -> Color {
        match self {
            LevelId::PinkSunset => Color::srgb(0.94, 0.70, 0.74),
            LevelId::NightForest => Color::srgb(0.06, 0.10, 0.14),
            LevelId::AmberRuins => Color::srgb(0.22, 0.13, 0.08),
            LevelId::Sanctuary => Color::srgb(0.04, 0.04, 0.06),
            LevelId::Dawn => Color::srgb(0.85, 0.85, 0.87),
        }
    }

    /// Couleur représentative du niveau pour la Carte du voyage.
    pub fn badge_color(self) -> Color {
        match self {
            LevelId::PinkSunset => Color::srgb(0.96, 0.55, 0.65),
            LevelId::NightForest => Color::srgb(0.22, 0.34, 0.50),
            LevelId::AmberRuins => Color::srgb(0.91, 0.66, 0.30),
            LevelId::Sanctuary => Color::srgb(0.86, 0.25, 0.31),
            LevelId::Dawn => Color::srgb(0.92, 0.92, 0.94),
        }
    }

    /// Tagline courte pour la carte (1 ligne).
    pub fn tagline(self) -> &'static str {
        match self {
            LevelId::PinkSunset => "Coucher rose, monolithes flottants",
            LevelId::NightForest => "Foret bleu nuit, arbre geant",
            LevelId::AmberRuins => "Ruines de cuivre, lumieres mortes",
            LevelId::Sanctuary => "Sanctuaire noir, rouge sang",
            LevelId::Dawn => "Aurore blanche, voyage acheve",
        }
    }

    /// Numéro romain pour l'affichage stylé.
    pub fn roman(self) -> &'static str {
        match self {
            LevelId::PinkSunset => "I",
            LevelId::NightForest => "II",
            LevelId::AmberRuins => "III",
            LevelId::Sanctuary => "IV",
            LevelId::Dawn => "V",
        }
    }

    pub fn all() -> &'static [LevelId] {
        &[
            LevelId::PinkSunset,
            LevelId::NightForest,
            LevelId::AmberRuins,
            LevelId::Sanctuary,
            LevelId::Dawn,
        ]
    }

    fn suffix(self) -> &'static str {
        match self {
            LevelId::PinkSunset => "",
            LevelId::NightForest => "_forest",
            LevelId::AmberRuins => "_amber",
            LevelId::Sanctuary => "_sanctuary",
            LevelId::Dawn => "_dawn",
        }
    }

    fn ground(self) -> String { format!("sprites/tile_ground{}.png", self.suffix()) }
    fn grass(self) -> String { format!("sprites/tile_grass{}.png", self.suffix()) }
    fn platform(self) -> String { format!("sprites/tile_platform{}.png", self.suffix()) }
    fn wall(self) -> String { format!("sprites/tile_wall{}.png", self.suffix()) }
    pub fn parallax_back(self) -> String { format!("sprites/parallax_back{}.png", self.suffix()) }
    pub fn parallax_mid(self) -> String { format!("sprites/parallax_mid{}.png", self.suffix()) }
    pub fn parallax_front(self) -> String { format!("sprites/parallax_front{}.png", self.suffix()) }
}

#[derive(Resource, Clone, Copy, Default, Debug)]
pub struct CurrentLevel(pub LevelId);

/// Marqueur de toutes les entités spawnées par le niveau (tiles, hazards,
/// checkpoints, goal, parallax). Permet de tout despawn pour transition.
#[derive(Component)]
pub struct LevelEntity;

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
        app.init_resource::<CurrentLevel>()
            .add_systems(Startup, spawn_level)
            .add_systems(Update, handle_level_transition);
    }
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
    mut clear_color: ResMut<ClearColor>,
) {
    let level = current_level.0;
    clear_color.0 = level.sky();

    // === Section 0 : zone d'échauffement ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(-700.0, -320.0), Vec2::new(800.0, 80.0), Tile::Ground);

    // === Section 1 : escaliers de plateformes ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(-180.0, -240.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(50.0, -160.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(280.0, -80.0), Vec2::new(160.0, 32.0), Tile::Platform);

    spawn_checkpoint(&mut commands, &asset_server, Vec2::new(280.0, -32.0), Vec2::new(280.0, -32.0));

    // === Section 2 : passage avec pics ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(620.0, -320.0), Vec2::new(700.0, 80.0), Tile::Ground);
    spawn_spike_field(&mut commands, &asset_server, Vec2::new(620.0, -268.0), 448.0);

    spawn_solid(&mut commands, &asset_server, level, Vec2::new(450.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(640.0, -80.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(840.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);

    // === Section 3 : ascension verticale ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(1020.0, -120.0), Vec2::new(64.0, 256.0), Tile::Wall);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(880.0, 20.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(700.0, 120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(880.0, 220.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(1080.0, 320.0), Vec2::new(192.0, 32.0), Tile::Platform);

    spawn_checkpoint(&mut commands, &asset_server, Vec2::new(1080.0, 368.0), Vec2::new(1080.0, 368.0));

    // === Section 4 : gap périlleux ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(1320.0, 300.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(1560.0, 280.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(1800.0, 260.0), Vec2::new(96.0, 32.0), Tile::Platform);

    // === Section 5 : arrivée ===
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(2100.0, 220.0), Vec2::new(384.0, 32.0), Tile::Ground);
    spawn_goal(&mut commands, &asset_server, Vec2::new(2200.0, 276.0));
    spawn_solid(&mut commands, &asset_server, level, Vec2::new(2300.0, 270.0), Vec2::new(64.0, 128.0), Tile::Wall);
}

fn spawn_solid(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level: LevelId,
    pos: Vec2,
    size: Vec2,
    tile: Tile,
) {
    let texture_path = match tile {
        Tile::Ground => level.ground(),
        Tile::Platform => level.platform(),
        Tile::Wall => level.wall(),
    };

    commands.spawn((
        LevelEntity,
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
            tile_y: true,
            stretch_value: 1.0,
        },
        Collider::new(size),
        Solid,
    ));

    if matches!(tile, Tile::Ground) {
        let grass_y = pos.y + size.y * 0.5 + GRASS_STRIP_HEIGHT * 0.5 - 4.0;
        commands.spawn((
            LevelEntity,
            SpriteBundle {
                texture: asset_server.load(level.grass()),
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

/// Listen to PlayerWon. If next level exists, switch + respawn world.
/// Else go to Win state.
#[allow(clippy::too_many_arguments)]
fn handle_level_transition(
    mut events: EventReader<crate::states::PlayerWon>,
    mut current_level: ResMut<CurrentLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    level_entities: Query<Entity, With<LevelEntity>>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
    mut player_q: Query<&mut Transform, With<crate::player::Player>>,
    mut respawn_point: ResMut<crate::level::RespawnPoint>,
    mut save_data: ResMut<crate::save::SaveData>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    if let Some(next) = current_level.0.next() {
        current_level.0 = next;
        // Débloque le niveau suivant sur la Carte du voyage
        if next.number() > save_data.highest_level {
            save_data.highest_level = next.number();
            crate::save::save_data(&save_data);
        }
        for e in &level_entities {
            commands.entity(e).despawn_recursive();
        }
        clear_color.0 = next.sky();
        respawn_point.0 = PLAYER_SPAWN;
        if let Ok(mut t) = player_q.get_single_mut() {
            t.translation.x = PLAYER_SPAWN.x;
            t.translation.y = PLAYER_SPAWN.y;
        }
        spawn_level_inline(&mut commands, &asset_server, next);
        crate::parallax::spawn_parallax_layers(&mut commands, &asset_server, next);
    } else {
        // Le dernier niveau est terminé : débloque tout (max)
        if save_data.highest_level < TOTAL_LEVELS {
            save_data.highest_level = TOTAL_LEVELS;
            crate::save::save_data(&save_data);
        }
        next_state.set(crate::states::GameState::Win);
    }
}

/// Variante inline de spawn_level pour le ré-spawn manuel (sans système).
fn spawn_level_inline(commands: &mut Commands, asset_server: &AssetServer, level: LevelId) {
    spawn_solid(commands, asset_server, level, Vec2::new(-700.0, -320.0), Vec2::new(800.0, 80.0), Tile::Ground);
    spawn_solid(commands, asset_server, level, Vec2::new(-180.0, -240.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(50.0, -160.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(280.0, -80.0), Vec2::new(160.0, 32.0), Tile::Platform);
    spawn_checkpoint(commands, asset_server, Vec2::new(280.0, -32.0), Vec2::new(280.0, -32.0));
    spawn_solid(commands, asset_server, level, Vec2::new(620.0, -320.0), Vec2::new(700.0, 80.0), Tile::Ground);
    spawn_spike_field(commands, asset_server, Vec2::new(620.0, -268.0), 448.0);
    spawn_solid(commands, asset_server, level, Vec2::new(450.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(640.0, -80.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(840.0, -120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(1020.0, -120.0), Vec2::new(64.0, 256.0), Tile::Wall);
    spawn_solid(commands, asset_server, level, Vec2::new(880.0, 20.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(700.0, 120.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(880.0, 220.0), Vec2::new(128.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(1080.0, 320.0), Vec2::new(192.0, 32.0), Tile::Platform);
    spawn_checkpoint(commands, asset_server, Vec2::new(1080.0, 368.0), Vec2::new(1080.0, 368.0));
    spawn_solid(commands, asset_server, level, Vec2::new(1320.0, 300.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(1560.0, 280.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(1800.0, 260.0), Vec2::new(96.0, 32.0), Tile::Platform);
    spawn_solid(commands, asset_server, level, Vec2::new(2100.0, 220.0), Vec2::new(384.0, 32.0), Tile::Ground);
    spawn_goal(commands, asset_server, Vec2::new(2200.0, 276.0));
    spawn_solid(commands, asset_server, level, Vec2::new(2300.0, 270.0), Vec2::new(64.0, 128.0), Tile::Wall);
}
