//! Items que le joueur peut lancer (bombe) ou poser (glace, plateforme
//! magique). Inventaire 3 slots, cycle au Tab, action au X.
//!
//! Touches :
//! - `Tab` : cycle la sélection
//! - `X` ou `J` : utiliser l'item sélectionné (lancer ou poser selon kind)

use crate::physics::{Collider, Solid, Velocity};
use crate::player::Player;
use crate::states::GameState;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrowableKind {
    Bomb,
    IceBlock,
    MagicPlatform,
    Rock,
    Torch,
    /// Lance, revient au joueur, traverse les ennemis (dégâts en passant).
    Boomerang,
    /// Mur invisible bloque les projectiles ennemis pendant 4 s.
    Shield,
    /// Piège posé au sol, blesse les ennemis qui marchent dessus 6 s.
    Trap,
    /// Drapeau cosmétique pour marquer un point (no-op gameplay).
    Marker,
    /// Tourelle posée, tire un mini projectile toutes les 1.5 s pendant 5 s.
    Turret,
}

impl ThrowableKind {
    pub fn label(self) -> &'static str {
        match self {
            ThrowableKind::Bomb => "Bombe",
            ThrowableKind::IceBlock => "Glace",
            ThrowableKind::MagicPlatform => "Plateforme",
            ThrowableKind::Rock => "Caillou",
            ThrowableKind::Torch => "Torche",
            ThrowableKind::Boomerang => "Boomerang",
            ThrowableKind::Shield => "Bouclier",
            ThrowableKind::Trap => "Piege",
            ThrowableKind::Marker => "Marqueur",
            ThrowableKind::Turret => "Tourelle",
        }
    }

    fn texture(self) -> &'static str {
        match self {
            ThrowableKind::Bomb => "sprites/throwable_bomb.png",
            ThrowableKind::IceBlock => "sprites/throwable_ice.png",
            ThrowableKind::MagicPlatform => "sprites/throwable_platform.png",
            ThrowableKind::Rock => "sprites/throwable_rock.png",
            ThrowableKind::Torch => "sprites/throwable_torch.png",
            ThrowableKind::Boomerang => "sprites/throwable_boomerang.png",
            ThrowableKind::Shield => "sprites/throwable_shield.png",
            ThrowableKind::Trap => "sprites/throwable_trap.png",
            ThrowableKind::Marker => "sprites/throwable_marker.png",
            ThrowableKind::Turret => "sprites/throwable_turret.png",
        }
    }
}

/// Inventaire fixe pré-rempli avec un exemplaire de chaque pour qu'on
/// puisse tester direct. À terme, les slots se remplissent via pickups
/// sur le niveau (TBD).
#[derive(Resource)]
pub struct Inventory {
    pub slots: [Option<ThrowableKind>; 3],
    pub selected: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [
                Some(ThrowableKind::Bomb),
                Some(ThrowableKind::Rock),
                Some(ThrowableKind::Torch),
            ],
            selected: 0,
        }
    }
}

/// Composant attaché aux bombes en vol.
#[derive(Component)]
pub struct Bomb {
    pub fuse: f32,
}

/// Composant attaché aux items posés (glace, plateforme magique).
#[derive(Component)]
pub struct Placed {
    pub ttl: f32,
}

/// Boomerang : marker pour le système de retour.
#[derive(Component)]
pub struct BoomerangThrow {
    pub age: f32,
}

/// Tourelle posée : tire toutes les fire_cooldown s.
#[derive(Component)]
pub struct TurretPlaced {
    pub fire_cooldown: f32,
    pub ttl: f32,
}

#[derive(Event)]
pub struct UseSelectedItem;

pub struct ThrowablesPlugin;

