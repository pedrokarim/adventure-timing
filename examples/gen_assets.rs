//! Générateur de sprites pixel art — direction artistique "nuit mystique"
//! (moodboard dans assets/inspirations/).
//!
//! Palette : dominantes teal/bleu nuit, silhouettes très sombres,
//! accents cyan (magie, glow) et ambre (feu, lanternes). Le perso est
//! une silhouette encapuchonnée à la Hollow Knight / Death's Door.
//!
//! Relancer après modif : `cargo run --example gen_assets`.

use image::{Rgba, RgbaImage};
use std::path::Path;

// ============================================================ Palette ===

const TR: Rgba<u8> = Rgba([0, 0, 0, 0]);

// Personnage (silhouette)
const CLOAK: Rgba<u8> = Rgba([20, 24, 36, 255]);
const CLOAK_DK: Rgba<u8> = Rgba([10, 12, 22, 255]);
const CLOAK_EDGE: Rgba<u8> = Rgba([42, 50, 70, 255]);
const SKIN: Rgba<u8> = Rgba([196, 200, 220, 255]);
const SKIN_DK: Rgba<u8> = Rgba([140, 146, 174, 255]);
const HAIR: Rgba<u8> = Rgba([232, 236, 250, 255]);
const EYE: Rgba<u8> = Rgba([8, 8, 16, 255]);

// Terre / sol — wine-pink foncé pour matcher le ciel coucher
const DIRT: Rgba<u8> = Rgba([84, 36, 60, 255]);
const DIRT_DK: Rgba<u8> = Rgba([54, 20, 42, 255]);
const DIRT_LT: Rgba<u8> = Rgba([122, 60, 86, 255]);
const MOSS: Rgba<u8> = Rgba([196, 88, 110, 255]);
const MOSS_DK: Rgba<u8> = Rgba([140, 56, 84, 255]);
const MOSS_LT: Rgba<u8> = Rgba([232, 130, 152, 255]);

// Pierre / mur — pink désaturé
const STONE: Rgba<u8> = Rgba([148, 88, 112, 255]);
const STONE_DK: Rgba<u8> = Rgba([92, 48, 70, 255]);
const STONE_LT: Rgba<u8> = Rgba([196, 132, 152, 255]);

// Accents lumineux
const CYAN: Rgba<u8> = Rgba([108, 196, 232, 255]);
const CYAN_BRIGHT: Rgba<u8> = Rgba([186, 232, 250, 255]);
const CYAN_DK: Rgba<u8> = Rgba([56, 120, 168, 255]);
const AMBER: Rgba<u8> = Rgba([232, 168, 76, 255]);
const AMBER_BRIGHT: Rgba<u8> = Rgba([248, 220, 140, 255]);
const AMBER_DK: Rgba<u8> = Rgba([148, 92, 32, 255]);

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

fn vline(img: &mut RgbaImage, x: i32, y: i32, h: i32, c: Rgba<u8>) {
    rect(img, x, y, 1, h, c);
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
// Sprite 24x36, 7 frames horizontaux → 168x36.
// Silhouette encapuchonnée. Le visage est juste un patch pâle dans la
// capuche, les cheveux blancs dépassent quand la capuche n'est pas
// relevée (idle/run). En saut/chute, la capuche se relève.

const PLAYER_FRAME_W: i32 = 24;
const PLAYER_FRAME_H: i32 = 36;

#[derive(Clone, Copy)]
struct PlayerPose {
    body_dy: i32,
    /// Décale les bords gauche/droite du bas de la cape (-1 à 2).
    cloak_flare: i32,
    /// Décale les deux pieds horizontalement (run cycle).
    left_foot_dx: i32,
    right_foot_dx: i32,
    /// Si vrai, capuche complètement relevée — pas de cheveux visibles.
    hood_up: bool,
    /// Si vrai, les bras sont écartés (chute).
    arms_out: bool,
}

const POSE_IDLE: PlayerPose = PlayerPose {
    body_dy: 0,
    cloak_flare: 0,
    left_foot_dx: 0,
    right_foot_dx: 0,
    hood_up: false,
    arms_out: false,
};

struct PlayerPalette {
    cloak: Rgba<u8>,
    cloak_dk: Rgba<u8>,
    cloak_edge: Rgba<u8>,
    skin: Rgba<u8>,
    skin_dk: Rgba<u8>,
    hair: Rgba<u8>,
    accent: Option<Rgba<u8>>,
}

const PALETTE_WANDERER: PlayerPalette = PlayerPalette {
    cloak: CLOAK,
    cloak_dk: CLOAK_DK,
    cloak_edge: CLOAK_EDGE,
    skin: SKIN,
    skin_dk: SKIN_DK,
    hair: HAIR,
    accent: Some(AMBER),
};

const PALETTE_TIGHTROPE: PlayerPalette = PlayerPalette {
    cloak: Rgba([46, 64, 132, 255]),
    cloak_dk: Rgba([22, 36, 80, 255]),
    cloak_edge: Rgba([86, 108, 188, 255]),
    skin: Rgba([238, 196, 158, 255]),
    skin_dk: Rgba([198, 154, 118, 255]),
    hair: Rgba([220, 92, 60, 255]),
    accent: Some(Rgba([252, 220, 140, 255])),
};

const PALETTE_GUARDIAN: PlayerPalette = PlayerPalette {
    cloak: Rgba([124, 116, 96, 255]),
    cloak_dk: Rgba([72, 64, 50, 255]),
    cloak_edge: Rgba([180, 168, 138, 255]),
    skin: Rgba([220, 188, 156, 255]),
    skin_dk: Rgba([158, 124, 100, 255]),
    hair: Rgba([196, 196, 188, 255]),
    accent: Some(CYAN_BRIGHT),
};

fn draw_player_frame_palette(
    img: &mut RgbaImage,
    frame_x: i32,
    pose: PlayerPose,
    p: &PlayerPalette,
) {
    let cx = frame_x + PLAYER_FRAME_W / 2;
    let head_y = 3 + pose.body_dy;

    // Capuche pointue
    rect(img, cx - 1, head_y, 2, 1, p.cloak);
    rect(img, cx - 2, head_y + 1, 4, 1, p.cloak);
    rect(img, cx - 3, head_y + 2, 6, 2, p.cloak);
    rect(img, cx - 5, head_y + 4, 10, 2, p.cloak);
    rect(img, cx - 6, head_y + 6, 12, 4, p.cloak);

    // Mèche
    if !pose.hood_up {
        rect(img, cx - 5, head_y + 9, 3, 2, p.hair);
        put(img, cx - 6, head_y + 9, p.hair);
        put(img, cx - 5, head_y + 11, p.hair);
    }

    // Visage
    let face_y = head_y + 5;
    rect(img, cx - 2, face_y, 5, 3, p.skin);
    hline(img, cx - 2, face_y, 5, p.skin_dk);
    put(img, cx - 1, face_y + 1, EYE);
    put(img, cx + 1, face_y + 1, EYE);

    // Épaules
    let shoulder_y = head_y + 10;
    rect(img, cx - 7, shoulder_y, 14, 3, p.cloak);
    vline(img, cx - 7, shoulder_y, 3, p.cloak_edge);
    hline(img, cx - 5, shoulder_y + 2, 10, p.cloak_dk);

    // Broche / rune
    if let Some(accent) = p.accent {
        put(img, cx, shoulder_y + 1, accent);
    }

    // Corps de la cape
    let body_top = shoulder_y + 3;
    let body_h = 14;
    for row in 0..body_h {
        let t = row as f32 / body_h as f32;
        let base = 12;
        let flare = (t * t * 4.0) as i32 + pose.cloak_flare;
        let width = (base + flare).min(20);
        rect(img, cx - width / 2, body_top + row, width, 1, p.cloak);
    }

    // Edge highlight gauche
    for row in 0..body_h {
        let t = row as f32 / body_h as f32;
        let base = 12;
        let flare = (t * t * 4.0) as i32 + pose.cloak_flare;
        let width = (base + flare).min(20);
        let left_x = cx - width / 2;
        put(img, left_x, body_top + row, p.cloak_edge);
    }

    // Hem
    let hem_y = body_top + body_h;
    let hem_width = (12 + 4 + pose.cloak_flare).min(20);
    rect(img, cx - hem_width / 2, hem_y, hem_width, 1, p.cloak_dk);
    put(img, cx - hem_width / 2 + 2, hem_y + 1, p.cloak_dk);
    put(img, cx - hem_width / 2 + (hem_width / 2), hem_y + 1, p.cloak_dk);
    put(img, cx - hem_width / 2 + hem_width - 3, hem_y + 1, p.cloak_dk);

    // Bras écartés
    if pose.arms_out {
        rect(img, cx - 10, body_top + 1, 2, 6, p.cloak);
        rect(img, cx + 8, body_top + 1, 2, 6, p.cloak);
        put(img, cx - 10, body_top + 7, p.skin_dk);
        put(img, cx + 9, body_top + 7, p.skin_dk);
    }

    // Pieds
    let feet_y = hem_y + 2;
    rect(img, cx - 3 + pose.left_foot_dx, feet_y, 2, 2, EYE);
    rect(img, cx + 1 + pose.right_foot_dx, feet_y, 2, 2, EYE);
}

fn draw_player_frame(img: &mut RgbaImage, frame_x: i32, pose: PlayerPose) {
    let cx = frame_x + PLAYER_FRAME_W / 2;
    let head_y = 3 + pose.body_dy;

    // === Capuche : pointe étroite qui s'élargit en cloche ===
    // Pointe (1-2 px de large)
    rect(img, cx - 1, head_y, 2, 1, CLOAK);
    rect(img, cx - 2, head_y + 1, 4, 1, CLOAK);
    // Mid-haut (4 px)
    rect(img, cx - 3, head_y + 2, 6, 2, CLOAK);
    // Couronne / élargissement
    rect(img, cx - 5, head_y + 4, 10, 2, CLOAK);
    // Capuche basse (s'évase pour englober la tête)
    rect(img, cx - 6, head_y + 6, 12, 4, CLOAK);

    // === Mèche blanche en bas de la capuche (dépasse à l'arrière) ===
    if !pose.hood_up {
        // Une touffe à l'arrière du cou + sur le côté
        rect(img, cx - 5, head_y + 9, 3, 2, HAIR);
        put(img, cx - 6, head_y + 9, HAIR);
        put(img, cx - 5, head_y + 11, HAIR);
    }

    // === Visage dans le creux de la capuche ===
    let face_y = head_y + 6;
    // Patch pâle clairement délimité
    rect(img, cx - 2, face_y, 5, 3, SKIN);
    // Ombre supérieure (l'ombre de la capuche)
    hline(img, cx - 2, face_y, 5, SKIN_DK);
    // Deux yeux clairement séparés
    put(img, cx - 1, face_y + 1, EYE);
    put(img, cx + 1, face_y + 1, EYE);

    // === Épaules : la cape s'élargit nettement ===
    let shoulder_y = head_y + 10;
    rect(img, cx - 7, shoulder_y, 14, 3, CLOAK);
    // Edge highlight gauche pour donner du volume
    vline(img, cx - 7, shoulder_y, 3, CLOAK_EDGE);
    // Trace sombre sous le col (sépare visuellement des épaules)
    hline(img, cx - 5, shoulder_y + 2, 10, CLOAK_DK);

    // === Cœur ambre (broche ou rune sur la cape) ===
    put(img, cx, shoulder_y + 1, AMBER);

    // === Corps de la cape (s'élargit en bas — flare) ===
    let body_top = shoulder_y + 3;
    let body_h = 14;
    for row in 0..body_h {
        let t = row as f32 / body_h as f32;
        let base = 12;
        let flare = (t * t * 4.0) as i32 + pose.cloak_flare;
        let width = (base + flare).min(20);
        rect(img, cx - width / 2, body_top + row, width, 1, CLOAK);
    }

    // === Edge highlight verticale sur le bord gauche du corps ===
    for row in 0..body_h {
        let t = row as f32 / body_h as f32;
        let base = 12;
        let flare = (t * t * 4.0) as i32 + pose.cloak_flare;
        let width = (base + flare).min(20);
        let left_x = cx - width / 2;
        put(img, left_x, body_top + row, CLOAK_EDGE);
    }

    // === Bord inférieur (hem sombre, irrégulier) ===
    let hem_y = body_top + body_h;
    let hem_width = (12 + 4 + pose.cloak_flare).min(20);
    rect(img, cx - hem_width / 2, hem_y, hem_width, 1, CLOAK_DK);
    // Léger débord en zig-zag (3 px qui dépassent)
    put(img, cx - hem_width / 2 + 2, hem_y + 1, CLOAK_DK);
    put(img, cx - hem_width / 2 + (hem_width / 2), hem_y + 1, CLOAK_DK);
    put(img, cx - hem_width / 2 + hem_width - 3, hem_y + 1, CLOAK_DK);

    // === Bras écartés (chute uniquement) ===
    if pose.arms_out {
        rect(img, cx - 10, body_top + 1, 2, 6, CLOAK);
        rect(img, cx + 8, body_top + 1, 2, 6, CLOAK);
        // Petites mains pâles
        put(img, cx - 10, body_top + 7, SKIN_DK);
        put(img, cx + 9, body_top + 7, SKIN_DK);
    }

    // === Pieds (2 mini-blocks sombres sous la cape) ===
    let feet_y = hem_y + 2;
    rect(img, cx - 3 + pose.left_foot_dx, feet_y, 2, 2, EYE);
    rect(img, cx + 1 + pose.right_foot_dx, feet_y, 2, 2, EYE);
}

fn make_player_with_palette(path: &str, palette: &PlayerPalette) {
    let mut img = RgbaImage::from_pixel(
        (PLAYER_FRAME_W * 7) as u32,
        PLAYER_FRAME_H as u32,
        TR,
    );

    let poses = [
        POSE_IDLE,
        PlayerPose {
            body_dy: 0,
            cloak_flare: 1,
            left_foot_dx: -1,
            right_foot_dx: 1,
            hood_up: false,
            arms_out: false,
        },
        PlayerPose {
            body_dy: -1,
            cloak_flare: 0,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: false,
            arms_out: false,
        },
        PlayerPose {
            body_dy: 0,
            cloak_flare: 1,
            left_foot_dx: 1,
            right_foot_dx: -1,
            hood_up: false,
            arms_out: false,
        },
        PlayerPose {
            body_dy: -1,
            cloak_flare: 0,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: false,
            arms_out: false,
        },
        PlayerPose {
            body_dy: -2,
            cloak_flare: -1,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: true,
            arms_out: false,
        },
        PlayerPose {
            body_dy: 0,
            cloak_flare: 3,
            left_foot_dx: -2,
            right_foot_dx: 2,
            hood_up: false,
            arms_out: true,
        },
    ];

    for (i, pose) in poses.iter().enumerate() {
        draw_player_frame_palette(&mut img, (i as i32) * PLAYER_FRAME_W, *pose, palette);
    }

    save(&img, path);
}

fn make_player() {
    make_player_with_palette("assets/sprites/player.png", &PALETTE_WANDERER);
    make_player_with_palette("assets/sprites/player_tightrope.png", &PALETTE_TIGHTROPE);
    make_player_with_palette("assets/sprites/player_guardian.png", &PALETTE_GUARDIAN);
    // Previews 1-frame pour l'écran de sélection
    make_preview("assets/sprites/preview_wanderer.png", &PALETTE_WANDERER);
    make_preview("assets/sprites/preview_tightrope.png", &PALETTE_TIGHTROPE);
    make_preview("assets/sprites/preview_guardian.png", &PALETTE_GUARDIAN);
}

fn make_preview(path: &str, palette: &PlayerPalette) {
    let mut img = RgbaImage::from_pixel(
        PLAYER_FRAME_W as u32,
        PLAYER_FRAME_H as u32,
        TR,
    );
    draw_player_frame_palette(&mut img, 0, POSE_IDLE, palette);
    save(&img, path);
}

#[allow(dead_code)]
fn make_player_old() {
    let mut img = RgbaImage::from_pixel(
        (PLAYER_FRAME_W * 7) as u32,
        PLAYER_FRAME_H as u32,
        TR,
    );

    // Frame 0 : idle
    draw_player_frame(&mut img, 0, POSE_IDLE);

    // Frames 1-4 : run cycle (la cape s'agite, les pieds alternent)
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W,
        PlayerPose {
            body_dy: 0,
            cloak_flare: 1,
            left_foot_dx: -1,
            right_foot_dx: 1,
            hood_up: false,
            arms_out: false,
        },
    );
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W * 2,
        PlayerPose {
            body_dy: -1,
            cloak_flare: 0,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: false,
            arms_out: false,
        },
    );
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W * 3,
        PlayerPose {
            body_dy: 0,
            cloak_flare: 1,
            left_foot_dx: 1,
            right_foot_dx: -1,
            hood_up: false,
            arms_out: false,
        },
    );
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W * 4,
        PlayerPose {
            body_dy: -1,
            cloak_flare: 0,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: false,
            arms_out: false,
        },
    );

    // Frame 5 : saut (capuche relevée par le vent, cape étirée)
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W * 5,
        PlayerPose {
            body_dy: -2,
            cloak_flare: -1,
            left_foot_dx: 0,
            right_foot_dx: 0,
            hood_up: true,
            arms_out: false,
        },
    );

    // Frame 6 : chute (cape déployée comme des ailes, bras écartés)
    draw_player_frame(
        &mut img,
        PLAYER_FRAME_W * 6,
        PlayerPose {
            body_dy: 0,
            cloak_flare: 3,
            left_foot_dx: -2,
            right_foot_dx: 2,
            hood_up: false,
            arms_out: true,
        },
    );

    save(&img, "assets/sprites/player.png");
}

