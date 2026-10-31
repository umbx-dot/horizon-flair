use super::umbx_flair_items::{Item, Moves, Paint, Slot, wave};
use super::umbx_flair_model::{Bone, Cube, Motion, Uv, bone, cube};
use super::umbx_flair_paint::{Canvas, Side, mix, noise, opaque};

const WHITE: u32 = 0xFF_FFFF;
const BLACK: u32 = 0;

pub fn items() -> Vec<Item> {
    let mut items = bandanas();
    items.extend([
        Item::new("top_hat", "Top Hat", Slot::Hat, top_hat(), Paint::Still(top_hat_paint()), Moves::Still),
        Item::new("halo", "Halo", Slot::Hat, halo(), Paint::Live(Box::new(halo_paint)), Moves::Live(Box::new(float))).glowing(),
        Item::new("crown", "Crown", Slot::Hat, crown(), Paint::Live(Box::new(crown_paint)), Moves::Still),
        Item::new("devil_horns", "Devil Horns", Slot::Hat, horns(), Paint::Still(horns_paint()), Moves::Still),
    ]);
    items
}

const GLYPHS: [[&str; 5]; 8] = [
    ["..#..", "#.#.#", "..#..", ".#.#.", "#...#"],
    ["..#..", "#####", ".###.", "#.#.#", "..#.."],
    ["..#..", "#.#.#", "#.#.#", "#.#.#", "#####"],
    ["#####", "..#..", ".###.", "..#..", "#####"],
    ["..#..", "#####", "..#..", ".#.#.", "#...#"],
    ["####.", "..#..", ".#.#.", "#.#.#", ".#.#."],
    ["..#..", "#####", "#.#.#", "#####", "..#.."],
    [".#...", "#####", ".#..#", "#...#", "#..##"],
];
const EMBLEM: [&str; 7] = ["..###..", ".#...#.", "#..#..#", "#.###.#", "#..#..#", ".#...#.", "..###.."];
const STRIP: usize = 128;
const FACE: usize = 32;
const ROWS: usize = 10;
const CELL: usize = 8;

#[derive(Clone, Copy)]
struct Band {
    cloth: u32,
    ink: u32,
    edge: u32,
    stars: bool,
}

fn bandanas() -> Vec<Item> {
    let band = |key: &str, name: &str, look: Band| Item::new(key, name, Slot::Hat, bandana(), Paint::Live(Box::new(move |f| bandana_paint(look, f))), Moves::Live(Box::new(tails)));
    vec![
        band("shinobi_bandana", "Shinobi", Band { cloth: 0xF2_F2EE, ink: 0x14_1414, edge: 0x30_3030, stars: false }),
        band("ronin_bandana", "Ronin", Band { cloth: 0xA8_1028, ink: 0x14_0406, edge: 0x5E_0814, stars: false }),
        band("shadow_bandana", "Shadow", Band { cloth: 0x15_151B, ink: 0xD8_2238, edge: 0x2E_2E3A, stars: false }),
        band("sakura_bandana", "Sakura", Band { cloth: 0xF3_A9C4, ink: 0xFF_FFFF, edge: 0xC8_5F8A, stars: false }),
        band("galaxy_bandana", "Galaxy", Band { cloth: 0x0E_0B1C, ink: 0xF4_F0FF, edge: 0x24_1C3C, stars: true }),
        band("neon_bandana", "Neon", Band { cloth: 0x0A_0C12, ink: 0x3C_E8FF, edge: 0x1A_4A58, stars: false }).glowing(),
    ]
}

fn bandana() -> Vec<Bone> {
    let none = [f32::NAN; 4];
    let f = FACE as f32;
    let r = ROWS as f32;
    let faces = [none, none, [f, r, 2.0 * f, 2.0 * r], [0.0, 0.0, f, r], [f, 0.0, 2.0 * f, r], [0.0, r, f, 2.0 * r]];
    let band = Cube { from: [-4.0, -8.0, -4.0], size: [8.0, 2.5, 8.0], uv: Uv::Faces(faces), inflate: 0.25, mirror: false };
    let low = bone("tail_low_l", [-0.5, -4.6, 4.5], vec![cube([-1.0, -4.6, 4.3], [1.0, 2.0, 0.4], [0.0, 24.0])]).turned([0.25, 0.0, 0.0]);
    let tail = bone("tail_l", [-0.5, -6.6, 4.5], vec![cube([-1.0, -6.6, 4.3], [1.0, 2.0, 0.4], [0.0, 24.0])]).turned([0.4, 0.0, 0.15]).with(vec![low]);
    vec![
        bone("band", [0.0; 3], vec![band]),
        bone("knot", [0.0, -6.75, 4.25], vec![cube([-0.75, -7.5, 4.25], [1.5, 1.5, 0.75], [0.0, 20.0])]),
        tail.mirrored("tail_r"),
        tail,
    ]
}

