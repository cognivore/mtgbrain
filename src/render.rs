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
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use base64::Engine;
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

/// On-card art window (the `.art` box) as fractions of the FACE: [x, y, w, h].
/// These are cardconjurer's packSeventh art bounds; they MUST stay in lock-step with
/// the AX/AY/AW/AH fed to `build_html` and with the frame PNG's transparent art hole.
/// A MANUAL crop is locked to this window's ASPECT so the boxed region maps 1:1 into
/// the art hole: `.art` paints with `background-size:cover`, and a matching aspect
/// means cover adds no further cropping — true WYSIWYG between editor box and card.
const ART_WINDOW_FRAC: [f64; 4] = [0.12, 0.0991, 0.7667, 0.4429];

/// Aspect (w/h) of the on-card art window — what a hand-positioned crop box locks to.
pub fn art_window_aspect() -> f64 {
    (ART_WINDOW_FRAC[2] * f64::from(FACE_W)) / (ART_WINDOW_FRAC[3] * f64::from(FACE_H))
}

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
    // Eighth-Edition (2003 / modern) frame pack: a COLOUR layer per frame + region
    // masks (frame/pinline/type/rules) the template composites via CSS, plus the
    // separate P/T boxes. Geometry comes from cardconjurer's pack8th.js.
    for letter in ["w", "u", "b", "r", "g", "m", "a", "c", "l", "wl", "ul", "bl", "rl", "gl", "ml"] {
        v.push((
            format!("{CC}/img/frames/8th/{letter}.png"),
            Box::leak(format!("frames8/{letter}.png").into_boxed_str()),
        ));
    }
    for mask in ["frame", "pinline", "type", "rules", "title"] {
        v.push((
            format!("{CC}/img/frames/8th/{mask}.png"),
            Box::leak(format!("frames8/mask_{mask}.png").into_boxed_str()),
        ));
    }
    for letter in ["w", "u", "b", "r", "g", "m", "a", "l"] {
        v.push((
            format!("{CC}/img/frames/8th/pt/{letter}.png"),
            Box::leak(format!("frames8/pt/{letter}.png").into_boxed_str()),
        ));
    }
    // old fonts (proprietary — kept out of git; personal-use only)
    v.push((format!("{CC}/fonts/goudy-medieval.ttf"), "fonts/goudy-medieval.ttf"));
    v.push((format!("{CC}/fonts/mplantin.ttf"), "fonts/mplantin.ttf"));
    // cardconjurer's italic is named mplantin-i.ttf; saved under our -italic.ttf name.
    v.push((format!("{CC}/fonts/mplantin-i.ttf"), "fonts/mplantin-italic.ttf"));
    v.push((format!("{CC}/fonts/matrix.ttf"), "fonts/matrix.ttf"));
    // mana font (OFL) — kept for any legacy use
    v.push((format!("{MANA}/fonts/mana.ttf"), "fonts/mana.ttf"));
    v.push((format!("{MANA}/css/mana.css"), "fonts/mana.css"));
    // cardconjurer's own mana-symbol SVGs (disk + glyph baked in) → pixel-match pips.
    let mut syms: Vec<String> =
        (0..=20).map(|n| n.to_string()).collect();
    for s in [
        "w", "u", "b", "r", "g", "c", "x", "s", "t", "untap", "e", "p",
        "wu", "wb", "ub", "ur", "br", "bg", "rg", "rw", "gw", "gu",
        "2w", "2u", "2b", "2r", "2g", "wp", "up", "bp", "rp", "gp",
        "hw", "hr",
    ] {
        syms.push(s.to_string());
    }
    for s in syms {
        v.push((
            format!("{CC}/img/manaSymbols/{s}.svg"),
            Box::leak(format!("mana/{s}.svg").into_boxed_str()),
        ));
    }
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
        .args(["-sSL", "--fail", "--max-time", "90", "-A", "Mozilla/5.0", "-o"])
        .arg(dst)
        .arg(url)
        .status()
        .context("running curl")?;
    if !status.success() {
        bail!("curl failed for {url}");
    }
    Ok(())
}

fn curl_post_json(url: &str, body: &str) -> Result<String> {
    let out = Command::new("curl")
        .args(["-sS", "--fail", "--max-time", "60", "-A", "Mozilla/5.0",
            "-H", "Content-Type: application/json", "-X", "POST", "-d", body, url])
        .output()
        .context("curl POST")?;
    if !out.status.success() {
        bail!("POST {url} failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ---------------------------------------------------------------------------
// card model (from editor DB, overrides applied)
// ---------------------------------------------------------------------------

struct Card {
    /// ORIGINAL card name (DB row) — used for ALL addressing: Scryfall/MPCfill lookup,
    /// art cache dir, render output path, flavor/set/rarity. NEVER the override.
    name: String,
    /// Printed title — the `name` override if set, else the original. Display only.
    display_name: String,
    mana_cost: String,
    type_line: String,
    oracle_text: String,
    flavor: String,
    /// True when the editor set a `flavor` override (even to empty). An explicit empty
    /// override SUPPRESSES flavor entirely; absence falls back to the Scryfall flavor.
    flavor_overridden: bool,
    power: String,
    toughness: String,
    loyalty: String,
    colors: String,
    /// WUBRG color identity (mana symbols in cost+rules). For LANDS (which have no
    /// `colors`) this picks the single-colour 7ED land frame, e.g. U → ul.png.
    color_identity: String,
    /// True when `color_identity` came from a MANUAL `color_identity` override (e.g. "UB"
    /// on a cost-removed card) — then it is authoritative for the frame + colour-indicator
    /// dot, reusing the coloured-artifact indicator for costless / off-colour-identity cards.
    ci_manual: bool,
    is_creature: bool,
    set: String,
    rarity: String,
    /// Printed "Illus." credit chosen in the editor (the artist pseudonym for GenAI
    /// art). Empty for cards still using real Scryfall art (which credit the real
    /// historical illustrator). NOT part of the `overrides` errata blob.
    illustrator: String,
    /// True when the card's printed text was changed for the cube (any `overrides`
    /// field and/or `errata_text`). Drives the errata scroll overlay.
    is_errata: bool,
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
    // Best-effort: tolerate older editor DBs that predate the `illustrator` column.
    let illustrator: String = db
        .query_row("SELECT COALESCE(illustrator,'') FROM cube_cards WHERE id=?1", params![id], |r| r.get(0))
        .unwrap_or_default();
    let errata_text: String = db
        .query_row("SELECT COALESCE(errata_text,'') FROM cube_cards WHERE id=?1", params![id], |r| r.get(0))
        .unwrap_or_default();
    // Colour identity: a MANUAL `color_identity` override (e.g. "UB" on a cost-removed card)
    // is authoritative for the frame + colour-indicator dot; otherwise the solved DB column.
    let ci_override = o
        .get("color_identity")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let ci_manual = ci_override.is_some();
    let color_identity: String = match ci_override {
        Some(s) => ci_letters(s).iter().map(char::to_string).collect::<Vec<_>>().join(", "),
        None => db
            .query_row("SELECT COALESCE(color_identity,'') FROM cube_cards WHERE id=?1", params![id], |r| r.get(0))
            .unwrap_or_default(),
    };
    // Errata scroll: auto when the printed text actually differs (any override field
    // besides the toggle itself, or an errata note), but a manual `errata_scroll`
    // override ("on"/"off"/true/false) in the overrides blob forces it either way.
    let scroll_override: Option<bool> = o.get("errata_scroll").and_then(|v| match v {
        Value::Bool(b) => Some(*b),
        Value::String(s) => match s.trim().to_lowercase().as_str() {
            "on" | "true" | "1" | "yes" => Some(true),
            "off" | "false" | "0" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    });
    // Ribbon = FUNCTIONAL change only. Cosmetic overrides (name, flavor) never trip it;
    // a manual `errata_scroll` override still forces it either way.
    const FUNCTIONAL: &[&str] =
        &["mana_cost", "type", "oracle_text", "power", "toughness", "loyalty", "colors"];
    let auto_errata = o.as_object()
        .is_some_and(|m| m.keys().any(|k| FUNCTIONAL.contains(&k.as_str())))
        || !errata_text.trim().is_empty();
    let is_errata = scroll_override.unwrap_or(auto_errata);
    let display_name = pick("name", Some(name.clone()));
    Ok(Card {
        name, // ORIGINAL — addressing/lookup; the "name" override only affects the title
        display_name,
        mana_cost: pick("mana_cost", mc),
        type_line: pick("type", tl),
        oracle_text: pick("oracle_text", ot),
        // "flavor" override wins; if the key is ABSENT, render_one fills from Scryfall.
        // An explicit empty override suppresses flavor (flavor_overridden gates the fill).
        flavor: pick("flavor", None),
        flavor_overridden: o.get("flavor").is_some(),
        power: pick("power", p),
        toughness: pick("toughness", t),
        loyalty: pick("loyalty", l),
        colors: pick("colors", colors),
        color_identity,
        ci_manual,
        is_creature: is_cr != 0,
        // "set"/"rarity" overrides pick the set-symbol printing (e.g. force "usg" when the
        // oldest print is a symbol-less judge promo); else filled from Scryfall in render_one.
        set: pick("set", None),
        rarity: pick("rarity", None),
        illustrator,
        is_errata,
    })
}

fn card_hash(c: &Card, art_ref: &str, artist: &str) -> String {
    // Use the FRAME file in the hash so a corrected frame colour forces a re-render.
    let canon = json!({
        "name": c.name, "display_name": c.display_name, "mana_cost": c.mana_cost, "type": c.type_line,
        "oracle_text": c.oracle_text, "flavor": c.flavor, "power": c.power, "toughness": c.toughness,
        "loyalty": c.loyalty, "frame_file": frame_file(c), "is_creature": c.is_creature,
        "set": c.set, "rarity": c.rarity, "errata": c.is_errata, "ci": c.color_identity,
        "art_ref": art_ref, "artist": artist, "frame": "seventh", "v": 11,
    });
    let mut h = Sha256::new();
    h.update(canon.to_string().as_bytes());
    format!("{:x}", h.finalize())
}

/// The card's WUBRG colours for frame selection. Derived from the mana cost's
/// coloured pips (authoritative — the DB `colors` field is sometimes wrong, e.g.
/// "W" on a {2}{G} card). Falls back to `colors` only when the cost has no coloured
/// mana (colour-indicator cards). Hybrids count every colour; {2/W}/{W/P} count W.
fn card_colors(c: &Card) -> Vec<char> {
    let mut set: Vec<char> = Vec::new();
    let mut chars = c.mana_cost.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut sym = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                sym.push(c2);
            }
            for k in sym.chars().filter(|k| "WUBRG".contains(*k)) {
                if !set.contains(&k) {
                    set.push(k);
                }
            }
        }
    }
    if set.is_empty() {
        set = c.colors.chars().filter(|ch| "WUBRG".contains(*ch)).collect();
    }
    set
}

/// WUBRG letters of a colour-identity string, accepting "U, B", "UB", "ub", etc.,
/// de-duplicated and sorted in WUBRG order.
fn ci_letters(ci: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for ch in ci.chars().map(|c| c.to_ascii_uppercase()) {
        if "WUBRG".contains(ch) && !out.contains(&ch) {
            out.push(ch);
        }
    }
    const ORDER: &str = "WUBRG";
    out.sort_by_key(|c| ORDER.find(*c).unwrap_or(9));
    out
}

/// A single WUBRG letter → its 7ED colour frame file.
fn color_frame(ch: char) -> &'static str {
    match ch {
        'W' => "frames/w.png",
        'U' => "frames/u.png",
        'B' => "frames/b.png",
        'R' => "frames/r.png",
        _ => "frames/g.png",
    }
}

fn frame_file(c: &Card) -> &'static str {
    let t = c.type_line.to_lowercase();
    let is_land = t.contains("land");
    let is_artifact = t.contains("artifact");
    let mut cols: Vec<char> = card_colors(c);
    if is_land {
        // Lands have no `colors`; the single-colour 7ED land frame is chosen from the
        // colour identity (the {U} in "Add {U}"). Only a MONO identity gets a tinted
        // land frame; colourless or multi → the generic land frame.
        let lc: Vec<char> = c.color_identity.chars().filter(|ch| "WUBRG".contains(*ch)).collect();
        return match lc.as_slice() {
            ['W'] => "frames/wl.png", ['U'] => "frames/ul.png",
            ['B'] => "frames/bl.png", ['R'] => "frames/rl.png",
            ['G'] => "frames/gl.png", _ => "frames/l.png",
        };
    }
    // Coloured ARTIFACTS keep the brown artifact frame as the base; their colour is
    // applied only to the text box via a clipped overlay (see frame_overlay).
    if is_artifact {
        return "frames/a.png";
    }
    // A MANUAL colour identity (e.g. "UB" on a cost-removed card) drives the frame — mono →
    // that colour, multi → gold — overriding the (now absent/stale) mana-cost colours.
    if c.ci_manual {
        cols = ci_letters(&c.color_identity);
    }
    match cols.len() {
        0 => "frames/c.png",
        1 => color_frame(cols[0]),
        _ => "frames/m.png", // Seventh has a real gold multicolor frame.
    }
}

/// For a COLOURED ARTIFACT, the colour frame whose TEXT BOX is overlaid (clipped) onto
/// the brown artifact base — mono colour for one colour, gold (`m`) for multicolour.
/// None for colourless artifacts and all non-artifacts (their base frame already fits).
#[allow(dead_code)]
fn frame_overlay(c: &Card) -> Option<&'static str> {
    let t = c.type_line.to_lowercase();
    if !t.contains("artifact") || t.contains("land") {
        return None;
    }
    match card_colors(c).as_slice() {
        [] => None,
        [one] => Some(match one {
            'W' => "frames/w.png", 'U' => "frames/u.png", 'B' => "frames/b.png",
            'R' => "frames/r.png", _ => "frames/g.png",
        }),
        _ => Some("frames/m.png"),
    }
}

// ---------------------------------------------------------------------------
// art acquisition (oldest Scryfall printing + artist; MPC-Autofill optional)
// ---------------------------------------------------------------------------

#[allow(clippy::struct_field_names)]
struct Art {
    path: PathBuf,
    art_ref: String,
    artist: String,
    year: String,
}

/// A settled MPCfill art choice: the cropped output plus everything needed to
/// RE-CROP it in place later (proxy source + box + aspect) without re-picking — so
/// crop-inset tweaks apply on the next render with no CV/Claude/network roundtrip.
struct Pick {
    out: PathBuf,
    art_ref: String,
    proxy: PathBuf,
    bx: [f64; 4],
    aspect: f64,
}

/// Acquire art: oldest Scryfall printing as reference+fallback (+artist/year); if a
/// MPC-Autofill `backend` and `ANTHROPIC_API_KEY` are present, pull all high-DPI
/// candidates (cached per bucket), pick the highest-DPI one whose illustration
/// Claude confirms matches the old art, and crop its art window cleanly.
fn acquire_art(name: &str, cache_root: &Path, card_dir: &Path, backend: Option<&str>, latest: bool) -> Result<Art> {
    let art_dir = cache_root.join("art");
    fs::create_dir_all(&art_dir)?;
    fs::create_dir_all(card_dir)?;

    let key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty());
    let can_repick = backend.is_some() && key.is_some();

    // FAST PATH: once the MPCfill crop is chosen, art.json records the source proxy +
    // crop box + aspect. Re-render then RE-CROPS in place from that box (a cheap local
    // crop, NO network/CV/Claude) so crop-inset changes apply without re-picking. If the
    // sidecar predates box-recording, we re-pick (CV on cached downloads) when a backend
    // is available, else just reuse the cached crop. Delete art.png + art.json to force a
    // full re-pick.
    let sidecar = card_dir.join("art.json");
    let crop = card_dir.join("art.png");
    if sidecar.exists() {
        if let Ok(v) = serde_json::from_str::<Value>(&fs::read_to_string(&sidecar)?) {
            let art_ref = v["art_ref"].as_str().unwrap_or("mpcfill:cached").to_string();
            let artist = v["artist"].as_str().unwrap_or("").to_string();
            let year = v["year"].as_str().unwrap_or("2001").to_string();
            // A MANUAL crop (operator hand-positioned the box in the editor) is sticky.
            let manual = v["manual"].as_bool().unwrap_or(false);
            // A GenAI choice is DELIBERATE — never re-pick/clobber it. Self-heal the crop
            // from the genai source (genai/<artist>/<hash>.png) if it has gone missing.
            if let Some(rest) = art_ref.strip_prefix("genai:") {
                if !crop.exists() {
                    if let Some((a, h)) = rest.rsplit_once(':') {
                        let src = card_dir.join("genai").join(sanitize_bucket(a)).join(format!("{h}.png"));
                        if src.exists() {
                            fs::copy(&src, &crop)?;
                        }
                    }
                }
                if crop.exists() {
                    return Ok(Art { path: crop, art_ref, artist, year });
                }
            }
            let bx = v["box"].as_array().filter(|a| a.len() == 4).map(|a| {
                let g = |i: usize| a[i].as_f64().unwrap_or(0.0);
                [g(0), g(1), g(2), g(3)]
            });
            let proxy = v["proxy"].as_str().map(PathBuf::from);
            // MANUAL crop FIRST: crop the source EXACTLY (no safety insets, no aspect
            // re-derivation) — the operator already positioned a window-aspect box by hand,
            // so it maps 1:1 into the art hole. The crop signature in art_ref makes the
            // render cache key move whenever the box moves (never reuse a stale crop).
            if manual {
                if let (Some(bx), Some(proxy)) = (bx, &proxy) {
                    if proxy.exists() {
                        crop_exact(proxy, bx, &crop)?;
                        let asp = v["art_aspect"].as_f64().unwrap_or_else(art_window_aspect);
                        let art_ref = format!("{art_ref}#{}", crop_sig(proxy, &bx, asp, true));
                        return Ok(Art { path: crop, art_ref, artist, year });
                    }
                }
            }
            // In-place re-crop from the stored proxy + box (picks up inset changes). The
            // crop signature is folded into art_ref so an auto re-pick that moves the box
            // produces a distinct cached render too.
            if let (Some(bx), Some(proxy), Some(asp)) = (bx, &proxy, v["art_aspect"].as_f64()) {
                if proxy.exists() {
                    crop_to(proxy, bx, &crop, asp)?;
                    let art_ref = format!("{art_ref}#{}", crop_sig(proxy, &bx, asp, false));
                    return Ok(Art { path: crop, art_ref, artist, year });
                }
            }
            // Any SETTLED crop is reused — do NOT re-pick and clobber a deliberate choice
            // (manual swap, reprint pick, etc.). Only the legacy boxless auto-cache
            // ("mpcfill:cached") is upgraded to a box-bearing pick when we can re-pick.
            if crop.exists() && !(can_repick && art_ref == "mpcfill:cached") {
                return Ok(Art { path: crop, art_ref, artist, year });
            }
            // else fall through → re-pick (will write a box-bearing sidecar).
        }
    }

    // 8ED back cube wants the LATEST high-DPI art; the old frame wants the oldest (authentic
    // pre-modern) print. Either becomes the reference the MPCfill matcher targets.
    let (mut ref_path, mut artist, mut year, set) = if latest {
        scryfall_latest(name, &art_dir)?
    } else {
        scryfall_oldest(name, &art_dir)?
    };
    // Optional per-render art-source override: MTGBRAIN_ART_SET=<setcode> matches the
    // MPCfill art against THAT printing's art_crop instead of the oldest one (the set
    // symbol still comes from the oldest printing). Used to pick a reprint's art.
    if let Some(s) = std::env::var("MTGBRAIN_ART_SET").ok().filter(|s| !s.is_empty()) {
        match scryfall_printing(name, &s, &art_dir) {
            Ok((p, a, y)) => {
                ref_path = p;
                artist = a;
                year = y;
                eprintln!("  art: using {s} printing's art for {name:?} (override)");
            }
            Err(e) => eprintln!("  art: MTGBRAIN_ART_SET={s} failed ({e}) — using oldest"),
        }
    }
    if let (Some(base), Some(key)) = (backend, key.as_deref()) {
        match mpcfill_pick(name, base, cache_root, card_dir, &ref_path, key) {
            Ok(Some(p)) => {
                // Sidecar keeps the BASE art_ref; the returned art_ref carries the crop
                // signature so the render cache key tracks the actual box (see crop_sig).
                let art_ref = format!("{}#{}", p.art_ref, crop_sig(&p.proxy, &p.bx, p.aspect, false));
                let _ = fs::write(
                    &sidecar,
                    json!({
                        "art_ref": p.art_ref, "artist": artist, "year": year,
                        "proxy": p.proxy.to_string_lossy(), "box": p.bx, "art_aspect": p.aspect,
                    })
                    .to_string(),
                );
                return Ok(Art { path: p.out, art_ref, artist, year });
            }
            Ok(None) => eprintln!("  art: no MPCfill match for {name:?} — using Scryfall art_crop"),
            Err(e) => eprintln!("  art: MPCfill/Claude error ({e}) — using Scryfall art_crop"),
        }
    }
    Ok(Art { path: ref_path, art_ref: format!("scryfall:{set}:{year}:art_crop"), artist, year })
}

/// Oldest paper printing's art_crop (authentic pre-modern art) + artist/year/set.
fn scryfall_oldest(name: &str, art_dir: &Path) -> Result<(PathBuf, String, String, String)> {
    let meta_file = art_dir.join(format!("{}.json", sanitize(name)));
    let dst = art_dir.join(format!("{}.jpg", sanitize(name)));
    // Reuse cached Scryfall metadata + art_crop if already downloaded.
    if !meta_file.exists() || !dst.exists() {
        let meta_url = format!(
            "https://api.scryfall.com/cards/search?order=released&dir=asc&unique=prints&q={}",
            percent(&format!("!\"{name}\" game:paper"))
        );
        curl_to_file(&meta_url, &meta_file)?;
    }
    let v: Value = serde_json::from_str(&fs::read_to_string(&meta_file)?)
        .with_context(|| format!("parsing Scryfall metadata for {name}"))?;
    let first = v["data"]
        .as_array()
        .and_then(|a| a.first())
        .with_context(|| format!("no Scryfall print found for {name}"))?;
    let artist = first["artist"].as_str().unwrap_or("").to_string();
    let year = first["released_at"].as_str().unwrap_or("").chars().take(4).collect::<String>();
    let set = first["set"].as_str().unwrap_or("?").to_string();
    if !dst.exists() {
        let art_url = first["image_uris"]["art_crop"]
            .as_str()
            .or_else(|| first["card_faces"][0]["image_uris"]["art_crop"].as_str())
            .with_context(|| format!("no art_crop for {name}"))?;
        curl_to_file(art_url, &dst)?;
    }
    Ok((dst, artist, year, set))
}

