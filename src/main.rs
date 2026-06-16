mod audio;
mod camera;
mod effects;
mod enemies;
mod heroes;
mod items;
mod level;
mod parallax;
mod physics;
mod player;
mod save;
mod states;
mod throwables;
mod ui;
mod weapons;
mod world;

use bevy::prelude::*;

const WINDOW_TITLE: &str = "Adventure Timing";
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
// Ciel coucher rose pâle, ambiance camille-unknown-home.
const SKY: Color = Color::srgb(0.94, 0.70, 0.74);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(SKY))
        // Splittés en deux groupes car le tuple add_plugins est limité à 15.
        .add_plugins((
            save::SavePlugin,
            states::StatesPlugin,
            heroes::HeroesPlugin,
            audio::AudioPlugin,
            physics::PhysicsPlugin,
            world::WorldPlugin,
            level::LevelPlugin,
            items::ItemsPlugin,
            throwables::ThrowablesPlugin,
            weapons::WeaponsPlugin,
            enemies::EnemiesPlugin,
            player::PlayerPlugin,
        ))
        .add_plugins((
            effects::EffectsPlugin,
            camera::CameraPlugin,
            parallax::ParallaxPlugin,
            ui::UiPlugin,
        ))
        .run();
}
