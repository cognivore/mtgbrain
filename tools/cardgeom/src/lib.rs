//! Pixel-by-pixel geometry analysis of old-frame ("Seventh"/7ED) MTG cards.
//!
//! Two coordinate spaces matter:
//!   * a **real Scryfall scan** is the 2.5×3.5 card *including* its thin printed
//!     black border — the whole image is the "face".
//!   * our **renderer output** is that same 2.5×3.5 face centered inside an
//!     800-DPI canvas with MPC bleed around it. To compare apples-to-apples we
//!     crop the bleed off first (see [`RENDER_BLEED_X`] / [`RENDER_BLEED_Y`]).
//!
//! Everything downstream is reported as a **fraction of the face** (0..1), so a
//! 745×1040 scan and a 2000×2800 face are directly comparable.

use anyhow::{Context, Result};
use image::{GenericImageView, Rgb, RgbImage};
use serde::Serialize;

/// Renderer bleed as a fraction of the full canvas: (2176-2000)/2 / 2176.
pub const RENDER_BLEED_X: f64 = 88.0 / 2176.0;
/// Renderer bleed as a fraction of the full canvas: (2960-2800)/2 / 2960.
pub const RENDER_BLEED_Y: f64 = 80.0 / 2960.0;

/// The card face: bleed already cropped off, so (0,0)..(w,h) == the 2.5×3.5 card.
pub struct Face {
    pub img: RgbImage,
    pub w: u32,
    pub h: u32,
}

#[inline]
fn lum(p: &Rgb<u8>) -> f64 {
    0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])
}

impl Face {
    /// Load an image and crop the given symmetric bleed fractions off each edge.
    /// Pass `0.0, 0.0` for a raw Scryfall scan; [`RENDER_BLEED_X`]/`_Y` for a render.
    pub fn load(path: &str, bleed_x: f64, bleed_y: f64) -> Result<Self> {
        let img = image::open(path).with_context(|| format!("open {path}"))?;
        let (w, h) = img.dimensions();
        let bx = (bleed_x * f64::from(w)).round() as u32;
        let by = (bleed_y * f64::from(h)).round() as u32;
        let fw = w - 2 * bx;
        let fh = h - 2 * by;
        let face = img.crop_imm(bx, by, fw, fh).to_rgb8();
        Ok(Self { img: face, w: fw, h: fh })
    }

    fn px(&self, fx: f64) -> u32 {
        ((fx * f64::from(self.w)).round() as u32).min(self.w - 1)
    }
    fn py(&self, fy: f64) -> u32 {
        ((fy * f64::from(self.h)).round() as u32).min(self.h - 1)
    }
}

