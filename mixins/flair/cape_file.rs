pub const SIZE: usize = 64;
pub const MAX_FRAMES: usize = 64;
const ROWS: usize = 32;

pub fn frames(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let (width, height) = (width as usize, height as usize);
    if rgba.len() != width * height * 4 {
        return Err(format!("{width}x{height} image with {} bytes", rgba.len()));
    }
    if width == 0 || width % SIZE != 0 {
        return Err(format!("width {width} is not a multiple of 64"));
    }
    let scale = width / SIZE;
    let frame_height = ROWS * scale;
    if height == 0 || height % frame_height != 0 {
        return Err(format!("height {height} is not a multiple of {frame_height} (half the width)"));
    }
    let count = height / frame_height;
    if count > MAX_FRAMES {
        return Err(format!("{count} frames, at most {MAX_FRAMES}"));
    }
    Ok((0..count)
        .map(|frame| {
            let mut out = vec![0; SIZE * SIZE * 4];
            for y in 0..ROWS {
                let row = (frame * frame_height + y * scale) * width;
                for x in 0..SIZE {
                    let from = (row + x * scale) * 4;
                    let to = (y * SIZE + x) * 4;
                    out[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
                }
            }
            out
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: usize, height: usize) -> Vec<u8> {
        (0..width * height).flat_map(|i| [(i % 251) as u8, (i / width) as u8, (i % width) as u8, 0xFF]).collect()
    }

    #[test]
    fn a_plain_cape_fills_the_top_half() {
        let rgba = image(64, 32);
        let frames = frames(64, 32, &rgba).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0][..64 * 32 * 4], rgba[..]);
        assert!(frames[0][64 * 32 * 4..].iter().all(|b| *b == 0));
    }

    #[test]
    fn hd_strips_scale_down_into_frames() {
        let rgba = image(128, 128);
        let frames = frames(128, 128, &rgba).unwrap();
        assert_eq!(frames.len(), 2);
        let pixel = |frame: &[u8], x: usize, y: usize| frame[(y * SIZE + x) * 4..][..4].to_vec();
        let source = |x: usize, y: usize| rgba[(y * 128 + x) * 4..][..4].to_vec();
        assert_eq!(pixel(&frames[0], 5, 7), source(10, 14));
        assert_eq!(pixel(&frames[1], 5, 7), source(10, 64 + 14));
    }

    #[test]
    fn wrong_sizes_are_refused() {
        assert!(frames(46, 22, &image(46, 22)).is_err());
        assert!(frames(64, 40, &image(64, 40)).is_err());
        assert!(frames(64, 32, &image(64, 31)).is_err());
        assert!(frames(64, 32 * 65, &image(64, 32 * 65)).is_err());
    }
}
