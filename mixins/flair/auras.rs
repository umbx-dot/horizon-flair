use super::umbx_flair_items::{Item, Moves, Paint, Slot};
use super::umbx_flair_model::{Bone, Motion, bone, cube};
use super::umbx_flair_paint::{Canvas, hue, noise, swatch_uv};
use std::f32::consts::{PI, TAU};

pub const VIEW: ([f32; 3], [f32; 3]) = ([-12.0, -8.0, -12.0], [12.0, 24.0, 12.0]);

type Path = fn(usize, f32) -> ([f32; 3], f32);

pub fn items() -> Vec<Item> {
    vec![
        aura("orbit", "Orbit", 6, orbit, Box::new(|f| swatches(6, |i| hue(i as f64 / 6.0 + f as f64 * 0.01)))),
        aura("helix", "Helix", 16, helix, Box::new(|_| swatches(16, |i| if i % 2 == 0 { 0x3C_E8FF } else { 0xFF_4CD8 }))),
        aura("sparkles", "Sparkles", 14, sparkle, Box::new(|f| swatches(14, |i| if (i + f as usize / 4) % 3 == 0 { 0xFF_E37A } else { 0xFF_FFFF }))),
        aura("embers", "Embers", 12, ember, Box::new(|f| swatches(12, |i| [0xFF_C23A, 0xFF_7A1C, 0xE0_2E12][(i + f as usize / 2) % 3]))),
        aura("snowfall", "Snowfall", 16, snow, Box::new(|_| swatches(16, |i| if i % 4 == 0 { 0xBF_E6FF } else { 0xFF_FFFF }))),
    ]
}

fn aura(key: &str, name: &str, count: usize, path: Path, paint: Box<dyn Fn(u32) -> Canvas + Send + Sync>) -> Item {
    Item::new(key, name, Slot::Aura, particles(count), Paint::Live(paint), Moves::Live(Box::new(move |bone, t| moved(path, bone, t)))).glowing()
}

fn particles(count: usize) -> Vec<Bone> {
    (0..count).map(|i| bone(&format!("p{i}"), [0.0; 3], vec![cube([-0.5, -0.5, -0.5], [1.0; 3], swatch_uv(i))])).collect()
}

fn swatches(count: usize, color: impl Fn(usize) -> u32) -> Canvas {
    let mut canvas = Canvas::default();
    for i in 0..count {
        canvas.swatch(i, color(i));
    }
    canvas
}

fn moved(path: Path, bone: &str, t: f32) -> Motion {
    let Some(i) = bone.strip_prefix('p').and_then(|n| n.parse().ok()) else { return Motion::REST };
    let (offset, size) = path(i, t);
    Motion { offset, rotation: [t * 1.3, t * 1.7, 0.0], scale: [size; 3] }
}

fn around(angle: f32, radius: f32, y: f32) -> [f32; 3] {
    [angle.cos() * radius, y, angle.sin() * radius]
}

fn orbit(i: usize, t: f32) -> ([f32; 3], f32) {
    let angle = t * 1.6 + i as f32 * TAU / 6.0;
    (around(angle, 10.0, 12.0 + 3.0 * (t * 2.0 + i as f32).sin()), 1.0 + 0.2 * (t * 4.0 + i as f32).sin())
}

fn helix(i: usize, t: f32) -> ([f32; 3], f32) {
    let phase = ((i / 2) as f32 / 8.0 + t * 0.25).rem_euclid(1.0);
    let angle = phase * TAU * 1.5 + (i % 2) as f32 * PI + t;
    (around(angle, 8.0, 24.0 - phase * 28.0), 0.4 + 0.8 * (phase * PI).sin())
}

fn sparkle(i: usize, t: f32) -> ([f32; 3], f32) {
    let period = 1.5;
    let shifted = t + noise(i as i64, 1) as f32 * period;
    let cycle = (shifted / period).floor() as i64;
    let phase = shifted / period - cycle as f32;
    let angle = noise(i as i64, cycle) as f32 * TAU;
    let radius = 6.0 + noise(cycle, i as i64) as f32 * 6.0;
    (around(angle, radius, noise(i as i64 * 3, cycle) as f32 * 26.0 - 2.0), (phase * PI).sin() * 1.1)
}

fn ember(i: usize, t: f32) -> ([f32; 3], f32) {
    let phase = (t * 0.5 + i as f32 / 12.0).rem_euclid(1.0);
    let angle = noise(i as i64, 5) as f32 * TAU + phase * 2.0;
    (around(angle, 4.0 + noise(i as i64, 6) as f32 * 6.0 + phase * 2.0, 24.0 - phase * 22.0), (1.0 - phase) * 1.2)
}

fn snow(i: usize, t: f32) -> ([f32; 3], f32) {
    let phase = (t * 0.3 + i as f32 / 16.0 + noise(i as i64, 9) as f32).rem_euclid(1.0);
    let angle = noise(i as i64, 8) as f32 * TAU;
    let radius = 3.0 + noise(i as i64, 7) as f32 * 9.0;
    let [x, y, z] = around(angle, radius, -8.0 + phase * 32.0);
    ([x + (t * 1.5 + i as f32).sin(), y, z], 0.7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_stay_around_the_player() {
        for item in items() {
            for bone in &item.bones {
                for step in 0..50 {
                    let m = item.motion(&bone.name, step as f32 * 0.37);
                    let [x, y, z] = m.offset;
                    assert!(x.abs() <= 14.0 && z.abs() <= 14.0 && (-10.0..=26.0).contains(&y), "{} {}", item.key, bone.name);
                    assert!((0.0..=1.4).contains(&m.scale[0]), "{} {}", item.key, bone.name);
                }
            }
        }
    }

    #[test]
    fn particles_move_over_time() {
        for item in items() {
            assert_ne!(item.motion("p1", 0.1), item.motion("p1", 0.6), "{}", item.key);
            assert_eq!(item.motion("other", 0.1), Motion::REST);
        }
    }
}