/// A rectangular search window, in face fractions.
#[derive(Clone, Copy)]
pub struct Band {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

/// A detected element's vertical+horizontal extent, all as face fractions.
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Extent {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    /// bottom - top
    pub height: f64,
    /// right - left
    pub width: f64,
    /// (top + bottom) / 2
    pub mid_y: f64,
}

/// Find the rows inside `b` that carry horizontal-edge "ink" (text / pip outlines),
/// then the columns, and report the tight bounding extent. `edge_t` is the
/// luminance step that counts as an edge; `frac` is how strong a row/col must be
/// (relative to the band's peak) to count as occupied — this rejects the smooth
/// beige nameplate while keeping glyph strokes.
pub fn ink_extent(face: &Face, b: &Band, edge_t: f64, frac: f64) -> Option<Extent> {
    let x0 = face.px(b.x0);
    let x1 = face.px(b.x1);
    let y0 = face.py(b.y0);
    let y1 = face.py(b.y1);
    if x1 <= x0 + 1 || y1 <= y0 + 1 {
        return None;
    }

    // Horizontal-edge count per row.
    let mut row: Vec<u32> = Vec::with_capacity((y1 - y0) as usize);
    for y in y0..y1 {
        let mut c = 0u32;
        for x in (x0 + 1)..x1 {
            if (lum(face.img.get_pixel(x - 1, y)) - lum(face.img.get_pixel(x, y))).abs() > edge_t {
                c += 1;
            }
        }
        row.push(c);
    }
    // Vertical-edge count per column.
    let mut col: Vec<u32> = Vec::with_capacity((x1 - x0) as usize);
    for x in x0..x1 {
        let mut c = 0u32;
        for y in (y0 + 1)..y1 {
            if (lum(face.img.get_pixel(x, y - 1)) - lum(face.img.get_pixel(x, y))).abs() > edge_t {
                c += 1;
            }
        }
        col.push(c);
    }

    let row_thr = (*row.iter().max()? as f64 * frac).max(2.0);
    let col_thr = (*col.iter().max()? as f64 * frac).max(2.0);

    let top_i = row.iter().position(|&c| c as f64 >= row_thr)?;
    let bot_i = row.iter().rposition(|&c| c as f64 >= row_thr)?;
    let left_i = col.iter().position(|&c| c as f64 >= col_thr)?;
    let right_i = col.iter().rposition(|&c| c as f64 >= col_thr)?;

    let top = f64::from(y0 + top_i as u32) / f64::from(face.h);
    let bottom = f64::from(y0 + bot_i as u32) / f64::from(face.h);
    let left = f64::from(x0 + left_i as u32) / f64::from(face.w);
    let right = f64::from(x0 + right_i as u32) / f64::from(face.w);
    Some(Extent {
        top,
        bottom,
        left,
        right,
        height: bottom - top,
        width: right - left,
        mid_y: (top + bottom) / 2.0,
    })
}

/// Fraction of pixels inside `b` darker than `dark_t` (0..255). A proxy for how
/// much black drop-shadow / outline ink the element carries — the "too shadowed"
/// metric.
pub fn dark_load(face: &Face, b: &Band, dark_t: f64) -> f64 {
    let x0 = face.px(b.x0);
    let x1 = face.px(b.x1);
    let y0 = face.py(b.y0);
    let y1 = face.py(b.y1);
    let mut dark = 0u64;
    let mut total = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            if lum(face.img.get_pixel(x, y)) < dark_t {
                dark += 1;
            }
            total += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        dark as f64 / total as f64
    }
}

/// Default search bands for the old frame's top row (title left, mana right).
/// `y1` stops just ABOVE the art window (which starts at y≈0.099) — otherwise the
/// dark art clouds bleed in and corrupt both the extent and the shadow metric.
pub const TITLE_BAND: Band = Band { x0: 0.085, x1: 0.62, y0: 0.030, y1: 0.097 };
pub const MANA_BAND: Band = Band { x0: 0.70, x1: 0.985, y0: 0.030, y1: 0.097 };

/// Full measurement of one card face.
#[derive(Serialize)]
pub struct Measurement {
    pub face_w: u32,
    pub face_h: u32,
    pub title: Option<Extent>,
    pub mana: Option<Extent>,
    /// dark-pixel fraction within the mana band (shadow load).
    pub mana_dark_load: f64,
    /// dark-pixel fraction within the title band (shadow load).
    pub title_dark_load: f64,
}

pub fn measure(face: &Face) -> Measurement {
    Measurement {
        face_w: face.w,
        face_h: face.h,
        title: ink_extent(face, &TITLE_BAND, 38.0, 0.18),
        mana: ink_extent(face, &MANA_BAND, 38.0, 0.18),
        mana_dark_load: dark_load(face, &MANA_BAND, 70.0),
        title_dark_load: dark_load(face, &TITLE_BAND, 70.0),
    }
}

/// Draw the detected extents as 1px boxes onto a copy of the face (debug aid).
pub fn annotate(face: &Face, m: &Measurement) -> RgbImage {
    let mut out = face.img.clone();
    let red = Rgb([255, 0, 0]);
    let green = Rgb([0, 220, 0]);
    if let Some(e) = m.title {
        draw_box(&mut out, e, face, red);
    }
    if let Some(e) = m.mana {
        draw_box(&mut out, e, face, green);
    }
    out
}

fn draw_box(img: &mut RgbImage, e: Extent, face: &Face, c: Rgb<u8>) {
    let x0 = (e.left * f64::from(face.w)) as u32;
    let x1 = (e.right * f64::from(face.w)) as u32;
    let y0 = (e.top * f64::from(face.h)) as u32;
    let y1 = (e.bottom * f64::from(face.h)) as u32;
    for x in x0..=x1.min(face.w - 1) {
        img.put_pixel(x, y0.min(face.h - 1), c);
        img.put_pixel(x, y1.min(face.h - 1), c);
    }
    for y in y0..=y1.min(face.h - 1) {
        img.put_pixel(x0.min(face.w - 1), y, c);
        img.put_pixel(x1.min(face.w - 1), y, c);
    }
}
