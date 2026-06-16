# Roadmap

État à `v1.3.0+` : étapes 0 à 7 (en partie) terminées. Liste ci-dessous
réorganisée : section "État actuel" pour ce qui est livré, puis "Suite"
détaillée pour les chantiers en cours / à venir.

---

## ✅ État actuel — livré

### Étape 0 — Setup
- [x] `cargo init`, dépendances (bevy 0.14, serde, hound, image, directories)
- [x] Fenêtre Bevy, `dynamic_linking`, linker mold
- [x] Git initialisé + tags v1.0.0 → v1.3.0

### Étape 1 — Carré qui bouge
- [x] Sprite affiché, déplacement clavier, gravité, sol plat

### Étape 2 — Game feel
- [x] Coyote time (100 ms), jump buffer (120 ms), saut variable
- [x] Accélération/décélération avec turnaround boost
- [x] **Double saut** avec recharge au sol + ring de particules cyan
- [x] Caméra cinématique (deadzone X/Y, lookahead lissé, anticipation
      verticale en saut/chute)

### Étape 3 — Niveau (hardcodé, pas encore LDtk)
- [x] Niveau de test dessiné en Rust dans `src/world.rs` (5 sections)
- [x] Sol/plateformes/murs avec collisions AABB
- [x] Drapeau de fin
- [ ] **À faire : LDtk pour ne plus recompiler à chaque modif niveau**

### Étape 4 — Animations
- [x] Sprite atlas 7 frames (idle, run×4, jump, fall)
- [x] State machine d'animation selon Velocity + Grounded
- [x] Flip horizontal selon direction
- [x] Squash & stretch procédural

### Étape 5 — Hazards (partiel)
- [x] Pics qui tuent au contact (cristaux cyan)
- [x] Checkpoints + respawn
- [x] Kill floor Y=-800
- [ ] **À faire : ennemis** (voir Étape D ci-dessous)

### Étape 6 — Boucle de jeu
- [x] États MainMenu / Settings / Credits / Playing / Paused / GameOver / Win
- [x] Boutons interactifs (clavier + souris, hover, focus)
- [x] Sauvegarde JSON persistée (best_time, fewest_deaths, runs_completed)
- [x] Paramètres persistés (fullscreen + 3 volumes)

### Étape 7 — Polish
- [x] Particules : poussière au saut/atterrissage, ring au double saut
- [x] Screen shake (atterrissage et mort)
- [x] SFX procéduraux (jump, land, death, checkpoint, win)
- [x] Parallax background (3 couches, rose Camille pour le niveau 1)
- [x] Background dédié pour le main menu (camille-unknown-home)
- [ ] **À faire : musique** (Étape C)
- [ ] **À faire : transitions visuelles entre états** (fade in/out)

---

# 🔜 Suite — détaillé

## Étape A — Niveaux supplémentaires + ambiances variées

**But** : passer de 1 niveau hardcodé à 4-5 niveaux avec ambiances
distinctes inspirées du portfolio de Camille Unknown
(`assets/inspirations/`).

### Ambiances proposées

| # | Nom | Inspiration | Palette | Hazard signature |
|---|---|---|---|---|
| 1 | **Au commencement** | `camille-unknown-home.webp` | Rose/coral sunset | Pics cyan |
| 2 | **Forêt silencieuse** | `world-of-orio.webp` | Teal nuit + arbres bleus | Spores tombantes |
| 3 | **Ruines d'ambre** | `orange0625.webp` | Sépia/ambre cyberpunk | Lasers horizontaux |
| 4 | **Sanctuaire** | `light.webp` | Noir profond + accent rouge | Vignes mortelles |
| 5 | **Aurore** | `castleinthesky.webp` | 1-bit grayscale dithered | Vent qui pousse |

### Architecture cible