/// NEWEST paper printing that still has a HIGH-RES art scan — the up-to-date, best-quality
/// illustration for the modern (8ED) back cube. Falls back to the newest printing with any
/// art_crop if none are flagged `highres_scan`. Cached separately so it never clobbers the
/// oldest-printing metadata the old frame uses.
fn scryfall_latest(name: &str, art_dir: &Path) -> Result<(PathBuf, String, String, String)> {
    let meta_file = art_dir.join(format!("{}__latest.json", sanitize(name)));
    let dst = art_dir.join(format!("{}__latest.jpg", sanitize(name)));
    if !meta_file.exists() || !dst.exists() {
        let meta_url = format!(
            "https://api.scryfall.com/cards/search?order=released&dir=desc&unique=prints&q={}",
            percent(&format!("!\"{name}\" game:paper"))
        );
        // Scryfall 404s a no-match exact search (a DFC FRONT name like "Value Town", or a
        // SPLIT combined name like "Bind // Liberate"); resolve via named?exact then ?fuzzy.
        if curl_to_file(&meta_url, &meta_file).is_err() {
            let exact = format!("https://api.scryfall.com/cards/named?exact={}", percent(name));
            let fuzzy = format!("https://api.scryfall.com/cards/named?fuzzy={}", percent(name));
            if curl_to_file(&exact, &meta_file).is_err() {
                curl_to_file(&fuzzy, &meta_file)?;
            }
        }
    }
    let mut v: Value = serde_json::from_str(&fs::read_to_string(&meta_file)?)
        .with_context(|| format!("parsing Scryfall metadata for {name}"))?;
    // Normalise to a `data` array — a single `named` card has no `data` wrapper.
    let prints: Vec<Value> = match v["data"].as_array() {
        Some(arr) if !arr.is_empty() => arr.clone(),
        _ if v["name"].is_string() => vec![v.clone()],
        _ => vec![],
    };
    // If it came from `named` (single card), rewrite the cache as {data:[card]} so the
    // adventure-face reader (expects data[0].card_faces) sees it too.
    if v["data"].as_array().map_or(true, |a| a.is_empty()) && !prints.is_empty() {
        v = json!({ "data": prints.clone() });
        let _ = fs::write(&meta_file, v.to_string());
    }
    let art_crop_of = |c: &Value| -> Option<String> {
        c["image_uris"]["art_crop"]
            .as_str()
            .or_else(|| c["card_faces"][0]["image_uris"]["art_crop"].as_str())
            .map(ToString::to_string)
    };
    // newest high-res scan with art; else newest with any art_crop.
    let pick = prints
        .iter()
        .find(|c| c["image_status"].as_str() == Some("highres_scan") && art_crop_of(c).is_some())
        .or_else(|| prints.iter().find(|c| art_crop_of(c).is_some()))
        .with_context(|| format!("no printing with art for {name}"))?;
    let artist = pick["artist"].as_str().unwrap_or("").to_string();
    let year = pick["released_at"].as_str().unwrap_or("").chars().take(4).collect::<String>();
    let set = pick["set"].as_str().unwrap_or("?").to_string();
    if !dst.exists() {
        let art_url = art_crop_of(pick).with_context(|| format!("no art_crop for {name}"))?;
        curl_to_file(&art_url, &dst)?;
    }
    Ok((dst, artist, year, set))
}

/// art_crop of a SPECIFIC printing (set code) — used to override which printing's
/// art we match against, e.g. to pull a reprint's illustration instead of the
/// oldest one. Cached separately (`<name>__<set>.jpg`) so it never clobbers the
/// oldest-printing metadata used for the set symbol.
fn scryfall_printing(name: &str, set: &str, art_dir: &Path) -> Result<(PathBuf, String, String)> {
    let tag = format!("{}__{}", sanitize(name), set);
    let meta_file = art_dir.join(format!("{tag}.json"));
    let dst = art_dir.join(format!("{tag}.jpg"));
    if !meta_file.exists() || !dst.exists() {
        let url = format!(
            "https://api.scryfall.com/cards/named?exact={}&set={}",
            percent(name),
            percent(set)
        );
        curl_to_file(&url, &meta_file)?;
    }
    let v: Value = serde_json::from_str(&fs::read_to_string(&meta_file)?)
        .with_context(|| format!("parsing Scryfall printing {name} [{set}]"))?;
    let artist = v["artist"].as_str().unwrap_or("").to_string();
    let year = v["released_at"].as_str().unwrap_or("").chars().take(4).collect::<String>();
    if !dst.exists() {
        let art_url = v["image_uris"]["art_crop"]
            .as_str()
            .or_else(|| v["card_faces"][0]["image_uris"]["art_crop"].as_str())
            .with_context(|| format!("no art_crop for {name} [{set}]"))?;
        curl_to_file(art_url, &dst)?;
    }
    Ok((dst, artist, year))
}

struct McCand {
    dpi: i64,
    identifier: String,
    label: String,
    ext: String,
    bucket: String,
    link: String,
    ext_link: String,
}

/// All MPCfill source `[pk,true]` pairs (cached) — needed to enable every source.
fn mpcfill_sources(base: &str, cache_root: &Path) -> Result<String> {
    let f = cache_root.join("mpcfill_sources.json");
    if !f.exists() {
        curl_to_file(&format!("{}/2/sources/", base.trim_end_matches('/')), &f)?;
    }
    let v: Value = serde_json::from_str(&fs::read_to_string(&f)?)?;
    let pairs: Vec<String> = v["results"]
        .as_object()
        .map(|o| o.values().filter_map(|s| s["pk"].as_i64()).map(|pk| format!("[{pk},true]")).collect())
        .unwrap_or_default();
    Ok(format!("[{}]", pairs.join(",")))
}

