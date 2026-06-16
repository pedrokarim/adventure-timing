mod camera;
mod effects;
mod level;
mod parallax;
mod physics;
mod player;
mod states;
mod ui;
mod world;

use bevy::prelude::*;

const WINDOW_TITLE: &str = "Adventure Timing";
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const SKY: Color = Color::srgb(0.45, 0.65, 0.85);

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
            states::StatesPlugin,
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
