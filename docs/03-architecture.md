# Architecture

État au `v1.3.0+` : architecture réellement implémentée + chantiers
prévus (modules vides, à créer).

## Arborescence

### Existant

```
adventure-timing/
├── Cargo.toml
├── assets/
│   ├── fonts/             DejaVuSansMono regular + bold (TTF)
│   ├── inspirations/      Réfs Camille Unknown (.webp, .gif)
│   ├── sfx/               WAV procéduraux (jump, land, death, …)
│   └── sprites/           PNG procéduraux (player, tiles, parallax, …)
├── docs/
├── examples/
│   ├── gen_assets.rs      Génère tous les PNG via crate image
│   └── gen_audio.rs       Génère tous les WAV SFX via crate hound
└── src/
    ├── main.rs            Composition des plugins + DefaultPlugins
    ├── states.rs          GameState + RunStats + events globaux
    ├── save.rs            SaveData + Settings sérialisés JSON
    ├── physics.rs         Velocity, Collider, Solid, Grounded, AABB
    ├── world.rs           Spawn niveau hardcodé (5 sections)
    ├── level.rs           Pics + checkpoints + goal + collision tests
    ├── player.rs          Input, contrôleur, animation, double saut
    ├── camera.rs          Caméra cinématique (deadzone + lookahead)
    ├── effects.rs         Squash/stretch, screen shake, particules
    ├── parallax.rs        3 couches background, scroll par camera.x
    ├── audio.rs           SFX (events → PlaybackBundle)
    └── ui.rs              Menus, HUD, buttons, font loading
```

### À créer (chantiers prévus, voir docs/04-roadmap.md)

```
src/
├── items.rs               Collectibles passifs (Étape C)
├── throwables.rs          Items projetables + posables + inventaire (C')
├── weapons.rs             Armes + hitbox + combat (C'')
├── enemies.rs             IA + stomp + projectiles ennemis (E)
└── levels.rs              LevelId enum + multi-niveau (A) - ou refactor world.rs

examples/
└── gen_music.rs           Boucles musicales générées (D)

assets/
├── music/                 Boucles WAV par niveau (D)
└── levels/                Fichiers .ron ou .ldtk pour data-driven (A)
```

## Découpage en plugins Bevy

Chaque module expose un `Plugin`. Le `main.rs` agrège :

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin { ... }))
        .insert_resource(ClearColor(SKY))
        .add_plugins((
            // Données / état global
            save::SavePlugin,
            states::StatesPlugin,
            // Audio (chargé tôt pour qu'il soit dispo dans les SFX)
            audio::AudioPlugin,
            // Simulation
            physics::PhysicsPlugin,
            world::WorldPlugin,
            level::LevelPlugin,
            player::PlayerPlugin,
            // Présentation
            effects::EffectsPlugin,
            camera::CameraPlugin,
            parallax::ParallaxPlugin,
            ui::UiPlugin,
            // À venir :
            // items::ItemsPlugin,
            // throwables::ThrowablesPlugin,
            // weapons::WeaponsPlugin,
            // enemies::EnemiesPlugin,
        ))
        .run();
}
```

### Ordre d'init important

- `SavePlugin` **avant** `UiPlugin` (le menu lit `SaveData` pour
  afficher les records et le bouton "Continuer")
- `AudioPlugin` **avant** les modules qui émettent ses events (player,
  effects, level) — sinon les `add_event` n'existent pas au moment où
  on lit l'event
- `StatesPlugin` **avant** tout le reste (init du GameState que tout
  le monde consulte)

## Choix techniques importants

### Contrôleur kinematic

Un platformer veut un contrôle précis (saut à hauteur exacte, coyote,
buffer, double saut, slowmo magique potentiel). Une rigid body
dynamique subirait la physique du moteur et serait imprécise.
**Implémentation** : `src/physics.rs` bouge manuellement la position
avec résolution AABB axe par axe (X puis Y), pas de moteur physique
externe (pas de rapier).

### Coyote time + jump buffer + double saut

Trois timers dans `PlayerController` :
- **Coyote time** (`COYOTE_TIME = 0.10`) : saut autorisé peu après avoir
  quitté une plateforme
- **Jump buffer** (`JUMP_BUFFER = 0.12`) : un appui saut anticipé est
  mémorisé puis consommé à l'atterrissage
- **Air jumps remaining** (`MAX_AIR_JUMPS = 1`) : compteur de sauts en
  l'air, rechargé au contact du sol

### Saut variable

`if jump_released && velocity.0.y > 0.0 → velocity.0.y *= JUMP_CUT_FACTOR`
Garder le bouton enfoncé = saut plus haut.

### Caméra cinématique

Dans `src/camera.rs` :
- **Deadzone** X = 24 px, Y = 16 px (le perso bouge dedans sans déplacer
  la caméra)
- **Lookahead horizontal** lissé (lerp 3.5) jusqu'à 110 px selon la
  vélocité X
- **Anticipation verticale** asymétrique : 60 px en montée, 90 px en
  chute (plus généreux vers le bas pour voir les gaps qui arrivent)
- **Lerp séparés** : X 7.0, Y 4.5 (vertical plus mou, évite le tremblement
  caméra sur les sauts)
- **Projection scale** 0.5 (zoom 2× pour rendre lisible le pixel art)

### États du jeu

```rust
#[derive(States, Default, Hash, Eq, PartialEq, Debug, Clone)]
pub enum GameState {
    #[default]
    MainMenu,
    Settings,
    Credits,
    Playing,
    Paused,
    GameOver,
    Win,
}
```

Systèmes gateables avec `.run_if(in_state(GameState::Playing))`. La
physique et l'input du joueur sont gatés à Playing pour pas que la
gravité s'applique en menu.

### Events globaux

Définis dans `src/states.rs` (anciens) et `src/audio.rs` (nouveaux) :

- `PlayerDied` (states.rs) — touché un pic ou tombé sous y=-800
- `PlayerWon` (states.rs) — touché le drapeau de fin
- `PlayerJumped` (audio.rs) — saut au sol OU air
- `PlayerAirJumped` (audio.rs) — saut en l'air uniquement (pour le ring
  de particules)
- `PlayerLanded(f32)` (audio.rs) — atterrissage avec intensité [0, 1]
- `CheckpointReached` (audio.rs) — nouveau checkpoint activé

À venir (chantiers items/armes/ennemis) :

- `PickupEvent { entity, kind }` — item ramassé
- `ThrowItem` — joueur lance l'item sélectionné
- `PlaceItem` — joueur pose l'item sélectionné
- `EnemyHit { entity, damage, knockback }` — n'importe quoi touche un
  ennemi
- `PlayerHit { damage, source }` — joueur prend des dégâts

## Conventions

- **Pas de mocks** dans le code de prod, les tests intégrés tournent
  sur du vrai monde
- **Pas de feature flags / backwards-compat** : on change le code
  directement (jeu solo, pas de prod déployée)
- **Pas de comments triviaux** : commenter le pourquoi non-évident, pas
  le quoi
- **Assets régénérables** : tous les sprites et SFX sont produits par
  des binaires Rust (`examples/gen_assets.rs`, `examples/gen_audio.rs`).
  Pas de PNG dessiné à la main commité.
  - Exception : `assets/fonts/*.ttf` (DejaVu) et
    `assets/inspirations/*.webp` (réfs Camille)
