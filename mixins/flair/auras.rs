use super::umbx_flair_items::{Effect, Item, Moves, Paint, Slot, Stroke};
use super::umbx_flair_paint::{Canvas, mix, noise, palette};
use std::f64::consts::{PI, TAU};

const WHITE: u32 = 0xFF_FFFF;
const WIDTH: f32 = 2.5;
const RING_WIDTH: f32 = 3.5;
const PLAYER_HEIGHT: f64 = 1.8;
const HALO_LIFT: f64 = 0.35;
const HALO_RADIUS: f64 = 0.32;
const HALO_SEGMENTS: usize = 40;
const STREAKS: usize = 14;
const STREAK_HEIGHT: f64 = 2.2;
const ORBIT_RADIUS: f64 = 0.95;
const ORBIT_TILT: f64 = 1.05;
const ORBIT_SEGMENTS: usize = 48;
const ORBIT_TAIL: usize = 8;
const WING_SPAN: f64 = 1.1;
const WING_STEPS: usize = 8;
const HELIX_STEPS: usize = 40;
const HELIX_RADIUS: f64 = 0.55;
const HELIX_HEIGHT: f64 = 2.1;
const RING_RADIUS: f64 = 0.85;
const RING_LIFT: f64 = 0.05;
const RING_SEGMENTS: usize = 96;
const BOLT_STEPS: usize = 4;
const VORTEX_ARMS: usize = 6;
const VORTEX_STEPS: usize = 16;
const VORTEX_HEIGHT: f64 = 0.9;
const VORTEX_COLOR: u32 = 0x9B_6BFF;
const GOLD: u32 = 0xFF_D86B;
const ICE: u32 = 0x9F_E8FF;
const ORBIT_COLORS: [u32; 3] = [0x9B_6BFF, 0x4A_D8FF, 0xFF_6BD5];
const PALE_GOLD: u32 = 0xFF_E9A8;
const HELIX_COLORS: [u32; 2] = [0x3C_E8FF, 0xFF_4CD8];

pub fn items() -> Vec<Item> {
    let aura = |key: &str, name: &str, effect: Effect| Item::new(key, name, Slot::Aura, Vec::new(), Paint::Still(Canvas::default()), Moves::Still).with_effect(effect);
    vec![
        aura("halo_rings", "Halo Rings", halo),
        aura("streaks", "Streaks", streaks),
        aura("orbits", "Orbits", orbits),
        aura("light_wings", "Light Wings", wings),
        aura("helix", "Helix", helix),
        aura("fire_ring", "Fire Ring", |t, _, out| ring(1, t, out)),
        aura("lava_ring", "Lava Ring", |t, _, out| ring(2, t, out)),
        aura("storm_ring", "Storm Ring", |t, _, out| ring(3, t, out)),
        aura("vortex", "Vortex", |t, _, out| ring(4, t, out)),
    ]
}

fn line(out: &mut Vec<Stroke>, a: [f64; 3], b: [f64; 3], rgb: u32, alpha: f64, width: f32) {
    out.push(Stroke { a, b, color: ((alpha.clamp(0.0, 1.0) * 255.0).round() as u32) << 24 | (rgb & 0xFF_FFFF), width });
}

fn halo(time: f64, _: f64, out: &mut Vec<Stroke>) {
    let center = [0.0, PLAYER_HEIGHT + HALO_LIFT + (time * 2.0).sin() * 0.03, 0.0];
    let tilt = (time * 0.9).sin() * 0.06;
    for ring in 0..2 {
        let radius = HALO_RADIUS - ring as f64 * 0.06;
        let point = |i: usize| {
            let angle = TAU * i as f64 / HALO_SEGMENTS as f64 + time * 0.8;
            [center[0] + radius * angle.cos(), center[1] + tilt * angle.sin(), center[2] + radius * angle.sin()]
        };
        for i in 0..HALO_SEGMENTS {
            let angle = TAU * i as f64 / HALO_SEGMENTS as f64;
            let shimmer = 0.55 + 0.45 * (angle * 3.0 + time * 4.0).sin();
            line(out, point(i), point(i + 1), mix(GOLD, WHITE, 0.5 * shimmer), shimmer * if ring == 0 { 1.0 } else { 0.5 }, WIDTH);
        }
    }
}

fn streaks(time: f64, _: f64, out: &mut Vec<Stroke>) {
    for j in 0..STREAKS {
        let seed = j as f64 * 0.618_034;
        let progress = (time * 0.35 + seed).fract();
        let fade = (PI * progress).sin();
        let angle = time * (0.6 + 0.4 * (seed * 3.0).fract()) + j as f64 * TAU / STREAKS as f64;
        let radius = 0.75 + 0.15 * (time * 1.3 + j as f64).sin();
        let y = progress * STREAK_HEIGHT;
        let length = 0.15 + 0.35 * fade;
        let start = [radius * angle.cos(), y, radius * angle.sin()];
        let end = [radius * (angle + 0.15).cos(), y + length, radius * (angle + 0.15).sin()];
        line(out, start, end, mix(ICE, WHITE, (seed * 7.0).fract()), 0.85 * fade, WIDTH);
    }
}