// ============================================================ Tiles ===

fn make_ground_tile() {
    // 32x32 terre sombre, tilable XY. Subtils cailloux et lueurs cyan
    // pour suggérer des résidus magiques.
    let mut img = RgbaImage::from_pixel(32, 32, DIRT);

    // Cailloux clairs et sombres mêlés
    for (x, y, c) in [
        (3, 4, DIRT_DK), (11, 2, DIRT_DK), (18, 6, DIRT_DK), (25, 3, DIRT_LT),
        (5, 12, DIRT_LT), (14, 14, DIRT_DK), (22, 11, DIRT_DK), (28, 16, DIRT_DK),
        (9, 19, DIRT_DK), (20, 20, DIRT_LT), (3, 17, DIRT_DK),
        (7, 25, DIRT_DK), (17, 27, DIRT_LT), (25, 24, DIRT_DK), (12, 29, DIRT_DK),
    ] {
        put(&mut img, x, y, c);
        put(&mut img, x + 1, y, c);
    }
    // Quelques pixels cyan très discrets (cristaux résiduels)
    put(&mut img, 9, 8, CYAN_DK);
    put(&mut img, 23, 22, CYAN_DK);

    save(&img, "assets/sprites/tile_ground.png");
}

fn make_grass_strip() {
    // 32x10 : touffes de mousse irrégulières sur seam terre.
    // Empilée par-dessus le sol pour ne pas se répéter en Y.
    let mut img = RgbaImage::from_pixel(32, 10, TR);

    // Hauteurs aléatoires-mais-stables
    let heights = [3i32, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4,
                   3, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4];

    for x in 0..32 {
        let top = heights[x as usize].min(6);
        // Brin principal
        for dy in 0..(7 - top) {
            put(&mut img, x, top + dy, MOSS);
        }
        // Pointe lumineuse
        put(&mut img, x, top, MOSS_LT);
    }

    // Seam de terre/mousse en bas
    hline(&mut img, 0, 7, 32, MOSS_DK);
    rect(&mut img, 0, 8, 32, 2, DIRT);

    save(&img, "assets/sprites/tile_grass.png");
}

fn make_platform_tile() {
    // 32x32 : pierre taillée sombre avec joints, tile XY proprement.
    let mut img = RgbaImage::from_pixel(32, 32, STONE);

    // Rangées de pierres décalées
    // Rangée 1
    hline(&mut img, 0, 0, 32, STONE_LT);
    hline(&mut img, 0, 1, 32, STONE);
    hline(&mut img, 0, 8, 32, STONE_DK);
    vline(&mut img, 12, 0, 9, STONE_DK);
    vline(&mut img, 24, 0, 9, STONE_DK);
    // Rangée 2 (décalée)
    hline(&mut img, 0, 16, 32, STONE_DK);
    vline(&mut img, 6, 9, 8, STONE_DK);
    vline(&mut img, 18, 9, 8, STONE_DK);
    vline(&mut img, 30, 9, 8, STONE_DK);
    // Rangée 3
    hline(&mut img, 0, 24, 32, STONE_DK);
    vline(&mut img, 12, 17, 8, STONE_DK);
    vline(&mut img, 24, 17, 8, STONE_DK);
    // Rangée 4 (décalée)
    vline(&mut img, 6, 25, 7, STONE_DK);
    vline(&mut img, 18, 25, 7, STONE_DK);
    vline(&mut img, 30, 25, 7, STONE_DK);
    // Bord inférieur
    hline(&mut img, 0, 31, 32, STONE_DK);

    // Quelques pixels clairs pour vie
    put(&mut img, 4, 3, STONE_LT);
    put(&mut img, 20, 11, STONE_LT);
    put(&mut img, 8, 19, STONE_LT);
    put(&mut img, 26, 27, STONE_LT);

    save(&img, "assets/sprites/tile_platform.png");
}

