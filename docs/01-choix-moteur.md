# Choix du moteur

## Comparaison

| Critère | Bevy | Macroquad | ggez |
|---|---|---|---|
| Paradigme | ECS | Immédiat | Immédiat / scène |
| Courbe d'apprentissage | Forte | Faible | Moyenne |
| Hot reload | Oui (assets) | Non | Non |
| Web (WASM) | Oui | Oui | Limité |
| Écosystème | Très actif | Actif | Plus calme |
| Physique 2D | `bevy_rapier2d` | À intégrer manuellement | `rapier2d` direct |
| Tilemap | `bevy_ecs_tilemap`, `bevy_ecs_ldtk` | À faire à la main | À faire à la main |
| Adapté aux gros projets | Oui | Moyen | Moyen |
| Adapté aux prototypes | Moyen | Excellent | Bon |

## Recommandation

- **Bevy** si tu vises un vrai jeu avec contenu, éditeur de niveaux, scènes multiples, et que tu veux profiter d'un écosystème de crates spécifiques au gamedev.
- **Macroquad** si c'est un premier projet ou un prototype, ou si tu veux comprendre les mécaniques de base sans la complexité ECS.
- **ggez** est un milieu de terrain mais avec un écosystème plus restreint, je ne le recommande pas en premier choix aujourd'hui.

## Décision retenue : Bevy

Raisons :
- ECS scale naturellement avec le nombre d'entités (ennemis, projectiles, particules)
- Intégration LDtk + Rapier mature
- Communauté très active en 2026, beaucoup de ressources

Tradeoff accepté : courbe d'apprentissage plus raide les premières semaines.

## Version

Cibler Bevy 0.14+ (API stabilisée sur les Required Components et les Observers).
