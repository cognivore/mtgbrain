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

#[allow(clippy::struct_field_names)]
struct Art {
    path: PathBuf,
    art_ref: String,
    artist: String,
    year: String,
}

/// Acquire art: oldest Scryfall printing as reference+fallback (+artist/year); if a
/// MPC-Autofill `backend` and `ANTHROPIC_API_KEY` are present, pull all high-DPI
/// candidates (cached per bucket), pick the highest-DPI one whose illustration
/// Claude confirms matches the old art, and crop its art window cleanly.
fn acquire_art(name: &str, cache_root: &Path, card_dir: &Path, backend: Option<&str>) -> Result<Art> {
    let art_dir = cache_root.join("art");
    fs::create_dir_all(&art_dir)?;
    fs::create_dir_all(card_dir)?;

    // FAST PATH: once the MPCfill crop is chosen, it (and its metadata) is cached
    // in art.json next to the crop. Re-render reuses it with NO network/Claude
    // roundtrip (text/frame can change freely; the art is settled). Delete
    // <card>/art.png + art.json to force re-picking the art.
    let sidecar = card_dir.join("art.json");
    let crop = card_dir.join("art.png");
    if sidecar.exists() && crop.exists() {
        if let Ok(v) = serde_json::from_str::<Value>(&fs::read_to_string(&sidecar)?) {
            return Ok(Art {
                path: crop,
                art_ref: v["art_ref"].as_str().unwrap_or("mpcfill:cached").to_string(),
                artist: v["artist"].as_str().unwrap_or("").to_string(),
                year: v["year"].as_str().unwrap_or("2001").to_string(),
            });
        }
    }

    let (ref_path, artist, year, set) = scryfall_oldest(name, &art_dir)?;
    let key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty());
    if let (Some(base), Some(key)) = (backend, key.as_deref()) {
        match mpcfill_pick(name, base, cache_root, card_dir, &ref_path, key) {
            Ok(Some((path, art_ref))) => {
                let _ = fs::write(
                    &sidecar,
                    json!({"art_ref": art_ref, "artist": artist, "year": year}).to_string(),
                );
                return Ok(Art { path, art_ref, artist, year });
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

/// Pull + cache all candidates, pick the best art-matching one, crop it.
fn mpcfill_pick(
    name: &str,
    base: &str,
    cache_root: &Path,
    card_dir: &Path,
    ref_path: &Path,
    key: &str,
) -> Result<Option<(PathBuf, String)>> {
    let base = base.trim_end_matches('/');
    let sources = mpcfill_sources(base, cache_root)?;
    // editorSearch
    let search = format!(
        r#"{{"searchSettings":{{"searchTypeSettings":{{"fuzzySearch":true,"filterCardbacks":false}},"sourceSettings":{{"sources":{sources}}},"filterSettings":{{"minimumDPI":300,"maximumDPI":1500,"maximumSize":50,"languages":[],"includesTags":[],"excludesTags":["NSFW"]}}}},"queries":[{{"query":{q},"cardType":"CARD"}}]}}"#,
        q = serde_json::to_string(name)?
    );
    let resp = curl_post_json(&format!("{base}/2/editorSearch/"), &search)?;
    let v: Value = serde_json::from_str(&resp).context("editorSearch response")?;
    let ids: Vec<String> = v["results"][name]["CARD"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(None);
    }
    // resolve metadata
    let cards_req = json!({ "cardIdentifiers": ids }).to_string();
    let resp = curl_post_json(&format!("{base}/2/cards/"), &cards_req)?;
    let cv: Value = serde_json::from_str(&resp).context("cards response")?;
    let mut cands: Vec<McCand> = cv["results"]
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

    // Cache EVERY download under cards/<Card>/mpcfill/<bucket>/<id>.<ext> (never re-fetch).
    for c in &cands {
        let dir = card_dir.join("mpcfill").join(sanitize_bucket(&c.bucket));
        fs::create_dir_all(&dir)?;
        let dst = dir.join(format!("{}.{}", c.identifier, c.ext));
        if !dst.exists() && !c.link.is_empty() && curl_to_file(&c.link, &dst).is_err() {
            eprintln!("    (download failed) {}", c.identifier);
        }
        record_bucket(&c.bucket, &c.ext_link)?;
    }

    // Prefer plain prints (skip extended/full-art/showcase/etc), then highest DPI.
    let banned = ["extended", "full art", "fullart", "textless", "showcase", "borderless", "alt"];
    cands.sort_by(|a, b| {
        let pa = banned.iter().any(|w| a.label.to_lowercase().contains(w));
        let pb = banned.iter().any(|w| b.label.to_lowercase().contains(w));
        pa.cmp(&pb).then(b.dpi.cmp(&a.dpi))
    });

    // Claude gives the crop box per candidate + whether the art matches the old
    // reference. Prefer a confirmed art-match; if none confirm, still use the best
    // (highest-DPI) candidate — never throw away high-DPI art for low-res Scryfall.
    let out = card_dir.join("art.png");
    let mut best_box: Option<(usize, [f64; 4])> = None;
    for (i, c) in cands.iter().take(3).enumerate() {
        let file = card_dir
            .join("mpcfill")
            .join(sanitize_bucket(&c.bucket))
            .join(format!("{}.{}", c.identifier, c.ext));
        if !file.exists() {
            continue;
        }
        match claude_match_and_box(key, ref_path, &file) {
            Ok((same, bx)) => {
                if same {
                    crop_to(&file, bx, &out)?;
                    return Ok(Some((out, format!("mpcfill:{}:dpi{}", c.identifier, c.dpi))));
                }
                best_box.get_or_insert((i, bx));
            }
            Err(e) => eprintln!("    (vision error) {e}"),
        }
    }
    // No confirmed match — use the top candidate with the box we already got.
    if let Some((i, bx)) = best_box {
        let c = &cands[i];
        let file = card_dir
            .join("mpcfill")
            .join(sanitize_bucket(&c.bucket))
            .join(format!("{}.{}", c.identifier, c.ext));
        crop_to(&file, bx, &out)?;
        return Ok(Some((out, format!("mpcfill:{}:dpi{}:unconfirmed", c.identifier, c.dpi))));
    }
    Ok(None)
}

/// Ask Claude: does candidate's illustration match the reference art? + art box.
fn claude_match_and_box(key: &str, ref_path: &Path, cand: &Path) -> Result<(bool, [f64; 4])> {
    let ref_b64 = img_b64_small(ref_path, 700)?;
    let cand_b64 = img_b64_small(cand, 1024)?;
    let prompt = "Image 1 is reference art from an older printing. Image 2 is a full proxy card. \
        Return ONLY compact JSON: {\"same_artwork\":true|false,\"art_box\":{\"x\":..,\"y\":..,\"w\":..,\"h\":..}}. \
        same_artwork = whether image 2's illustration depicts the SAME painting as image 1. \
        art_box = image 2's illustration window (painted art only, exclude frame/title/text/border) \
        as FRACTIONS of image 2 (0..1), precise to the art's inner edge.";
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
fn crop_to(path: &Path, bx: [f64; 4], out: &Path) -> Result<()> {
    let img = image::open(path).with_context(|| format!("opening {}", path.display()))?;
    let (iw, ih) = (f64::from(img.width()), f64::from(img.height()));
    let x = (bx[0] * iw).clamp(0.0, iw - 1.0) as u32;
    let y = (bx[1] * ih).clamp(0.0, ih - 1.0) as u32;
    let w = (bx[2] * iw).clamp(1.0, iw - f64::from(x)) as u32;
    let h = (bx[3] * ih).clamp(1.0, ih - f64::from(y)) as u32;
    img.crop_imm(x, y, w, h).save(out).with_context(|| format!("saving {}", out.display()))?;
    Ok(())
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
// GenAI art (Claude art-direction -> OpenAI gpt-image-1, in chosen-artist styles)
// ---------------------------------------------------------------------------

/// Late-90s / early-00s Magic painters whose styles we offer per card.
/// (artist name, style descriptor). The descriptor is a no-name fallback for
/// when OpenAI's safety system rejects "in the style of <living artist>".
pub const ARTISTS: &[(&str, &str)] = &[
    ("John Avon", "luminous panoramic fantasy landscapes with soft atmospheric light"),
    ("Christopher Rush", "classic early-1990s fantasy illustration, bold and iconic"),
    ("Brom", "dark gothic oil-painted fantasy, moody and edgy"),
    ("Rob Alexander", "atmospheric painted fantasy landscapes and architecture"),
    ("Greg Staples", "bold dynamic comic-influenced fantasy painting"),
    ("Donato Giancola", "classical realistic oil painting, Renaissance fine-art fantasy"),
    ("Wayne Reynolds", "dynamic action-packed ink-and-paint fantasy with energetic linework"),
    ("Rebecca Guay", "ethereal art-nouveau watercolor fantasy, flowing organic lines, soft luminous palette"),
];

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
    claude_text(key, &prompt)
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
    let body = json!({"model":"gpt-image-1","n":1,"size":"1536x1024","prompt":prompt});
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
    for (artist, descriptor) in ARTISTS {
        // Stable per (artist, direction) regardless of which prompt variant succeeds.
        let h = short_hash(&format!("{artist}:{direction}"));
        let adir = genai_dir.join(sanitize_bucket(artist));
        let art_png = adir.join(format!("{h}.png"));
        if !art_png.exists() {
            let Some(k) = &openai else {
                errors.push(format!("{artist}: OPENAI_API_KEY not set"));
                continue;
            };
            let named = format!(
                "You are making Magic: the Gathering art from the late 1990s in the style of {artist}. \
                 Paint it ALLA PRIMA as a loose painterly oil study (an ébauche / working lay-in): \
                 confident economical brushwork, broad masses and suggested forms, FEWER details — \
                 a final draft in real paint, NOT a tight, glossy, fully-rendered gallery piece. \
                 You need to make a card art for the following art direction:\n{direction}"
            );
            let mut res = gen_art_openai(&named, &art_png, k);
            // OpenAI rejects some living-artist names — retry with the style descriptor only.
            if res.as_ref().err().is_some_and(|e| e.to_string().contains("safety")) {
                let styled = format!(
                    "You are making Magic: the Gathering art from the late 1990s, {descriptor}. \
                     Paint it ALLA PRIMA as a loose painterly oil study (an ébauche / working lay-in): \
                     confident economical brushwork, broad masses and suggested forms, FEWER details — \
                     a final draft in real paint, NOT a tight, glossy, fully-rendered gallery piece. \
                     You need to make a card art for the following art direction:\n{direction}"
                );
                res = gen_art_openai(&styled, &art_png, k);
            }
            if let Err(e) = res {
                let msg = e.to_string();
                log_event(cache_dir, id, &card.name, crate::events::ART_FAILED, Some(artist), None, Some(&direction), Some(&msg));
                errors.push(format!("{artist}: {msg}"));
                continue;
            }
            log_event(cache_dir, id, &card.name, crate::events::ART_GENERATED, Some(artist), Some(&h), Some(&direction), None);
        }
        let card_png = adir.join("card.png");
        if let Err(e) = compose(
            &card, &art_png, artist, &year, assets_dir, &frame_rel, false, &card_png, chrome,
            &format!("genai-{id}-{}", sanitize_bucket(artist)),
        ) {
            errors.push(format!("{artist} (render): {e}"));
            continue;
        }
        options.push(json!({"artist": artist, "hash": h}));
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
    fs::copy(&art, card_dir.join("art.png"))?;
    fs::write(
        card_dir.join("art.json"),
        json!({"art_ref": format!("genai:{artist}:{hash}"), "artist": artist, "year": year}).to_string(),
    )?;
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
    log_event(cache_dir, id, &card.name, crate::events::UNCHOSEN, None, None, None, None);
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
    Ok(json!({"cards": cards, "artists": ARTISTS.iter().map(|(a, _)| *a).collect::<Vec<_>>()}))
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
        for (artist, _) in ARTISTS {
            let h = short_hash(&format!("{artist}:{direction}"));
            if genai_dir.join(sanitize_bucket(artist)).join(format!("{h}.png")).exists() {
                options.push(json!({"artist": artist, "hash": h}));
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
            "SELECT id, name FROM cube_cards WHERE genai_art=1 AND removed=0 AND in_db_found=1 ORDER BY id",
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
    for (i, (id, name)) in ids.iter().enumerate() {
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

fn build_html(c: &Card, frame_abs: &Path, art_abs: &Path, assets_dir: &Path, art: &Art, foil: bool) -> String {
    let fonts = assets_dir.join("fonts");
    let f = |p: &str| format!("file://{}", fonts.join(p).display());
    let mana_base = format!("file://{}", assets_dir.join("mana").display());
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
        ("AX", pc(0.12)), ("AY", pc(0.0991)), ("AW", pc(0.7667)), ("AH", pc(0.4429)),
        ("TX", pc(0.1134)), ("TY", pc(0.0481)), ("TW", pc(0.7734)), ("TH", pc(0.041)),
        // Pips share the TITLE's vertical band (MAY=TY, MAH=TH) and are centred in it,
        // so pip-centre == title-centre. (cardconjurer's literal mana y=0.0539 is offset
        // back up by its textSize*0.34 baseline math, which we don't replicate — using it
        // raw dropped the pips a full box too low.)
        ("MAX", pc(0.1067)), ("MAY", pc(0.0451)), ("MAW", pc(0.8174)), ("MAH", px(0.041)),
        ("TSZ", px(0.041)), ("MSZ", px(72.0 / 1638.0)),
        ("SHX", px(0.002 * f64::from(FACE_W) / f64::from(FACE_H))), ("SHY", px(0.0015)),
        ("TYX", pc(0.1074)), ("TYY", pc(0.5486)), ("TYW", pc(0.7852)), ("TYH", pc(0.0543)), ("TYSZ", px(0.032)),
        ("RX", pc(0.128)), ("RY", pc(0.6067)), ("RW", pc(0.744)), ("RH", pc(0.2724)), ("RSZ", px(0.0358)),
        ("PX", pc(0.8074)), ("PY", pc(0.9043)), ("PW", pc(0.1367)), ("PSZ", px(0.0429)),
        ("IY", pc(1908.0 / 2100.0)), ("ISZ", px(0.0172)),
        ("LY", pc(1940.0 / 2100.0)), ("LSZ", px(0.0143)),
    ];
    let content: [(&str, String); 9] = [
        ("ART", format!("file://{}", art_abs.display())),
        ("FRAME", format!("file://{}", frame_abs.display())),
        ("NAME", esc(&c.name)),
        ("MANA", manaify(&c.mana_cost, &mana_base)),
        ("TYPE", esc(&c.type_line)),
        ("RULES", rules_html(&c.oracle_text, &mana_base)),
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
fn rules_html(text: &str, mana_base: &str) -> String {
    let mut s = String::new();
    for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
        s.push_str("<p>");
        s.push_str(&reminder_italic(&manaify(line, mana_base)));
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
    let card = load_card(&db, id)?;

    let frame_rel = assets_dir.join(frame_file(&card));
    if !frame_rel.exists() {
        bail!("missing frame asset {} — run `mtgbrain render assets`", frame_rel.display());
    }
    let dir = cache_dir.join("cards").join(sanitize(&card.name));
    let art = acquire_art(&card.name, cache_dir, &dir, backend)?;
    let hash = card_hash(&card, &art.art_ref, &art.artist);

    let out = if foil { dir.join("foil").join(format!("{hash}.png")) } else { dir.join(format!("{hash}.png")) };
    if out.exists() && !force {
        return Ok(out);
    }
    compose(&card, &art.path, &art.artist, &art.year, assets_dir, &frame_rel, foil, &out, chrome,
        &format!("{id}{}", u8::from(foil)))?;
    Ok(out)
}

/// Composite a full card: frame + given art + text, screenshot to `out`.
#[allow(clippy::too_many_arguments)]
fn compose(card: &Card, art_path: &Path, artist: &str, year: &str, assets_dir: &Path,
    frame_rel: &Path, foil: bool, out: &Path, chrome: &str, tag: &str) -> Result<()> {
    fs::create_dir_all(out.parent().unwrap())?;
    let assets_abs = fs::canonicalize(assets_dir)?;
    let frame_abs = fs::canonicalize(frame_rel)?;
    let art_abs = fs::canonicalize(art_path)?;
    let art = Art { path: art_abs.clone(), art_ref: String::new(), artist: artist.to_string(), year: year.to_string() };
    let html = build_html(card, &frame_abs, &art_abs, &assets_abs, &art, foil);
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
