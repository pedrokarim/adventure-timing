//! Générateur de sprites pixel art. À relancer après modification :
//! `cargo run --example gen_assets`.
//!
//! Tout est dessiné procéduralement avec le crate `image` (dev-dep), ce
//! qui évite d'embarquer des fichiers binaires versionnés et garantit
//! qu'on peut reproduire les assets à tout moment.

use image::{Rgba, RgbaImage};
use std::path::Path;

// ============================================================ Palette ===

const TR: Rgba<u8> = Rgba([0, 0, 0, 0]);

// Joueur
const HAIR: Rgba<u8> = Rgba([60, 36, 28, 255]);
const SKIN: Rgba<u8> = Rgba([238, 196, 158, 255]);
const SKIN_DK: Rgba<u8> = Rgba([198, 154, 118, 255]);
const SHIRT: Rgba<u8> = Rgba([206, 58, 66, 255]);
const SHIRT_DK: Rgba<u8> = Rgba([148, 36, 42, 255]);
const PANTS: Rgba<u8> = Rgba([56, 68, 124, 255]);
const PANTS_DK: Rgba<u8> = Rgba([34, 44, 86, 255]);
const BOOTS: Rgba<u8> = Rgba([46, 30, 22, 255]);
const EYE: Rgba<u8> = Rgba([28, 22, 36, 255]);

// Tiles
const GRASS: Rgba<u8> = Rgba([78, 168, 88, 255]);
const GRASS_DK: Rgba<u8> = Rgba([48, 124, 60, 255]);
const GRASS_LT: Rgba<u8> = Rgba([110, 196, 116, 255]);
const DIRT: Rgba<u8> = Rgba([124, 86, 56, 255]);
const DIRT_DK: Rgba<u8> = Rgba([88, 60, 38, 255]);
const STONE: Rgba<u8> = Rgba([130, 130, 145, 255]);
const STONE_DK: Rgba<u8> = Rgba([88, 88, 105, 255]);
const WOOD: Rgba<u8> = Rgba([150, 102, 60, 255]);
const WOOD_DK: Rgba<u8> = Rgba([102, 68, 38, 255]);
const WOOD_LT: Rgba<u8> = Rgba([186, 132, 78, 255]);

// Hazards
const SPIKE_LT: Rgba<u8> = Rgba([225, 225, 240, 255]);
const SPIKE_DK: Rgba<u8> = Rgba([140, 140, 160, 255]);
const SPIKE_BASE: Rgba<u8> = Rgba([72, 72, 88, 255]);

// Drapeaux
const POLE: Rgba<u8> = Rgba([70, 50, 36, 255]);
const POLE_DK: Rgba<u8> = Rgba([44, 30, 22, 255]);
const CHECK_FLAG: Rgba<u8> = Rgba([238, 200, 64, 255]);
const CHECK_FLAG_DK: Rgba<u8> = Rgba([170, 132, 38, 255]);
const GOAL_FLAG: Rgba<u8> = Rgba([238, 70, 178, 255]);
const GOAL_FLAG_DK: Rgba<u8> = Rgba([164, 40, 122, 255]);

// =========================================================== Helpers ===

fn put(img: &mut RgbaImage, x: i32, y: i32, c: Rgba<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, c);
    }
}

fn rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, c: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, x + dx, y + dy, c);
        }
    }
}

fn hline(img: &mut RgbaImage, x: i32, y: i32, w: i32, c: Rgba<u8>) {
    rect(img, x, y, w, 1, c);
}

fn save(img: &RgbaImage, path: &str) {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    img.save(p).unwrap();
    println!("écrit {path}");
}

// ============================================================ Player ===

#[derive(Clone, Copy)]
struct Pose {
    body_dy: i32,
    head_dy: i32,
    left_arm_dx: i32,
    left_arm_dy: i32,
    right_arm_dx: i32,
    right_arm_dy: i32,
    left_leg_dx: i32,
    left_leg_dy: i32,
    right_leg_dx: i32,
    right_leg_dy: i32,
    arms_up: bool,
}

