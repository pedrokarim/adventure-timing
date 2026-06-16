//! Physique 2D simple : intégration verlet-like et collisions AABB
//! résolues axe par axe. Pas de moteur dynamique : on contrôle tout
//! manuellement pour un feel platformer précis.

use crate::states::GameState;
use bevy::prelude::*;

/// Gravité vers le bas (px/s²). Calibrée pour que les sauts décrits dans
/// le module player atteignent ~120 px de hauteur en ~0.4 s.
pub const GRAVITY: f32 = 2400.0;

/// Vitesse de chute max, évite de traverser une plateforme à grande
/// vitesse (tunneling) et plafonne la sensation de chute.
pub const MAX_FALL_SPEED: f32 = 900.0;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec2);

/// Boîte de collision centrée sur le Transform de l'entité.
#[derive(Component, Debug, Clone, Copy)]
pub struct Collider {
    pub size: Vec2,
}

impl Collider {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

/// Marqueur d'un solide statique (sol, mur, plateforme).
#[derive(Component)]
pub struct Solid;

/// Marqueur pour les entités qui ne subissent pas la gravité (projectiles
/// magiques, items lévitants, …).
#[derive(Component)]
pub struct NoGravity;

/// Drapeau de contact bas, mis à jour à chaque pas par le système de
/// collision. Utilisé par le contrôleur du joueur (coyote time, saut).
#[derive(Component, Default, Debug)]
pub struct Grounded(pub bool);

/// Renvoie une AABB (min, max) pour une entité positionnée à `translation`
/// avec un `Collider` donné.
fn aabb(translation: Vec3, collider: &Collider) -> (Vec2, Vec2) {
    let half = collider.size * 0.5;
    let center = translation.truncate();
    (center - half, center + half)
}

/// Teste l'intersection de deux AABB (strictement, sans tolérance).
fn intersects(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> bool {
    a.0.x < b.1.x && a.1.x > b.0.x && a.0.y < b.1.y && a.1.y > b.0.y
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_gravity, move_and_collide)
                .chain()
                .in_set(PhysicsSet)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Set permettant aux autres systèmes (animation, caméra) de se placer
/// après la physique sans dépendance directe.
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub struct PhysicsSet;

fn apply_gravity(time: Res<Time>, mut q: Query<&mut Velocity, Without<NoGravity>>) {
    let dt = time.delta_seconds();
    for mut v in &mut q {
        v.0.y -= GRAVITY * dt;
        if v.0.y < -MAX_FALL_SPEED {
            v.0.y = -MAX_FALL_SPEED;
        }
    }
}

/// Déplace les entités dynamiques (avec Velocity) et résout les
/// collisions axe par axe contre tous les `Solid`. Met `Grounded` à jour.
fn move_and_collide(
    time: Res<Time>,
    mut dynamics: Query<(&mut Transform, &mut Velocity, &Collider, Option<&mut Grounded>)>,
    solids: Query<(&Transform, &Collider), (With<Solid>, Without<Velocity>)>,
) {
    let dt = time.delta_seconds();
    let solids: Vec<(Vec2, Vec2)> = solids
        .iter()
        .map(|(t, c)| aabb(t.translation, c))
        .collect();

    for (mut transform, mut velocity, collider, mut grounded) in &mut dynamics {
        // --- Axe X ---
        let dx = velocity.0.x * dt;
        transform.translation.x += dx;
        let mut self_box = aabb(transform.translation, collider);

        for solid in &solids {
            if intersects(self_box, *solid) {
                let overlap = if dx > 0.0 {
                    self_box.1.x - solid.0.x // pousse à gauche
                } else if dx < 0.0 {
                    self_box.0.x - solid.1.x // pousse à droite (négatif)
                } else {
                    0.0
                };
                transform.translation.x -= overlap;
                velocity.0.x = 0.0;
                self_box = aabb(transform.translation, collider);
            }
        }

        // --- Axe Y ---
        let dy = velocity.0.y * dt;
        transform.translation.y += dy;
        let mut self_box = aabb(transform.translation, collider);
        let mut landed = false;

        for solid in &solids {
            if intersects(self_box, *solid) {
                let overlap = if dy > 0.0 {
                    self_box.1.y - solid.0.y // pousse vers le bas
                } else if dy < 0.0 {
                    landed = true;
                    self_box.0.y - solid.1.y // pousse vers le haut (négatif)
                } else {
                    0.0
                };
                transform.translation.y -= overlap;
                velocity.0.y = 0.0;
                self_box = aabb(transform.translation, collider);
            }
        }

        if let Some(g) = grounded.as_deref_mut() {
            g.0 = landed;
        }
    }
}
