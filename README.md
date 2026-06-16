# Adventure Timing

Side-scroller platformer en Rust avec Bevy 0.14.

## Jouer

```bash
cargo run --release   # build optimisé, recommandé pour tester le feel
cargo run             # build dev rapide après la première compilation
```

### Contrôles

| Action | Touche |
|---|---|
| Gauche / Droite | `Q D` ou `← →` |
| Saut | `Espace`, `W` ou `↑` |
| Pause | `Échap` |
| Menu (depuis pause/fin) | `Q` |

Le saut est variable : tap court = petit saut, maintenir = saut haut.
Coyote time (100 ms) et jump buffer (120 ms) intégrés.

### Niveau

Un niveau de test traverse 5 sections : zone d'échauffement, escaliers
de plateformes, passage avec pics blancs (mortels), ascension verticale
contre un mur, puis trois piliers étroits avant le drapeau rose.

- Les **drapeaux jaunes** sont des checkpoints (deviennent verts au passage).
- Les **pics blancs** tuent au contact, respawn au dernier checkpoint.
- Tomber sous le niveau = mort.
- Le **drapeau rose** termine le niveau et affiche le temps + le nombre
  de morts.

## Build pour distribution

### Linux (build local optimisé)

```bash
cargo build --release        # binaire dans target/release/adventure_timing
# ou plus rapide pour itérer :
cargo build --profile release-fast
```

Le binaire est statique pour son code, mais a besoin des `assets/` à
côté (chemin relatif `./assets/`). Pour redistribuer : zip le binaire +
le dossier `assets/`.

### Windows / macOS via `cross`

```bash
# Install cross une fois
cargo install cross --git https://github.com/cross-rs/cross
# Build
cross build --release --target x86_64-pc-windows-gnu
cross build --release --target x86_64-apple-darwin
```

### WASM (itch.io / page web)

```bash
rustup target add wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir wasm \
    --out-name "adventure_timing" \
    target/wasm32-unknown-unknown/release/adventure_timing.wasm
# Servir le dossier wasm/ + assets/ avec un script index.html
```

À noter : Bevy en WASM nécessite `bevy = { ..., default-features = false }`
et désactiver `dynamic_linking`. Le bundle WASM final pèse ~20-30 Mo
(Bevy n'est pas tree-shake-friendly).

### Profils Cargo

- `release` : LTO fat + strip + panic=abort. Binaire minimal mais build
  lent (~3-5 min).
- `release-fast` : LTO thin sans strip. Pour itérer sur les builds
  release (~1-2 min).

## Direction artistique

Palette **nuit mystique** inspirée du travail de Camille Unknown
(références dans `assets/inspirations/`) : dominantes teal/bleu nuit,
silhouettes encapuchonnées, accents cyan (cristaux) et ambre (cœur du
goal). Forêt en silhouette, étoiles, mountains.

## Features v1.3

- Menu principal interactif (boutons, navigation clavier + souris)
- Écrans Paramètres (plein écran + 3 volumes) et Crédits
- Sauvegarde persistante (`~/.local/share/adventure_timing/`) : meilleur
  temps, moins de morts, runs complétées
- SFX procéduraux (saut, atterrissage, mort, checkpoint, victoire)
- Support manette via `bevy_gilrs`
- Police DejaVu Sans Mono (gère tous les accents FR)

## Features v1.2

- Contrôleur platformer kinematic complet (coyote, buffer, saut variable)
- Collisions AABB axe par axe contre solides statiques
- États du jeu : menu principal, jeu, pause, game over, victoire
- Checkpoints, hazards, drapeau de fin
- HUD compteur de morts + temps écoulé
- Squash & stretch du joueur (saut, atterrissage, chute)
- Screen shake à l'atterrissage et à la mort
- Particules de poussière au saut et à l'atterrissage
- Caméra qui suit avec lookahead et lerp framerate-indépendant
- **Sprites pixel art procéduraux** (joueur 7 frames animées, tiles
  herbe/terre/bois/pierre, pics, drapeaux) générés par
  `cargo run --example gen_assets`

## Régénérer les assets

Tous les sprites sont générés par un binaire dédié, pas de fichier
binaire commité « à la main ». Pour les modifier ou les régénérer :

```bash
cargo run --example gen_assets
```

Les PNGs atterrissent dans `assets/sprites/`.

## Documentation

- [Choix du moteur](docs/01-choix-moteur.md) — Bevy vs Macroquad vs ggez
- [Stack technique](docs/02-stack.md) — physique, audio, assets, tiles
- [Architecture](docs/03-architecture.md) — organisation du code et ECS
- [Roadmap](docs/04-roadmap.md) — étapes de développement
- [Getting started](docs/05-getting-started.md) — premier prototype

## Décisions rapides

| Question | Réponse par défaut |
|---|---|
| Moteur | Bevy (ou Macroquad si prototype) |
| Physique | `bevy_rapier2d` |
| Tilemap | `bevy_ecs_tilemap` + LDtk via `bevy_ecs_ldtk` |
| Édition niveaux | LDtk |
| Audio | `bevy_kira_audio` |