const IDLE: Pose = Pose {
    body_dy: 0,
    head_dy: 0,
    left_arm_dx: 0,
    left_arm_dy: 0,
    right_arm_dx: 0,
    right_arm_dy: 0,
    left_leg_dx: 0,
    left_leg_dy: 0,
    right_leg_dx: 0,
    right_leg_dy: 0,
    arms_up: false,
};

fn draw_player_frame(img: &mut RgbaImage, frame_x: i32, pose: Pose) {
    let cx = frame_x + 16;
    let head_y = 2 + pose.head_dy;
    let torso_y = 16 + pose.body_dy;
    let leg_top = torso_y + 13;

    // === Tête ===
    // Cheveux : couronne haut + côtés
    rect(img, cx - 5, head_y, 10, 4, HAIR);
    rect(img, cx - 6, head_y + 1, 1, 4, HAIR);
    rect(img, cx + 5, head_y + 1, 1, 4, HAIR);
    // Visage (peau)
    rect(img, cx - 5, head_y + 4, 10, 8, SKIN);
    // Mèches qui pendent sur les côtés du visage
    put(img, cx - 5, head_y + 4, HAIR);
    put(img, cx + 4, head_y + 4, HAIR);
    // Ombre menton
    hline(img, cx - 4, head_y + 11, 8, SKIN_DK);
    // Yeux
    put(img, cx - 3, head_y + 7, EYE);
    put(img, cx + 2, head_y + 7, EYE);
    // Bouche
    hline(img, cx - 1, head_y + 9, 2, SKIN_DK);

    // === Cou ===
    rect(img, cx - 2, head_y + 12, 4, 2, SKIN);

    // === Torse (chemise) ===
    rect(img, cx - 5, torso_y, 10, 12, SHIRT);
    // Encolure et ombrage
    hline(img, cx - 3, torso_y, 6, SHIRT_DK);
    hline(img, cx - 5, torso_y + 11, 10, SHIRT_DK);
    // Petite bande verticale sombre au centre (manteau ouvert)
    rect(img, cx - 1, torso_y + 1, 2, 10, SHIRT_DK);

    // === Bras ===
    if pose.arms_up {
        // Bras levés (saut) : montent au-dessus de la tête
        rect(img, cx - 5, head_y, 2, 6, SHIRT);
        rect(img, cx + 3, head_y, 2, 6, SHIRT);
        // Mains
        rect(img, cx - 5, head_y - 1, 2, 1, SKIN);
        rect(img, cx + 3, head_y - 1, 2, 1, SKIN);
    } else {
        let la_x = cx - 7 + pose.left_arm_dx;
        let la_y = torso_y + 1 + pose.left_arm_dy;
        let ra_x = cx + 5 + pose.right_arm_dx;
        let ra_y = torso_y + 1 + pose.right_arm_dy;
        rect(img, la_x, la_y, 2, 9, SHIRT);
        rect(img, ra_x, ra_y, 2, 9, SHIRT);
        // Mains
        rect(img, la_x, la_y + 9, 2, 2, SKIN);
        rect(img, ra_x, ra_y + 9, 2, 2, SKIN);
    }

    // === Ceinture ===
    hline(img, cx - 5, torso_y + 12, 10, EYE);

    // === Jambes + pantalon ===
    let ll_x = cx - 5 + pose.left_leg_dx;
    let rl_x = cx + 1 + pose.right_leg_dx;
    let ll_y = leg_top;
    let rl_y = leg_top;
    let ll_h = 10 + pose.left_leg_dy;
    let rl_h = 10 + pose.right_leg_dy;
    rect(img, ll_x, ll_y, 4, ll_h, PANTS);
    rect(img, rl_x, rl_y, 4, rl_h, PANTS);
    // Ombrage pantalon
    rect(img, ll_x, ll_y + ll_h - 1, 4, 1, PANTS_DK);
    rect(img, rl_x, rl_y + rl_h - 1, 4, 1, PANTS_DK);

    // === Bottes ===
    rect(img, ll_x - 1, ll_y + ll_h, 6, 3, BOOTS);
    rect(img, rl_x - 1, rl_y + rl_h, 6, 3, BOOTS);
}