fn make_wall_tile() {
    // 32x32 pierre sombre type donjon, joints plus larges.
    let mut img = RgbaImage::from_pixel(32, 32, STONE_DK);

    // Briques massives
    for row in 0..4 {
        let y = row * 8;
        let offset = if row % 2 == 0 { 0 } else { 8 };
        for col in 0..4 {
            let x = col * 8 + offset;
            if x >= 32 {
                continue;
            }
            // Corps de la brique légèrement plus clair
            rect(&mut img, x + 1, y + 1, 6, 6, STONE);
            // Highlight subtile en haut-gauche
            put(&mut img, x + 1, y + 1, STONE_LT);
        }
    }

    save(&img, "assets/sprites/tile_wall.png");
}

// =========================================================== Hazards ===

fn make_spike() {
    // 32x24 : cristal sombre avec arête cyan luminescente.
    let mut img = RgbaImage::from_pixel(32, 24, TR);

    // Socle bas (ombre du sol)
    rect(&mut img, 2, 20, 28, 4, CLOAK_DK);

    let draw_crystal = |img: &mut RgbaImage, base_x: i32| {
        // Triangle sombre
        for row in 0..16 {
            let y = 4 + row;
            let width = row + 1;
            let x = base_x - width / 2;
            rect(img, x, y, width * 2 - 1, 1, STONE);
            // Côté droit assombri
            put(img, x + width * 2 - 2, y, STONE_DK);
        }
        // Arête cyan au centre (luminescence)
        vline(img, base_x, 5, 14, CYAN);
        vline(img, base_x - 1, 6, 12, CYAN_DK);
        // Pointe extra brillante
        put(img, base_x, 4, CYAN_BRIGHT);
        put(img, base_x, 5, CYAN_BRIGHT);
    };

    draw_crystal(&mut img, 8);
    draw_crystal(&mut img, 24);

    save(&img, "assets/sprites/spike.png");
}

// ========================================================== Drapeaux ===

fn make_checkpoint() {
    // 32x64 : lanterne sur poteau. Cyan = inactif (froid), tinted en
    // ambre/vert dans le code quand activé.
    let mut img = RgbaImage::from_pixel(32, 64, TR);

    // Socle pierre
    rect(&mut img, 8, 60, 16, 4, STONE_DK);
    rect(&mut img, 6, 58, 20, 2, STONE);
    // Highlight socle
    hline(&mut img, 8, 58, 16, STONE_LT);

    // Mât (bois ou métal sombre)
    rect(&mut img, 14, 18, 4, 42, CLOAK);
    // Edge clair
    vline(&mut img, 14, 18, 42, CLOAK_EDGE);

    // Bras horizontal (où pend la lanterne)
    rect(&mut img, 11, 14, 10, 2, CLOAK);
    put(&mut img, 11, 14, CLOAK_EDGE);

    // Chaîne (3 px verticaux entre le bras et la lanterne)
    vline(&mut img, 16, 16, 2, CLOAK_DK);

    // Lanterne (cadre sombre)
    rect(&mut img, 11, 18, 10, 12, CLOAK);
    rect(&mut img, 10, 19, 1, 10, CLOAK);
    rect(&mut img, 21, 19, 1, 10, CLOAK);
    // Toit de la lanterne
    rect(&mut img, 10, 17, 12, 1, CLOAK_DK);

    // Vitre (cristal cyan luminescent à l'intérieur)
    rect(&mut img, 13, 20, 6, 8, CYAN_DK);
    rect(&mut img, 14, 21, 4, 6, CYAN);
    rect(&mut img, 15, 22, 2, 4, CYAN_BRIGHT);
    // Point central très brillant
    put(&mut img, 15, 23, AMBER_BRIGHT);
    put(&mut img, 16, 23, AMBER_BRIGHT);

    // Halo cyan autour de la lanterne (quelques pixels semi-transparents)
    let halo = Rgba([108, 196, 232, 80]);
    put(&mut img, 9, 23, halo);
    put(&mut img, 22, 23, halo);
    put(&mut img, 15, 16, halo);
    put(&mut img, 16, 30, halo);

    save(&img, "assets/sprites/checkpoint.png");
}

fn make_goal() {
    // 48x80 : torii sombre avec cristal géant suspendu au centre, halo
    // ambre. C'est la fin du niveau : doit pop.
    let mut img = RgbaImage::from_pixel(48, 80, TR);

    // Socle large
    rect(&mut img, 8, 75, 32, 5, STONE_DK);
    rect(&mut img, 6, 72, 36, 3, STONE);
    hline(&mut img, 8, 72, 32, STONE_LT);

    // Deux piliers
    rect(&mut img, 8, 16, 5, 56, CLOAK);
    rect(&mut img, 35, 16, 5, 56, CLOAK);
    // Edge highlights
    vline(&mut img, 8, 16, 56, CLOAK_EDGE);
    vline(&mut img, 35, 16, 56, CLOAK_EDGE);

    // Linteau supérieur (torii)
    rect(&mut img, 4, 12, 40, 4, CLOAK);
    rect(&mut img, 2, 14, 44, 2, CLOAK_DK);
    // Liseré clair
    hline(&mut img, 4, 12, 40, CLOAK_EDGE);

    // Linteau inférieur (plus petit)
    rect(&mut img, 10, 22, 28, 3, CLOAK);

    // Cristal central suspendu (rotation diamant, gros)
    let cx = 24;
    let crystal_y = 32;
    // Diamant 12 wide x 16 tall centré
    for row in 0..16 {
        let half_width = if row < 8 { row + 1 } else { 16 - row };
        let y = crystal_y + row;
        rect(&mut img, cx - half_width, y, half_width * 2, 1, AMBER_DK);
    }
    // Cœur du cristal (plus brillant)
    for row in 0..12 {
        let half_width = if row < 6 { row + 1 } else { 12 - row };
        let y = crystal_y + 2 + row;
        rect(&mut img, cx - half_width, y, half_width * 2, 1, AMBER);
    }
    // Highlight blanc
    rect(&mut img, cx - 2, crystal_y + 5, 1, 6, AMBER_BRIGHT);
    put(&mut img, cx - 1, crystal_y + 4, AMBER_BRIGHT);
    put(&mut img, cx, crystal_y + 4, AMBER_BRIGHT);

    // Halo (pixels semi-transparents tout autour)
    let halo = Rgba([232, 168, 76, 100]);
    let halo_dim = Rgba([232, 168, 76, 50]);
    for (dx, dy) in [
        (-6, 0), (6, 0), (0, -8), (0, 12),
        (-5, -5), (5, -5), (-5, 5), (5, 5),
    ] {
        put(&mut img, cx + dx, crystal_y + 8 + dy, halo);
    }
    for (dx, dy) in [
        (-9, 0), (9, 0), (-7, -7), (7, -7), (-7, 7), (7, 7),
        (0, -10), (0, 14),
    ] {
        put(&mut img, cx + dx, crystal_y + 8 + dy, halo_dim);
    }

    save(&img, "assets/sprites/goal.png");
}

// ====================================================== Parallax bg ===

fn make_parallax_back() {
    // 512x320 : montagnes roses lointaines + nuages puffies. Ambiance
    // coucher de soleil appliquée au gameplay. Tilable en X.
    let mut img = RgbaImage::from_pixel(512, 320, TR);

    // Pétales / poussières en suspension dans la haute couche
    for (x, y) in [
        (45, 20), (110, 40), (180, 30), (250, 25),
        (310, 45), (380, 22), (440, 35), (480, 50),
        (60, 60), (200, 70), (350, 65),
    ] {
        put(&mut img, x, y, Rgba([232, 168, 178, 220]));
        put(&mut img, x + 1, y, Rgba([220, 140, 156, 220]));
    }

    // Nuages puffies clairs
    let draw_cloud = |img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>| {
        for blob in 0..10 {
            let angle = (blob as f32) * 0.52;
            let dist = ((blob as f32) * 0.7).sin().abs() * (r as f32 * 0.65);
            let bx = cx + (angle.cos() * dist) as i32;
            let by = cy + (angle.sin() * dist * 0.35) as i32;
            let br = (r as f32 * (0.5 + (blob as f32 * 0.3).sin().abs() * 0.5)) as i32;
            for dy in -br..=br {
                for dx in -br..=br {
                    if dx * dx + dy * dy <= br * br {
                        put(img, bx + dx, by + dy, color);
                    }
                }
            }
        }
    };
    let cloud = Rgba([244, 196, 200, 255]);
    draw_cloud(&mut img, 80, 130, 50, cloud);
    draw_cloud(&mut img, 240, 110, 60, cloud);
    draw_cloud(&mut img, 380, 140, 55, cloud);
    draw_cloud(&mut img, 470, 100, 40, cloud);

    // Montagnes lointaines (rose clair)
    let peaks_y = 220;
    let peaks = [
        (0, 0), (40, 40), (80, 20), (120, 55), (160, 30),
        (200, 50), (240, 15), (280, 45), (320, 30),
        (360, 60), (400, 20), (440, 40), (472, 25), (512, 38),
    ];
    let mountain = Rgba([222, 132, 144, 255]);
    for w in peaks.windows(2) {
        let (x0, h0) = w[0];
        let (x1, h1) = w[1];
        let dx = x1 - x0;
        for px in 0..dx {
            let t = px as f32 / dx as f32;
            let h = (h0 as f32 * (1.0 - t) + h1 as f32 * t) as i32;
            rect(&mut img, x0 + px, peaks_y - h, 1, h + (320 - peaks_y), mountain);
        }
    }

    save(&img, "assets/sprites/parallax_back.png");
}

fn make_parallax_mid() {
    // 512x260 : montagnes intermédiaires en rose moyen.
    let mut img = RgbaImage::from_pixel(512, 260, TR);

    let peaks_y = 140;
    let peaks = [
        (0, 30), (60, 70), (130, 35), (200, 85), (270, 45),
        (340, 75), (410, 30), (470, 65), (512, 40),
    ];
    let color = Rgba([180, 92, 116, 255]);
    for w in peaks.windows(2) {
        let (x0, h0) = w[0];
        let (x1, h1) = w[1];
        let dx = x1 - x0;
        for px in 0..dx {
            let t = px as f32 / dx as f32;
            let h = (h0 as f32 * (1.0 - t) + h1 as f32 * t) as i32;
            rect(&mut img, x0 + px, peaks_y - h, 1, h + (260 - peaks_y), color);
        }
    }

    save(&img, "assets/sprites/parallax_mid.png");
}

