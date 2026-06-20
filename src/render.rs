//! Old-frame card renderer → print-ready MPC PNGs.
//!
//! Pipeline (all headless, no Photoshop): build an HTML document that lays the
//! Alpha/Beta/Unlimited ("ABU") old-frame art (from cardconjurer's flat PNG
//! assets) under typeset title/type/rules/PT text positioned by cardconjurer's
//! exact ABU bounds, then screenshot it with headless Google Chrome at the MPC
//! 800-DPI-with-bleed target (2176×2960). The card content is read from the
//! editor DB with per-field overrides applied (read-only — we never write it).
//!
//! Caching: the render-relevant fields are canonicalised (sorted keys via
//! serde_json's BTreeMap-backed Value, sorted arrays) → SHA-256 → the image is
//! cached at `cards/<Card Name>/<hash>.png` and the foil variant at
//! `cards/<Card Name>/foil/<hash>.png`. A stale hash (any field changed) simply
//! produces a new filename, so a mismatch is self-evidently a regenerate.
//!
//! Art source is pluggable: an MPC-Autofill community backend (high-DPI art off
//! Google Drive) when `--art-backend <url>` is given, else Scryfall `art_crop`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// cardconjurer raw asset root (frame art + old fonts + foil overlay).
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

/// Files we pull into `<assets_dir>`: (remote, local relative path).
fn asset_list() -> Vec<(String, &'static str)> {
    let mut v: Vec<(String, &'static str)> = Vec::new();
    // ABU old frames (one per colour + artifact + land + coloured lands)
    for (letter, local) in [
        ("w", "frames/w.png"),
        ("u", "frames/u.png"),
        ("b", "frames/b.png"),
        ("r", "frames/r.png"),
        ("g", "frames/g.png"),
        ("a", "frames/a.png"),
        ("l", "frames/l.png"),
        ("wl", "frames/wl.png"),
        ("ul", "frames/ul.png"),
        ("bl", "frames/bl.png"),
        ("rl", "frames/rl.png"),
        ("gl", "frames/gl.png"),
    ] {
        v.push((format!("{CC}/img/frames/old/abu/{letter}.png"), local));
    }
    v.push((format!("{CC}/img/frames/effects/foil.png"), "foil/sheen.png"));
    v.push((format!("{CC}/img/frames/seventh/foilStar.svg"), "foil/star.svg"));
    // old fonts (proprietary — kept out of git; personal-use only)
    v.push((format!("{CC}/fonts/goudy-medieval.ttf"), "fonts/goudy-medieval.ttf"));
    v.push((format!("{CC}/fonts/mplantin.ttf"), "fonts/mplantin.ttf"));
    v.push((format!("{CC}/fonts/matrix.ttf"), "fonts/matrix.ttf"));
    // mana font (OFL)
    v.push((format!("{MANA}/fonts/mana.ttf"), "fonts/mana.ttf"));
    v.push((format!("{MANA}/css/mana.css"), "fonts/mana.css"));
    v
}

/// Download all render assets into `assets_dir` (idempotent unless `force`).
pub fn assets(assets_dir: &Path, force: bool) -> Result<()> {
    let mut got = 0u32;
    let mut skipped = 0u32;
    for (url, rel) in asset_list() {
        let dst = assets_dir.join(rel);
        if dst.exists() && !force {
            skipped += 1;
            continue;
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        curl_to_file(&url, &dst)?;
        got += 1;
        println!("  ↓ {rel}");
    }
    // Rewrite mana.css font URL to our local mana.ttf so it resolves offline.
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
    colors: String,      // e.g. "U" or "U, R"
    is_creature: bool,
}

type RawRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    String,
);

fn load_card(db: &Connection, id: i64) -> Result<Card> {
    let (name, mc, tl, ot, p, t, l, colors, is_cr, ov): RawRow = db
        .query_row(
            "SELECT name,mana_cost,type,oracle_text,power,toughness,loyalty,colors,
                    is_creature,overrides FROM cube_cards WHERE id=?1",
            params![id],
            |r| {
                Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                    r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                ))
            },
        )
        .with_context(|| format!("no card id {id} in editor DB"))?;
    let o: Value = serde_json::from_str(&ov).unwrap_or_else(|_| json!({}));
    let pick = |key: &str, base: Option<String>| -> String {
        o.get(key)
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or(base)
            .unwrap_or_default()
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

/// Canonical SHA-256 over the render-relevant fields (sorted keys + sorted
/// colour array). Foil shares the base hash; the foil flag lives in the path.
fn card_hash(c: &Card, art_ref: &str) -> String {
    let mut colors: Vec<String> = c
        .colors
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    colors.sort();
    // serde_json Value::Object is BTreeMap-backed → keys serialise sorted.
    let canon = json!({
        "name": c.name,
        "mana_cost": c.mana_cost,
        "type": c.type_line,
        "oracle_text": c.oracle_text,
        "power": c.power,
        "toughness": c.toughness,
        "loyalty": c.loyalty,
        "colors": colors,
        "is_creature": c.is_creature,
        "art_ref": art_ref,
        "frame": "abu",
        "v": 1,
    });
    let mut h = Sha256::new();
    h.update(canon.to_string().as_bytes());
    format!("{:x}", h.finalize())
}

// ---------------------------------------------------------------------------
// frame + colour selection
// ---------------------------------------------------------------------------

fn frame_file(c: &Card) -> &'static str {
    let t = c.type_line.to_lowercase();
    let is_land = t.contains("land");
    let is_artifact = t.contains("artifact");
    let cols: Vec<char> = c
        .colors
        .chars()
        .filter(|ch| "WUBRG".contains(*ch))
        .collect();
    if is_land {
        return match cols.first() {
            Some('W') => "frames/wl.png",
            Some('U') => "frames/ul.png",
            Some('B') => "frames/bl.png",
            Some('R') => "frames/rl.png",
            Some('G') => "frames/gl.png",
            _ => "frames/l.png",
        };
    }
    let _ = is_artifact; // colourless (artifact or not) uses the ABU artifact frame
    if cols.is_empty() {
        return "frames/a.png";
    }
    // Mono → that colour; multi → dominant (first) colour (ABU pack has no gold).
    match cols[0] {
        'W' => "frames/w.png",
        'U' => "frames/u.png",
        'B' => "frames/b.png",
        'R' => "frames/r.png",
        _ => "frames/g.png",
    }
}

