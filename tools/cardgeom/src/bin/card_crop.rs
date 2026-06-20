//! card-crop — magnify a face-fraction region for eyeballing. Bleed-aware.
//!   card-crop IMG OUT x0 y0 x1 y1 [--render]
use anyhow::Result;
use cardgeom::{Face, RENDER_BLEED_X, RENDER_BLEED_Y};
use image::imageops::{resize, FilterType};

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let render = a.iter().any(|s| s == "--render");
    let p: Vec<&String> = a[1..].iter().filter(|s| !s.starts_with("--")).collect();
    let (img, out) = (p[0], p[1]);
    let (x0, y0, x1, y1): (f64, f64, f64, f64) =
        (p[2].parse()?, p[3].parse()?, p[4].parse()?, p[5].parse()?);
    let (bx, by) = if render { (RENDER_BLEED_X, RENDER_BLEED_Y) } else { (0.0, 0.0) };
    let face = Face::load(img, bx, by)?;
    let (fw, fh) = (f64::from(face.w), f64::from(face.h));
    let sub = image::imageops::crop_imm(
        &face.img,
        (x0 * fw) as u32,
        (y0 * fh) as u32,
        ((x1 - x0) * fw) as u32,
        ((y1 - y0) * fh) as u32,
    )
    .to_image();
    let big = resize(&sub, sub.width() * 3, sub.height() * 3, FilterType::Nearest);
    big.save(out)?;
    println!("{out}");
    Ok(())
}