fn make_menu_background() {
    // 1280x720 : vibe coucher de soleil rose pour l'écran-titre.
    // - Ciel rose en gradient
    // - Deux gros amas de nuages roses pâles (gauche + droite)
    // - Trois monolithes verticaux suspendus dans le ciel, semi-transparents
    // - Anneau lumineux (soleil/lune) au centre
    // - Trois couches de montagnes en dégradé de roses
    // - Colline sombre en bas à gauche avec un mini-perso + compagnon
    // - Pylône à droite avec fils tombant
    // - Pétales éparses concentrées au centre
    let w = 1280u32;
    let h = 720u32;
    let mut img = RgbaImage::new(w, h);

    // ===== Couleurs =====
    let sky_top = (246.0, 188.0, 198.0);
    let sky_bot = (228.0, 130.0, 156.0);
    let cloud_lt = Rgba([252, 218, 218, 255]);
    let cloud_mid = Rgba([248, 198, 200, 255]);
    let cloud_edge = Rgba([240, 174, 188, 255]);
    let mountain_far = Rgba([222, 130, 144, 255]);
    let mountain_mid = Rgba([196, 100, 122, 255]);
    let mountain_near = Rgba([158, 74, 98, 255]);
    let ground_dark = Rgba([88, 36, 60, 255]);
    let ground_light = Rgba([110, 50, 76, 255]);
    let silhouette = Rgba([38, 14, 30, 255]);
    let monolith = Rgba([214, 124, 144, 230]);
    let monolith_edge = Rgba([192, 92, 122, 230]);
    let ring = Rgba([252, 220, 210, 255]);

    // ===== Ciel gradient =====
    for y in 0..h {
        let t = y as f32 / h as f32;
        let r = sky_top.0 * (1.0 - t) + sky_bot.0 * t;
        let g = sky_top.1 * (1.0 - t) + sky_bot.1 * t;
        let b = sky_top.2 * (1.0 - t) + sky_bot.2 * t;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r as u8, g as u8, b as u8, 255]));
        }
    }

    // ===== Monolithes flottants (3, suspendus depuis le haut, fade vers le bas) =====
    let monoliths = [
        (430i32, 0i32, 36i32, 430i32),
        (720, 0, 50, 520),
        (1080, 0, 26, 380),
    ];
    for (x, y_top, mw, mh) in monoliths {
        // Corps principal avec fade alpha vers le bas
        for row in 0..mh {
            let t = row as f32 / mh as f32;
            let alpha = ((1.0 - t * t) * 230.0) as u8;
            let c = Rgba([monolith.0[0], monolith.0[1], monolith.0[2], alpha]);
            rect(&mut img, x, y_top + row, mw, 1, c);
        }
        // Edge sombre à droite
        for row in 0..mh {
            let t = row as f32 / mh as f32;
            let alpha = ((1.0 - t * t) * 230.0) as u8;
            let c = Rgba([
                monolith_edge.0[0],
                monolith_edge.0[1],
                monolith_edge.0[2],
                alpha,
            ]);
            put(&mut img, x + mw - 1, y_top + row, c);
            put(&mut img, x + mw - 2, y_top + row, c);
        }
    }

    // ===== Anneau lumineux au centre (juste un cercle outline) =====
    let ring_cx = 640i32;
    let ring_cy = 240i32;
    let ring_r = 22i32;
    for theta_i in 0..240 {
        let theta = theta_i as f32 / 240.0 * std::f32::consts::TAU;
        let x = ring_cx + (theta.cos() * ring_r as f32) as i32;
        let y = ring_cy + (theta.sin() * ring_r as f32) as i32;
        put(&mut img, x, y, ring);
        put(&mut img, x + 1, y, ring);
    }

    // ===== Nuages : deux masses puffies (gauche + droite) =====
    // Approche : on dessine plein de cercles flous superposés
    let draw_cloud = |img: &mut RgbaImage, cx: i32, cy: i32, max_r: i32, color: Rgba<u8>| {
        for blob_i in 0..12 {
            let blob_angle = (blob_i as f32) * 0.52;
            let dist = ((blob_i as f32) * 0.7).sin().abs() * (max_r as f32 * 0.7);
            let bx = cx + (blob_angle.cos() * dist) as i32;
            let by = cy + (blob_angle.sin() * dist * 0.4) as i32;
            let br = (max_r as f32 * (0.55 + (blob_i as f32 * 0.31).sin().abs() * 0.45)) as i32;
            for dy in -br..=br {
                for dx in -br..=br {
                    if dx * dx + dy * dy <= br * br {
                        put(img, bx + dx, by + dy, color);
                    }
                }
            }
        }
    };

    // Nuage gauche
    draw_cloud(&mut img, 240, 320, 130, cloud_lt);
    draw_cloud(&mut img, 180, 380, 110, cloud_mid);
    draw_cloud(&mut img, 80, 360, 90, cloud_edge);
    // Nuage droite
    draw_cloud(&mut img, 1050, 340, 140, cloud_lt);
    draw_cloud(&mut img, 1150, 410, 100, cloud_mid);
    draw_cloud(&mut img, 980, 280, 80, cloud_edge);
    // Nuage central bas (camoufle un peu les monolithes)
    draw_cloud(&mut img, 720, 460, 130, cloud_mid);
    draw_cloud(&mut img, 600, 480, 100, cloud_edge);

    // ===== Couches de montagnes =====
    let mut fill_mountain = |img: &mut RgbaImage, peaks: &[(i32, i32)], color: Rgba<u8>| {
        for w in peaks.windows(2) {
            let (x0, y0) = w[0];
            let (x1, y1) = w[1];
            let dx = x1 - x0;
            if dx <= 0 {
                continue;
            }
            for px in 0..dx {
                let t = px as f32 / dx as f32;
                let y_top = (y0 as f32 * (1.0 - t) + y1 as f32 * t) as i32;
                rect(img, x0 + px, y_top, 1, 720 - y_top, color);
            }
        }
    };

    let far_peaks = [
        (0i32, 510i32), (140, 490), (280, 520), (420, 485),
        (560, 510), (700, 478), (840, 505), (980, 485),
        (1120, 510), (1280, 495),
    ];
    fill_mountain(&mut img, &far_peaks, mountain_far);

    let mid_peaks = [
        (0i32, 555i32), (130, 520), (260, 555), (400, 515),
        (540, 545), (680, 510), (820, 540), (960, 520),
        (1100, 555), (1280, 530),
    ];
    fill_mountain(&mut img, &mid_peaks, mountain_mid);

    let near_peaks = [
        (0i32, 605i32), (110, 580), (230, 615), (360, 575),
        (500, 605), (640, 580), (780, 615), (920, 590),
        (1060, 615), (1200, 585), (1280, 605),
    ];
    fill_mountain(&mut img, &near_peaks, mountain_near);

    // ===== Sol foncé tout en bas =====
    rect(&mut img, 0, 645, 1280, 75, ground_dark);
    // Touffes d'herbe en silhouette
    for x in (0..1280).step_by(2) {
        let height = ((x as f32 * 0.09).sin().abs() * 3.0 + 1.0) as i32;
        rect(&mut img, x, 645 - height, 1, height, ground_light);
    }

    // ===== Colline en silhouette à gauche (sur laquelle est le perso) =====
    // Une bosse douce qui monte de y=645 jusqu'à y~610
    for x in 0..420 {
        let t = x as f32 / 420.0;
        let bump = (1.0 - (t - 0.5).abs() * 1.6).max(0.0);
        let y_top = 645 - (bump * 38.0) as i32;
        rect(&mut img, x, y_top, 1, 720 - y_top, ground_dark);
        // Brins d'herbe sur le dessus
        let blade_h = ((x as f32 * 0.21).sin().abs() * 4.0 + 1.0) as i32;
        rect(&mut img, x, y_top - blade_h, 1, blade_h, ground_light);
    }

    // ===== Mini-personnage encapuchonné + compagnon =====
    // Position : sommet de la colline
    let hill_top_x = 210;
    let hill_top_y = 618;
    // Perso (silhouette compacte)
    rect(&mut img, hill_top_x, hill_top_y - 11, 5, 11, silhouette);
    rect(&mut img, hill_top_x + 1, hill_top_y - 13, 3, 2, silhouette);
    // Compagnon (chat/chien à droite, plus petit)
    rect(&mut img, hill_top_x + 8, hill_top_y - 4, 6, 4, silhouette);
    rect(&mut img, hill_top_x + 13, hill_top_y - 5, 2, 1, silhouette);
    rect(&mut img, hill_top_x + 8, hill_top_y - 5, 2, 2, silhouette);

    // ===== Pylône / antenne à droite =====
    let post_x = 1140;
    rect(&mut img, post_x, 530, 3, 115, silhouette);
    // Bras horizontaux
    rect(&mut img, post_x - 6, 542, 16, 2, silhouette);
    rect(&mut img, post_x - 4, 552, 12, 2, silhouette);
    // Fils qui descendent vers la gauche, hors-cadre
    for x in 200..post_x {
        let t = (x - 200) as f32 / (post_x - 200) as f32;
        let sag = (t * std::f32::consts::PI).sin();
        let y = 543 - (sag * 6.0) as i32 + ((1.0 - t) * 20.0) as i32;
        put(&mut img, x, y, Rgba([38, 14, 30, 180]));
    }

    // ===== Pétales tombantes (concentrées au centre, comme l'image) =====
    for i in 0..60 {
        let phi = (i as f32) * 0.61803 * std::f32::consts::TAU;
        let x = (640.0 + phi.cos() * 320.0 + ((i as f32 * 3.1).sin() * 80.0)) as i32;
        let y = (440.0 + phi.sin() * 180.0 + ((i as f32 * 1.7).cos() * 40.0)) as i32;
        let color = if i % 3 == 0 {
            Rgba([200, 80, 110, 255])
        } else if i % 3 == 1 {
            Rgba([232, 120, 140, 255])
        } else {
            Rgba([252, 196, 200, 240])
        };
        // Pétale = 2 px côte à côte
        put(&mut img, x, y, color);
        put(&mut img, x + 1, y, color);
    }

    save(&img, "assets/sprites/menu_background.png");
}

fn make_parallax_front() {
    // 512x180 : montagnes proches en wine sombre + pylônes silhouettes
    // sous le ciel rose.
    let mut img = RgbaImage::from_pixel(512, 180, TR);

    let ground_y = 110;
    let color = Rgba([108, 50, 78, 255]);

    // Sol ondulé
    let ground_dy = [0i32, 2, 4, 3, 1, 2, 5, 3, 1, 0, 2, 4, 3, 1, 2, 5,
                     3, 1, 0, 2, 4, 3, 1, 2, 5, 3, 1, 0, 2, 4, 3, 1];
    for x in 0..512 {
        let dy = ground_dy[(x as usize) % ground_dy.len()];
        rect(&mut img, x, ground_y + dy, 1, 180 - ground_y - dy, color);
    }

    // Pylônes / poteaux silhouettés (au lieu des arbres)
    let posts = [60, 180, 320, 430];
    for &px in &posts {
        rect(&mut img, px, ground_y - 22, 2, 22, color);
        rect(&mut img, px - 5, ground_y - 18, 12, 2, color);
        rect(&mut img, px - 4, ground_y - 11, 10, 2, color);
    }

    // Brins d'herbe par-dessus la silhouette
    for x in (0..512).step_by(3) {
        let h = ((x as f32 * 0.12).sin().abs() * 4.0 + 1.0) as i32;
        let dy = ground_dy[(x as usize) % ground_dy.len()];
        rect(&mut img, x, ground_y + dy - h, 1, h, Rgba([148, 72, 100, 255]));
    }

    save(&img, "assets/sprites/parallax_front.png");
}

