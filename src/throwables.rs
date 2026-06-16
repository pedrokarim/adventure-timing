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
    /// Lancée en arc, explose au contact d'un solide ou après 2 s.
    Bomb,
    /// Bloc 32×16 posé au pied du joueur, solide, despawn après 6 s.
    IceBlock,
    /// Plateforme cyan 48×6 posée devant le joueur, solide, despawn 4 s.
    MagicPlatform,
}

impl ThrowableKind {
    pub fn label(self) -> &'static str {
        match self {
            ThrowableKind::Bomb => "Bombe",
            ThrowableKind::IceBlock => "Glace",
            ThrowableKind::MagicPlatform => "Plateforme",
        }
    }

    fn texture(self) -> &'static str {
        match self {
            ThrowableKind::Bomb => "sprites/throwable_bomb.png",
            ThrowableKind::IceBlock => "sprites/throwable_ice.png",
            ThrowableKind::MagicPlatform => "sprites/throwable_platform.png",
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
                Some(ThrowableKind::IceBlock),
                Some(ThrowableKind::MagicPlatform),
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
            spawn_bomb(&mut commands, &asset_server, pos + Vec2::new(20.0 * dir, 6.0), dir);
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
