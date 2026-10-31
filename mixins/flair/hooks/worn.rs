fn flair_worn(
    game: &crate::game::Game<'_>,
    mob: &horizon_client::mobs::Mob,
    camera: glam::DVec3,
    alpha: f64,
    light: u32,
    out: &mut Vec<horizon_client::mesher::Vertex>,
) {
    crate::game::umbx_flair_render::worn(game, mob, camera, alpha, light, out);
}
