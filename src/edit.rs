//! Cube editor — a tiny local web app for reviewing a card list one card at a
//! time, accepting each card or attaching an errata, plus a per-card
//! "GenAI new art in the style of Odyssey-block artists?" flag.
//!
//! Two steps:
//!
//! 1. `edit seed` — build an editor SQLite DB from a plain card-name list (one
//!    per line), snapshotting every field from `mtg.sqlite` and applying the
//!    pre-set rules below.
//! 2. `edit serve` — serve a self-contained single-page UI on a high local port
//!    to walk the list and record decisions.
//!
//! Pre-set rules applied at seed time (a starting point — every card is then
//! reviewed by hand in the UI):
//!
//! * `approved` (decision = "accepted") for Odyssey-block creatures only: a
//!   Creature with a printing in ODY / TOR / JUD. Nothing else is pre-approved.
//! * `genai_art` for every *modern-frame* card, i.e. one with no printing in any
//!   pre-8th-Edition (pre-2003 "old frame") set. Old-frame cards keep their
//!   original art; nothing else gets the flag pre-set.

use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Value};
use tiny_http::{Header, Method, Response, Server};

/// Odyssey-block set codes: Odyssey, Torment, Judgment.
const ODYSSEY_BLOCK: &[&str] = &["ODY", "TOR", "JUD"];

/// Old-frame set codes: every set released before 8th Edition (the modern card
/// frame, mid-2003). A card with any of these among its printings keeps its
/// original art and does NOT get the GenAI-new-art flag pre-set. Everything else
/// is modern-frame and is pre-flagged. (Heuristic seed; reviewer toggles per card.)
const OLD_FRAME_SETS: &[&str] = &[
    // Core editions (7th is the last old-frame core set)
    "LEA", "LEB", "2ED", "3ED", "4ED", "5ED", "6ED", "7ED", "CED", "CEI", "SUM", "FBB", "4BB",
    // Reprints / boxed / compilations (pre-2003)
    "BCHR", "CHR", "REN", "RIN", "ITP", "MGB", "ATH", "BRB", "DKM", "BTD",
    // Expansions through Scourge (last old-frame expansion, May 2003)
    "ARN", "ATQ", "LEG", "DRK", "FEM", "ICE", "HML", "ALL", "MIR", "VIS", "WTH", "TMP", "STH",
    "EXO", "USG", "ULG", "UDS", "MMQ", "NEM", "PCY", "INV", "PLS", "APC", "ODY", "TOR", "JUD",
    "ONS", "LGN", "SCG",
    // Portal / Starter / Un (pre-2003)
    "POR", "P02", "PTK", "S99", "UGL",
    // World Championship / Arena league / gateway promos (pre-2003)
    "WC97", "WC98", "WC99", "WC00", "WC01", "WC02", "PAL99", "PAL00", "PAL01", "PAL02", "PARL",
    "PSUS", "PRED", "G00", "G01", "G02", "G03",
];

// ---------------------------------------------------------------------------
// seed
// ---------------------------------------------------------------------------

/// Build the editor DB from `list_path`, snapshotting fields from `mtg_db`.
pub fn seed(mtg_db: &Path, list_path: &Path, out: &Path, force: bool, no_genai: bool) -> Result<()> {
    if out.exists() {
        if force {
            fs::remove_file(out).with_context(|| format!("removing {}", out.display()))?;
        } else {
            bail!(
                "{} already exists; pass --force to rebuild (this discards saved decisions)",
                out.display()
            );
        }
    }

    let names = read_list(list_path)?;
    if names.is_empty() {
        bail!("card list {} is empty", list_path.display());
    }

    let src = Connection::open_with_flags(mtg_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening source DB {}", mtg_db.display()))?;
    let mut db = Connection::open(out).with_context(|| format!("creating {}", out.display()))?;
    db.execute_batch(SCHEMA)?;

    let tx = db.transaction()?;
    let (mut found, mut approved, mut genai, mut missing) = (0u32, 0u32, 0u32, 0u32);
    for (idx, (name, qty)) in names.iter().enumerate() {
        if let Some(c) = lookup_card(&src, name)? {
            found += 1;
            let is_creature = c.type_line.contains("Creature");
            let printings: Vec<&str> = c.printings.split(", ").collect();
            let odyssey_creature =
                is_creature && printings.iter().any(|p| ODYSSEY_BLOCK.contains(p));
            let modern_frame = !printings.iter().any(|p| OLD_FRAME_SETS.contains(p));
            // `--no-genai` (the 8ED back cube uses latest high-DPI art, never GenAI).
            let want_genai = modern_frame && !no_genai;
            if odyssey_creature {
                approved += 1;
            }
            if want_genai {
                genai += 1;
            }
            tx.execute(
                INSERT,
                params![
                    idx as i64,
                    name,
                    c.mana_cost,
                    c.mana_value,
                    c.type_line,
                    c.colors,
                    c.color_identity,
                    c.power,
                    c.toughness,
                    c.loyalty,
                    c.oracle_text,
                    c.keywords,
                    c.produced_mana,
                    c.printings,
                    c.cube_elo,
                    c.edhrec_rank,
                    i64::from(is_creature),
                    i64::from(odyssey_creature),
                    i64::from(want_genai), // genai_art (pre-set)
                    if odyssey_creature {
                        "accepted"
                    } else {
                        "pending"
                    },
                    1i64, // in_db_found
                ],
            )?;
        } else {
            missing += 1;
            tx.execute(INSERT_MISSING, params![idx as i64, name])?;
        }
        if *qty != 1 {
            tx.execute("UPDATE cube_cards SET qty=?2 WHERE id=?1", params![idx as i64, qty])?;
        }
    }
    tx.execute(
        "INSERT INTO meta(key,value) VALUES ('source_list',?1),('seeded_total',?2)",
        params![list_path.to_string_lossy(), names.len() as i64],
    )?;
    tx.commit()?;

    println!("seeded {} -> {}", list_path.display(), out.display());
    println!("  cards:            {}", names.len());
    println!("  matched in DB:    {found}");
    if missing > 0 {
        println!("  NOT found in DB:  {missing}  (rows created, marked unmatched)");
    }
    println!("  pre-approved (Odyssey-block creatures): {approved}");
    println!("  GenAI-art pre-set (modern frame / pre-8ED has no printing): {genai}");
    println!("\nnext:  mtgbrain edit serve --db {}", out.display());
    Ok(())
}

struct CardRow {
    mana_cost: Option<String>,
    mana_value: Option<f64>,
    type_line: String,
    colors: Option<String>,
    color_identity: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    oracle_text: Option<String>,
    keywords: Option<String>,
    produced_mana: Option<String>,
    printings: String,
    cube_elo: Option<f64>,
    edhrec_rank: Option<i64>,
}

fn lookup_card(src: &Connection, name: &str) -> Result<Option<CardRow>> {
    // Match the full card `name` first; if that misses — a multi-face card listed by its
    // FRONT-face name (e.g. "Value Town" for "Value Town // Take a Trip", or the "prepare"
    // cards listed by their creature half) — fall back to the matching FACE so it seeds with
    // that face's data and renders it single-sided.
    for col in ["name", "display_name", "face_name"] {
        let sql = format!(
            "SELECT mana_cost, mana_value, type, colors, color_identity, power, toughness,
                    loyalty, text, keywords, produced_mana, printings, cube_elo, edhrec_rank
               FROM cards WHERE {col} = ?1 ORDER BY COALESCE(face_index, 0) LIMIT 1"
        );
        let mut stmt = src.prepare(&sql)?;
        let mut rows = stmt.query(params![name])?;
        if let Some(r) = rows.next()? {
            return Ok(Some(CardRow {
                mana_cost: r.get(0)?,
                mana_value: r.get(1)?,
                type_line: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                colors: r.get(3)?,
                color_identity: r.get(4)?,
                power: r.get(5)?,
                toughness: r.get(6)?,
                loyalty: r.get(7)?,
                oracle_text: r.get(8)?,
                keywords: r.get(9)?,
                produced_mana: r.get(10)?,
                printings: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
                cube_elo: r.get(12)?,
                edhrec_rank: r.get(13)?,
            }));
        }
    }
    Ok(None)
}

/// One entry per UNIQUE card name (first-seen order), with `qty` = how many times it
/// appeared in the list (so 16× Evolving Wilds → one row, qty 16).
fn read_list(path: &Path) -> Result<Vec<(String, i64)>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out: Vec<(String, i64)> = Vec::new();
    let mut idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in text.lines() {
        let n = line.trim();
        if n.is_empty() || n.starts_with('#') {
            continue;
        }
        if let Some(&i) = idx.get(n) {
            out[i].1 += 1;
        } else {
            idx.insert(n.to_string(), out.len());
            out.push((n.to_string(), 1));
        }
    }
    Ok(out)
}

