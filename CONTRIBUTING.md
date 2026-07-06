# Contribuer à Adventure Timing

Merci de vouloir contribuer ! Ce document décrit comment le repo est
organisé, comment proposer des changements, et à quoi s'attendre côté
review.

## Prérequis

- **Rust stable** (edition 2024). Installer via [rustup](https://rustup.rs/).
- Sous Linux : dépendances système Bevy (Ubuntu/Debian) :
  ```bash
  sudo apt install libasound2-dev libudev-dev pkg-config libwayland-dev libxkbcommon-dev
  ```
- (Optionnel) `cargo install cargo-watch` pour un rebuild continu
  pendant les itérations.

## Cloner et lancer

```bash
git clone git@github.com:pedrokarim/adventure-timing.git
cd adventure-timing
cargo run --release
```

Premier build ~3-5 min (Bevy + LTO). Les suivants sont incrémentaux.

## Organisation du code

Un module Bevy = une préoccupation gameplay. Rien de magique.

| Module | Rôle |
|---|---|
| `main.rs` | wire les plugins, config `App` |
| `states.rs` | `GameState` (Menu, Playing, Paused, GameOver, Victory, LevelSelect, Tutorial) |
| `player.rs` | contrôleur kinematic, saut variable, coyote/buffer |
| `physics.rs` | AABB axis-separated, gravité, collisions statiques |
| `level.rs` | définition des niveaux, checkpoints, hazards, drapeau |
| `world.rs` | spawn du monde, tiles, parallax background |
| `camera.rs` | follow + lookahead + lerp framerate-indep |
| `parallax.rs` | couches d'arrière-plan |
| `effects.rs` | screen shake, particules, squash & stretch |
| `enemies.rs` | IA basique, patterns |
| `weapons.rs` | armes équipables |
| `throwables.rs` | projectiles |
| `items.rs` | pickups |
| `heroes.rs` | personnages jouables + sélection |
| `audio.rs` | plugin audio + hooks SFX/music |
| `ui.rs` | menus, HUD, écrans (settings, credits, level select) |
| `save.rs` | persistance JSON dans le user data dir |

Les assets binaires (PNG, WAV, OGG) sont **générés** par les binaires
d'exemple, pas édités à la main :

```bash
cargo run --example gen_assets   # sprites -> assets/sprites/
cargo run --example gen_audio    # SFX -> assets/audio/sfx/
cargo run --example gen_music    # musique -> assets/audio/music/
```

Si tu changes le style pixel art ou un pattern audio, **modifie le
générateur** puis regénère, ne remplace pas les fichiers à la main.

## Style et qualité

- `cargo fmt` avant chaque commit.
- `cargo clippy -- -D warnings` doit passer.
- `cargo test` doit passer (CI vérifie).
- Pas de `unwrap()` dans du code hot / gameplay : préférer `if let`,
  `let else`, ou log + fallback.
- Les commentaires expliquent le **pourquoi**, pas le **quoi**. Éviter
  les commentaires qui paraphrasent le code.

## Workflow PR

1. Fork + branche dédiée (`feat/xxx`, `fix/xxx`, `docs/xxx`).
2. Commits atomiques, message impératif court (ex. `feat: add double
   jump`, `fix: coyote timer reset on death`).
3. Push + ouvre une PR contre `main`.
4. Coche la checklist du template PR.
5. La CI doit passer (fmt + clippy + test + build). Corrige avant de
   demander review si elle est rouge.
6. Une review au minimum avant merge. On squash-merge par défaut.

## Rapporter un bug

Utilise l'[issue template Bug report](.github/ISSUE_TEMPLATE/bug_report.yml).
Inclure :
- Version du jeu (commit SHA si dev),
- OS + version,
- Étapes de reproduction,
- Comportement attendu vs observé,
- Logs pertinents (`RUST_LOG=info cargo run` pour verbose).

## Proposer une feature

Ouvre une [Feature request](.github/ISSUE_TEMPLATE/feature_request.yml)
**avant** de coder, surtout pour du gameplay ou de l'architectural.
On peut discuter du fit avec la roadmap avant d'engager du travail.

## Licence des contributions

En soumettant une PR, tu acceptes que ta contribution soit publiée
sous [MIT](LICENSE), la licence du projet.