impl Plugin for ThrowablesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_event::<UseSelectedItem>()
            .add_systems(
                Update,
                (
                    handle_input,
                    process_use,
                    tick_bombs,
                    tick_placed,
                    tick_boomerangs,
                    tick_turrets,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut use_ev: EventWriter<UseSelectedItem>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        // Cycle vers le prochain slot non-vide, sinon avance d'un cran.
        for step in 1..=3 {
            let next = (inventory.selected + step) % 3;
            if inventory.slots[next].is_some() || step == 3 {
                inventory.selected = next;
                break;
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyX) || keys.just_pressed(KeyCode::KeyJ) {
        use_ev.send(UseSelectedItem);
    }
}

fn process_use(
    mut events: EventReader<UseSelectedItem>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    asset_server: Res<AssetServer>,
    player: Query<(&Transform, &Player)>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    let Ok((player_t, p)) = player.get_single() else {
        return;
    };
    let dir = if p.facing > 0.0 { 1.0 } else { -1.0 };
    let pos = player_t.translation.truncate();

    let Some(kind) = inventory.slots[inventory.selected] else {
        return;
    };

    match kind {
        ThrowableKind::Bomb => {
            spawn_bomb(
                &mut commands,
                &asset_server,
                pos + Vec2::new(20.0 * dir, 6.0),
                dir,
            );
        }
        ThrowableKind::IceBlock => {
            spawn_placed(
                &mut commands,
                &asset_server,
                ThrowableKind::IceBlock,
                pos + Vec2::new(0.0, -22.0),
                Vec2::new(32.0, 16.0),
                6.0,
            );
        }
        ThrowableKind::MagicPlatform => {
            spawn_placed(
                &mut commands,
                &asset_server,
                ThrowableKind::MagicPlatform,
                pos + Vec2::new(40.0 * dir, -10.0),
                Vec2::new(48.0, 6.0),
                4.0,
            );
        }
        ThrowableKind::Rock => {
            spawn_rock(
                &mut commands,
                &asset_server,
                pos + Vec2::new(16.0 * dir, 4.0),
                dir,
            );
        }
        ThrowableKind::Torch => {
            spawn_torch(&mut commands, &asset_server, pos + Vec2::new(0.0, -18.0));
        }
        ThrowableKind::Boomerang => {
            spawn_boomerang(
                &mut commands,
                &asset_server,
                pos + Vec2::new(16.0 * dir, 0.0),
                dir,
            );
        }
        ThrowableKind::Shield => {
            spawn_shield(
                &mut commands,
                &asset_server,
                pos + Vec2::new(40.0 * dir, 0.0),
            );
        }
        ThrowableKind::Trap => {
            spawn_trap(&mut commands, &asset_server, pos + Vec2::new(0.0, -22.0));
        }
        ThrowableKind::Marker => {
            spawn_marker(&mut commands, &asset_server, pos + Vec2::new(0.0, -10.0));
        }
        ThrowableKind::Turret => {
            spawn_turret(&mut commands, &asset_server, pos + Vec2::new(0.0, -12.0));
        }
    }

    // Consomme l'item du slot (pas refill automatique pour l'instant)
    let slot = inventory.selected;
    inventory.slots[slot] = None;
}

fn spawn_bomb(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, dir: f32) {
    let size = Vec2::splat(12.0);
    commands.spawn((
        Bomb { fuse: 2.0 },
        Velocity(Vec2::new(360.0 * dir, 360.0)),
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Bomb.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        },
    ));
}

fn spawn_rock(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, dir: f32) {
    let size = Vec2::splat(10.0);
    commands.spawn((
        // Le caillou utilise le système des projectiles d'armes : c'est
        // un Projectile (weapons.rs) avec faible dégâts qui touche les
        // ennemis. Plus simple que d'avoir un autre type de hitbox.
        crate::weapons::Projectile {
            damage: 1,
            remaining: 0.8,
        },
        Velocity(Vec2::new(320.0 * dir, 200.0)),
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load("sprites/throwable_rock.png"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        },
    ));
}

fn spawn_torch(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(14.0, 22.0);
    // La torche utilise une AttackHitbox persistante : tout ennemi qui
    // touche prend des dégâts. Le système tick_attack_hitboxes gère le
    // despawn quand `remaining` arrive à 0 (8 s ici).
    commands.spawn((
        crate::weapons::AttackHitbox {
            damage: 1,
            knockback: Vec2::new(0.0, 200.0),
            remaining: 8.0,
            is_pogo: false,
        },
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load("sprites/throwable_torch.png"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        },
    ));
}

fn spawn_placed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: ThrowableKind,
    pos: Vec2,
    size: Vec2,
    ttl: f32,
) {
    commands.spawn((
        Placed { ttl },
        Solid,
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(kind.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.5)),
            ..default()
        },
    ));
}

