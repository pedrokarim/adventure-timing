//! Armes & combat actif. Dague (court range, combo 3 coups) et Bâton
//! magique (long range, projectile énergie). L'événement `WeaponHitEnemy`
//! sera consommé par le futur module `enemies`.
//!
//! Touche d'attaque : `F` (séparée de Espace = saut)
//! Pogostick : attaque vers le bas en l'air si maintenir `S` ou `↓` —
//! rebondit + recharge le double saut

use crate::physics::{Collider, NoGravity, Velocity};
use crate::player::Player;
use crate::states::GameState;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum WeaponKind {
    Dagger,
    MagicStaff,
}

impl WeaponKind {
    pub fn label(self) -> &'static str {
        match self {
            WeaponKind::Dagger => "Dague",
            WeaponKind::MagicStaff => "Baton magique",
        }
    }
}

#[derive(Resource)]
pub struct CurrentWeapon(pub WeaponKind);

#[derive(Resource, Default)]
pub struct WeaponState {
    /// Temps restant avant prochaine attaque possible.
    pub cooldown: f32,
    /// Étape du combo dague (0..=2).
    pub combo_step: u8,
    /// Fenêtre temporelle pour enchaîner le prochain coup du combo.
    pub combo_window: f32,
}

/// Hitbox d'attaque mêlée. Spawnée pour ~150 ms puis despawn.
#[derive(Component)]
pub struct AttackHitbox {
    pub damage: u32,
    pub knockback: Vec2,
    pub remaining: f32,
    /// Si vrai, c'est une attaque vers le bas en saut (pogostick).
    pub is_pogo: bool,
}

/// Projectile à distance (bâton magique, plus tard flèche).
#[derive(Component)]
pub struct Projectile {
    pub damage: u32,
    pub remaining: f32,
}

/// Émis quand une hitbox/projectile touche un ennemi. Le module
/// `enemies` (futur) écoute pour appliquer dégâts + knockback.
#[derive(Event, Debug)]
pub struct WeaponHitEnemy {
    pub target: Entity,
    pub damage: u32,
    pub knockback: Vec2,
    /// `true` si c'est un coup de pogo (l'attaquant doit rebondir).
    pub is_pogo: bool,
}

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentWeapon(WeaponKind::Dagger))
            .init_resource::<WeaponState>()
            .add_event::<WeaponHitEnemy>()
            .add_systems(
                Update,
                (
                    tick_weapon_state,
                    handle_attack_input,
                    tick_attack_hitboxes,
                    tick_projectiles,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn tick_weapon_state(time: Res<Time>, mut state: ResMut<WeaponState>) {
    let dt = time.delta_seconds();
    state.cooldown = (state.cooldown - dt).max(0.0);
    state.combo_window = (state.combo_window - dt).max(0.0);
    if state.combo_window <= 0.0 {
        state.combo_step = 0;
    }
}

fn handle_attack_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<WeaponState>,
    weapon: Res<CurrentWeapon>,
    player: Query<(&Transform, &Player)>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    if state.cooldown > 0.0 {
        return;
    }

    let Ok((t, p)) = player.get_single() else {
        return;
    };
    let dir = if p.facing > 0.0 { 1.0 } else { -1.0 };
    let down_pressed =
        keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);

    match weapon.0 {
        WeaponKind::Dagger => {
            // Dégât augmente avec le combo (1, 1, 2).
            let damage = if state.combo_step >= 2 { 2 } else { 1 };
            if down_pressed {
                // Pogo : hitbox sous le perso (carrée, 20×20)
                let pos = t.translation.truncate() + Vec2::new(0.0, -22.0);
                spawn_hitbox(
                    &mut commands,
                    pos,
                    Vec2::new(22.0, 22.0),
                    damage,
                    Vec2::new(0.0, -150.0),
                    true,
                );
            } else {
                let pos = t.translation.truncate() + Vec2::new(20.0 * dir, 2.0);
                spawn_hitbox(
                    &mut commands,
                    pos,
                    Vec2::new(22.0, 26.0),
                    damage,
                    Vec2::new(220.0 * dir, 80.0),
                    false,
                );
            }
            state.cooldown = 0.20;
            state.combo_step = (state.combo_step + 1).min(2);
            state.combo_window = 0.42;
        }
        WeaponKind::MagicStaff => {
            let pos = t.translation.truncate() + Vec2::new(20.0 * dir, 4.0);
            spawn_projectile(&mut commands, &asset_server, pos, dir);
            state.cooldown = 0.45;
        }
    }
}

fn spawn_hitbox(
    commands: &mut Commands,
    pos: Vec2,
    size: Vec2,
    damage: u32,
    knockback: Vec2,
    is_pogo: bool,
) {
    let color = if is_pogo {
        Color::srgba(0.55, 0.92, 1.0, 0.55)
    } else {
        Color::srgba(0.95, 0.96, 1.0, 0.55)
    };
    commands.spawn((
        AttackHitbox {
            damage,
            knockback,
            remaining: 0.16,
            is_pogo,
        },
        Collider::new(size),
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(2.0)),
            ..default()
        },
    ));
}

fn spawn_projectile(commands: &mut Commands, asset_server: &AssetServer, pos: Vec2, dir: f32) {
    let size = Vec2::new(18.0, 10.0);
    commands.spawn((
        Projectile {
            damage: 1,
            remaining: 1.2,
        },
        Velocity(Vec2::new(440.0 * dir, 0.0)),
        NoGravity,
        Collider::new(size),
        SpriteBundle {
            texture: asset_server.load("sprites/projectile_magic.png"),
            sprite: Sprite {
                custom_size: Some(size),
                flip_x: dir < 0.0,
                ..default()
            },
            transform: Transform::from_translation(pos.extend(2.0)),
            ..default()
        },
    ));
}

fn tick_attack_hitboxes(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut AttackHitbox)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut h) in &mut q {
        h.remaining -= dt;
        if h.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Projectile)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut p) in &mut q {
        p.remaining -= dt;
        if p.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