// =============================================================== main ===

// ============================================================ Items ===

fn make_items() {
    // 16x16 chacun.

    // Cristal cyan : losange brillant
    let mut crystal = RgbaImage::from_pixel(16, 16, TR);
    for row in 0..14 {
        let half = if row < 7 { row + 1 } else { 14 - row };
        let y = 1 + row;
        rect(&mut crystal, 8 - half, y, half * 2, 1, CYAN_DK);
    }
    // cœur clair au centre
    for row in 0..8 {
        let half = if row < 4 { row + 1 } else { 8 - row };
        let y = 4 + row;
        rect(&mut crystal, 8 - half, y, half * 2, 1, CYAN);
    }
    put(&mut crystal, 7, 6, CYAN_BRIGHT);
    put(&mut crystal, 8, 6, CYAN_BRIGHT);
    put(&mut crystal, 7, 7, CYAN_BRIGHT);
    save(&crystal, "assets/sprites/item_crystal.png");

    // Pétale d'ambre
    let mut petal = RgbaImage::from_pixel(16, 16, TR);
    // forme de goutte inversée
    for row in 0..12 {
        let t = row as f32 / 12.0;
        let half = ((1.0 - (t - 0.5).abs() * 1.4) * 6.0) as i32;
        if half > 0 {
            let y = 2 + row;
            rect(&mut petal, 8 - half, y, half * 2, 1, AMBER);
        }
    }
    // highlight clair
    put(&mut petal, 7, 5, AMBER_BRIGHT);
    put(&mut petal, 8, 5, AMBER_BRIGHT);
    put(&mut petal, 7, 6, AMBER_BRIGHT);
    // base sombre
    put(&mut petal, 7, 13, AMBER_DK);
    put(&mut petal, 8, 13, AMBER_DK);
    save(&petal, "assets/sprites/item_petal.png");

    // Plume blanche : forme allongée
    let mut feather = RgbaImage::from_pixel(16, 16, TR);
    // Hampe diagonale
    for i in 0..12 {
        put(&mut feather, 4 + i / 3, 2 + i, HAIR);
    }
    // Barbes (lignes obliques)
    for i in 0..6 {
        let y = 4 + i * 2;
        rect(&mut feather, 5 + i / 2, y, 4 - i / 3, 1, SKIN);
        rect(&mut feather, 2 + i / 2, y + 1, 4 - i / 3, 1, SKIN);
    }
    save(&feather, "assets/sprites/item_feather.png");

    // Cœur 16x16 (icône d'item, gros)
    let mut heart = RgbaImage::from_pixel(16, 16, TR);
    let heart_red = Rgba([232, 56, 72, 255]);
    let heart_dk = Rgba([148, 28, 36, 255]);
    let heart_lt = Rgba([252, 132, 140, 255]);
    // Forme cœur (lobes + pointe)
    rect(&mut heart, 2, 3, 5, 5, heart_red);
    rect(&mut heart, 9, 3, 5, 5, heart_red);
    rect(&mut heart, 3, 8, 10, 2, heart_red);
    rect(&mut heart, 4, 10, 8, 1, heart_red);
    rect(&mut heart, 5, 11, 6, 1, heart_red);
    rect(&mut heart, 6, 12, 4, 1, heart_red);
    rect(&mut heart, 7, 13, 2, 1, heart_red);
    // Outline
    rect(&mut heart, 1, 4, 1, 4, heart_dk);
    rect(&mut heart, 14, 4, 1, 4, heart_dk);
    rect(&mut heart, 2, 2, 5, 1, heart_dk);
    rect(&mut heart, 9, 2, 5, 1, heart_dk);
    // Highlight
    put(&mut heart, 4, 4, heart_lt);
    put(&mut heart, 5, 4, heart_lt);
    put(&mut heart, 4, 5, heart_lt);
    save(&heart, "assets/sprites/item_heart.png");

    // Pétale mémoire 16x16 — pétale violet/rose flottante
    let mut mem = RgbaImage::from_pixel(16, 16, TR);
    let petal_lt = Rgba([232, 140, 220, 255]);
    let petal_mid = Rgba([180, 92, 168, 255]);
    let petal_dk = Rgba([108, 48, 100, 255]);
    // Pétale en goutte
    for row in 0..12 {
        let t = row as f32 / 12.0;
        let half = ((1.0 - (t - 0.5).abs() * 1.4) * 6.0) as i32;
        if half > 0 {
            let y = 2 + row;
            rect(&mut mem, 8 - half, y, half * 2, 1, petal_mid);
        }
    }
    put(&mut mem, 7, 5, petal_lt);
    put(&mut mem, 8, 5, petal_lt);
    put(&mut mem, 7, 13, petal_dk);
    put(&mut mem, 8, 13, petal_dk);
    // Petite étoile au centre (mémoire / résurrection)
    put(&mut mem, 7, 8, HAIR);
    put(&mut mem, 8, 8, HAIR);
    save(&mem, "assets/sprites/item_memory.png");

    // Sablier : deux triangles connectés
    let mut hourglass = RgbaImage::from_pixel(16, 16, TR);
    // Cadres
    hline(&mut hourglass, 3, 1, 10, AMBER_DK);
    hline(&mut hourglass, 3, 14, 10, AMBER_DK);
    // Triangle haut (rempli rose pâle)
    for row in 0..6 {
        let half = 5 - row;
        let y = 2 + row;
        rect(&mut hourglass, 8 - half, y, half * 2, 1, AMBER);
    }
    // Triangle bas
    for row in 0..6 {
        let half = row + 1;
        let y = 8 + row;
        rect(&mut hourglass, 8 - half, y, half * 2, 1, AMBER);
    }
    // Goutte au centre (sable qui tombe)
    rect(&mut hourglass, 7, 7, 2, 1, AMBER_BRIGHT);
    // Highlight cadre
    put(&mut hourglass, 3, 1, AMBER_BRIGHT);
    put(&mut hourglass, 12, 14, AMBER_BRIGHT);
    save(&hourglass, "assets/sprites/item_hourglass.png");
}

// =================================================== Palette niveau ===

#[derive(Clone, Copy)]
struct LevelPalette {
    suffix: &'static str,
    dirt: Rgba<u8>,
    dirt_dk: Rgba<u8>,
    dirt_lt: Rgba<u8>,
    moss: Rgba<u8>,
    moss_dk: Rgba<u8>,
    moss_lt: Rgba<u8>,
    stone: Rgba<u8>,
    stone_dk: Rgba<u8>,
    stone_lt: Rgba<u8>,
    accent: Rgba<u8>,
    sky_high: Rgba<u8>,
    sky_low: Rgba<u8>,
    mountain_far: Rgba<u8>,
    mountain_mid: Rgba<u8>,
    mountain_near: Rgba<u8>,
}

const PALETTE_AMBER: LevelPalette = LevelPalette {
    suffix: "_amber",
    dirt: Rgba([60, 36, 18, 255]),
    dirt_dk: Rgba([36, 20, 12, 255]),
    dirt_lt: Rgba([100, 64, 30, 255]),
    moss: Rgba([170, 96, 30, 255]),
    moss_dk: Rgba([122, 64, 18, 255]),
    moss_lt: Rgba([232, 168, 76, 255]),
    stone: Rgba([108, 76, 40, 255]),
    stone_dk: Rgba([56, 36, 14, 255]),
    stone_lt: Rgba([170, 124, 70, 255]),
    accent: Rgba([252, 220, 140, 255]),
    sky_high: Rgba([76, 38, 20, 255]),
    sky_low: Rgba([180, 92, 32, 255]),
    mountain_far: Rgba([124, 64, 28, 255]),
    mountain_mid: Rgba([94, 46, 18, 255]),
    mountain_near: Rgba([56, 30, 14, 255]),
};

const PALETTE_SANCTUARY: LevelPalette = LevelPalette {
    suffix: "_sanctuary",
    dirt: Rgba([22, 14, 18, 255]),
    dirt_dk: Rgba([10, 6, 10, 255]),
    dirt_lt: Rgba([46, 28, 36, 255]),
    moss: Rgba([78, 24, 32, 255]),
    moss_dk: Rgba([42, 12, 18, 255]),
    moss_lt: Rgba([148, 36, 48, 255]),
    stone: Rgba([42, 30, 38, 255]),
    stone_dk: Rgba([20, 14, 22, 255]),
    stone_lt: Rgba([72, 52, 64, 255]),
    accent: Rgba([220, 64, 80, 255]),
    sky_high: Rgba([8, 6, 12, 255]),
    sky_low: Rgba([24, 12, 18, 255]),
    mountain_far: Rgba([28, 18, 26, 255]),
    mountain_mid: Rgba([18, 12, 18, 255]),
    mountain_near: Rgba([8, 4, 8, 255]),
};

const PALETTE_DAWN: LevelPalette = LevelPalette {
    suffix: "_dawn",
    dirt: Rgba([110, 110, 116, 255]),
    dirt_dk: Rgba([60, 60, 66, 255]),
    dirt_lt: Rgba([164, 164, 168, 255]),
    moss: Rgba([180, 180, 188, 255]),
    moss_dk: Rgba([130, 130, 138, 255]),
    moss_lt: Rgba([220, 220, 226, 255]),
    stone: Rgba([138, 138, 146, 255]),
    stone_dk: Rgba([78, 78, 84, 255]),
    stone_lt: Rgba([200, 200, 206, 255]),
    accent: Rgba([255, 255, 255, 255]),
    sky_high: Rgba([222, 222, 228, 255]),
    sky_low: Rgba([158, 158, 168, 255]),
    mountain_far: Rgba([142, 142, 150, 255]),
    mountain_mid: Rgba([108, 108, 118, 255]),
    mountain_near: Rgba([70, 70, 80, 255]),
};

