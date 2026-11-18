fn hide(value: Vec<(&Part, [f32; 4], u32, u32, bool)>) -> Vec<(&Part, [f32; 4], u32, u32, bool)> {
    if crate::game::umbx_flair_glue::hides_armor() { Vec::new() } else { value }
}
