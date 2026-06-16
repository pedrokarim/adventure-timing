# Roadmap

Ordre suggéré pour éviter de se perdre. Chaque étape est un livrable jouable.

## Étape 0 — Setup (1 jour)

- [ ] `cargo new`, ajout des dépendances
- [ ] Fenêtre Bevy qui s'ouvre avec un fond coloré
- [ ] Hot-reload `dynamic_linking` confirmé
- [ ] Git initialisé

## Étape 1 — Carré qui bouge (2-3 jours)

- [ ] Un sprite rectangulaire affiché
- [ ] Déplacement gauche/droite au clavier
- [ ] Saut basique avec gravité
- [ ] Sol plat infini en collision

Objectif : sentir que ça répond bien avant d'aller plus loin.

## Étape 2 — Game feel (3-5 jours)

- [ ] Coyote time
- [ ] Jump buffer
- [ ] Saut variable (court tap = petit saut)
- [ ] Accélération / décélération (pas binaire)
- [ ] Caméra qui suit avec smoothing

Étape la plus importante. Un mauvais game feel ici tue le jeu, peu importe le contenu après.

## Étape 3 — Niveaux (3-5 jours)

- [ ] Intégration LDtk
- [ ] Chargement d'un niveau de test
- [ ] Collisions auto sur les tiles solides
- [ ] Spawn du joueur depuis une entité LDtk
- [ ] Téléportation entre niveaux (portes)

## Étape 4 — Animations (2-3 jours)

- [ ] Sprite animé pour le joueur (idle, run, jump, fall)
- [ ] State machine d'animation
- [ ] Flip horizontal selon direction

## Étape 5 — Ennemis et hazards (1 semaine)

- [ ] Ennemi qui patrouille
- [ ] Pics qui tuent au contact
- [ ] Système de vie / mort / respawn
- [ ] Knockback

## Étape 6 — Boucle de gameplay (1 semaine)

- [ ] Menu principal
- [ ] Pause
- [ ] Game over et reset
- [ ] Sauvegarde de la progression (au minimum dernier checkpoint)
- [ ] Écran de fin

## Étape 7 — Polish (continu)

- [ ] Particules (poussière au saut, à l'atterrissage)
- [ ] Screen shake
- [ ] SFX
- [ ] Musique
- [ ] Parallax background
- [ ] Transitions entre niveaux

## Étape 8 — Distribution

- [ ] Build release optimisé
- [ ] Cibles Windows / Linux / macOS
- [ ] Optionnel : WASM pour itch.io
- [ ] Page itch.io ou Steam