fn make_palette_tiles(p: &LevelPalette) {
    // Ground
    let mut img = RgbaImage::from_pixel(32, 32, p.dirt);
    for (x, y, c) in [
        (3, 4, p.dirt_dk), (11, 2, p.dirt_dk), (18, 6, p.dirt_lt), (25, 3, p.dirt_lt),
        (5, 12, p.dirt_lt), (14, 14, p.dirt_dk), (22, 11, p.dirt_dk), (28, 16, p.dirt_lt),
        (9, 19, p.dirt_dk), (20, 20, p.dirt_lt), (3, 17, p.dirt_dk),
        (7, 25, p.dirt_dk), (17, 27, p.dirt_lt), (25, 24, p.dirt_dk), (12, 29, p.dirt_dk),
    ] {
        put(&mut img, x, y, c);
        put(&mut img, x + 1, y, c);
    }
    put(&mut img, 9, 8, p.accent);
    put(&mut img, 23, 22, p.accent);
    save(&img, &format!("assets/sprites/tile_ground{}.png", p.suffix));

    // Grass strip
    let mut grass = RgbaImage::from_pixel(32, 10, TR);
    let heights = [3i32, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4,
                   3, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4];
    for x in 0..32 {
        let top = heights[x as usize].min(6);
        for dy in 0..(7 - top) {
            put(&mut grass, x, top + dy, p.moss);
        }
        put(&mut grass, x, top, p.moss_lt);
    }
    hline(&mut grass, 0, 7, 32, p.moss_dk);
    rect(&mut grass, 0, 8, 32, 2, p.dirt);
    save(&grass, &format!("assets/sprites/tile_grass{}.png", p.suffix));

    // Platform (briques)
    let mut platform = RgbaImage::from_pixel(32, 32, p.stone);
    hline(&mut platform, 0, 0, 32, p.stone_lt);
    hline(&mut platform, 0, 1, 32, p.stone);
    hline(&mut platform, 0, 8, 32, p.stone_dk);
    vline(&mut platform, 12, 0, 9, p.stone_dk);
    vline(&mut platform, 24, 0, 9, p.stone_dk);
    hline(&mut platform, 0, 16, 32, p.stone_dk);
    vline(&mut platform, 6, 9, 8, p.stone_dk);
    vline(&mut platform, 18, 9, 8, p.stone_dk);
    vline(&mut platform, 30, 9, 8, p.stone_dk);
    hline(&mut platform, 0, 24, 32, p.stone_dk);
    vline(&mut platform, 12, 17, 8, p.stone_dk);
    vline(&mut platform, 24, 17, 8, p.stone_dk);
    vline(&mut platform, 6, 25, 7, p.stone_dk);
    vline(&mut platform, 18, 25, 7, p.stone_dk);
    vline(&mut platform, 30, 25, 7, p.stone_dk);
    hline(&mut platform, 0, 31, 32, p.stone_dk);
    put(&mut platform, 4, 3, p.stone_lt);
    put(&mut platform, 20, 11, p.stone_lt);
    put(&mut platform, 8, 19, p.stone_lt);
    put(&mut platform, 26, 27, p.stone_lt);
    save(&platform, &format!("assets/sprites/tile_platform{}.png", p.suffix));

    // Wall
    let mut wall = RgbaImage::from_pixel(32, 32, p.stone_dk);
    for row in 0..4 {
        let y = row * 8;
        let offset = if row % 2 == 0 { 0 } else { 8 };
        for col in 0..4 {
            let x = col * 8 + offset;
            if x >= 32 { continue; }
            rect(&mut wall, x + 1, y + 1, 6, 6, p.stone);
            put(&mut wall, x + 1, y + 1, p.stone_lt);
        }
    }
    save(&wall, &format!("assets/sprites/tile_wall{}.png", p.suffix));
}

fn make_palette_parallax(p: &LevelPalette) {
    // Back : ciel gradient + montagnes lointaines
    let mut back = RgbaImage::new(512, 320);
    for y in 0..320 {
        let t = y as f32 / 320.0;
        let r = p.sky_high.0[0] as f32 * (1.0 - t) + p.sky_low.0[0] as f32 * t;
        let g = p.sky_high.0[1] as f32 * (1.0 - t) + p.sky_low.0[1] as f32 * t;
        let b = p.sky_high.0[2] as f32 * (1.0 - t) + p.sky_low.0[2] as f32 * t;
        for x in 0..512 {
            back.put_pixel(x, y, Rgba([r as u8, g as u8, b as u8, 255]));
        }
    }
    // Étoiles / particules
    for i in 0..30 {
        let x = ((i as f32 * 137.5) as i32) % 510 + 1;
        let y = ((i as f32 * 71.3) as i32) % 200 + 10;
        put(&mut back, x, y, p.accent);
    }
    // Crêtes lointaines
    let far_peaks = [
        (0i32, 200i32), (40, 230), (80, 210), (120, 250), (160, 220),
        (200, 240), (240, 200), (280, 230), (320, 215), (360, 250),
        (400, 210), (440, 235), (472, 220), (512, 240),
    ];
    for w in far_peaks.windows(2) {
        let (x0, y0) = w[0]; let (x1, y1) = w[1];
        let dx = x1 - x0;
        for px in 0..dx {
            let t = px as f32 / dx as f32;
            let y_top = (y0 as f32 * (1.0 - t) + y1 as f32 * t) as i32;
            rect(&mut back, x0 + px, y_top, 1, 320 - y_top, p.mountain_far);
        }
    }
    save(&back, &format!("assets/sprites/parallax_back{}.png", p.suffix));

    // Mid
    let mut mid = RgbaImage::from_pixel(512, 260, TR);
    let mid_peaks = [
        (0i32, 130i32), (60, 175), (130, 130), (200, 180), (270, 140),
        (340, 175), (410, 130), (470, 170), (512, 140),
    ];
    for w in mid_peaks.windows(2) {
        let (x0, y0) = w[0]; let (x1, y1) = w[1];
        let dx = x1 - x0;
        for px in 0..dx {
            let t = px as f32 / dx as f32;
            let y_top = (y0 as f32 * (1.0 - t) + y1 as f32 * t) as i32;
            rect(&mut mid, x0 + px, y_top, 1, 260 - y_top, p.mountain_mid);
        }
    }
    save(&mid, &format!("assets/sprites/parallax_mid{}.png", p.suffix));

    // Front
    let mut front = RgbaImage::from_pixel(512, 180, TR);
    rect(&mut front, 0, 110, 512, 70, p.mountain_near);
    // Brins
    for x in (0..512).step_by(3) {
        let h = ((x as f32 * 0.12).sin().abs() * 4.0 + 1.0) as i32;
        rect(&mut front, x, 110 - h, 1, h, p.moss);
    }
    save(&front, &format!("assets/sprites/parallax_front{}.png", p.suffix));
}

// =================================================== Forest tiles ===

const FOREST_DIRT: Rgba<u8> = Rgba([24, 36, 46, 255]);
const FOREST_DIRT_DK: Rgba<u8> = Rgba([12, 22, 30, 255]);
const FOREST_DIRT_LT: Rgba<u8> = Rgba([46, 62, 78, 255]);
const FOREST_MOSS: Rgba<u8> = Rgba([40, 88, 90, 255]);
const FOREST_MOSS_DK: Rgba<u8> = Rgba([20, 56, 60, 255]);
const FOREST_MOSS_LT: Rgba<u8> = Rgba([78, 144, 138, 255]);
const FOREST_STONE: Rgba<u8> = Rgba([40, 60, 78, 255]);
const FOREST_STONE_DK: Rgba<u8> = Rgba([18, 30, 44, 255]);
const FOREST_STONE_LT: Rgba<u8> = Rgba([64, 96, 116, 255]);
const FOREST_TREE_BLUE: Rgba<u8> = Rgba([56, 60, 218, 255]);
const FOREST_TREE_BLUE_DK: Rgba<u8> = Rgba([30, 32, 156, 255]);

fn make_forest_tiles() {
    // Ground 32x32 wine sombre teinté bleu nuit + petits éclats cyan
    let mut img = RgbaImage::from_pixel(32, 32, FOREST_DIRT);
    for (x, y, c) in [
        (3, 4, FOREST_DIRT_DK), (11, 2, FOREST_DIRT_DK), (18, 6, FOREST_DIRT_LT),
        (25, 3, FOREST_DIRT_LT), (5, 12, FOREST_DIRT_DK), (14, 14, FOREST_DIRT_LT),
        (22, 11, FOREST_DIRT_DK), (28, 16, FOREST_DIRT_LT),
        (9, 19, FOREST_DIRT_DK), (20, 20, FOREST_DIRT_LT), (3, 17, FOREST_DIRT_DK),
        (7, 25, FOREST_DIRT_DK), (17, 27, FOREST_DIRT_LT),
        (25, 24, FOREST_DIRT_DK), (12, 29, FOREST_DIRT_DK),
    ] {
        put(&mut img, x, y, c);
        put(&mut img, x + 1, y, c);
    }
    put(&mut img, 9, 8, CYAN);
    put(&mut img, 23, 22, CYAN_DK);
    save(&img, "assets/sprites/tile_ground_forest.png");

    // Grass strip 32x10 : mousse teal éparse
    let mut grass = RgbaImage::from_pixel(32, 10, TR);
    let heights = [3i32, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4,
                   3, 5, 2, 4, 6, 3, 5, 4, 2, 5, 3, 4, 6, 3, 5, 4];
    for x in 0..32 {
        let top = heights[x as usize].min(6);
        for dy in 0..(7 - top) {
            put(&mut grass, x, top + dy, FOREST_MOSS);
        }
        put(&mut grass, x, top, FOREST_MOSS_LT);
    }
    hline(&mut grass, 0, 7, 32, FOREST_MOSS_DK);
    rect(&mut grass, 0, 8, 32, 2, FOREST_DIRT);
    save(&grass, "assets/sprites/tile_grass_forest.png");

    // Platform 32x32 : pierre noir-bleu type donjon
    let mut platform = RgbaImage::from_pixel(32, 32, FOREST_STONE);
    hline(&mut platform, 0, 0, 32, FOREST_STONE_LT);
    hline(&mut platform, 0, 1, 32, FOREST_STONE);
    hline(&mut platform, 0, 8, 32, FOREST_STONE_DK);
    vline(&mut platform, 12, 0, 9, FOREST_STONE_DK);
    vline(&mut platform, 24, 0, 9, FOREST_STONE_DK);
    hline(&mut platform, 0, 16, 32, FOREST_STONE_DK);
    vline(&mut platform, 6, 9, 8, FOREST_STONE_DK);
    vline(&mut platform, 18, 9, 8, FOREST_STONE_DK);
    vline(&mut platform, 30, 9, 8, FOREST_STONE_DK);
    hline(&mut platform, 0, 24, 32, FOREST_STONE_DK);
    vline(&mut platform, 12, 17, 8, FOREST_STONE_DK);
    vline(&mut platform, 24, 17, 8, FOREST_STONE_DK);
    vline(&mut platform, 6, 25, 7, FOREST_STONE_DK);
    vline(&mut platform, 18, 25, 7, FOREST_STONE_DK);
    vline(&mut platform, 30, 25, 7, FOREST_STONE_DK);
    hline(&mut platform, 0, 31, 32, FOREST_STONE_DK);
    put(&mut platform, 4, 3, FOREST_STONE_LT);
    put(&mut platform, 20, 11, FOREST_STONE_LT);
    put(&mut platform, 8, 19, FOREST_STONE_LT);
    put(&mut platform, 26, 27, FOREST_STONE_LT);
    save(&platform, "assets/sprites/tile_platform_forest.png");

    // Wall 32x32 — briques noires bleutées
    let mut wall = RgbaImage::from_pixel(32, 32, FOREST_STONE_DK);
    for row in 0..4 {
        let y = row * 8;
        let offset = if row % 2 == 0 { 0 } else { 8 };
        for col in 0..4 {
            let x = col * 8 + offset;
            if x >= 32 { continue; }
            rect(&mut wall, x + 1, y + 1, 6, 6, FOREST_STONE);
            put(&mut wall, x + 1, y + 1, FOREST_STONE_LT);
        }
    }
    save(&wall, "assets/sprites/tile_wall_forest.png");
}

