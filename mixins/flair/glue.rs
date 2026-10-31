use super::Game;
use super::umbx_flair_auras as auras;
use super::umbx_flair_back as back;
use super::umbx_flair_capes as capes;
use super::umbx_flair_catalog::{self as catalog, Layout};
use super::umbx_flair_custom as custom;
use super::umbx_flair_hats as hats;
use super::umbx_flair_items::{Item, SLOTS, Slot};
use super::umbx_flair_pets as pets;
use super::umbx_flair_render::{self as render, Shape};
use super::umbx_flair_screen as screen;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

pub(super) const CAPE: &str = "capes/flair";
pub(super) const FRONT_YAW: f32 = 200.0;
const BACK_YAW: f32 = 20.0;
const HIDE_ARMOR: &str = "hide_armor";

pub(super) struct Entry {
    pub(super) item: Item,
    pub(super) shape: Option<Shape>,
}

pub(super) struct State {
    pub(super) open: bool,
    pub(super) category: usize,
    pub(super) scroll: f32,
    pub(super) entries: Vec<Entry>,
    pub(super) equipped: [Option<String>; 5],
    pub(super) first_person: [bool; 5],
    pub(super) hide_armor: bool,
    pub(super) preview_yaw: f32,
    pub(super) dragging: Option<f32>,
    ready: bool,
    uploaded: Vec<(String, usize)>,
    original: Option<String>,
    started: Option<Instant>,
}

static STATE: Mutex<State> = Mutex::new(State {
    open: false,
    category: 0,
    scroll: 0.0,
    entries: Vec::new(),
    equipped: [None, None, None, None, None],
    first_person: [false; 5],
    hide_armor: false,
    preview_yaw: FRONT_YAW,
    dragging: None,
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

    pub(super) fn wearing(&self) -> Vec<(Slot, usize)> {
        SLOTS.iter().filter_map(|s| self.worn(*s).map(|i| (*s, i))).collect()
    }

    pub(super) fn equip(&mut self, game: &Game<'_>, slot: Slot, key: Option<String>) {
        game.native_set_setting(catalog::ID, slot.key(), key.as_deref().unwrap_or(""));
        if key.is_some() {
            self.preview_yaw = if matches!(slot, Slot::Cape | Slot::Back) { BACK_YAW } else { FRONT_YAW };
        }
        self.equipped[slot as usize] = key;
    }

    pub(super) fn toggle_first_person(&mut self, game: &Game<'_>, slot: Slot) {
        let on = !self.first_person[slot as usize];
        self.first_person[slot as usize] = on;
        game.native_set_setting(catalog::ID, &first_person_key(slot), &on.to_string());
    }

    pub(super) fn toggle_hide_armor(&mut self, game: &Game<'_>) {
        self.hide_armor = !self.hide_armor;
        game.native_set_setting(catalog::ID, HIDE_ARMOR, &self.hide_armor.to_string());
    }

    pub(super) fn load(&mut self, game: &Game<'_>) {
        let items = [capes::items(), hats::items(), back::items(), pets::items(), auras::items(), custom::items(&game.native_data_dir(catalog::ID))];
        self.entries = items.into_iter().flatten().map(|item| Entry { shape: (!item.bones.is_empty()).then(|| render::shape(&item)), item }).collect();
        if !self.ready {
            self.equipped = SLOTS.map(|slot| game.native_setting(catalog::ID, slot.key()).filter(|key| !key.is_empty()));
            self.first_person = SLOTS.map(|slot| game.native_setting(catalog::ID, &first_person_key(slot)).as_deref() == Some("true"));
            self.hide_armor = game.native_setting(catalog::ID, HIDE_ARMOR).as_deref() == Some("true");
            self.ready = true;
        }
        self.uploaded.clear();
    }
}

pub(super) fn toggles(slot: Slot) -> bool {
    matches!(slot, Slot::Pet | Slot::Aura)
}

fn first_person_key(slot: Slot) -> String {
    format!("first_person.{}", slot.key())
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
    let local = game.player_model.as_ref().map(|m| m.id);
    if !game.native_on(catalog::ID) {
        horizon_client::mobs::render::umbx_flair_armor::set(false, local);
        return;
    }
    let mut flair = state();
    flair.started.get_or_insert_with(Instant::now);
    if !flair.ready {
        flair.load(game);
    }
    horizon_client::mobs::render::umbx_flair_armor::set(flair.hide_armor, local);
    let ms = flair.millis();
    let mut wanted: Vec<usize> = SLOTS.iter().filter_map(|s| flair.worn(*s)).collect();
    if flair.open && SLOTS[flair.category] != Slot::Cape {
        let (w, h) = game.gui_size();
        let shown = flair.shown();
        wanted.extend(Layout::new(w, h, screen::previewing(game)).visible(shown.len(), flair.scroll).map(|i| shown[i]));
    }
    wanted.retain(|i| flair.entries[*i].item.textured());
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

pub(super) fn wear(game: &mut Game<'_>, camera: glam::DVec3, alpha: f64, out: &mut Vec<horizon_client::mesher::Vertex>) {
    if !game.native_on(catalog::ID) {
        return;
    }
    cape(game);
    render::first_person(game, camera, alpha, out);
}

fn cape(game: &mut Game<'_>) {
    let mut flair = state();
    let back = flair.worn(Slot::Back).is_some();
    let shown = !back && flair.worn(Slot::Cape).is_some() && game.gui.atlas.sprites.contains_key(CAPE);
    let Some(model) = game.player_model.as_mut() else { return };
    if model.cape.as_deref() != Some(CAPE) {
        flair.original = model.cape.clone();
    }
    model.cape = if shown {
        Some(CAPE.to_owned())
    } else if back {
        None
    } else {
        flair.original.clone()
    };
}