```rust
// src/level.rs (ou nouveau src/levels.rs)
#[derive(States, Hash, Eq, PartialEq, Debug, Clone, Default)]
pub enum LevelId {
    #[default]
    PinkSunset,    // niveau 1
    NightForest,
    AmberRuins,
    Sanctuary,
    Dawn,
}

impl LevelId {
    pub fn name(&self) -> &'static str { ... }
    pub fn next(&self) -> Option<LevelId> { ... }
    /// Couleur du ciel pour ClearColor
    pub fn sky(&self) -> Color { ... }
    /// Paths des 3 couches de parallax pour ce niveau
    pub fn parallax(&self) -> [&'static str; 3] { ... }
    /// Paths des tiles (ground, grass, platform, wall)
    pub fn tiles(&self) -> [&'static str; 4] { ... }
}
```

### Tâches détaillées

- [ ] Refactor `world.rs` pour prendre un `LevelId` en paramètre
- [ ] Extraire la géométrie de chaque niveau dans des fonctions séparées
      (`spawn_level_1`, `spawn_level_2`, ...) ou dans des fichiers
      `assets/levels/*.ron` pour pas recompiler
- [ ] Ajouter `CurrentLevel(LevelId)` comme Resource
- [ ] Sky color appliqué dynamiquement OnEnter(Playing) selon CurrentLevel
- [ ] Parallax recharge ses textures OnEnter(Playing) selon CurrentLevel
- [ ] Goal du niveau N déclenche transition vers niveau N+1 (au lieu
      d'aller directement à Win)
- [ ] Écran "Niveau X terminé !" entre les niveaux (mini fade)
- [ ] Mettre à jour `world::TOTAL_LEVELS` à la valeur réelle
- [ ] Génération assets : 4 jeux de tiles + 4 jeux de parallax dans
      `examples/gen_assets.rs` (modulariser par palette)
- [ ] **Plus tard : LDtk** pour éditer les niveaux à la souris
      (`bevy_ecs_ldtk` crate). On bascule quand on a 3+ niveaux et que
      modifier en Rust devient lourd.

### Estimation : 1-2 semaines (gros)

---

## Étape B — Police "vrai jeu"

**But** : remplacer la police DejaVu (utilitaire) par une police plus
"identité gaming" pour les titres et un fallback lisible pour le corps
de texte.

### Recommandations

| Police | Style | Bon pour |
|---|---|---|
| **Press Start 2P** | Pixel 8x8, NES-like | Titres, menus pixel-art |
| **VT323** | Terminal vintage | Compteurs, HUD |
| **Pixelify Sans** | Pixel-art moderne, lisible | Titres + UI tout-en-un ⭐ |
| **Daydream** | Pixel-art handwritten | Sous-titres mystiques |
| **m5x7** | 5×7 pixel | HUD ultra compact |
| **Acme** | Sans grotesque cute | Lisible pour gros textes |

**Recommandation principale** : `Pixelify Sans` (Google Fonts, OFL) pour
les titres et boutons, garder `DejaVu Sans Mono` pour le HUD numérique
(meilleure lisibilité des chiffres). Tester `Press Start 2P` en
alternative si on veut un côté "rétro arcade".

### Tâches détaillées

- [ ] Télécharger les fichiers TTF/OTF (Google Fonts → download family)
- [ ] Les déposer dans `assets/fonts/` (`PixelifySans-Regular.ttf`,
      `PixelifySans-Bold.ttf`)
- [ ] Ajouter dans la `Resource UiFont` un champ `display: Handle<Font>`
      pour la police de titres
- [ ] Utiliser `font.display` dans `spawn_title()`, `spawn_button*()`
- [ ] Garder `font.regular` pour le HUD et les textes longs
- [ ] Tester taille : police pixel-art a souvent besoin de tailles
      multiples de la grille (8, 16, 24, 32 pour Press Start 2P)
- [ ] Vérifier que les accents FR sont supportés (sinon fallback DejaVu)

### Estimation : 1 jour

---

## Étape C — Items collectibles + effets

**But** : ajouter des items à ramasser sur les niveaux pour donner des
buffs temporaires ou des points. Inspiré de Hollow Knight (charms),
Celeste (fraises bonus).

### Items proposés

| Item | Effet | Durée | Visuel |
|---|---|---|---|
| 💎 **Cristal cyan** | +1 double saut (recharge en l'air) | instantané | Petit losange cyan brillant |
| 🟠 **Pétale d'ambre** | Invincibilité (pics ne tuent pas) | 3 s | Pétale 8x8 ambre pulsant |
| 🪶 **Plume blanche** | Saut +30 % hauteur | 5 s | Plume stylisée, halo blanc |
| ⏳ **Sablier** | Slowmo : monde ralenti à 50 % | 4 s | Sablier rotatif rose |
| 🌸 **Pétale mémoire** | Le prochain mort ne compte pas | jusqu'à utilisation | Pétale violet flottant |
| ⭐ **Étoile bonus** | +1 au compteur "items collectés" | — | Étoile 6 px qui tourne |

### Architecture

```
src/items.rs                   ← nouveau module
├── ItemKind enum
├── Item Component
├── ActiveEffect Resource (liste timed effects)
├── PickupEvent
├── ItemsPlugin
│   ├── check_pickup_overlap   (système AABB joueur ↔ items)
│   ├── apply_effect_on_pickup (déclenche le buff)
│   ├── tick_active_effects    (décrémente les timers)
│   └── animate_items          (rotation, halo, bobbing)
└── spawn_item helpers
```

### Détails de wiring

- **Cristal cyan** : modifie `PlayerController.air_jumps_remaining`
  directement
- **Pétale d'ambre** : nouveau `Resource Invincible(f32)`, consulté par
  `level::check_spike_collision` avant d'émettre `PlayerDied`
- **Plume blanche** : multiplicateur sur `JUMP_VELOCITY` pendant la durée
- **Sablier** : `Resource TimeScale(f32)`, le module `physics` et le
  `Time` côté Bevy l'utilisent (utile aussi pour les ennemis plus tard)
- **Pétale mémoire** : nouveau `Resource SkipNextDeath(bool)`

### Tâches

- [ ] `src/items.rs` avec tout ci-dessus
- [ ] HUD : icônes des effets actifs en haut à droite (timer barre)
- [ ] Spawn des items dans `world.rs` (positions hardcodées dans le
      niveau de test pour valider, ensuite LDtk)
- [ ] Sprites 16×16 dans `examples/gen_assets.rs` (`item_crystal.png`,
      `item_petal.png`, etc.)
- [ ] SFX procédural `pickup.wav` dans `examples/gen_audio.rs` (bell
      cristallin court, ~150 ms)
- [ ] Particules pickup (burst de quelques px à la position de l'item)

### Estimation : 3-5 jours

---

## Étape C' — Items projetables / posables

**But** : au-delà des collectibles passifs (C), des items que le joueur
peut **lancer** (projectile) ou **poser** dans le monde pour interagir
avec ennemis et environnement.

### Items proposés

| Item | Action | Effet | Comportement |
|---|---|---|---|
| 💣 **Bombe** | Lancer (touche X) | Explose à l'impact ou après 2 s : tue ennemis dans rayon 60 px | Arc parabolique sous gravité |
| 🪨 **Caillou** | Lancer | Dégât léger 1 hp, knockback | Trajectoire droite, gravité légère |
| 🌟 **Étoile boomerang** | Lancer | Va + revient, traverse les ennemis | Trajectoire arc retour |
| 🧊 **Glace** | Poser | Crée un bloc solide 32×32 temporaire (~6 s) → plateforme improvisée | Fixe, despawn auto |
| 🔥 **Torche** | Poser | Brûle pendant ~8 s, éclaire zone sombre, brûle ennemis qui passent | Statique au sol |
| ⚙️ **Tourelle** | Poser | Tire 1 projectile / 1.5 s sur ennemi le plus proche | Statique, ~5 s de vie |
| 🛡️ **Bouclier** | Poser | Mur invisible bloque projectiles ennemis (~4 s) | Vertical 16×64 |
| 🪤 **Piège** | Poser | Au passage ennemi : 1 hp + ralentit 1 s | Carré au sol invisible |
| 🌐 **Plateforme magique** | Poser sous soi | Plateforme cyan 48×8 qui dure 4 s | Solid temporaire |
| 🗺️ **Marqueur** | Poser | Petit drapeau, sert de point de téléport (1 utilisation) | Cosmétique + waypoint |

### Limites pour éviter l'abus

- **Inventaire** : 3 slots, on tient 3 items max, ramassés sur le niveau
- **Cooldown** : 0.5 s entre deux lancers
- **Items posés** : max 2 actifs à la fois (despawn le plus vieux si on
  en pose un 3ᵉ)

### Architecture

```rust
// src/throwables.rs (nouveau module séparé des passifs)
#[derive(Component)]
pub struct Throwable { pub kind: ThrowableKind }

#[derive(Clone, Copy)]
pub enum ThrowableKind {
    Bomb, Rock, BoomerangStar,
    IcePlatform, Torch, Turret,
    Shield, Trap, MagicPlatform, Marker,
}

#[derive(Component)]
pub struct InWorld { pub ttl: f32 }  // posés ont une durée de vie

#[derive(Resource, Default)]
pub struct Inventory {
    pub slots: [Option<ThrowableKind>; 3],
    pub selected: usize,  // 0..3
}

// Events
struct ThrowItem;       // lance l'item sélectionné
struct PlaceItem;       // pose l'item sélectionné
struct CycleSelection;  // tab pour changer de slot
```

### Contrôles proposés

| Action | Touche |
|---|---|
| Lancer item (devant) | `X` ou `J` |
| Poser item (sous le perso) | `C` ou `K` |
| Cycler le slot inventaire | `Tab` ou `Shift` |
| Détonner manuellement (bombe à retardement) | `B` |

Pour manette : `RB` lancer, `LB` poser, `LT` cycler.

### Tâches détaillées

- [ ] `src/throwables.rs` : composants, events, inventory
- [ ] HUD inventaire en haut à droite : 3 slots horizontaux + slot
      sélectionné highlighted
- [ ] Système de physique projectile : Velocity + gravité + collision
      avec Solid (rebond ou despawn selon kind)
- [ ] Système de placement : checks sol sous le joueur, place l'entité
- [ ] Animation lancer : petit windup du sprite joueur (2 frames extra
      dans `player.png`, frames 7 et 8)
- [ ] Sprites d'items + leur version "posée" / "explosion" / "etc"
- [ ] SFX procéduraux : `throw.wav` (whoosh), `bomb_explode.wav`
      (impact + boom), `ice_freeze.wav` (cracking), `place.wav` (clack)
- [ ] Interaction avec ennemis : `EnemyHit { damage, knockback }` event
      consommé par le système ennemis (Étape G)
- [ ] Multi-effets : bombe explose → écrase ennemis ET détruit blocs
      destructibles potentiels (futur)

### Estimation : 1-2 semaines

---

## Étape C'' — Armes (combat actif)

**But** : permettre au joueur d'attaquer les ennemis au corps à corps
et/ou à distance, avec une arme persistante (pas consommable comme les
throwables).

### Types d'armes

| Arme | Range | Vitesse | Particularité |
|---|---|---|---|
| 🗡️ **Dague** | Court (30 px) | Très rapide | Attaque combo 3 coups, dégât 1 |
| ⚔️ **Épée** | Moyen (45 px) | Moyenne | Coup chargé (maintenir) → dégât x2 |
| 🏹 **Arc** | Long (infini) | Lent | Coût : flèches limitées (5 par checkpoint) |
| 🔮 **Bâton magique** | Long (300 px) | Moyen | Projectile énergie, coût mana 1/tir, mana recharge en marchant |
| 🔨 **Marteau** | Très court (25 px) | Lent | Dégât x3, knockback ennemis x4, peut casser blocs |
| 🪃 **Boomerang** | Long (240 px) | Moyen | Reviens vers le joueur, peut traverser ennemis |

### Mécanique de combat

- **Touche d'attaque** : `Espace` ou bouton dédié (à séparer du saut si
  on garde Espace pour saut). Suggestion : **`F`** ou clic gauche souris
- **Direction** : par défaut dans le sens du `Player.facing`. Optionnel :
  attaque verticale (en bas en sautant comme Hollow Knight = pogostick)
- **Hitbox d'attaque** : rectangle 16×24 devant le joueur, durée ~120 ms,
  dégât appliqué aux ennemis qui overlap pendant cette fenêtre
- **Cooldown** : varie selon arme (dague 200 ms, marteau 600 ms)
- **Combo** : pour la dague, 3 coups en chaîne si presse Attaque dans
  les 400 ms après le précédent → dégât escalant 1/1/2

### Stats & progression

- L'arme courante est une `Resource CurrentWeapon(WeaponKind)`
- Switch d'arme via un menu pause/inventaire ou autre touche (ex: Q)
- À terme : système de progression / upgrade (charms qui boostent
  l'arme courante)

### Architecture

```rust
// src/weapons.rs
#[derive(Resource, Clone, Copy)]
pub enum WeaponKind {
    Dagger, Sword, Bow,
    MagicStaff, Hammer, Boomerang,
}

#[derive(Component)]
struct AttackHitbox {
    pub size: Vec2,
    pub damage: u32,
    pub knockback: Vec2,
    pub remaining: f32,
}

#[derive(Component)]
struct Projectile {
    pub damage: u32,
    pub returning: bool,  // pour le boomerang
}

#[derive(Resource)]
struct WeaponState {
    cooldown_remaining: f32,
    combo_step: u8,
    combo_window: f32,
    arrows: u32,    // pour l'arc
    mana: f32,      // pour le bâton
}

// Events
struct WeaponSwung;
struct WeaponHitEnemy { entity: Entity, damage: u32 }
```

### Pogostick (attaque vers le bas en sautant)

Inspiré directement de Hollow Knight :
- Si le joueur attaque alors qu'il est en l'air ET maintient `S` ou ↓ :
  hitbox sous le perso au lieu de devant
- Si la hitbox touche un ennemi : rebond `velocity.y = 500` + relance le
  double saut → permet le platforming sur ennemis répétés
- Ouvre une mécanique d'enchaînement très satisfaisante

### Tâches

- [ ] `src/weapons.rs` avec WeaponKind enum + état
- [ ] Input : touche d'attaque mappable (placeholder : `F`)
- [ ] Animation joueur : 2 frames d'attaque (windup + swing) à ajouter
      à `player.png` (frames 7, 8) ou créer un sprite atlas séparé
- [ ] Spawn de la `AttackHitbox` Entity au moment du swing, despawn
      après `remaining = 0`
- [ ] Système `apply_attack_damage` : check overlap hitbox/ennemi,
      émet `WeaponHitEnemy`, despawn hitbox
- [ ] Projectiles (arc, bâton) : Component dédié + system de mouvement
- [ ] Boomerang : phase aller puis retour vers le joueur
- [ ] Pogostick (peut être livré en v1)
- [ ] SFX : un par arme (`dagger_swing`, `bow_shoot`, `hammer_hit`, …)
- [ ] Particules d'impact (étoiles, fumée, étincelles selon arme)
- [ ] HUD : icône arme courante en haut à droite + munitions/mana si
      applicable

### Estimation : 1-2 semaines

---

## Étape D — Musique d'ambiance

**But** : une boucle musicale par niveau, générée procéduralement pour
ne pas avoir de samples externes à licencier.

### Approche procédurale

- `examples/gen_music.rs` (à créer, dev-dep `hound` déjà présente)
- WAV mono 44.1 kHz, ~30-60 s par boucle
- Synthèse simple :
  - Basse drone (sine + slight detune) pour le pad
  - Mélodie pentatonique ou modale en mineur
  - Pad d'accords arpégés
  - Optionnel : bruit de vent / pluie subtil en sous-couche
- Tonalité par niveau (matching ambiance) :
  - Niveau 1 (rose) : F# mineur — nostalgique mais doux
  - Niveau 2 (teal forêt) : D mineur — sombre, mystique
  - Niveau 3 (ambre) : C mineur — industriel, oppressant
  - Niveau 4 (sanctuaire) : B mineur — solennel, lent
  - Niveau 5 (aurore) : A majeur — résolution, lumineux

### Architecture

```
src/audio.rs (étendre)
├── struct MusicTracks { level_1, level_2, level_3, level_4, level_5 }
├── play_music_for_level system (OnEnter(Playing))
├── stop_music (OnExit(Playing))
└── crossfade entre tracks au changement de niveau
```

### Tâches

- [ ] `examples/gen_music.rs` : helpers pour générer arpèges, drones,
      mélodies pentatoniques. Compose 5 boucles.
- [ ] Sauver dans `assets/music/`
- [ ] Étendre `AudioPlugin` :
  - Charger toutes les boucles au plugin build
  - `PlaybackSettings::LOOP` au lieu de `DESPAWN`
  - Sink dédié musique (séparé des SFX pour pouvoir cut/fade)
  - Volume = `settings.master * settings.music`
- [ ] Crossfade : système qui décroît le volume de l'ancienne piste
      sur 1 s pendant que la nouvelle monte
- [ ] Tester sur le niveau 1 d'abord, puis multiplier

### Notes
- Pour qualité musicale au-delà du procédural, alternatives à
  considérer plus tard :
  - **Beepbox** (web) — export WAV gratuit, son chiptune
  - **LMMS** ou **Bosca Ceoil** pour composer à la main
  - Crate `fundsp` pour synthèse plus expressive en Rust
- La piste générée procéduralement sera correcte mais pas marquante.
  À itérer.

### Estimation : 3-7 jours (selon ambition musicale)

---

## Étape E — Ennemis

**But** : peupler les niveaux avec des entités hostiles, donner un
gameplay actif au-delà des hazards statiques.

### Types proposés

| Ennemi | Comportement | Apparence | Touche |
|---|---|---|---|
| **Crawler** | Patrouille sur une plateforme (gauche/droite, demi-tour sur bord) | Petit insecte cyan 16x12 | Stompable ⬇️ |
| **Flyer** | Sinusoïde verticale, suit le joueur en X si dans range | Œil flottant 16x16 | Stompable ⬇️ |
| **Spitter** | Statique, tire projectile à intervalles (~2 s) | Buisson 24x24 | Stompable ⬇️ |
| **Charger** | Patrouille, charge sur le joueur si <100 px | Boule cornue 20x20 | Stompable, mais charge fait reculer le joueur |
| **Wraith** | Phase à travers les murs, suit lentement | Silhouette 16x24 | Insensible, à fuir |

### Système de stomp

Inspiré de Mario/Hollow Knight : sauter SUR la tête d'un ennemi le tue
ET relance le double saut du joueur (= traversée verticale). Le contact
latéral ou par-dessous tue le joueur (= comme un hazard).

### Architecture

```
src/enemies.rs                  ← nouveau module
├── EnemyKind enum
├── Enemy Component
├── EnemyAI states (Patrol, Chase, Attack, Hurt, Dead)
├── Health Component (pour ennemis multi-hit éventuels)
├── EnemyHitEvent (joueur sur ennemi)
├── EnemiesPlugin
│   ├── enemy_ai_system        (logique IA)
│   ├── enemy_animation        (anim selon AI state)
│   ├── check_stomp            (collision verticale par le haut)
│   ├── check_side_damage      (collision latérale = mort joueur)
│   └── spawn_enemy_particles  (poof à la mort)
```

### Wiring

- **Stomp** : si Player.velocity.y < -100 ET collision avec haut de la
  hitbox ennemi → ennemi prend dégât (ou meurt), Player rebondit
  (`velocity.y = 400`), `air_jumps_remaining` rechargé
- **Damage joueur** : tout autre contact → `PlayerDied` event
- **Patrol** : Crawler check les bords de plateforme via raycast 8 px
  devant ses pieds, demi-tour sinon
- **Projectiles** : nouveau Component `Projectile` avec velocity + TTL,
  collisions joueur = mort, collision sol = despawn

### Tâches

- [ ] `src/enemies.rs` : framework AI + EnemyKind enum
- [ ] Implémenter Crawler en premier (le plus simple)
- [ ] Ajouter Flyer (+ logique sin wave)
- [ ] Spitter + Projectile
- [ ] Charger (un peu plus complexe : trigger zone + ramp-up)
- [ ] Wraith (peut traverser les murs : retirer Solid des collisions
      pour cette entité)
- [ ] Sprites animés dans `gen_assets` : 2-4 frames par ennemi
- [ ] SFX : `enemy_hurt.wav`, `enemy_die.wav`, `projectile.wav`
- [ ] Spawn dans niveau de test pour valider, puis distribuer dans les
      autres niveaux

### Estimation : 1-2 semaines

---

## Étape F — Distribution

- [ ] Build release optimisé (`lto = "fat"`, strip)
- [ ] Cibles Windows / Linux / macOS via `cross` ou GitHub Actions
- [ ] WASM pour itch.io (`cargo run --target wasm32-unknown-unknown`,
      `wasm-bindgen`, `trunk`)
- [ ] Page itch.io avec screenshots, GIFs gameplay
- [ ] Bandeau crédit Camille Unknown (avec autorisation idéalement)

---

## Ordre suggéré

Le développement va devenir plus difficile avec plus de niveaux à
itérer. Recommandation pour l'enchaînement :

1. **B (police)** — quick win visuel, 1 jour
2. **C (items passifs)** — donne du gameplay au niveau 1 existant,
   validable immédiatement, framework simple
3. **C' (items projetables/posables)** — ajoute la verbe "lancer/poser",
   prépare le terrain pour le combat
4. **C'' (armes)** — combat actif. Cohabite avec C' (les items
   throwables sont à part des armes)
5. **E (ennemis)** — cibles pour tester C', C''. Beaucoup de variété
   potentielle, à étaler sur du temps
6. **A (niveaux multi)** — refactor structurel, plus riche maintenant
   qu'on a items + armes + ennemis à répartir
7. **D (musique)** — fait passer le jeu de "demo" à "produit", à faire
   quand le contenu visuel/gameplay est mature
8. **F (distribution)** — quand tout vaut le coup d'être joué

Si on doit accélérer une démo : faire B + C'' (1 arme) + 1 ennemi +
D (1 boucle) pour avoir une démo nerveuse.

Si on veut prioriser exploration/contemplation : faire B + C (items
passifs) + A (3 niveaux) + D (musique).

## Inventaire des chantiers ouverts

Récap rapide des modules à créer (vide pour l'instant) :

- `src/items.rs` — items collectibles passifs (Étape C)
- `src/throwables.rs` — items projetables/posables + inventaire (C')
- `src/weapons.rs` — armes + combat actif (C'')
- `src/enemies.rs` — IA ennemis + stomp + projectiles ennemis (E)
- `src/music.rs` — gestion des boucles musicales (D, peut rester dans
  `audio.rs` si pas trop gros)
- `src/levels.rs` ou refactor `world.rs` — multi-niveau (A)
- `examples/gen_music.rs` — générateur de boucles WAV (D)

Et côté assets à générer :
- 4 jeux de tiles (1 par ambiance, A)
- 4 jeux de parallax (1 par ambiance, A)
- Sprites items (~10 fichiers 16×16, C + C')
- Sprites ennemis (5 types avec animations, E)
- Frames d'attaque joueur (player.png passe de 7 à 10 frames, C'')
- Boucles musicales 30-60 s (5 fichiers WAV, D)