fn make_forest_parallax() {
    // Back : ciel nuit + grand arbre bleu central
    let mut back = RgbaImage::from_pixel(512, 320, Rgba([18, 28, 40, 255]));
    // Étoiles
    for (x, y) in [
        (30i32, 20i32), (80, 40), (140, 30), (200, 25), (260, 45),
        (320, 22), (380, 35), (440, 50), (490, 25),
        (60, 80), (200, 90), (350, 75), (450, 85),
        (110, 130), (300, 120), (480, 130),
    ] {
        put(&mut back, x, y, HAIR);
        put(&mut back, x + 1, y, Rgba([200, 220, 240, 200]));
    }
    // Petits points cyan rares (cristaux)
    put(&mut back, 90, 70, CYAN);
    put(&mut back, 250, 110, CYAN);
    put(&mut back, 410, 80, CYAN);

    // Grand arbre stylisé au centre (silhouette)
    let trunk_x = 256i32;
    rect(&mut back, trunk_x - 12, 220, 24, 100, Rgba([22, 30, 38, 255]));
    rect(&mut back, trunk_x - 8, 200, 16, 20, Rgba([22, 30, 38, 255]));
    // Boules de feuillage bleu
    let leaf_centers = [
        (trunk_x - 60, 180, 32),
        (trunk_x + 50, 160, 38),
        (trunk_x - 20, 140, 36),
        (trunk_x + 70, 130, 30),
        (trunk_x - 80, 130, 28),
        (trunk_x + 20, 100, 34),
    ];
    for (cx, cy, r) in leaf_centers {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    put(&mut back, cx + dx, cy + dy, FOREST_TREE_BLUE);
                }
            }
        }
    }
    // Highlights
    for (cx, cy, _) in leaf_centers {
        put(&mut back, cx - 8, cy - 8, FOREST_TREE_BLUE);
        put(&mut back, cx - 4, cy - 10, Rgba([90, 100, 240, 255]));
    }
    save(&back, "assets/sprites/parallax_back_forest.png");

    // Mid : silhouettes d'arbres lointains
    let mut mid = RgbaImage::from_pixel(512, 260, TR);
    let tree_xs = [40, 110, 180, 260, 330, 400, 470];
    for &tx in &tree_xs {
        rect(&mut mid, tx, 160, 6, 100, Rgba([10, 18, 26, 255]));
        // Cime
        for row in 0..40 {
            let half = (20.0 - row as f32 * 0.4) as i32;
            if half > 0 {
                let y = 120 + row;
                rect(&mut mid, tx - half + 3, y, half * 2, 1, Rgba([14, 24, 34, 255]));
            }
        }
    }
    save(&mid, "assets/sprites/parallax_mid_forest.png");

    // Front : sol foncé + brins de mousse + quelques rochers
    let mut front = RgbaImage::from_pixel(512, 180, TR);
    rect(&mut front, 0, 110, 512, 70, Rgba([10, 14, 18, 255]));
    // Touffes
    for x in (0..512).step_by(3) {
        let h = ((x as f32 * 0.12).sin().abs() * 4.0 + 1.0) as i32;
        rect(&mut front, x, 110 - h, 1, h, FOREST_MOSS);
    }
    // Rochers
    for &rx in &[60, 200, 340, 460] {
        rect(&mut front, rx, 102, 18, 8, Rgba([18, 26, 36, 255]));
        rect(&mut front, rx + 2, 100, 14, 2, Rgba([18, 26, 36, 255]));
    }
    save(&front, "assets/sprites/parallax_front_forest.png");
}

// ========================================================= Enemies ===

fn make_wraith() {
    // 20x28 : silhouette spectrale drapée avec yeux rouges
    let mut img = RgbaImage::from_pixel(20, 28, TR);
    let body = Rgba([180, 196, 220, 200]);
    let body_dk = Rgba([120, 138, 168, 220]);
    let body_lt = Rgba([220, 232, 248, 220]);
    let eye = Rgba([232, 48, 48, 255]);

    for row in 0..8 {
        let half = match row {
            0 => 3, 1 => 5, 2 | 3 => 6,
            _ => 7,
        };
        let y = row;
        rect(&mut img, 10 - half, y, half * 2, 1, body);
    }
    put(&mut img, 7, 4, eye);
    put(&mut img, 12, 4, eye);
    for row in 0..18 {
        let t = row as f32 / 18.0;
        let half = (7.0 + t * 2.0) as i32;
        let y = 8 + row;
        rect(&mut img, 10 - half, y, half * 2, 1, body);
    }
    for x in 0..20 {
        let h = ((x as f32 * 1.2).sin().abs() * 2.0) as i32;
        rect(&mut img, x, 26 - h, 1, h + 2, body_dk);
    }
    hline(&mut img, 7, 1, 6, body_lt);
    save(&img, "assets/sprites/enemy_wraith.png");
}

fn make_charger() {
    // 22x18 : boule cornue, taureau miniature qui fonce
    let mut img = RgbaImage::from_pixel(22, 18, TR);
    // Corps rond
    for row in 0..12 {
        let half = match row {
            0 | 11 => 4,
            1 | 10 => 6,
            2 | 9 => 7,
            _ => 8,
        };
        let y = 3 + row;
        rect(&mut img, 11 - half, y, half * 2, 1, Rgba([84, 42, 32, 255]));
    }
    // Highlights
    hline(&mut img, 6, 4, 10, Rgba([130, 70, 56, 255]));
    // Cornes
    rect(&mut img, 3, 1, 2, 4, Rgba([196, 188, 174, 255]));
    rect(&mut img, 17, 1, 2, 4, Rgba([196, 188, 174, 255]));
    put(&mut img, 4, 0, Rgba([220, 212, 200, 255]));
    put(&mut img, 17, 0, Rgba([220, 212, 200, 255]));
    // Yeux rouges
    put(&mut img, 7, 7, Rgba([232, 48, 48, 255]));
    put(&mut img, 14, 7, Rgba([232, 48, 48, 255]));
    // Naseaux
    put(&mut img, 10, 11, EYE);
    put(&mut img, 11, 11, EYE);
    // Pattes
    rect(&mut img, 4, 15, 4, 3, Rgba([42, 18, 14, 255]));
    rect(&mut img, 14, 15, 4, 3, Rgba([42, 18, 14, 255]));
    save(&img, "assets/sprites/enemy_charger.png");
}

fn make_spitter_and_projectile() {
    // Spitter 24x20 : champignon trapu avec bouche au centre
    let mut img = RgbaImage::from_pixel(24, 20, TR);
    // Chapeau
    for row in 0..8 {
        let half = match row {
            0 | 7 => 5,
            1 | 6 => 8,
            _ => 10,
        };
        let y = row;
        rect(&mut img, 12 - half, y, half * 2, 1, AMBER_DK);
    }
    // Highlight
    hline(&mut img, 6, 1, 12, AMBER);
    // Tronc
    rect(&mut img, 9, 8, 6, 8, CLOAK);
    rect(&mut img, 8, 9, 1, 6, CLOAK);
    rect(&mut img, 15, 9, 1, 6, CLOAK);
    // Bouche (s'ouvre pour cracher)
    rect(&mut img, 10, 11, 4, 3, EYE);
    put(&mut img, 11, 12, AMBER_BRIGHT);
    // Yeux
    put(&mut img, 9, 5, AMBER_BRIGHT);
    put(&mut img, 14, 5, AMBER_BRIGHT);
    // Pieds
    rect(&mut img, 8, 16, 8, 4, CLOAK_DK);
    save(&img, "assets/sprites/enemy_spitter.png");

    // Projectile 12x12 : sphère ambre brûlante
    let mut proj = RgbaImage::from_pixel(12, 12, TR);
    for row in 0..10 {
        let half = match row {
            0 | 9 => 2,
            1 | 8 => 3,
            _ => 4,
        };
        let y = 1 + row;
        rect(&mut proj, 6 - half, y, half * 2, 1, AMBER_DK);
    }
    rect(&mut proj, 4, 4, 4, 4, AMBER);
    put(&mut proj, 5, 5, AMBER_BRIGHT);
    put(&mut proj, 6, 5, AMBER_BRIGHT);
    save(&proj, "assets/sprites/enemy_projectile.png");
}

fn make_enemies() {
    // Crawler 20x14 : petite araignée/insecte sombre avec yeux cyan
    let mut crawler = RgbaImage::from_pixel(20, 14, TR);
    // Corps oval
    for row in 0..8 {
        let half = match row {
            0 | 7 => 4,
            1 | 6 => 6,
            _ => 7,
        };
        let y = 3 + row;
        rect(&mut crawler, 10 - half, y, half * 2, 1, CLOAK);
    }
    // Highlight dorsal
    hline(&mut crawler, 6, 4, 8, CLOAK_EDGE);
    put(&mut crawler, 9, 3, CLOAK_EDGE);
    put(&mut crawler, 10, 3, CLOAK_EDGE);
    // Yeux cyan
    put(&mut crawler, 13, 6, CYAN_BRIGHT);
    put(&mut crawler, 15, 6, CYAN_BRIGHT);
    put(&mut crawler, 13, 7, CYAN);
    put(&mut crawler, 15, 7, CYAN);
    // Pattes
    for x in [2, 5, 14, 17].iter() {
        put(&mut crawler, *x, 11, CLOAK);
        put(&mut crawler, *x, 12, CLOAK);
    }
    save(&crawler, "assets/sprites/enemy_crawler.png");

    // Flyer 18x18 : œil flottant avec ailes
    let mut flyer = RgbaImage::from_pixel(18, 18, TR);
    // Corps central rond
    for row in 0..12 {
        let half = match row {
            0 | 11 => 3,
            1 | 10 => 5,
            2 | 9 => 6,
            _ => 7,
        };
        let y = 3 + row;
        rect(&mut flyer, 9 - half, y, half * 2, 1, CLOAK);
    }
    // Pupille cyan
    rect(&mut flyer, 6, 7, 6, 4, EYE);
    put(&mut flyer, 7, 8, CYAN_BRIGHT);
    put(&mut flyer, 9, 8, CYAN_BRIGHT);
    put(&mut flyer, 8, 9, CYAN);
    put(&mut flyer, 10, 9, CYAN);
    // Ailes (rectangles latéraux semi-transparents)
    let wing = Rgba([46, 110, 148, 180]);
    rect(&mut flyer, 0, 6, 3, 5, wing);
    rect(&mut flyer, 15, 6, 3, 5, wing);
    put(&mut flyer, 0, 5, wing);
    put(&mut flyer, 17, 5, wing);
    save(&flyer, "assets/sprites/enemy_flyer.png");
}

// ====================================================== Projectiles ===