/// Search MPCfill for `name` and download EVERY candidate into the card cache, filtering
/// to `>= min_dpi` (with a low-floor fallback so a card is never left with zero options).
/// Records a DPI index at `mpcfill/index.json` (rel -> {dpi,bucket,label,ext}) so the
/// editor can label each proxy's resolution. Returns the candidates (already on disk); no
/// CV/Claude pick happens here — the auto-pick and the editor's "Fetch all" both use this.
fn mpcfill_search_download(
    name: &str, base: &str, cache_root: &Path, card_dir: &Path, min_dpi: i64,
) -> Result<Vec<McCand>> {
    let base = base.trim_end_matches('/');
    let sources = mpcfill_sources(base, cache_root)?;
    let run = |min: i64| -> Result<Vec<String>> {
        let search = format!(
            r#"{{"searchSettings":{{"searchTypeSettings":{{"fuzzySearch":true,"filterCardbacks":false}},"sourceSettings":{{"sources":{sources}}},"filterSettings":{{"minimumDPI":{min},"maximumDPI":1500,"maximumSize":50,"languages":[],"includesTags":[],"excludesTags":["NSFW"]}}}},"queries":[{{"query":{q},"cardType":"CARD"}}]}}"#,
            q = serde_json::to_string(name)?
        );
        let resp = curl_post_json(&format!("{base}/2/editorSearch/"), &search)?;
        let v: Value = serde_json::from_str(&resp).context("editorSearch response")?;
        Ok(v["results"][name]["CARD"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default())
    };
    let mut ids = run(min_dpi)?;
    if ids.is_empty() && min_dpi > 100 {
        ids = run(100)?; // never leave a card with zero options just because all proxies are <600 DPI
    }
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let cards_req = json!({ "cardIdentifiers": ids }).to_string();
    let resp = curl_post_json(&format!("{base}/2/cards/"), &cards_req)?;
    let cv: Value = serde_json::from_str(&resp).context("cards response")?;
    let cands: Vec<McCand> = cv["results"]
        .as_object()
        .map(|o| {
            o.values()
                .map(|c| McCand {
                    dpi: c["dpi"].as_i64().unwrap_or(0),
                    identifier: c["identifier"].as_str().unwrap_or("").to_string(),
                    label: c["name"].as_str().unwrap_or("").to_string(),
                    ext: c["extension"].as_str().unwrap_or("jpg").to_string(),
                    bucket: c["sourceName"].as_str().unwrap_or("unknown").to_string(),
                    link: c["downloadLink"].as_str().unwrap_or("").to_string(),
                    ext_link: c["sourceExternalLink"].as_str().unwrap_or("").to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Cache EVERY download under cards/<Card>/mpcfill/<bucket>/<id>.<ext> (never re-fetch),
    // and record its DPI in the per-card index.
    let mut index = read_json(&card_dir.join("mpcfill").join("index.json"))
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    for c in &cands {
        let dir = card_dir.join("mpcfill").join(sanitize_bucket(&c.bucket));
        fs::create_dir_all(&dir)?;
        let dst = dir.join(format!("{}.{}", c.identifier, c.ext));
        if !dst.exists() && !c.link.is_empty() && curl_to_file(&c.link, &dst).is_err() {
            eprintln!("    (download failed) {}", c.identifier);
        }
        record_bucket(&c.bucket, &c.ext_link)?;
        index.insert(
            format!("mpcfill/{}/{}.{}", sanitize_bucket(&c.bucket), c.identifier, c.ext),
            json!({"dpi": c.dpi, "bucket": c.bucket, "label": c.label, "ext": c.ext}),
        );
    }
    let _ = fs::write(
        card_dir.join("mpcfill").join("index.json"),
        Value::Object(index).to_string(),
    );
    Ok(cands)
}

/// Pull + cache all candidates, pick the best art-matching one, crop it.
fn mpcfill_pick(
    name: &str,
    base: &str,
    cache_root: &Path,
    card_dir: &Path,
    ref_path: &Path,
    key: &str,
) -> Result<Option<Pick>> {
    let base = base.trim_end_matches('/');
    // Search + download every candidate (>=600 DPI, low-floor fallback) into the cache and
    // record the DPI index. The editor's "Fetch all" uses the SAME path, so the picker and
    // the auto-pick share one set of cached proxies.
    let mut cands = mpcfill_search_download(name, base, cache_root, card_dir, 600)?;
    if cands.is_empty() {
        return Ok(None);
    }

    // Prefer plain prints (skip extended/full-art/showcase/etc), then highest DPI.
    let banned = ["extended", "full art", "fullart", "textless", "showcase", "borderless", "alt"];
    cands.sort_by(|a, b| {
        let pa = banned.iter().any(|w| a.label.to_lowercase().contains(w));
        let pb = banned.iter().any(|w| b.label.to_lowercase().contains(w));
        pa.cmp(&pb).then(b.dpi.cmp(&a.dpi))
    });

    if std::env::var("MTGBRAIN_DEBUG_PICK").is_ok() {
        for (i, c) in cands.iter().enumerate() {
            eprintln!("  cand[{i}] dpi={} bucket={} label={:?}", c.dpi, c.bucket, c.label);
        }
    }

    let out = card_dir.join("art.png");
    // TRUE art aspect from Scryfall's canonical art_crop — drives crop height so the
    // proxy's type line can't leak into our art window.
    let art_aspect = image_aspect(ref_path).unwrap_or(1.30);
    let path_of = |c: &McCand| {
        card_dir.join("mpcfill").join(sanitize_bucket(&c.bucket)).join(format!("{}.{}", c.identifier, c.ext))
    };

    // PRIMARY: deterministic computer-vision match (template-match the reference art
    // inside each proxy). Pick the highest-DPI candidate whose CV score clears the
    // threshold; crop exactly where the art was located. No LLM, no cost.
    const CV_OK: f64 = 0.42;
    let mut best_cv: Option<(usize, [f64; 4], f64)> = None;
    for (i, c) in cands.iter().take(12).enumerate() {
        let file = path_of(c);
        if !file.exists() {
            continue;
        }
        if let Some((bx, score)) = cv_art_box(ref_path, &file) {
            if score >= CV_OK {
                crop_to(&file, bx, &out, art_aspect)?;
                return Ok(Some(Pick {
                    out: out.clone(),
                    art_ref: format!("mpcfill:{}:dpi{}:cv{:.2}", c.identifier, c.dpi, score),
                    proxy: file,
                    bx,
                    aspect: art_aspect,
                }));
            }
            if best_cv.is_none_or(|(_, _, b)| score > b) {
                best_cv = Some((i, bx, score));
            }
        }
    }

    // FALLBACK: Claude vision (art match + box) when CV is unsure (e.g. recropped /
    // alt art that doesn't pixel-match the reference). Evaluate ALL top candidates and,
    // among those Claude confirms as the same artwork, prefer the MOST-BORDERED one: a
    // normal-frame proxy insets the painting from the card edge (left inset x well > 0,
    // narrower width), whereas an extended/borderless print runs the art to the edges
    // (x ≈ 0, w ≈ 0.92) and bakes the title bar into our crop. Rank by left inset x, then
    // by tighter width — this reliably skips full-art proxies for the clean original print.
    let bordered = |b: &[f64; 4]| b[0] - b[2] * 0.15; // reward left inset, lightly penalise width
    let mut best_box: Option<(usize, [f64; 4])> = None;
    let mut best_match: Option<(usize, [f64; 4])> = None;
    for (i, c) in cands.iter().take(8).enumerate() {
        let file = path_of(c);
        if !file.exists() {
            continue;
        }
        match claude_match_and_box(key, ref_path, &file) {
            Ok((same, bx)) => {
                if std::env::var("MTGBRAIN_DEBUG_PICK").is_ok() {
                    eprintln!("  claude cand[{i}] {} same={same} box={bx:?}", c.bucket);
                }
                if same && best_match.is_none_or(|(_, b)| bordered(&bx) > bordered(&b)) {
                    best_match = Some((i, bx));
                }
                best_box.get_or_insert((i, bx));
            }
            Err(e) => eprintln!("    (vision error) {e}"),
        }
    }
    if let Some((i, bx)) = best_match {
        let c = &cands[i];
        let proxy = path_of(c);
        crop_to(&proxy, bx, &out, art_aspect)?;
        return Ok(Some(Pick {
            out: out.clone(),
            art_ref: format!("mpcfill:{}:dpi{}", c.identifier, c.dpi),
            proxy,
            bx,
            aspect: art_aspect,
        }));
    }
    // Last resort: the best CV box (even if below threshold) beats an unverified guess.
    if let Some((i, bx, score)) = best_cv {
        let proxy = path_of(&cands[i]);
        crop_to(&proxy, bx, &out, art_aspect)?;
        return Ok(Some(Pick {
            out: out.clone(),
            art_ref: format!("mpcfill:{}:dpi{}:cv{:.2}", cands[i].identifier, cands[i].dpi, score),
            proxy,
            bx,
            aspect: art_aspect,
        }));
    }
    if let Some((i, bx)) = best_box {
        let proxy = path_of(&cands[i]);
        crop_to(&proxy, bx, &out, art_aspect)?;
        return Ok(Some(Pick {
            out: out.clone(),
            art_ref: format!("mpcfill:{}:dpi{}:unconfirmed", cands[i].identifier, cands[i].dpi),
            proxy,
            bx,
            aspect: art_aspect,
        }));
    }
    Ok(None)
}

/// Ask Claude: does candidate's illustration match the reference art? + art box.
fn claude_match_and_box(key: &str, ref_path: &Path, cand: &Path) -> Result<(bool, [f64; 4])> {
    let ref_b64 = img_b64_small(ref_path, 700)?;
    let cand_b64 = img_b64_small(cand, 1024)?;
    let prompt = "Image 1 is reference art from an older printing. Image 2 is a full proxy card \
        (it may use a MODERN frame with a title bar, type bar, mana symbols, set symbol and an \
        outer border). Return ONLY compact JSON: \
        {\"same_artwork\":true|false,\"art_box\":{\"x\":..,\"y\":..,\"w\":..,\"h\":..}}. \
        same_artwork = whether image 2's illustration depicts the SAME painting as image 1. \
        art_box = the painted illustration ONLY, as FRACTIONS of image 2 (0..1). It MUST exclude \
        EVERYTHING that is not painting: the title bar and any text above the art, the type/rules \
        bars and text below it, mana symbols, set symbol, and the entire outer border/frame on all \
        four sides. The TOP edge in particular must sit strictly BELOW the title bar — no metallic \
        or coloured frame strip may remain. If you are unsure where the inner art edge is, crop \
        TIGHTER (a little into the painting) rather than risk leaving any frame.";
    let body = json!({
        "model": "claude-opus-4-8",
        "max_tokens": 300,
        "messages": [{"role":"user","content":[
            {"type":"image","source":{"type":"base64","media_type":"image/jpeg","data":ref_b64}},
            {"type":"image","source":{"type":"base64","media_type":"image/jpeg","data":cand_b64}},
            {"type":"text","text":prompt}
        ]}]
    });
    let body_file = std::env::temp_dir().join("mtgbrain-claude-req.json");
    fs::write(&body_file, body.to_string())?;
    let out = Command::new("curl")
        .args(["-sS", "--max-time", "90", "https://api.anthropic.com/v1/messages",
            "-H", &format!("x-api-key: {key}"), "-H", "anthropic-version: 2023-06-01",
            "-H", "content-type: application/json", "--data"])
        .arg(format!("@{}", body_file.display()))
        .output()
        .context("curl claude")?;
    if !out.status.success() {
        bail!("claude request failed");
    }
    let resp: Value = serde_json::from_slice(&out.stdout).context("claude response")?;
    let text = resp["content"][0]["text"].as_str().unwrap_or("");
    let json_str = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let parsed: Value = serde_json::from_str(json_str).with_context(|| format!("parsing vision JSON: {text}"))?;
    let same = parsed["same_artwork"].as_bool().unwrap_or(false);
    let b = &parsed["art_box"];
    let g = |k: &str| b[k].as_f64().unwrap_or(0.0);
    Ok((same, [g("x"), g("y"), g("w"), g("h")]))
}

fn img_b64_small(path: &Path, max: u32) -> Result<String> {
    let img = image::open(path).with_context(|| format!("opening {}", path.display()))?;
    let small = img.thumbnail(max, max);
    let mut buf = Vec::new();
    small.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

/// Crop `path` to the fractional box and save as PNG at `out`.
///
/// A **safety inset** shaves a margin off the vision box so a slightly-loose crop
/// can never leak the proxy's frame into our art window. It's invisible in the
/// final card because the art window is `background-size:cover` (it crops to fill
/// anyway). The top is shaved a touch harder — that's where the title bar lives.
/// Crop the proxy to the art. `bx` is the LLM's art box; `art_aspect` is the TRUE
/// art width/height from Scryfall's canonical art_crop. We trust the box's top, left
/// and width, but DERIVE the height from `art_aspect` (anchored at the box top) — the
/// LLM box height is unreliable and routinely ran into the proxy's type line, baking
/// "Creature — …" + set symbol into our art window. Aspect-from-top kills that.
fn crop_to(path: &Path, bx: [f64; 4], out: &Path, art_aspect: f64) -> Result<()> {
    // Insets (fractions of the art region) shave the proxy's thin art-window border
    // lines off every edge — a bit more at the BOTTOM where the inner pinline + the
    // type bar sit. The art window is `cover`, so this slight zoom is invisible.
    // Bias HARD toward cropping into the painting rather than ever leaving a sliver of
    // the proxy's frame: it's invisible (the art window is `cover`) and a stray frame
    // line at the art edge is far more jarring than losing a few px of art.
    const SIDE: f64 = 0.030;
    const TOP: f64 = 0.052;
    const BOT: f64 = 0.050;

    let img = image::open(path).with_context(|| format!("opening {}", path.display()))?;
    let (iw, ih) = (f64::from(img.width()), f64::from(img.height()));

    let bw_full = bx[2] * iw;
    // Height = the SMALLER of (a) the true-art-aspect estimate and (b) the box's own
    // height. Capping by both stops EITHER a too-small aspect (proxy framed wider than
    // the reference printing) OR a too-tall box from spilling into the type line.
    let ah = (bw_full / art_aspect.max(0.1)).min(bx[3] * ih);

    let bx0 = bx[0] * iw + bw_full * SIDE;
    let by0 = bx[1] * ih + ah * TOP;
    let bw = bw_full * (1.0 - 2.0 * SIDE);
    let bh = (ah * (1.0 - TOP - BOT)).min(ih - by0);

    let x = bx0.clamp(0.0, iw - 1.0) as u32;
    let y = by0.clamp(0.0, ih - 1.0) as u32;
    let w = bw.clamp(1.0, iw - f64::from(x)) as u32;
    let h = bh.clamp(1.0, ih - f64::from(y)) as u32;
    img.crop_imm(x, y, w, h).save(out).with_context(|| format!("saving {}", out.display()))?;
    Ok(())
}

/// Crop `path` to EXACTLY the fractional box (clamped to the image), with NO safety
/// insets and NO aspect re-derivation — for a MANUAL crop the operator positioned by
/// hand in the editor. The box is already locked to the art-window aspect, so the
/// result drops straight into the card's art hole (`background-size:cover`, matching
/// aspect ⇒ no further crop). Contrast `crop_to`, which shaves insets + caps height by
/// the reference aspect to hide an AUTO pick's frame leakage.
fn crop_exact(path: &Path, bx: [f64; 4], out: &Path) -> Result<()> {
    let img = image::open(path).with_context(|| format!("opening {}", path.display()))?;
    let (iw, ih) = (f64::from(img.width()), f64::from(img.height()));
    let x = (bx[0] * iw).clamp(0.0, iw - 1.0) as u32;
    let y = (bx[1] * ih).clamp(0.0, ih - 1.0) as u32;
    let w = (bx[2] * iw).clamp(1.0, iw - f64::from(x)) as u32;
    let h = (bx[3] * ih).clamp(1.0, ih - f64::from(y)) as u32;
    img.crop_imm(x, y, w, h).save(out).with_context(|| format!("saving {}", out.display()))?;
    Ok(())
}

/// A short, stable signature of a SETTLED crop — proxy file + box + aspect (+ a manual
/// flag) — folded into `art_ref` so the render cache key (`card_hash`) changes the
/// instant the crop box moves. Without this, a repositioned box keeps the same hash and
/// a stale cached render is served; with it, every distinct crop yields its own render
/// and a plain `render all` (no --force) is always correct.
fn crop_sig(proxy: &Path, bx: &[f64; 4], aspect: f64, manual: bool) -> String {
    let name = proxy.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let s = format!(
        "{name}|{:.5},{:.5},{:.5},{:.5}|{aspect:.5}|{manual}",
        bx[0], bx[1], bx[2], bx[3]
    );
    short_hash(&s)
}

/// Per-pixel gradient magnitude (central differences) of a greyscale image, row-major.
/// Edges are 0. Used so template matching keys on structure, not flat brightness.
fn grad_mag(img: &image::GrayImage, w: u32, h: u32) -> Vec<f64> {
    let g = |x: u32, y: u32| f64::from(img.get_pixel(x, y)[0]);
    let mut out = vec![0.0; (w * h) as usize];
    if w < 3 || h < 3 {
        return out;
    }
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let gx = g(x + 1, y) - g(x - 1, y);
            let gy = g(x, y + 1) - g(x, y - 1);
            out[(y * w + x) as usize] = gx.hypot(gy);
        }
    }
    out
}

/// width/height of an image, for aspect math.
fn image_aspect(path: &Path) -> Option<f64> {
    let img = image::open(path).ok()?;
    let h = f64::from(img.height());
    (h > 0.0).then(|| f64::from(img.width()) / h)
}

/// Locate the reference art (Scryfall art_crop) inside a full-card proxy by
/// multi-scale **normalized cross-correlation** on greyscale — deterministic, no LLM.
/// Returns (box as fractions of the proxy, score in -1..1). NCC is invariant to the
/// brightness/contrast differences between a clean scan and a community proxy.
///
/// Search is bounded by how real cards are laid out: the art is horizontally near-
/// centred and spans ~0.68–0.96 of the card width, with its top in the upper third.
/// (Extended/borderless/full-art proxies are already filtered out before this runs;
/// for those the art bleeds past the frame and template scale/answers are unreliable.)
fn cv_art_box(ref_path: &Path, proxy_path: &Path) -> Option<([f64; 4], f64)> {
    use image::imageops::{resize, FilterType};
    let refimg = image::open(ref_path).ok()?.to_luma8();
    let proxy = image::open(proxy_path).ok()?.to_luma8();
    let tpl_aspect = f64::from(refimg.width()) / f64::from(refimg.height()).max(1.0);

    let pw: u32 = 256;
    let ph = (f64::from(pw) * f64::from(proxy.height()) / f64::from(proxy.width())).round() as u32;
    if ph < 16 {
        return None;
    }
    let proxy = resize(&proxy, pw, ph, FilterType::Triangle);
    // Match on EDGE (gradient) magnitude, not raw intensity: uniform regions (sky,
    // title bar, borders) have ~0 gradient, so a light art-top can't correlate with a
    // light title bar and shift the match up. Localises the art sharply.
    let pf = grad_mag(&proxy, pw, ph);

    let mut best: Option<([f64; 4], f64)> = None;
    for si in 0..=14 {
        let s = 0.68 + 0.02 * f64::from(si); // art width as a fraction of card width
        let tw = (s * f64::from(pw)).round() as u32;
        let th = (f64::from(tw) / tpl_aspect).round() as u32;
        if tw < 8 || th < 8 || tw >= pw || th >= ph {
            continue;
        }
        let tpl = resize(&refimg, tw, th, FilterType::Triangle);
        let tf = grad_mag(&tpl, tw, th);
        let n = f64::from(tw * th);
        let tmean = tf.iter().sum::<f64>() / n;
        let tvar = tf.iter().map(|v| (v - tmean).powi(2)).sum::<f64>() / n;
        if tvar < 1.0 {
            continue;
        }
        let tstd = tvar.sqrt();

        let xc = (pw - tw) / 2;
        let xr = pw / 12;
        let (xlo, xhi) = (xc.saturating_sub(xr), (xc + xr).min(pw - tw));
        // The art top is physically constrained: it sits below the title bar and never
        // above ~0.10 of the card, nor below ~0.24. Bounding the vertical search here
        // stops a tall template from scoring well while shifted UP into the title bar
        // (which baked the proxy's own title into the crop).
        let ylo = (0.10 * f64::from(ph)) as u32;
        let yhi = ((0.24 * f64::from(ph)) as u32).min(ph - th);

        let mut y = ylo;
        while y <= yhi {
            let mut x = xlo;
            while x <= xhi {
                // window mean/std + cross term
                let (mut sp, mut spp, mut spt) = (0.0, 0.0, 0.0);
                for ty in 0..th {
                    let row = (y + ty) * pw + x;
                    let trow = ty * tw;
                    for tx in 0..tw {
                        let pv = pf[(row + tx) as usize];
                        sp += pv;
                        spp += pv * pv;
                        spt += pv * tf[(trow + tx) as usize];
                    }
                }
                let pmean = sp / n;
                let pvar = (spp / n) - pmean * pmean;
                if pvar > 1.0 {
                    let cov = (spt / n) - pmean * tmean;
                    let ncc = cov / (pvar.sqrt() * tstd);
                    if best.is_none_or(|(_, b)| ncc > b) {
                        best = Some((
                            [
                                f64::from(x) / f64::from(pw),
                                f64::from(y) / f64::from(ph),
                                f64::from(tw) / f64::from(pw),
                                f64::from(th) / f64::from(ph),
                            ],
                            ncc,
                        ));
                    }
                }
                x += 2;
            }
            y += 2;
        }
    }
    best
}

fn sanitize_bucket(b: &str) -> String {
    b.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect()
}

/// Maintain projects/odyssey2026/MPCFILLBUCKETS.md (bucket → drive link).
fn record_bucket(bucket: &str, link: &str) -> Result<()> {
    let md = Path::new("projects/odyssey2026/MPCFILLBUCKETS.md");
    let header = "# MPCFill buckets\n\nSources art was pulled from. Image files live under \
        `render-cache/cards/<Card>/mpcfill/<bucket>/<id>.<ext>` (gitignored).\n\n\
        | Bucket | Drive / source |\n|---|---|\n";
    let existing = fs::read_to_string(md).unwrap_or_default();
    if existing.lines().any(|l| l.starts_with(&format!("| {bucket} |"))) {
        return Ok(());
    }
    if let Some(parent) = md.parent() {
        fs::create_dir_all(parent)?;
    }
    let base = if existing.is_empty() { header.to_string() } else { existing };
    let link = if link.is_empty() { "—" } else { link };
    fs::write(md, format!("{base}| {bucket} | {link} |\n"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// manual art crop (per-card editor: reposition the underlying MPCfill/Scryfall art)
// ---------------------------------------------------------------------------
//
// The auto crop (CV / Claude vision in mpcfill_pick) is right most of the time but
// "slightly off" for some cards (Narset, Jace, …). Rather than chase it with more
// heuristics, the editor lets the operator drag a window-aspect box over the underlying
// art and save it. That writes a `manual` sidecar (proxy = the chosen source image, box
// = the hand-placed rectangle); `acquire_art` then crops it EXACTLY on every render. The
// prior auto pick is stashed under `auto` so "reset to auto" is free (no re-pick).

/// Parse any JSON file, if present + valid.
fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Parse a card dir's `art.json` sidecar, if present.
fn read_sidecar(card_dir: &Path) -> Option<Value> {
    read_json(&card_dir.join("art.json"))
}

/// Look up a card's ORIGINAL name (the art/render addressing key) by editor-DB id.
fn card_name(db: &Connection, id: i64) -> Result<String> {
    db.query_row("SELECT name FROM cube_cards WHERE id=?1", params![id], |r| r.get(0))
        .with_context(|| format!("no card id {id} in editor DB"))
}

/// The current crop box `[x,y,w,h]` (fractions of the source image), if the sidecar has one.
fn sidecar_box(v: &Value) -> Option<[f64; 4]> {
    v["box"].as_array().filter(|a| a.len() == 4).map(|a| {
        let g = |i: usize| a[i].as_f64().unwrap_or(0.0);
        [g(0), g(1), g(2), g(3)]
    })
}

/// Effective print DPI of a cached proxy, MEASURED from its pixel dimensions — an MPC proxy
/// is a full card with bleed (2.72in × 3.70in), so DPI ≈ longest-edge px / 3.70in. Reads
/// only the image header (fast), rounded to the nearest 10 to match MPCfill's own numbers.
fn measure_proxy_dpi(path: &Path) -> Option<i64> {
    let (w, h) = image::image_dimensions(path).ok()?;
    let dpi = (f64::from(w.max(h)) / 3.70 / 10.0).round() * 10.0;
    (dpi > 0.0).then_some(dpi as i64)
}

/// Illustrators whose work we never want on a cube card (their art is auto-flagged in the
/// art picker so the operator picks an alternative printing or GenAI). Seeded with Harold
/// McNeill (a self-identified NSDAP supporter). Matched case-insensitively, trimmed.
pub const BLOCKED_ARTISTS: &[&str] = &["Harold McNeill", "Harold McNeil"];

/// Whether an illustrator credit is on the blocklist.
fn artist_blocked(artist: &str) -> bool {
    let a = artist.trim().to_lowercase();
    !a.is_empty() && BLOCKED_ARTISTS.iter().any(|b| b.trim().to_lowercase() == a)
}

/// Resolve a source `rel` token to an existing image path (no DB access). Tokens:
///   `@scryfall`        — the oldest paper printing's art_crop
///   `@set:<CODE>`      — a SPECIFIC printing's art_crop (the "alternative art" picker)
///   `@current`         — the sidecar's currently-cropped source
///   `@upload`          — a disk-uploaded image at `<card_dir>/upload.<ext>` (see `art_upload`)
///   `mpcfill/<b>/<f>`  — a cached MPCfill proxy (validated to stay under the card dir)
fn resolve_source(name: &str, cache_dir: &Path, card_dir: &Path, rel: &str) -> Result<PathBuf> {
    let art_dir = cache_dir.join("art");
    match rel {
        "@scryfall" => Ok(scryfall_oldest(name, &art_dir)?.0),
        // The newest printing's art_crop — for SPLIT cards this is the combined two-half
        // illustration the crop editor slices into a left + right window.
        "@latest" => Ok(scryfall_latest(name, &art_dir)?.0),
        "@current" => read_sidecar(card_dir)
            .and_then(|v| v["proxy"].as_str().map(PathBuf::from))
            .filter(|p| p.exists())
            .context("no current source image for this card"),
        r if r.starts_with("@set:") => {
            let set = r.trim_start_matches("@set:");
            Ok(scryfall_printing(name, set, &art_dir)?.0)
        }
        r if r.starts_with("mpcfill/") => {
            if r.contains("..") {
                bail!("invalid source path");
            }
            let p = card_dir.join(r);
            let mpc = card_dir.join("mpcfill");
            let base = fs::canonicalize(&mpc).unwrap_or(mpc);
            let cp = fs::canonicalize(&p).with_context(|| format!("no such source: {r}"))?;
            if !cp.starts_with(&base) {
                bail!("source path escapes the card dir");
            }
            Ok(p)
        }
        "@upload" => {
            // Disk-uploaded art lives at <card_dir>/upload.<ext> under a FIXED name (no
            // traversal, no user-supplied path component).
            for ext in ["png", "jpg", "jpeg", "webp"] {
                let p = card_dir.join(format!("upload.{ext}"));
                if p.exists() {
                    return Ok(p);
                }
            }
            bail!("no uploaded art for {name:?} (expected {}/upload.*)", card_dir.display())
        }
        _ => bail!("unknown source token: {rel}"),
    }
}

/// All paper printings of a card as `(set, artist, year)`, newest-set first deduped by
/// (set, artist). Reads the cached Scryfall prints metadata (ensured downloaded). Used to
/// offer "alternative art" per printing (e.g. Greed → 7ED / Peter Bollinger).
fn printings_list(name: &str, art_dir: &Path) -> Vec<(String, String, String)> {
    let _ = scryfall_oldest(name, art_dir); // ensure <name>.json (the prints search) is cached
    let Some(v) = read_json(&art_dir.join(format!("{}.json", sanitize(name)))) else {
        return Vec::new();
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for c in v["data"].as_array().into_iter().flatten() {
        let set = c["set"].as_str().unwrap_or("").to_uppercase();
        let artist = c["artist"].as_str().unwrap_or("").to_string();
        let year = c["released_at"].as_str().unwrap_or("").chars().take(4).collect::<String>();
        if set.is_empty() || !seen.insert(format!("{set}|{artist}")) {
            continue;
        }
        out.push((set, artist, year));
    }
    out
}

/// Re-materialise the cache sidecar from the DURABLE editor-DB art override (if any). This
/// runs at the TOP of every render so a hand-picked alternative art survives a cache wipe
/// or an auto re-pick — the DB is the source of truth, the sidecar is just its view, so a
/// chosen art can never be silently trampled back to the original.
fn materialize_override(db: &Connection, cache_dir: &Path, id: i64, name: &str, card_dir: &Path) -> Result<()> {
    let ov: String = db
        .query_row("SELECT COALESCE(art_override,'') FROM cube_cards WHERE id=?1", params![id], |r| r.get(0))
        .unwrap_or_default();
    if ov.trim().is_empty() {
        return Ok(());
    }
    let v: Value = serde_json::from_str(ov.trim()).unwrap_or_else(|_| json!({}));
    let (Some(rel), Some(bx)) = (v["rel"].as_str(), sidecar_box(&v)) else {
        return Ok(());
    };
    let source = match resolve_source(name, cache_dir, card_dir, rel) {
        Ok(p) if p.exists() => p,
        _ => {
            eprintln!("  art override for {name:?}: source {rel} unavailable — using auto art");
            return Ok(());
        }
    };
    fs::create_dir_all(card_dir)?;
    let side = json!({
        "art_ref": format!("override:{rel}"),
        "artist": v["artist"].as_str().unwrap_or(""),
        "year": v["year"].as_str().unwrap_or("2001"),
        "proxy": source.to_string_lossy(),
        "box": bx,
        "art_aspect": art_window_aspect(),
        "manual": true,
        "source_rel": rel,
        "override": true,
    });
    fs::write(card_dir.join("art.json"), side.to_string())?;
    Ok(())
}

/// Metadata for the per-card crop editor: the current box + source, every selectable
/// source image (Scryfall art_crop + each cached MPCfill proxy), and the art-window
/// aspect the box is locked to. `genai` cards can't be repositioned here (their art is
/// the generated image itself, not a sub-crop of a proxy) — the UI disables editing.
/// The per-card cache dir for the *active frame*. The modern (8th) pipeline keeps a card's
/// art, MPCfill proxies and crops under `cards8/<name>`; the classic frame uses `cards/<name>`.
/// The crop editor MUST read the same dir its render rips art from — otherwise the picker scans
/// the empty `cards/` dir on the 8th server and shows no MPCfill sources (only Scryfall prints).
fn frame_card_dir(cache_dir: &Path, name: &str, eighth: bool) -> PathBuf {
    cache_dir.join(if eighth { "cards8" } else { "cards" }).join(sanitize(name))
}

pub fn art_meta(editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool) -> Result<Value> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let name = card_name(&db, id)?;
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    let mpc_dir = card_dir.join("mpcfill");
    let art_dir = cache_dir.join("art");
    // SPLIT cards (" // " in the name): the crop editor slices the combined two-half art with
    // TWO rectangles. Make sure the combined art + its metadata are cached, then confirm the
    // layout really is split before switching the modal into two-rectangle mode.
    let is_split = name.contains(" // ") && {
        let _ = scryfall_latest(&name, &art_dir);
        split_faces(cache_dir, &name).is_some()
    };
    let side = read_sidecar(&card_dir);
    let art_ref = side.as_ref().and_then(|v| v["art_ref"].as_str()).unwrap_or("");
    let manual = side.as_ref().and_then(|v| v["manual"].as_bool()).unwrap_or(false);
    let is_genai = art_ref.starts_with("genai:") && !manual;
    let proxy = side.as_ref().and_then(|v| v["proxy"].as_str()).map(PathBuf::from);

    // The DURABLE override (editor DB) is authoritative if present — its rel is the current
    // source and its box the current crop, regardless of the (materialised) sidecar.
    let ov: Value = db
        .query_row("SELECT COALESCE(art_override,'') FROM cube_cards WHERE id=?1", params![id], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    let override_rel = ov["rel"].as_str().map(str::to_string);

    // `rel` token for a source path under the card dir (forward slashes for the URL).
    let rel_of = |p: &Path| -> Option<String> {
        p.strip_prefix(&card_dir).ok().map(|r| r.to_string_lossy().replace('\\', "/"))
    };
    let dpi_index = read_json(&mpc_dir.join("index.json")).unwrap_or_else(|| json!({}));

    // Sources, in pick order: Scryfall original, then every alternative PRINTING (with its
    // own artist — the "Greed → 7ED" case), then every cached MPCfill proxy (DPI-labelled).
    let mut sources: Vec<Value> = Vec::new();
    if is_split {
        // A split has no single "original art" — the combined newest art_crop is the source
        // both half-rectangles are sliced from.
        sources.push(json!({
            "rel": "@latest", "kind": "scryfall",
            "label": "Combined split art (newest printing)", "artist": "", "year": "",
            "blocked": false,
        }));
    } else {
        let (a, y) = scryfall_oldest(&name, &art_dir).map_or_else(
            |_| (String::new(), String::new()),
            |(_, a, y, _)| (a, y),
        );
        sources.push(json!({
            "rel": "@scryfall", "kind": "scryfall",
            "label": "Scryfall — original printing", "artist": a, "year": y,
            "blocked": artist_blocked(&a),
        }));
    }
    for (set, artist, year) in printings_list(&name, &art_dir) {
        sources.push(json!({
            "rel": format!("@set:{set}"), "kind": "printing",
            "label": format!("{set} — {artist}"), "set": set, "artist": artist, "year": year,
            "blocked": artist_blocked(&artist),
        }));
    }
    if let Ok(rd) = fs::read_dir(&mpc_dir) {
        let mut buckets: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        buckets.sort();
        for b in &buckets {
            let bucket = b.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if let Ok(files) = fs::read_dir(b) {
                let mut fl: Vec<PathBuf> = files.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
                fl.sort();
                for f in &fl {
                    if let Some(rel) = rel_of(f) {
                        // Prefer MPCfill's reported DPI (from the fetch index); else MEASURE
                        // it from the image dimensions — a proxy is a full card, so
                        // DPI ≈ long-edge px / 3.70in (MPC bleed height). No "? DPI".
                        let dpi = dpi_index[&rel]["dpi"].as_i64().or_else(|| measure_proxy_dpi(f));
                        sources.push(json!({
                            "rel": rel, "kind": "mpcfill", "label": bucket,
                            "dpi": dpi, "low_dpi": dpi.map(|d| d < 600),
                        }));
                    }
                }
            }
        }
    }

    // Current source: the durable override wins; else infer from the sidecar's proxy path.
    let current_rel = override_rel.clone().or_else(|| {
        if is_split {
            return Some("@latest".to_string());
        }
        match &proxy {
            Some(p) if p.starts_with(&mpc_dir) => rel_of(p),
            Some(p) if p.starts_with(&art_dir) => Some("@scryfall".to_string()),
            Some(_) => Some("@current".to_string()),
            None => None,
        }
    });
    let current_box = sidecar_box(&ov).or_else(|| sidecar_box(side.as_ref().unwrap_or(&json!({}))));
    // Split's RIGHT-half box (the editor's second rectangle), from the override then sidecar.
    let box2 = box_field(&ov, "box2").or_else(|| side.as_ref().and_then(|v| box_field(v, "box2")));
    // The two half names label the crop rectangles + their live previews ("Bind" / "Liberate").
    let split_names: Vec<String> = if is_split {
        split_faces(cache_dir, &name)
            .map(|h| vec![h[0].0.clone(), h[1].0.clone()])
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let artist = ov["artist"].as_str()
        .or_else(|| side.as_ref().and_then(|v| v["artist"].as_str()))
        .unwrap_or("");

    Ok(json!({
        "id": id,
        "name": name,
        // A split locks each rectangle to ONE half's art-window aspect, not the full window.
        "window_aspect": if is_split { split_window_aspect() } else { art_window_aspect() },
        "split": is_split,
        "split_names": split_names,
        "box": current_box,
        "box2": box2,
        "art_aspect": side.as_ref().and_then(|v| v["art_aspect"].as_f64()),
        "manual": manual || override_rel.is_some(),
        "override": override_rel.is_some(),
        "min_dpi": 600,
        "genai": is_genai,
        "current_rel": current_rel,
        "artist": artist,
        "artist_blocked": artist_blocked(artist),
        "sources": sources,
        "has_sidecar": side.is_some(),
    }))
}

/// Resolve a source `rel` token to a real, existing image path (validated to stay
/// inside the card's cache dir). `@scryfall` downloads the art_crop on demand;
/// `@current` is the sidecar's current source; `mpcfill/<bucket>/<file>` is a cached proxy.
pub fn art_image_path(editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool, rel: &str) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let name = card_name(&db, id)?;
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    resolve_source(&name, cache_dir, &card_dir, rel)
}

/// The credit `(artist, year)` for a chosen source: a specific printing carries its OWN
/// illustrator (e.g. Greed → 7ED → Peter Bollinger), `@scryfall` the oldest printing's,
/// and a proxy keeps the prior credit (the proxy file doesn't say who painted it).
fn source_credit(name: &str, cache_dir: &Path, rel: &str, prev: Option<&Value>) -> (String, String) {
    let art_dir = cache_dir.join("art");
    let oldest = || {
        scryfall_oldest(name, &art_dir)
            .map_or_else(|_| (String::new(), "2001".to_string()), |(_, a, y, _)| (a, y))
    };
    match rel {
        r if r.starts_with("@set:") => scryfall_printing(name, r.trim_start_matches("@set:"), &art_dir)
            .map_or_else(|_| oldest(), |(_, a, y)| (a, y)),
        "@scryfall" => oldest(),
        _ => match (
            prev.and_then(|v| v["artist"].as_str()).map(str::to_string),
            prev.and_then(|v| v["year"].as_str()).map(str::to_string),
        ) {
            (Some(a), Some(y)) => (a, y),
            _ => oldest(),
        },
    }
}

/// Largest centred crop box (fractions of the source) at the on-card art-window aspect, so
/// the cropped region maps 1:1 into the art hole (no further `cover` cropping).
fn window_fit_box(src_w: u32, src_h: u32) -> [f64; 4] {
    let aspect = art_window_aspect();
    let src_aspect = f64::from(src_w) / f64::from(src_h);
    if src_aspect > aspect {
        let w = aspect / src_aspect; // source is wider: trim the sides
        [(1.0 - w) / 2.0, 0.0, w, 1.0]
    } else {
        let h = src_aspect / aspect; // source is taller: trim top/bottom
        [0.0, (1.0 - h) / 2.0, 1.0, h]
    }
}

/// On-card art-window size in inches at the 800-DPI face (FACE_W=2000px=2.5in).
fn art_window_inches() -> (f64, f64) {
    (
        ART_WINDOW_FRAC[2] * f64::from(FACE_W) / 800.0,
        ART_WINDOW_FRAC[3] * f64::from(FACE_H) / 800.0,
    )
}

/// Effective print DPI of a source image as it will sit in the on-card art window, given
/// the crop box (fractions of the source): the cropped pixels are scaled to fill the
/// ~1.92×1.55in window, so the binding (worst) resolution is the smaller of the two axes.
fn effective_art_dpi(src_w: u32, src_h: u32, bx: [f64; 4]) -> i64 {
    let (win_w_in, win_h_in) = art_window_inches();
    let dpi_w = (bx[2] * f64::from(src_w)) / win_w_in;
    let dpi_h = (bx[3] * f64::from(src_h)) / win_h_in;
    dpi_w.min(dpi_h).round() as i64
}

/// Best-effort artist credit inferred from an upload's file name: the stem with `_`/`-`
/// turned to spaces (e.g. `Tyler_Miles_Lockett.jpg` → "Tyler Miles Lockett"). Pass an
/// explicit artist when the file name also carries a subject prefix.
fn infer_artist(src: &Path) -> String {
    src.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.split(['_', '-']).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

/// Replace a card's art with a LOCAL image file. Copies the file into the card's cache dir
/// (`upload.<ext>`, resolved by the `@upload` source token), crops it to the on-card art
/// window, records a DURABLE art override + printed `illustrator` credit in the editor DB
/// (so it can never be auto-trampled — `materialize_override` re-feeds it every render), and
/// measures the effective print DPI, returning a warning (or, with `strict`, an error) when
/// it is below `dpi_threshold`. Artist defaults to a guess from the file name; the printed
/// "Illus." credit defaults to the artist. Returns a JSON report.
#[allow(clippy::too_many_arguments)]
pub fn art_upload(
    editor_db: &Path,
    cache_dir: &Path,
    id: i64,
    eighth: bool,
    src: &Path,
    artist: Option<&str>,
    year: Option<&str>,
    illustrator: Option<&str>,
    dpi_threshold: i64,
    strict: bool,
) -> Result<Value> {
    if !src.exists() {
        bail!("art file not found: {}", src.display());
    }
    let (sw, sh) = image::image_dimensions(src)
        .with_context(|| format!("decoding {} (is it a valid image?)", src.display()))?;

    let name = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        card_name(&db, id)?
    };
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    fs::create_dir_all(&card_dir)?;

    // Store under a fixed name the `@upload` token resolves to. Keep a loadable extension;
    // re-encode anything exotic to PNG. Clear any prior upload first (avoid stale matches).
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let keep = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp");
    for e in ["png", "jpg", "jpeg", "webp"] {
        let _ = fs::remove_file(card_dir.join(format!("upload.{e}")));
    }
    let stored = card_dir.join(format!("upload.{}", if keep { ext.as_str() } else { "png" }));
    if keep {
        fs::copy(src, &stored).with_context(|| format!("copying art → {}", stored.display()))?;
    } else {
        image::open(src)?
            .save(&stored)
            .with_context(|| format!("re-encoding art → {}", stored.display()))?;
    }

    let bx = window_fit_box(sw, sh);
    let dpi = effective_art_dpi(sw, sh, bx);
    let low = dpi < dpi_threshold;
    let (win_w, win_h) = art_window_inches();
    let warning = low.then(|| {
        format!(
            "{name:?} art is ~{dpi} DPI in the {win_w:.2}×{win_h:.2}in card art window \
             ({sw}×{sh}px source) — below the {dpi_threshold} DPI threshold; it will look soft at print size"
        )
    });
    if let Some(w) = &warning {
        eprintln!("  ⚠ {w}");
        if strict {
            bail!("{w} (use a higher-resolution image, lower --dpi-threshold, or drop --strict)");
        }
    }

    let artist = artist
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| infer_artist(src));
    let year = year.unwrap_or("2001").to_string();
    let illus = illustrator
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| artist.clone());

    // Sidecar (cache view): a MANUAL window-aspect crop of the upload. Mirrors art_save_crop
    // so the editor preview shows immediately, and acquire_art re-crops in place each render.
    let side = json!({
        "art_ref": "upload:@upload",
        "artist": artist, "year": year,
        "proxy": stored.to_string_lossy(),
        "box": bx, "art_aspect": art_window_aspect(),
        "manual": true, "source_rel": "@upload", "upload": true,
    });
    fs::write(card_dir.join("art.json"), side.to_string())?;
    crop_exact(&stored, bx, &card_dir.join("art.png"))?;

    // DURABLE: the editor DB is the source of truth (anti-trample). `illustrator` is the
    // printed "Illus." credit (preferred over the art's own artist at render time).
    let override_json =
        json!({"rel": "@upload", "box": bx, "artist": artist, "year": year}).to_string();
    let w = Connection::open(editor_db)?;
    w.execute(
        "UPDATE cube_cards SET art_override=?2, illustrator=?3, updated_at=datetime('now') WHERE id=?1",
        params![id, override_json, illus],
    )?;

    Ok(json!({
        "ok": true, "id": id, "name": name,
        "source": src.to_string_lossy(), "stored": stored.to_string_lossy(),
        "pixels": [sw, sh], "box": bx,
        "artist": artist, "year": year, "illustrator": illus,
        "dpi": dpi, "dpi_threshold": dpi_threshold, "low_dpi": low,
        "warning": warning,
    }))
}

/// Save a MANUAL crop: write a `manual` sidecar (source + hand-placed box) and re-crop
/// `art.png` immediately so the change shows even without a fresh render. The crop box is
/// in fractions of the source image and is expected to already carry the art-window
/// aspect (the editor locks it). Bumps `updated_at` so the editor preview cache-busts.
pub fn art_save_crop(
    editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool, rel: &str, bx: [f64; 4], bx2: Option<[f64; 4]>,
) -> Result<()> {
    let name = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        card_name(&db, id)?
    };
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    fs::create_dir_all(&card_dir)?;
    let source = art_image_path(editor_db, cache_dir, id, eighth, rel)?;
    if !source.exists() {
        bail!("source image missing: {}", source.display());
    }

    let prev = read_sidecar(&card_dir);
    // Credit follows the CHOSEN source (a printing carries its own illustrator/year).
    let (artist, year) = source_credit(&name, cache_dir, rel, prev.as_ref());
    // Stash the prior AUTO pick so "reset to auto" is free (no re-pick / CV / Claude).
    let auto = prev.as_ref().and_then(|v| {
        if v["manual"].as_bool().unwrap_or(false) {
            v.get("auto").cloned() // already manual: carry forward the original auto
        } else if v.get("box").is_some() && v.get("proxy").is_some() {
            Some(json!({
                "art_ref": v["art_ref"], "box": v["box"],
                "proxy": v["proxy"], "art_aspect": v["art_aspect"],
            }))
        } else {
            None
        }
    });

    let mut side = json!({
        "art_ref": format!("manual:{rel}"),
        "artist": artist,
        "year": year,
        "proxy": source.to_string_lossy(),
        "box": bx,
        "art_aspect": art_window_aspect(),
        "manual": true,
        "source_rel": rel,
    });
    if let Some(a) = auto {
        side["auto"] = a;
    }
    // A SPLIT card carries a SECOND box — the right half's crop of the combined art. (The
    // render slices both halves from the source at render time; the single art.png below is
    // just the left/primary crop, harmless for the rotated split layout.)
    if let Some(b2) = bx2 {
        side["box2"] = json!(b2);
    }
    fs::write(card_dir.join("art.json"), side.to_string())?;
    crop_exact(&source, bx, &card_dir.join("art.png"))?;

    // DURABLE: record the choice in the editor DB (the source of truth). The sidecar lives
    // in the gitignored cache and can be wiped/re-picked; the DB override can't be trampled
    // — every render re-materialises the sidecar from it (see materialize_override).
    let mut override_val = json!({"rel": rel, "box": bx, "artist": artist, "year": year});
    if let Some(b2) = bx2 {
        override_val["box2"] = json!(b2);
    }
    let override_json = override_val.to_string();
    let w = Connection::open(editor_db)?;
    let _ = w.execute(
        "UPDATE cube_cards SET art_override=?2, updated_at=datetime('now') WHERE id=?1",
        params![id, override_json],
    );
    Ok(())
}

/// Reset a card's crop to AUTOMATIC: clear the durable DB override, then restore the
/// stashed auto pick if present (free re-crop) or drop the sidecar so the next render
/// re-picks from scratch. Bumps `updated_at` so the editor preview refreshes.
pub fn art_reset(editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool) -> Result<()> {
    let name = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        card_name(&db, id)?
    };
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    let prev = read_sidecar(&card_dir);
    if let Some(auto) = prev.as_ref().and_then(|v| v.get("auto")).cloned() {
        let artist = prev.as_ref().and_then(|v| v["artist"].as_str()).unwrap_or("");
        let year = prev.as_ref().and_then(|v| v["year"].as_str()).unwrap_or("2001");
        let side = json!({
            "art_ref": auto["art_ref"], "artist": artist, "year": year,
            "proxy": auto["proxy"], "box": auto["box"], "art_aspect": auto["art_aspect"],
        });
        fs::write(card_dir.join("art.json"), side.to_string())?;
        if let (Some(p), Some(b), Some(a)) = (
            auto["proxy"].as_str().map(PathBuf::from),
            sidecar_box(&auto),
            auto["art_aspect"].as_f64(),
        ) {
            if p.exists() {
                crop_to(&p, b, &card_dir.join("art.png"), a)?;
            }
        }
    } else {
        let _ = fs::remove_file(card_dir.join("art.json"));
        let _ = fs::remove_file(card_dir.join("art.png"));
    }
    let w = Connection::open(editor_db)?;
    let _ = w.execute(
        "UPDATE cube_cards SET art_override='', updated_at=datetime('now') WHERE id=?1",
        params![id],
    );
    Ok(())
}

/// "Fetch all" for the picker: download every MPCfill candidate (>=600 DPI, low-floor
/// fallback) and cache every alternative printing's art_crop, so the sidebar shows real
/// thumbnails to choose from. Returns the refreshed `art_meta`. Proxies need the MPCfill
/// backend URL; printings come from Scryfall and work even without it.
pub fn art_fetch_all(editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool, backend: Option<&str>) -> Result<Value> {
    let name = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        card_name(&db, id)?
    };
    let card_dir = frame_card_dir(cache_dir, &name, eighth);
    let art_dir = cache_dir.join("art");
    // Pre-cache every printing's art_crop (cheap; powers the alternative-printing thumbnails).
    for (set, _, _) in printings_list(&name, &art_dir) {
        let _ = scryfall_printing(&name, &set, &art_dir);
    }
    // Download all MPCfill proxies, if a backend is configured.
    match backend {
        Some(base) => {
            if let Err(e) = mpcfill_search_download(&name, base, cache_dir, &card_dir, 600) {
                eprintln!("  fetch-all {name:?}: MPCfill error: {e}");
            }
        }
        None => eprintln!("  fetch-all {name:?}: no --art-backend; printings only"),
    }
    art_meta(editor_db, cache_dir, id, eighth)
}

/// A small cached JPEG thumbnail (~420 px long edge) of a source image, for the picker
/// sidebar — serving the full multi-MB proxies as thumbnails would be far too heavy.
pub fn art_thumb(editor_db: &Path, cache_dir: &Path, id: i64, eighth: bool, rel: &str) -> Result<PathBuf> {
    let src = art_image_path(editor_db, cache_dir, id, eighth, rel)?;
    let name = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        card_name(&db, id)?
    };
    let thumb_dir = frame_card_dir(cache_dir, &name, eighth).join(".thumbs");
    fs::create_dir_all(&thumb_dir)?;
    let out = thumb_dir.join(format!("{}.jpg", short_hash(rel)));
    let fresh = match (
        fs::metadata(&out).and_then(|m| m.modified()),
        fs::metadata(&src).and_then(|m| m.modified()),
    ) {
        (Ok(o), Ok(s)) => s <= o,
        _ => false,
    };
    if !out.exists() || !fresh {
        let img = image::open(&src).with_context(|| format!("opening {}", src.display()))?;
        img.thumbnail(420, 420)
            .to_rgb8()
            .save_with_format(&out, image::ImageFormat::Jpeg)
            .with_context(|| format!("writing thumb {}", out.display()))?;
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// GenAI art (Claude art-direction -> OpenAI gpt-image-1, in chosen-artist styles)
// ---------------------------------------------------------------------------

/// Late-90s / early-00s Magic painters whose styles we offer per card.
/// (real name, PRECISE signature, printed pseudonym). The signature names exactly
/// what the artist is famous for — unmistakably *them*, never generic fantasy; it is
/// also the no-name fallback when OpenAI's safety system rejects "in the style of
/// <living artist>", so it must NOT contain the artist's name.
///
/// The REAL name is used only for our own bookkeeping (the `genai/<RealName>/` folders
/// and the event log — a mark of respect). The PSEUDONYM — a tasteful play on words,
/// never cringe, never mentioning AI — is what gets printed as the "Illus." credit and
/// stored in the DB, so a machine-painted card never claims to be the master's own hand.
pub const ARTISTS: &[(&str, &str, &str)] = &[
    ("John Avon",
     "luminous airbrushed skies and sweeping panoramic vistas; vast aerial scale, \
      jewel-toned atmospheric gradients, dreamlike soft light, tiny figures dwarfed by \
      enormous serene landscapes — the master of the horizon and the glowing sky",
     "John Thames"),
    ("Christopher Rush",
     "bold, iconic earliest-Magic fantasy, painted with brush not ink: form is defined by \
      VALUE and COLOUR CONTRAST and confident painted edges — NOT by black outlines. Do NOT \
      put thick black ink lines, comic-style inking or a dark stroke around figures and \
      objects; almost no outlining at all, just paint. Flat-ish saturated primaries, heraldic \
      graphic clarity and a slightly naive storybook directness — the Alpha look (he painted \
      the Black Lotus). The subject is a HIGH-CONTRAST foreground hero, sharply lit (contrast \
      from light vs shadow, not from outlines); the background is ALWAYS a plain, simple, \
      almost flat gradient fill — NEVER a detailed or busy background, never a rendered scene \
      behind the subject",
     "Chris Pace"),
    ("Brom",
     "dark gothic oil-painting STYLE ONLY — this is a way of PAINTING, not a fixed subject. \
      Rich low-key chiaroscuro, a single cold raking light out of deep shadow, a murky \
      earth-and-soot palette shot with sickly green and dull blood-red, sinewy texture on \
      leathery, weathered surfaces, a brooding fine-art horror atmosphere. CRITICAL: paint \
      THE CARD'S OWN SUBJECT (whatever the art direction describes) in this style — do NOT \
      default to a gaunt menacing humanoid; the recurring-dark-figure habit is banned. The \
      subject must match this specific card, only the rendering is Brom's",
     "Brian Ohm"),
    ("Rob Alexander",
     "atmospheric naturalistic gouache STYLE ONLY — this is a way of PAINTING, not a fixed \
      subject. Deep aerial/atmospheric perspective, soft diffused daylight, layered misty \
      distance, masterful luminous skies, a meticulous painterly touch. CRITICAL: paint THE \
      CARD'S OWN SUBJECT (whatever the art direction describes) in this style — do NOT default \
      to a landscape, vista, ruins or building; if the card is about a creature, figure or \
      object, THAT is the subject, simply rendered with his atmospheric naturalism. The \
      landscape-with-a-building habit is banned",
     "Rob Macedon"),
    ("Greg Staples",
     "muscular British-comic dynamism (2000 AD / Judge Dredd); heavy theatrical \
      chiaroscuro, gritty kinetic figures mid-action, bold confident paint strokes and \
      high-contrast spotlit drama",
     "Greg Mainstay"),
    ("Donato Giancola",
     "classical museum-grade realism; photoreal oil rendering, Renaissance/Caravaggio \
      chiaroscuro, dignified noble figures, rich warm glazed light and fine-art gravitas — \
      it should look like an Old Master canvas",
     "Renato Colonna"),
    ("Wayne Reynolds",
     "kinetic ink-heavy action illustration; razor-sharp angular linework, extreme \
      dramatic foreshortening and motion, snarling characters in elaborately detailed \
      armour and gear, comic-book punch and energy",
     "Wane Danger"),
    ("Rebecca Guay",
     "art-nouveau Mucha-esque watercolour and gouache; flowing organic linework, elongated \
      graceful figures, soft luminous translucent washes, botanical detail and dreamlike \
      romantic tenderness. CRITICAL: paint ONLY the actual subject(s) of the scene — \
      absolutely NO decorative frames, borders, filigree, gold trim, panels, cartouches, \
      art-deco/art-nouveau ornament or patterned edging anywhere in the finished image; \
      the painting fills the whole image with the subject, nothing framing it",
     "Reverie Quay"),
];

/// The printed pseudonym for a real artist (falls back to the real name if unknown —
/// only ever called with an ARTISTS entry, so the fallback is just defensive).
fn pseudonym(real: &str) -> &str {
    ARTISTS.iter().find(|(a, _, _)| *a == real).map_or(real, |(_, _, p)| *p)
}

fn short_hash(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())[..16].to_string()
}

/// Generic Claude text call.
fn claude_text(key: &str, prompt: &str) -> Result<String> {
    let body = json!({
        "model": "claude-opus-4-8", "max_tokens": 350,
        "messages": [{"role":"user","content": prompt}]
    });
    let body_file = std::env::temp_dir().join("mtgbrain-claude-text.json");
    fs::write(&body_file, body.to_string())?;
    let out = Command::new("curl")
        .args(["-sS", "--max-time", "90", "https://api.anthropic.com/v1/messages",
            "-H", &format!("x-api-key: {key}"), "-H", "anthropic-version: 2023-06-01",
            "-H", "content-type: application/json", "--data"])
        .arg(format!("@{}", body_file.display()))
        .output()
        .context("curl claude")?;
    let resp: Value = serde_json::from_slice(&out.stdout).context("claude response")?;
    resp["content"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .with_context(|| format!("claude returned no text: {}", String::from_utf8_lossy(&out.stdout)))
}

/// The original Scryfall flavor for a card (for the editor's flavor field to prefill).
/// Reads the cached print metadata; empty if the card hasn't been fetched/rendered yet.
pub fn base_flavor(name: &str, cache_dir: &Path) -> String {
    card_flavor(name, &cache_dir.join("art"))
}

/// Flavor text of the oldest printing (from the cached Scryfall metadata).
fn card_flavor(name: &str, art_dir: &Path) -> String {
    let meta = art_dir.join(format!("{}.json", sanitize(name)));
    let Ok(txt) = fs::read_to_string(&meta) else { return String::new() };
    let Ok(v) = serde_json::from_str::<Value>(&txt) else { return String::new() };
    v["data"][0]["flavor_text"]
        .as_str()
        .or_else(|| v["data"][0]["card_faces"][0]["flavor_text"].as_str())
        .unwrap_or("")
        .to_string()
}

/// (set code, rarity) of the oldest printing — the print whose art we use, so the
/// set symbol matches the illustration.
fn card_set_rarity(name: &str, art_dir: &Path) -> (String, String) {
    let meta = art_dir.join(format!("{}.json", sanitize(name)));
    let Ok(txt) = fs::read_to_string(&meta) else { return (String::new(), String::new()) };
    let Ok(v) = serde_json::from_str::<Value>(&txt) else { return (String::new(), String::new()) };
    let d = &v["data"][0];
    (
        d["set"].as_str().unwrap_or("").to_string(),
        d["rarity"].as_str().unwrap_or("").to_string(),
    )
}

/// Ensure the OLDEST-first prints metadata (`<name>.json`) is cached — the file
/// `card_set_rarity` and `card_flavor` read (data[0] = earliest printing). The 8ED path
/// fetches only `__latest.json` (the newest high-DPI art), so without this its set-symbol
/// and flavour sources would be missing. Metadata only — no art download. Same exact-name
/// search as `scryfall_oldest`; a 404 (split / DFC names) just leaves the file absent.
fn ensure_prints_meta(name: &str, art_dir: &Path) {
    let meta = art_dir.join(format!("{}.json", sanitize(name)));
    if meta.metadata().map(|m| m.len() >= 64).unwrap_or(false) {
        return;
    }
    let _ = fs::create_dir_all(art_dir);
    let url = format!(
        "https://api.scryfall.com/cards/search?order=released&dir=asc&unique=prints&q={}",
        percent(&format!("!\"{name}\" game:paper"))
    );
    let _ = curl_to_file(&url, &meta);
}

/// Cache + return the Scryfall set-symbol SVG — a clean MONOCOLOUR silhouette.
/// We colour it ourselves (solid rarity fill + thin perimeter bevel + thin white
/// keyline), which is how real old-frame symbols are actually built — NOT the full
/// metallic gradient hexproof bakes in.
fn set_symbol_svg(set: &str, cache_dir: &Path) -> Option<PathBuf> {
    if set.is_empty() {
        return None;
    }
    let dir = cache_dir.join("sets");
    let _ = fs::create_dir_all(&dir);
    let dst = dir.join(format!("{set}.svg"));
    let big_enough = |p: &Path| fs::metadata(p).map(|m| m.len() >= 64).unwrap_or(false);
    if !big_enough(&dst) {
        let _ = curl_to_file(&format!("https://svgs.scryfall.io/sets/{set}.svg"), &dst);
    }
    big_enough(&dst).then_some(dst)
}

/// Per-rarity MONOCOLOUR body fill for a set symbol (solid — the dark inner border
/// that hugs the contour is a separate masked layer underneath, see build_html).
/// Rasterise a set-symbol SVG, trim to the tight INK bounding box, and return
/// `(png_bytes, ink_aspect)`. Scryfall set SVGs pad the artwork inside a SQUARE
/// 1024×1024 viewBox, so the viewBox aspect lies (a wide broom reads as 1.0). We
/// measure the real ink: render to a bitmap, find the non-transparent bbox, crop to
/// it. The padding-free PNG then fills the placement box and the aspect is honest,
/// so the WIDE/SQUARE two-mode logic works for Nemesis, M10, M11, etc.
fn set_symbol_ink(svg_bytes: &[u8]) -> Option<(Vec<u8>, f64)> {
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &resvg::usvg::Options::default()).ok()?;
    let size = tree.size();
    let scale = 512.0 / size.width().max(size.height());
    let pw = (size.width() * scale).ceil() as u32;
    let ph = (size.height() * scale).ceil() as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pw, ph)?;
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let data = pixmap.data();
    let (mut minx, mut miny, mut maxx, mut maxy) = (pw, ph, 0u32, 0u32);
    for y in 0..ph {
        for x in 0..pw {
            if data[((y * pw + x) * 4 + 3) as usize] > 12 {
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
            }
        }
    }
    if maxx < minx || maxy < miny {
        return None;
    }
    let (bw, bh) = (maxx - minx + 1, maxy - miny + 1);
    let rect = resvg::tiny_skia::IntRect::from_xywh(minx as i32, miny as i32, bw, bh)?;
    let png = pixmap.clone_rect(rect)?.encode_png().ok()?;
    Some((png, f64::from(bw) / f64::from(bh)))
}

/// Width/height aspect of an SVG, read from its `viewBox` (falling back to width/height
/// attributes, then 1.0). Drives the set symbol's two placement modes.
#[allow(dead_code)]
fn svg_aspect(bytes: &[u8]) -> f64 {
    let s = String::from_utf8_lossy(bytes);
    let attr_quoted = |key: &str| -> Option<String> {
        let i = s.find(key)?;
        let rest = &s[i + key.len()..];
        let q = rest.find(['"', '\''])?;
        let after = &rest[q + 1..];
        let e = after.find(['"', '\''])?;
        Some(after[..e].to_string())
    };
    if let Some(vb) = attr_quoted("viewBox") {
        let nums: Vec<f64> = vb
            .split([' ', ','])
            .filter(|t| !t.is_empty())
            .filter_map(|t| t.parse().ok())
            .collect();
        if nums.len() == 4 && nums[2] > 0.0 && nums[3] > 0.0 {
            return nums[2] / nums[3];
        }
    }
    let dim = |k: &str| attr_quoted(k).and_then(|v| v.trim_end_matches("px").parse::<f64>().ok());
    if let (Some(w), Some(h)) = (dim("width="), dim("height=")) {
        if h > 0.0 {
            return w / h;
        }
    }
    1.0
}

/// A clean square swatch of a frame's marble texture (the title strip — pure colour,
/// no baked text), resized to `diam`×`diam`.
fn frame_swatch(frame_path: &Path, diam: u32) -> Option<image::RgbaImage> {
    use image::imageops::{crop_imm, resize, FilterType};
    let img = image::open(frame_path).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let fx = |f: f32| (f * w as f32) as u32;
    let fy = |f: f32| (f * h as f32) as u32;
    let (x0, y0, x1, y1) = (fx(0.22), fy(0.040), fx(0.78), fy(0.075));
    let patch =
        crop_imm(&img, x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1)).to_image();
    Some(resize(&patch, diam, diam, FilterType::Triangle))
}

/// A circular colour-indicator PNG for coloured artifacts, built from one or more
/// frame textures. One frame → a solid circle of that colour's marble. Two frames →
/// the circle split VERTICALLY 50:50 (left half = frames[0], right half = frames[1]).
/// (Callers pass the gold frame alone for 3+ colours.) The set-symbol decoration
/// (white keyline + inner border) is added by the SVG that embeds this PNG.
fn color_indicator_png(frames: &[&Path], diam: u32) -> Option<Vec<u8>> {
    let mut sq = frame_swatch(frames.first()?, diam)?;
    if let Some(second) = frames.get(1) {
        let right = frame_swatch(second, diam)?;
        for y in 0..diam {
            for x in (diam / 2)..diam {
                *sq.get_pixel_mut(x, y) = *right.get_pixel(x, y);
            }
        }
    }
    let r = diam as f32 / 2.0;
    for y in 0..diam {
        for x in 0..diam {
            let (dx, dy) = (x as f32 + 0.5 - r, y as f32 + 0.5 - r);
            let d = (dx * dx + dy * dy).sqrt();
            let a = if d <= r - 1.0 {
                255.0
            } else if d >= r {
                0.0
            } else {
                (r - d) * 255.0
            };
            sq.get_pixel_mut(x, y)[3] = a as u8;
        }
    }
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(sq)
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .ok()?;
    Some(buf)
}

fn rarity_fill(rarity: &str) -> &'static str {
    match rarity {
        "uncommon" => "#b3bbbf", // silver
        "rare" => "#c9a23c",     // gold
        "mythic" => "#d4501b",   // mythic orange-red
        _ => "#0c0c0c",          // common / land / token: black
    }
}

