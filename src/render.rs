//! Old-frame card renderer → print-ready MPC PNGs.
//!
//! Frame + typography are reverse-engineered from **cardconjurer**'s Seventh
//! (2001 / 7th-Edition) pack (`js/frames/packSeventh.js`): its exact frame art,
//! art/title/type/rules/PT bounds, fonts, and bottom-info (Illus. / Wizards /
//! NOT FOR SALE). Text is typeset in HTML with an in-page **auto-fit** pass
//! (shrink-to-fit like cardconjurer), reminder text italicised, mana drawn with
//! the OFL Mana font, then screenshotted by headless Google Chrome at the MPC
//! 800-DPI-with-bleed target (2176×2960). Card content comes from the editor DB
//! with per-field overrides applied (read-only — never written).
//!
//! Art = the **oldest Scryfall printing** (authentic pre-modern illustration) +
//! its artist credit. Foil variant = the **single white 7ED falling star** at
//! its real position, nothing else (you print on foil paper — no sheen/texture).
//!
//! Cache is content-addressed: render-relevant fields → canonical JSON (sorted
//! keys via serde_json's BTreeMap Value, sorted arrays) → SHA-256 →
//! `cards/<Name>/<hash>.png` (foil: `.../foil/<hash>.png`).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// cardconjurer raw asset root.
const CC: &str = "https://raw.githubusercontent.com/Investigamer/cardconjurer/master";
/// Andrew Gioia's open (OFL) mana-symbol font.
const MANA: &str = "https://raw.githubusercontent.com/andrewgioia/mana/master";

/// Default Chrome on macOS; override with --chrome / MTGBRAIN_CHROME.
pub const CHROME_DEFAULT: &str = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

/// MPC 800-DPI target with bleed (2.72in × 3.70in). Face (2.5×3.5) is centered.
const W: u32 = 2176;
const H: u32 = 2960;
const FACE_W: u32 = 2000;
const FACE_H: u32 = 2800;

// ---------------------------------------------------------------------------
// assets
// ---------------------------------------------------------------------------