fn orbits(time: f64, _: f64, out: &mut Vec<Stroke>) {
    let center = [0.0, 1.0, 0.0];
    for k in 0..3 {
        let phi = time * 0.5 + k as f64 * TAU / 3.0;
        let u = [phi.cos(), 0.0, phi.sin()];
        let v = [-phi.sin() * ORBIT_TILT.sin(), ORBIT_TILT.cos(), phi.cos() * ORBIT_TILT.sin()];
        let at = |a: f64| -> [f64; 3] { std::array::from_fn(|i| center[i] + ORBIT_RADIUS * (u[i] * a.cos() + v[i] * a.sin())) };
        let color = ORBIT_COLORS[k];
        for i in 0..ORBIT_SEGMENTS {
            let (a0, a1) = (TAU * i as f64 / ORBIT_SEGMENTS as f64, TAU * (i + 1) as f64 / ORBIT_SEGMENTS as f64);
            line(out, at(a0), at(a1), color, 0.3, WIDTH * 0.7);
        }
        let head = time * 3.0 + k as f64 * 2.0;
        for j in 0..ORBIT_TAIL {
            let a0 = head - j as f64 * 0.09;
            line(out, at(a0), at(a0 - 0.09), mix(color, WHITE, 0.4), 1.0 - j as f64 / ORBIT_TAIL as f64, WIDTH * (2.0 - 0.18 * j as f32));
        }
    }
}

fn wings(time: f64, yaw: f64, out: &mut Vec<Stroke>) {
    let yaw = yaw.to_radians();
    let forward = [-yaw.sin(), 0.0, yaw.cos()];
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    let flap = 0.45 + 0.3 * (time * 3.2).sin();
    let shoulder = [-forward[0] * 0.28, 1.35, -forward[2] * 0.28];
    for side in [-1.0, 1.0] {
        let bone = |t: f64| -> [f64; 3] {
            let reach = WING_SPAN * t;
            let back = reach * flap.sin() * 0.5 + 0.05;
            let lift = 0.45 * t - 0.25 * t * t;
            std::array::from_fn(|i| shoulder[i] + right[i] * side * reach * flap.cos() - forward[i] * back + if i == 1 { lift } else { 0.0 })
        };
        for k in 0..WING_STEPS {
            let (t0, t1) = (k as f64 / WING_STEPS as f64, (k + 1) as f64 / WING_STEPS as f64);
            line(out, bone(t0), bone(t1), mix(WHITE, PALE_GOLD, t0), 0.95, WIDTH * 1.2);
            let root = bone(t1);
            let length = 0.75 * (1.0 - 0.55 * t1);
            let sway = 0.12 + 0.08 * (time * 4.0 + k as f64).sin();
            let tip = [root[0] - forward[0] * sway, root[1] - length, root[2] - forward[2] * sway];
            line(out, root, tip, mix(WHITE, PALE_GOLD, t1), 0.75 * (1.0 - 0.4 * t1), WIDTH);
        }
    }
}

fn helix_point(time: f64, strand: usize, k: usize) -> [f64; 3] {
    let t = k as f64 / HELIX_STEPS as f64;
    let a = time * 2.2 + t * TAU * 2.0 + strand as f64 * PI;
    [HELIX_RADIUS * a.cos(), t * HELIX_HEIGHT, HELIX_RADIUS * a.sin()]
}

fn helix(time: f64, _: f64, out: &mut Vec<Stroke>) {
    for strand in 0..2 {
        for k in 0..HELIX_STEPS {
            line(out, helix_point(time, strand, k), helix_point(time, strand, k + 1), HELIX_COLORS[strand], 0.85, WIDTH);
        }
    }
    for k in (0..=HELIX_STEPS).step_by(4) {
        let pulse = 0.5 + 0.5 * (time * 5.0 - k as f64 * 0.4).sin();
        line(out, helix_point(time, 0, k), helix_point(time, 1, k), mix(HELIX_COLORS[0], HELIX_COLORS[1], 0.5), 0.2 + 0.4 * pulse, WIDTH * 0.6);
    }
}

