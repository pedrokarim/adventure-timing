# Stack technique

## Dépendances Cargo

```toml
[dependencies]
bevy = { version = "0.14", features = ["dynamic_linking"] }
bevy_rapier2d = "0.27"
bevy_ecs_tilemap = "0.14"
bevy_ecs_ldtk = "0.10"
bevy_kira_audio = "0.20"
bevy-inspector-egui = "0.25"   # debug uniquement
leafwing-input-manager = "0.14"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

> `dynamic_linking` accélère drastiquement les rebuilds en dev. À retirer pour les builds release.

## Rôle de chaque crate

| Crate | Rôle |
|---|---|
| `bevy` | Moteur, ECS, rendu, fenêtre, input |
| `bevy_rapier2d` | Physique 2D, collisions, raycasts |
| `bevy_ecs_tilemap` | Rendu performant de tiles |
| `bevy_ecs_ldtk` | Import direct des niveaux LDtk |
| `bevy_kira_audio` | Audio (musique + SFX, meilleur que l'audio de base) |
| `leafwing-input-manager` | Remap clavier/manette, actions plutôt que touches brutes |
| `bevy-inspector-egui` | Inspecteur runtime des entités (dev only) |

## Outils externes

- **LDtk** ([ldtk.io](https://ldtk.io)) — éditeur de niveaux 2D, gratuit, format JSON propre, intégration Bevy native.
- **Aseprite** — sprites et animations. Crate `bevy_aseprite_ultra` pour charger directement les `.aseprite`.
- **Tiled** — alternative à LDtk si tu préfères, supporté par `bevy_ecs_tilemap`.

## Linter / format

```bash
cargo clippy -- -D warnings
cargo fmt
```

Activer aussi `rust-analyzer` dans l'éditeur.