// ---------------------------------------------------------------------------
// art acquisition (pluggable)
// ---------------------------------------------------------------------------

/// Resolve+download art for `name` into the art cache, returning (local_path, art_ref).
/// `art_ref` is a stable identity string folded into the content hash.
fn acquire_art(
    name: &str,
    art_dir: &Path,
    backend: Option<&str>,
    dpi_target: u32,
) -> Result<(PathBuf, String)> {
    fs::create_dir_all(art_dir)?;
    // 1) MPC-Autofill community backend (high-DPI off Google Drive), if configured.
    if let Some(base) = backend {
        match mpcfill_art(name, base, art_dir, dpi_target) {
            Ok(Some(hit)) => return Ok(hit),
            Ok(None) => eprintln!("  art: no MPC-Autofill hit for {name:?}, falling back to Scryfall"),
            Err(e) => eprintln!("  art: MPC-Autofill error ({e}); falling back to Scryfall"),
        }
    }
    // 2) Scryfall art_crop (reliable fallback).
    let dst = art_dir.join(format!("{}.jpg", sanitize(name)));
    let url = format!(
        "https://api.scryfall.com/cards/named?format=image&version=art_crop&exact={}",
        percent(name)
    );
    curl_to_file(&url, &dst)?;
    Ok((dst, format!("scryfall:art_crop:{name}")))
}

/// MPC-Autofill: search the community backend for the highest-DPI image of `name`,
/// then download it from Google Drive. Returns None if nothing usable is found.
fn mpcfill_art(
    name: &str,
    base: &str,
    art_dir: &Path,
    _dpi_target: u32,
) -> Result<Option<(PathBuf, String)>> {
    let base = base.trim_end_matches('/');
    // editorSearch: minimal valid payload (one query, permissive settings).
    let search = json!({
        "searchSettings": {
            "searchTypeSettings": {"fuzzySearch": false, "filterCardbacks": false},
            "sourceSettings": {"sources": null},
            "filterSettings": {
                "minimumDPI": 0, "maximumDPI": 1500, "maximumSize": 30,
                "languages": [], "includesTags": [], "excludesTags": ["NSFW"]
            }
        },
        "queries": {"q": {"query": name, "cardType": "CARD"}}
    });
    let resp = curl_post_json(&format!("{base}/2/editorSearch/"), &search.to_string())?;
    let v: Value = serde_json::from_str(&resp).context("parsing editorSearch response")?;
    let ids: Vec<String> = v["results"]["q"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(None);
    }
    // cards: resolve identifiers → metadata incl. per-image dpi; pick the max.
    let cards_req = json!({ "cardIdentifiers": ids });
    let resp = curl_post_json(&format!("{base}/2/cards/"), &cards_req.to_string())?;
    let cv: Value = serde_json::from_str(&resp).context("parsing cards response")?;
    let results = cv["results"].as_object().cloned().unwrap_or_default();
    let best = results
        .values()
        .filter_map(|c| {
            let dpi = c.get("dpi").and_then(Value::as_i64).unwrap_or(0);
            let ident = c.get("identifier").and_then(Value::as_str)?;
            Some((dpi, ident.to_string()))
        })
        .max_by_key(|(dpi, _)| *dpi);
    let Some((dpi, ident)) = best else {
        return Ok(None);
    };
    // Download from Drive. No-auth path: the thumbnail endpoint at a large width.
    // (True full-res needs a Drive service account; thumbnails cap ~w1600.)
    let dst = art_dir.join(format!("{}-mpc.png", sanitize(name)));
    let url = format!("https://drive.google.com/thumbnail?id={ident}&sz=w1600");
    curl_to_file(&url, &dst)?;
    Ok(Some((dst, format!("mpcfill:{ident}:dpi{dpi}"))))
}

