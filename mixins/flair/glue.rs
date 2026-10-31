use super::Game;
use super::umbx_flair_cape_file as cape_file;
use super::umbx_flair_capes as capes;
use super::umbx_flair_catalog::{self as catalog, Hit, Layout};
use super::umbx_flair_screen as screen;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const CAPE: &str = "capes/flair";
const CAPE_KEY: &str = "cape";
const TICK_MS: u128 = 50;
const STYLE_FRAME_MS: u128 = 100;

enum Art {
    Style(usize),
    Frames { frames: Vec<Vec<u8>>, ticks: u32 },
}

pub(super) struct Cape {
    pub(super) key: String,
    pub(super) name: String,
    art: Art,
}

impl Cape {
    fn frame(&self, ms: u128) -> usize {
        match &self.art {
            Art::Style(_) => (ms / STYLE_FRAME_MS) as usize,
            Art::Frames { frames, ticks } => (ms / (u128::from(*ticks) * TICK_MS)) as usize % frames.len(),
        }
    }

    pub(super) fn pixels(&self, ms: u128) -> Cow<'_, [u8]> {
        match &self.art {
            Art::Style(style) => Cow::Owned(capes::cape(*style, self.frame(ms) as u32)),
            Art::Frames { frames, .. } => Cow::Borrowed(&frames[self.frame(ms)]),
        }
    }
}

pub(super) struct State {
    pub(super) open: bool,
    pub(super) category: usize,
    pub(super) scroll: usize,
    pub(super) capes: Vec<Cape>,
    pub(super) equipped: Option<String>,
    ready: bool,
    uploaded: Option<(String, usize)>,
    original: Option<String>,
    started: Option<Instant>,
}

static STATE: Mutex<State> = Mutex::new(State {
    open: false,
    category: 0,
    scroll: 0,
    capes: Vec::new(),
    equipped: None,
    ready: false,
    uploaded: None,
    original: None,
    started: None,
});

pub(super) fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl State {
    pub(super) fn millis(&mut self) -> u128 {
        self.started.get_or_insert_with(Instant::now).elapsed().as_millis()
    }

    pub(super) fn cards(&self) -> usize {
        if self.category == 0 { self.capes.len() } else { 0 }
    }

    fn load(&mut self, game: &Game<'_>) {
        self.capes = capes::STYLES.iter().enumerate().map(|(i, name)| Cape { key: name.to_lowercase(), name: (*name).into(), art: Art::Style(i) }).collect();
        self.capes.extend(custom_capes(game));
        if !self.ready {
            self.equipped = game.native_setting(catalog::ID, CAPE_KEY).filter(|key| !key.is_empty());
            self.ready = true;
        }
        self.uploaded = None;
    }
}

fn custom_capes(game: &Game<'_>) -> Vec<Cape> {
    let dir = game.native_data_dir(catalog::ID).join("capes");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))).collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|path| custom_cape(path).map_err(|error| log::warn!("Flair: cape {} skipped: {error}", path.display())).ok())
        .collect()
}

fn custom_cape(path: &Path) -> Result<Cape, String> {
    let (width, height, rgba) = horizon_client::hud::decode(path).map_err(|e| e.to_string())?;
    let frames = cape_file::frames(width, height, &rgba)?;
    let file = path.file_name().unwrap_or_default().to_string_lossy();
    let name = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    Ok(Cape { key: format!("custom:{file}"), name, art: Art::Frames { frames, ticks: frametime(path) } })
}

fn frametime(path: &Path) -> u32 {
    let mut meta = path.as_os_str().to_owned();
    meta.push(".mcmeta");
    let Ok(text) = std::fs::read_to_string(&meta) else { return 1 };
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(json) => json["animation"]["frametime"].as_u64().map_or(1, |t| t.clamp(1, 1200) as u32),
        Err(error) => {
            log::warn!("Flair: {} ignored: {error}", Path::new(&meta).display());
            1
        }
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
        WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => click(game, &mut flair, Layout::new(size.0, size.1), cursor),
        WindowEvent::MouseInput { .. } => {}
        WindowEvent::MouseWheel { delta, .. } => {
            let steps = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(p) => p.y as f32,
            };
            let max = Layout::new(size.0, size.1).max_scroll(flair.cards());
            flair.scroll = if steps > 0.0 { flair.scroll.saturating_sub(1) } else if steps < 0.0 { (flair.scroll + 1).min(max) } else { flair.scroll };
        }
        _ => return ControlFlow::Continue(()),
    }
    ControlFlow::Break(())
}

fn click(game: &mut Game<'_>, flair: &mut State, layout: Layout, cursor: [f32; 2]) {
    match layout.hit(cursor, flair.cards(), flair.scroll) {
        Hit::Panel => return,
        Hit::Outside => flair.open = false,
        Hit::Category(i) => {
            flair.category = i;
            flair.scroll = 0;
        }
        Hit::Card(i) => {
            let key = flair.capes[i].key.clone();
            flair.equipped = (flair.equipped.as_deref() != Some(key.as_str())).then_some(key);
            game.native_set_setting(catalog::ID, CAPE_KEY, flair.equipped.as_deref().unwrap_or(""));
        }
    }
    game.audio.play_button_click();
}

pub(super) fn wear(game: &mut Game<'_>) {
    if !game.native_on(catalog::ID) || game.player_model.is_none() {
        return;
    }
    let mut flair = state();
    if !flair.ready {
        flair.load(game);
    }
    let ms = flair.millis();
    let worn = flair.equipped.as_ref().and_then(|key| flair.capes.iter().position(|c| &c.key == key));
    if let Some(i) = worn {
        let cape = &flair.capes[i];
        let shown = (cape.key.clone(), cape.frame(ms));
        if flair.uploaded.as_ref() != Some(&shown) || !game.gui.atlas.sprites.contains_key(CAPE) {
            game.store_skin(CAPE.to_owned(), cape.pixels(ms).into_owned());
            flair.uploaded = Some(shown);
        }
    }
    game.skin_downloads.borrow_mut().drawn(CAPE);
    let shown = worn.is_some() && game.gui.atlas.sprites.contains_key(CAPE);
    let Some(model) = game.player_model.as_mut() else { return };
    if model.cape.as_deref() != Some(CAPE) {
        flair.original = model.cape.clone();
    }
    model.cape = if shown { Some(CAPE.to_owned()) } else { flair.original.clone() };
}
