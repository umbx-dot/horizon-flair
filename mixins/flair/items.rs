use super::umbx_flair_model::{Animation, Bone, Motion};
use super::umbx_flair_paint::Canvas;
use std::borrow::Cow;

pub const FRAME_MS: u128 = 100;
pub const TICK_MS: u128 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Cape,
    Hat,
    Back,
    Pet,
    Aura,
}

pub const SLOTS: [Slot; 5] = [Slot::Cape, Slot::Hat, Slot::Back, Slot::Pet, Slot::Aura];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attach {
    Head,
    Body,
    Root,
}

impl Slot {
    pub fn title(self) -> &'static str {
        ["Capes", "Hats", "Back", "Pets", "Auras"][self as usize]
    }

    pub fn key(self) -> &'static str {
        ["cape", "hat", "back", "pet", "aura"][self as usize]
    }

    pub fn parse(text: &str) -> Option<Slot> {
        SLOTS.into_iter().find(|s| s.key() == text)
    }

    pub fn attach(self) -> Attach {
        match self {
            Slot::Hat => Attach::Head,
            Slot::Aura => Attach::Root,
            Slot::Cape | Slot::Back | Slot::Pet => Attach::Body,
        }
    }
}

impl Attach {
    pub fn parse(text: &str) -> Option<Attach> {
        [("head", Attach::Head), ("body", Attach::Body), ("root", Attach::Root)].into_iter().find(|(k, _)| *k == text).map(|(_, a)| a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub a: [f64; 3],
    pub b: [f64; 3],
    pub color: u32,
    pub width: f32,
}

pub type Effect = fn(f64, f64, &mut Vec<Stroke>);
pub type Painter = Box<dyn Fn(u32) -> Canvas + Send + Sync>;
pub type Mover = Box<dyn Fn(&str, f32) -> Motion + Send + Sync>;

pub enum Paint {
    Still(Canvas),
    Live(Painter),
    Frames { frames: Vec<Canvas>, ticks: u32 },
}

pub enum Moves {
    Still,
    Live(Mover),
    Keys(Animation),
}

pub struct Item {
    pub key: String,
    pub name: String,
    pub slot: Slot,
    pub attach: Attach,
    pub glow: bool,
    pub bones: Vec<Bone>,
    pub paint: Paint,
    pub moves: Moves,
    pub effect: Option<Effect>,
}

impl Item {
    pub fn new(key: &str, name: &str, slot: Slot, bones: Vec<Bone>, paint: Paint, moves: Moves) -> Self {
        Item { key: key.into(), name: name.into(), slot, attach: slot.attach(), glow: false, bones, paint, moves, effect: None }
    }

    pub fn with_effect(self, effect: Effect) -> Self {
        Item { effect: Some(effect), ..self }
    }

    pub fn textured(&self) -> bool {
        self.slot == Slot::Cape || !self.bones.is_empty()
    }

    pub fn glowing(self) -> Self {
        Item { glow: true, ..self }
    }

    pub fn frame(&self, ms: u128) -> usize {
        match &self.paint {
            Paint::Still(_) => 0,
            Paint::Live(_) => (ms / FRAME_MS) as usize,
            Paint::Frames { frames, ticks } => (ms / (u128::from(*ticks) * TICK_MS)) as usize % frames.len(),
        }
    }

    pub fn canvas(&self, ms: u128) -> Cow<'_, Canvas> {
        match &self.paint {
            Paint::Still(canvas) => Cow::Borrowed(canvas),
            Paint::Live(paint) => Cow::Owned(paint(self.frame(ms) as u32)),
            Paint::Frames { frames, .. } => Cow::Borrowed(&frames[self.frame(ms)]),
        }
    }

    pub fn motion(&self, bone: &str, seconds: f32) -> Motion {
        match &self.moves {
            Moves::Still => Motion::REST,
            Moves::Live(mover) => mover(bone, seconds),
            Moves::Keys(animation) => animation.motion(bone, seconds),
        }
    }
}

pub fn wave(seconds: f32, speed: f32, phase: f32) -> f32 {
    (seconds * speed + phase).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_round_trip_their_keys() {
        for slot in SLOTS {
            assert_eq!(Slot::parse(slot.key()), Some(slot));
        }
        assert_eq!(Slot::Hat.attach(), Attach::Head);
        assert_eq!(Attach::parse("root"), Some(Attach::Root));
        assert_eq!(Slot::parse("shoes"), None);
    }

    #[test]
    fn frames_follow_their_paint() {
        let frames = vec![Canvas::default(); 3];
        let item = Item::new("x", "X", Slot::Cape, Vec::new(), Paint::Frames { frames, ticks: 2 }, Moves::Still);
        assert_eq!(item.frame(0), 0);
        assert_eq!(item.frame(100), 1);
        assert_eq!(item.frame(300), 0);
        let live = Item::new("y", "Y", Slot::Hat, Vec::new(), Paint::Live(Box::new(|_| Canvas::default())), Moves::Still);
        assert_eq!(live.frame(250), 2);
        assert_eq!(live.motion("any", 1.0), Motion::REST);
    }
}
