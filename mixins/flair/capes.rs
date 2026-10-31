pub const SIZE: usize = 64;
pub const STYLES: [&str; 6] = ["Royal", "Nebula", "Ember", "Prism", "Frost", "Tide"];
pub const FACE_W: usize = 10;
pub const FACE_H: usize = 16;
const CAPE_ROWS: usize = 32;
const WHITE: u32 = 0xFF_FFFF;
const BLACK: u32 = 0;
const ROYAL: u32 = 0x4B_2496;
const GOLD: u32 = 0xF2_C14E;

pub fn cape(style: usize, frame: u32) -> Vec<u8> {
    let mut rgba = vec![0; SIZE * SIZE * 4];
    let t = f64::from(frame);
    for y in 0..CAPE_ROWS {
        let v = y.saturating_sub(1).min(FACE_H - 1);
        for x in 0..SIZE {
            let (u, inner) = match x {
                1..=10 => (x - 1, false),
                12..=21 => (21 - x, true),
                _ => (x % FACE_W, true),
            };
            let color = shade(style, u, v, t);
            put(&mut rgba, x, y, if inner { mix(color, BLACK, 0.3) } else { color });
        }
    }
    rgba
}

pub fn front(rgba: &[u8], u: usize, v: usize) -> u32 {
    let i = ((v + 1) * SIZE + u + 1) * 4;
    u32::from_be_bytes([rgba[i + 3], rgba[i], rgba[i + 1], rgba[i + 2]])
}

fn shade(style: usize, u: usize, v: usize, t: f64) -> u32 {
    let (fu, fv) = (u as f64 / (FACE_W - 1) as f64, v as f64 / (FACE_H - 1) as f64);
    match style {
        1 => {
            let nebula = 0.5 + 0.5 * (fu * 4.0 + t * 0.05).sin() * (fv * 3.0 - t * 0.03).cos();
            let base = palette(0x05_020F, 0x1B_0B3A, 0x4A_2380, nebula * (0.6 + 0.4 * fv));
            let star = noise((u * 16 + v) as i64, 7);
            if star > 0.86 { mix(base, WHITE, 0.45 + 0.55 * (t * 0.6 + star * 50.0).sin().max(0.0)) } else { base }
        }
        2 => {
            let flicker = 0.5 * (noise(u as i64, v as i64 + t as i64) + noise(u as i64, v as i64 + 1 + t as i64));
            palette(0x1A_0000, 0xD8_300A, 0xFF_D24A, (fv * 1.1 + 0.45 * flicker - 0.3).clamp(0.0, 1.0))
        }
        3 => mix(hue(fu * 0.25 + fv * 0.35 - t * 0.02), BLACK, 0.15 - 0.15 * (fv * 6.0 - t * 0.5).sin()),
        4 => {
            let base = palette(0xE4_F6FF, 0x7F_C4F0, 0x23_4F8C, fv);
            let fall = (v as f64 - t * 0.3).floor() as i64;
            let flake = noise(u as i64 * 31, fall) > 0.88;
            let shimmer = ((u + 2 * v) as f64 - t * 0.6).rem_euclid(24.0) < 1.5;
            if flake { WHITE } else if shimmer { mix(base, WHITE, 0.5) } else { base }
        }
        5 => {
            let wave = (fv * 5.0 + 0.35 * (fu * 6.0 + t * 0.25).sin() - t * 0.06).rem_euclid(1.0);
            let water = palette(0x04_1E3A, 0x0B_5F8A, 0x2A_B7D6, 0.3 + 0.7 * wave);
            if wave > 0.9 { mix(water, 0xD8_FBFF, 0.8) } else { water }
        }
        _ => {
            let diamond = (u as f64 - 4.5).abs() + (v as f64 - 7.0).abs();
            let glint = ((u + v) as f64 - t * 0.8).rem_euclid(30.0) < 2.0;
            let color = if u == 0 || u == FACE_W - 1 || v == 0 || v == FACE_H - 1 {
                GOLD
            } else if diamond > 2.5 && diamond <= 3.5 {
                mix(GOLD, WHITE, 0.3 + 0.3 * (t * 0.4).sin())
            } else if diamond <= 2.5 {
                mix(ROYAL, WHITE, 0.15)
            } else {
                mix(ROYAL, BLACK, 0.2 + 0.4 * fv)
            };
            if glint { mix(color, WHITE, 0.4) } else { color }
        }
    }
}

fn put(rgba: &mut [u8], x: usize, y: usize, rgb: u32) {
    let i = (y * SIZE + x) * 4;
    rgba[i..i + 4].copy_from_slice(&[(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 0xFF]);
}

fn mix(a: u32, b: u32, t: f64) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |shift: u32| {
        let (x, y) = (f64::from((a >> shift) & 0xFF), f64::from((b >> shift) & 0xFF));
        ((x + (y - x) * t).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
}

fn palette(low: u32, mid: u32, high: u32, t: f64) -> u32 {
    if t < 0.5 { mix(low, mid, t * 2.0) } else { mix(mid, high, (t - 0.5) * 2.0) }
}

fn hue(turns: f64) -> u32 {
    let h = turns.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    let byte = |v: f64| (v * 255.0).round() as u32;
    byte(r) << 16 | byte(g) << 8 | byte(b)
}

fn noise(a: i64, b: i64) -> f64 {
    let mut h = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(rgba: &[u8], x: usize, y: usize) -> u8 {
        rgba[(y * SIZE + x) * 4 + 3]
    }

    #[test]
    fn every_style_fills_the_top_half_only() {
        for style in 0..STYLES.len() {
            let rgba = cape(style, 3);
            assert_eq!(rgba.len(), SIZE * SIZE * 4);
            assert!((0..CAPE_ROWS).all(|y| (0..SIZE).all(|x| alpha(&rgba, x, y) == 0xFF)));
            assert!((CAPE_ROWS..SIZE).all(|y| (0..SIZE).all(|x| alpha(&rgba, x, y) == 0)));
        }
    }

    #[test]
    fn every_style_animates() {
        for style in 0..STYLES.len() {
            assert_ne!(cape(style, 1), cape(style, 9), "{}", STYLES[style]);
        }
    }

    #[test]
    fn front_reads_the_outside_face_as_argb() {
        let mut rgba = vec![0; SIZE * SIZE * 4];
        put(&mut rgba, 1, 1, 0x12_3456);
        put(&mut rgba, 10, 16, 0xAB_CDEF);
        assert_eq!(front(&rgba, 0, 0), 0xFF12_3456);
        assert_eq!(front(&rgba, FACE_W - 1, FACE_H - 1), 0xFFAB_CDEF);
    }
}