fn spawn_boomerang(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, dir: f32) {
    let size = Vec2::splat(14.0);
    commands.spawn((
        BoomerangThrow { age: 0.0 },
        crate::weapons::Projectile {
            damage: 1,
            remaining: 2.0,
        },
        Velocity(Vec2::new(360.0 * dir, 80.0)),
        crate::physics::NoGravity,
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Boomerang.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        },
    ));
}

fn spawn_shield(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(8.0, 64.0);
    commands.spawn((
        Placed { ttl: 4.0 },
        Solid,
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Shield.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        },
    ));
}

fn spawn_trap(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(28.0, 8.0);
    commands.spawn((
        crate::weapons::AttackHitbox {
            damage: 1,
            knockback: Vec2::new(0.0, 240.0),
            remaining: 6.0,
            is_pogo: false,
        },
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Trap.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.6)),
            ..default()
        },
    ));
}

fn spawn_marker(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(16.0, 24.0);
    commands.spawn((
        Placed { ttl: 60.0 },
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Marker.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.4)),
            ..default()
        },
    ));
}

fn spawn_turret(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2) {
    let size = Vec2::new(18.0, 22.0);
    commands.spawn((
        TurretPlaced {
            fire_cooldown: 0.0,
            ttl: 5.0,
        },
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load(ThrowableKind::Turret.texture()),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.7)),
            ..default()
        },
    ));
}

fn tick_boomerangs(
    time: Res<Time>,
    mut q: Query<(&mut BoomerangThrow, &mut Velocity, &Transform), Without<crate::player::Player>>,
    player: Query<&Transform, With<crate::player::Player>>,
) {
    let dt = time.delta_seconds();
    let Ok(player_t) = player.get_single() else {
        return;
    };
    for (mut b, mut vel, t) in &mut q {
        b.age += dt;
        if b.age > 0.5 {
            // Phase de retour : interpole vers le joueur
            let target = player_t.translation.truncate();
            let to_player = (target - t.translation.truncate()).normalize_or_zero();
            vel.0 = vel.0 * 0.85 + to_player * 360.0 * 0.15;
        }
    }
}

fn tick_turrets(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut turrets: Query<(Entity, &mut TurretPlaced, &Transform)>,
    enemies: Query<&Transform, (With<crate::enemies::Enemy>, Without<TurretPlaced>)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut turret, t) in &mut turrets {
        turret.ttl -= dt;
        turret.fire_cooldown -= dt;
        if turret.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if turret.fire_cooldown <= 0.0 {
            // Trouve l'ennemi le plus proche
            let mut nearest_dist = f32::MAX;
            let mut target_dir = Vec2::X;
            for e_t in &enemies {
                let d = e_t.translation.distance(t.translation);
                if d < nearest_dist && d < 250.0 {
                    nearest_dist = d;
                    target_dir =
                        (e_t.translation.truncate() - t.translation.truncate()).normalize_or_zero();
                }
            }
            if nearest_dist < f32::MAX {
                // Tire un projectile
                let pos = t.translation.truncate() + target_dir * 14.0;
                commands.spawn((
                    crate::weapons::Projectile {
                        damage: 1,
                        remaining: 1.0,
                    },
                    Velocity(target_dir * 360.0),
                    crate::physics::NoGravity,
                    Collider::new(Vec2::splat(8.0)),
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectile_magic.png"),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(8.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(pos.extend(1.5)),
                        ..default()
                    },
                ));
                turret.fire_cooldown = 1.5;
            }
        }
    }
}

fn tick_bombs(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Bomb, &Transform)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut bomb, _t) in &mut q {
        bomb.fuse -= dt;
        if bomb.fuse <= 0.0 {
            // TODO: spawn explosion particles + damage event for enemies (Étape E)
            commands.entity(entity).despawn();
        }
    }
}

fn tick_placed(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Placed)>) {
    let dt = time.delta_seconds();
    for (entity, mut p) in &mut q {
        p.ttl -= dt;
        if p.ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
