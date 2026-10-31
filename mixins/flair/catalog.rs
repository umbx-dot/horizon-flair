pub const ID: &str = "flair";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "A cosmetics catalog: animated capes, hats, wings, shoulder pets and auras, and your own. Only you see them.";
pub const CARD_W: f32 = 56.0;
pub const CARD_H: f32 = 74.0;
pub const HEADER: f32 = 22.0;
pub const SIDEBAR: f32 = 72.0;
pub const CATEGORY_H: f32 = 18.0;
pub const PREVIEW_W: f32 = 96.0;
const GAP: f32 = 6.0;
const PAD: f32 = 8.0;
const MARGIN: f32 = 12.0;
const MAX_W: f32 = 460.0;
const MAX_H: f32 = 250.0;

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
    pub preview: Option<[f32; 4]>,
    pub columns: usize,
    pub rows: usize,
}

pub fn inside(rect: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= rect[0] && p[0] < rect[0] + rect[2] && p[1] >= rect[1] && p[1] < rect[1] + rect[3]
}

impl Layout {
    pub fn new(width: f32, height: f32, preview: bool) -> Self {
        let (w, h) = ((width - 2.0 * MARGIN).min(MAX_W), (height - 2.0 * MARGIN).min(MAX_H));
        let (x, y) = (((width - w) / 2.0).floor(), ((height - h) / 2.0).floor());
        let side = if preview && w - SIDEBAR - PREVIEW_W - 2.0 * PAD >= CARD_W { PREVIEW_W } else { 0.0 };
        let content = [x + SIDEBAR + PAD, y + HEADER + PAD, w - SIDEBAR - side - 2.0 * PAD, h - HEADER - 2.0 * PAD];
        let fit = |space: f32, item: f32| (((space + GAP) / (item + GAP)).floor() as usize).max(1);
        Layout {
            panel: [x, y, w, h],
            content,
            preview: (side > 0.0).then_some([x + w - side, y + HEADER, side, h - HEADER]),
            columns: fit(content[2], CARD_W),
            rows: fit(content[3], CARD_H),
        }
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

    pub fn visible(&self, cards: usize, scroll: usize) -> std::ops::Range<usize> {
        let first = (scroll * self.columns).min(cards);
        first..(first + self.slots()).min(cards)
    }

    pub fn hit(&self, p: [f32; 2], categories: usize, cards: usize, scroll: usize) -> Hit {
        if !inside(self.panel, p) {
            return Hit::Outside;
        }
        if let Some(i) = (0..categories).find(|i| inside(self.category(*i), p)) {
            return Hit::Category(i);
        }
        let first = scroll * self.columns;
        self.visible(cards, scroll).find(|index| inside(self.card(index - first), p)).map_or(Hit::Panel, Hit::Card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(rect: [f32; 4]) -> [f32; 2] {
        [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0]
    }

    #[test]
    fn panel_cards_and_preview_stay_on_screen() {
        for (w, h) in [(320.0, 180.0), (480.0, 270.0), (960.0, 540.0)] {
            for preview in [false, true] {
                let layout = Layout::new(w, h, preview);
                let [x, y, pw, ph] = layout.panel;
                assert!(x >= 0.0 && y >= 0.0 && x + pw <= w && y + ph <= h);
                let last = layout.card(layout.slots() - 1);
                let right = layout.preview.map_or(x + pw, |p| p[0]);
                assert!(last[0] + last[2] <= right && last[1] + last[3] <= y + ph, "{w}x{h} {preview}");
                assert!(layout.category(4)[1] + CATEGORY_H <= y + ph);
            }
        }
        assert!(Layout::new(480.0, 270.0, true).preview.is_some());
        assert!(Layout::new(480.0, 270.0, false).preview.is_none());
    }

    #[test]
    fn hits_follow_the_scroll() {
        let layout = Layout::new(480.0, 270.0, true);
        let first = center(layout.card(0));
        assert_eq!(layout.hit(first, 5, 20, 0), Hit::Card(0));
        assert_eq!(layout.hit(first, 5, 20, 1), Hit::Card(layout.columns));
        assert_eq!(layout.hit(first, 5, 0, 0), Hit::Panel);
        assert_eq!(layout.hit(center(layout.category(2)), 5, 20, 0), Hit::Category(2));
        assert_eq!(layout.hit([1.0, 1.0], 5, 20, 0), Hit::Outside);
    }

    #[test]
    fn scroll_and_visible_cards_stop_at_the_end() {
        let layout = Layout::new(480.0, 270.0, false);
        assert_eq!(layout.max_scroll(layout.slots()), 0);
        assert_eq!(layout.max_scroll(layout.slots() + 1), 1);
        assert_eq!(layout.visible(3, 0), 0..3);
        assert_eq!(layout.visible(layout.slots() + 2, 1), layout.columns..layout.slots() + 2);
    }
}