/// Ask Claude for the ART_DIRECTION_CARD_DESCRIPTION, reading the card's name,
/// rules AND flavor so the scene reflects what the card is *about* (e.g. Beloved
/// Chaplain is loved by beasts, not repelling them), not just the mechanics.
fn make_direction(card: &Card, flavor: &str, key: &str) -> Result<String> {
    let flavor_line = if flavor.is_empty() {
        "Flavor: (none)".to_string()
    } else {
        format!("Flavor text: \"{flavor}\"")
    };
    let prompt = format!(
        "You are a Magic: the Gathering ART DIRECTOR writing an art description to hand to an \
         illustrator, in Wizards of the Coast's exact house format.\n\n\
         Card.\nName: {}\nType: {}\nRules: {}\n{}\n\n\
         FIRST (internally) work out what the card is THEMATICALLY about by reading its NAME and \
         FLAVOR TEXT (and the feeling of its rules) — the picture must convey the card's MEANING, \
         not the literal rules. (e.g. a card called 'Beloved' whose flavor says beasts are charmed \
         by her shows animals lovingly drawn to her, not menacing her.)\n\n\
         NEVER depict the CARD-GAME mechanic itself. In particular, NEVER show drawing cards, \
         'looting' (draw-and-discard), rummaging, discarding, card selection/scry, or any hands, \
         stacks, decks, scrolls-as-cards or libraries of playing cards — and never a card frame \
         inside the art. A creature is shown as that CREATURE acting in the world per its name, \
         type and flavor — never performing a card draw. (e.g. Glint-Horn Buccaneer is a fierce \
         horned minotaur raider mid-battle, NOT someone reading, drawing or shuffling cards.)\n\n\
         HARD BAN on Earth-Christian / Catholic and generic-angelic imagery: do NOT include \
         angels, wings, halos, glowing divine auras, cathedrals, churches, chapels, crosses, \
         crucifixes, bishops, popes, nuns, monks, friars, choir robes, censers or stained glass — \
         NONE of it — UNLESS the card's name, type or rules text EXPLICITLY calls for it (e.g. the \
         type line literally says 'Angel'). Otaria is NOT medieval Europe and NOT a church setting. \
         Never write 'winged' for a non-Angel. Do NOT default human figures to priests or clergy; \
         default them instead to the ACTUAL peoples of the Odyssey block listed below — \
         especially NOMADS, BARBARIANS and CENTAURS.\n\n\
         WRITE LIKE A PROFESSIONAL, TASTEFUL ART DIRECTOR, NOT A VFX ARTIST. Describe a grounded, \
         real-painting scene. NEVER ask for: glowing auras, glowing eyes, wisps or tendrils of \
         magic/smoke/shadow, ethereal mist-energy, floating particles/sparkles, literal sound-waves \
         or sound-rings or shockwave rings or 'rings of distorted air', concentric energy ripples, \
         radiating lines, neon/digital glow, or any CGI/video-game effect. Convey action, sound and \
         magic through POSTURE, EXPRESSION, gesture, body language, light and shadow and composition \
         — a shrieking creature is shown shrieking by its gaping jaws and strained body, NOT by \
         drawn sound-rings. Keep it understated and painterly.\n\n\
         GROUND EVERY SCENE IN THE ODYSSEY BLOCK (the sets Odyssey / Torment / Judgment), set on \
         the continent of OTARIA on Dominaria. The picture MUST read as belonging to this world, \
         never as generic fantasy and never as another Magic plane. NEVER name or reference any \
         OTHER plane or its factions — no Ravnica or its guilds (Azorius, Boros, Dimir, etc.), no \
         Innistrad, Zendikar, Theros, Kamigawa, Mirrodin/Phyrexia, no planeswalkers, no \
         guild/faction names from anywhere but Otaria, and no real-world Earth places or religions. \
         Use ONLY the Odyssey-block places, peoples, factions and themes below:\n\
         - The CABAL: sinister black-aligned cult behind the pit-fighting 'Games', dementia magic \
         and necromancy; Cabal City and the rotting Aphetto swamps; led by the First (the \
         Patriarch); figures such as Chainer, Braids, the scheming merfolk Laquatus, and Phage the \
         Untouchable.\n\
         - WHITE = NOMADS (the signature white people of the block): sun-weathered wandering \
         Nomad clans of the plains and deserts — drifters, mystics, herders and mounted scouts — \
         plus Aven bird-folk and rugged free soldiers/mercenaries who oppose the Cabal. Think \
         travellers and tribes, NOT churchmen.\n\
         - The KROSAN FOREST (green): vast primal wilds full of CENTAURS, druids, elves and \
         enormous beasts; the centaur druid Seton; the barbarian Kamahl reborn as a druid.\n\
         - The PARDIC MOUNTAINS (red): fierce BARBARIAN clans — Kamahl, the dwarf Balthor, Jeska — \
         tattooed warriors, pit-fighters and mountain raiders.\n\
         - The CEPHALID COAST & SEAS (blue): cephalid empire of Aboshan, merfolk, and the Riptide \
         Project of wizards (Empress Llawan).\n\
         - The CABAL (black): sinister cult behind the pit-fighting 'Games', dementia magic, \
         zombies and necromancy; Cabal City and the rotting Aphetto swamps; the First (the \
         Patriarch), Chainer, Braids, the scheming merfolk Laquatus, Phage the Untouchable.\n\
         - The MIRARI: a coveted wish-granting orb whose corruption drives the whole saga.\n\
         Themes to draw on: the Games and the graveyard/'threshold' (Odyssey), creeping darkness, \
         madness and dementia-horror (Torment), and the return of light and the wild (Judgment). \
         STRONGLY FAVOUR the block's signature peoples and creatures — NOMADS, BARBARIANS, \
         CENTAURS, druids, giant Krosan beasts, cephalids and merfolk, dwarves, zombies and \
         dementia-monsters, Aven and mercenaries — over knights, soldiers-in-plate or any clergy. \
         Pick whatever single element best fits THIS card's colour, type and meaning, and stage the \
         scene there so it unmistakably feels like Otaria.\n\n\
         Then OUTPUT ONLY these five fields, each on its own line, nothing before or after — this \
         is how real Magic art descriptions are written (vivid but open-ended, giving the artist \
         room to interpret):\n\
         Color: <the card's colour and kind, e.g. 'White creature' or 'Blue spell'>\n\
         Location: <where the scene is; may be loose or 'unimportant' if it doesn't matter>\n\
         Action: <1-3 sentences describing what is happening — a clear scene with room to interpret>\n\
         Focus: <what the viewer's eye should land on>\n\
         Mood: <one short evocative line capturing the feeling>",
        card.name, card.type_line, card.oracle_text, flavor_line
    );
    // Guardrail: the LLM occasionally slips and depicts the card-game mechanic itself
    // (playing cards / a card-draw / looting). Validate every output and re-prompt — harder
    // each time — until it is clean. Belt-and-braces on top of the in-prompt ban so this can
    // never silently ruin a card again.
    let mut p = prompt.clone();
    for attempt in 1..=3 {
        let d = claude_text(key, &p)?;
        if !mentions_card_mechanic(&d) {
            return Ok(d);
        }
        eprintln!("    (direction depicted a card-game mechanic — re-prompting, attempt {attempt})");
        p = format!(
            "{prompt}\n\nYOUR PREVIOUS ATTEMPT WAS REJECTED: it referred to PLAYING CARDS, a \
             card-draw, looting, discarding or rummaging. Rewrite the five fields with ABSOLUTELY \
             ZERO mention of cards, card-draw, looting, discarding, rummaging or decks — depict \
             ONLY the creature or scene acting in the world."
        );
    }
    // After 3 strict re-prompts this is essentially impossible; take the final attempt.
    claude_text(key, &p)
}

