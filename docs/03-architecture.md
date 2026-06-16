# Architecture

## Arborescence proposée

```
adventure-timing/
├── Cargo.toml
├── assets/
│   ├── sprites/        # .aseprite ou .png
│   ├── levels/         # .ldtk
│   ├── audio/
│   └── fonts/
├── src/
│   ├── main.rs
│   ├── lib.rs          # GamePlugin qui agrège tout
│   ├── states.rs       # GameState (Splash, MainMenu, InGame, Paused, GameOver)
│   ├── player/
│   │   ├── mod.rs
│   │   ├── movement.rs # marche, saut, dash
│   │   ├── animation.rs
│   │   └── input.rs    # mapping leafwing
│   ├── physics/
│   │   ├── mod.rs
│   │   └── kinematic.rs # contrôleur kinematic (pas dynamic)
│   ├── world/
│   │   ├── mod.rs
│   │   ├── ldtk.rs
│   │   ├── camera.rs   # follow + lookahead + smoothing
│   │   └── parallax.rs
│   ├── enemies/
│   ├── ui/
│   └── audio/
└── docs/
```

## Découpage en plugins Bevy

Chaque module expose un `Plugin`. Le `main.rs` ne fait qu'orchestrer :

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            PhysicsPlugin,
            PlayerPlugin,
            WorldPlugin,
            EnemiesPlugin,
            UiPlugin,
            AudioPlugin,
        ))
        .init_state::<GameState>()
        .run();
}
```

## Choix techniques importants

### Contrôleur de personnage : Kinematic, pas Dynamic

Un platformer veut un contrôle précis (saut à hauteur exacte, coyote time, jump buffer). Une rigid body dynamique de Rapier subit la physique et c'est imprécis. Utilise un `KinematicCharacterController` ou bouge manuellement la position avec des `shape_cast` pour les collisions.

### Coyote time + jump buffer

Deux timers indispensables pour un feel correct :
- **Coyote time** (~100 ms) : autoriser le saut peu après avoir quitté une plateforme
- **Jump buffer** (~150 ms) : si le joueur appuie un peu avant de toucher le sol, déclencher le saut au contact

### Saut variable

Garder le bouton enfoncé = saut plus haut. Implémenter en réduisant la gravité tant que le bouton est tenu ET que la vélocité Y est positive.

### Caméra

- Suivi avec lerp (smoothing)
- Lookahead dans la direction du mouvement
- Deadzone au centre pour éviter le tremblement
- Clamp aux bords du niveau

## États du jeu

```rust
#[derive(States, Default, Hash, Eq, PartialEq, Debug, Clone)]
enum GameState {
    #[default]
    Splash,
    MainMenu,
    Loading,
    InGame,
    Paused,
    GameOver,
}
```

Chaque système se restreint à l'état pertinent avec `.run_if(in_state(...))`.
