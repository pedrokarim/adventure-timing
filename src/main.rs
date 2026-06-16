use bevy::prelude::*;

const WINDOW_TITLE: &str = "Adventure Timing";
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

const SKY: Color = Color::srgb(0.45, 0.65, 0.85);
const PLAYER_COLOR: Color = Color::srgb(0.85, 0.30, 0.30);
const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 48.0);

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: PLAYER_COLOR,
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        ..default()
    });
}
