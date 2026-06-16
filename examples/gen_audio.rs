//! Générateur de SFX WAV procéduraux. Aucune dépendance audio dans le
//! binaire du jeu — on synthétise les sons à la main puis on les sauve
//! comme WAV mono 22 kHz / 16-bit dans `assets/sfx/`.
//!
//! Relance après modif : `cargo run --example gen_audio`.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::f32::consts::TAU;
use std::path::Path;

const SAMPLE_RATE: u32 = 22050;

fn spec() -> WavSpec {
    WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    }
}

fn save(path: &str, samples: &[f32]) {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut writer = WavWriter::create(p, spec()).unwrap();
    for &s in samples {
        // Clamp + conversion en i16
        let clamped = s.clamp(-1.0, 1.0);
        writer
            .write_sample((clamped * i16::MAX as f32) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();
    println!("écrit {path}");
}

/// Génère N samples à partir d'un closure `(t, t_normalized) → amplitude`.
fn synth(duration_s: f32, f: impl Fn(f32, f32) -> f32) -> Vec<f32> {
    let total = (duration_s * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let t = i as f32 / SAMPLE_RATE as f32;
        let t_norm = i as f32 / total as f32;
        out.push(f(t, t_norm));
    }
    out
}

/// Bruit blanc pseudo-aléatoire stable basé sur l'index.
fn noise(t: f32, t_norm: f32) -> f32 {
    let x = (t * 99371.0 + t_norm * 1373.0).sin() * 43758.5453;
    (x - x.floor()) * 2.0 - 1.0
}

/// Enveloppe exponentielle (attack rapide, decay exponentiel).
fn env_pluck(t_norm: f32, decay: f32) -> f32 {
    let attack = (t_norm / 0.02).min(1.0);
    let dec = (-decay * t_norm).exp();
    attack * dec
}

/// Enveloppe avec sustain (pour drapeaux/wins).
fn env_chord(t_norm: f32) -> f32 {
    let attack = (t_norm / 0.05).min(1.0);
    let release = if t_norm > 0.7 {
        (1.0 - (t_norm - 0.7) / 0.3).max(0.0)
    } else {
        1.0
    };
    attack * release
}

// ====================================================== sons ===

/// Saut : balayage rapide montant (250 → 700 Hz).
fn make_jump() -> Vec<f32> {
    synth(0.18, |t, t_norm| {
        let freq = 250.0 + 450.0 * t_norm;
        let phase = TAU * freq * t;
        let saw = (phase.sin() * 0.6) + (phase * 2.0).sin() * 0.2;
        saw * env_pluck(t_norm, 6.0) * 0.65
    })
}

/// Atterrissage : impact bref, basses fréquences + bruit court.
fn make_land() -> Vec<f32> {
    synth(0.16, |t, t_norm| {
        let freq = 110.0 * (1.0 - t_norm * 0.4);
        let body = (TAU * freq * t).sin() * 0.7;
        let crunch = noise(t, t_norm) * (0.4 - t_norm).max(0.0);
        (body + crunch) * env_pluck(t_norm, 12.0) * 0.7
    })
}

/// Mort : balayage descendant dissonant, légèrement vibré.
fn make_death() -> Vec<f32> {
    synth(0.55, |t, t_norm| {
        let freq = 320.0 * (1.0 - t_norm * 0.85);
        let vibrato = (TAU * 7.0 * t).sin() * 8.0;
        let phase = TAU * (freq + vibrato) * t;
        let body = phase.sin() * 0.5 + (phase * 1.5).sin() * 0.3;
        let attack = (t_norm / 0.05).min(1.0);
        let decay = (1.0 - t_norm).powf(1.3);
        body * attack * decay * 0.7
    })
}

/// Checkpoint : cloche pure (440 + 880 Hz), legère reverb par envelope.
fn make_checkpoint() -> Vec<f32> {
    synth(0.45, |t, t_norm| {
        let fundamental = (TAU * 880.0 * t).sin() * 0.55;
        let overtone = (TAU * 1320.0 * t).sin() * 0.20;
        let sub = (TAU * 440.0 * t).sin() * 0.25;
        (fundamental + overtone + sub) * env_pluck(t_norm, 3.5) * 0.55
    })
}

/// Win : accord majeur C-E-G qui s'épanouit.
fn make_win() -> Vec<f32> {
    synth(0.95, |t, t_norm| {
        let c = (TAU * 523.25 * t).sin() * 0.32;
        let e = (TAU * 659.25 * t).sin() * 0.28;
        let g = (TAU * 783.99 * t).sin() * 0.28;
        let c_oct = (TAU * 1046.50 * t).sin() * 0.18;
        // Petite modulation d'amplitude pour un effet de "shimmer"
        let shimmer = 1.0 + 0.06 * (TAU * 5.0 * t).sin();
        (c + e + g + c_oct) * env_chord(t_norm) * shimmer * 0.55
    })
}

// ============================================================ main ===

fn main() {
    save("assets/sfx/jump.wav", &make_jump());
    save("assets/sfx/land.wav", &make_land());
    save("assets/sfx/death.wav", &make_death());
    save("assets/sfx/checkpoint.wav", &make_checkpoint());
    save("assets/sfx/win.wav", &make_win());
    println!("SFX générés dans assets/sfx/");
}
