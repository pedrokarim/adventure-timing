//! État global du jeu. Sert à gater les systèmes (le joueur ne bouge
//! pas dans le menu, la pause fige la physique, etc.) et à piloter
//! l'affichage des écrans (HUD, menus).

use bevy::prelude::*;

#[derive(States, Default, Hash, Eq, PartialEq, Debug, Clone)]
pub enum GameState {
    #[default]
    MainMenu,
    HeroSelect,
    LevelMap,
    Tutorial,
    Settings,
    Credits,
    Playing,
    Paused,
    GameOver,
    Win,
}

/// Évènement émis quand le joueur touche un hazard ou tombe hors du
/// monde. Consommé par le système de respawn.
#[derive(Event, Debug)]
pub struct PlayerDied;

/// Évènement émis quand le joueur atteint le drapeau.
#[derive(Event, Debug)]
pub struct PlayerWon;

/// Statistiques de run, exposées dans le HUD et les écrans de fin.
#[derive(Resource, Default, Debug)]
pub struct RunStats {
    pub deaths: u32,
    pub time_seconds: f32,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<RunStats>()
            .add_event::<PlayerDied>()
            .add_event::<PlayerWon>()
            .add_systems(Update, tick_run_timer.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::MainMenu), reset_run_stats);
    }
}

fn tick_run_timer(time: Res<Time>, mut stats: ResMut<RunStats>) {
    stats.time_seconds += time.delta_seconds();
}

fn reset_run_stats(mut stats: ResMut<RunStats>) {
    *stats = RunStats::default();
}