/// True if an art direction wrongly references the card-game mechanic (playing cards /
/// looting / rummaging). Whole-word match so it won't false-positive on "cardinal",
/// "placard", or legit scene words like "loot", "deck", "library" or "discarded armour".
fn mentions_card_mechanic(d: &str) -> bool {
    d.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| matches!(w, "card" | "cards" | "rummage" | "rummaging" | "rummaged"))
}

/// Best-effort append to the GenAI event log (never fails a render).
#[allow(clippy::too_many_arguments)]
fn log_event(
    cache_dir: &Path, card_id: i64, card_name: &str, action: &str,
    artist: Option<&str>, art_hash: Option<&str>, prompt: Option<&str>, detail: Option<&str>,
) {
    if let Ok(db) = crate::events::open(cache_dir) {
        let _ = crate::events::log(&db, card_id, card_name, action, artist, art_hash, prompt, detail);
    }
}

/// Get (cached) or (re)generate the art direction for a card.
pub fn genai_direction(
    editor_db: &Path, cache_dir: &Path, id: i64, regenerate: bool,
) -> Result<String> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    let genai_dir = cache_dir.join("cards").join(sanitize(&card.name)).join("genai");
    let f = genai_dir.join("direction.txt");
    if f.exists() && !regenerate {
        return Ok(fs::read_to_string(&f)?);
    }
    let key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .context("ANTHROPIC_API_KEY not set")?;
    let art_dir = cache_dir.join("art");
    let _ = scryfall_oldest(&card.name, &art_dir); // ensure flavor metadata is cached
    let flavor = card_flavor(&card.name, &art_dir);
    let d = make_direction(&card, &flavor, &key)?;
    fs::create_dir_all(&genai_dir)?;
    fs::write(&f, &d)?;
    log_event(cache_dir, id, &card.name, crate::events::DIRECTION_SET, None, None, Some(&d), None);
    Ok(d)
}

/// Persist a (possibly user-edited) art direction for a card.
fn save_direction(cache_dir: &Path, name: &str, direction: &str) -> Result<()> {
    let genai_dir = cache_dir.join("cards").join(sanitize(name)).join("genai");
    fs::create_dir_all(&genai_dir)?;
    fs::write(genai_dir.join("direction.txt"), direction)?;
    Ok(())
}

/// Generate one artist's art via OpenAI gpt-image-1 (cached at `out`).
fn gen_art_openai(prompt: &str, out: &Path, key: &str) -> Result<()> {
    if let Some(p) = out.parent() {
        fs::create_dir_all(p)?;
    }
    // quality:"high" — max fidelity (gpt-image-1 defaults to "auto", which picks low/cheap).
    let body = json!({"model":"gpt-image-1","n":1,"size":"1536x1024","quality":"high","prompt":prompt});
    let body_file = std::env::temp_dir().join("mtgbrain-openai.json");
    fs::write(&body_file, body.to_string())?;
    let resp = Command::new("curl")
        .args(["-sS", "--max-time", "240", "https://api.openai.com/v1/images/generations",
            "-H", &format!("Authorization: Bearer {key}"), "-H", "Content-Type: application/json",
            "--data"])
        .arg(format!("@{}", body_file.display()))
        .output()
        .context("curl openai")?;
    let v: Value = serde_json::from_slice(&resp.stdout).context("openai response")?;
    if let Some(b64) = v["data"][0]["b64_json"].as_str() {
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).context("decode image")?;
        fs::write(out, bytes)?;
        Ok(())
    } else {
        bail!("openai: {}", v["error"]["message"].as_str().unwrap_or("no image returned"));
    }
}

/// Generate (or reuse) a GenAI art option per artist + render each into a FULL card.
/// Reads ANTHROPIC_API_KEY (art direction) and OPENAI_API_KEY (image gen) from env.
pub fn genai_options(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str, id: i64,
    direction_override: Option<&str>,
) -> Result<Value> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    let openai = std::env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty());
    let card_dir = cache_dir.join("cards").join(sanitize(&card.name));
    let genai_dir = card_dir.join("genai");
    let frame_rel = assets_dir.join(frame_file(&card));
    let year = scryfall_oldest(&card.name, &cache_dir.join("art"))
        .map_or_else(|_| "2001".to_string(), |(_, _, y, _)| y);

    // Use the user-edited direction if supplied (and persist it); else cached/Claude.
    let direction = match direction_override {
        Some(d) if !d.trim().is_empty() => {
            save_direction(cache_dir, &card.name, d.trim())?;
            log_event(cache_dir, id, &card.name, crate::events::DIRECTION_SET, None, None, Some(d.trim()), Some("edited"));
            d.trim().to_string()
        }
        _ => genai_direction(editor_db, cache_dir, id, false)?,
    };

    let mut options = Vec::new();
    let mut errors = Vec::new();
    for (artist, signature, pseudo) in ARTISTS {
        // Stable per (artist, direction) regardless of which prompt variant succeeds.
        let h = short_hash(&format!("{artist}:{direction}"));
        let adir = genai_dir.join(sanitize_bucket(artist));
        let art_png = adir.join(format!("{h}.png"));
        if !art_png.exists() {
            let Some(k) = &openai else {
                errors.push(format!("{artist}: OPENAI_API_KEY not set"));
                continue;
            };
            // Per-artist medium: Rob Alexander famously works in GOUACHE, not oil.
            let medium = if *artist == "Rob Alexander" {
                "a janky, raw, deliberately UNPOLISHED GOUACHE painting (opaque watercolour, \
                 matte chalky finish) worked wet-into-wet — visible bold brushwork, edges that \
                 blur and bleed, broad masses and suggested forms, FEWER details not more. A real \
                 gouache painting that is a loose FINAL DRAFT, NOT a clean, smooth, glossy, \
                 fully-rendered gallery showpiece. Embrace happy accidents and imperfection."
            } else {
                "a janky, raw, deliberately UNPOLISHED wet-on-wet ALLA PRIMA oil — wet paint \
                 dragged into wet paint, visible bold brushwork, edges that blur and bleed, broad \
                 masses and suggested forms, FEWER details not more. A real painting that is a \
                 loose FINAL DRAFT, NOT a clean, smooth, glossy, fully-rendered gallery showpiece. \
                 Embrace happy accidents and imperfection."
            };
            // Universal: paint like a real artist using composition theory, not cram-it-all-in.
            let composition =
                "COMPOSITION (be brave): compose like a real painter using composition theory — \
                 you do NOT have to fit the whole scene into the frame. Crop in close, pick one \
                 strong focal point, use dramatic negative space, asymmetry, leading lines, an \
                 off-centre subject or an unusual viewpoint. A confident, bold partial view beats \
                 a timid wide shot that squeezes everything in. \
                 FORBIDDEN: do NOT draw on Cubism or Constructivism in any way — no geometric \
                 fragmentation, faceting, planar/angular abstraction, collage-like splitting or \
                 constructivist poster style; keep forms representational and painterly. \
                 PAINT ONLY WHAT IS DESCRIBED: do NOT add wings, halos, horns or other \
                 angelic/demonic/divine features to any character unless the art direction \
                 explicitly calls for them — an ordinary human is an ordinary wingless human. \
                 HARD BAN — NO CGI / VIDEO-GAME / VFX LOOK: this is a PROFESSIONAL traditional \
                 painting (oil or gouache) from the late-90s/early-2000s, with real-paint surface \
                 and grounded, tasteful realism. Absolutely NO glowing auras, NO wisps or tendrils \
                 of magic/smoke/shadow, NO ethereal mist-energy, NO floating particles or sparkles, \
                 NO literal sound-waves, sound-rings, shockwave rings or 'visible rings of distorted \
                 air', NO concentric energy ripples or radiating-line effects, NO neon/digital glow, \
                 NO lens flares. Convey action, sound and magic through POSTURE, EXPRESSION, gesture, \
                 light and shadow and composition — exactly as a master illustrator would — NOT \
                 through glowing CGI overlays. A shrieking creature is shown shrieking by its body \
                 and gaping jaws, not by cartoon sound-rings.";
            // NEVER name the artist to OpenAI: its moderation rejects (often living) artist
            // names outright ("moderation_blocked"), burning a generation call every time and
            // leaving only the fallback to actually succeed. Describe the style DIRECTLY via the
            // concentrated signature — which is what truly drives the look anyway. One clean call.
            let styled = format!(
                "Make Magic: the Gathering card art from the late-1990s / early-2000s era, \
                 painted in this EXACT, unmistakable signature style: {signature}. \
                 Lean hard into those specific traits — it must NOT read as generic fantasy art.\n\n\
                 MEDIUM (drive this hard): {medium}\n\n{composition}\n\n\
                 The art direction below is LOOSE INSPIRATION ONLY — a mood, not a spec. Serve the \
                 style and one strong, bold image first; freely reinterpret, simplify, or drop \
                 parts of it (rules are made to be broken), but never contradict it just to be \
                 contradictory.\n\nART DIRECTION:\n{direction}"
            );
            let res = gen_art_openai(&styled, &art_png, k);
            if let Err(e) = res {
                let msg = e.to_string();
                log_event(cache_dir, id, &card.name, crate::events::ART_FAILED, Some(artist), None, Some(&direction), Some(&msg));
                errors.push(format!("{artist}: {msg}"));
                continue;
            }
            log_event(cache_dir, id, &card.name, crate::events::ART_GENERATED, Some(artist), Some(&h), Some(&direction), None);
        }
        let card_png = adir.join("card.png");
        // Folder/tag keep the REAL name; the printed credit is the pseudonym.
        if let Err(e) = compose(
            &card, &art_png, pseudo, &year, assets_dir, &frame_rel, false, &card_png, chrome,
            &format!("genai-{id}-{}", sanitize_bucket(artist)),
            set_symbol_svg(&card.set, cache_dir).as_deref(),
        ) {
            errors.push(format!("{artist} (render): {e}"));
            continue;
        }
        options.push(json!({"artist": artist, "pseudonym": pseudo, "hash": h}));
    }
    Ok(json!({"direction": direction, "options": options, "errors": errors}))
}

/// Path to a rendered GenAI full-card option (for serving in the gallery).
pub fn genai_card_path(cache_dir: &Path, card_name: &str, artist: &str) -> PathBuf {
    cache_dir
        .join("cards")
        .join(sanitize(card_name))
        .join("genai")
        .join(sanitize_bucket(artist))
        .join("card.png")
}

/// Choose a GenAI option: set it as the card's art (settles art.json/art.png).
pub fn genai_choose(
    editor_db: &Path, cache_dir: &Path, id: i64, artist: &str, hash: &str,
) -> Result<()> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    let card_dir = cache_dir.join("cards").join(sanitize(&card.name));
    let art = card_dir
        .join("genai")
        .join(sanitize_bucket(artist))
        .join(format!("{hash}.png"));
    if !art.exists() {
        bail!("genai art not found for {artist}");
    }
    let year = scryfall_oldest(&card.name, &cache_dir.join("art"))
        .map_or_else(|_| "2001".to_string(), |(_, _, y, _)| y);
    // The printed credit is the pseudonym; the real name stays in the event log.
    let pseudo = pseudonym(artist);
    fs::copy(&art, card_dir.join("art.png"))?;
    fs::write(
        card_dir.join("art.json"),
        json!({"art_ref": format!("genai:{artist}:{hash}"), "artist": pseudo, "year": year}).to_string(),
    )?;
    // Persist the chosen illustrator into the DB (a dedicated column, NOT the overrides
    // errata blob) so every render — preview and final — credits the pseudonym.
    {
        let w = Connection::open(editor_db)?;
        let _ = w.execute(
            "UPDATE cube_cards SET illustrator=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, pseudo],
        );
    }
    log_event(cache_dir, id, &card.name, crate::events::CHOSEN, Some(artist), Some(hash), None, None);
    Ok(())
}

/// Clear the chosen GenAI art for a card (logged; art.json removed so the render
/// falls back to MPCfill/Scryfall on next render).
pub fn genai_unchoose(editor_db: &Path, cache_dir: &Path, id: i64) -> Result<()> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    let card_dir = cache_dir.join("cards").join(sanitize(&card.name));
    let _ = fs::remove_file(card_dir.join("art.json"));
    let _ = fs::remove_file(card_dir.join("art.png"));
    {
        let w = Connection::open(editor_db)?;
        let _ = w.execute(
            "UPDATE cube_cards SET illustrator='', updated_at=datetime('now') WHERE id=?1",
            params![id],
        );
    }
    log_event(cache_dir, id, &card.name, crate::events::UNCHOSEN, None, None, None, None);
    Ok(())
}

