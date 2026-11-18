#[path = "paint.rs"]
pub mod umbx_flair_paint;
#[path = "model.rs"]
pub mod umbx_flair_model;
#[path = "items.rs"]
pub mod umbx_flair_items;
#[path = "hd_capes.rs"]
pub mod umbx_flair_hd_capes;
#[path = "capes.rs"]
pub mod umbx_flair_capes;
#[path = "bandanas.rs"]
pub mod umbx_flair_bandanas;
#[path = "hats.rs"]
pub mod umbx_flair_hats;
#[path = "back.rs"]
pub mod umbx_flair_back;
#[path = "pets.rs"]
pub mod umbx_flair_pets;
#[path = "critters.rs"]
pub mod umbx_flair_critters;
#[path = "auras.rs"]
pub mod umbx_flair_auras;
#[path = "texture_file.rs"]
pub mod umbx_flair_texture_file;
#[path = "catalog.rs"]
pub mod umbx_flair_catalog;

use mod_api::{self, Hooks};

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {}
