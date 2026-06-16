# Getting started

## Prérequis

```bash
# Rust stable (>= 1.80 recommandé)
rustup update stable

# Dépendances système Linux (Ubuntu/Debian)
sudo apt install -y \
    g++ pkg-config libx11-dev libasound2-dev libudev-dev \
    libxkbcommon-dev libwayland-dev mold

# Linker rapide (mold) — accélère les rebuilds
```

## Configuration du linker rapide

Créer `.cargo/config.toml` :

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## Initialisation du projet

```bash
cd /home/karim/Desktop/programming-laboratory/projects-ascencia/adventure-timing
cargo init --name adventure_timing
```

## Premier `main.rs` minimal

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Adventure Timing".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.8, 0.3, 0.3),
            custom_size: Some(Vec2::new(32., 48.)),
            ..default()
        },
        ..default()
    });
}
```

## Lancer

```bash
cargo run
```

Premier compile = lent (5-10 min). Les suivants sont rapides grâce à `dynamic_linking` + `mold`.

## Prochaine étape

Suivre la [roadmap](04-roadmap.md), étape 1.

## Ressources

- [Bevy Cheat Book](https://bevy-cheatbook.github.io/) — référence pratique
- [Bevy examples](https://github.com/bevyengine/bevy/tree/main/examples) — code officiel
- [Game Feel (Steve Swink)](https://www.gamefeelbook.com/) — bible du juicing
- [Celeste — Forging Lonely Roads](https://www.youtube.com/watch?v=4Wj4uoEM_e0) — étude du contrôleur
- [LDtk docs](https://ldtk.io/docs/) — éditeur de niveaux