fn make_arrow() {
    // 20x6 : flèche en bois avec pointe et fletching
    let mut img = RgbaImage::from_pixel(20, 6, TR);
    let wood = Rgba([180, 100, 50, 255]);
    let metal = Rgba([180, 180, 184, 255]);
    let feather = Rgba([232, 168, 76, 255]);
    // Corps
    rect(&mut img, 4, 2, 12, 2, wood);
    // Pointe
    rect(&mut img, 16, 2, 3, 2, metal);
    put(&mut img, 19, 3, metal);
    // Fletching
    rect(&mut img, 0, 1, 4, 4, feather);
    put(&mut img, 0, 0, feather);
    put(&mut img, 0, 5, feather);
    save(&img, "assets/sprites/projectile_arrow.png");
}

fn make_projectile_magic() {
    // 18x10 : éclair de magie cyan avec halo
    let mut img = RgbaImage::from_pixel(18, 10, TR);
    // Core elongé
    rect(&mut img, 4, 4, 12, 2, CYAN_BRIGHT);
    rect(&mut img, 3, 3, 14, 4, CYAN);
    rect(&mut img, 2, 4, 16, 2, CYAN);
    // Pointe avant (côté droit)
    rect(&mut img, 16, 4, 1, 2, CYAN_BRIGHT);
    put(&mut img, 17, 5, CYAN);
    // Queue (côté gauche)
    rect(&mut img, 0, 4, 2, 2, CYAN_DK);
    // Halo subtil
    let halo = Rgba([108, 196, 232, 120]);
    rect(&mut img, 3, 1, 12, 1, halo);
    rect(&mut img, 3, 8, 12, 1, halo);
    save(&img, "assets/sprites/projectile_magic.png");
}

// ====================================================== Throwables ===

fn make_full_throwables() {
    // Boomerang 14x14 : X stylisé en bois
    let mut boom = RgbaImage::from_pixel(14, 14, TR);
    let wood = Rgba([180, 100, 50, 255]);
    let wood_dk = Rgba([108, 56, 22, 255]);
    let wood_lt = Rgba([232, 168, 76, 255]);
    // Branche \\
    for i in 0..10 {
        put(&mut boom, 2 + i, 2 + i, wood);
        put(&mut boom, 3 + i, 2 + i, wood);
    }
    // Branche //
    for i in 0..10 {
        put(&mut boom, 11 - i, 2 + i, wood);
        put(&mut boom, 10 - i, 2 + i, wood);
    }
    put(&mut boom, 6, 6, wood_lt);
    put(&mut boom, 7, 6, wood_lt);
    put(&mut boom, 6, 7, wood_dk);
    put(&mut boom, 7, 7, wood_dk);
    save(&boom, "assets/sprites/throwable_boomerang.png");

    // Bouclier 8x64 : mur vertical translucide cyan
    let mut shield = RgbaImage::from_pixel(8, 64, TR);
    rect(&mut shield, 0, 0, 8, 64, Rgba([130, 200, 240, 130]));
    rect(&mut shield, 1, 0, 6, 64, Rgba([180, 224, 248, 100]));
    vline(&mut shield, 0, 0, 64, Rgba([130, 200, 240, 220]));
    vline(&mut shield, 7, 0, 64, Rgba([130, 200, 240, 220]));
    save(&shield, "assets/sprites/throwable_shield.png");

    // Piège 28x8 : ligne dentée au sol
    let mut trap = RgbaImage::from_pixel(28, 8, TR);
    rect(&mut trap, 0, 4, 28, 4, Rgba([60, 30, 22, 255]));
    // Dents pointues
    for i in 0..7 {
        let x = i * 4;
        rect(&mut trap, x, 2, 1, 2, Rgba([180, 180, 184, 255]));
        rect(&mut trap, x + 1, 1, 1, 1, Rgba([220, 220, 224, 255]));
        rect(&mut trap, x + 2, 2, 1, 2, Rgba([180, 180, 184, 255]));
    }
    save(&trap, "assets/sprites/throwable_trap.png");

    // Marqueur 16x24 : drapeau triangulaire ambre sur mât
    let mut marker = RgbaImage::from_pixel(16, 24, TR);
    rect(&mut marker, 7, 0, 2, 24, Rgba([60, 36, 20, 255]));
    rect(&mut marker, 6, 23, 4, 1, Rgba([60, 36, 20, 255]));
    // Drapeau triangulaire
    for row in 0..8 {
        let width = 8 - row;
        rect(&mut marker, 9, 2 + row, width, 1, Rgba([232, 168, 76, 255]));
    }
    save(&marker, "assets/sprites/throwable_marker.png");

    // Tourelle 18x22 : pied stable + tube qui sort
    let mut turret = RgbaImage::from_pixel(18, 22, TR);
    // Pied
    rect(&mut turret, 3, 18, 12, 4, Rgba([60, 60, 70, 255]));
    rect(&mut turret, 4, 17, 10, 1, Rgba([100, 100, 110, 255]));
    // Base ronde
    rect(&mut turret, 5, 10, 8, 7, Rgba([80, 80, 90, 255]));
    rect(&mut turret, 6, 8, 6, 2, Rgba([100, 100, 110, 255]));
    // Tube canon (horizontal)
    rect(&mut turret, 8, 12, 10, 3, Rgba([60, 60, 70, 255]));
    put(&mut turret, 17, 13, Rgba([232, 168, 76, 255])); // sortie ambre
    // Détail rouge (caméra ?)
    put(&mut turret, 8, 11, Rgba([220, 60, 60, 255]));
    put(&mut turret, 9, 11, Rgba([220, 60, 60, 255]));
    save(&turret, "assets/sprites/throwable_turret.png");
}

fn make_extra_throwables() {
    // Caillou 10x10 : petit galet gris irrégulier
    let mut rock = RgbaImage::from_pixel(10, 10, TR);
    let rock_dk = Rgba([62, 56, 48, 255]);
    let rock_mid = Rgba([108, 100, 88, 255]);
    let rock_lt = Rgba([160, 152, 138, 255]);
    for (x, y) in [
        (3i32, 1i32), (4, 1), (5, 1), (6, 1),
        (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2),
        (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3),
        (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), (8, 4),
        (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5),
        (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6),
        (3, 7), (4, 7), (5, 7), (6, 7),
    ] {
        put(&mut rock, x, y, rock_mid);
    }
    // Highlight
    put(&mut rock, 3, 2, rock_lt);
    put(&mut rock, 4, 2, rock_lt);
    put(&mut rock, 2, 3, rock_lt);
    // Shadow
    put(&mut rock, 6, 6, rock_dk);
    put(&mut rock, 7, 5, rock_dk);
    put(&mut rock, 5, 7, rock_dk);
    save(&rock, "assets/sprites/throwable_rock.png");

    // Torche 14x22 : manche bois + flamme
    let mut torch = RgbaImage::from_pixel(14, 22, TR);
    let handle_dk = Rgba([56, 36, 20, 255]);
    let handle = Rgba([108, 72, 40, 255]);
    let flame_lt = Rgba([252, 220, 140, 255]);
    let flame = Rgba([232, 168, 76, 255]);
    let flame_dk = Rgba([164, 92, 32, 255]);
    // Manche
    rect(&mut torch, 5, 10, 4, 12, handle);
    vline(&mut torch, 5, 10, 12, handle_dk);
    hline(&mut torch, 5, 21, 4, handle_dk);
    // Coupelle de retenue
    rect(&mut torch, 3, 9, 8, 2, handle_dk);
    rect(&mut torch, 4, 8, 6, 1, handle);
    // Flamme (forme triangulaire)
    for row in 0..8 {
        let half = match row {
            0 => 1, 1 | 2 => 2, 3 | 4 => 3, 5 => 2, 6 => 2, _ => 1,
        };
        let color = if row < 3 { flame_lt } else if row < 6 { flame } else { flame_dk };
        let y = row;
        rect(&mut torch, 7 - half, y, half * 2, 1, color);
    }
    // Cœur très clair
    put(&mut torch, 6, 3, Rgba([255, 248, 224, 255]));
    put(&mut torch, 7, 3, Rgba([255, 248, 224, 255]));
    save(&torch, "assets/sprites/throwable_torch.png");
}

fn make_throwables() {
    // Bombe 12x12 : sphère noire avec mèche
    let mut bomb = RgbaImage::from_pixel(12, 12, TR);
    for row in 0..10 {
        let half = match row {
            0 | 9 => 2,
            1 | 8 => 3,
            _ => 4,
        };
        let y = 1 + row;
        rect(&mut bomb, 6 - half, y, half * 2, 1, EYE);
    }
    // Highlight
    put(&mut bomb, 4, 3, Rgba([90, 100, 130, 255]));
    put(&mut bomb, 5, 3, Rgba([90, 100, 130, 255]));
    // Mèche
    put(&mut bomb, 6, 0, AMBER);
    put(&mut bomb, 7, 1, AMBER_BRIGHT);
    save(&bomb, "assets/sprites/throwable_bomb.png");

    // Bloc de glace 32x16 : carreau cyan translucide
    let mut ice = RgbaImage::from_pixel(32, 16, TR);
    rect(&mut ice, 0, 0, 32, 16, CYAN_DK);
    rect(&mut ice, 1, 1, 30, 14, CYAN);
    rect(&mut ice, 2, 2, 28, 12, Rgba([196, 232, 248, 200]));
    // Reflets diagonaux
    for i in 0..6 {
        put(&mut ice, 4 + i, 3 + i, CYAN_BRIGHT);
    }
    for i in 0..4 {
        put(&mut ice, 20 + i, 4 + i, CYAN_BRIGHT);
    }
    // Bordure plus claire
    hline(&mut ice, 0, 0, 32, CYAN_BRIGHT);
    save(&ice, "assets/sprites/throwable_ice.png");

    // Plateforme magique 48x6 : barre cyan glowing
    let mut platform = RgbaImage::from_pixel(48, 6, TR);
    rect(&mut platform, 0, 0, 48, 6, CYAN_DK);
    rect(&mut platform, 0, 1, 48, 4, CYAN);
    rect(&mut platform, 0, 2, 48, 2, CYAN_BRIGHT);
    // Effet stries
    for i in (0..48).step_by(4) {
        put(&mut platform, i, 2, Rgba([232, 244, 252, 255]));
    }
    save(&platform, "assets/sprites/throwable_platform.png");
}

fn main() {
    make_player();
    make_ground_tile();
    make_grass_strip();
    make_platform_tile();
    make_wall_tile();
    make_spike();
    make_checkpoint();
    make_goal();
    make_parallax_back();
    make_parallax_mid();
    make_parallax_front();
    make_menu_background();
    make_items();
    make_throwables();
    make_extra_throwables();
    make_full_throwables();
    make_projectile_magic();
    make_arrow();
    make_enemies();
    make_spitter_and_projectile();
    make_charger();
    make_wraith();
    make_forest_tiles();
    make_forest_parallax();
    for palette in [&PALETTE_AMBER, &PALETTE_SANCTUARY, &PALETTE_DAWN] {
        make_palette_tiles(palette);
        make_palette_parallax(palette);
    }
    println!("Assets générés dans assets/sprites/");
}
