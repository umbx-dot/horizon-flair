pub const SIZE: usize = 64;

#[derive(Clone, Debug, PartialEq)]
pub struct Canvas(pub Vec<u8>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Right,
    Front,
    Left,
    Back,
}

impl Default for Canvas {
    fn default() -> Self {
        Canvas(vec![0; SIZE * SIZE * 4])
    }
}

impl Canvas {
    pub fn put(&mut self, x: usize, y: usize, argb: u32) {
        if x < SIZE && y < SIZE {
            let i = (y * SIZE + x) * 4;
            self.0[i..i + 4].copy_from_slice(&[(argb >> 16) as u8, (argb >> 8) as u8, argb as u8, (argb >> 24) as u8]);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        let i = (y * SIZE + x) * 4;
        u32::from_be_bytes([self.0[i + 3], self.0[i], self.0[i + 1], self.0[i + 2]])
    }

    pub fn rect(&mut self, [x, y, w, h]: [usize; 4], paint: impl Fn(usize, usize) -> u32) {
        for dy in 0..h {
            for dx in 0..w {
                self.put(x + dx, y + dy, paint(dx, dy));
            }
        }
    }

    pub fn cube(&mut self, [u, v]: [usize; 2], [w, h, d]: [usize; 3], paint: impl Fn(Side, usize, usize, usize, usize) -> u32) {
        let faces = [
            (Side::Top, [u + d, v, w, d]),
            (Side::Bottom, [u + d + w, v, w, d]),
            (Side::Right, [u, v + d, d, h]),
            (Side::Front, [u + d, v + d, w, h]),
            (Side::Left, [u + d + w, v + d, d, h]),
            (Side::Back, [u + 2 * d + w, v + d, w, h]),
        ];
        for (side, rect) in faces {
            self.rect(rect, |x, y| paint(side, x, y, rect[2], rect[3]));
        }
    }
}

pub const fn opaque(rgb: u32) -> u32 {
    rgb | 0xFF00_0000
}

pub fn mix(a: u32, b: u32, t: f64) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |shift: u32| {
        let (x, y) = (f64::from((a >> shift) & 0xFF), f64::from((b >> shift) & 0xFF));
        ((x + (y - x) * t).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
}

pub fn palette(low: u32, mid: u32, high: u32, t: f64) -> u32 {
    if t < 0.5 { mix(low, mid, t * 2.0) } else { mix(mid, high, (t - 0.5) * 2.0) }
}

pub fn hue(turns: f64) -> u32 {
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

pub fn noise(a: i64, b: i64) -> f64 {
    let mut h = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_get_round_trip_argb() {
        let mut canvas = Canvas::default();
        canvas.put(3, 5, 0x8012_3456);
        assert_eq!(canvas.get(3, 5), 0x8012_3456);
        canvas.put(SIZE, 0, 0xFFFF_FFFF);
        assert_eq!(canvas.get(0, 0), 0);
    }

    #[test]
    fn cube_paints_the_box_uv_layout() {
        let mut canvas = Canvas::default();
        canvas.cube([0, 0], [8, 8, 8], |side, _, _, _, _| opaque(side as u32));
        assert_eq!(canvas.get(9, 1), opaque(Side::Top as u32));
        assert_eq!(canvas.get(17, 1), opaque(Side::Bottom as u32));
        assert_eq!(canvas.get(1, 9), opaque(Side::Right as u32));
        assert_eq!(canvas.get(9, 9), opaque(Side::Front as u32));
        assert_eq!(canvas.get(17, 9), opaque(Side::Left as u32));
        assert_eq!(canvas.get(25, 9), opaque(Side::Back as u32));
        assert_eq!(canvas.get(1, 1), 0);
    }

    #[test]
    fn colors_blend_and_noise_stays_in_range() {
        assert_eq!(mix(0x00_0000, 0xFF_FFFF, 0.5), 0x80_8080);
        assert_eq!(hue(0.0), 0xFF_0000);
        assert_eq!(hue(1.25), hue(0.25));
        assert!((0..1000).all(|i| (0.0..1.0).contains(&noise(i, i * 7 - 3))));
    }
}
