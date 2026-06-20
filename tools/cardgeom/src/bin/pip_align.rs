//! pip-align — prove title-top vs pip-top alignment by COLOUR (not edge density,
//! which catches frame lines). Title glyphs are warm off-white (#f9f7ef); the
//! generic "2" pip sits on a warm grey disk and coloured pips are saturated.
//! Prints both top fractions + delta, and draws guide lines for visual proof.
//!   pip-align RENDER.png OUT.png
use anyhow::Result;
use cardgeom::{Face, RENDER_BLEED_X, RENDER_BLEED_Y};
use image::Rgb;

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let (img, out) = (&a[1], &a[2]);
    let face = Face::load(img, RENDER_BLEED_X, RENDER_BLEED_Y)?;
    let (w, h) = (face.w, face.h);
    let px = |fx: f64| (fx * f64::from(w)) as u32;
    let py = |fy: f64| (fy * f64::from(h)) as u32;

    // warm off-white title ink: bright, and red >= blue (warm), not the bluish frame.
    let is_title = |p: &Rgb<u8>| {
        let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
        r > 205 && g > 195 && r - b >= 8 && (r - g).abs() < 22
    };
    // pip ink: the generic "2" sits on a warm GREY disk (r ≳ b) — unambiguous vs the
    // bluish frame (b > r). That disk's top == the pip-cluster top we align to.
    let is_pip = |p: &Rgb<u8>| {
        let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
        (150..228).contains(&r) && (143..220).contains(&g) && (135..212).contains(&b)
            && (r - b) >= 4 && (r - b) <= 40 && (r - g).abs() < 26
    };

    let top_in = |x0: f64, x1: f64, pred: &dyn Fn(&Rgb<u8>) -> bool| -> Option<u32> {
        let (xa, xb) = (px(x0), px(x1));
        for y in py(0.026)..py(0.12) {
            let mut c = 0u32;
            for x in xa..xb {
                if pred(face.img.get_pixel(x, y)) {
                    c += 1;
                }
            }
            if c >= 6 {
                return Some(y);
            }
        }
        None
    };

    let title_top = top_in(0.11, 0.55, &is_title);
    let pip_top = top_in(0.78, 0.965, &is_pip);

    let f = |o: Option<u32>| o.map(|y| f64::from(y) / f64::from(h));
    let (tt, pt) = (f(title_top), f(pip_top));
    println!("title_top = {tt:?}");
    println!("pip_top   = {pt:?}");
    if let (Some(t), Some(p)) = (tt, pt) {
        println!("delta (pip - title) = {:+.4}  ({:+.1}px)", p - t, (p - t) * f64::from(h));
        println!("{}", if (p - t).abs() <= 0.0015 { "ALIGNED ✓" } else { "MISALIGNED ✗" });
    }

    let mut canvas = face.img.clone();
    let mut hline = |y: u32, col: Rgb<u8>| {
        for x in 0..w {
            canvas.put_pixel(x, y.min(h - 1), col);
        }
    };
    if let Some(y) = title_top {
        hline(y, Rgb([255, 0, 0]));
    }
    if let Some(y) = pip_top {
        hline(y, Rgb([0, 200, 0]));
    }
    canvas.save(out)?;
    Ok(())
}