fn make_player() {
    // 7 frames de 32x48, sprite sheet 224x48.
    let mut img = RgbaImage::from_pixel(224, 48, TR);

    // Frame 0 : idle
    draw_player_frame(&mut img, 0, IDLE);

    // Frames 1-4 : run cycle (legs swing + body bob)
    // Frame 1 : contact, gauche devant
    draw_player_frame(
        &mut img,
        32,
        Pose {
            body_dy: 0,
            head_dy: 0,
            left_arm_dx: 0,
            left_arm_dy: 1,
            right_arm_dx: 0,
            right_arm_dy: -1,
            left_leg_dx: -1,
            left_leg_dy: -2,
            right_leg_dx: 1,
            right_leg_dy: -1,
            arms_up: false,
        },
    );
    // Frame 2 : passage haut, gauche en bas
    draw_player_frame(
        &mut img,
        64,
        Pose {
            body_dy: -1,
            head_dy: -1,
            left_arm_dx: 0,
            left_arm_dy: 0,
            right_arm_dx: 0,
            right_arm_dy: 0,
            left_leg_dx: 0,
            left_leg_dy: 0,
            right_leg_dx: 0,
            right_leg_dy: 0,
            arms_up: false,
        },
    );
    // Frame 3 : contact, droite devant (miroir frame 1)
    draw_player_frame(
        &mut img,
        96,
        Pose {
            body_dy: 0,
            head_dy: 0,
            left_arm_dx: 0,
            left_arm_dy: -1,
            right_arm_dx: 0,
            right_arm_dy: 1,
            left_leg_dx: -1,
            left_leg_dy: -1,
            right_leg_dx: 1,
            right_leg_dy: -2,
            arms_up: false,
        },
    );
    // Frame 4 : passage haut (re)
    draw_player_frame(
        &mut img,
        128,
        Pose {
            body_dy: -1,
            head_dy: -1,
            left_arm_dx: 0,
            left_arm_dy: 0,
            right_arm_dx: 0,
            right_arm_dy: 0,
            left_leg_dx: 0,
            left_leg_dy: 0,
            right_leg_dx: 0,
            right_leg_dy: 0,
            arms_up: false,
        },
    );

    // Frame 5 : saut (corps remonté, bras levés)
    draw_player_frame(
        &mut img,
        160,
        Pose {
            body_dy: -2,
            head_dy: -2,
            left_arm_dx: 0,
            left_arm_dy: 0,
            right_arm_dx: 0,
            right_arm_dy: 0,
            left_leg_dx: 0,
            left_leg_dy: -3,
            right_leg_dx: 0,
            right_leg_dy: -3,
            arms_up: true,
        },
    );
    // Frame 6 : chute (bras écartés)
    draw_player_frame(
        &mut img,
        192,
        Pose {
            body_dy: 0,
            head_dy: 0,
            left_arm_dx: -2,
            left_arm_dy: -1,
            right_arm_dx: 2,
            right_arm_dy: -1,
            left_leg_dx: -1,
            left_leg_dy: -1,
            right_leg_dx: 1,
            right_leg_dy: -1,
            arms_up: false,
        },
    );

    save(&img, "assets/sprites/player.png");
}

// ============================================================== Tiles ===

fn make_ground_tile() {
    // 32x32 terre pure, tileable dans les deux directions. La bande
    // d'herbe est dans un sprite séparé (tile_grass.png) pour qu'elle
    // n'apparaisse qu'au sommet du sol, pas à chaque répétition Y.
    let mut img = RgbaImage::from_pixel(32, 32, DIRT);

    // Cailloux dispersés (petits clusters 2x1 pour éviter le bruit).
    for (x, y) in [
        (3, 4), (11, 2), (18, 6), (25, 3),
        (5, 12), (14, 14), (22, 11), (28, 16),
        (9, 19), (20, 20), (3, 17),
        (7, 25), (17, 27), (25, 24), (12, 29),
    ] {
        put(&mut img, x, y, DIRT_DK);
        put(&mut img, x + 1, y, DIRT_DK);
    }

    save(&img, "assets/sprites/tile_ground.png");
}

