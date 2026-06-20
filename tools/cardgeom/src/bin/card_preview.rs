//! card-preview — fill the LIVE `src/card_template.html` for one card and write
//! an HTML file, so the template/geometry can be iterated and screenshotted with
//! headless Chrome WITHOUT rebuilding the main mtgbrain crate.
//!
//! It mirrors the substitution table in `src/render.rs::build_html`. The handful
//! of values under active tuning are overridable on the CLI (`--ty`, `--tsz`,
//! `--msz`) so the right number can be found here, then written into render.rs.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

const W: u32 = 2176;
const H: u32 = 2960;
const FACE_W: u32 = 2000;
const FACE_H: u32 = 2800;

fn pc(f: f64) -> String {
    format!("{:.4}", f * 100.0)
}
fn px(f: f64) -> String {
    format!("{}", (f * f64::from(FACE_H)) as u32)
}

#[derive(Parser)]
struct Cli {
    /// Path to the live card_template.html.
    #[arg(long, default_value = "../../src/card_template.html")]
    template: PathBuf,
    /// Fonts directory (file:// is prepended).
    #[arg(long, default_value = "../../assets/fonts")]
    fonts: PathBuf,
    #[arg(long)]
    name: String,
    #[arg(long, default_value = "")]
    mana: String,
    #[arg(long, default_value = "")]
    type_line: String,
    #[arg(long, default_value = "")]
    pt: String,
    /// Rules text (\n for line breaks).
    #[arg(long, default_value = "")]
    rules: String,
    #[arg(long)]
    frame: PathBuf,
    #[arg(long)]
    art: PathBuf,
    #[arg(long, default_value = "Doug Shuler")]
    artist: String,
    #[arg(long, default_value = "1997")]
    year: String,
    #[arg(long)]
    out: PathBuf,
    // ---- geometry knobs under tuning (defaults match render.rs) ----
    #[arg(long, default_value_t = 0.0481)]
    ty: f64,
    #[arg(long, default_value_t = 0.041)]
    tsz: f64,
    #[arg(long, default_value_t = 72.0 / 1638.0)]
    msz: f64,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn mana_icon(sym: &str, base: &str) -> String {
    let key = match sym.to_uppercase().as_str() {
        "T" => "t".to_string(),
        "Q" => "untap".to_string(),
        other => other.replace('/', "").to_lowercase(),
    };
    format!(r#"<img class="ms-img" src="{base}/{key}.svg">"#)
}

fn manaify(text: &str, base: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut sym = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                sym.push(c2);
            }
            out.push_str(&mana_icon(&sym, base));
        } else {
            out.push_str(&esc(&ch.to_string()));
        }
    }
    out
}

fn reminder_italic(html: &str) -> String {
    let mut out = String::new();
    let mut depth: u32 = 0;
    for ch in html.chars() {
        match ch {
            '(' => {
                if depth == 0 {
                    out.push_str("<i>(");
                } else {
                    out.push('(');
                }
                depth += 1;
            }
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    out.push_str(")</i>");
                } else {
                    out.push(')');
                }
            }
            _ => out.push(ch),
        }
    }
    if depth > 0 {
        out.push_str("</i>");
    }
    out
}

fn rules_html(text: &str, base: &str) -> String {
    let mut s = String::new();
    for line in text.split("\\n").filter(|l| !l.trim().is_empty()) {
        s.push_str("<p>");
        s.push_str(&reminder_italic(&manaify(line, base)));
        s.push_str("</p>");
    }
    s
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // file:// URLs must be absolute or the @font-face faces silently fall back.
    let fonts_abs = cli.fonts.canonicalize().unwrap_or_else(|_| cli.fonts.clone());
    let f = |p: &str| format!("file://{}", fonts_abs.join(p).display());
    let mana_base = format!(
        "file://{}",
        fonts_abs.parent().unwrap_or(&fonts_abs).join("mana").display()
    );
    let italic_face = if cli.fonts.join("mplantin-italic.ttf").exists() {
        format!(
            "@font-face {{ font-family:'mplantin'; font-style:italic; src:url('{}'); }}",
            f("mplantin-italic.ttf")
        )
    } else {
        String::new()
    };
    let pt = if cli.pt.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="box pt"><span>{}</span></div>"#, esc(&cli.pt))
    };
    let illus = format!(r#"<div class="info illus"><span>Illus. {}</span></div>"#, esc(&cli.artist));

    let pairs: Vec<(&str, String)> = vec![
        ("MANA_CSS", f("mana.css")),
        ("GOUDY", f("goudy-medieval.ttf")),
        ("MPLANTIN", f("mplantin.ttf")),
        ("ITALIC_FACE", italic_face),
        ("W", W.to_string()),
        ("H", H.to_string()),
        ("FW", FACE_W.to_string()),
        ("FH", FACE_H.to_string()),
        ("BX", ((W - FACE_W) / 2).to_string()),
        ("BY", ((H - FACE_H) / 2).to_string()),
        ("AX", pc(0.12)),
        ("AY", pc(0.0991)),
        ("AW", pc(0.7667)),
        ("AH", pc(0.4429)),
        ("TX", pc(0.1134)),
        ("TY", pc(cli.ty)),
        ("TW", pc(0.7734)),
        ("TH", pc(0.041)),
        ("MAX", pc(0.1067)),
        ("MAY", pc(0.0481)),
        ("MAW", pc(0.8174)),
        ("MAH", px(0.041)),
        ("TSZ", px(cli.tsz)),
        ("MSZ", px(cli.msz)),
        ("SHX", px(0.002 * f64::from(FACE_W) / f64::from(FACE_H))),
        ("SHY", px(0.0015)),
        ("TYX", pc(0.1074)),
        ("TYY", pc(0.5486)),
        ("TYW", pc(0.7852)),
        ("TYH", pc(0.0543)),
        ("TYSZ", px(0.032)),
        ("RX", pc(0.128)),
        ("RY", pc(0.6067)),
        ("RW", pc(0.744)),
        ("RH", pc(0.2724)),
        ("RSZ", px(0.0358)),
        ("PX", pc(0.8074)),
        ("PY", pc(0.9043)),
        ("PW", pc(0.1367)),
        ("PSZ", px(0.0429)),
        ("IY", pc(1908.0 / 2100.0)),
        ("ISZ", px(0.0172)),
        ("LY", pc(1940.0 / 2100.0)),
        ("LSZ", px(0.0143)),
    ];
    let content: Vec<(&str, String)> = vec![
        ("ART", format!("file://{}", cli.art.canonicalize().unwrap_or(cli.art.clone()).display())),
        ("FRAME", format!("file://{}", cli.frame.canonicalize().unwrap_or(cli.frame.clone()).display())),
        ("NAME", esc(&cli.name)),
        ("MANA", manaify(&cli.mana, &mana_base)),
        ("TYPE", esc(&cli.type_line)),
        ("RULES", rules_html(&cli.rules, &mana_base)),
        ("PT", pt),
        ("ILLUS", illus),
        ("YEAR", cli.year.clone()),
        ("FOIL", String::new()),
    ];

    let mut html = std::fs::read_to_string(&cli.template)?;
    for (k, v) in pairs.iter().chain(content.iter()) {
        html = html.replace(&format!("%%{k}%%"), v);
    }
    std::fs::write(&cli.out, html)?;
    println!("{}", cli.out.display());
    Ok(())
}
