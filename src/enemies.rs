//! Ennemis avec IA simple : Crawler patrouilleur, Flyer sinusoïdal.
//! Tuables au saut sur la tête (stomp) ou avec la dague / le projectile
//! magique. Touche le joueur sur les côtés → mort.

use crate::audio::PlayerJumped;
use crate::effects::ScreenShake;
use crate::items::ActiveEffects;
use crate::physics::{Collider, Velocity};
use crate::player::{Player, PlayerController, PlayerHit};
use crate::states::GameState;
use crate::weapons::{AttackHitbox, Projectile};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    /// Patrouille gauche-droite sur une plateforme.
    Crawler,
    /// Vole en sinusoïde Y, suit le joueur X.
    Flyer,
    /// Statique, tire un projectile toutes les ~2 s vers le joueur.
    Spitter,
    /// Patrouille, charge sur le joueur s'il est dans range.
    Charger,
    /// Silhouette qui traverse les murs et suit lentement le joueur.
    /// Plus de PV, ne peut pas être stompée.
    Wraith,
}

impl EnemyKind {
    fn texture(self) -> &'static str {
        match self {
            EnemyKind::Crawler => "sprites/enemy_crawler.png",
            EnemyKind::Flyer => "sprites/enemy_flyer.png",
            EnemyKind::Spitter => "sprites/enemy_spitter.png",
            EnemyKind::Charger => "sprites/enemy_charger.png",
            EnemyKind::Wraith => "sprites/enemy_wraith.png",
        }
    }

    fn size(self) -> Vec2 {
        match self {
            EnemyKind::Crawler => Vec2::new(20.0, 14.0),
            EnemyKind::Flyer => Vec2::new(18.0, 18.0),
            EnemyKind::Spitter => Vec2::new(24.0, 20.0),
            EnemyKind::Charger => Vec2::new(22.0, 18.0),
            EnemyKind::Wraith => Vec2::new(20.0, 28.0),
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: u32,
    /// Cooldown d'invincibilité après un hit (évite les hits multiples).
    pub hit_cooldown: f32,
    /// Phase utilisée par l'IA (timer pour patrol direction ou sin).
    pub phase: f32,
    /// Direction courante de patrouille (1.0 droite, -1.0 gauche).
    pub direction: f32,
    /// Bornes de patrouille gauche/droite (Crawler uniquement).
    pub patrol_min_x: f32,
    pub patrol_max_x: f32,
    /// Y de base utilisé pour la sinusoïde du Flyer.
    pub home_y: f32,
}

const STOMP_BOUNCE: f32 = 480.0;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemies).add_systems(
            Update,
            (
                tick_enemy_state,
                ai_crawler,
                ai_flyer,
                ai_spitter,
                ai_charger,
                ai_wraith,
                tick_enemy_projectiles,
                check_stomp_or_damage,
                apply_attack_hits,
                apply_projectile_hits,
                despawn_dead,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Projectile tiré par un Spitter. Tue le joueur au contact (sauf
/// invul). Se despawn au contact d'un solide ou après 3 s.
#[derive(Component)]
pub struct EnemyProjectile {
    pub remaining: f32,
}

fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Crawler,
        Vec2::new(-100.0, -260.0),
        -400.0..200.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Crawler,
        Vec2::new(700.0, -260.0),
        500.0..900.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Flyer,
        Vec2::new(450.0, 100.0),
        300.0..900.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Flyer,
        Vec2::new(1500.0, 320.0),
        1300.0..1900.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Spitter,
        Vec2::new(880.0, 60.0),
        0.0..0.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Spitter,
        Vec2::new(1080.0, 360.0),
        0.0..0.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Charger,
        Vec2::new(1500.0, -258.0),
        1200.0..1800.0,
    );
    spawn_enemy(
        &mut commands,
        &asset_server,
        EnemyKind::Wraith,
        Vec2::new(1700.0, 100.0),
        1400.0..2100.0,
    );
}

pub fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: EnemyKind,
    pos: Vec2,
    patrol: std::ops::Range<f32>,
) {
    let size = kind.size();
    let hp = match kind {
        EnemyKind::Crawler => 2,
        EnemyKind::Flyer => 1,
        EnemyKind::Spitter => 3,
        EnemyKind::Charger => 3,
        EnemyKind::Wraith => 5,
    };
    commands.spawn((
        Enemy {
            kind,
            hp,
            hit_cooldown: 0.0,
            phase: 0.0,
            direction: 1.0,
            patrol_min_x: patrol.start,
            patrol_max_x: patrol.end,
            home_y: pos.y,
        },
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

fn tick_enemy_state(time: Res<Time>, mut q: Query<&mut Enemy>) {
    let dt = time.delta_seconds();
    for mut e in &mut q {
        e.hit_cooldown = (e.hit_cooldown - dt).max(0.0);
        e.phase += dt;
    }
}

const CRAWLER_SPEED: f32 = 80.0;

fn ai_crawler(time: Res<Time>, mut q: Query<(&mut Enemy, &mut Transform, &mut Sprite)>) {
    let dt = time.delta_seconds();
    for (mut e, mut t, mut sprite) in &mut q {
        if e.kind != EnemyKind::Crawler {
            continue;
        }
        t.translation.x += e.direction * CRAWLER_SPEED * dt;
        // Demi-tour aux bornes
        if t.translation.x <= e.patrol_min_x {
            t.translation.x = e.patrol_min_x;
            e.direction = 1.0;
        } else if t.translation.x >= e.patrol_max_x {
            t.translation.x = e.patrol_max_x;
            e.direction = -1.0;
        }
        sprite.flip_x = e.direction < 0.0;
    }
}

const FLYER_AMPLITUDE: f32 = 28.0;
const FLYER_FREQUENCY: f32 = 1.8;
const FLYER_FOLLOW_SPEED: f32 = 140.0;
const FLYER_FOLLOW_LERP: f32 = 1.2;

fn ai_flyer(
    time: Res<Time>,
    mut flyers: Query<(&mut Enemy, &mut Transform, &mut Sprite), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_seconds();
    let player_x = player.get_single().map(|t| t.translation.x).unwrap_or(0.0);
    for (mut e, mut t, mut sprite) in &mut flyers {
        if e.kind != EnemyKind::Flyer {
            continue;
        }
        // Y sinusoïdal
        t.translation.y = e.home_y + (e.phase * FLYER_FREQUENCY).sin() * FLYER_AMPLITUDE;
        // X : suit le joueur si dans la fenêtre de patrouille
        let target_x = player_x.clamp(e.patrol_min_x, e.patrol_max_x);
        let direction = (target_x - t.translation.x).signum();
        let alpha = 1.0 - (-FLYER_FOLLOW_LERP * dt).exp();
        let new_x = t.translation.x
            + (target_x - t.translation.x) * alpha
            + direction * FLYER_FOLLOW_SPEED * dt * 0.3;
        t.translation.x = new_x.clamp(e.patrol_min_x, e.patrol_max_x);
        sprite.flip_x = (player_x - t.translation.x) < 0.0;
        e.direction = if sprite.flip_x { -1.0 } else { 1.0 };
    }
}

fn aabb_overlap(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let ah = a_size * 0.5;
    let bh = b_size * 0.5;
    (a_pos.x - b_pos.x).abs() < ah.x + bh.x && (a_pos.y - b_pos.y).abs() < ah.y + bh.y
}

#[allow(clippy::too_many_arguments)]
fn check_stomp_or_damage(
    mut commands: Commands,
    mut player_q: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PlayerController,
            &Collider,
        ),
        With<Player>,
    >,
    mut enemies: Query<(Entity, &Transform, &Collider, &mut Enemy), Without<Player>>,
    effects: Res<ActiveEffects>,
    mut hit: EventWriter<PlayerHit>,
    mut shake: ResMut<ScreenShake>,
    mut jumped: EventWriter<PlayerJumped>,
) {
    let Ok((p_t, mut p_v, mut p_ctrl, p_c)) = player_q.get_single_mut() else {
        return;
    };

    for (entity, e_t, e_c, mut enemy) in &mut enemies {
        if !aabb_overlap(p_t.translation, p_c.size, e_t.translation, e_c.size) {
            continue;
        }

        // Détection stomp : joueur AU-DESSUS de l'ennemi et tombant.
        // Le Wraith n'est pas stompable.
        let player_bottom = p_t.translation.y - p_c.size.y * 0.5;
        let enemy_top = e_t.translation.y + e_c.size.y * 0.5;
        let stomped =
            enemy.kind != EnemyKind::Wraith && p_v.0.y < -50.0 && player_bottom >= enemy_top - 8.0;

        if stomped {
            // L'ennemi prend dégâts max, joueur rebondit + recharge double saut.
            enemy.hp = enemy.hp.saturating_sub(2);
            enemy.hit_cooldown = 0.25;
            p_v.0.y = STOMP_BOUNCE;
            p_ctrl.air_jumps_remaining = 1;
            shake.add(0.20);
            jumped.send(PlayerJumped);
            if enemy.hp == 0 {
                commands.entity(entity).despawn();
            }
        } else if effects.invincible <= 0.0 {
            hit.send(PlayerHit { damage: 1 });
            return;
        }
    }
}

fn apply_attack_hits(
    mut enemies: Query<(Entity, &Transform, &Collider, &mut Enemy)>,
    hitboxes: Query<(&Transform, &Collider, &AttackHitbox)>,
    mut commands: Commands,
) {
    for (h_t, h_c, hb) in &hitboxes {
        for (entity, e_t, e_c, mut enemy) in &mut enemies {
            if enemy.hit_cooldown > 0.0 {
                continue;
            }
            if aabb_overlap(h_t.translation, h_c.size, e_t.translation, e_c.size) {
                enemy.hp = enemy.hp.saturating_sub(hb.damage);
                enemy.hit_cooldown = 0.25;
                if enemy.hp == 0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn apply_projectile_hits(
    mut commands: Commands,
    mut enemies: Query<(Entity, &Transform, &Collider, &mut Enemy)>,
    projectiles: Query<(Entity, &Transform, &Collider, &Projectile)>,
) {
    for (p_ent, p_t, p_c, proj) in &projectiles {
        for (e_ent, e_t, e_c, mut enemy) in &mut enemies {
            if enemy.hit_cooldown > 0.0 {
                continue;
            }
            if aabb_overlap(p_t.translation, p_c.size, e_t.translation, e_c.size) {
                enemy.hp = enemy.hp.saturating_sub(proj.damage);
                enemy.hit_cooldown = 0.25;
                if enemy.hp == 0 {
                    commands.entity(e_ent).despawn();
                }
                commands.entity(p_ent).despawn();
                break; // un projectile ne touche qu'un ennemi
            }
        }
    }
}

/// IA du Spitter : tire un projectile vers le joueur tous les ~2 s.
fn ai_spitter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut q: Query<(&mut Enemy, &Transform), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_t) = player.get_single() else {
        return;
    };
    for (e, t) in &mut q {
        if e.kind != EnemyKind::Spitter {
            continue;
        }
        // Phase utilisée comme cooldown : tire tous les 2.0 s
        if (e.phase % 2.0) < time.delta_seconds() {
            let dir =
                (player_t.translation.truncate() - t.translation.truncate()).normalize_or_zero();
            spawn_enemy_projectile(
                &mut commands,
                &asset_server,
                t.translation.truncate() + dir * 18.0,
                dir * 240.0,
            );
        }
    }
}

fn spawn_enemy_projectile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    velocity: Vec2,
) {
    let size = Vec2::splat(12.0);
    commands.spawn((
        EnemyProjectile { remaining: 3.0 },
        Velocity(velocity),
        crate::physics::NoGravity,
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load("sprites/enemy_projectile.png"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(1.5)),
            ..default()
        },
    ));
}

fn tick_enemy_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &Transform, &Collider, &mut EnemyProjectile)>,
    player: Query<(&Transform, &Collider), With<Player>>,
    effects: Res<ActiveEffects>,
    mut hit: EventWriter<PlayerHit>,
) {
    let dt = time.delta_seconds();
    let Ok((p_t, p_c)) = player.get_single() else {
        return;
    };
    for (entity, t, c, mut proj) in &mut projectiles {
        proj.remaining -= dt;
        if proj.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if aabb_overlap(t.translation, c.size, p_t.translation, p_c.size)
            && effects.invincible <= 0.0
        {
            hit.send(PlayerHit { damage: 1 });
            commands.entity(entity).despawn();
        }
    }
}

