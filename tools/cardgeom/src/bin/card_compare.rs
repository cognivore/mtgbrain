//! card-compare — measure a real scan (reference) and our render, then report
//! the geometry deltas and a plain-language verdict.
//!
//!   card-compare --ref <scryfall.png> --render <render.png>
//!
//! All deltas are in face fractions (so a 745×1040 scan and a 2000×2800 face are
//! comparable). Positive `title mid_y delta` ⇒ our title sits LOWER than the real
//! card ("text too low"). Ratios > 1 ⇒ our element is bigger / darker.

use anyhow::{bail, Result};
use cardgeom::{measure, Extent, Face, Measurement, RENDER_BLEED_X, RENDER_BLEED_Y};
use clap::Parser;

#[derive(Parser)]
#[command(about = "Compare our render geometry against a real card scan")]
struct Cli {
    /// Reference image (raw Scryfall scan; whole image is the face).
    #[arg(long = "ref")]
    reference: String,
    /// Our renderer output (MPC bleed cropped automatically).
    #[arg(long)]
    render: String,
}

fn fpct(f: f64) -> String {
    format!("{:+.2}%", f * 100.0)
}

fn line(label: &str, refv: f64, renv: f64) {
    let d = renv - refv;
    let ratio = if refv.abs() > 1e-9 { renv / refv } else { f64::NAN };
    println!(
        "  {label:<22} ref={:>7.4}  render={:>7.4}  Δ={:>8}  ×{ratio:.3}",
        refv,
        renv,
        fpct(d)
    );
}

fn need(e: Option<Extent>, what: &str) -> Result<Extent> {
    e.ok_or_else(|| anyhow::anyhow!("could not detect {what}"))
}

fn report(refm: &Measurement, renm: &Measurement) -> Result<()> {
    let rt = need(refm.title, "title in reference")?;
    let nt = need(renm.title, "title in render")?;
    let rm = need(refm.mana, "mana in reference")?;
    let nm = need(renm.mana, "mana in render")?;

    println!("== TITLE ==");
    line("top y", rt.top, nt.top);
    line("mid y", rt.mid_y, nt.mid_y);
    line("bottom y", rt.bottom, nt.bottom);
    line("cap height", rt.height, nt.height);
    line("dark load", refm.title_dark_load, renm.title_dark_load);

    println!("== MANA PIPS ==");
    line("top y", rm.top, nm.top);
    line("mid y", rm.mid_y, nm.mid_y);
    line("bottom y", rm.bottom, nm.bottom);
    line("pip height (diam)", rm.height, nm.height);
    line("cluster right edge", rm.right, nm.right);
    line("dark load (shadow)", refm.mana_dark_load, renm.mana_dark_load);

    println!("\n== VERDICT ==");
    let mut ok = true;

    // Title vertical position: tolerate ±0.6% of face height.
    let dmid = nt.mid_y - rt.mid_y;
    if dmid > 0.006 {
        ok = false;
        println!(
            "  ✗ title too LOW by {} of face height — raise it (decrease TY by ~{})",
            fpct(dmid),
            fpct(dmid)
        );
    } else if dmid < -0.006 {
        ok = false;
        println!("  ✗ title too HIGH by {} — lower it", fpct(-dmid));
    } else {
        println!("  ✓ title vertical position within tolerance ({})", fpct(dmid));
    }

    // Pip size: tolerate ±8%.
    let pr = nm.height / rm.height;
    if pr > 1.08 {
        ok = false;
        println!(
            "  ✗ pips too BIG ×{pr:.3} — shrink ms-cost / MSZ by ~{:.0}%",
            (pr - 1.0) * 100.0
        );
    } else if pr < 0.92 {
        ok = false;
        println!("  ✗ pips too SMALL ×{pr:.3} — enlarge", );
    } else {
        println!("  ✓ pip size within tolerance (×{pr:.3})");
    }

    // Shadow load: tolerate ±25% (shadow is a small effect, noisier metric).
    let sr = if refm.mana_dark_load > 1e-6 {
        renm.mana_dark_load / refm.mana_dark_load
    } else {
        f64::NAN
    };
    if sr.is_finite() && sr > 1.25 {
        ok = false;
        println!(
            "  ✗ pips too SHADOWED ×{sr:.3} — lighten box-shadow (less offset/opacity)"
        );
    } else if sr.is_finite() && sr < 0.75 {
        println!("  ⚠ pips lighter than reference ×{sr:.3}");
    } else {
        println!("  ✓ pip shadow within tolerance (×{sr:.3})");
    }

    println!("\n{}", if ok { "PASS" } else { "FAIL — adjust geometry above" });
    if !ok {
        bail!("geometry mismatch");
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let refface = Face::load(&cli.reference, 0.0, 0.0)?;
    let renface = Face::load(&cli.render, RENDER_BLEED_X, RENDER_BLEED_Y)?;
    let refm = measure(&refface);
    let renm = measure(&renface);
    report(&refm, &renm)
}
