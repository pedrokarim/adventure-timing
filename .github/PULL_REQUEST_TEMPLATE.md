<!--
Merci pour ta PR ! Remplis les sections applicables et coche la checklist.
-->

## Résumé

<!-- 1-3 lignes : quoi + pourquoi -->

## Type de changement

- [ ] Bug fix (non-breaking, corrige un comportement cassé)
- [ ] Feature (non-breaking, ajoute une capacité)
- [ ] Breaking change (change une API/save/config existante)
- [ ] Docs uniquement
- [ ] Refactor / cleanup (aucun changement de comportement)
- [ ] Assets (sprites, audio, musique)

## Comment tester

<!-- Étapes concrètes pour valider le changement, en local -->

1. …
2. …

## Checklist

- [ ] `cargo fmt --check` passe
- [ ] `cargo clippy -- -D warnings` passe
- [ ] `cargo test` passe
- [ ] `cargo build --release` réussit
- [ ] Le gameplay a été testé manuellement si le changement est jouable
- [ ] Si assets modifiés : les générateurs (`gen_assets` / `gen_audio` / `gen_music`) ont été mis à jour, pas juste les fichiers
- [ ] Documentation à jour (README / docs/)

## Screenshots / clips (si UI/gameplay)

<!-- Un GIF vaut mille mots -->

## Issues liées

<!-- Closes #123 -->