/// Choose the card's ORIGINAL (real) art as a deliberate decision: clear any GenAI
/// sidecar (so the card renders its real art) and record a CHOSEN event crediting
/// "Original art" — so the card stays in the Review list, marked decided. The genai_art
/// flag is kept ON so it remains visible/auditable there.
pub fn genai_choose_original(editor_db: &Path, cache_dir: &Path, id: i64) -> Result<()> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    // GUARD: never mark "Original art" for a card with no GenAI generation yet — that would
    // settle it as decided and the pass would skip it, so it'd never get a gallery to review.
    let store = crate::events::open(cache_dir)?;
    if !crate::events::has_generated(&store, id)? {
        bail!("{}: generate the GenAI gallery before choosing Original art", card.name);
    }
    let card_dir = cache_dir.join("cards").join(sanitize(&card.name));
    let _ = fs::remove_file(card_dir.join("art.json")); // → render falls back to the real art
    let _ = fs::remove_file(card_dir.join("art.png"));
    {
        let w = Connection::open(editor_db)?;
        let _ = w.execute(
            "UPDATE cube_cards SET illustrator='', updated_at=datetime('now') WHERE id=?1",
            params![id],
        );
    }
    log_event(cache_dir, id, &card.name, crate::events::CHOSEN,
        Some("Original art"), Some(""), None, Some("original"));
    Ok(())
}

/// History of GenAI events for a card (for the review UI / time-travel).
pub fn genai_history(cache_dir: &Path, id: i64) -> Result<Value> {
    let db = crate::events::open(cache_dir)?;
    Ok(json!({
        "history": crate::events::history(&db, id)?,
        "current_direction": crate::events::current_direction(&db, id)?,
        "current_choice": crate::events::current_choice(&db, id).ok().flatten().map(|(a, _)| a),
    }))
}

/// Set an old event's prompt as the current direction (time-travel / reprompt),
/// logged as a fresh direction edit.
pub fn genai_reprompt(editor_db: &Path, cache_dir: &Path, id: i64, event_id: i64) -> Result<String> {
    let store = crate::events::open(cache_dir)?;
    let prompt = crate::events::prompt_of(&store, event_id)?
        .context("that event has no prompt to restore")?;
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    save_direction(cache_dir, &card.name, &prompt)?;
    log_event(cache_dir, id, &card.name, crate::events::DIRECTION_SET, None, None, Some(&prompt),
        Some(&format!("reprompt from event {event_id}")));
    Ok(prompt)
}

/// Cube-wide GenAI review dashboard: every genai-flagged card with its status.
pub fn genai_dashboard(editor_db: &Path, cache_dir: &Path) -> Result<Value> {
    let store = crate::events::open(cache_dir)?;
    let stat = crate::events::status(&store)?;
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = db.prepare(
        "SELECT id, name FROM cube_cards WHERE genai_art=1 AND removed=0 ORDER BY id",
    )?;
    let cards: Vec<Value> = stmt
        .query_map([], |r| {
            let id: i64 = r.get(0)?;
            let name: String = r.get(1)?;
            Ok((id, name))
        })?
        .filter_map(std::result::Result::ok)
        .map(|(id, name)| {
            let s = stat.get(&id).cloned().unwrap_or_else(|| json!({}));
            json!({"id": id, "name": name, "status": s})
        })
        .collect();
    Ok(json!({"cards": cards, "artists": ARTISTS.iter().map(|(a, _, _)| *a).collect::<Vec<_>>()}))
}

/// Cached options for a card's CURRENT direction (no generation) — for the review
/// gallery: which artists already have art on disk, plus the current choice.
pub fn genai_existing(editor_db: &Path, cache_dir: &Path, id: i64) -> Result<Value> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let card = load_card(&db, id)?;
    let genai_dir = cache_dir.join("cards").join(sanitize(&card.name)).join("genai");
    let direction = fs::read_to_string(genai_dir.join("direction.txt")).unwrap_or_default();
    let mut options = Vec::new();
    if !direction.is_empty() {
        for (artist, _, pseudo) in ARTISTS {
            let h = short_hash(&format!("{artist}:{direction}"));
            if genai_dir.join(sanitize_bucket(artist)).join(format!("{h}.png")).exists() {
                options.push(json!({"artist": artist, "pseudonym": pseudo, "hash": h}));
            }
        }
    }
    let store = crate::events::open(cache_dir)?;
    let chosen = crate::events::current_choice(&store, id).ok().flatten();
    Ok(json!({
        "direction": direction,
        "options": options,
        "chosen": chosen.map(|(a, _)| a),
    }))
}

/// Async full pass: generate the GenAI gallery for every genai-flagged card.
/// Idempotent/resumable (cached arts are skipped); logs every action. Run it in
/// the background, then review/choose in the editor's "Review GenAI" tab.
pub fn genai_pass(editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str) -> Result<()> {
    let ids: Vec<(i64, String)> = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut stmt = db.prepare(
            "SELECT id, name FROM cube_cards WHERE genai_art=1 AND removed=0 AND in_db_found=1 \
             AND COALESCE(genai_done,0)=0 ORDER BY id",
        )?;
        let mut v = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            v.push((r.get::<_, i64>(0)?, r.get::<_, String>(1)?));
        }
        v
    };
    let total = ids.len();
    println!("GenAI full pass: {total} flagged cards");
    let store = crate::events::open(cache_dir).ok();
    for (i, (id, name)) in ids.iter().enumerate() {
        // Respect ANY settled decision (an artist pick OR "Original art") — never regenerate.
        let decided = store
            .as_ref()
            .and_then(|s| crate::events::current_choice(s, *id).ok().flatten())
            .is_some();
        if decided {
            println!("[{}/{total}] {name}: skip (already decided)", i + 1);
            continue;
        }
        match genai_options(editor_db, assets_dir, cache_dir, chrome, *id, None) {
            Ok(v) => {
                let ok = v["options"].as_array().map_or(0, Vec::len);
                let err = v["errors"].as_array().map_or(0, Vec::len);
                println!("[{}/{total}] {name}: {ok} ok, {err} failed", i + 1);
            }
            Err(e) => eprintln!("[{}/{total}] {name}: {e}", i + 1),
        }
    }
    println!("done — review in the editor's Review GenAI tab");
    Ok(())
}

// ---------------------------------------------------------------------------
// HTML (Seventh bounds; in-page auto-fit; reminder italics; white-star foil)
// ---------------------------------------------------------------------------

fn pc(f: f64) -> String { format!("{:.4}", f * 100.0) }
fn px(f: f64) -> String { format!("{}", (f * f64::from(FACE_H)) as u32) }

const TEMPLATE: &str = include_str!("card_template.html");

fn build_html(c: &Card, frame_abs: &Path, art_abs: &Path, assets_dir: &Path, art: &Art, foil: bool, set_svg: Option<&Path>) -> String {
    let fonts = assets_dir.join("fonts");
    let f = |p: &str| format!("file://{}", fonts.join(p).display());
    let mana_base = format!("file://{}", assets_dir.join("mana").display());
    let italic_face = if fonts.join("mplantin-italic.ttf").exists() {
        format!("@font-face {{ font-family:'mplantin'; font-style:italic; src:url('{}'); }}", f("mplantin-italic.ttf"))
    } else {
        String::new()
    };
    // Any card with a printed P/T gets the box — creatures AND **Vehicles** (Artifact —
    // Vehicle: a P/T but is_creature=0, e.g. The Last Ride 13/13). Loyalty → planeswalkers.
    let pt = if !c.power.is_empty() || !c.toughness.is_empty() {
        format!(r#"<div class="box pt"><span>{}/{}</span></div>"#, esc(&c.power), esc(&c.toughness))
    } else if !c.loyalty.is_empty() {
        format!(r#"<div class="box pt"><span>{}</span></div>"#, esc(&c.loyalty))
    } else {
        String::new()
    };
    // The printed "Illus." credit prefers the editor-DB `illustrator` override (set by a
    // GenAI pseudonym pick or an art upload); otherwise it falls back to the art's own
    // historical artist credit.
    let credit = if !c.illustrator.trim().is_empty() {
        c.illustrator.trim()
    } else {
        art.artist.trim()
    };
    let illus = if credit.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="info illus"><span>Illus. {}</span></div>"#, esc(credit))
    };
    // Coloured-artifact COLOUR INDICATOR: the frame stays fully brown; a big circular
    // swatch of the card's colour (a patch of the matching colour frame's marble
    // texture) sits to the LEFT of the type line, decorated exactly like the set symbol
    // (white keyline + inlaid black inner-border gradient via the svg filter). The type
    // text is indented (TYPAD) so it clears the indicator.
    let diam_px = (0.030 * f64::from(FACE_H)) as u32; // ≈84px — half size, same centre
    // Indicator colour logic: 1 colour → solid that colour; 2 colours → vertical 50:50
    // split of the two; 3+ colours → gold. Only on coloured (non-land) artifacts.
    let ind_frames: Vec<PathBuf> = {
        let t = c.type_line.to_lowercase();
        let is_land = t.contains("land");
        let is_artifact = t.contains("artifact");
        let ci = ci_letters(&c.color_identity);
        let cost = card_colors(c);
        // The dot shows the colour identity the FRAME doesn't already convey: a coloured
        // artifact (brown frame), a card with a MANUAL identity (costless / off-cost), or a
        // coloured-via-ability artifact. Ordinary coloured cards show it through their pips.
        let letters: Vec<char> = if c.ci_manual && !ci.is_empty() {
            // A hand-declared identity gets the dot, EXCEPT a mono-colour land that already
            // conveys it through its tinted land frame (wl/ul/…). A multi-colour land has no
            // tinted frame, so it DOES get the dot (e.g. a W/U dual-identity land).
            if is_land && ci.len() == 1 { vec![] } else { ci }
        } else if is_land {
            vec![]
        } else if is_artifact && !cost.is_empty() {
            cost
        } else if is_artifact && !ci.is_empty() {
            ci
        } else {
            vec![]
        };
        if letters.is_empty() {
            vec![]
        } else if letters.len() >= 3 {
            vec![assets_dir.join("frames/m.png")]
        } else {
            letters.iter().map(|ch| assets_dir.join(color_frame(*ch))).collect()
        }
    };
    let ind_refs: Vec<&Path> = ind_frames.iter().map(PathBuf::as_path).collect();
    let (colorind, typad) = color_indicator_png(&ind_refs, 256)
        .map_or_else(
            || (String::new(), "0".to_string()),
            |png| {
                let uri = format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&png)
                );
                let d = f64::from(diam_px);
                let w_frac = d / f64::from(FACE_W) * 100.0; // % of face width
                let h_frac = d / f64::from(FACE_H) * 100.0; // % of face height
                let left = 10.3; // just inside the frame's inner edge
                let top = (0.5757 - d / f64::from(FACE_H) / 2.0) * 100.0; // centred on type bar
                let svg = format!(
                    r##"<svg class="colorind" style="left:{left:.2}%;top:{top:.2}%;width:{w_frac:.3}%;height:{h_frac:.3}%" viewBox="0 0 {diam_px} {diam_px}" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
<defs><filter id="ci" x="-30%" y="-30%" width="160%" height="160%" color-interpolation-filters="sRGB">
<feMorphology in="SourceAlpha" operator="dilate" radius="2.2" result="d"/>
<feFlood flood-color="#fbfaf3"/><feComposite in2="d" operator="in" result="key"/>
<feFlood flood-color="#000" flood-opacity="1"/><feComposite in2="SourceAlpha" operator="out" result="o"/>
<feGaussianBlur in="o" stdDeviation="3.3" result="ob"/><feComposite in="ob" in2="SourceAlpha" operator="in" result="ir"/>
<feComponentTransfer in="ir" result="inner"><feFuncA type="linear" slope="1.35"/></feComponentTransfer>
<feMerge><feMergeNode in="key"/><feMergeNode in="SourceGraphic"/><feMergeNode in="inner"/></feMerge>
</filter></defs>
<image href="{uri}" width="{diam_px}" height="{diam_px}" preserveAspectRatio="xMidYMid meet" filter="url(#ci)"/>
</svg>"##
                );
                // indent the type text just past the indicator's right edge
                let pad = ((left / 100.0 + w_frac / 100.0) * f64::from(FACE_W)
                    - 0.1074 * f64::from(FACE_W)
                    + 18.0) as i32;
                (svg, format!("{pad}px"))
            },
        );
    let year = if art.year.is_empty() { "2001".to_string() } else { art.year.clone() };
    let foil_layer = if foil {
        format!(r#"<img class="foilstar" src="file://{}">"#, assets_dir.join("foil/star.svg").display())
    } else {
        String::new()
    };

    let pairs: Vec<(&str, String)> = vec![
        ("MANA_CSS", f("mana.css")),
        ("GOUDY", f("goudy-medieval.ttf")),
        ("MPLANTIN", f("mplantin.ttf")),
        ("ITALIC_FACE", italic_face),
        ("W", W.to_string()), ("H", H.to_string()),
        ("FW", FACE_W.to_string()), ("FH", FACE_H.to_string()),
        ("BX", ((W - FACE_W) / 2).to_string()), ("BY", ((H - FACE_H) / 2).to_string()),
        // All bounds/sizes are EXACT cardconjurer packSeventh.js values (fractions of the
        // FACE). Font px = fraction * FACE_H. Shadows match CC: sharp black, no blur,
        // offset (0.002*FACE_W, 0.0015*FACE_H).
        ("AX", pc(ART_WINDOW_FRAC[0])), ("AY", pc(ART_WINDOW_FRAC[1])),
        ("AW", pc(ART_WINDOW_FRAC[2])), ("AH", pc(ART_WINDOW_FRAC[3])),
        ("TX", pc(0.1134)), ("TY", pc(0.0481)), ("TW", pc(0.7734)), ("TH", pc(0.041)),
        // Pips share the TITLE's vertical band (MAY=TY, MAH=TH) and are centred in it,
        // so pip-centre == title-centre. (cardconjurer's literal mana y=0.0539 is offset
        // back up by its textSize*0.34 baseline math, which we don't replicate — using it
        // raw dropped the pips a full box too low.) MAY tuned with tools/cardgeom
        // `pip-align` (colour probe) so the pip TOPS line up with the title-text top.
        ("MAX", pc(0.1067)), ("MAY", pc(0.0415)), ("MAW", pc(0.8174)), ("MAH", px(0.041)),
        ("TSZ", px(0.041)), ("MSZ", px(72.0 / 1638.0)),
        ("SHX", px(0.002 * f64::from(FACE_W) / f64::from(FACE_H))), ("SHY", px(0.0015)),
        // Type width stops short of the set symbol (~0.858) so a long type line
        // auto-fits/shrinks instead of overlapping the symbol.
        ("TYX", pc(0.1074)), ("TYY", pc(0.5486)), ("TYW", pc(0.70)), ("TYH", pc(0.0543)), ("TYSZ", px(0.032)),
        ("RX", pc(0.128)), ("RY", pc(0.6067)), ("RW", pc(0.744)), ("RH", pc(0.2724)), ("RSZ", px(0.0358)),
        ("PX", pc(0.8074)), ("PY", pc(0.9043)), ("PW", pc(0.1367)), ("PSZ", px(0.0429)),
        ("IY", pc(1908.0 / 2100.0)), ("ISZ", px(0.0172)),
        ("LY", pc(1940.0 / 2100.0)), ("LSZ", px(0.0143)),
        // Set symbol: square box at the right end of the type line, vertically centred
        // (centre ≈ 0.576). A touch smaller than the type height so big modern symbols
        // don't crowd the frame borders. Monocolour mask + dark inner border + white key.
        // Wide box, right-anchored: symbols share a constant height and grow LEFT when
        // wide (Nemesis etc.) instead of being squashed into a square. Square symbols
        // keep their size and right-edge position.
        // NEAR-SQUARE box → contain gives the real "two placement modes": a SQUARE
        // symbol binds on HEIGHT (fills the type bar); a WIDE symbol (Nemesis) binds
        // on WIDTH and so comes out SHORTER, right-aligned & vertically centred —
        // exactly like the printed card, instead of ballooning to full height.
        ("SSR", pc(0.088)), ("SSY", pc(0.5566)), ("SSW", pc(0.062)), ("SSH", pc(0.0382)),
        // Errata scroll: a HORIZONTAL parchment banner wrapping around the LEFT edge of
        // the frame (left fold off the card edge, rolled end resting on the art). Left-
        // anchored (ESX=0); vertically centred at the GOLDEN-RATIO point of the art
        // window (0.618 down → 0.373), lower than centre, toward the type line.
        ("ESX", pc(0.035)), ("ESY", pc(0.3728 - 0.052 / 2.0)), ("ESW", pc(0.10)), ("ESH", pc(0.052)),
    ];
    // Set-symbol element: an inline <svg> that embeds the Scryfall silhouette as an
    // <image> and runs a single FILTER to paint, in order:
    //   1. a thin WHITE keyline  (dilate the alpha, flood white)
    //   2. the solid MONOCOLOUR body  (flood rarity colour, clip to the alpha)
    //   3. a contour-following GRADIENT INNER BORDER — flood black, composite `out`
    //      (black everywhere EXCEPT the shape), blur it so the black bleeds inward
    //      across the edge, then composite `in` (keep only the part inside the shape).
    //      That yields a dark ring that hugs the WHOLE inner contour and fades toward
    //      the centre = the real inset bevel, NOT a directional drop-shadow.
    // The svg is a data: <image> (file:// is blocked in headless Chrome). viewBox is the
    // box's own px size so the filter radii read as pixels; xMaxYMid right-anchors it.
    let setsym = set_svg
        .and_then(|p| fs::read(p).ok())
        .and_then(|bytes| set_symbol_ink(&bytes))
        .map(|(png, ink_aspect)| {
            let uri = format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&png)
            );
            // TWO PLACEMENT MODES keyed on the symbol's true INK aspect (set_symbol_ink
            // measures it; Scryfall's square viewBox can't be trusted):
            //  • WIDE / rectangular (aspect ≥ 1.45, e.g. Nemesis, M10, M11): bind on WIDTH
            //    — fill a generous width and let height follow → WIDE but SHORT.
            //  • SQUARE / TALL (aspect < 1.45, e.g. Odyssey): bind on a HEIGHT cap — so it
            //    just FITS the type bar instead of ballooning.
            // We size the <svg> element itself to those exact px so it never letterboxes;
            // the filter radii are in the same px space → a constant keyline everywhere.
            let aspect = ink_aspect.clamp(0.25, 6.0);
            let (w_px, h_px) = if aspect >= 1.45 {
                let mut w = 250.0_f64;
                let mut h = w / aspect;
                if h > 92.0 {
                    h = 92.0;
                    w = h * aspect;
                }
                (w, h)
            } else {
                let h = 100.0_f64;
                (h * aspect, h)
            };
            let w_frac = w_px / f64::from(FACE_W) * 100.0;
            let h_frac = h_px / f64::from(FACE_H) * 100.0;
            let top = 57.57 - h_frac / 2.0; // vertically centred on the type bar (≈0.576)
            format!(
                r##"<svg class="setsym" style="width:{w_frac:.3}%;height:{h_frac:.3}%;top:{top:.3}%;right:8.8%" viewBox="0 0 {w_px:.1} {h_px:.1}" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
<defs><filter id="ss" x="-30%" y="-30%" width="160%" height="160%" color-interpolation-filters="sRGB">
<feMorphology in="SourceAlpha" operator="dilate" radius="2.2" result="d"/>
<feFlood flood-color="#fbfaf3"/><feComposite in2="d" operator="in" result="key"/>
<feFlood flood-color="{fill}"/><feComposite in2="SourceAlpha" operator="in" result="body"/>
<feFlood flood-color="#000" flood-opacity="1"/><feComposite in2="SourceAlpha" operator="out" result="o"/>
<feGaussianBlur in="o" stdDeviation="3.3" result="ob"/><feComposite in="ob" in2="SourceAlpha" operator="in" result="ir"/>
<feComponentTransfer in="ir" result="inner"><feFuncA type="linear" slope="1.35"/></feComponentTransfer>
<feMerge><feMergeNode in="key"/><feMergeNode in="body"/><feMergeNode in="inner"/></feMerge>
</filter></defs>
<image href="{uri}" width="{w_px:.1}" height="{h_px:.1}" preserveAspectRatio="xMidYMid meet" filter="url(#ss)"/>
</svg>"##,
                fill = rarity_fill(&c.rarity),
            )
        })
        .unwrap_or_default();
    let content: [(&str, String); 12] = [
        ("ART", format!("file://{}", art_abs.display())),
        ("FRAME", format!("file://{}", frame_abs.display())),
        ("COLORIND", colorind),
        ("TYPAD", typad),
        ("NAME", esc(&c.display_name)),
        ("MANA", manaify(&c.mana_cost, &mana_base)),
        ("TYPE", esc(&c.type_line)),
        ("RULES", rules_html(&c.oracle_text, &c.flavor, &mana_base)),
        ("PT", pt),
        ("ILLUS", illus),
        ("YEAR", year),
        ("SETSYM", setsym),
    ];
    // Errata scroll overlay (only when the card's printed text was changed for the cube).
    let scroll = if c.is_errata {
        format!(
            r#"<div class="errata-scroll" style="background-image:url('file://{}')"></div>"#,
            assets_dir.join("overlays/errata-scroll.svg").display()
        )
    } else {
        String::new()
    };
    let extra_pair = [("FOIL", foil_layer), ("SCROLL", scroll)];

    let mut html = TEMPLATE.to_string();
    for (k, v) in pairs.iter().chain(content.iter()).chain(extra_pair.iter()) {
        html = html.replace(&format!("%%{k}%%"), v);
    }
    html
}