fn make_grass_strip() {
    // 32x12 : bande d'herbe avec seam de terre en bas, à poser sur
    // le sommet d'un solide. Tilable en X seulement.
    let mut img = RgbaImage::from_pixel(32, 12, TR);

    // Brins variables sur le dessus (irréguliers, font une ligne dentelée)
    let heights = [3i32, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4,
                   3, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4];
    for x in 0..32 {
        let top = heights[x as usize];
        // Pixels d'herbe : du sommet à la base (top = nb px en haut)
        for dy in 0..(8 - top.min(7)) {
            put(&mut img, x, top.min(7) + dy, GRASS);
        }
        // Pixel de surbrillance au sommet
        put(&mut img, x, top.min(7), GRASS_LT);
    }
    // Ligne sombre marquant la base de l'herbe
    hline(&mut img, 0, 8, 32, GRASS_DK);
    // 3 px d'herbe sombre pour assurer la continuité avec la terre
    rect(&mut img, 0, 9, 32, 3, GRASS_DK);

    save(&img, "assets/sprites/tile_grass.png");
}

fn make_platform_tile() {
    // 32x32 plein, conçu pour tiler proprement en X. Planches
    // horizontales avec nœuds discrets pour casser la régularité.
    let mut img = RgbaImage::from_pixel(32, 32, WOOD);

    // Bord supérieur clair (le "rebord" de la planche)
    rect(&mut img, 0, 0, 32, 2, WOOD_LT);
    // Petite ombre sous le rebord
    hline(&mut img, 0, 2, 32, WOOD);

    // Trois lignes de séparation des planches (toutes 8 px)
    hline(&mut img, 0, 8, 32, WOOD_DK);
    hline(&mut img, 0, 16, 32, WOOD_DK);
    hline(&mut img, 0, 24, 32, WOOD_DK);
    // Joint vertical à mi-tile
    rect(&mut img, 15, 0, 1, 32, WOOD_DK);

    // Nœuds du bois pour casser la régularité
    put(&mut img, 7, 5, WOOD_DK);
    put(&mut img, 23, 11, WOOD_DK);
    put(&mut img, 5, 19, WOOD_DK);
    put(&mut img, 27, 27, WOOD_DK);

    // Bord inférieur sombre
    hline(&mut img, 0, 31, 32, WOOD_DK);

    save(&img, "assets/sprites/tile_platform.png");
}

fn make_wall_tile() {
    // Mur pierre 32x32 : briques.
    let mut img = RgbaImage::from_pixel(32, 32, STONE);

    // Joints de briques (alternance ligne par ligne)
    // Rangée 1 : briques de 16
    hline(&mut img, 0, 7, 32, STONE_DK);
    rect(&mut img, 15, 0, 1, 8, STONE_DK);
    // Rangée 2 : décalée
    hline(&mut img, 0, 15, 32, STONE_DK);
    rect(&mut img, 7, 8, 1, 8, STONE_DK);
    rect(&mut img, 23, 8, 1, 8, STONE_DK);
    // Rangée 3
    hline(&mut img, 0, 23, 32, STONE_DK);
    rect(&mut img, 15, 16, 1, 8, STONE_DK);
    // Rangée 4 : décalée
    rect(&mut img, 7, 24, 1, 8, STONE_DK);
    rect(&mut img, 23, 24, 1, 8, STONE_DK);

    // Quelques éclats lumineux
    for (x, y) in [(3, 3), (19, 11), (10, 19), (26, 27)] {
        put(&mut img, x, y, Rgba([170, 170, 185, 255]));
    }

    save(&img, "assets/sprites/tile_wall.png");
}

