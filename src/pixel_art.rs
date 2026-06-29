//! Testable pixel art data used by the Bevy renderer.

pub type Rgba = [u8; 4];

pub const TRANSPARENT: Rgba = [0, 0, 0, 0];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PixelArt {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba>,
}

impl PixelArt {
    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgba> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.pixels.get((y * self.width + x) as usize).copied()
    }

    pub fn into_rgba_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in self.pixels {
            data.extend_from_slice(&pixel);
        }
        data
    }
}

pub fn car() -> PixelArt {
    generate(12, 18, |x, y| {
        const TIRE: Rgba = [22, 24, 28, 255];
        const BODY: Rgba = [226, 45, 48, 255];
        const SHADOW: Rgba = [126, 28, 34, 255];
        const GLASS: Rgba = [76, 172, 202, 255];
        const LIGHT: Rgba = [255, 239, 128, 255];

        if ((x <= 1 || x >= 10) && (4..=14).contains(&y))
            || ((x == 2 || x == 9) && (15..=16).contains(&y))
        {
            TIRE
        } else if (4..=7).contains(&x) && y <= 1 {
            LIGHT
        } else if (3..=8).contains(&x) && (4..=7).contains(&y) {
            GLASS
        } else if (4..=7).contains(&x) && (13..=16).contains(&y) {
            SHADOW
        } else if (2..=9).contains(&x) && (1..=16).contains(&y) {
            BODY
        } else {
            TRANSPARENT
        }
    })
}

pub fn start_line() -> PixelArt {
    generate(16, 3, |x, y| {
        if (x + y) % 2 == 0 {
            [245, 245, 232, 255]
        } else {
            [18, 20, 24, 255]
        }
    })
}

pub fn tree() -> PixelArt {
    generate(8, 8, |x, y| {
        const LEAF_DARK: Rgba = [24, 96, 44, 255];
        const LEAF_LIGHT: Rgba = [62, 168, 70, 255];
        const TRUNK: Rgba = [104, 72, 42, 255];

        if (3..=4).contains(&x) && y >= 5 {
            TRUNK
        } else if (1..=6).contains(&x) && (1..=5).contains(&y) {
            if (x + y) % 2 == 0 {
                LEAF_LIGHT
            } else {
                LEAF_DARK
            }
        } else if (2..=5).contains(&x) && y == 0 {
            LEAF_LIGHT
        } else {
            TRANSPARENT
        }
    })
}

pub fn barrier() -> PixelArt {
    generate(8, 4, |x, y| {
        if (x / 2 + y) % 2 == 0 {
            [232, 48, 50, 255]
        } else {
            [245, 245, 232, 255]
        }
    })
}

fn generate(width: u32, height: u32, mut color_at: impl FnMut(u32, u32) -> Rgba) -> PixelArt {
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.push(color_at(x, y));
        }
    }

    PixelArt {
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_art_marks_shape_and_transparent_corners() {
        let art = car();

        assert_eq!((art.width, art.height), (12, 18));
        assert_eq!(art.pixel(0, 0), Some(TRANSPARENT));
        assert_eq!(art.pixel(0, 4), Some([22, 24, 28, 255]));
        assert_eq!(art.pixel(5, 1), Some([255, 239, 128, 255]));
        assert_eq!(art.pixel(4, 4), Some([76, 172, 202, 255]));
        assert_eq!(art.pixel(4, 14), Some([126, 28, 34, 255]));
    }

    #[test]
    fn start_line_uses_checkerboard_pixels() {
        let art = start_line();

        assert_eq!((art.width, art.height), (16, 3));
        assert_eq!(art.pixel(0, 0), Some([245, 245, 232, 255]));
        assert_eq!(art.pixel(1, 0), Some([18, 20, 24, 255]));
        assert_eq!(art.pixel(0, 1), Some([18, 20, 24, 255]));
    }

    #[test]
    fn tree_art_contains_leaf_and_trunk_regions() {
        let art = tree();

        assert_eq!((art.width, art.height), (8, 8));
        assert_eq!(art.pixel(0, 0), Some(TRANSPARENT));
        assert_eq!(art.pixel(2, 0), Some([62, 168, 70, 255]));
        assert_eq!(art.pixel(1, 2), Some([24, 96, 44, 255]));
        assert_eq!(art.pixel(3, 6), Some([104, 72, 42, 255]));
    }

    #[test]
    fn barrier_art_stripes_alternate_every_two_columns() {
        let art = barrier();

        assert_eq!((art.width, art.height), (8, 4));
        assert_eq!(art.pixel(0, 0), Some([232, 48, 50, 255]));
        assert_eq!(art.pixel(2, 0), Some([245, 245, 232, 255]));
        assert_eq!(art.pixel(0, 1), Some([245, 245, 232, 255]));
    }

    #[test]
    fn rgba_bytes_are_row_major() {
        let bytes = start_line().into_rgba_bytes();

        assert_eq!(&bytes[0..4], &[245, 245, 232, 255]);
        assert_eq!(&bytes[4..8], &[18, 20, 24, 255]);
        assert_eq!(bytes.len(), 16 * 3 * 4);
    }
}