/// Rules text → paragraphs, mana symbols inline, parenthetical reminder italic.
fn rules_html(text: &str, flavor: &str, mana_base: &str) -> String {
    let mut s = String::new();
    for para in paragraphs(text) {
        let parts: Vec<String> = para
            .iter()
            .map(|l| reminder_italic(&manaify(l, mana_base)))
            .collect();
        s.push_str("<p>");
        s.push_str(&parts.join("<br>"));
        s.push_str("</p>");
    }
    // Flavor text: italic, below the rules, separated by a thin divider (real old frame).
    for (i, para) in paragraphs(flavor).iter().enumerate() {
        if i == 0 {
            s.push_str(r#"<div class="flavbar"></div>"#);
        }
        let parts: Vec<String> = para.iter().map(|l| flav_emph(l)).collect();
        s.push_str(&format!(r#"<p class="flav">{}</p>"#, parts.join("<br>")));
    }
    s
}

/// Group `\n`-separated lines into paragraphs, honouring Markdown's hard-break syntax.
///
/// A line ending in two (or more) trailing spaces is a Markdown hard line break: it stays
/// in the *same* paragraph as the following line, joined later by `<br>` — a CONDENSED
/// newline (tight `line-height`), used for verse/poetry. Any other non-blank line ends its
/// paragraph, which renders with the usual spaced paragraph margin. Blank lines are dropped.
///
/// Text with no trailing double-spaces behaves exactly as before: one paragraph per line.
fn paragraphs(text: &str) -> Vec<Vec<String>> {
    let mut paras: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.trim_end_matches('\r');
        let hard_break = raw.ends_with("  ");
        let line = raw.trim();
        if line.is_empty() {
            if !cur.is_empty() {
                paras.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(line.to_string());
        if !hard_break {
            paras.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        paras.push(cur);
    }
    paras
}

/// Flavor emphasis the real-MTG way: flavor is set in italics, so emphasis (`*word*`
/// or `**word**`) is rendered UPRIGHT/roman — inverted italics, like "Principia" — not
/// extra-italic. Operates on raw text; everything outside the markers is HTML-escaped.
fn flav_emph(line: &str) -> String {
    let mut out = String::new();
    let mut emph = false;
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        if c == '*' {
            if it.peek() == Some(&'*') {
                it.next(); // treat ** (bold) the same as * — both invert to upright
            }
            out.push_str(if emph { "</span>" } else { r#"<span class="up">"# });
            emph = !emph;
        } else {
            out.push_str(&esc_char(c));
        }
    }
    if emph {
        out.push_str("</span>");
    }
    out
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

fn manaify(text: &str, mana_base: &str) -> String {
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
            out.push_str(&mana_icon(&sym, mana_base));
        } else {
            out.push_str(&esc_char(ch));
        }
    }
    out
}

/// One pip = cardconjurer's own mana-symbol SVG (disk + glyph baked in), so the
/// pips match cardconjurer exactly. `{T}`→t.svg, `{Q}`→untap.svg, else the
/// lowercased code with `/` stripped (e.g. `{W/U}`→wu.svg, `{2/U}`→2u.svg).
fn mana_icon(sym: &str, mana_base: &str) -> String {
    let key = match sym.to_uppercase().as_str() {
        "T" => "t".to_string(),
        "Q" => "untap".to_string(),
        other => other.replace('/', "").to_lowercase(),
    };
    format!(r#"<img class="ms-img" src="{mana_base}/{key}.svg">"#)
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
    let mut card = load_card(&db, id)?;

    let frame_rel = assets_dir.join(frame_file(&card));
    if !frame_rel.exists() {
        bail!("missing frame asset {} — run `mtgbrain render assets`", frame_rel.display());
    }
    let dir = cache_dir.join("cards").join(sanitize(&card.name));
    // Durable art override (editor DB) wins and is re-materialised every render so it can
    // never be trampled by a cache wipe / auto re-pick.
    materialize_override(&db, cache_dir, id, &card.name, &dir)?;
    let art = acquire_art(&card.name, cache_dir, &dir, backend, false)?;
    // Flavor text (italic, below rules) — from the cached Scryfall print metadata that
    // acquire_art just settled. An "flavor" override wins if the editor set one.
    // Fall back to the Scryfall flavor only when the editor set NO flavor override.
    // An explicit (even empty) override suppresses flavor — printing none.
    if card.flavor.is_empty() && !card.flavor_overridden {
        card.flavor = card_flavor(&card.name, &cache_dir.join("art"));
    }
    if card.set.is_empty() || card.rarity.is_empty() {
        let (set, rarity) = card_set_rarity(&card.name, &cache_dir.join("art"));
        if card.set.is_empty() {
            card.set = set;
        }
        if card.rarity.is_empty() {
            card.rarity = rarity;
        }
    }
    // Printed credit: the editor-chosen illustrator (GenAI pseudonym) wins; otherwise
    // the real historical illustrator from the oldest Scryfall printing.
    let credit = if card.illustrator.is_empty() { art.artist.clone() } else { card.illustrator.clone() };
    let hash = card_hash(&card, &art.art_ref, &credit);

    let out = if foil { dir.join("foil").join(format!("{hash}.png")) } else { dir.join(format!("{hash}.png")) };
    if out.exists() && !force {
        return Ok(out);
    }
    let set_svg = set_symbol_svg(&card.set, cache_dir);
    compose(&card, &art.path, &credit, &art.year, assets_dir, &frame_rel, foil, &out, chrome,
        &format!("{id}{}", u8::from(foil)), set_svg.as_deref())?;
    Ok(out)
}

/// Render a preview of the card with its REAL (Scryfall) art — the "Original art" option
/// shown in the Review tab so the operator can SEE and vote for the base art. Rendered
/// into an isolated `genai/_base/` dir so it never touches a chosen-art sidecar; credits
/// the real historical illustrator. Always re-rendered (cheap; cached art crop).
pub fn genai_base_card(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str, id: i64,
) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut card = load_card(&db, id)?;
    let frame_rel = assets_dir.join(frame_file(&card));
    let base_dir = cache_dir.join("cards").join(sanitize(&card.name)).join("genai").join("_base");
    fs::create_dir_all(&base_dir)?;
    // Acquire the REAL art into the ISOLATED base dir (Scryfall, fast/free — same
    // illustration MPCfill would pick, just lower-res; enough to vote on).
    let art = acquire_art(&card.name, cache_dir, &base_dir, None, false)?;
    if card.flavor.is_empty() && !card.flavor_overridden {
        card.flavor = card_flavor(&card.name, &cache_dir.join("art"));
    }
    if card.set.is_empty() || card.rarity.is_empty() {
        let (set, rarity) = card_set_rarity(&card.name, &cache_dir.join("art"));
        if card.set.is_empty() {
            card.set = set;
        }
        if card.rarity.is_empty() {
            card.rarity = rarity;
        }
    }
    let out = base_dir.join("card.png");
    let set_svg = set_symbol_svg(&card.set, cache_dir);
    compose(&card, &art.path, &art.artist, &art.year, assets_dir, &frame_rel, false, &out, chrome,
        &format!("base-{id}"), set_svg.as_deref())?;
    Ok(out)
}

/// Composite a full card: frame + given art + text, screenshot to `out`.
#[allow(clippy::too_many_arguments)]
fn compose(card: &Card, art_path: &Path, artist: &str, year: &str, assets_dir: &Path,
    frame_rel: &Path, foil: bool, out: &Path, chrome: &str, tag: &str, set_svg: Option<&Path>) -> Result<()> {
    fs::create_dir_all(out.parent().unwrap())?;
    let assets_abs = fs::canonicalize(assets_dir)?;
    let frame_abs = fs::canonicalize(frame_rel)?;
    let art_abs = fs::canonicalize(art_path)?;
    let set_abs = set_svg.and_then(|p| fs::canonicalize(p).ok());
    let art = Art { path: art_abs.clone(), art_ref: String::new(), artist: artist.to_string(), year: year.to_string() };
    let html = build_html(card, &frame_abs, &art_abs, &assets_abs, &art, foil, set_abs.as_deref());
    let html_path = std::env::temp_dir().join(format!("mtgbrain-render-{tag}.html"));
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
    Ok(())
}

// ---------------------------------------------------------------------------
// Eighth-Edition (2003 / modern) frame — same Chrome-screenshots-HTML pipeline,
// pack8th geometry + Matrix fonts. The 8th colour frame is a self-contained PNG
// (transparent art hole) so it overlays like Seventh; the P/T box is a separate
// layer. Used by the second editor instance (`edit serve --frame 8th`) for the
// OTHER cube on the back of the MPC card.
// ---------------------------------------------------------------------------

const TEMPLATE_8TH: &str = include_str!("card_template_8th.html");

fn color_frame_8th(ch: char) -> &'static str {
    match ch {
        'W' => "frames8/w.png", 'U' => "frames8/u.png", 'B' => "frames8/b.png",
        'R' => "frames8/r.png", _ => "frames8/g.png",
    }
}

fn frame_file_8th(c: &Card) -> &'static str {
    let t = c.type_line.to_lowercase();
    if t.contains("land") {
        let lc: Vec<char> = c.color_identity.chars().filter(|ch| "WUBRG".contains(*ch)).collect();
        return match lc.as_slice() {
            ['W'] => "frames8/wl.png", ['U'] => "frames8/ul.png", ['B'] => "frames8/bl.png",
            ['R'] => "frames8/rl.png", ['G'] => "frames8/gl.png", _ => "frames8/l.png",
        };
    }
    if t.contains("artifact") {
        return "frames8/a.png";
    }
    let cols = if c.ci_manual { ci_letters(&c.color_identity) } else { card_colors(c) };
    match cols.len() {
        0 => "frames8/c.png",
        1 => color_frame_8th(cols[0]),
        _ => "frames8/m.png",
    }
}

/// The WUBRG colours an 8ED frame should show: a manual identity wins, else the cost's
/// colours (`card_colors` already falls back to the `colors` field). Empty = colourless.
fn frame_colors_8th(c: &Card) -> Vec<char> {
    if c.ci_manual {
        ci_letters(&c.color_identity)
    } else {
        card_colors(c)
    }
}

/// Build (and cache) a COLOURED-ARTIFACT 8ED frame: the artifact base (`a.png`) with the
/// card's colour overlaid on the "twin" bars — the title + type pills — through
/// cardconjurer's region masks. A colour PAIR splits the twins left/right (left colour on
/// the left half of each bar, right colour on the right); 3+ colours use the gold (`m`)
/// frame. This is the real modern colour-artifact look — a metal card whose name/type bars
/// carry its colour — instead of a plain artifact frame plus a colour-indicator dot. Cached
/// at `<cache>/frames8_gen/a_<COLS>.png`; None (caller falls back to `a.png`) if an asset is
/// missing.
fn colored_artifact_frame(assets_dir: &Path, cache_dir: &Path, cols: &[char]) -> Option<PathBuf> {
    if cols.is_empty() {
        return None;
    }
    let key: String = cols.iter().collect();
    let out_dir = cache_dir.join("frames8_gen");
    let _ = fs::create_dir_all(&out_dir);
    let out = out_dir.join(format!("a_{key}.png"));
    if out.exists() {
        return Some(out);
    }
    let mut base = image::open(assets_dir.join("frames8/a.png")).ok()?.to_rgba8();
    let (w, h) = base.dimensions();
    let load_fit = |rel: String| -> Option<image::RgbaImage> {
        let img = image::open(assets_dir.join(rel)).ok()?.to_rgba8();
        Some(if img.dimensions() == (w, h) {
            img
        } else {
            image::imageops::resize(&img, w, h, image::imageops::FilterType::Triangle)
        })
    };
    let title = load_fit("frames8/mask_title.png".into())?;
    let typ = load_fit("frames8/mask_type.png".into())?;
    // Colour layer(s): 1 → that colour; 2 → left/right split twins; 3+ → gold.
    let frames: Vec<image::RgbaImage> = if cols.len() >= 3 {
        vec![load_fit("frames8/m.png".into())?]
    } else {
        cols.iter().filter_map(|ch| load_fit(color_frame_8th(*ch).into())).collect()
    };
    if frames.is_empty() {
        return None;
    }
    // The masks are 1-bit palette PNGs (saturated-RED shape on a transparent/white field);
    // treat a pixel as in-region if it's that opaque red — robust to either background.
    let inmask = |m: &image::RgbaImage, x: u32, y: u32| -> bool {
        let p = m.get_pixel(x, y).0;
        p[3] > 64 && i32::from(p[0]) - i32::from(p[1]) > 60 && i32::from(p[0]) - i32::from(p[2]) > 60
    };
    let half = w / 2;
    for y in 0..h {
        for x in 0..w {
            if inmask(&title, x, y) || inmask(&typ, x, y) {
                let fr = if frames.len() == 2 {
                    if x < half { &frames[0] } else { &frames[1] }
                } else {
                    &frames[0]
                };
                *base.get_pixel_mut(x, y) = *fr.get_pixel(x, y);
            }
        }
    }
    base.save(&out).ok()?;
    Some(out)
}

/// The matching 8th P/T box art (overlaid at pack8th's PT bounds).
fn pt_box_8th(c: &Card) -> &'static str {
    let t = c.type_line.to_lowercase();
    if t.contains("land") {
        return "frames8/pt/l.png";
    }
    let cols = frame_colors_8th(c);
    // A COLOURED artifact takes its colour's P/T box (matching its twin frame); a colourless
    // artifact keeps the metal one.
    if t.contains("artifact") && cols.is_empty() {
        return "frames8/pt/a.png";
    }
    match cols.as_slice() {
        [] => "frames8/pt/a.png",
        [one] => match one {
            'W' => "frames8/pt/w.png", 'U' => "frames8/pt/u.png", 'B' => "frames8/pt/b.png",
            'R' => "frames8/pt/r.png", _ => "frames8/pt/g.png",
        },
        _ => "frames8/pt/m.png",
    }
}

/// Ink + shadow for the BOTTOM credit line only (title/type/PT are always black in 8ED).
/// pack8th: black, but WHITE on Black / Land / Colorless frames (their bottom border is black).
fn info_ink_8th(frame: &str) -> (&'static str, &'static str) {
    let base = frame.rsplit('/').next().unwrap_or(frame);
    let dark_border = base == "b.png" || base == "c.png" || base.ends_with("l.png");
    if dark_border {
        ("#f4f1e8", "1px 1px 0 rgba(0,0,0,0.85)")
    } else {
        ("#161310", "none")
    }
}

/// Build the set-symbol `<svg>` (white keyline + monocolour body + contour inner-border),
/// right-anchored `right_pct`% in and vertically centred at face-fraction `center_frac`.
/// Same filter construction as the Seventh frame; empty when no set svg.
fn set_symbol_svg_html(set_svg: Option<&Path>, rarity: &str, center_frac: f64, right_pct: f64) -> String {
    set_svg
        .and_then(|p| fs::read(p).ok())
        .and_then(|bytes| set_symbol_ink(&bytes))
        .map(|(png, ink_aspect)| {
            let uri = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png));
            let aspect = ink_aspect.clamp(0.25, 6.0);
            let (w_px, h_px) = if aspect >= 1.45 {
                let mut w = 250.0_f64;
                let mut h = w / aspect;
                if h > 92.0 { h = 92.0; w = h * aspect; }
                (w, h)
            } else {
                let h = 100.0_f64;
                (h * aspect, h)
            };
            let w_frac = w_px / f64::from(FACE_W) * 100.0;
            let h_frac = h_px / f64::from(FACE_H) * 100.0;
            let top = center_frac * 100.0 - h_frac / 2.0;
            format!(
                r##"<svg class="setsym" style="width:{w_frac:.3}%;height:{h_frac:.3}%;top:{top:.3}%;right:{right_pct}%" viewBox="0 0 {w_px:.1} {h_px:.1}" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
<defs><filter id="ss" x="-30%" y="-30%" width="160%" height="160%" color-interpolation-filters="sRGB">
<feMorphology in="SourceAlpha" operator="dilate" radius="2.2" result="d"/>
<feFlood flood-color="#fbfaf3"/><feComposite in2="d" operator="in" result="key"/>
<feFlood flood-color="{fill}"/><feComposite in2="SourceAlpha" operator="in" result="body"/>
<feFlood flood-color="#000" flood-opacity="1"/><feComposite in2="SourceAlpha" operator="out" result="o"/>
<feGaussianBlur in="o" stdDeviation="3.3" result="ob"/><feComposite in="ob" in2="SourceAlpha" operator="in" result="ir"/>
<feComponentTransfer in="ir" result="inner"><feFuncA type="linear" slope="1.35"/></feComponentTransfer>
<feMerge><feMergeNode in="key"/><feMergeNode in="body"/><feMergeNode in="inner"/></feMerge>
</filter></defs>
<image href="{uri}" width="{w_px:.1}" height="{h_px:.1}" preserveAspectRatio="xMidYMid meet" filter="url(#ss)"/>
</svg>"##,
                fill = rarity_fill(rarity),
            )
        })
        .unwrap_or_default()
}

/// The spell ("adventure" / "prepare") half of a two-part card — (title, mana, type, rules) —
/// read from the cached Scryfall metadata. `front` is the main face's name (what the editor
/// card renders); the other face is the sub-spell. None for normal single-part cards.
fn adventure_face(cache_dir: &Path, card_name: &str, front: &str) -> Option<(String, String, String, String)> {
    let meta = cache_dir.join("art").join(format!("{}__latest.json", sanitize(card_name)));
    let v: Value = serde_json::from_str(&fs::read_to_string(meta).ok()?).ok()?;
    let card = v["data"].as_array()?.first()?;
    if !matches!(card["layout"].as_str().unwrap_or(""), "adventure" | "prepare") {
        return None;
    }
    let faces = card["card_faces"].as_array()?;
    if faces.len() < 2 {
        return None;
    }
    let other = faces.iter().find(|f| f["name"].as_str() != Some(front))?;
    Some((
        other["name"].as_str().unwrap_or_default().to_string(),
        other["mana_cost"].as_str().unwrap_or_default().to_string(),
        other["type_line"].as_str().unwrap_or_default().to_string(),
        other["oracle_text"].as_str().unwrap_or_default().to_string(),
    ))
}

const TEMPLATE_8TH_SPLIT: &str = include_str!("card_template_8th_split.html");

/// The two halves of a SPLIT card (e.g. Bind // Liberate), each (name, mana, type, rules),
/// from the cached card_faces. None for non-split cards.
fn split_faces(cache_dir: &Path, name: &str) -> Option<[(String, String, String, String); 2]> {
    let meta = cache_dir.join("art").join(format!("{}__latest.json", sanitize(name)));
    let v: Value = serde_json::from_str(&fs::read_to_string(meta).ok()?).ok()?;
    let card = v["data"].as_array()?.first()?;
    if card["layout"].as_str() != Some("split") {
        return None;
    }
    let faces = card["card_faces"].as_array()?;
    if faces.len() < 2 {
        return None;
    }
    let face = |i: usize| {
        let f = &faces[i];
        (
            f["name"].as_str().unwrap_or_default().to_string(),
            f["mana_cost"].as_str().unwrap_or_default().to_string(),
            f["type_line"].as_str().unwrap_or_default().to_string(),
            f["oracle_text"].as_str().unwrap_or_default().to_string(),
        )
    };
    Some([face(0), face(1)])
}

/// 8ED colour frame chosen from a mana-cost string's WUBRG pips.
fn frame_for_cost_8th(mana_cost: &str) -> &'static str {
    let mut cols: Vec<char> = Vec::new();
    for ch in mana_cost.chars() {
        if "WUBRG".contains(ch) && !cols.contains(&ch) {
            cols.push(ch);
        }
    }
    match cols.len() {
        0 => "frames8/c.png",
        1 => color_frame_8th(cols[0]),
        _ => "frames8/m.png",
    }
}

/// Art-window aspect of ONE split half. The rotated stage is FACE_H×FACE_W; each of the two
/// halves is FACE_H/2 wide × FACE_W tall, and its art window is 82.4% × 43.48% of that (the
/// split template's `.sh-art`). The crop editor locks each of the two rectangles to this.
fn split_window_aspect() -> f64 {
    (0.824 * f64::from(FACE_H) / 2.0) / (0.4348 * f64::from(FACE_W))
}

/// Read the durable DB art-override JSON for a card (`{}` if none).
fn art_override_json(db: &Connection, id: i64) -> Value {
    db.query_row("SELECT COALESCE(art_override,'') FROM cube_cards WHERE id=?1", params![id], |r| {
        r.get::<_, String>(0)
    })
    .ok()
    .filter(|s| !s.trim().is_empty())
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_else(|| json!({}))
}

/// A 4-tuple `[x,y,w,h]` box from a JSON value's named key (e.g. "box2").
fn box_field(v: &Value, key: &str) -> Option<[f64; 4]> {
    v[key].as_array().filter(|a| a.len() == 4).map(|a| {
        let g = |i: usize| a[i].as_f64().unwrap_or(0.0);
        [g(0), g(1), g(2), g(3)]
    })
}

/// The two half-crop boxes for a split card (fractions of the combined source). The durable
/// DB override wins, then the cache sidecar; absent → the source split 50/50 left|right.
fn split_boxes(db: &Connection, id: i64, card_dir: &Path) -> ([f64; 4], [f64; 4]) {
    let both = |v: &Value| -> Option<([f64; 4], [f64; 4])> {
        Some((box_field(v, "box")?, box_field(v, "box2")?))
    };
    if let Some(b) = both(&art_override_json(db, id)) {
        return b;
    }
    if let Some(b) = read_sidecar(card_dir).as_ref().and_then(both) {
        return b;
    }
    ([0.0, 0.0, 0.5, 1.0], [0.5, 0.0, 0.5, 1.0])
}

/// The combined split-art source image: the durable override's chosen source if pinned, else
/// the newest printing's art_crop (the two-half illustration the halves are sliced from).
fn split_source(db: &Connection, id: i64, name: &str, cache_root: &Path, card_dir: &Path) -> Result<PathBuf> {
    if let Some(rel) = art_override_json(db, id)["rel"].as_str() {
        if let Ok(p) = resolve_source(name, cache_root, card_dir, rel) {
            if p.exists() {
                return Ok(p);
            }
        }
    }
    Ok(scryfall_latest(name, &cache_root.join("art"))?.0)
}

/// Crop the combined split art into the two half-window images `(left, right)` the rotated
/// split layout shows — each an EXACT crop of its saved (or default left|right) box.
fn split_half_arts(db: &Connection, id: i64, name: &str, cache_root: &Path, card_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(card_dir)?;
    let source = split_source(db, id, name, cache_root, card_dir)?;
    let (b1, b2) = split_boxes(db, id, card_dir);
    let left = card_dir.join("split_l.png");
    let right = card_dir.join("split_r.png");
    crop_exact(&source, b1, &left)?;
    crop_exact(&source, b2, &right)?;
    Ok((left, right))
}