const SCHEMA: &str = r"
CREATE TABLE cube_cards (
    id              INTEGER PRIMARY KEY,   -- list order
    name            TEXT NOT NULL UNIQUE,
    mana_cost       TEXT,
    mana_value      REAL,
    type            TEXT,
    colors          TEXT,
    color_identity  TEXT,
    power           TEXT,
    toughness       TEXT,
    loyalty         TEXT,
    oracle_text     TEXT,
    keywords        TEXT,
    produced_mana   TEXT,
    printings       TEXT,
    cube_elo        REAL,
    edhrec_rank     INTEGER,
    is_creature             INTEGER NOT NULL DEFAULT 0,
    is_odysseyblock_creature INTEGER NOT NULL DEFAULT 0,
    genai_art       INTEGER NOT NULL DEFAULT 0,  -- 'GenAI new art in style of Odyssey artists?'
    decision        TEXT NOT NULL DEFAULT 'pending',  -- pending | accepted | errata
    errata_text     TEXT NOT NULL DEFAULT '',
    notes           TEXT NOT NULL DEFAULT '',
    in_db_found     INTEGER NOT NULL DEFAULT 1,
    qty             INTEGER NOT NULL DEFAULT 1,   -- how many copies to print (from list duplicates)
    overrides       TEXT NOT NULL DEFAULT '{}',  -- JSON: per-field errata {field: new value}
    removed         INTEGER NOT NULL DEFAULT 0,  -- soft-delete: kept for history, hidden from cube
    illustrator     TEXT NOT NULL DEFAULT '',     -- printed Illus. credit (GenAI artist pseudonym)
    genai_done      INTEGER NOT NULL DEFAULT 0,    -- gallery fully generated + accepted; pass skips it
    art_override    TEXT NOT NULL DEFAULT '',      -- DURABLE art choice {rel,box,artist,year} (anti-trample)
    updated_at      TEXT
);
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
";

const INSERT: &str = r"
INSERT INTO cube_cards
    (id,name,mana_cost,mana_value,type,colors,color_identity,power,toughness,loyalty,
     oracle_text,keywords,produced_mana,printings,cube_elo,edhrec_rank,
     is_creature,is_odysseyblock_creature,genai_art,decision,in_db_found)
VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)
";

const INSERT_MISSING: &str = r"
INSERT INTO cube_cards (id,name,in_db_found,decision) VALUES (?1,?2,0,'pending')
";

// ---------------------------------------------------------------------------
// serve
// ---------------------------------------------------------------------------

/// Render config the UI uses to produce on-demand card images.
#[derive(Clone)]
pub struct RenderCfg {
    pub assets: PathBuf,
    pub cache: PathBuf,
    pub chrome: String,
    pub backend: Option<String>,
    /// Source card DB (mtg.sqlite) for adding new cards by name.
    pub source_db: PathBuf,
    /// Frame style for this editor instance: "seventh" (old frame) or "8th" (8ED/modern).
    pub frame: String,
}

impl RenderCfg {
    fn is_eighth(&self) -> bool {
        matches!(self.frame.as_str(), "8th" | "8ed" | "8ED" | "eighth")
    }
}

/// Add a card to the cube by exact name: snapshot its fields from `source_db`,
/// apply the same pre-sets as seeding, append it to the source list, return it.
fn add_card(db: &Connection, source_db: &Path, name: &str, tags: &str) -> Result<Value> {
    let name = name.trim();
    if name.is_empty() {
        bail!("empty card name");
    }
    let exists: i64 =
        db.query_row("SELECT COUNT(*) FROM cube_cards WHERE name=?1", params![name], |r| r.get(0))?;
    if exists > 0 {
        bail!("'{name}' is already in the cube");
    }
    let src = Connection::open_with_flags(source_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening source DB {}", source_db.display()))?;
    let c = lookup_card(&src, name)?
        .with_context(|| format!("'{name}' not found in the card DB — check exact spelling"))?;
    let is_creature = c.type_line.contains("Creature");
    let printings: Vec<&str> = c.printings.split(", ").collect();
    let odyssey_creature = is_creature && printings.iter().any(|p| ODYSSEY_BLOCK.contains(p));
    let modern_frame = !printings.iter().any(|p| OLD_FRAME_SETS.contains(p));
    let next_id: i64 =
        db.query_row("SELECT COALESCE(MAX(id),-1)+1 FROM cube_cards", [], |r| r.get(0))?;
    db.execute(
        INSERT,
        params![
            next_id, name, c.mana_cost, c.mana_value, c.type_line, c.colors, c.color_identity,
            c.power, c.toughness, c.loyalty, c.oracle_text, c.keywords, c.produced_mana,
            c.printings, c.cube_elo, c.edhrec_rank, i64::from(is_creature),
            i64::from(odyssey_creature), i64::from(modern_frame), "pending", 1i64,
        ],
    )?;
    let tags = tags.trim();
    if !tags.is_empty() {
        db.execute("UPDATE cube_cards SET tags=?2 WHERE id=?1", params![next_id, tags])?;
    }
    // Keep the source list file in sync so a future re-seed remembers this card.
    if let Ok(list_path) =
        db.query_row("SELECT value FROM meta WHERE key='source_list'", [], |r| r.get::<_, String>(0))
    {
        let _ = append_to_list(&list_path, name);
    }
    get_card(db, next_id).context("reading back the added card")
}

fn append_to_list(path: &str, name: &str) -> Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == name) {
        return Ok(());
    }
    let mut f = fs::OpenOptions::new().create(true).append(true).open(path)?;
    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(f)?;
    }
    writeln!(f, "{name}")?;
    Ok(())
}

