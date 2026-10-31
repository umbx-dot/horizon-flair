use super::Game;
use super::umbx_flair_auras as auras;
use super::umbx_flair_catalog as catalog;
use super::umbx_flair_glue::{self as glue, Entry};
use super::umbx_flair_items::{Attach, Item, Slot};
use super::umbx_flair_model::{self as model, Baked};
use glam::{DVec3, Mat4, Vec3};
use horizon_client::hud::HudBatch;
use horizon_client::mesher::Vertex;
use horizon_client::mobs::{Mob, Part, Polygon, render, render_model};

pub(super) const FULL_LIGHT: u32 = 240 | 240 << 8;
const TEXTURE: [f32; 2] = [64.0, 64.0];
const HELMET_GROW: f32 = 1.25;
const HEAD_CENTER: f32 = 0.25;
const ARMOR_LIFT: f32 = 1.0 / 16.0;
const ICON_TILT: f32 = 0.4;
const ICON_SPIN: f32 = 0.8;
const ICON_FILL: f32 = 0.8;

pub(super) struct Shape {
    part: Part,
    center: Vec3,
    extent: f32,
}

pub(super) fn shape(item: &Item) -> Shape {
    let root = Baked {
        name: String::new(),
        pose: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        faces: Vec::new(),
        children: item.bones.iter().map(|b| model::bake(b, [0.0; 3], TEXTURE)).collect(),
    };
    let (lo, hi) = if item.slot == Slot::Aura { auras::VIEW } else { model::bounds(&item.bones).unwrap_or(([-4.0, -8.0, -4.0], [4.0, 0.0, 4.0])) };
    let extent = (0..3).map(|i| hi[i] - lo[i]).fold(1.0, f32::max) / 16.0;
    Shape { part: part(&root), center: Vec3::from(lo).lerp(Vec3::from(hi), 0.5) / 16.0, extent }
}

fn part(baked: &Baked) -> Part {
    Part {
        name: baked.name.clone(),
        pose: baked.pose,
        polygons: baked.faces.iter().map(|f| Polygon { normal: f.normal, vertices: f.vertices }).collect(),
        children: baked.children.iter().map(part).collect(),
    }
}

fn sprite(game: &Game<'_>, name: &str) -> Option<[f32; 4]> {
    game.gui.atlas.sprites.get(name).map(|r| [r.u0, r.v0, r.u1, r.v1])
}

pub(super) fn worn(game: &Game<'_>, mob: &Mob, camera: DVec3, alpha: f64, light: u32, out: &mut Vec<Vertex>) {
    let local = game.player_model.as_ref().is_some_and(|p| p.id == mob.id && p.kind.id == mob.kind.id);
    if !local || !game.native_on(catalog::ID) || game.own_invisible() {
        return;
    }
    let flair = glue::state();
    let seconds = flair.seconds();
    for slot in [Slot::Hat, Slot::Back, Slot::Pet, Slot::Aura] {
        let Some(entry) = flair.worn(slot).map(|i| &flair.entries[i]) else { continue };
        let (Some(shape), Some(uv)) = (&entry.shape, sprite(game, &glue::texture(&entry.item))) else { continue };
        let Some(base) = attach(game, mob, camera, alpha as f32, entry.item.attach, slot) else { continue };
        let light = if entry.item.glow { FULL_LIGHT } else { light };
        render_model(&shape.part, base, &|p| entry.item.motion(&p.name, seconds).apply(p.pose), uv, light, 0, out);
    }
}

fn attach(game: &Game<'_>, mob: &Mob, camera: DVec3, alpha: f32, attach: Attach, slot: Slot) -> Option<Mat4> {
    let data = game.res.mob_data();
    match attach {
        Attach::Head => {
            let head = render::part_transform(mob, alpha, camera, data, "head")?;
            if game.armor[0].is_none() {
                return Some(head);
            }
            let center = Mat4::from_translation(Vec3::new(0.0, -HEAD_CENTER, 0.0));
            Some(head * center * Mat4::from_scale(Vec3::splat(HELMET_GROW)) * center.inverse())
        }
        Attach::Body => {
            let body = render::part_transform(mob, alpha, camera, data, "body")?;
            let lift = match (game.armor[1].is_some(), slot) {
                (true, Slot::Back) => Vec3::new(0.0, 0.0, ARMOR_LIFT),
                (true, Slot::Pet) => Vec3::new(0.0, -ARMOR_LIFT, 0.0),
                _ => Vec3::ZERO,
            };
            Some(body * Mat4::from_translation(lift))
        }
        Attach::Root => Some(render::base_transform(mob, alpha, camera)),
    }
}

pub(super) fn icon(game: &Game<'_>, batch: &mut HudBatch, entry: &Entry, seconds: f32, rect: [f32; 4]) {
    let (Some(shape), Some(uv)) = (&entry.shape, sprite(game, &glue::texture(&entry.item))) else { return };
    let mut verts = Vec::new();
    render_model(&shape.part, Mat4::IDENTITY, &|p| entry.item.motion(&p.name, seconds).apply(p.pose), uv, FULL_LIGHT, 0, &mut verts);
    let [x, y, w, h] = rect;
    let size = w.min(h) / shape.extent * ICON_FILL;
    let gui = Mat4::from_translation(Vec3::new(x + w / 2.0, y + h / 2.0, 0.0))
        * Mat4::from_scale(Vec3::new(size, size, -size))
        * Mat4::from_rotation_x(ICON_TILT)
        * Mat4::from_rotation_y(seconds * ICON_SPIN)
        * Mat4::from_translation(-shape.center);
    let mut quads: Vec<(f32, [[f32; 3]; 4], [[f32; 2]; 4], u32)> = Vec::new();
    for q in verts.chunks_exact(4) {
        let p: [Vec3; 4] = std::array::from_fn(|i| gui.transform_point3(Vec3::from_array(q[i].pos)));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        if n.z >= 0.0 {
            continue;
        }
        let n = n.normalize_or_zero();
        let shade = (0.55 + 0.45 * -n.z - 0.15 * n.y).clamp(0.4, 1.0);
        let c = q[0].color;
        let channel = |s: u32| ((((c >> s) & 255) as f32 * shade) as u32) << s;
        let color = (c & 0xFF00_0000) | channel(0) | channel(8) | channel(16);
        quads.push((p.iter().map(|v| v.z).sum::<f32>() / 4.0, p.map(|v| v.to_array()), std::array::from_fn(|i| q[i].uv), color));
    }
    quads.sort_by(|a, b| a.0.total_cmp(&b.0));
    let start = batch.vertices.len();
    for (_, p, uv, color) in quads {
        batch.quad_3d(p, uv, color);
    }
    batch.depth_since(start, rect);
}
