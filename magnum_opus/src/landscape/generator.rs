//! Deterministic terrain generator: splitmix64 + bilinear value noise + fBm.
//! Zero external dependencies. f32 ops are scalar (single-threaded), so same
//! seed produces bit-identical cells across platforms.

use super::resource::{TerrainCell, TerrainKind};

#[inline]
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
fn hash3(seed: u64, x: i32, y: i32) -> u64 {
    let ux = (x as u32) as u64;
    let uy = (y as u32) as u64;
    splitmix64(seed ^ (ux << 32) ^ uy)
}

#[inline]
fn rand_f32(seed: u64, x: i32, y: i32) -> f32 {
    (hash3(seed, x, y) >> 40) as f32 / (1u64 << 24) as f32
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(seed: u64, x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let a = rand_f32(seed, xi, yi);
    let b = rand_f32(seed, xi + 1, yi);
    let c = rand_f32(seed, xi, yi + 1);
    let d = rand_f32(seed, xi + 1, yi + 1);
    let u = smoothstep(xf);
    let v = smoothstep(yf);
    let ab = a + u * (b - a);
    let cd = c + u * (d - c);
    ab + v * (cd - ab)
}

fn fbm(seed: u64, x: f32, y: f32, octaves: u32, base_freq: f32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = base_freq;
    let mut norm = 0.0;
    for o in 0..octaves {
        let octave_seed = splitmix64(seed.wrapping_add(o as u64));
        sum += amp * value_noise(octave_seed, x * freq, y * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

fn classify(e: f32, lava_mask: f32, moisture: f32) -> (TerrainKind, u8) {
    if e < -0.40 {
        let d = (((-0.40) - e) * 100.0).clamp(1.0, 255.0);
        return (TerrainKind::Pit, d as u8);
    }
    if e < -0.15 {
        let d = (((-0.15) - e) * 40.0).clamp(1.0, 255.0);
        return (TerrainKind::Water, d as u8);
    }
    if e < 0.15 {
        if lava_mask > 0.70 {
            return (TerrainKind::Lava, 0);
        }
        if moisture < 0.30 {
            return (TerrainKind::Sand, 0);
        }
        return (TerrainKind::Grass, 0);
    }
    if e < 0.50 {
        return (TerrainKind::Rock, 0);
    }
    (TerrainKind::Mountain, 0)
}

pub fn generate_terrain(seed: u64, width: u32, height: u32) -> Vec<TerrainCell> {
    let e_seed = splitmix64(seed ^ 0xE1E7);
    let m_seed = splitmix64(seed ^ 0x407F);
    let l_seed = splitmix64(seed ^ 0x1A7A);
    let mut cells = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let elev_raw = fbm(e_seed, x as f32, y as f32, 5, 1.0 / 32.0);
            let moist_raw = fbm(m_seed, x as f32, y as f32, 3, 1.0 / 24.0);
            let lava_raw = fbm(l_seed, x as f32, y as f32, 2, 1.0 / 16.0);

            let e = elev_raw * 2.0 - 1.0;
            let (kind, depth) = classify(e, lava_raw, moist_raw);
            let elevation = (e.clamp(-1.0, 1.0) * 64.0) as i8;
            let moisture = (moist_raw.clamp(0.0, 1.0) * 255.0) as u8;

            cells.push(TerrainCell {
                kind,
                elevation,
                depth,
                moisture,
            });
        }
    }
    cells
}
