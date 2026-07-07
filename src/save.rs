//! Persistance des données de progression et des préférences.
//!
//! Deux fichiers JSON dans le répertoire de données utilisateur
//! (`~/.local/share/adventure_timing/` sur Linux) :
//! - `save.json` : meilleurs scores, total morts, runs complétées
//! - `settings.json` : fullscreen, volumes
//!
//! L'écriture est explicite (pas d'autosave à chaque frame) : on
//! sauvegarde à la fin d'une run et à la sortie d'un Settings.

use crate::heroes::Hero;
use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "ascencia";
const APPLICATION: &str = "adventure_timing";

#[derive(Resource, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveData {
    pub best_time: Option<f32>,
    pub fewest_deaths: Option<u32>,
    pub total_deaths: u32,
    pub runs_completed: u32,
    #[serde(default)]
    pub selected_hero: Hero,
    /// Niveau le plus haut atteint (1 par défaut, monte quand un niveau
    /// est terminé). Sert au déblocage sur la Carte du voyage.
    #[serde(default = "default_highest_level")]
    pub highest_level: u32,
}

fn default_highest_level() -> u32 {
    1
}

impl SaveData {
    pub fn record_run(&mut self, time: f32, deaths: u32) {
        self.runs_completed += 1;
        self.total_deaths += deaths;
        self.best_time = Some(self.best_time.map(|prev| prev.min(time)).unwrap_or(time));
        self.fewest_deaths = Some(
            self.fewest_deaths
                .map(|prev| prev.min(deaths))
                .unwrap_or(deaths),
        );
    }
}

#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub fullscreen: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            master_volume: 0.8,
            music_volume: 0.6,
            sfx_volume: 0.8,
        }
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        let save_data = load::<SaveData>("save.json").unwrap_or_default();
        let settings = load::<Settings>("settings.json").unwrap_or_default();
        // Restaure le héros sélectionné depuis la save.
        let selected_hero = crate::heroes::SelectedHero(save_data.selected_hero);
        app.insert_resource(save_data)
            .insert_resource(settings)
            .insert_resource(selected_hero);
    }
}

fn data_dir() -> Option<PathBuf> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn load<T: serde::de::DeserializeOwned>(filename: &str) -> Option<T> {
    let path = data_dir()?.join(filename);
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write<T: Serialize>(filename: &str, value: &T) {
    let Some(path) = data_dir().map(|d| d.join(filename)) else {
        return;
    };
    let Ok(content) = serde_json::to_string_pretty(value) else {
        return;
    };
    let _ = fs::write(&path, content);
}

pub fn save_data(data: &SaveData) {
    write("save.json", data);
}

pub fn save_settings(settings: &Settings) {
    write("settings.json", settings);
}