fn remove_from_list(path: &str, name: &str) -> Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let kept: Vec<&str> = existing.lines().filter(|l| l.trim() != name).collect();
    fs::write(path, format!("{}\n", kept.join("\n")))?;
    Ok(())
}

/// Mark a card removed (1) or restored (0), keeping the row, and sync the list file.
fn set_removed(db: &Connection, id: i64, removed: bool) -> Result<Option<Value>> {
    // Removing a card sets its verdict to 'removed'; restoring sends it back to 'pending'
    // (a restored card is re-reviewed from scratch).
    let decision = if removed { "removed" } else { "pending" };
    db.execute(
        "UPDATE cube_cards SET removed=?2, decision=?3, updated_at=datetime('now') WHERE id=?1",
        params![id, i64::from(removed), decision],
    )?;
    if let (Ok(list_path), Ok(name)) = (
        db.query_row("SELECT value FROM meta WHERE key='source_list'", [], |r| r.get::<_, String>(0)),
        db.query_row("SELECT name FROM cube_cards WHERE id=?1", params![id], |r| r.get::<_, String>(0)),
    ) {
        let _ = if removed {
            remove_from_list(&list_path, &name)
        } else {
            append_to_list(&list_path, &name)
        };
    }
    Ok(get_card(db, id))
}