fn build_split_html(halves: &[(String, String, String, String); 2], art1_abs: &Path, art2_abs: &Path, assets_dir: &Path) -> String {
    let fonts = assets_dir.join("fonts");
    let f = |p: &str| format!("file://{}", fonts.join(p).display());
    let mana_base = format!("file://{}", assets_dir.join("mana").display());
    let italic_face = if fonts.join("mplantin-italic.ttf").exists() {
        format!("@font-face {{ font-family:'mplantin'; font-style:italic; src:url('{}'); }}", f("mplantin-italic.ttf"))
    } else {
        String::new()
    };
    let frame_uri = |cost: &str| format!("file://{}", assets_dir.join(frame_for_cost_8th(cost)).display());
    let pairs: Vec<(&str, String)> = vec![
        ("MANA_CSS", f("mana.css")), ("MATRIX", f("matrix.ttf")), ("MPLANTIN", f("mplantin.ttf")),
        ("ITALIC_FACE", italic_face),
        ("W", W.to_string()), ("H", H.to_string()), ("FW", FACE_W.to_string()), ("FH", FACE_H.to_string()),
        ("BX", ((W - FACE_W) / 2).to_string()), ("BY", ((H - FACE_H) / 2).to_string()),
        ("ART1", format!("file://{}", art1_abs.display())),
        ("ART2", format!("file://{}", art2_abs.display())),
        ("FRAME1", frame_uri(&halves[0].1)), ("FRAME2", frame_uri(&halves[1].1)),
        ("N1", esc(&halves[0].0)), ("M1", manaify(&halves[0].1, &mana_base)),
        ("T1", esc(&halves[0].2)), ("R1", rules_html(&halves[0].3, "", &mana_base)),
        ("N2", esc(&halves[1].0)), ("M2", manaify(&halves[1].1, &mana_base)),
        ("T2", esc(&halves[1].2)), ("R2", rules_html(&halves[1].3, "", &mana_base)),
    ];
    let mut html = TEMPLATE_8TH_SPLIT.to_string();
    for (k, v) in &pairs {
        html = html.replace(&format!("%%{k}%%"), v);
    }
    html
}

fn compose_split_8th(halves: &[(String, String, String, String); 2], art1_path: &Path, art2_path: &Path, assets_dir: &Path, out: &Path, chrome: &str, tag: &str) -> Result<()> {
    fs::create_dir_all(out.parent().unwrap())?;
    let assets_abs = fs::canonicalize(assets_dir)?;
    let art1_abs = fs::canonicalize(art1_path)?;
    let art2_abs = fs::canonicalize(art2_path)?;
    let html = build_split_html(halves, &art1_abs, &art2_abs, &assets_abs);
    let html_path = std::env::temp_dir().join(format!("mtgbrain-split-{tag}.html"));
    fs::write(&html_path, html)?;
    let status = Command::new(chrome)
        .args([
            "--headless", "--disable-gpu", "--hide-scrollbars", "--no-default-browser-check",
            "--no-first-run", "--force-device-scale-factor=1",
            "--run-all-compositor-stages-before-draw", "--virtual-time-budget=15000",
            &format!("--window-size={W},{H}"), &format!("--screenshot={}", out.display()),
            &format!("file://{}", html_path.display()),
        ])
        .status()
        .with_context(|| format!("running Chrome at {chrome}"))?;
    if !status.success() || !out.exists() {
        bail!("Chrome screenshot failed (split)");
    }
    Ok(())
}

fn build_html_8th(c: &Card, frame_abs: &Path, ptbox_abs: &Path, art_abs: &Path, assets_dir: &Path, art: &Art, set_svg: Option<&Path>, adv: Option<&(String, String, String, String)>) -> String {
    let fonts = assets_dir.join("fonts");
    let f = |p: &str| format!("file://{}", fonts.join(p).display());
    let mana_base = format!("file://{}", assets_dir.join("mana").display());
    let italic_face = if fonts.join("mplantin-italic.ttf").exists() {
        format!("@font-face {{ font-family:'mplantin'; font-style:italic; src:url('{}'); }}", f("mplantin-italic.ttf"))
    } else {
        String::new()
    };
    // Any card with a printed power/toughness gets the P/T box — creatures AND **Vehicles**
    // (which are Artifacts, not creatures, but still have a P/T). Loyalty → planeswalkers.
    let pt = if !c.power.is_empty() || !c.toughness.is_empty() {
        format!(r#"<div class="box pt"><span>{}/{}</span></div>"#, esc(&c.power), esc(&c.toughness))
    } else if !c.loyalty.is_empty() {
        format!(r#"<div class="box pt"><span>{}</span></div>"#, esc(&c.loyalty))
    } else {
        String::new()
    };
    // The P/T frame box only exists for cards that HAVE a P/T (creatures, vehicles) or
    // loyalty — never on an enchantment/instant/sorcery.
    let ptboxdiv = if pt.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="ptbox" style="background-image:url('file://{}')"></div>"#, ptbox_abs.display())
    };
    let credit = if !c.illustrator.trim().is_empty() { c.illustrator.trim() } else { art.artist.trim() };
    let illus = if credit.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="info illus"><span>Illus. {}</span></div>"#, esc(credit))
    };
    let year = if art.year.is_empty() { "2003".to_string() } else { art.year.clone() };
    let (info_ink, info_shadow) = info_ink_8th(frame_file_8th(c));
    // ---- colour indicator: coloured artifacts / manual-identity cards show a colour dot
    // to the LEFT of the type line (same logic + filter as the old frame). ----
    let diam_px = (0.030 * f64::from(FACE_H)) as u32;
    let ind_frames: Vec<PathBuf> = {
        let t = c.type_line.to_lowercase();
        let is_land = t.contains("land");
        let is_artifact = t.contains("artifact");
        let ci = ci_letters(&c.color_identity);
        let letters: Vec<char> = if is_artifact && !is_land {
            // Coloured artifacts now carry their colour in the TWIN frame (title/type bars),
            // not a colour-indicator dot; colourless artifacts have nothing to show.
            vec![]
        } else if c.ci_manual && !ci.is_empty() {
            // A hand-declared identity the frame can't otherwise convey (e.g. a costless gold
            // card). A mono-colour land already shows it through its tinted land frame.
            if is_land && ci.len() == 1 { vec![] } else { ci }
        } else {
            vec![]
        };
        if letters.is_empty() {
            vec![]
        } else if letters.len() >= 3 {
            vec![assets_dir.join("frames8/m.png")]
        } else {
            letters.iter().map(|ch| assets_dir.join(color_frame_8th(*ch))).collect()
        }
    };
    let ind_refs: Vec<&Path> = ind_frames.iter().map(PathBuf::as_path).collect();
    let (colorind, typad) = color_indicator_png(&ind_refs, 256).map_or_else(
        || (String::new(), "0".to_string()),
        |png| {
            let uri = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png));
            let d = f64::from(diam_px);
            let w_frac = d / f64::from(FACE_W) * 100.0;
            let h_frac = d / f64::from(FACE_H) * 100.0;
            let left = 10.4; // just inside the type box's left edge
            let top = (0.591 - d / f64::from(FACE_H) / 2.0) * 100.0; // centred on the type line
            let svg = format!(
                r##"<svg class="colorind" style="left:{left:.2}%;top:{top:.3}%;width:{w_frac:.3}%;height:{h_frac:.3}%" viewBox="0 0 {diam_px} {diam_px}" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
<defs><filter id="ci" x="-30%" y="-30%" width="160%" height="160%" color-interpolation-filters="sRGB">
<feMorphology in="SourceAlpha" operator="dilate" radius="2.2" result="d"/>
<feFlood flood-color="#fbfaf3"/><feComposite in2="d" operator="in" result="key"/>
<feFlood flood-color="#000" flood-opacity="1"/><feComposite in2="SourceAlpha" operator="out" result="o"/>
<feGaussianBlur in="o" stdDeviation="3.3" result="ob"/><feComposite in="ob" in2="SourceAlpha" operator="in" result="ir"/>
<feComponentTransfer in="ir" result="inner"><feFuncA type="linear" slope="1.35"/></feComponentTransfer>
<feMerge><feMergeNode in="key"/><feMergeNode in="SourceGraphic"/><feMergeNode in="inner"/></feMerge>
</filter></defs>
<image href="{uri}" width="{diam_px}" height="{diam_px}" preserveAspectRatio="xMidYMid meet" filter="url(#ci)"/>
</svg>"##
            );
            let pad = ((left / 100.0 + w_frac / 100.0) * f64::from(FACE_W) - 0.10 * f64::from(FACE_W) + 18.0) as i32;
            (svg, format!("{pad}px"))
        },
    );
    // adventure / prepare sub-box: the spell half on the LEFT, creature rules reflow right.
    let (advbox, rules_left, rules_width) = match adv {
        Some((title, mana, ty, rules)) => {
            let html = format!(
                r#"<div class="adv"><div class="adv-head"><span class="adv-title">{}</span><span class="adv-mana">{}</span></div><div class="adv-type">{}</div><div class="adv-rules">{}</div></div>"#,
                esc(title), manaify(mana, &mana_base), esc(ty), rules_html(rules, "", &mana_base),
            );
            (html, "53%".to_string(), "37%".to_string())
        }
        None => (String::new(), "10%".to_string(), "80%".to_string()),
    };
    let pairs: Vec<(&str, String)> = vec![
        ("COLORIND", colorind), ("TYPAD", typad),
        ("ADVBOX", advbox), ("RULESLEFT", rules_left), ("RULESWIDTH", rules_width),
        ("MANA_CSS", f("mana.css")),
        ("MATRIX", f("matrix.ttf")),
        ("MPLANTIN", f("mplantin.ttf")),
        ("ITALIC_FACE", italic_face),
        ("W", W.to_string()), ("H", H.to_string()),
        ("FW", FACE_W.to_string()), ("FH", FACE_H.to_string()),
        ("BX", ((W - FACE_W) / 2).to_string()), ("BY", ((H - FACE_H) / 2).to_string()),
        // Font sizes as fractions of the FACE height (the Seventh frame's px() approach), so
        // 8ED typography scales with the canvas instead of baking in fixed pixels.
        ("TSZ", px(120.0 / f64::from(FACE_H))),  // card name (matrix)
        ("MSZ", px(111.0 / f64::from(FACE_H))),  // mana cost
        ("TYSZ", px(100.0 / f64::from(FACE_H))), // type line (matrix)
        ("RSZ", px(101.0 / f64::from(FACE_H))),  // rules / flavour (mplantin)
        ("PSZ", px(131.0 / f64::from(FACE_H))),  // power / toughness (matrix)
        ("ART", format!("file://{}", art_abs.display())),
        ("FRAME", format!("file://{}", frame_abs.display())),
        ("PTBOXDIV", ptboxdiv),
        ("INFOINK", info_ink.to_string()), ("INFOSHADOW", info_shadow.to_string()),
        ("NAME", esc(&c.name)),
        ("MANA", manaify(&c.mana_cost, &mana_base)),
        ("TYPE", esc(&c.type_line)),
        ("RULES", rules_html(&c.oracle_text, &c.flavor, &mana_base)),
        ("PT", pt),
        // set symbol on the type bar, right edge ~0.90 (right:10%), centred on the type bar
        ("SETSYM", set_symbol_svg_html(set_svg, &c.rarity, 0.589, 10.0)),
        ("ILLUS", illus),
        ("YEAR", year),
    ];
    let mut html = TEMPLATE_8TH.to_string();
    for (k, v) in &pairs {
        html = html.replace(&format!("%%{k}%%"), v);
    }
    html
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn compose_8th(card: &Card, art_path: &Path, artist: &str, year: &str, assets_dir: &Path,
    frame_rel: &Path, ptbox_rel: &Path, out: &Path, chrome: &str, tag: &str, set_svg: Option<&Path>,
    adv: Option<&(String, String, String, String)>) -> Result<()> {
    fs::create_dir_all(out.parent().unwrap())?;
    let assets_abs = fs::canonicalize(assets_dir)?;
    let frame_abs = fs::canonicalize(frame_rel)?;
    let ptbox_abs = fs::canonicalize(ptbox_rel)?;
    let art_abs = fs::canonicalize(art_path)?;
    let set_abs = set_svg.and_then(|p| fs::canonicalize(p).ok());
    let art = Art { path: art_abs.clone(), art_ref: String::new(), artist: artist.to_string(), year: year.to_string() };
    let html = build_html_8th(card, &frame_abs, &ptbox_abs, &art_abs, &assets_abs, &art, set_abs.as_deref(), adv);
    let html_path = std::env::temp_dir().join(format!("mtgbrain-render8-{tag}.html"));
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
    Ok(())
}

/// Render one card from `editor_db` in the EIGHTH-EDITION (modern) frame.
pub fn render_card_8th(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str, id: i64, force: bool, backend: Option<&str>,
) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening editor DB {} (read-only)", editor_db.display()))?;
    let mut card = load_card(&db, id)?;
    // Colour-bearing artifacts get a composited TWIN frame (artifact metal + the card's
    // colour on the title/type bars); everything else uses its self-contained colour/type
    // frame. The colourless artifact, lands, and ordinary colours fall through to frame_file_8th.
    let frame_rel = {
        let t = card.type_line.to_lowercase();
        let cols = frame_colors_8th(&card);
        if t.contains("artifact") && !t.contains("land") && !cols.is_empty() {
            colored_artifact_frame(assets_dir, cache_dir, &cols)
                .unwrap_or_else(|| assets_dir.join(frame_file_8th(&card)))
        } else {
            assets_dir.join(frame_file_8th(&card))
        }
    };
    let ptbox_rel = assets_dir.join(pt_box_8th(&card));
    if !frame_rel.exists() {
        bail!("missing 8th frame asset {} — run `mtgbrain render assets`", frame_rel.display());
    }
    let dir = cache_dir.join("cards8").join(sanitize(&card.name));
    materialize_override(&db, cache_dir, id, &card.name, &dir)?;
    // 8ED back cube: fetch the LATEST high-DPI art, not the oldest printing.
    let art = acquire_art(&card.name, cache_dir, &dir, backend, true)?;
    // SET SYMBOL + FLAVOUR come from the card's EARLIEST printing (its debut set's mark, the
    // original flavour) — like the old-frame path — even though the ART above is the latest
    // high-DPI scan. acquire_art only cached `__latest.json`, so make sure the oldest-first
    // prints metadata (`<name>.json`, what card_set_rarity / card_flavor read) exists first.
    // Editor-DB overrides still win (only fill when empty). The set symbol carries a white
    // keyline so even a black common mark reads on the darkest frame.
    ensure_prints_meta(&card.name, &cache_dir.join("art"));
    if card.flavor.is_empty() && !card.flavor_overridden {
        card.flavor = card_flavor(&card.name, &cache_dir.join("art"));
    }
    if card.set.is_empty() || card.rarity.is_empty() {
        let (set, rarity) = card_set_rarity(&card.name, &cache_dir.join("art"));
        if card.set.is_empty() {
            card.set = set;
        }
        if card.rarity.is_empty() {
            card.rarity = rarity;
        }
    }
    let credit = if card.illustrator.is_empty() { art.artist.clone() } else { card.illustrator.clone() };
    let out = dir.join("card8.png");
    // SPLIT cards (Bind // Liberate) get the rotated two-half layout: each half shows its OWN
    // crop of the combined two-half illustration (the editor's two crop rectangles), not the
    // same art twice. Adventure / set-symbol decoration only applies to the single-face layout.
    let split = split_faces(cache_dir, &card.name);
    let set_svg = if split.is_some() { None } else { set_symbol_svg(&card.set, cache_dir) };
    let adv = if split.is_some() { None } else { adventure_face(cache_dir, &card.name, &card.name) };
    // Content-hash cache (mirrors render_one): the headless-Chrome screenshot is by far the
    // slow step, so skip it when nothing that affects this render changed. The editor revisits
    // a card on every click — without this, each visit re-spawned Chrome, the "slow blink".
    let hash = {
        let canon = json!({
            "name": card.name, "display_name": card.display_name, "mana_cost": card.mana_cost,
            "type": card.type_line, "oracle_text": card.oracle_text, "flavor": card.flavor,
            "power": card.power, "toughness": card.toughness, "loyalty": card.loyalty,
            "set": card.set, "rarity": card.rarity, "errata": card.is_errata, "ci": card.color_identity,
            "art_ref": art.art_ref, "artist": credit, "year": art.year,
            "frame": frame_rel.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            "ptbox": ptbox_rel.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            "override": art_override_json(&db, id).to_string(),
            "split": split.is_some(), "adv": adv.is_some(), "frame_kind": "eighth", "v": 1,
        });
        let mut h = Sha256::new();
        h.update(canon.to_string().as_bytes());
        format!("{:x}", h.finalize())
    };
    let hash_file = dir.join("card8.hash");
    if !force && out.exists() && fs::read_to_string(&hash_file).ok().as_deref() == Some(hash.as_str()) {
        return Ok(out);
    }
    if let Some(halves) = split {
        let (left, right) = split_half_arts(&db, id, &card.name, cache_dir, &dir)?;
        compose_split_8th(&halves, &left, &right, assets_dir, &out, chrome, &format!("8-{id}"))?;
    } else {
        compose_8th(&card, &art.path, &credit, &art.year, assets_dir, &frame_rel, &ptbox_rel, &out, chrome, &format!("8-{id}"), set_svg.as_deref(), adv.as_ref())?;
    }
    let _ = fs::write(&hash_file, &hash);
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
pub fn render_all(
    editor_db: &Path, assets_dir: &Path, cache_dir: &Path, chrome: &str,
    eighth: bool, foil: bool, force: bool, backend: Option<&str>,
) -> Result<()> {
    let ids: Vec<i64> = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut stmt = db.prepare("SELECT id FROM cube_cards WHERE in_db_found=1 AND removed=0 ORDER BY id")?;
        let mut v = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            v.push(r.get::<_, i64>(0)?);
        }
        v
    };
    let total = ids.len();
    for (i, id) in ids.iter().enumerate() {
        // The modern (8th) frame has its own pipeline (no foil); everything else is render_one.
        let res = if eighth {
            render_card_8th(editor_db, assets_dir, cache_dir, chrome, *id, force, backend)
        } else {
            render_one(editor_db, assets_dir, cache_dir, chrome, *id, false, force, backend)
        };
        match res {
            Ok(_) => print!("\r[{}/{total}] id {id}        ", i + 1),
            Err(e) => eprintln!("\n  id {id}: {e}"),
        }
        if foil && !eighth {
            let _ = render_one(editor_db, assets_dir, cache_dir, chrome, *id, true, force, backend);
        }
        std::io::stdout().flush().ok();
    }
    let sub = if eighth { "cards8" } else { "cards" };
    println!("\ndone: {total} cards -> {}", cache_dir.join(sub).display());
    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>().trim_matches('_').to_string()
}

// ---------------------------------------------------------------------------
// selfhost — Scryfall-format derivative of the forefront MPC render
// ---------------------------------------------------------------------------

/// Scryfall's `png` image size (what cards.scryfall.io serves at /png/).
const SELFHOST_W: u32 = 745;
const SELFHOST_H: u32 = 1040;
/// MPC bleed per side as a fraction of the full canvas (the face is centered).
const BLEED_FRAC_X: f64 = (W - FACE_W) as f64 / (2.0 * W as f64); // 88/2176
const BLEED_FRAC_Y: f64 = (H - FACE_H) as f64 / (2.0 * H as f64); // 80/2960
/// Card corner radius as a fraction of card width (≈2.5 mm on a 63 mm card).
const CORNER_FRAC: f64 = 0.0397;

/// The forefront (latest) MPC render in a card dir: the most-recently-written
/// top-level `<hash>.png`, excluding the art crop and any prior selfhost output.
fn forefront_render(card_dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(card_dir).ok()?.flatten() {
        let p = entry.path();
        if !p.is_file() || p.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        match p.file_name().and_then(|n| n.to_str()) {
            Some("art.png" | "selfhost.png") | None => continue,
            _ => {}
        }
        let Ok(m) = entry.metadata().and_then(|md| md.modified()) else { continue };
        if best.as_ref().is_none_or(|(bm, _)| m > *bm) {
            best = Some((m, p));
        }
    }
    best.map(|(_, p)| p)
}

/// Zero the alpha outside a rounded rectangle (1px coverage AA on the arc).
fn round_corners(img: &mut image::RgbaImage, frac: f64) {
    let (w, h) = (img.width(), img.height());
    let r = (frac * w as f64).round() as i32;
    if r <= 0 {
        return;
    }
    let rf = r as f64;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            // Snap to the nearest corner-arc center; if either axis is in the
            // straight middle, that axis contributes 0 (edges stay square).
            let cx = if x < r { r } else if x >= w as i32 - r { w as i32 - 1 - r } else { x };
            let cy = if y < r { r } else if y >= h as i32 - r { h as i32 - 1 - r } else { y };
            if cx == x || cy == y {
                continue; // on a straight edge/interior, not a corner arc
            }
            let d = (((x - cx) as f64).powi(2) + ((y - cy) as f64).powi(2)).sqrt();
            if d > rf {
                img.get_pixel_mut(x as u32, y as u32)[3] = 0;
            } else if d > rf - 1.0 {
                let px = img.get_pixel_mut(x as u32, y as u32);
                px[3] = (f64::from(px[3]) * (rf - d).clamp(0.0, 1.0)) as u8;
            }
        }
    }
}

/// Build `cards/<Name>/selfhost.png` from the card's forefront MPC render: crop
/// the printable face out of the bleed canvas, downscale to Scryfall's 745×1040,
/// and round the corners (transparent outside the radius). `Ok(None)` if the card
/// has no render yet.
pub fn selfhost_one(card_dir: &Path) -> Result<Option<PathBuf>> {
    let Some(src) = forefront_render(card_dir) else {
        return Ok(None);
    };
    let img = image::open(&src).with_context(|| format!("opening {}", src.display()))?;
    let (w, h) = (img.width(), img.height());
    let cx = (f64::from(w) * BLEED_FRAC_X).round() as u32;
    let cy = (f64::from(h) * BLEED_FRAC_Y).round() as u32;
    let face = img.crop_imm(cx, cy, w.saturating_sub(2 * cx), h.saturating_sub(2 * cy));
    let resized = face.resize_exact(SELFHOST_W, SELFHOST_H, image::imageops::FilterType::Lanczos3);
    let mut rgba = resized.to_rgba8();
    round_corners(&mut rgba, CORNER_FRAC);
    let out = card_dir.join("selfhost.png");
    rgba.save(&out).with_context(|| format!("saving {}", out.display()))?;
    Ok(Some(out))
}

/// Generate the selfhost image for every active card in the editor DB. Reports
/// how many were written and which cards still lack a render.
pub fn render_selfhost(editor_db: &Path, cache_dir: &Path, only: Option<i64>) -> Result<()> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening {}", editor_db.display()))?;
    let names: Vec<String> = if let Some(id) = only {
        // Just one card by editor-DB id (e.g. after updating only that card).
        let name: String = db
            .query_row("SELECT name FROM cube_cards WHERE id=?1 AND removed=0", params![id], |r| r.get(0))
            .with_context(|| format!("no active card with id {id}"))?;
        vec![name]
    } else {
        let mut stmt = db.prepare("SELECT name FROM cube_cards WHERE removed=0 ORDER BY id")?;
        let v = stmt.query_map([], |r| r.get(0))?.collect::<std::result::Result<_, _>>()?;
        v
    };
    let (mut done, mut missing) = (0usize, Vec::new());
    for name in &names {
        let dir = cache_dir.join("cards").join(sanitize(name));
        match selfhost_one(&dir)? {
            Some(_) => done += 1,
            None => missing.push(name.clone()),
        }
    }
    println!("selfhost: {done} written, {} missing a render", missing.len());
    for m in &missing {
        println!("  (no render) {m}");
    }
    Ok(())
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
