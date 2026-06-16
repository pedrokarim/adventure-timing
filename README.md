# Adventure Timing

Side-scroller platformer en Rust.

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
