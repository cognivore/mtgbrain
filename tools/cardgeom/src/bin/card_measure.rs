//! card-measure — measure one card image and emit its element geometry as JSON.
//!
//! Examples:
//!   card-measure --render render-cache/cards/Serra_Angel/<hash>.png
//!   card-measure --scan   render-cache/cards/Serra_Angel/scryfall/7ed.png --annotate /tmp/serra.png

use anyhow::Result;
use cardgeom::{annotate, measure, Face, RENDER_BLEED_X, RENDER_BLEED_Y};
use clap::Parser;

#[derive(Parser)]
#[command(about = "Pixel geometry of a single old-frame card (fractions of the face)")]
struct Cli {
    /// Path to the image.
    image: String,
    /// Image is a renderer output (crop MPC bleed before measuring).
    #[arg(long, conflicts_with = "scan")]
    render: bool,
    /// Image is a raw Scryfall scan (whole image is the face). Default.
    #[arg(long)]
    scan: bool,
    /// Optionally write an annotated PNG with the detected boxes drawn.
    #[arg(long)]
    annotate: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let (bx, by) = if cli.render {
        (RENDER_BLEED_X, RENDER_BLEED_Y)
    } else {
        (0.0, 0.0)
    };
    let face = Face::load(&cli.image, bx, by)?;
    let m = measure(&face);
    if let Some(path) = &cli.annotate {
        annotate(&face, &m).save(path)?;
        eprintln!("annotated → {path}");
    }
    println!("{}", serde_json::to_string_pretty(&m)?);
    Ok(())
}
