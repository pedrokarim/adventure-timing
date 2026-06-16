//! Sélection de héros. Trois personnages distincts avec leurs propres
//! stats et sprites. Le héros choisi est persisté dans la save.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub enum Hero {
    /// L'Errante : équilibrée, cape sombre + mèche blanche, broche ambre.
    #[default]
    Wanderer,
    /// Le Funambule : rapide, agile, triple saut, cape bleue.
    Tightrope,
    /// Le Gardien : lourd, saut puissant, pas de double saut mais invul
    /// 0.3 s après respawn, armure beige.
    Guardian,
}

impl Hero {
    pub fn all() -> &'static [Hero] {
        &[Hero::Wanderer, Hero::Tightrope, Hero::Guardian]
    }

    pub fn label(self) -> &'static str {
        match self {
            Hero::Wanderer => "L'Errante",
            Hero::Tightrope => "Le Funambule",
            Hero::Guardian => "Le Gardien",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            Hero::Wanderer => "Equilibree, polyvalente",
            Hero::Tightrope => "Rapide, triple saut",
            Hero::Guardian => "Lourd, saut puissant",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Hero::Wanderer => {
                "La voyageuse par defaut : vitesse 320, saut 760, double saut.\nA toi pour explorer."
            }
            Hero::Tightrope => {
                "L'acrobate : vitesse 380, saut 700, TRIPLE saut.\nPour le platforming serre."
            }
            Hero::Guardian => {
                "Le tank : vitesse 240, saut 820, pas de double saut.\nInvincible 0.3 s apres chaque mort."
            }
        }
    }

    pub fn sprite_path(self) -> &'static str {
        match self {
            Hero::Wanderer => "sprites/player.png",
            Hero::Tightrope => "sprites/player_tightrope.png",
            Hero::Guardian => "sprites/player_guardian.png",
        }
    }

    /// Preview standalone (1 frame) pour l'écran de sélection.
    pub fn preview_path(self) -> &'static str {
        match self {
            Hero::Wanderer => "sprites/preview_wanderer.png",
            Hero::Tightrope => "sprites/preview_tightrope.png",
            Hero::Guardian => "sprites/preview_guardian.png",
        }
    }

    pub fn move_speed(self) -> f32 {
        match self {
            Hero::Wanderer => 320.0,
            Hero::Tightrope => 380.0,
            Hero::Guardian => 240.0,
        }
    }

    pub fn jump_velocity(self) -> f32 {
        match self {
            Hero::Wanderer => 760.0,
            Hero::Tightrope => 700.0,
            Hero::Guardian => 820.0,
        }
    }

    pub fn air_jump_velocity(self) -> f32 {
        match self {
            Hero::Wanderer => 680.0,
            Hero::Tightrope => 640.0,
            Hero::Guardian => 0.0, // pas de saut en l'air
        }
    }

    pub fn max_air_jumps(self) -> u8 {
        match self {
            Hero::Wanderer => 1,
            Hero::Tightrope => 2,
            Hero::Guardian => 0,
        }
    }

    pub fn respawn_invincibility(self) -> f32 {
        match self {
            Hero::Guardian => 0.5,
            _ => 0.0,
        }
    }
}

/// Le héros actuellement sélectionné. Lu par player.rs et les autres
/// modules qui s'adaptent aux stats.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SelectedHero(pub Hero);

pub struct HeroesPlugin;

impl Plugin for HeroesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedHero>();
    }
}
