use super::umbx_flair_capes as capes;
use super::umbx_flair_catalog::{self as catalog, CARD_H, CARD_W, CATEGORIES, Layout};
use super::umbx_flair_glue::{self as glue, State};
use super::{Game, Screen};
use horizon_client::hud::HudBatch;
use horizon_client::text::Text;
use horizon_client::title::Mode;
use horizon_client::ui;

const BACKDROP: u32 = 0x9006_0609;
const PANEL: u32 = 0xF212_1219;
const HEADER: u32 = 0xFF19_1923;
const SIDEBAR: u32 = 0xFF15_151E;
const CARD: u32 = 0xFF1E_1E2A;
const CARD_HOVER: u32 = 0xFF29_2939;
const SELECTED: u32 = 0xFF26_2238;
const ACCENT: u32 = 0xFF9B_6BFF;
const TEXT: u32 = 0xEC_ECF4;
const MUTED: u32 = 0x8A_8A9C;
const PIXEL: f32 = 3.0;
const TITLE_BUTTON_Y: f32 = 96.0;
const PAUSE_GRID: [f32; 2] = [212.0, 166.0];
const PAUSE_LINKS: f32 = 24.0;
const PAUSE_BUTTON: [f32; 3] = [4.0, 98.0, 52.0];

pub(super) fn button(game: &Game<'_>, (w, h): (f32, f32)) -> Option<[f32; 4]> {
    if matches!(game.screen, Screen::Options | Screen::Win) || game.server_packs.prompt.is_some() || game.code_of_conduct.is_some() || game.dialog_open() {
        return None;
    }
    if let Some(title) = &game.title {
        let (w, h) = (w.ceil(), h.ceil());
        return matches!(title.mode, Mode::Main).then(|| [(w / 2.0).floor() - 100.0, (h / 4.0).floor() + TITLE_BUTTON_Y, 200.0, 20.0]);
    }
    if game.screen != Screen::Paused || !game.pause_menu || game.player.is_none() {
        return None;
    }
    let links = if game.server_links().is_empty() { 0.0 } else { PAUSE_LINKS };
    let x0 = ((w - PAUSE_GRID[0]) * 0.5).floor();
    let y0 = ((h - PAUSE_GRID[1] - links) * 0.25).floor();
    Some([x0 + PAUSE_BUTTON[0], y0 + PAUSE_BUTTON[1], PAUSE_BUTTON[2], 20.0])
}

pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch) {
    if !game.native_on(catalog::ID) {
        return;
    }
    let mut flair = glue::state();
    let Some(rect) = button(game, (batch.width, batch.height)) else {
        flair.open = false;
        return;
    };
    if flair.open {
        catalog_view(game, batch, &mut flair);
    } else {
        let hover = ui::hovered(game.cursor, rect[0], rect[1], rect[2], rect[3]);
        ui::button(batch, &game.gui.atlas, rect[0], rect[1], rect[2], rect[3], &Text::literal("Flair"), true, hover);
    }
}

fn catalog_view(game: &Game<'_>, batch: &mut HudBatch, flair: &mut State) {
    let layout = Layout::new(batch.width, batch.height);
    let cursor = [game.cursor.0, game.cursor.1];
    game.native_fill(batch, [0.0, 0.0, batch.width, batch.height], BACKDROP);
    game.native_fill(batch, layout.panel, PANEL);
    let header = layout.header();
    game.native_fill(batch, header, HEADER);
    game.native_fill(batch, [header[0], header[1] + header[3] - 1.0, header[2], 1.0], ACCENT);
    game.native_text(batch, "Flair", [header[0] + 8.0, header[1] + 7.0], TEXT, true);
    let hint = "Esc to close";
    game.native_text(batch, hint, [header[0] + header[2] - 8.0 - game.native_text_width(hint), header[1] + 7.0], MUTED, false);
    game.native_fill(batch, layout.sidebar(), SIDEBAR);
    for (i, name) in CATEGORIES.iter().enumerate() {
        let rect = layout.category(i);
        let chosen = i == flair.category;
        if chosen || catalog::inside(rect, cursor) {
            game.native_fill(batch, rect, if chosen { SELECTED } else { CARD_HOVER });
        }
        if chosen {
            game.native_fill(batch, [rect[0], rect[1], 2.0, rect[3]], ACCENT);
        }
        game.native_text(batch, name, [rect[0] + 7.0, rect[1] + 5.0], if chosen { TEXT } else { MUTED }, false);
    }
    if flair.cards() == 0 {
        let note = "Coming soon";
        let [x, y, w, h] = layout.content;
        game.native_text(batch, note, [x + ((w - game.native_text_width(note)) / 2.0).floor(), y + (h / 2.0).floor() - 4.0], MUTED, false);
        return;
    }
    let ms = flair.millis();
    let first = flair.scroll * layout.columns;
    for (slot, cape) in flair.capes.iter().enumerate().skip(first).take(layout.slots()).map(|(i, c)| (i - first, c)) {
        let rect = layout.card(slot);
        let worn = flair.equipped.as_deref() == Some(cape.key.as_str());
        game.native_fill(batch, rect, if catalog::inside(rect, cursor) { CARD_HOVER } else { CARD });
        if worn {
            outline(game, batch, rect);
        }
        let pixels = cape.pixels(ms);
        let (px, py) = (rect[0] + ((CARD_W - capes::FACE_W as f32 * PIXEL) / 2.0).floor(), rect[1] + 6.0);
        for v in 0..capes::FACE_H {
            for u in 0..capes::FACE_W {
                let argb = capes::front(&pixels, u, v);
                if argb >> 24 != 0 {
                    game.native_fill(batch, [px + u as f32 * PIXEL, py + v as f32 * PIXEL, PIXEL, PIXEL], argb);
                }
            }
        }
        let name = fitted(game, &cape.name, CARD_W - 4.0);
        let x = rect[0] + ((CARD_W - game.native_text_width(&name)) / 2.0).floor();
        game.native_text(batch, &name, [x, rect[1] + CARD_H - 13.0], if worn { ACCENT & 0xFF_FFFF } else { TEXT }, false);
    }
    let max = layout.max_scroll(flair.cards());
    if max > 0 {
        let [x, y, w, h] = layout.content;
        let thumb = h / (max + 1) as f32;
        game.native_fill(batch, [x + w + 3.0, y + thumb * flair.scroll as f32, 2.0, thumb], ACCENT);
    }
}

fn outline(game: &Game<'_>, batch: &mut HudBatch, [x, y, w, h]: [f32; 4]) {
    for edge in [[x, y, w, 1.0], [x, y + h - 1.0, w, 1.0], [x, y, 1.0, h], [x + w - 1.0, y, 1.0, h]] {
        game.native_fill(batch, edge, ACCENT);
    }
}

fn fitted(game: &Game<'_>, name: &str, width: f32) -> String {
    if game.native_text_width(name) <= width {
        return name.to_owned();
    }
    let mut short: String = name.to_owned();
    while !short.is_empty() && game.native_text_width(&format!("{short}..")) > width {
        short.pop();
    }
    format!("{short}..")
}
