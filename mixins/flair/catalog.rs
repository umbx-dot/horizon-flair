pub const ID: &str = "flair";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "A cosmetics catalog: animated capes and your own capes, only you see them.";
pub const CATEGORIES: [&str; 5] = ["Capes", "Hats", "Back", "Pets", "Auras"];
pub const CARD_W: f32 = 56.0;
pub const CARD_H: f32 = 74.0;
pub const HEADER: f32 = 22.0;
pub const SIDEBAR: f32 = 72.0;
pub const CATEGORY_H: f32 = 18.0;
const GAP: f32 = 6.0;
const PAD: f32 = 8.0;
const MARGIN: f32 = 12.0;
const MAX_W: f32 = 380.0;
const MAX_H: f32 = 240.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Category(usize),
    Card(usize),
    Panel,
    Outside,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub panel: [f32; 4],
    pub content: [f32; 4],
    pub columns: usize,
    pub rows: usize,
}

pub fn inside(rect: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= rect[0] && p[0] < rect[0] + rect[2] && p[1] >= rect[1] && p[1] < rect[1] + rect[3]
}

impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let (w, h) = ((width - 2.0 * MARGIN).min(MAX_W), (height - 2.0 * MARGIN).min(MAX_H));
        let (x, y) = (((width - w) / 2.0).floor(), ((height - h) / 2.0).floor());
        let content = [x + SIDEBAR + PAD, y + HEADER + PAD, w - SIDEBAR - 2.0 * PAD, h - HEADER - 2.0 * PAD];
        let fit = |space: f32, item: f32| (((space + GAP) / (item + GAP)).floor() as usize).max(1);
        Layout { panel: [x, y, w, h], content, columns: fit(content[2], CARD_W), rows: fit(content[3], CARD_H) }
    }

    pub fn header(&self) -> [f32; 4] {
        [self.panel[0], self.panel[1], self.panel[2], HEADER]
    }

    pub fn sidebar(&self) -> [f32; 4] {
        [self.panel[0], self.panel[1] + HEADER, SIDEBAR, self.panel[3] - HEADER]
    }

    pub fn category(&self, i: usize) -> [f32; 4] {
        [self.panel[0] + 4.0, self.panel[1] + HEADER + 6.0 + i as f32 * (CATEGORY_H + 2.0), SIDEBAR - 8.0, CATEGORY_H]
    }

    pub fn slots(&self) -> usize {
        self.columns * self.rows
    }

    pub fn card(&self, slot: usize) -> [f32; 4] {
        let (col, row) = ((slot % self.columns) as f32, (slot / self.columns) as f32);
        [self.content[0] + col * (CARD_W + GAP), self.content[1] + row * (CARD_H + GAP), CARD_W, CARD_H]
    }

    pub fn max_scroll(&self, cards: usize) -> usize {
        cards.div_ceil(self.columns).saturating_sub(self.rows)
    }

    pub fn hit(&self, p: [f32; 2], cards: usize, scroll: usize) -> Hit {
        if !inside(self.panel, p) {
            return Hit::Outside;
        }
        if let Some(i) = (0..CATEGORIES.len()).find(|i| inside(self.category(*i), p)) {
            return Hit::Category(i);
        }
        (0..self.slots())
            .map(|slot| (slot, scroll * self.columns + slot))
            .find(|(slot, index)| *index < cards && inside(self.card(*slot), p))
            .map_or(Hit::Panel, |(_, index)| Hit::Card(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(rect: [f32; 4]) -> [f32; 2] {
        [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0]
    }

    #[test]
    fn panel_and_cards_stay_on_small_and_large_screens() {
        for (w, h) in [(320.0, 180.0), (480.0, 270.0), (960.0, 540.0)] {
            let layout = Layout::new(w, h);
            let [x, y, pw, ph] = layout.panel;
            assert!(x >= 0.0 && y >= 0.0 && x + pw <= w && y + ph <= h);
            let last = layout.card(layout.slots() - 1);
            assert!(last[0] + last[2] <= x + pw && last[1] + last[3] <= y + ph, "{w}x{h}");
            assert!(layout.category(CATEGORIES.len() - 1)[1] + CATEGORY_H <= y + ph);
        }
    }

    #[test]
    fn hits_follow_the_scroll() {
        let layout = Layout::new(480.0, 270.0);
        let first = center(layout.card(0));
        assert_eq!(layout.hit(first, 20, 0), Hit::Card(0));
        assert_eq!(layout.hit(first, 20, 1), Hit::Card(layout.columns));
        assert_eq!(layout.hit(first, 0, 0), Hit::Panel);
        assert_eq!(layout.hit(center(layout.category(2)), 20, 0), Hit::Category(2));
        assert_eq!(layout.hit([1.0, 1.0], 20, 0), Hit::Outside);
    }

    #[test]
    fn scroll_stops_at_the_last_row() {
        let layout = Layout::new(480.0, 270.0);
        assert_eq!(layout.max_scroll(layout.slots()), 0);
        assert_eq!(layout.max_scroll(layout.slots() + 1), 1);
    }
}