fn tails(name: &str, t: f32) -> Motion {
    if !name.starts_with("tail_") {
        return Motion::REST;
    }
    let right = name.starts_with("tail_r");
    let low = name.contains("low");
    let phase = if low { 0.9 } else { 0.0 } + if right { 0.5 } else { 0.0 };
    let size = if low { 0.3 } else { 0.15 };
    let side = if right { 1.0 } else { -1.0 };
    Motion { rotation: [size * wave(t, 3.2, phase), 0.0, side * 0.08 * wave(t, 2.1, phase)], ..Motion::REST }
}

fn glyph(index: usize, column: usize, row: usize) -> bool {
    GLYPHS[index % GLYPHS.len()][row].as_bytes()[column] == b'#'
}

fn strip_at(s: usize, row: usize) -> (usize, usize) {
    if s < 2 * FACE { (s, row) } else { (s - 2 * FACE, row + ROWS) }
}

fn bandana_paint(look: Band, frame: u32) -> Canvas {
    let mut canvas = Canvas::default();
    let shift = frame as usize;
    for s in 0..STRIP {
        for row in 0..ROWS {
            let front = s < FACE;
            let (column, line) = ((s + shift) % CELL, row as isize - 3);
            let cell = (s + shift) / CELL;
            let mut color = if row == 0 || row + 1 == ROWS {
                look.edge
            } else if (0..5).contains(&line) && column < 5 && !(look.stars && front) && glyph(cell, column, line as usize) {
                look.ink
            } else {
                look.cloth
            };
            if look.stars {
                let (ex, ey) = (s as isize - 12, row as isize - 1);
                if front && (0..7).contains(&ex) && (0..7).contains(&ey) && EMBLEM[ey as usize].as_bytes()[ex as usize] == b'#' {
                    color = look.ink;
                } else if row > 0 && row + 1 < ROWS && noise(s as i64, row as i64 + (frame / 4) as i64 * 131) > 0.9 {
                    color = [0x8C_5CFF, 0x5C_E0B0, 0xFF_FFFF][s % 3];
                }
            }
            let (x, y) = strip_at(s, row);
            canvas.put(x, y, opaque(color));
        }
    }
    canvas.rect([0, 20, 8, 4], |x, y| opaque(if y == 0 || x == 0 { look.edge } else { look.cloth }));
    canvas.rect([0, 24, 8, 4], |x, y| opaque(if y == 3 || x == 0 { look.edge } else { look.cloth }));
    canvas
}

fn top_hat() -> Vec<Bone> {
    vec![bone(
        "hat",
        [0.0; 3],
        vec![
            cube([-6.0, -9.0, -6.0], [12.0, 1.0, 12.0], [0.0, 0.0]),
            cube([-4.0, -17.0, -4.0], [8.0, 8.0, 8.0], [0.0, 13.0]),
            cube([-4.0, -10.5, -4.0], [8.0, 1.5, 8.0], [32.0, 13.0]).inflated(0.15),
        ],
    )]
}

fn top_hat_paint() -> Canvas {
    let mut canvas = Canvas::default();
    let felt = |side: Side, x: usize, w: usize| {
        let shade = if side == Side::Top { 0.12 } else { 0.05 * (x as f64 / w.max(1) as f64 * std::f64::consts::PI).sin() };
        opaque(mix(0x16_161C, WHITE, shade))
    };
    canvas.cube([0, 0], [12, 1, 12], |side, x, _, w, _| felt(side, x, w));
    canvas.cube([0, 13], [8, 8, 8], |side, x, _, w, _| felt(side, x, w));
    canvas.cube([32, 13], [8, 1, 8], |side, x, _, w, _| {
        let buckle = side == Side::Front && (3..5).contains(&x);
        opaque(if buckle { 0xF2_C14E } else { mix(0x5B_2DB8, BLACK, 0.1 * (x as f64 / w.max(1) as f64)) })
    });
    canvas
}

fn halo() -> Vec<Bone> {
    vec![bone(
        "ring",
        [0.0, -11.0, 0.0],
        vec![
            cube([-5.0, -11.5, -5.0], [10.0, 1.0, 1.0], [0.0, 0.0]),
            cube([-5.0, -11.5, 4.0], [10.0, 1.0, 1.0], [0.0, 0.0]),
            cube([-5.0, -11.5, -4.0], [1.0, 1.0, 8.0], [0.0, 4.0]),
            cube([4.0, -11.5, -4.0], [1.0, 1.0, 8.0], [0.0, 4.0]),
        ],
    )]
}