fn curl_post_json(url: &str, body: &str) -> Result<String> {
    let out = Command::new("curl")
        .args(["-sSL", "--fail", "--max-time", "30", "-H", "Content-Type: application/json", "-X", "POST", "-d", body, url])
        .output()
        .context("curl POST")?;
    if !out.status.success() {
        bail!("POST {url} failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ---------------------------------------------------------------------------
// HTML template (ABU bounds, as fractions of the FACE)
// ---------------------------------------------------------------------------

fn build_html(c: &Card, frame_abs: &Path, art_abs: &Path, assets_dir: &Path, foil: bool) -> String {
    let fonts = assets_dir.join("fonts");
    let mana_css = fonts.join("mana.css");
    let pt = if c.is_creature && (!c.power.is_empty() || !c.toughness.is_empty()) {
        format!(
            r#"<div class="pt">{}/{}</div>"#,
            esc(&c.power),
            esc(&c.toughness)
        )
    } else if !c.loyalty.is_empty() {
        format!(r#"<div class="pt">{}</div>"#, esc(&c.loyalty))
    } else {
        String::new()
    };
    let foil_layer = if foil {
        let star = assets_dir.join("foil/star.svg");
        let sheen = assets_dir.join("foil/sheen.png");
        format!(
            r#"<div class="foil sheen" style="background-image:url('file://{}')"></div>
               <div class="foil stars" style="background-image:url('file://{}')"></div>"#,
            sheen.display(),
            star.display()
        )
    } else {
        String::new()
    };

    format!(
        r#"<!doctype html><html><head><meta charset="utf-8">
<link rel="stylesheet" href="file://{mana_css}">
<style>
@font-face {{ font-family:'goudy'; src:url('file://{goudy}'); }}
@font-face {{ font-family:'mplantin'; src:url('file://{mplantin}'); }}
* {{ margin:0; padding:0; box-sizing:border-box; }}
html,body {{ width:{W}px; height:{H}px; background:#000; overflow:hidden; }}
.face {{ position:absolute; left:{bx}px; top:{by}px; width:{fw}px; height:{fh}px; }}
.layer {{ position:absolute; left:0; top:0; width:100%; height:100%; }}
.art {{ position:absolute; left:11.6%; top:10.43%; width:76.54%; height:44.1%;
        background-size:cover; background-position:center; }}
.frame {{ background-size:100% 100%; }}
.box {{ position:absolute; color:#0c0c0c; }}
.title {{ left:7%; top:4.4%; width:86.67%; height:5%;
          font-family:'goudy'; font-size:108px; line-height:120px; white-space:nowrap;
          display:flex; align-items:center; justify-content:space-between; }}
.title .nm {{ overflow:hidden; text-overflow:ellipsis; }}
.title .mana {{ flex:none; white-space:nowrap; font-size:96px; }}
.type {{ left:8%; top:55.5%; width:80%; height:4%;
         font-family:'goudy'; font-size:84px; line-height:96px; white-space:nowrap;
         display:flex; align-items:center; }}
.rules {{ left:11%; top:60.8%; width:78%; height:30%;
          font-family:'mplantin'; font-size:80px; line-height:1.18; }}
.rules p {{ margin:0 0 .5em; }}
.pt {{ position:absolute; left:78%; top:89.2%; width:18%; height:4.5%;
       font-family:'goudy'; font-size:104px; line-height:1; color:#0c0c0c;
       display:flex; align-items:center; justify-content:center; }}
.ms.ms-cost {{ font-size:.92em; vertical-align:baseline; }}
.foil {{ position:absolute; left:0; top:0; width:100%; height:100%;
         pointer-events:none; background-repeat:repeat; }}
.foil.sheen {{ background-size:cover; mix-blend-mode:screen; opacity:.20; }}
.foil.stars {{ background-size:18% 18%; mix-blend-mode:screen; opacity:.45;
               transform:rotate(20deg) scale(1.4); filter:brightness(2); }}
</style></head>
<body>
  <div class="face">
    <div class="art layer-art" style="background-image:url('file://{art}')"></div>
    <div class="layer frame" style="background-image:url('file://{frame}')"></div>
    <div class="box title"><span class="nm">{name}</span><span class="mana">{mana}</span></div>
    <div class="box type">{type}</div>
    <div class="box rules">{rules}</div>
    {pt}
    {foil}
  </div>
</body></html>"#,
        mana_css = mana_css.display(),
        goudy = fonts.join("goudy-medieval.ttf").display(),
        mplantin = fonts.join("mplantin.ttf").display(),
        W = W, H = H, fw = FACE_W, fh = FACE_H,
        bx = (W - FACE_W) / 2, by = (H - FACE_H) / 2,
        art = art_abs.display(),
        frame = frame_abs.display(),
        name = esc(&c.name),
        mana = manaify(&c.mana_cost),
        type = esc(&c.type_line),
        rules = rules_html(&c.oracle_text),
        pt = pt,
        foil = foil_layer,
    )
}

fn rules_html(text: &str) -> String {
    let mut s = String::new();
    for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
        s.push_str("<p>");
        s.push_str(&manaify(line));
        s.push_str("</p>");
    }
    s
}

/// Replace `{W}`,`{2}`,`{T}`,`{W/U}`… with mana-font icons.
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
    editor_db: &Path,
    assets_dir: &Path,
    cache_dir: &Path,
    chrome: &str,
    id: i64,
    foil: bool,
    force: bool,
    backend: Option<&str>,
) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening editor DB {} (read-only)", editor_db.display()))?;
    let card = load_card(&db, id)?;

    let frame_rel = assets_dir.join(frame_file(&card));
    if !frame_rel.exists() {
        bail!("missing frame asset {} — run `mtgbrain render assets`", frame_rel.display());
    }

    let (art_path, art_ref) = acquire_art(&card.name, &cache_dir.join("art"), backend, 800)?;
    let hash = card_hash(&card, &art_ref);

    // file:// subresources must be absolute or Chrome silently drops them.
    let assets_abs = fs::canonicalize(assets_dir)?;
    let frame_abs = fs::canonicalize(&frame_rel)?;
    let art_abs = fs::canonicalize(&art_path)?;

    let dir = cache_dir.join("cards").join(sanitize(&card.name));
    let out = if foil {
        dir.join("foil").join(format!("{hash}.png"))
    } else {
        dir.join(format!("{hash}.png"))
    };
    if out.exists() && !force {
        return Ok(out);
    }
    fs::create_dir_all(out.parent().unwrap())?;

    let html = build_html(&card, &frame_abs, &art_abs, &assets_abs, foil);
    let html_path = std::env::temp_dir().join(format!("mtgbrain-render-{id}{}.html", u8::from(foil)));
    fs::write(&html_path, html)?;

    let status = Command::new(chrome)
        .args([
            "--headless",
            "--disable-gpu",
            "--hide-scrollbars",
            "--no-default-browser-check",
            "--no-first-run",
            "--force-device-scale-factor=1",
            "--run-all-compositor-stages-before-draw",
            "--virtual-time-budget=12000",
            &format!("--window-size={W},{H}"),
            &format!("--screenshot={}", out.display()),
            &format!("file://{}", html_path.display()),
        ])
        .status()
        .with_context(|| format!("running Chrome at {chrome}"))?;
    if !status.success() {
        bail!("Chrome screenshot failed (is the path right? --chrome / MTGBRAIN_CHROME)");
    }
    if !out.exists() {
        bail!("Chrome produced no output at {}", out.display());
    }
    Ok(out)
}

/// Render every card in the editor DB (both normal + foil).
#[allow(clippy::too_many_arguments)]
pub fn render_all(
    editor_db: &Path,
    assets_dir: &Path,
    cache_dir: &Path,
    chrome: &str,
    foil: bool,
    force: bool,
    backend: Option<&str>,
) -> Result<()> {
    let ids: Vec<i64> = {
        let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut stmt = db.prepare("SELECT id FROM cube_cards WHERE in_db_found=1 ORDER BY id")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        rows.filter_map(std::result::Result::ok).collect()
    };
    let total = ids.len();
    for (i, id) in ids.iter().enumerate() {
        match render_one(editor_db, assets_dir, cache_dir, chrome, *id, false, force, backend) {
            Ok(_) => print!("\r[{}/{total}] rendered id {id}        ", i + 1),
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

/// Filesystem-safe card-name folder (keep it readable: spaces/commas → _).
pub fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
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