fn ring(style: usize, time: f64, out: &mut Vec<Stroke>) {
    let point = |i: usize, r: f64| {
        let angle = TAU * i as f64 / RING_SEGMENTS as f64;
        [r * angle.cos(), RING_LIFT, r * angle.sin()]
    };
    let radius = RING_RADIUS;
    let frame = (time * 18.0) as i64;
    for i in 0..RING_SEGMENTS {
        let angle = TAU * i as f64 / RING_SEGMENTS as f64;
        match style {
            1 => {
                let flame = (0.5 + 0.35 * (angle * 7.0 + time * 5.0).sin() + 0.15 * (angle * 17.0 - time * 11.0).sin()).clamp(0.0, 1.0);
                line(out, point(i, radius), point(i + 1, radius), palette(0xB0_1400, 0xFF_6A00, 0xFF_D84A, flame), 0.75 + 0.25 * flame, RING_WIDTH);
                if i % 4 == 0 {
                    let height = 0.1 + 0.45 * flame * (0.5 + 0.5 * (time * 9.0 + i as f64).sin());
                    let base = point(i, radius);
                    let mid = [base[0], base[1] + height * 0.55, base[2]];
                    let tip = [base[0], base[1] + height, base[2]];
                    line(out, base, mid, 0xFF_6A00, 0.85, RING_WIDTH * 0.8);
                    line(out, mid, tip, 0xFF_D84A, 0.55, RING_WIDTH * 0.5);
                }
            }
            2 => {
                let wave = 0.5 + 0.5 * (angle * 4.0 - time * 1.6).sin();
                let crack = ((angle * 23.0 + time * 0.7).sin() * (time * 2.3 + angle * 5.0).cos()).max(0.0).powi(6);
                let color = mix(palette(0x7A_0A00, 0xE0_3A00, 0xE0_3A00, wave), 0xFF_B030, crack);
                line(out, point(i, radius), point(i + 1, radius), color, 1.0, RING_WIDTH * 1.3);
                line(out, point(i, radius - 0.07), point(i + 1, radius - 0.07), 0x7A_0A00, 0.6, RING_WIDTH * 0.8);
            }
            3 => {
                let jag = |k: usize| radius + (noise((k % RING_SEGMENTS) as i64, frame) - 0.5) * 0.16;
                let flicker = 0.6 + 0.4 * noise(i as i64 / 8, frame);
                line(out, point(i, radius), point(i + 1, radius), 0x4A_C8FF, 0.3, RING_WIDTH * 1.8);
                line(out, point(i, jag(i)), point(i + 1, jag(i + 1)), mix(0x4A_C8FF, WHITE, noise(i as i64, frame + 1)), flicker, RING_WIDTH * 0.8);
                if i % 16 == 0 && noise(i as i64, frame / 3) > 0.55 {
                    let mut at = point(i, radius);
                    for k in 0..BOLT_STEPS {
                        let step = frame + k as i64;
                        let next = [at[0] + (noise(i as i64, step) - 0.5) * 0.25, at[1] + 0.18, at[2] + (noise(i as i64 + 1, step) - 0.5) * 0.25];
                        line(out, at, next, WHITE, 1.0 - 0.2 * k as f64, RING_WIDTH * 0.6);
                        at = next;
                    }
                }
            }
            _ => {
                let pulse = 0.5 + 0.5 * (angle * 6.0 - time * 6.0).sin();
                line(out, point(i, radius), point(i + 1, radius), mix(VORTEX_COLOR, WHITE, pulse * 0.5), 0.6 + 0.4 * pulse, RING_WIDTH);
            }
        }
    }
    if style == 4 {
        for arm in 0..VORTEX_ARMS {
            let at = |t: f64| {
                let a = arm as f64 * TAU / VORTEX_ARMS as f64 + time * 1.8 + t * 3.0;
                let r = radius * (1.0 - 0.75 * t);
                [r * a.cos(), RING_LIFT + VORTEX_HEIGHT * t * t, r * a.sin()]
            };
            for k in 0..VORTEX_STEPS {
                let (t0, t1) = (k as f64 / VORTEX_STEPS as f64, (k + 1) as f64 / VORTEX_STEPS as f64);
                line(out, at(t0), at(t1), mix(VORTEX_COLOR, WHITE, t0), 1.0 - t0, RING_WIDTH * (1.0 - 0.6 * t0 as f32));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_aura_draws_near_the_player_and_moves() {
        for item in items() {
            let effect = item.effect.expect("aura effect");
            let (mut a, mut b) = (Vec::new(), Vec::new());
            effect(1.0, 30.0, &mut a);
            effect(1.7, 30.0, &mut b);
            assert!(!a.is_empty(), "{}", item.key);
            assert_ne!(a, b, "{}", item.key);
            for s in &a {
                for p in [s.a, s.b] {
                    assert!(p[0].abs() < 2.0 && p[2].abs() < 2.0 && (-0.1..3.0).contains(&p[1]), "{} {p:?}", item.key);
                }
            }
        }
    }
}
