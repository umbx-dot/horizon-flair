use super::umbx_flair_items::{Item, Moves, Paint, Slot, wave};
use super::umbx_flair_model::{Bone, Motion, bone, cube};
use super::umbx_flair_paint::{Canvas, Side, hue, mix, noise, opaque};

const BACK: f32 = 2.5;

pub fn items() -> Vec<Item> {
    vec![
        Item::new("angel_wings", "Angel Wings", Slot::Back, pair(angel()), Paint::Still(angel_paint()), Moves::Live(Box::new(|n, t| flap(n, t, 2.0, 0.22)))),
        Item::new("dragon_wings", "Dragon Wings", Slot::Back, pair(dragon()), Paint::Still(dragon_paint()), Moves::Live(Box::new(|n, t| flap(n, t, 1.5, 0.35)))),
        Item::new("fairy_wings", "Fairy Wings", Slot::Back, pair(fairy()), Paint::Live(Box::new(fairy_paint)), Moves::Live(Box::new(|n, t| flap(n, t, 9.0, 0.3)))).glowing(),
    ]
}

fn pair(left: Vec<Bone>) -> Vec<Bone> {
    let right: Vec<Bone> = left.iter().map(|b| b.mirrored(&b.name.replace("_l", "_r"))).collect();
    left.into_iter().chain(right).collect()
}

fn flap(name: &str, t: f32, speed: f32, size: f32) -> Motion {
    let side = if name.contains("_r") { -1.0 } else { 1.0 };
    Motion { rotation: [0.0, side * size * wave(t, speed, 0.0), 0.0], ..Motion::REST }
}

fn angel() -> Vec<Bone> {
    vec![
        bone(
            "wing_l",
            [1.5, 2.0, BACK],
            vec![cube([1.5, -1.0, BACK], [6.0, 9.0, 1.0], [0.0, 0.0]), cube([7.5, -3.0, BACK], [4.0, 9.0, 1.0], [14.0, 0.0]), cube([11.5, -4.0, BACK], [3.0, 7.0, 1.0], [24.0, 0.0])],
        )
        .turned([0.1, -0.6, 0.1]),
    ]
}

fn feathers(canvas: &mut Canvas, base: u32, line: u32, tip: u32) {
    for (uv, size) in [([0, 0], [6, 9, 1]), ([14, 0], [4, 9, 1]), ([24, 0], [3, 7, 1])] {
        canvas.cube(uv, size, |side, x, y, _, h| {
            let edge = y + 1 == h && matches!(side, Side::Front | Side::Back);
            opaque(if edge { tip } else if (y + x / 2) % 3 == 0 { line } else { base })
        });
    }
}

fn angel_paint() -> Canvas {
    let mut canvas = Canvas::default();
    feathers(&mut canvas, 0xF4_F6FF, 0xCC_D3E6, 0xDD_EAFF);
    canvas
}

fn dragon() -> Vec<Bone> {
    vec![
        bone(
            "wing_l",
            [1.5, 1.0, BACK],
            vec![cube([1.5, -2.0, BACK], [6.0, 9.0, 1.0], [0.0, 0.0]), cube([7.5, -4.0, BACK], [4.0, 9.0, 1.0], [14.0, 0.0]), cube([11.5, -6.0, BACK], [3.0, 7.0, 1.0], [24.0, 0.0])],
        )
        .turned([0.15, -0.5, 0.25]),
    ]
}

fn dragon_paint() -> Canvas {
    let mut canvas = Canvas::default();
    for (uv, size) in [([0, 0], [6, 9, 1]), ([14, 0], [4, 9, 1]), ([24, 0], [3, 7, 1])] {
        canvas.cube(uv, size, |_, x, y, _, h| {
            let ridge = x % 3 == 0;
            let glow = noise(x as i64, y as i64) * 0.15;
            opaque(if ridge { 0x1C_0A28 } else if y + 1 == h { 0x8C_3CFF } else { mix(0x3A_1450, 0x6A_2A90, glow + y as f64 * 0.03) })
        });
    }
    canvas
}

fn fairy() -> Vec<Bone> {
    vec![
        bone("wing_l", [1.0, 1.0, BACK], vec![cube([1.0, -5.0, BACK], [5.0, 7.0, 0.5], [0.0, 0.0])]).turned([0.0, -0.45, -0.35]),
        bone("wing_low_l", [1.0, 3.0, BACK], vec![cube([1.0, 3.0, BACK], [4.0, 5.0, 0.5], [0.0, 10.0])]).turned([0.0, -0.45, 0.3]),
    ]
}

fn fairy_paint(frame: u32) -> Canvas {
    let mut canvas = Canvas::default();
    let shade = |x: usize, y: usize| {
        let base = hue(0.85 + (x + y) as f64 * 0.02 + frame as f64 * 0.01);
        let sparkle = noise((x * 7 + y) as i64, (frame / 2) as i64) > 0.9;
        opaque(if sparkle { 0xFF_FFFF } else { mix(base, 0xFF_FFFF, 0.45) })
    };
    canvas.cube([0, 0], [5, 7, 1], |_, x, y, _, _| shade(x, y));
    canvas.cube([0, 10], [4, 5, 1], |_, x, y, _, _| shade(x, y + 7));
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::umbx_flair_model::bounds;

    #[test]
    fn wings_come_in_mirrored_pairs_on_the_back() {
        for item in items() {
            let names: Vec<_> = item.bones.iter().map(|b| b.name.clone()).collect();
            assert!(names.iter().any(|n| n.ends_with("_l")) && names.iter().any(|n| n.ends_with("_r")), "{}", item.key);
            let (lo, hi) = bounds(&item.bones).unwrap();
            assert!(lo[2] >= BACK && (lo[0] + hi[0]).abs() < 1e-4, "{}", item.key);
        }
    }

    #[test]
    fn wings_flap_in_opposite_directions() {
        let (l, r) = (flap("wing_l", 0.5, 2.0, 0.2), flap("wing_r", 0.5, 2.0, 0.2));
        assert_eq!(l.rotation[1], -r.rotation[1]);
        assert_ne!(l.rotation[1], 0.0);
    }
}
