use super::Game;
use super::umbx_flair_auras as auras;
use super::umbx_flair_back as back;
use super::umbx_flair_capes as capes;
use super::umbx_flair_catalog::{self as catalog, Hit, Layout};
use super::umbx_flair_custom as custom;
use super::umbx_flair_hats as hats;
use super::umbx_flair_items::{Item, SLOTS, Slot};
use super::umbx_flair_pets as pets;
use super::umbx_flair_render::{self as render, Shape};
use super::umbx_flair_screen as screen;
use std::ops::ControlFlow;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) const CAPE: &str = "capes/flair";

pub(super) struct Entry {
    pub(super) item: Item,
    pub(super) shape: Option<Shape>,
}

pub(super) struct State {
    pub(super) open: bool,
    pub(super) category: usize,
    pub(super) scroll: usize,
    pub(super) entries: Vec<Entry>,
    pub(super) equipped: [Option<String>; 5],
    ready: bool,
    uploaded: Vec<(String, usize)>,
    original: Option<String>,
    started: Option<Instant>,
}

static STATE: Mutex<State> = Mutex::new(State {
    open: false,
    category: 0,
    scroll: 0,
    entries: Vec::new(),
    equipped: [None, None, None, None, None],
    ready: false,
    uploaded: Vec::new(),
    original: None,
    started: None,
});

pub(super) fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn texture(item: &Item) -> String {
    if item.slot == Slot::Cape { CAPE.to_owned() } else { format!("flair/{}", item.key) }
}

impl State {
    pub(super) fn millis(&self) -> u128 {
        self.started.map_or(0, |s| s.elapsed().as_millis())
    }

    pub(super) fn seconds(&self) -> f32 {
        self.millis() as f32 / 1000.0
    }

    pub(super) fn shown(&self) -> Vec<usize> {
        (0..self.entries.len()).filter(|i| self.entries[*i].item.slot == SLOTS[self.category]).collect()
    }

    pub(super) fn worn(&self, slot: Slot) -> Option<usize> {
        let key = self.equipped[slot as usize].as_deref()?;
        self.entries.iter().position(|e| e.item.slot == slot && e.item.key == key)
    }

    fn load(&mut self, game: &Game<'_>) {
        let items = [capes::items(), hats::items(), back::items(), pets::items(), auras::items(), custom::items(&game.native_data_dir(catalog::ID))];
        self.entries = items.into_iter().flatten().map(|item| Entry { shape: (item.slot != Slot::Cape).then(|| render::shape(&item)), item }).collect();
        if !self.ready {
            self.equipped = SLOTS.map(|slot| game.native_setting(catalog::ID, slot.key()).filter(|key| !key.is_empty()));
            self.ready = true;
        }
        self.uploaded.clear();
    }
}

pub(super) fn listed(mods: &mut Vec<super::native_hooks::NativeMod>) {
    mods.push(super::native_hooks::NativeMod {
        id: catalog::ID.into(),
        title: format!("Flair {}", catalog::VERSION),
        description: catalog::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

pub(super) fn frame(game: &mut Game<'_>) {
    if !game.native_on(catalog::ID) {
        return;
    }
    let mut flair = state();
    flair.started.get_or_insert_with(Instant::now);
    if !flair.ready {
        flair.load(game);
    }
    let ms = flair.millis();
    let mut wanted: Vec<usize> = SLOTS.iter().filter_map(|s| flair.worn(*s)).collect();
    if flair.open && SLOTS[flair.category] != Slot::Cape {
        let (w, h) = game.gui_size();
        let shown = flair.shown();
        wanted.extend(Layout::new(w, h, screen::previewing(game)).visible(shown.len(), flair.scroll).map(|i| shown[i]));
    }
    for i in wanted {
        let item = &flair.entries[i].item;
        let (location, frame) = (texture(item), item.frame(ms));
        let current = flair.uploaded.iter().position(|(l, _)| *l == location);
        if current.map(|c| flair.uploaded[c].1) != Some(frame) || !game.gui.atlas.sprites.contains_key(&location) {
            game.store_skin(location.clone(), item.canvas(ms).into_owned().0);
            match current {
                Some(c) => flair.uploaded[c].1 = frame,
                None => flair.uploaded.push((location.clone(), frame)),
            }
        }
        game.skin_downloads.borrow_mut().drawn(&location);
    }
}

pub(super) fn window(game: &mut Game<'_>, event: &WindowEvent) -> ControlFlow<()> {
    if !game.native_on(catalog::ID) {
        return ControlFlow::Continue(());
    }
    let size = game.gui_size();
    let cursor = [game.cursor.0, game.cursor.1];
    let mut flair = state();
    if !flair.open {
        let pressed = matches!(event, WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. });
        if !pressed || !screen::button(game, size).is_some_and(|rect| catalog::inside(rect, cursor)) {
            return ControlFlow::Continue(());
        }
        flair.load(game);
        flair.open = true;
        flair.scroll = 0;
        game.audio.play_button_click();
        return ControlFlow::Break(());
    }
    let layout = Layout::new(size.0, size.1, screen::previewing(game));
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            if let PhysicalKey::Code(code) = event.physical_key {
                if event.state == ElementState::Pressed {
                    flair.open &= code != KeyCode::Escape;
                    game.physical_keys.insert(code);
                } else {
                    game.physical_keys.remove(&code);
                }
            }
        }
        WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => click(game, &mut flair, layout, cursor),
        WindowEvent::MouseInput { .. } => {}
        WindowEvent::MouseWheel { delta, .. } => {
            let steps = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(p) => p.y as f32,
            };
            let max = layout.max_scroll(flair.shown().len());
            flair.scroll = if steps > 0.0 { flair.scroll.saturating_sub(1) } else if steps < 0.0 { (flair.scroll + 1).min(max) } else { flair.scroll };
        }
        _ => return ControlFlow::Continue(()),
    }
    ControlFlow::Break(())
}

fn click(game: &mut Game<'_>, flair: &mut State, layout: Layout, cursor: [f32; 2]) {
    let shown = flair.shown();
    match layout.hit(cursor, SLOTS.len(), shown.len(), flair.scroll) {
        Hit::Panel => return,
        Hit::Outside => flair.open = false,
        Hit::Category(i) => {
            flair.category = i;
            flair.scroll = 0;
        }
        Hit::Card(i) => {
            let item = &flair.entries[shown[i]].item;
            let (slot, key) = (item.slot, item.key.clone());
            let worn = &mut flair.equipped[slot as usize];
            *worn = (worn.as_deref() != Some(key.as_str())).then_some(key);
            game.native_set_setting(catalog::ID, slot.key(), worn.as_deref().unwrap_or(""));
        }
    }
    game.audio.play_button_click();
}

pub(super) fn wear(game: &mut Game<'_>) {
    if !game.native_on(catalog::ID) {
        return;
    }
    let mut flair = state();
    let shown = flair.worn(Slot::Cape).is_some() && game.gui.atlas.sprites.contains_key(CAPE);
    let Some(model) = game.player_model.as_mut() else { return };
    if model.cape.as_deref() != Some(CAPE) {
        flair.original = model.cape.clone();
    }
    model.cape = if shown { Some(CAPE.to_owned()) } else { flair.original.clone() };
}
