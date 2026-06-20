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
pub fn seed(mtg_db: &Path, list_path: &Path, out: &Path, force: bool) -> Result<()> {
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
    for (idx, name) in names.iter().enumerate() {
        if let Some(c) = lookup_card(&src, name)? {
            found += 1;
            let is_creature = c.type_line.contains("Creature");
            let printings: Vec<&str> = c.printings.split(", ").collect();
            let odyssey_creature =
                is_creature && printings.iter().any(|p| ODYSSEY_BLOCK.contains(p));
            let modern_frame = !printings.iter().any(|p| OLD_FRAME_SETS.contains(p));
            if odyssey_creature {
                approved += 1;
            }
            if modern_frame {
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
                    i64::from(modern_frame), // genai_art (pre-set)
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
    let mut stmt = src.prepare_cached(
        "SELECT mana_cost, mana_value, type, colors, color_identity, power, toughness,
                loyalty, text, keywords, produced_mana, printings, cube_elo, edhrec_rank
           FROM cards WHERE name = ?1 ORDER BY COALESCE(face_index, 0) LIMIT 1",
    )?;
    let mut rows = stmt.query(params![name])?;
    let Some(r) = rows.next()? else {
        return Ok(None);
    };
    Ok(Some(CardRow {
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
    }))
}

fn read_list(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in text.lines() {
        let n = line.trim();
        if n.is_empty() || n.starts_with('#') {
            continue;
        }
        if seen.insert(n.to_string()) {
            out.push(n.to_string());
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
    overrides       TEXT NOT NULL DEFAULT '{}',  -- JSON: per-field errata {field: new value}
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

/// Serve the editor UI on `127.0.0.1:port`.
pub fn serve(editor_db: &Path, port: u16) -> Result<()> {
    if !editor_db.exists() {
        bail!(
            "{} not found — run `mtgbrain edit seed` first",
            editor_db.display()
        );
    }
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::http(addr).map_err(|e| anyhow::anyhow!("starting server: {e}"))?;
    println!(
        "cube editor: http://127.0.0.1:{port}/   (db: {})",
        editor_db.display()
    );
    println!("Ctrl-C to stop.");

    for mut req in server.incoming_requests() {
        let method = req.method().clone();
        let url = req.url().to_string();
        let path = url.split('?').next().unwrap_or("").to_string();

        let resp = match (&method, path.as_str()) {
            (Method::Get, "/") => html_response(INDEX_HTML),
            (Method::Get, "/api/cards") => json_response(&list_cards(&db)),
            (Method::Get, p) if p.starts_with("/api/card/") => match id_from(p, "/api/card/") {
                Some(id) => match get_card(&db, id) {
                    Some(v) => json_response(&v),
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
            _ => not_found(),
        };
        let _ = req.respond(resp);
    }
    Ok(())
}

fn id_from(path: &str, prefix: &str) -> Option<i64> {
    path.strip_prefix(prefix)?.parse().ok()
}

fn list_cards(db: &Connection) -> Value {
    let mut stmt = db
        .prepare(
            "SELECT id,name,type,mana_cost,cube_elo,decision,genai_art,
                    is_odysseyblock_creature,in_db_found
               FROM cube_cards ORDER BY id",
        )
        .expect("prepare list");
    let rows = stmt
        .query_map([], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "type": r.get::<_, Option<String>>(2)?,
                "mana_cost": r.get::<_, Option<String>>(3)?,
                "cube_elo": r.get::<_, Option<f64>>(4)?,
                "decision": r.get::<_, String>(5)?,
                "genai_art": r.get::<_, i64>(6)? != 0,
                "odyssey_creature": r.get::<_, i64>(7)? != 0,
                "found": r.get::<_, i64>(8)? != 0,
            }))
        })
        .expect("query list")
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();

    let counts = db
        .query_row(
            "SELECT
                COUNT(*),
                SUM(decision='accepted'),
                SUM(decision='errata'),
                SUM(decision='pending'),
                SUM(genai_art)
             FROM cube_cards",
            [],
            |r| {
                Ok(json!({
                    "total":   r.get::<_, i64>(0)?,
                    "accepted":r.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    "errata":  r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    "pending": r.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    "genai":   r.get::<_, Option<i64>>(4)?.unwrap_or(0),
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
                in_db_found,updated_at,overrides
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
    if let Some(o) = v.get("overrides") {
        if !o.is_object() {
            bail!("overrides must be a JSON object");
        }
        db.execute(
            "UPDATE cube_cards SET overrides=?2, updated_at=datetime('now') WHERE id=?1",
            params![id, o.to_string()],
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

// Default DB path next to the source list.
pub fn default_editor_db(data_dir: &Path) -> PathBuf {
    data_dir.join("cube_editor.sqlite")
}

const INDEX_HTML: &str = include_str!("edit_ui.html");
