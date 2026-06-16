//! Ennemis avec IA simple : Crawler patrouilleur, Flyer sinusoïdal.
//! Tuables au saut sur la tête (stomp) ou avec la dague / le projectile
//! magique. Touche le joueur sur les côtés → mort.

use crate::audio::PlayerJumped;
use crate::effects::ScreenShake;
use crate::items::ActiveEffects;
use crate::physics::{Collider, Velocity};
use crate::player::{Player, PlayerController};
use crate::states::{GameState, PlayerDied};
use crate::weapons::{AttackHitbox, Projectile};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    /// Patrouille gauche-droite sur une plateforme.
    Crawler,
    /// Vole en sinusoïde Y, suit le joueur X.
    Flyer,
}

impl EnemyKind {
    fn texture(self) -> &'static str {
        match self {
            EnemyKind::Crawler => "sprites/enemy_crawler.png",
            EnemyKind::Flyer => "sprites/enemy_flyer.png",
        }
    }

    fn size(self) -> Vec2 {
        match self {
            EnemyKind::Crawler => Vec2::new(20.0, 14.0),
            EnemyKind::Flyer => Vec2::new(18.0, 18.0),
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
                check_stomp_or_damage,
                apply_attack_hits,
                apply_projectile_hits,
                despawn_dead,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Distribution sur le niveau de test.
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
}

fn spawn_enemy(
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
        (&mut Transform, &mut Velocity, &mut PlayerController, &Collider),
        With<Player>,
    >,
    mut enemies: Query<(Entity, &Transform, &Collider, &mut Enemy), Without<Player>>,
    effects: Res<ActiveEffects>,
    mut died: EventWriter<PlayerDied>,
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
        let player_bottom = p_t.translation.y - p_c.size.y * 0.5;
        let enemy_top = e_t.translation.y + e_c.size.y * 0.5;
        let stomped = p_v.0.y < -50.0 && player_bottom >= enemy_top - 8.0;

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
            died.send(PlayerDied);
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

/// Vide pour l'instant : utilisé plus tard pour les animations de mort
/// (poof particles, fade out). Les enemies sont déjà despawn dans les
/// systèmes d'attaque.
fn despawn_dead() {}