/// Charger : patrouille comme un crawler, mais accélère brutalement
/// quand le joueur est dans une fenêtre de 160 px horizontale et à la
/// même altitude approximative.
fn ai_charger(
    time: Res<Time>,
    mut q: Query<(&mut Enemy, &mut Transform, &mut Sprite), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_seconds();
    let Ok(player_t) = player.get_single() else {
        return;
    };
    for (mut e, mut t, mut sprite) in &mut q {
        if e.kind != EnemyKind::Charger {
            continue;
        }
        let dx = player_t.translation.x - t.translation.x;
        let dy = (player_t.translation.y - t.translation.y).abs();
        let in_range = dx.abs() < 240.0 && dy < 80.0;
        let speed = if in_range { 240.0 } else { 90.0 };

        if in_range {
            e.direction = dx.signum();
        }
        t.translation.x += e.direction * speed * dt;
        if t.translation.x <= e.patrol_min_x {
            t.translation.x = e.patrol_min_x;
            e.direction = 1.0;
        } else if t.translation.x >= e.patrol_max_x {
            t.translation.x = e.patrol_max_x;
            e.direction = -1.0;
        }
        sprite.flip_x = e.direction < 0.0;
    }
}

/// Wraith : flotte lentement vers le joueur en passant à travers les
/// murs (pas de collision avec les solides puisqu'on bouge à la main).
/// Non-stompable (le check est fait dans check_stomp_or_damage). Plus
/// dangereux car on peut pas s'en débarrasser facilement.
const WRAITH_SPEED: f32 = 60.0;

fn ai_wraith(
    time: Res<Time>,
    mut q: Query<(&mut Enemy, &mut Transform, &mut Sprite), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_seconds();
    let Ok(player_t) = player.get_single() else {
        return;
    };
    for (mut e, mut t, mut sprite) in &mut q {
        if e.kind != EnemyKind::Wraith {
            continue;
        }
        let to_player = player_t.translation.truncate() - t.translation.truncate();
        let dir = to_player.normalize_or_zero();
        t.translation.x += dir.x * WRAITH_SPEED * dt;
        t.translation.y += dir.y * WRAITH_SPEED * dt;
        // Reste dans la fenêtre de patrouille
        t.translation.x = t.translation.x.clamp(e.patrol_min_x, e.patrol_max_x);
        sprite.flip_x = dir.x < 0.0;
        e.direction = if dir.x < 0.0 { -1.0 } else { 1.0 };
        // Sinusoïde subtile sur Y pour l'effet "flottant"
        t.translation.y += (e.phase * 2.0).sin() * 0.3;
    }
}

fn despawn_dead() {}
