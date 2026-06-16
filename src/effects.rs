//! Effets visuels : screen shake de la caméra, squash & stretch du
//! joueur, particules de poussière au saut et à l'atterrissage.

use crate::audio::{PlayerAirJumped, PlayerLanded};
use crate::physics::{Grounded, Velocity};
use crate::player::Player;
use bevy::prelude::*;

const SHAKE_DECAY: f32 = 8.0;
const SHAKE_MAX: f32 = 16.0;

const SQUASH_LERP_RATE: f32 = 18.0;
const SQUASH_LAND_X: f32 = 1.35;
const SQUASH_LAND_Y: f32 = 0.65;
const STRETCH_JUMP_X: f32 = 0.75;
const STRETCH_JUMP_Y: f32 = 1.30;

const DUST_LIFETIME: f32 = 0.45;
const DUST_COLOR: Color = Color::srgba(0.95, 0.95, 0.85, 0.85);
const DUST_GRAVITY: f32 = 600.0;

#[derive(Resource, Default, Debug)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub remaining: f32,
    pub initial: f32,
}

/// Marque le joueur pour appliquer le squash/stretch sans dépendre
/// directement du composant Player (évite un cycle).
#[derive(Component, Default)]
pub struct SquashStretch {
    /// Mémorise l'état grounded au tick précédent pour détecter
    /// l'atterrissage (transition false → true).
    pub was_grounded: bool,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>().add_systems(
            Update,
            (
                squash_stretch_system,
                spawn_dust_on_events,
                spawn_air_jump_ring,
                tick_particles,
                apply_screen_shake,
            ),
        );
    }
}

fn squash_stretch_system(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &Velocity, &Grounded, &mut SquashStretch), With<Player>>,
) {
    let dt = time.delta_seconds();
    let alpha = 1.0 - (-SQUASH_LERP_RATE * dt).exp();

    for (mut transform, velocity, grounded, mut state) in &mut q {
        let target = if grounded.0 {
            if !state.was_grounded {
                // Atterrissage : squash fort, retour rapide.
                Vec3::new(SQUASH_LAND_X, SQUASH_LAND_Y, 1.0)
            } else {
                Vec3::ONE
            }
        } else if velocity.0.y > 50.0 {
            Vec3::new(STRETCH_JUMP_X, STRETCH_JUMP_Y, 1.0)
        } else if velocity.0.y < -50.0 {
            // Chute : léger stretch vers le bas.
            Vec3::new(0.85, 1.15, 1.0)
        } else {
            Vec3::ONE
        };

        transform.scale = transform.scale.lerp(target, alpha);
        state.was_grounded = grounded.0;
    }
}

fn spawn_dust_on_events(
    mut commands: Commands,
    q: Query<(&Transform, &Velocity, &Grounded, &SquashStretch), With<Player>>,
    mut shake: ResMut<ScreenShake>,
    mut landed: EventWriter<PlayerLanded>,
) {
    for (transform, velocity, grounded, state) in &q {
        if grounded.0 && !state.was_grounded {
            let impact = (velocity.0.y.abs() / 600.0).clamp(0.0, 1.0);
            shake.add(impact * 0.35);
            spawn_dust_burst(&mut commands, transform.translation.truncate(), 8, false);
            landed.send(PlayerLanded(impact));
        }
        if !grounded.0 && state.was_grounded && velocity.0.y > 200.0 {
            spawn_dust_burst(&mut commands, transform.translation.truncate(), 5, true);
        }
    }
}

fn spawn_dust_burst(commands: &mut Commands, pos: Vec2, count: u32, jump: bool) {
    // RNG pseudo-aléatoire stable basé sur la position + index, évite
    // d'ajouter une crate rand pour des particules cosmétiques.
    for i in 0..count {
        let seed = (pos.x * 13.0 + pos.y * 7.0 + i as f32 * 31.0).sin();
        let angle = if jump {
            // Saut : explosion vers le bas + côtés.
            std::f32::consts::PI + seed * 0.6
        } else {
            // Atterrissage : explosion latérale.
            if i % 2 == 0 {
                seed * 0.4
            } else {
                std::f32::consts::PI + seed * 0.4
            }
        };
        let speed = 90.0 + seed.abs() * 80.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin().abs() * speed * 0.6 + 60.0);
        let size = 4.0 + seed.abs() * 3.0;
        let foot_y = pos.y - 18.0;

        commands.spawn((
            Particle {
                velocity,
                remaining: DUST_LIFETIME,
                initial: DUST_LIFETIME,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: DUST_COLOR,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(pos.x, foot_y, 2.0)),
                ..default()
            },
        ));
    }
}

fn tick_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut particle, mut transform, mut sprite) in &mut q {
        particle.remaining -= dt;
        if particle.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        particle.velocity.y -= DUST_GRAVITY * dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let t = particle.remaining / particle.initial;
        sprite.color.set_alpha(t.clamp(0.0, 1.0) * 0.85);
    }
}

/// Burst circulaire de particules à la position du joueur quand il
/// active son double saut. Plus aérien que la poussière au sol (couleur
/// cyan, partent en cercle).
fn spawn_air_jump_ring(
    mut commands: Commands,
    mut events: EventReader<PlayerAirJumped>,
    player: Query<&Transform, With<Player>>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    let Ok(transform) = player.get_single() else {
        return;
    };
    let pos = transform.translation.truncate();
    const RING_COLOR: Color = Color::srgba(0.70, 0.94, 1.00, 0.90);
    const COUNT: u32 = 12;
    const SPEED: f32 = 180.0;
    for i in 0..COUNT {
        let angle = (i as f32 / COUNT as f32) * std::f32::consts::TAU;
        let velocity = Vec2::new(angle.cos(), angle.sin().abs() * 0.4) * SPEED;
        commands.spawn((
            Particle {
                velocity,
                remaining: 0.35,
                initial: 0.35,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: RING_COLOR,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(pos.x, pos.y - 12.0, 2.0)),
                ..default()
            },
        ));
    }
}

fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let Ok(mut cam) = camera.get_single_mut() else {
        return;
    };
    if shake.trauma <= 0.0 {
        return;
    }

    let intensity = shake.trauma * shake.trauma; // courbe ease-in
    // Offset pseudo-aléatoire basé sur le temps. Pas de crate rand.
    let t = time.elapsed_seconds();
    let offset_x = (t * 71.0).sin() * SHAKE_MAX * intensity;
    let offset_y = (t * 53.0).cos() * SHAKE_MAX * intensity;
    cam.translation.x += offset_x;
    cam.translation.y += offset_y;

    shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_seconds()).max(0.0);
}
