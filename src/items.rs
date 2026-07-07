//! Items passifs ramassables. Bobbent au sol, déclenchent un effet
//! temporaire ou instant à la collecte. Effets actifs trackés dans
//! `ActiveEffects`, consultés par les autres modules (player, level).

use crate::physics::Collider;
use crate::player::{Player, PlayerController, PlayerHp};
use crate::states::GameState;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub enum ItemKind {
    /// +1 saut en l'air immédiat (recharge le double saut)
    AirJumpCrystal,
    /// Invincibilité 3 s contre les pics et la chute hors monde
    AmberPetal,
    /// Saut +30 % pour 5 s
    WhiteFeather,
    /// Slowmo : monde tourne à 50 % pendant 4 s
    Hourglass,
    /// Restaure 1 PV au joueur
    Heart,
    /// La prochaine mort sera ignorée (consommable persistant)
    MemoryPetal,
}

impl ItemKind {
    fn texture(self) -> &'static str {
        match self {
            ItemKind::AirJumpCrystal => "sprites/item_crystal.png",
            ItemKind::AmberPetal => "sprites/item_petal.png",
            ItemKind::WhiteFeather => "sprites/item_feather.png",
            ItemKind::Hourglass => "sprites/item_hourglass.png",
            ItemKind::Heart => "sprites/item_heart.png",
            ItemKind::MemoryPetal => "sprites/item_memory.png",
        }
    }

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            ItemKind::AirJumpCrystal => "Cristal",
            ItemKind::AmberPetal => "Petale d'ambre",
            ItemKind::WhiteFeather => "Plume",
            ItemKind::Hourglass => "Sablier",
            ItemKind::Heart => "Coeur",
            ItemKind::MemoryPetal => "Petale memoire",
        }
    }
}

#[derive(Component)]
pub struct Item {
    pub kind: ItemKind,
    pub home_y: f32,
    pub bob_phase: f32,
}

/// Effets actifs sur le joueur. Les modules consommateurs lisent ces
/// timers et changent leur comportement quand > 0.
#[derive(Resource, Default, Debug)]
pub struct ActiveEffects {
    pub invincible: f32,
    pub jump_boost: f32,
    pub time_slow: f32,
    /// Si true, la prochaine PlayerDied sera annulée (HP restauré).
    /// Le système de death qui consomme l'événement doit aussi consommer
    /// ce flag.
    pub skip_next_death: bool,
}

#[derive(Event, Debug)]
pub struct ItemPickedUp {
    pub kind: ItemKind,
}

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveEffects>()
            .add_event::<ItemPickedUp>()
            .add_systems(Startup, spawn_items)
            .add_systems(
                Update,
                (
                    animate_items,
                    check_pickup,
                    tick_active_effects,
                    apply_time_slow,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

const ITEM_SIZE: Vec2 = Vec2::new(16.0, 16.0);

pub fn spawn_item(commands: &mut Commands, asset_server: &AssetServer, kind: ItemKind, pos: Vec2) {
    commands.spawn((
        Item {
            kind,
            home_y: pos.y,
            bob_phase: 0.0,
        },
        Collider::new(ITEM_SIZE),
        SpriteBundle {
            texture: asset_server.load(kind.texture()),
            sprite: Sprite {
                custom_size: Some(ITEM_SIZE),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.4)),
            ..default()
        },
    ));
}

fn spawn_items(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::AirJumpCrystal,
        Vec2::new(-180.0, -200.0),
    );
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::AmberPetal,
        Vec2::new(50.0, -120.0),
    );
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::WhiteFeather,
        Vec2::new(280.0, -38.0),
    );
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::Hourglass,
        Vec2::new(1080.0, 360.0),
    );
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::Heart,
        Vec2::new(640.0, -80.0),
    );
    spawn_item(
        &mut commands,
        &asset_server,
        ItemKind::MemoryPetal,
        Vec2::new(1800.0, 280.0),
    );
}

fn animate_items(time: Res<Time>, mut q: Query<(&mut Item, &mut Transform)>) {
    let dt = time.delta_seconds();
    for (mut item, mut transform) in &mut q {
        item.bob_phase += dt * 2.5;
        transform.translation.y = item.home_y + item.bob_phase.sin() * 3.5;
        // léger balancement, ça donne vie
        transform.rotation = Quat::from_rotation_z((item.bob_phase * 0.5).sin() * 0.18);
    }
}

fn aabb_overlap(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let ah = a_size * 0.5;
    let bh = b_size * 0.5;
    (a_pos.x - b_pos.x).abs() < ah.x + bh.x && (a_pos.y - b_pos.y).abs() < ah.y + bh.y
}

fn check_pickup(
    mut commands: Commands,
    items: Query<(Entity, &Transform, &Collider, &Item), Without<Player>>,
    mut player_q: Query<
        (&Transform, &Collider, &mut PlayerController, &mut PlayerHp),
        With<Player>,
    >,
    mut events: EventWriter<ItemPickedUp>,
    mut active: ResMut<ActiveEffects>,
) {
    let Ok((p_t, p_c, mut ctrl, mut hp)) = player_q.get_single_mut() else {
        return;
    };
    for (entity, t, c, item) in &items {
        if aabb_overlap(p_t.translation, p_c.size, t.translation, c.size) {
            match item.kind {
                ItemKind::AirJumpCrystal => {
                    ctrl.air_jumps_remaining = (ctrl.air_jumps_remaining + 1).min(3);
                }
                ItemKind::AmberPetal => active.invincible = 3.0,
                ItemKind::WhiteFeather => active.jump_boost = 5.0,
                ItemKind::Hourglass => active.time_slow = 4.0,
                ItemKind::Heart => {
                    hp.current = (hp.current + 1).min(hp.max);
                }
                ItemKind::MemoryPetal => {
                    active.skip_next_death = true;
                }
            }
            events.send(ItemPickedUp { kind: item.kind });
            commands.entity(entity).despawn();
        }
    }
}

fn tick_active_effects(time: Res<Time>, mut active: ResMut<ActiveEffects>) {
    let dt = time.delta_seconds();
    active.invincible = (active.invincible - dt).max(0.0);
    active.jump_boost = (active.jump_boost - dt).max(0.0);
    active.time_slow = (active.time_slow - dt).max(0.0);
}

/// Slowmo via Time::<Virtual>::relative_speed. Tout (physique, anim,
/// caméra) ralentit en même temps, donne une vraie sensation magique.
fn apply_time_slow(active: Res<ActiveEffects>, mut time: ResMut<Time<Virtual>>) {
    if active.time_slow > 0.0 {
        time.set_relative_speed(0.5);
    } else if time.relative_speed() < 1.0 {
        time.set_relative_speed(1.0);
    }
}