// =========================================================== Hazards ===

fn make_spike() {
    // 32x24 : deux pointes triangulaires sur un socle sombre.
    let mut img = RgbaImage::from_pixel(32, 24, TR);

    // Socle
    rect(&mut img, 0, 20, 32, 4, SPIKE_BASE);

    // Deux triangles. Pointe à y=4, base à y=20.
    let draw_spike = |img: &mut RgbaImage, base_x: i32| {
        // Triangle : à chaque y, la largeur double.
        for row in 0..16 {
            let y = 4 + row;
            let width = row + 1;
            let x_start = base_x - width / 2 - (width % 2);
            rect(img, x_start, y, width * 2 - 1, 1, SPIKE_LT);
        }
        // Ombre verticale à droite
        for row in 4..16 {
            let y = 4 + row;
            let width = row + 1;
            let x_end = base_x + width - 1;
            put(img, x_end - 1, y, SPIKE_DK);
        }
    };

    draw_spike(&mut img, 8);
    draw_spike(&mut img, 24);

    save(&img, "assets/sprites/spike.png");
}

// ========================================================== Drapeaux ===

fn make_checkpoint() {
    // 32x64 : mât + drapeau triangulaire.
    let mut img = RgbaImage::from_pixel(32, 64, TR);

    // Socle
    rect(&mut img, 8, 60, 16, 4, STONE_DK);
    rect(&mut img, 6, 58, 20, 2, STONE);

    // Mât
    rect(&mut img, 14, 4, 3, 58, POLE);
    rect(&mut img, 14, 4, 1, 58, POLE_DK);
    // Pommeau au sommet
    rect(&mut img, 13, 2, 5, 2, CHECK_FLAG);

    // Drapeau (triangle pointant à droite)
    let flag_top = 6;
    for row in 0..16 {
        let y = flag_top + row;
        let width = if row < 8 { 14 - row } else { row - 2 };
        if width > 0 {
            rect(&mut img, 17, y, width, 1, CHECK_FLAG);
            // Ombre sur la bordure droite
            put(&mut img, 17 + width - 1, y, CHECK_FLAG_DK);
        }
    }

    save(&img, "assets/sprites/checkpoint.png");
}

fn make_goal() {
    // 48x80 : grand drapeau de fin
    let mut img = RgbaImage::from_pixel(48, 80, TR);

    // Socle large en pierre
    rect(&mut img, 10, 75, 28, 5, STONE_DK);
    rect(&mut img, 8, 72, 32, 3, STONE);

    // Mât
    rect(&mut img, 22, 6, 4, 70, POLE);
    rect(&mut img, 22, 6, 1, 70, POLE_DK);
    // Pommeau doré
    rect(&mut img, 20, 3, 8, 3, CHECK_FLAG);
    rect(&mut img, 21, 1, 6, 2, CHECK_FLAG_DK);

    // Drapeau ondulé (rectangle avec sinusoïdes simulées)
    let flag_top = 8;
    for row in 0..32 {
        let y = flag_top + row;
        // Ondulation : amplitude 2 px
        let phase = (row as f32 * 0.5).sin();
        let x_off = (phase * 2.0) as i32;
        let width = 18;
        rect(&mut img, 26 + x_off, y, width, 1, GOAL_FLAG);
        // Ombre en bas
        if row % 4 == 3 {
            rect(&mut img, 26 + x_off, y, width, 1, GOAL_FLAG_DK);
        }
        // Bordure droite plus sombre
        put(&mut img, 26 + x_off + width - 1, y, GOAL_FLAG_DK);
    }

    save(&img, "assets/sprites/goal.png");
}

// =============================================================== main ===

fn main() {
    make_player();
    make_ground_tile();
    make_grass_strip();
    make_platform_tile();
    make_wall_tile();
    make_spike();
    make_checkpoint();
    make_goal();
    println!("Assets générés dans assets/sprites/");
}
