mod audio;
mod camera;
mod effects;
mod level;
mod parallax;
mod physics;
mod player;
mod save;
mod states;
mod ui;
mod world;

use bevy::prelude::*;

const WINDOW_TITLE: &str = "Adventure Timing";
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
// Ciel nuit mystique : teal très sombre, plus bleu que noir.
const SKY: Color = Color::srgb(0.06, 0.10, 0.16);

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
        .add_plugins((
            save::SavePlugin,
            states::StatesPlugin,
            audio::AudioPlugin,
            physics::PhysicsPlugin,
            world::WorldPlugin,
            level::LevelPlugin,
            player::PlayerPlugin,
            effects::EffectsPlugin,
            camera::CameraPlugin,
            parallax::ParallaxPlugin,
            ui::UiPlugin,
        ))
        .run();
}