/// Serve the editor UI on `127.0.0.1:port`.
pub fn serve(editor_db: &Path, port: u16, rc: &RenderCfg) -> Result<()> {
    if !editor_db.exists() {
        bail!(
            "{} not found — run `mtgbrain edit seed` first",
            editor_db.display()
        );
    }
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    // Non-destructive migrations for older editor DBs (ignore "duplicate column" errors).
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN removed INTEGER NOT NULL DEFAULT 0", []);
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN illustrator TEXT NOT NULL DEFAULT ''", []);
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN genai_done INTEGER NOT NULL DEFAULT 0", []);
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN art_override TEXT NOT NULL DEFAULT ''", []);
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN qty INTEGER NOT NULL DEFAULT 1", []);
    // Free-form comma-separated tags (e.g. "banger") — shown as a list badge + searchable `is:banger`.
    let _ = db.execute("ALTER TABLE cube_cards ADD COLUMN tags TEXT NOT NULL DEFAULT ''", []);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::http(addr).map_err(|e| anyhow::anyhow!("starting server: {e}"))?;
    println!(
        "cube editor: http://127.0.0.1:{port}/   (db: {})",
        editor_db.display()
    );
    println!("Ctrl-C to stop.");

    for req in server.incoming_requests() {
        // Handle each request on its OWN thread with its OWN DB connection, so a slow render
        // (MPCfill/Claude/Chrome on a freshly-added card) can never block the whole editor.
        // Accepting stays on this thread (fast); only handling fans out.
        let editor_db_buf = editor_db.to_path_buf();
        let rc_owned = rc.clone();
        std::thread::spawn(move || {
        let mut req = req;
        let editor_db: &Path = &editor_db_buf;
        let rc: &RenderCfg = &rc_owned;
        let db = if let Ok(d) = Connection::open(editor_db) { d } else {
            let _ = req.respond(Response::from_string("db open failed").with_status_code(500));
            return;
        };
        // Wait (don't error) if another thread/process holds a write lock.
        let _ = db.busy_timeout(std::time::Duration::from_secs(15));
        let method = req.method().clone();
        let url = req.url().to_string();
        let path = url.split('?').next().unwrap_or("").to_string();
        // Browser's cached validator, for conditional GETs on card images (snappy nav).
        let if_none_match = req
            .headers()
            .iter()
            .find(|h| h.field.equiv("If-None-Match"))
            .map(|h| h.value.as_str().to_string());

        let resp = match (&method, path.as_str()) {
            (Method::Get, "/") => html_response(INDEX_HTML),
            (Method::Get, "/api/cards") => json_response(&list_cards(&db)),
            (Method::Get, p) if p.starts_with("/api/card/") => match id_from(p, "/api/card/") {
                Some(id) => match get_card(&db, id) {
                    Some(mut v) => {
                        // Attach the original Scryfall flavor so the editor's flavor field
                        // can prefill it (override wins; this is the fallback baseline).
                        let name = v["name"].as_str().unwrap_or("").to_string();
                        if let Some(o) = v.as_object_mut() {
                            o.insert(
                                "flavor_base".to_string(),
                                json!(crate::render::base_flavor(&name, &rc.cache)),
                            );
                        }
                        json_response(&v)
                    }
                    None => not_found(),
                },
                None => not_found(),
            },
            (Method::Post, p) if p.starts_with("/api/card/") => {
                let mut body = String::new();
                req.as_reader().read_to_string(&mut body).ok();
                match id_from(p, "/api/card/") {
                    Some(id) => match save_card(&db, id, &body) {
                        Ok(Some(v)) => json_response(&v),
                        Ok(None) => not_found(),
                        Err(e) => json_response(&json!({"error": e.to_string()})),
                    },
                    None => not_found(),
                }
            }
            // Add a card to the cube by exact name.
            (Method::Post, "/api/add") => {
                let mut body = String::new();
                req.as_reader().read_to_string(&mut body).ok();
                let parsed = serde_json::from_str::<Value>(&body).ok();
                let name = parsed
                    .as_ref()
                    .and_then(|v| v.get("name").and_then(Value::as_str).map(ToString::to_string))
                    .unwrap_or_default();
                let tags = parsed
                    .as_ref()
                    .and_then(|v| v.get("tags").and_then(Value::as_str).map(ToString::to_string))
                    .unwrap_or_default();
                match add_card(&db, &rc.source_db, &name, &tags) {
                    Ok(v) => json_response(&v),
                    Err(e) => json_response(&json!({"error": e.to_string()})),
                }
            }
            // Set the print quantity (number of copies) for a card.
            (Method::Post, p) if p.starts_with("/api/qty/") => match id_from(p, "/api/qty/") {
                Some(id) => {
                    let mut body = String::new();
                    req.as_reader().read_to_string(&mut body).ok();
                    let q = serde_json::from_str::<Value>(&body)
                        .ok()
                        .and_then(|v| v.get("qty").and_then(Value::as_i64))
                        .unwrap_or(1)
                        .max(0);
                    let _ = db.execute("UPDATE cube_cards SET qty=?2 WHERE id=?1", params![id, q]);
                    json_response(&json!({"id": id, "qty": q}))
                }
                None => not_found(),
            },
            // Soft-remove / restore a card (kept in DB; removed from the cube list).
            (Method::Post, p) if p.starts_with("/api/remove/") => match id_from(p, "/api/remove/") {
                Some(id) => set_removed(&db, id, true).ok().flatten().map_or_else(not_found, |v| json_response(&v)),
                None => not_found(),
            },
            (Method::Post, p) if p.starts_with("/api/restore/") => match id_from(p, "/api/restore/") {
                Some(id) => set_removed(&db, id, false).ok().flatten().map_or_else(not_found, |v| json_response(&v)),
                None => not_found(),
            },
            // GenAI art gallery: generate options, serve option cards, choose one.
            (_, p) if p.starts_with("/api/genai/") => {
                let rest = p.strip_prefix("/api/genai/").unwrap_or("");
                let segs: Vec<&str> = rest.split('/').collect();
                let gid = segs.first().and_then(|s| s.parse::<i64>().ok());
                match (&method, segs.as_slice(), gid) {
                    // Review dashboard: every genai-flagged card + its status.
                    (Method::Get, ["dashboard"], _) => {
                        match crate::render::genai_dashboard(editor_db, &rc.cache) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Cached options (no generation) for the review gallery.
                    (Method::Get, [_, "options"], Some(gid)) => {
                        match crate::render::genai_existing(editor_db, &rc.cache, gid) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Event history / time-travel for a card.
                    (Method::Get, [_, "history"], Some(gid)) => {
                        match crate::render::genai_history(&rc.cache, gid) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Restore an old event's prompt as the current direction.
                    (Method::Post, [_, "reprompt"], Some(gid)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let ev = serde_json::from_str::<Value>(&body)
                            .ok()
                            .and_then(|v| v["event_id"].as_i64())
                            .unwrap_or(0);
                        match crate::render::genai_reprompt(editor_db, &rc.cache, gid, ev) {
                            Ok(d) => json_response(&json!({"direction": d})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Clear the chosen art (logged).
                    (Method::Post, [_, "unchoose"], Some(gid)) => {
                        match crate::render::genai_unchoose(editor_db, &rc.cache, gid) {
                            Ok(()) => json_response(&json!({"ok": true})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Choose the card's ORIGINAL (real) art — recorded as a decision so the
                    // card STAYS in the Review list marked "✓ Original art" (genai flag kept).
                    (Method::Post, [_, "original"], Some(gid)) => {
                        match crate::render::genai_choose_original(editor_db, &rc.cache, gid) {
                            Ok(()) => json_response(&json!({"ok": true})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Art direction: fetch (cached/Claude) or regenerate; user edits before generating.
                    (Method::Post, [_, "direction"], Some(gid)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let regen = serde_json::from_str::<Value>(&body)
                            .ok()
                            .and_then(|v| v["regenerate"].as_bool())
                            .unwrap_or(false);
                        match crate::render::genai_direction(editor_db, &rc.cache, gid, regen) {
                            Ok(d) => json_response(&json!({"direction": d})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    (Method::Post, [_, "generate"], Some(gid)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let dir = serde_json::from_str::<Value>(&body)
                            .ok()
                            .and_then(|v| v["direction"].as_str().map(ToString::to_string));
                        match crate::render::genai_options(
                            editor_db, &rc.assets, &rc.cache, &rc.chrome, gid, dir.as_deref(),
                        ) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    (Method::Get, [_, "card", bucket], Some(gid)) => {
                        let nm = db
                            .query_row("SELECT name FROM cube_cards WHERE id=?1", params![gid], |r| {
                                r.get::<_, String>(0)
                            })
                            .unwrap_or_default();
                        let path = crate::render::genai_card_path(&rc.cache, &nm, bucket);
                        match std::fs::read(&path) {
                            Ok(bytes) => Response::from_data(bytes)
                                .with_header(header("Content-Type", "image/png")),
                            Err(_) => not_found(),
                        }
                    }
                    // Render + serve the REAL-art preview (the "Original art" gallery option).
                    (Method::Get, [_, "base-card"], Some(gid)) => {
                        match crate::render::genai_base_card(editor_db, &rc.assets, &rc.cache, &rc.chrome, gid) {
                            Ok(path) => match std::fs::read(&path) {
                                Ok(bytes) => Response::from_data(bytes)
                                    .with_header(header("Content-Type", "image/png")),
                                Err(_) => not_found(),
                            },
                            Err(_) => not_found(),
                        }
                    }
                    (Method::Post, [_, "choose"], Some(gid)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let v: Value = serde_json::from_str(&body).unwrap_or(json!({}));
                        let artist = v["artist"].as_str().unwrap_or("");
                        let hash = v["hash"].as_str().unwrap_or("");
                        match crate::render::genai_choose(editor_db, &rc.cache, gid, artist, hash) {
                            Ok(()) => json_response(&json!({"ok": true})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    _ => not_found(),
                }
            }
            // Per-card crop editor: reposition the underlying MPCfill/Scryfall art.
            (_, p) if p.starts_with("/api/art/") => {
                let rest = p.strip_prefix("/api/art/").unwrap_or("");
                let segs: Vec<&str> = rest.split('/').collect();
                let aid = segs.first().and_then(|s| s.parse::<i64>().ok());
                match (&method, segs.as_slice(), aid) {
                    // Metadata: current box + every selectable source + window aspect.
                    (Method::Get, [_], Some(id)) => {
                        match crate::render::art_meta(editor_db, &rc.cache, id, rc.is_eighth()) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Serve a source image (the underlying card/art the user crops over).
                    // `&thumb=1` serves a small cached JPEG for the picker sidebar.
                    (Method::Get, [_, "img"], Some(id)) => {
                        let rel = query_param(&url, "rel").unwrap_or_else(|| "@current".to_string());
                        let resolved = if url.contains("thumb=1") {
                            crate::render::art_thumb(editor_db, &rc.cache, id, rc.is_eighth(), &rel)
                        } else {
                            crate::render::art_image_path(editor_db, &rc.cache, id, rc.is_eighth(), &rel)
                        };
                        match resolved {
                            Ok(path) => match std::fs::read(&path) {
                                Ok(bytes) => Response::from_data(bytes)
                                    .with_header(header("Content-Type", mime_of(&path))),
                                Err(_) => not_found(),
                            },
                            Err(_) => not_found(),
                        }
                    }
                    // Fetch ALL candidates (MPCfill proxies >=600 DPI + alt-printing art),
                    // then return the refreshed source list.
                    (Method::Post, [_, "fetch"], Some(id)) => {
                        match crate::render::art_fetch_all(editor_db, &rc.cache, id, rc.is_eighth(), rc.backend.as_deref()) {
                            Ok(v) => json_response(&v),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    // Upload art from a local disk path:
                    // {path, artist?, year?, illustrator?, dpi_threshold?, strict?}.
                    (Method::Post, [_, "upload"], Some(id)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
                        let path = v["path"].as_str().unwrap_or("").trim().to_string();
                        if path.is_empty() {
                            json_response(&json!({"error": "missing 'path' (local image file)"}))
                        } else {
                            match crate::render::art_upload(
                                editor_db,
                                &rc.cache,
                                id,
                                rc.is_eighth(),
                                std::path::Path::new(&path),
                                v["artist"].as_str(),
                                v["year"].as_str(),
                                v["illustrator"].as_str(),
                                v["dpi_threshold"].as_i64().unwrap_or(600),
                                v["strict"].as_bool().unwrap_or(false),
                            ) {
                                Ok(res) => json_response(&res),
                                Err(e) => json_response(&json!({"error": e.to_string()})),
                            }
                        }
                    }
                    // Save a hand-placed crop: {rel, box:[x,y,w,h], box2?:[x,y,w,h]}.
                    // box2 is a SPLIT card's RIGHT-half rectangle (the left half is `box`).
                    (Method::Post, [_], Some(id)) => {
                        let mut body = String::new();
                        req.as_reader().read_to_string(&mut body).ok();
                        let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
                        let rel = v["rel"].as_str().unwrap_or("@current").to_string();
                        let parse_box = |k: &str| {
                            v[k].as_array().filter(|a| a.len() == 4).map(|a| {
                                let g = |i: usize| a[i].as_f64().unwrap_or(0.0);
                                [g(0), g(1), g(2), g(3)]
                            })
                        };
                        let bx = parse_box("box");
                        let bx2 = parse_box("box2");
                        match bx {
                            Some(bx) => match crate::render::art_save_crop(editor_db, &rc.cache, id, rc.is_eighth(), &rel, bx, bx2) {
                                Ok(()) => json_response(&json!({"ok": true})),
                                Err(e) => json_response(&json!({"error": e.to_string()})),
                            },
                            None => json_response(&json!({"error": "missing or malformed box"})),
                        }
                    }
                    // Reset to the automatic crop (restores the stashed auto pick if any).
                    (Method::Post, [_, "reset"], Some(id)) => {
                        match crate::render::art_reset(editor_db, &rc.cache, id, rc.is_eighth()) {
                            Ok(()) => json_response(&json!({"ok": true})),
                            Err(e) => json_response(&json!({"error": e.to_string()})),
                        }
                    }
                    _ => not_found(),
                }
            }
            // On-demand render (cached by content hash). ?foil=1, &force=1, &full=1.
            // By default serves a small cached JPEG preview (~40× smaller than the print PNG)
            // with ETag + Cache-Control so navigation is snappy and revisits are 304s; `full=1`
            // (the "full DPI" link) serves the original print PNG.
            (Method::Get, p) if p.starts_with("/api/render/") => {
                let foil = url.contains("foil=1");
                let force = url.contains("force=1");
                let want_full = url.contains("full=1");
                match id_from(p, "/api/render/") {
                    Some(id) => match if rc.is_eighth() {
                        crate::render::render_card_8th(
                            editor_db, &rc.assets, &rc.cache, &rc.chrome, id, force, rc.backend.as_deref(),
                        )
                    } else {
                        crate::render::render_one(
                            editor_db, &rc.assets, &rc.cache, &rc.chrome, id, foil, force,
                            rc.backend.as_deref(),
                        )
                    } {
                        Ok(full_path) => {
                            let (serve_path, mime) = if want_full {
                                (full_path.clone(), "image/png")
                            } else {
                                match crate::render::card_preview(&full_path, 760) {
                                    Ok(p) => (p, "image/jpeg"),
                                    Err(_) => (full_path.clone(), "image/png"),
                                }
                            };
                            let etag = file_etag(&serve_path);
                            if etag.is_some() && etag == if_none_match {
                                Response::from_data(Vec::new())
                                    .with_status_code(304)
                                    .with_header(header("ETag", etag.as_deref().unwrap_or("")))
                                    .with_header(header("Cache-Control", "private, no-cache"))
                            } else {
                                image_response(&serve_path, mime, etag)
                            }
                        }
                        Err(e) => Response::from_string(e.to_string()).with_status_code(500),
                    },
                    None => not_found(),
                }
            }
            _ => not_found(),
        };
        let _ = req.respond(resp);
        }); // end per-request worker thread
    }
    Ok(())
}

fn id_from(path: &str, prefix: &str) -> Option<i64> {
    path.strip_prefix(prefix)?.parse().ok()
}

fn list_cards(db: &Connection) -> Value {
    let mut stmt = db
        .prepare(
            // Includes the fields the front-end Scryfall-style search filters on
            // (colors, stats, oracle text, etc.) so the whole query runs client-side
            // over just this cube's cards. `overrides` is applied below so the search
            // sees each card's EFFECTIVE (post-errata / colour-shifted) values, not the
            // original printed snapshot.
            "SELECT id,name,type,mana_cost,cube_elo,decision,genai_art,
                    is_odysseyblock_creature,in_db_found,removed,
                    (overrides IS NOT NULL AND overrides != '{}' AND overrides != ''),
                    mana_value,colors,color_identity,power,toughness,loyalty,
                    oracle_text,keywords,produced_mana,printings,edhrec_rank,is_creature,
                    COALESCE(overrides,'{}'),COALESCE(tags,''),COALESCE(qty,1)
               FROM cube_cards ORDER BY id",
        )
        .expect("prepare list");
    let rows = stmt
        .query_map([], |r| {
            let ov: Value =
                serde_json::from_str(&r.get::<_, String>(23)?).unwrap_or_else(|_| json!({}));
            // Effective value of an erratable field: the override wins, else the base column.
            let eff = |k: &str, base: Option<String>| {
                ov.get(k).and_then(Value::as_str).map(str::to_string).or(base)
            };

            let eff_mc = eff("mana_cost", r.get::<_, Option<String>>(3)?);
            let eff_ot = eff("oracle_text", r.get::<_, Option<String>>(17)?);

            // Effective colour identity: a manual `color_identity` override wins, else it is
            // re-solved from the effective mana cost + rules text — the same value that drives
            // the gold frame / colour dot / CSV. Effective colours: an explicit `colors`
            // override wins, else a deliberate colour-shift (identity override) defines the
            // colour, else the colours are the pips of the effective mana cost. This is what
            // makes `c:`/`id:` search match the cube's shifted colour, not the printed one.
            let ci_ov = ov.get("color_identity").and_then(Value::as_str);
            let mc_s = eff_mc.as_deref().unwrap_or("");
            let eff_ci = effective_color_identity(mc_s, eff_ot.as_deref().unwrap_or(""), ci_ov);
            let eff_colors = if let Some(s) = ov.get("colors").and_then(Value::as_str) {
                effective_color_identity("", "", Some(s))
            } else if ci_ov.map(str::trim).is_some_and(|s| !s.is_empty()) {
                eff_ci.clone()
            } else {
                color_identity_of(mc_s, "")
            };

            // Mana value: recompute from an errata'd cost, else trust the seeded value.
            let mana_value = if ov.get("mana_cost").is_some() {
                Some(mana_value_of(mc_s))
            } else {
                r.get::<_, Option<f64>>(11)?
            };

            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": eff("name", r.get::<_, Option<String>>(1)?),
                "type": eff("type", r.get::<_, Option<String>>(2)?),
                "mana_cost": eff_mc,
                "cube_elo": r.get::<_, Option<f64>>(4)?,
                "decision": r.get::<_, String>(5)?,
                "genai_art": r.get::<_, i64>(6)? != 0,
                "odyssey_creature": r.get::<_, i64>(7)? != 0,
                "found": r.get::<_, i64>(8)? != 0,
                "removed": r.get::<_, i64>(9)? != 0,
                "errata": r.get::<_, i64>(10)? != 0,
                "mana_value": mana_value,
                "colors": eff_colors,
                "color_identity": eff_ci,
                "power": eff("power", r.get::<_, Option<String>>(14)?),
                "toughness": eff("toughness", r.get::<_, Option<String>>(15)?),
                "loyalty": eff("loyalty", r.get::<_, Option<String>>(16)?),
                "oracle_text": eff_ot,
                "keywords": eff("keywords", r.get::<_, Option<String>>(18)?),
                "produced_mana": eff("produced_mana", r.get::<_, Option<String>>(19)?),
                "printings": r.get::<_, Option<String>>(20)?,
                "edhrec_rank": r.get::<_, Option<i64>>(21)?,
                "is_creature": r.get::<_, i64>(22)? != 0,
                "tags": r.get::<_, String>(24)?,
                "banger": r.get::<_, String>(24)?.to_lowercase().contains("banger"),
                "qty": r.get::<_, i64>(25)?,
            }))
        })
        .expect("query list")
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();

    let counts = db
        .query_row(
            "SELECT
                SUM(removed=0),
                SUM(decision='accepted' AND removed=0),
                SUM((overrides IS NOT NULL AND overrides != '{}' AND overrides != '') AND removed=0),
                SUM(decision='pending' AND removed=0),
                SUM(genai_art AND removed=0),
                SUM(removed)
             FROM cube_cards",
            [],
            |r| {
                Ok(json!({
                    "total":   r.get::<_, Option<i64>>(0)?.unwrap_or(0),
                    "accepted":r.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    "errata":  r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    "pending": r.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    "genai":   r.get::<_, Option<i64>>(4)?.unwrap_or(0),
                    "removed": r.get::<_, Option<i64>>(5)?.unwrap_or(0),
                }))
            },
        )
        .unwrap_or_else(|_| json!({}));

    json!({ "cards": rows, "counts": counts })
}

fn get_card(db: &Connection, id: i64) -> Option<Value> {
    db.query_row(
        "SELECT id,name,mana_cost,mana_value,type,colors,color_identity,power,toughness,
                loyalty,oracle_text,keywords,produced_mana,printings,cube_elo,edhrec_rank,
                is_creature,is_odysseyblock_creature,genai_art,decision,errata_text,notes,
                in_db_found,updated_at,overrides,removed,COALESCE(illustrator,''),COALESCE(qty,1)
           FROM cube_cards WHERE id=?1",
        params![id],
        |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "mana_cost": r.get::<_, Option<String>>(2)?,
                "mana_value": r.get::<_, Option<f64>>(3)?,
                "type": r.get::<_, Option<String>>(4)?,
                "colors": r.get::<_, Option<String>>(5)?,
                "color_identity": r.get::<_, Option<String>>(6)?,
                "power": r.get::<_, Option<String>>(7)?,
                "toughness": r.get::<_, Option<String>>(8)?,
                "loyalty": r.get::<_, Option<String>>(9)?,
                "oracle_text": r.get::<_, Option<String>>(10)?,
                "keywords": r.get::<_, Option<String>>(11)?,
                "produced_mana": r.get::<_, Option<String>>(12)?,
                "printings": r.get::<_, Option<String>>(13)?,
                "cube_elo": r.get::<_, Option<f64>>(14)?,
                "edhrec_rank": r.get::<_, Option<i64>>(15)?,
                "is_creature": r.get::<_, i64>(16)? != 0,
                "odyssey_creature": r.get::<_, i64>(17)? != 0,
                "genai_art": r.get::<_, i64>(18)? != 0,
                "decision": r.get::<_, String>(19)?,
                "errata_text": r.get::<_, String>(20)?,
                "notes": r.get::<_, String>(21)?,
                "found": r.get::<_, i64>(22)? != 0,
                "updated_at": r.get::<_, Option<String>>(23)?,
                "overrides": serde_json::from_str::<Value>(&r.get::<_, String>(24)?)
                    .unwrap_or_else(|_| json!({})),
                "removed": r.get::<_, i64>(25)? != 0,
                "illustrator": r.get::<_, String>(26)?,
                "qty": r.get::<_, i64>(27)?,
            }))
        },
    )
    .ok()
}

fn save_card(db: &Connection, id: i64, body: &str) -> Result<Option<Value>> {
    let v: Value = serde_json::from_str(body).context("parsing request body")?;
    if let Some(d) = v.get("decision").and_then(Value::as_str) {
        if !matches!(d, "pending" | "accepted" | "errata") {
            bail!("invalid decision: {d}");
        }
        db.execute(
            "UPDATE cube_cards SET decision=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, d],
        )?;
    }
    if let Some(g) = v.get("genai_art").and_then(Value::as_bool) {
        db.execute(
            "UPDATE cube_cards SET genai_art=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, i64::from(g)],
        )?;
    }
    if let Some(e) = v.get("errata_text").and_then(Value::as_str) {
        db.execute(
            "UPDATE cube_cards SET errata_text=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, e],
        )?;
    }
    if let Some(n) = v.get("notes").and_then(Value::as_str) {
        db.execute(
            "UPDATE cube_cards SET notes=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, n],
        )?;
    }
    // Artist / printed "Illus." credit (a dedicated column, NOT an `overrides` errata field).
    // The renderer prefers this over the art's own historical artist; blank reverts to it.
    if let Some(a) = v.get("illustrator").and_then(Value::as_str) {
        db.execute(
            "UPDATE cube_cards SET illustrator=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, a.trim()],
        )?;
    }
    if let Some(o) = v.get("overrides") {
        let Some(incoming) = o.as_object() else {
            bail!("overrides must be a JSON object");
        };
        let mut merged = incoming.clone();
        // The editor UI only manages the card FIELDS (name/cost/type/text/PT/colors). Keys
        // it doesn't surface — `flavor` and `errata_scroll` — must NOT be dropped when a
        // field edit is saved. Preserve them from the existing overrides unless the payload
        // explicitly carries them.
        let existing: Value = db
            .query_row(
                "SELECT overrides FROM cube_cards WHERE id=?1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}));
        for key in ["flavor", "errata_scroll"] {
            if !merged.contains_key(key) {
                if let Some(val) = existing.get(key) {
                    merged.insert(key.to_string(), val.clone());
                }
            }
        }
        // Re-solve the colour identity from the new EFFECTIVE mana cost + rules text (a manual
        // `color_identity` override wins), so the frame / colour dot / CSV stay aligned without
        // a separate `recolor` pass.
        let merged_v = Value::Object(merged.clone());
        let mpick = |k: &str, base: Option<String>| {
            merged_v.get(k).and_then(Value::as_str).map(ToString::to_string).or(base).unwrap_or_default()
        };
        let (base_mc, base_ot): (Option<String>, Option<String>) = db
            .query_row(
                "SELECT mana_cost, oracle_text FROM cube_cards WHERE id=?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((None, None));
        let ci = effective_color_identity(
            &mpick("mana_cost", base_mc),
            &mpick("oracle_text", base_ot),
            merged_v.get("color_identity").and_then(Value::as_str),
        );
        db.execute(
            "UPDATE cube_cards SET overrides=?2, color_identity=?3, updated_at=datetime('now') WHERE id=?1",
            params![id, Value::Object(merged).to_string(), ci],
        )?;
    }
    Ok(get_card(db, id))
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn html_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(header("Content-Type", "text/html; charset=utf-8"))
}

fn json_response(v: &Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(v.to_string())
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
}

fn not_found() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("not found").with_status_code(404)
}

/// Cheap, strong-enough ETag for a cached image: its byte length + mtime. Lets the editor
/// answer a revisit with `304 Not Modified` (zero bytes) instead of re-shipping the file.
fn file_etag(path: &std::path::Path) -> Option<String> {
    let m = std::fs::metadata(path).ok()?;
    let secs = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    Some(format!("\"{:x}-{:x}\"", m.len(), secs))
}

/// Serve a cached image file with caching headers so the browser keeps it and revalidates
/// cheaply (see [`file_etag`]). Falls back to 404 if the file vanished between stat and read.
fn image_response(path: &std::path::Path, mime: &str, etag: Option<String>) -> Response<std::io::Cursor<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut r = Response::from_data(bytes)
                .with_header(header("Content-Type", mime))
                .with_header(header("Cache-Control", "private, no-cache"));
            if let Some(tag) = etag {
                r = r.with_header(header("ETag", tag.as_str()));
            }
            r
        }
        Err(_) => not_found(),
    }
}

/// Read one query-string parameter (percent-decoded) from a request URL.
fn query_param(url: &str, key: &str) -> Option<String> {
    let q = url.split('?').nth(1)?;
    q.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
        (k == key).then(|| percent_decode(v))
    })
}

/// Minimal `application/x-www-form-urlencoded` decode (`%XX` escapes, `+` → space).
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let hex = |c: u8| (c as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hex(b[i + 1]), hex(b[i + 2])) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Content-Type for a served image, by extension (defaults to octet-stream).
fn mime_of(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

// Default DB path next to the source list.
pub fn default_editor_db(data_dir: &Path) -> PathBuf {
    data_dir.join("cube_editor.sqlite")
}

// ---------------------------------------------------------------------------
// cubecobra-csv — export a CubeCobra import CSV (custom image URL + colours)
// ---------------------------------------------------------------------------

/// CubeCobra's "Color Category" bucket for a card, from its type + color identity.
fn color_category(type_line: &str, ci_letters: &[char]) -> &'static str {
    if type_line.to_lowercase().contains("land") {
        return "Lands";
    }
    match ci_letters {
        [] => "Colorless",
        ['W'] => "White",
        ['U'] => "Blue",
        ['B'] => "Black",
        ['R'] => "Red",
        ['G'] => "Green",
        _ => "Multicolored",
    }
}

/// Minimal CSV field quoting (wrap in quotes + double any inner quotes).
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Whether a card shows the errata ribbon — MUST match `render.rs::load_card`: a
/// manual `errata_scroll` override forces it on/off, otherwise it auto-trips on any
/// FUNCTIONAL field override or a non-empty errata note (cosmetic overrides like
/// `name`/`flavor` never trip it).
fn errata_ribbon(overrides: &Value, errata_text: &str) -> bool {
    const FUNCTIONAL: &[&str] =
        &["mana_cost", "type", "oracle_text", "power", "toughness", "loyalty", "colors"];
    let scroll_override = overrides.get("errata_scroll").and_then(|v| match v {
        Value::Bool(b) => Some(*b),
        Value::String(s) => match s.trim().to_lowercase().as_str() {
            "on" | "true" | "1" | "yes" => Some(true),
            "off" | "false" | "0" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    });
    let auto = overrides
        .as_object()
        .is_some_and(|m| m.keys().any(|k| FUNCTIONAL.contains(&k.as_str())))
        || !errata_text.trim().is_empty();
    scroll_override.unwrap_or(auto)
}

/// Write a CubeCobra bulk-import CSV for every active card: the real card name
/// (so CubeCobra resolves the card), a custom `Image URL` pointing at the card's
/// uploaded selfhost render (`<base>/<sanitized-name>.png` — the same `sanitize`
/// the renderer uses for cache dirs / S3 keys), and the recomputed colour
/// identity as `Color`/`Color Category`. Paste the result into the cube's
/// "Replace from CSV" on CubeCobra.
pub fn cubecobra_csv(editor_db: &Path, base: &str, out: &Path) -> Result<()> {
    let base = base.trim_end_matches('/');
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening {}", editor_db.display()))?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, Option<f64>, Option<String>, Option<String>, String, String, String)> = {
        let mut stmt = db.prepare(
            "SELECT name,mana_value,type,color_identity,COALESCE(overrides,'{}'),COALESCE(errata_text,''),COALESCE(tags,'')
             FROM cube_cards WHERE removed=0 ORDER BY id",
        )?;
        let v = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<f64>>(1)?, r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?, r.get::<_, String>(4)?, r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?))
            })?
            .collect::<std::result::Result<_, _>>()?;
        v
    };
    let mut w = String::from(
        "Name,CMC,Type,Color,Set,Collector Number,Rarity,Color Category,Status,Finish,Maybeboard,Image URL,Image Back URL,Tags,Notes,MTGO ID\n",
    );
    let mut n = 0usize;
    for (name, mv, ty, ci, ov, errata_text, card_tags) in rows {
        let o: Value = serde_json::from_str(&ov).unwrap_or_else(|_| json!({}));
        let type_line = o.get("type").and_then(Value::as_str).map(ToString::to_string)
            .or(ty).unwrap_or_default();
        let cmc = o.get("mana_value").and_then(Value::as_f64).or(mv).unwrap_or(0.0);
        let cmc = if cmc.fract() == 0.0 { format!("{}", cmc as i64) } else { format!("{cmc}") };
        let letters: Vec<char> =
            ci.unwrap_or_default().chars().filter(|c| "WUBRG".contains(*c)).collect();
        let color: String = letters.iter().collect();
        let cat = color_category(&type_line, &letters);
        let url = format!("{base}/{}.png", crate::render::sanitize(&name));
        // Tags: the errata ribbon (same rule as the renderer) plus any cube_cards.tags
        // (e.g. "banger"), title-cased, so CubeCobra can show/filter them too.
        let mut tag_list: Vec<String> = Vec::new();
        if errata_ribbon(&o, &errata_text) {
            tag_list.push("Errata".to_string());
        }
        for t in card_tags.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let mut cs = t.chars();
            let titled = cs.next().map_or_else(String::new, |c| {
                c.to_uppercase().collect::<String>() + cs.as_str()
            });
            tag_list.push(titled);
        }
        let tags = tag_list.join(",");
        w.push_str(&format!(
            "{},{cmc},{},{color},,,,{cat},Owned,Non-foil,false,{},,{},,\n",
            csv_field(&name),
            csv_field(&type_line),
            csv_field(&url),
            csv_field(&tags),
        ));
        n += 1;
    }
    if let Some(p) = out.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(out, w)?;
    println!("cubecobra-csv: {n} cards -> {}", out.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// recolor — recompute color identity from the effective mana cost + rules text
// ---------------------------------------------------------------------------

/// Mana value of a `{..}`-symbol cost: numeric symbols add their number, `{X}`/`{Y}`/`{Z}`
/// count 0, and every other symbol (coloured, hybrid, Phyrexian, `{C}`, snow, `{S}`) counts 1.
/// Used to keep mana value aligned with an errata'd mana cost in the editor search.
fn mana_value_of(mana_cost: &str) -> f64 {
    let mut total = 0.0;
    let mut chars = mana_cost.chars();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut sym = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                sym.push(c2);
            }
            // Hybrid like "2/W" contributes the numeric side (2); "W/U" contributes 1.
            let numeric = sym.split(['/', '\u{2044}']).find_map(|p| p.trim().parse::<f64>().ok());
            if let Some(n) = numeric {
                total += n;
            } else if sym.eq_ignore_ascii_case("x")
                || sym.eq_ignore_ascii_case("y")
                || sym.eq_ignore_ascii_case("z")
            {
                // variable: counts 0
            } else {
                total += 1.0;
            }
        }
    }
    total
}

/// A card's color identity = every WUBRG letter appearing in any `{..}` mana
/// symbol of its mana cost OR its rules text (hybrid/Phyrexian symbols count each
/// colour; `{C}`/generic/`{T}` contribute nothing), sorted WUBRG and joined as
/// `"W, U"` to match the seeded format. Colour *words* in rules text never count —
/// only mana symbols, exactly as the Comprehensive Rules define colour identity.
fn color_identity_of(mana_cost: &str, oracle_text: &str) -> String {
    let mut set: Vec<char> = Vec::new();
    for src in [mana_cost, oracle_text] {
        let mut chars = src.chars();
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
    }
    const ORDER: &str = "WUBRG";
    set.sort_by_key(|c| ORDER.find(*c).unwrap_or(9));
    set.iter().map(char::to_string).collect::<Vec<_>>().join(", ")
}

/// `color_identity_of`, but a non-empty MANUAL override (e.g. "UB", "U, B") wins outright,
/// letting a cost-removed card still declare its identity for the solver/frame/dot.
fn effective_color_identity(mana_cost: &str, oracle_text: &str, ci_override: Option<&str>) -> String {
    match ci_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => {
            let mut set: Vec<char> = Vec::new();
            for ch in s.chars().map(|c| c.to_ascii_uppercase()) {
                if "WUBRG".contains(ch) && !set.contains(&ch) {
                    set.push(ch);
                }
            }
            const ORDER: &str = "WUBRG";
            set.sort_by_key(|c| ORDER.find(*c).unwrap_or(9));
            set.iter().map(char::to_string).collect::<Vec<_>>().join(", ")
        }
        None => color_identity_of(mana_cost, oracle_text),
    }
}

/// Recompute `color_identity` for every active card from its effective (override-
/// applied) mana cost + rules text, and write it back. The DB mana cost (with the
/// `overrides` blob applied) is the source of truth — this is what re-aligns the
/// colour-shifted lock pieces. Prints every change; `dry_run` writes nothing.
pub fn recolor(editor_db: &Path, dry_run: bool) -> Result<()> {
    let mut db = Connection::open(editor_db)
        .with_context(|| format!("opening {}", editor_db.display()))?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, String, Option<String>, Option<String>, Option<String>, String)> = {
        let mut stmt = db.prepare(
            "SELECT id,name,mana_cost,oracle_text,color_identity,COALESCE(overrides,'{}')
             FROM cube_cards WHERE removed=0 ORDER BY id",
        )?;
        let v = stmt
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
            })?
            .collect::<std::result::Result<_, _>>()?;
        v
    };
    let mut updates: Vec<(i64, String)> = Vec::new();
    for (id, name, mc, ot, ci_old, ov) in rows {
        let o: Value = serde_json::from_str(&ov).unwrap_or_else(|_| json!({}));
        let pick = |k: &str, base: Option<String>| {
            o.get(k).and_then(Value::as_str).map(ToString::to_string).or(base).unwrap_or_default()
        };
        let mana = pick("mana_cost", mc);
        let oracle = pick("oracle_text", ot);
        let ci = effective_color_identity(&mana, &oracle, o.get("color_identity").and_then(Value::as_str));
        let ci_old = ci_old.unwrap_or_default();
        if ci != ci_old {
            let show = |s: &str| if s.is_empty() { "—".to_string() } else { s.to_string() };
            println!("  {name}: [{}] -> [{}]", show(&ci_old), show(&ci));
            updates.push((id, ci));
        }
    }
    if dry_run {
        println!("recolor (dry-run): {} card(s) would change", updates.len());
        return Ok(());
    }
    let tx = db.transaction()?;
    for (id, ci) in &updates {
        tx.execute(
            "UPDATE cube_cards SET color_identity=?1, updated_at=datetime('now') WHERE id=?2",
            params![ci, id],
        )?;
    }
    tx.commit()?;
    println!("recolor: {} card(s) updated", updates.len());
    Ok(())
}

const INDEX_HTML: &str = include_str!("edit_ui.html");
