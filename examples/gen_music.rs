//! Générateur de boucles musicales WAV procédurales par niveau.
//! Mélodie pentatonique mineure + drone basse + pad d'accords.
//!
//! Relance après modif : `cargo run --example gen_music`

use hound::{SampleFormat, WavSpec, WavWriter};
use std::f32::consts::TAU;
use std::path::Path;

const SAMPLE_RATE: u32 = 44100;

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
        let clamped = s.clamp(-0.95, 0.95);
        writer
            .write_sample((clamped * i16::MAX as f32) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();
    println!("écrit {path}");
}

/// Mix N pistes en somme normalisée.
fn mix(tracks: &[&[f32]]) -> Vec<f32> {
    let n = tracks.iter().map(|t| t.len()).max().unwrap_or(0);
    let mut out = vec![0.0_f32; n];
    for track in tracks {
        for (i, &s) in track.iter().enumerate() {
            if i < n {
                out[i] += s;
            }
        }
    }
    out
}

/// Génère une basse en sine + slight detune (warmth).
fn synth_bass(duration_s: f32, root_freq: f32, level: f32) -> Vec<f32> {
    let total = (duration_s * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(total);
    let f = root_freq * 0.5;
    for i in 0..total {
        let t = i as f32 / SAMPLE_RATE as f32;
        let core = (TAU * f * t).sin();
        let warmth = (TAU * f * 1.004 * t).sin();
        // Léger volume swell
        let swell = 0.85 + 0.15 * (TAU * t / duration_s).sin();
        out.push((core * 0.6 + warmth * 0.4) * level * swell);
    }
    out
}

/// Mélodie : notes successives selon un pattern, enveloppe pluck.
fn synth_melody(
    duration_s: f32,
    bpm: f32,
    root_freq: f32,
    scale: &[f32],
    pattern: &[usize],
    level: f32,
) -> Vec<f32> {
    let total = (duration_s * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0_f32; total];
    let beat = 60.0 / bpm;
    let note_dur = beat * 0.5; // croches
    let note_samples = (note_dur * SAMPLE_RATE as f32) as usize;
    let mut note_idx = 0;
    let mut pos = 0;
    while pos < total {
        let scale_step = pattern[note_idx % pattern.len()];
        let freq = root_freq * scale[scale_step % scale.len()];
        // L'octave saute parfois (vibe gamelan/clochette)
        let freq = if note_idx % 7 == 6 { freq * 2.0 } else { freq };
        for i in 0..note_samples {
            if pos + i >= total {
                break;
            }
            let local_t = i as f32 / SAMPLE_RATE as f32;
            let env = (-local_t * 5.0).exp() * (local_t * 80.0).min(1.0);
            let osc = (TAU * freq * local_t).sin();
            let harmonic = (TAU * freq * 2.0 * local_t).sin() * 0.25;
            out[pos + i] += (osc + harmonic) * env * level;
        }
        pos += note_samples;
        note_idx += 1;
    }
    out
}

/// Pad d'accords : sustained avec attack/release.
fn synth_pad(
    duration_s: f32,
    bpm: f32,
    root_freq: f32,
    scale: &[f32],
    chord_progression: &[(usize, usize, usize)],
    level: f32,
) -> Vec<f32> {
    let total = (duration_s * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0_f32; total];
    let beat = 60.0 / bpm;
    let chord_dur = beat * 4.0; // une mesure par accord
    let chord_samples = (chord_dur * SAMPLE_RATE as f32) as usize;
    let mut pos = 0;
    let mut idx = 0;
    while pos < total {
        let (i1, i2, i3) = chord_progression[idx % chord_progression.len()];
        let freqs = [
            root_freq * scale[i1],
            root_freq * scale[i2],
            root_freq * scale[i3],
        ];
        for s in 0..chord_samples {
            if pos + s >= total {
                break;
            }
            let local_t = s as f32 / SAMPLE_RATE as f32;
            let attack = (local_t / 0.3).min(1.0);
            let release = (1.0 - (local_t / chord_dur) * 0.7).max(0.0);
            let env = attack * release * level;
            let mut sample = 0.0;
            for &f in &freqs {
                sample += (TAU * f * local_t).sin() * 0.33;
            }
            // léger shimmer
            sample *= 1.0 + 0.04 * (TAU * 4.0 * local_t).sin();
            out[pos + s] += sample * env;
        }
        pos += chord_samples;
        idx += 1;
    }
    out
}

fn main() {
    // ===== Niveau 1 : "Au commencement" — F# mineur pentatonique =====
    // Tempo lent, ambiance nostalgique rose
    let pentatonic_minor = [1.0, 1.189, 1.335, 1.498, 1.781];
    let duration = 24.0;
    let bpm = 78.0;
    let root_l1 = 92.5; // F#2

    let bass1 = synth_bass(duration, root_l1, 0.32);
    let melody1 = synth_melody(
        duration,
        bpm,
        root_l1 * 2.0,
        &pentatonic_minor,
        &[0, 2, 1, 3, 0, 2, 4, 2, 1, 3, 2, 0],
        0.20,
    );
    let pad1 = synth_pad(
        duration,
        bpm,
        root_l1 * 2.0,
        &pentatonic_minor,
        &[(0, 2, 4), (1, 3, 0), (2, 4, 1), (3, 0, 2)],
        0.18,
    );
    let track1 = mix(&[&bass1, &melody1, &pad1]);
    save("assets/music/level_1.wav", &track1);

    // ===== Niveau 2 : "Forêt silencieuse" — D mineur =====
    // Tempo encore plus lent, ambiance mystique sombre
    let duration = 28.0;
    let bpm = 64.0;
    let root_l2 = 73.4; // D2

    let bass2 = synth_bass(duration, root_l2, 0.36);
    let melody2 = synth_melody(
        duration,
        bpm,
        root_l2 * 2.0,
        &pentatonic_minor,
        &[0, 1, 3, 2, 4, 1, 0, 2, 3, 1, 2, 0, 4, 2],
        0.18,
    );
    let pad2 = synth_pad(
        duration,
        bpm,
        root_l2 * 2.0,
        &pentatonic_minor,
        &[(0, 2, 4), (0, 1, 3), (1, 3, 0)],
        0.16,
    );
    let track2 = mix(&[&bass2, &melody2, &pad2]);
    save("assets/music/level_2.wav", &track2);

    println!("Musique générée dans assets/music/");
}