fn float(name: &str, t: f32) -> Motion {
    if name == "ring" { Motion { offset: [0.0, 0.6 * wave(t, 2.0, 0.0), 0.0], rotation: [0.0, t * 0.9, 0.0], ..Motion::REST } } else { Motion::REST }
}

fn halo_paint(frame: u32) -> Canvas {
    let mut canvas = Canvas::default();
    let shine = |x: usize| {
        let glint = ((x as f64 - frame as f64 * 0.7).rem_euclid(14.0) - 2.0).abs() < 1.5;
        opaque(if glint { 0xFF_F8D8 } else { 0xFF_D86B })
    };
    canvas.cube([0, 0], [10, 1, 1], |_, x, _, _, _| shine(x));
    canvas.cube([0, 4], [1, 1, 8], |_, x, _, _, _| shine(x + 5));
    canvas
}

fn crown() -> Vec<Bone> {
    let point = |x: f32, z: f32, tall: f32| cube([x, -10.0 - tall, z], [1.0, tall, 1.0], [0.0, 12.0]);
    vec![bone(
        "crown",
        [0.0; 3],
        vec![
            cube([-4.5, -10.0, -4.5], [9.0, 2.0, 9.0], [0.0, 0.0]),
            point(-4.5, -4.5, 2.0),
            point(3.5, -4.5, 2.0),
            point(-4.5, 3.5, 2.0),
            point(3.5, 3.5, 2.0),
            point(-0.5, -4.5, 2.5),
            point(-0.5, 3.5, 2.5),
        ],
    )]
}

fn crown_paint(frame: u32) -> Canvas {
    let mut canvas = Canvas::default();
    let glint = |x: usize| ((x as f64 - frame as f64 * 0.5).rem_euclid(20.0) - 1.0).abs() < 1.0;
    canvas.cube([0, 0], [9, 2, 9], |side, x, y, _, _| {
        let gem = side != Side::Top && side != Side::Bottom && y == 0 && x % 4 == 2;
        let gem_color = if x == 6 { 0xD8_1B3C } else { 0x2E_6BFF };
        opaque(if gem { gem_color } else if glint(x) { 0xFF_F2B0 } else { mix(0xF2_B632, 0x9C_6A10, y as f64 * 0.4) })
    });
    canvas.cube([0, 12], [1, 3, 1], |_, _, y, _, _| opaque(if y == 0 { 0xFF_F2B0 } else { 0xF2_B632 }));
    canvas
}

fn horns() -> Vec<Bone> {
    let horn = bone(
        "horn_l",
        [-2.5, -8.0, -1.0],
        vec![
            cube([-3.5, -10.0, -2.0], [2.0, 2.0, 2.0], [0.0, 0.0]),
            cube([-3.25, -12.0, -1.75], [1.5, 2.0, 1.5], [0.0, 5.0]),
            cube([-3.0, -13.5, -1.5], [1.0, 1.5, 1.0], [8.0, 0.0]),
        ],
    )
    .turned([-0.2, 0.0, -0.35]);
    vec![horn.mirrored("horn_r"), horn]
}

fn horns_paint() -> Canvas {
    let mut canvas = Canvas::default();
    canvas.rect([0, 0, 16, 10], |_, y| opaque(mix(0xE0_301E, 0x5A_0606, y as f64 / 9.0)));
    canvas.rect([0, 0, 8, 4], |_, y| opaque(mix(0x9A_1010, 0x5A_0606, y as f64 / 4.0)));
    canvas.rect([8, 0, 4, 3], |_, _| opaque(0xFF_6A3A));
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::umbx_flair_model::bounds;

    #[test]
    fn hats_sit_on_the_head() {
        for item in items() {
            let (lo, hi) = bounds(&item.bones).unwrap();
            assert!(lo[1] >= -18.0 && hi[1] <= 1.0, "{}", item.key);
            assert!(lo[0] >= -7.0 && hi[0] <= 7.0, "{}", item.key);
            assert_eq!(item.canvas(0).0.len(), 64 * 64 * 4);
        }
    }

    #[test]
    fn bandana_tails_sway_and_the_halo_floats() {
        assert_ne!(tails("tail_l", 0.3), tails("tail_l", 0.9));
        assert_ne!(tails("tail_r.tail_low_l", 0.3), tails("tail_l", 0.3));
        assert_eq!(tails("band", 0.3), Motion::REST);
        let look = Band { cloth: 1, ink: 2, edge: 3, stars: false };
        assert_ne!(bandana_paint(look, 0), bandana_paint(look, 1));
        assert_eq!(strip_at(70, 2), (6, 12));
        assert_ne!(float("ring", 0.0).rotation, float("ring", 1.0).rotation);
    }
}