fn asset_list() -> Vec<(String, &'static str)> {
    let mut v: Vec<(String, &'static str)> = Vec::new();
    // Seventh (2001) frames: one self-contained PNG per colour/type.
    for letter in ["w", "u", "b", "r", "g", "m", "a", "c", "l", "wl", "ul", "bl", "rl", "gl"] {
        v.push((
            format!("{CC}/img/frames/seventh/regular/{letter}.png"),
            Box::leak(format!("frames/{letter}.png").into_boxed_str()),
        ));
    }
    // 7ED foil "falling star" (white path, correctly positioned).
    v.push((format!("{CC}/img/frames/seventh/foilStar.svg"), "foil/star.svg"));
    // old fonts (proprietary — kept out of git; personal-use only)
    v.push((format!("{CC}/fonts/goudy-medieval.ttf"), "fonts/goudy-medieval.ttf"));
    v.push((format!("{CC}/fonts/mplantin.ttf"), "fonts/mplantin.ttf"));
    v.push((format!("{CC}/fonts/mplantin-italic.ttf"), "fonts/mplantin-italic.ttf"));
    v.push((format!("{CC}/fonts/matrix.ttf"), "fonts/matrix.ttf"));
    // mana font (OFL)
    v.push((format!("{MANA}/fonts/mana.ttf"), "fonts/mana.ttf"));
    v.push((format!("{MANA}/css/mana.css"), "fonts/mana.css"));
    v
}

/// Download all render assets into `assets_dir` (idempotent unless `force`).
pub fn assets(assets_dir: &Path, force: bool) -> Result<()> {
    let (mut got, mut skipped) = (0u32, 0u32);
    for (url, rel) in asset_list() {
        let dst = assets_dir.join(rel);
        if dst.exists() && !force {
            skipped += 1;
            continue;
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        // mplantin-italic may not exist upstream; tolerate a miss.
        if curl_to_file(&url, &dst).is_err() {
            eprintln!("  (skip, not found) {rel}");
            continue;
        }
        got += 1;
        println!("  ↓ {rel}");
    }
    let css_path = assets_dir.join("fonts/mana.css");
    if css_path.exists() {
        let css = fs::read_to_string(&css_path)?;
        let fixed = css
            .replace("../fonts/mana.woff2", "mana.ttf")
            .replace("../fonts/mana.woff", "mana.ttf")
            .replace("../fonts/mana.ttf", "mana.ttf");
        fs::write(&css_path, fixed)?;
    }
    println!("assets: {got} downloaded, {skipped} present  ->  {}", assets_dir.display());
    if got > 0 {
        println!("note: frame art + old fonts are WotC-derived/proprietary — personal use only, keep out of git.");
    }
    Ok(())
}

fn curl_to_file(url: &str, dst: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-sSL", "--fail", "--max-time", "60", "-o"])
        .arg(dst)
        .arg(url)
        .status()
        .context("running curl")?;
    if !status.success() {
        bail!("curl failed for {url}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// card model (from editor DB, overrides applied)
// ---------------------------------------------------------------------------

struct Card {
    name: String,
    mana_cost: String,
    type_line: String,
    oracle_text: String,
    power: String,
    toughness: String,
    loyalty: String,
    colors: String,
    is_creature: bool,
}

type RawRow = (
    String, Option<String>, Option<String>, Option<String>, Option<String>,
    Option<String>, Option<String>, Option<String>, i64, String,
);

fn load_card(db: &Connection, id: i64) -> Result<Card> {
    let (name, mc, tl, ot, p, t, l, colors, is_cr, ov): RawRow = db
        .query_row(
            "SELECT name,mana_cost,type,oracle_text,power,toughness,loyalty,colors,
                    is_creature,overrides FROM cube_cards WHERE id=?1",
            params![id],
            |r| Ok((
                r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
            )),
        )
        .with_context(|| format!("no card id {id} in editor DB"))?;
    let o: Value = serde_json::from_str(&ov).unwrap_or_else(|_| json!({}));
    let pick = |key: &str, base: Option<String>| -> String {
        o.get(key).and_then(Value::as_str).map(ToString::to_string).or(base).unwrap_or_default()
    };
    Ok(Card {
        name: pick("name", Some(name)),
        mana_cost: pick("mana_cost", mc),
        type_line: pick("type", tl),
        oracle_text: pick("oracle_text", ot),
        power: pick("power", p),
        toughness: pick("toughness", t),
        loyalty: pick("loyalty", l),
        colors: pick("colors", colors),
        is_creature: is_cr != 0,
    })
}

fn card_hash(c: &Card, art_ref: &str, artist: &str) -> String {
    let mut colors: Vec<String> = c
        .colors.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    colors.sort();
    let canon = json!({
        "name": c.name, "mana_cost": c.mana_cost, "type": c.type_line,
        "oracle_text": c.oracle_text, "power": c.power, "toughness": c.toughness,
        "loyalty": c.loyalty, "colors": colors, "is_creature": c.is_creature,
        "art_ref": art_ref, "artist": artist, "frame": "seventh", "v": 2,
    });
    let mut h = Sha256::new();
    h.update(canon.to_string().as_bytes());
    format!("{:x}", h.finalize())
}

fn frame_file(c: &Card) -> &'static str {
    let t = c.type_line.to_lowercase();
    let is_land = t.contains("land");
    let is_artifact = t.contains("artifact");
    let cols: Vec<char> = c.colors.chars().filter(|ch| "WUBRG".contains(*ch)).collect();
    if is_land {
        return match cols.first() {
            Some('W') => "frames/wl.png", Some('U') => "frames/ul.png",
            Some('B') => "frames/bl.png", Some('R') => "frames/rl.png",
            Some('G') => "frames/gl.png", _ => "frames/l.png",
        };
    }
    match cols.len() {
        0 => if is_artifact { "frames/a.png" } else { "frames/c.png" },
        1 => match cols[0] {
            'W' => "frames/w.png", 'U' => "frames/u.png", 'B' => "frames/b.png",
            'R' => "frames/r.png", _ => "frames/g.png",
        },
        _ => "frames/m.png", // Seventh has a real gold multicolor frame.
    }
}

// ---------------------------------------------------------------------------
// art acquisition (oldest Scryfall printing + artist; MPC-Autofill optional)
// ---------------------------------------------------------------------------

struct Art {
    path: PathBuf,
    art_ref: String,
    artist: String,
    year: String,
}

fn acquire_art(name: &str, art_dir: &Path, _backend: Option<&str>) -> Result<Art> {
    fs::create_dir_all(art_dir)?;
    // Scryfall: all paper prints oldest-first → take the oldest (authentic old art).
    let meta_url = format!(
        "https://api.scryfall.com/cards/search?order=released&dir=asc&unique=prints&q={}",
        percent(&format!("!\"{name}\" game:paper"))
    );
    let meta_file = art_dir.join(format!("{}.json", sanitize(name)));
    curl_to_file(&meta_url, &meta_file)?;
    let v: Value = serde_json::from_str(&fs::read_to_string(&meta_file)?)
        .with_context(|| format!("parsing Scryfall metadata for {name}"))?;
    let first = v["data"].as_array().and_then(|a| a.first())
        .with_context(|| format!("no Scryfall print found for {name}"))?;
    let artist = first["artist"].as_str().unwrap_or("").to_string();
    let year = first["released_at"].as_str().unwrap_or("").chars().take(4).collect::<String>();
    let set = first["set"].as_str().unwrap_or("?").to_string();
    let art_url = first["image_uris"]["art_crop"].as_str()
        .or_else(|| first["card_faces"][0]["image_uris"]["art_crop"].as_str())
        .with_context(|| format!("no art_crop for {name}"))?;
    let dst = art_dir.join(format!("{}.jpg", sanitize(name)));
    curl_to_file(art_url, &dst)?;
    Ok(Art { path: dst, art_ref: format!("scryfall:{set}:{year}:art_crop"), artist, year })
}

// ---------------------------------------------------------------------------
// HTML (Seventh bounds; in-page auto-fit; reminder italics; white-star foil)
// ---------------------------------------------------------------------------

fn pc(f: f64) -> String { format!("{:.4}", f * 100.0) }
fn px(f: f64) -> String { format!("{}", (f * f64::from(FACE_H)) as u32) }

const TEMPLATE: &str = include_str!("card_template.html");

fn build_html(c: &Card, frame_abs: &Path, art_abs: &Path, assets_dir: &Path, art: &Art, foil: bool) -> String {
    let fonts = assets_dir.join("fonts");
    let f = |p: &str| format!("file://{}", fonts.join(p).display());
    let italic_face = if fonts.join("mplantin-italic.ttf").exists() {
        format!("@font-face {{ font-family:'mplantin'; font-style:italic; src:url('{}'); }}", f("mplantin-italic.ttf"))
    } else {
        String::new()
    };
    let pt = if c.is_creature && (!c.power.is_empty() || !c.toughness.is_empty()) {
        format!(r#"<div class="box pt"><span>{}/{}</span></div>"#, esc(&c.power), esc(&c.toughness))
    } else if !c.loyalty.is_empty() {
        format!(r#"<div class="box pt"><span>{}</span></div>"#, esc(&c.loyalty))
    } else {
        String::new()
    };
    let illus = if art.artist.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="info illus"><span>Illus. {}</span></div>"#, esc(&art.artist))
    };
    let year = if art.year.is_empty() { "2001".to_string() } else { art.year.clone() };
    let foil_layer = if foil {
        format!(r#"<img class="foilstar" src="file://{}">"#, assets_dir.join("foil/star.svg").display())
    } else {
        String::new()
    };

    let pairs: [(&str, String); 35] = [
        ("MANA_CSS", f("mana.css")),
        ("GOUDY", f("goudy-medieval.ttf")),
        ("MPLANTIN", f("mplantin.ttf")),
        ("ITALIC_FACE", italic_face),
        ("W", W.to_string()), ("H", H.to_string()),
        ("FW", FACE_W.to_string()), ("FH", FACE_H.to_string()),
        ("BX", ((W - FACE_W) / 2).to_string()), ("BY", ((H - FACE_H) / 2).to_string()),
        ("AX", pc(0.12)), ("AY", pc(0.0991)), ("AW", pc(0.7667)), ("AH", pc(0.4429)),
        ("TX", pc(0.1067)), ("TY", pc(0.0481)), ("TW", pc(0.824)), ("TH", pc(0.05)),
        ("TSZ", px(0.041)), ("MSZ", px(72.0 / 2100.0)),
        ("TYX", pc(0.1074)), ("TYY", pc(0.5486)), ("TYW", pc(0.7852)), ("TYSZ", px(0.032)),
        ("RX", pc(0.128)), ("RY", pc(0.6067)), ("RW", pc(0.744)), ("RH", pc(0.2724)), ("RSZ", px(0.0358)),
        ("PX", pc(0.8074)), ("PY", pc(0.9043)), ("PSZ", px(0.0429)),
        ("IY", pc(0.903)), ("WY", pc(0.927)), ("NY", pc(0.948)),
    ];
    let content: [(&str, String); 9] = [
        ("ART", format!("file://{}", art_abs.display())),
        ("FRAME", format!("file://{}", frame_abs.display())),
        ("NAME", esc(&c.name)),
        ("MANA", manaify(&c.mana_cost)),
        ("TYPE", esc(&c.type_line)),
        ("RULES", rules_html(&c.oracle_text)),
        ("PT", pt),
        ("ILLUS", illus),
        ("YEAR", year),
    ];
    let foil_pair = [("FOIL", foil_layer)];

    let mut html = TEMPLATE.to_string();
    for (k, v) in pairs.iter().chain(content.iter()).chain(foil_pair.iter()) {
        html = html.replace(&format!("%%{k}%%"), v);
    }
    html
}

/// Rules text → paragraphs, mana symbols inline, parenthetical reminder italic.
fn rules_html(text: &str) -> String {
    let mut s = String::new();
    for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
        s.push_str("<p>");
        s.push_str(&reminder_italic(&manaify(line)));
        s.push_str("</p>");
    }
    s
}

/// Wrap parenthetical reminder text in <i>…</i> (operates on already-escaped HTML).
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

fn manaify(text: &str) -> String {
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
            out.push_str(&mana_icon(&sym));
        } else {
            out.push_str(&esc_char(ch));
        }
    }
    out
}

fn mana_icon(sym: &str) -> String {
    let key = match sym.to_uppercase().as_str() {
        "T" => "tap".to_string(),
        "Q" => "untap".to_string(),
        other => other.replace('/', "").to_lowercase(),
    };
    format!(r#"<i class="ms ms-{key} ms-cost"></i>"#)
}

// ---------------------------------------------------------------------------
// render
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn render_one(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str,
    id: i64, foil: bool, force: bool, backend: Option<&str>,
) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening editor DB {} (read-only)", editor_db.display()))?;
    let card = load_card(&db, id)?;

    let frame_rel = assets_dir.join(frame_file(&card));
    if !frame_rel.exists() {
        bail!("missing frame asset {} — run `mtgbrain render assets`", frame_rel.display());
    }
    let art = acquire_art(&card.name, &cache_dir.join("art"), backend)?;
    let hash = card_hash(&card, &art.art_ref, &art.artist);

    let dir = cache_dir.join("cards").join(sanitize(&card.name));
    let out = if foil { dir.join("foil").join(format!("{hash}.png")) } else { dir.join(format!("{hash}.png")) };
    if out.exists() && !force {
        return Ok(out);
    }
    fs::create_dir_all(out.parent().unwrap())?;

    let assets_abs = fs::canonicalize(assets_dir)?;
    let frame_abs = fs::canonicalize(&frame_rel)?;
    let art_abs = fs::canonicalize(&art.path)?;
    let html = build_html(&card, &frame_abs, &art_abs, &assets_abs, &art, foil);
    let html_path = std::env::temp_dir().join(format!("mtgbrain-render-{id}{}.html", u8::from(foil)));
    fs::write(&html_path, html)?;

    let status = Command::new(chrome)
        .args([
            "--headless", "--disable-gpu", "--hide-scrollbars",
            "--no-default-browser-check", "--no-first-run",
            "--force-device-scale-factor=1",
            "--run-all-compositor-stages-before-draw",
            "--virtual-time-budget=15000",
            &format!("--window-size={W},{H}"),
            &format!("--screenshot={}", out.display()),
            &format!("file://{}", html_path.display()),
        ])
        .status()
        .with_context(|| format!("running Chrome at {chrome}"))?;
    if !status.success() || !out.exists() {
        bail!("Chrome screenshot failed (check --chrome / MTGBRAIN_CHROME path)");
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
pub fn render_all(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str,
    foil: bool, force: bool, backend: Option<&str>,
) -> Result<()> {
    let ids: Vec<i64> = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut stmt = db.prepare("SELECT id FROM cube_cards WHERE in_db_found=1 ORDER BY id")?;
        let mut v = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            v.push(r.get::<_, i64>(0)?);
        }
        v
    };
    let total = ids.len();
    for (i, id) in ids.iter().enumerate() {
        match render_one(editor_db, assets_dir, cache_dir, chrome, *id, false, force, backend) {
            Ok(_) => print!("\r[{}/{total}] id {id}        ", i + 1),
            Err(e) => eprintln!("\n  id {id}: {e}"),
        }
        if foil {
            let _ = render_one(editor_db, assets_dir, cache_dir, chrome, *id, true, force, backend);
        }
        std::io::stdout().flush().ok();
    }
    println!("\ndone: {total} cards -> {}", cache_dir.join("cards").display());
    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>().trim_matches('_').to_string()
}

fn percent(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => o.push(b as char),
            _ => o.push_str(&format!("%{b:02X}")),
        }
    }
    o
}

fn esc(s: &str) -> String {
    s.chars().map(esc_char).collect()
}
fn esc_char(c: char) -> String {
    match c {
        '&' => "&amp;".into(),
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '"' => "&quot;".into(),
        other => other.to_string(),
    }
}
