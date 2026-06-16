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

## Features v1

- Contrôleur platformer kinematic complet (coyote, buffer, saut variable)
- Collisions AABB axe par axe contre solides statiques
- États du jeu : menu principal, jeu, pause, game over, victoire
- Checkpoints, hazards, drapeau de fin
- HUD compteur de morts + temps écoulé
- Squash & stretch du joueur (saut, atterrissage, chute)
- Screen shake à l'atterrissage et à la mort
- Particules de poussière au saut et à l'atterrissage
- Caméra qui suit avec lookahead et lerp framerate-indépendant

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
