//! Maybeboard modules — batched card research for the cube.
//!
//! A *module* is one research question ("threshold matters", "cycling payoffs")
//! captured as a Scryfall-style query plus the frozen list of candidate cards it
//! matched, stored in the editor DB next to the cube. Three modes of operation:
//!
//! 1. **Launch** — run a query (in the `/maybe` UI over `/api/pool`, or via
//!    `mtgbrain edit maybe-launch` with an explicit name list / SQL) and snapshot
//!    every matching card that is not already in the cube.
//! 2. **Review** — walk the module as a visual spoiler grid and mark each card
//!    `want` / `rejected`; promoting a module adds every `want` to the cube
//!    (tagged `maybe:<module>`) and appends it to the source list.
//! 3. **Trim** — the `/trim` grid over the live cube for cutting back down to the
//!    print target (612) while keeping the creature / non-creature balance.
//!
//! The query itself is evaluated client-side (the same Scryfall-ish engine the
//! editor's sidebar search uses) over `/api/pool` — the whole card corpus with
//! `in_cube` / `oldframe` flags — so the server only ever stores *names*.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Value};

/// Editor-DB tables for maybeboard research. Idempotent; called at serve boot
/// and before any CLI launch so older editor DBs pick the feature up unchanged.
pub fn ensure_schema(db: &Connection) -> Result<()> {
    db.execute_batch(
        r"
CREATE TABLE IF NOT EXISTS maybe_modules (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    query       TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT '',
    created_at  TEXT,
    updated_at  TEXT
);
CREATE TABLE IF NOT EXISTS maybe_cards (
    module_id   INTEGER NOT NULL REFERENCES maybe_modules(id),
    name        TEXT NOT NULL,
    mana_cost   TEXT,
    mana_value  REAL,
    type        TEXT,
    colors      TEXT,
    color_identity TEXT,
    power       TEXT,
    toughness   TEXT,
    loyalty     TEXT,
    oracle_text TEXT,
    keywords    TEXT,
    produced_mana TEXT,
    printings   TEXT,
    cube_elo    REAL,
    edhrec_rank INTEGER,
    is_creature INTEGER NOT NULL DEFAULT 0,
    is_token_gen INTEGER NOT NULL DEFAULT 0,
    is_oldframe INTEGER NOT NULL DEFAULT 0,
    verdict     TEXT NOT NULL DEFAULT 'pending',  -- pending | want | rejected | added
    updated_at  TEXT,
    PRIMARY KEY (module_id, name)
);
",
    )?;
    Ok(())
}

/// "Body-like" card: makes creatures even if it isn't one. The cube is
/// deliberately creature-light (CREATURE-COUNT.md), so token generators count
/// toward board presence in every grid's stats.
fn is_token_gen(oracle_text: &str) -> bool {
    oracle_text.to_lowercase().contains("creature token")
}

/// A card counts as old-frame when any printing predates 8th Edition (mid-2003).
fn is_oldframe(printings: &str) -> bool {
    printings
        .split(", ")
        .any(|p| crate::edit::OLD_FRAME_SETS.contains(&p))
}

/// Names of every ACTIVE cube card (soft-removed ones excluded — a card the user
/// trimmed is fair game for rediscovery in a later module).
fn cube_names(editor_db: &Path) -> Result<HashSet<String>> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening {}", editor_db.display()))?;
    let mut stmt = db.prepare("SELECT name FROM cube_cards WHERE removed=0")?;
    let names = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(std::result::Result::ok)
        .collect();
    Ok(names)
}

// ---------------------------------------------------------------------------
// /api/pool — the whole card corpus, shaped like the editor's card objects
// ---------------------------------------------------------------------------

/// Every card in `mtg.sqlite` (faces collapsed to one object per name, silver
/// border and Alchemy rebalances excluded), each flagged `in_cube` / `oldframe`,
/// shaped so the client-side Scryfall-style query engine can filter it directly.
/// ~35k cards / ~15 MB of JSON — fetched once per `/maybe` page load, locally.
pub fn pool(source_db: &Path, editor_db: &Path) -> Result<Value> {
    struct Agg {
        name: String,
        mana_cost: Option<String>,
        mana_value: Option<f64>,
        type_line: String,
        colors: String,
        color_identity: Option<String>,
        power: Option<String>,
        toughness: Option<String>,
        loyalty: Option<String>,
        text: String,
        keywords: Option<String>,
        produced_mana: Option<String>,
        printings: String,
        cube_elo: Option<f64>,
        edhrec_rank: Option<i64>,
    }

    let in_cube = cube_names(editor_db)?;
    let src = Connection::open_with_flags(source_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening source DB {}", source_db.display()))?;

    let mut stmt = src.prepare(
        "SELECT name, mana_cost, mana_value, type, colors, color_identity, power, toughness,
                loyalty, text, keywords, produced_mana, printings, cube_elo, edhrec_rank
           FROM cards
          WHERE is_funny = 0 AND name NOT LIKE 'A-%'
          ORDER BY name, COALESCE(face_index, 0)",
    )?;

    let mut cards: Vec<Value> = Vec::with_capacity(36_000);
    let mut cur: Option<Agg> = None;
    let flush = |a: &Agg, out: &mut Vec<Value>| {
        out.push(json!({
            "name": a.name,
            "mana_cost": a.mana_cost,
            "mana_value": a.mana_value,
            "type": a.type_line,
            "colors": a.colors,
            "color_identity": a.color_identity,
            "power": a.power,
            "toughness": a.toughness,
            "loyalty": a.loyalty,
            "oracle_text": a.text,
            "keywords": a.keywords,
            "produced_mana": a.produced_mana,
            "printings": a.printings,
            "cube_elo": a.cube_elo,
            "edhrec_rank": a.edhrec_rank,
            "is_creature": a.type_line.contains("Creature"),
            "oldframe": is_oldframe(&a.printings),
            "in_cube": in_cube.contains(&a.name),
        }));
    };

    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let name: String = r.get(0)?;
        let type_line: String = r.get::<_, Option<String>>(3)?.unwrap_or_default();
        let colors: String = r.get::<_, Option<String>>(4)?.unwrap_or_default();
        let text: String = r.get::<_, Option<String>>(9)?.unwrap_or_default();
        match cur.as_mut() {
            // Another face of the same card: merge the searchable text fields.
            Some(a) if a.name == name => {
                if !type_line.is_empty() && !a.type_line.contains(&type_line) {
                    a.type_line = format!("{} // {}", a.type_line, type_line);
                }
                if !text.is_empty() {
                    if !a.text.is_empty() {
                        a.text.push_str("\n//\n");
                    }
                    a.text.push_str(&text);
                }
                for ch in colors.chars().filter(|c| "WUBRG".contains(*c)) {
                    if !a.colors.contains(ch) {
                        a.colors.push(ch);
                    }
                }
            }
            _ => {
                if let Some(a) = cur.take() {
                    flush(&a, &mut cards);
                }
                cur = Some(Agg {
                    name,
                    mana_cost: r.get(1)?,
                    mana_value: r.get(2)?,
                    type_line,
                    colors: colors.chars().filter(|c| "WUBRG".contains(*c)).collect(),
                    color_identity: r.get(5)?,
                    power: r.get(6)?,
                    toughness: r.get(7)?,
                    loyalty: r.get(8)?,
                    text,
                    keywords: r.get(10)?,
                    produced_mana: r.get(11)?,
                    printings: r.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    cube_elo: r.get(13)?,
                    edhrec_rank: r.get(14)?,
                });
            }
        }
    }
    if let Some(a) = cur.take() {
        flush(&a, &mut cards);
    }
    Ok(json!({ "cards": cards, "in_cube_total": in_cube.len() }))
}

// ---------------------------------------------------------------------------
// launch / list / review / promote
// ---------------------------------------------------------------------------

/// Create a module and snapshot `names` into it from `source_db`. Names already
/// in the ACTIVE cube are skipped (they're not "maybes"), unknown names are
/// reported back, duplicates collapse. Returns the module summary.
pub fn launch(
    editor_db: &Path,
    source_db: &Path,
    module_name: &str,
    query: &str,
    notes: &str,
    names: &[String],
) -> Result<Value> {
    let module_name = module_name.trim();
    if module_name.is_empty() {
        bail!("module name is empty");
    }
    if names.is_empty() {
        bail!("no candidate card names given");
    }
    let in_cube = cube_names(editor_db)?;
    let src = Connection::open_with_flags(source_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening source DB {}", source_db.display()))?;
    let mut db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    ensure_schema(&db)?;

    let exists: i64 = db.query_row(
        "SELECT COUNT(*) FROM maybe_modules WHERE name=?1",
        params![module_name],
        |r| r.get(0),
    )?;
    if exists > 0 {
        bail!("module '{module_name}' already exists — pick another name or delete it first");
    }

    let tx = db.transaction()?;
    tx.execute(
        "INSERT INTO maybe_modules (name, query, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        params![module_name, query.trim(), notes.trim()],
    )?;
    let module_id = tx.last_insert_rowid();

    let (mut added, mut skipped_cube) = (0u32, 0u32);
    let mut missing: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for raw in names {
        let name = raw.trim();
        if name.is_empty() || !seen.insert(name.to_string()) {
            continue;
        }
        if in_cube.contains(name) {
            skipped_cube += 1;
            continue;
        }
        let Some(c) = crate::edit::lookup_card(&src, name)? else {
            missing.push(name.to_string());
            continue;
        };
        let oracle = c.oracle_text.clone().unwrap_or_default();
        tx.execute(
            "INSERT OR IGNORE INTO maybe_cards
                (module_id, name, mana_cost, mana_value, type, colors, color_identity,
                 power, toughness, loyalty, oracle_text, keywords, produced_mana, printings,
                 cube_elo, edhrec_rank, is_creature, is_token_gen, is_oldframe, verdict, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,'pending',datetime('now'))",
            params![
                module_id,
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
                i64::from(c.type_line.contains("Creature")),
                i64::from(is_token_gen(&oracle)),
                i64::from(is_oldframe(&c.printings)),
            ],
        )?;
        added += 1;
    }
    tx.commit()?;

    Ok(json!({
        "id": module_id,
        "name": module_name,
        "cards": added,
        "skipped_in_cube": skipped_cube,
        "missing": missing,
    }))
}

/// Every module with its verdict tallies, newest first.
pub fn modules(editor_db: &Path) -> Result<Value> {
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    ensure_schema(&db)?;
    let mut stmt = db.prepare(
        "SELECT m.id, m.name, m.query, m.notes, m.created_at,
                COUNT(c.name),
                COALESCE(SUM(c.verdict='pending'),0),
                COALESCE(SUM(c.verdict='want'),0),
                COALESCE(SUM(c.verdict='rejected'),0),
                COALESCE(SUM(c.verdict='added'),0)
           FROM maybe_modules m
           LEFT JOIN maybe_cards c ON c.module_id = m.id
          GROUP BY m.id
          ORDER BY m.id DESC",
    )?;
    let mods = stmt
        .query_map([], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "query": r.get::<_, String>(2)?,
                "notes": r.get::<_, String>(3)?,
                "created_at": r.get::<_, Option<String>>(4)?,
                "total": r.get::<_, i64>(5)?,
                "pending": r.get::<_, i64>(6)?,
                "want": r.get::<_, i64>(7)?,
                "rejected": r.get::<_, i64>(8)?,
                "added": r.get::<_, i64>(9)?,
            }))
        })?
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();
    Ok(json!({ "modules": mods }))
}

/// One module + its full card snapshots (the review grid's data).
pub fn module_cards(editor_db: &Path, module_id: i64) -> Result<Value> {
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    ensure_schema(&db)?;
    let meta = db
        .query_row(
            "SELECT name, query, notes, created_at FROM maybe_modules WHERE id=?1",
            params![module_id],
            |r| {
                Ok(json!({
                    "id": module_id,
                    "name": r.get::<_, String>(0)?,
                    "query": r.get::<_, String>(1)?,
                    "notes": r.get::<_, String>(2)?,
                    "created_at": r.get::<_, Option<String>>(3)?,
                }))
            },
        )
        .with_context(|| format!("module {module_id} not found"))?;
    let mut stmt = db.prepare(
        "SELECT name, mana_cost, mana_value, type, colors, color_identity, power, toughness,
                loyalty, oracle_text, keywords, produced_mana, printings, cube_elo, edhrec_rank,
                is_creature, is_token_gen, is_oldframe, verdict
           FROM maybe_cards WHERE module_id=?1 ORDER BY name",
    )?;
    let cards = stmt
        .query_map(params![module_id], |r| {
            Ok(json!({
                "name": r.get::<_, String>(0)?,
                "mana_cost": r.get::<_, Option<String>>(1)?,
                "mana_value": r.get::<_, Option<f64>>(2)?,
                "type": r.get::<_, Option<String>>(3)?,
                "colors": r.get::<_, Option<String>>(4)?,
                "color_identity": r.get::<_, Option<String>>(5)?,
                "power": r.get::<_, Option<String>>(6)?,
                "toughness": r.get::<_, Option<String>>(7)?,
                "loyalty": r.get::<_, Option<String>>(8)?,
                "oracle_text": r.get::<_, Option<String>>(9)?,
                "keywords": r.get::<_, Option<String>>(10)?,
                "produced_mana": r.get::<_, Option<String>>(11)?,
                "printings": r.get::<_, Option<String>>(12)?,
                "cube_elo": r.get::<_, Option<f64>>(13)?,
                "edhrec_rank": r.get::<_, Option<i64>>(14)?,
                "is_creature": r.get::<_, i64>(15)? != 0,
                "is_token_gen": r.get::<_, i64>(16)? != 0,
                "oldframe": r.get::<_, i64>(17)? != 0,
                "verdict": r.get::<_, String>(18)?,
            }))
        })?
        .filter_map(std::result::Result::ok)
        .collect::<Vec<_>>();
    Ok(json!({ "module": meta, "cards": cards }))
}

/// Record a review verdict for one card of a module. `added` is promote-only —
/// a promoted card can't be un-promoted from the grid (remove it from the cube
/// in the editor instead), so both directions involving 'added' are refused.
pub fn set_verdict(editor_db: &Path, module_id: i64, card: &str, verdict: &str) -> Result<Value> {
    if !matches!(verdict, "pending" | "want" | "rejected") {
        bail!("invalid verdict: {verdict}");
    }
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    let current: String = db
        .query_row(
            "SELECT verdict FROM maybe_cards WHERE module_id=?1 AND name=?2",
            params![module_id, card],
            |r| r.get(0),
        )
        .with_context(|| format!("'{card}' not in module {module_id}"))?;
    if current == "added" {
        bail!("'{card}' was already promoted to the cube — remove it in the editor instead");
    }
    db.execute(
        "UPDATE maybe_cards SET verdict=?3, updated_at=datetime('now')
          WHERE module_id=?1 AND name=?2",
        params![module_id, card, verdict],
    )?;
    db.execute(
        "UPDATE maybe_modules SET updated_at=datetime('now') WHERE id=?1",
        params![module_id],
    )?;
    Ok(json!({ "ok": true, "name": card, "verdict": verdict }))
}

/// Promote every `want` in the module into the cube: `edit::add_card` snapshots
/// it (same pre-sets as seeding), tags it `maybe:<module>`, and appends it to the
/// source cube list. Cards that raced into the cube meanwhile count as skipped;
/// both ways the verdict becomes `added`. Failures stay `want` for a retry.
pub fn promote(editor_db: &Path, source_db: &Path, module_id: i64) -> Result<Value> {
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    ensure_schema(&db)?;
    let module_name: String = db
        .query_row(
            "SELECT name FROM maybe_modules WHERE id=?1",
            params![module_id],
            |r| r.get(0),
        )
        .with_context(|| format!("module {module_id} not found"))?;
    let tag = format!(
        "maybe:{}",
        module_name.replace(',', " ").trim().replace(' ', "-")
    );
    let wants: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT name FROM maybe_cards WHERE module_id=?1 AND verdict='want' ORDER BY name",
        )?;
        let v = stmt
            .query_map(params![module_id], |r| r.get::<_, String>(0))?
            .filter_map(std::result::Result::ok)
            .collect();
        v
    };
    let (mut added, mut skipped, mut failed) = (Vec::new(), Vec::new(), Vec::new());
    for name in &wants {
        match crate::edit::add_card(&db, source_db, name, &tag) {
            Ok(_) => added.push(name.clone()),
            Err(e) if e.to_string().contains("already in the cube") => skipped.push(name.clone()),
            Err(e) => {
                failed.push(json!({ "name": name, "error": e.to_string() }));
                continue;
            }
        }
        db.execute(
            "UPDATE maybe_cards SET verdict='added', updated_at=datetime('now')
              WHERE module_id=?1 AND name=?2",
            params![module_id, name],
        )?;
    }
    db.execute(
        "UPDATE maybe_modules SET updated_at=datetime('now') WHERE id=?1",
        params![module_id],
    )?;
    Ok(json!({
        "module": module_name,
        "added": added,
        "skipped_already_in_cube": skipped,
        "failed": failed,
    }))
}

/// Delete a module and its snapshots. Promoted cards stay in the cube — this
/// only discards the research batch itself.
pub fn delete_module(editor_db: &Path, module_id: i64) -> Result<Value> {
    let db =
        Connection::open(editor_db).with_context(|| format!("opening {}", editor_db.display()))?;
    db.execute(
        "DELETE FROM maybe_cards WHERE module_id=?1",
        params![module_id],
    )?;
    let n = db.execute("DELETE FROM maybe_modules WHERE id=?1", params![module_id])?;
    if n == 0 {
        bail!("module {module_id} not found");
    }
    Ok(json!({ "ok": true, "deleted": module_id }))
}

// ---------------------------------------------------------------------------
// card images for the grids
// ---------------------------------------------------------------------------

/// A spoiler grid fires dozens of tile requests at once, each handled on its own
/// server thread — the per-request curl gate spaces request *starts* but not whole
/// downloads, so unserialised they still hit Scryfall together and it 429s most of
/// them. This lock makes the misses queue politely instead.
static SCRYFALL_IMG_LOCK: Mutex<()> = Mutex::new(());

/// Scryfall `normal` (488×680 JPG) card image, cached on disk forever. Grid
/// tiles for cards we have no render of (maybeboard candidates, un-rendered
/// cube cards). Downloads via the throttled curl helper; `.part` + rename keeps
/// a concurrent double-fetch from serving a torn file.
pub fn scryfall_normal(cache_dir: &Path, name: &str) -> Result<PathBuf> {
    let dir = cache_dir.join("scryfall-normal");
    fs::create_dir_all(&dir)?;
    let dst = dir.join(format!("{}.jpg", crate::render::sanitize(name)));
    if dst.exists() {
        return Ok(dst);
    }
    let _serialised = SCRYFALL_IMG_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if dst.exists() {
        return Ok(dst); // a queued waiter often finds the work already done
    }
    let part = dst.with_extension("jpg.part");
    let url = |kind: &str| {
        format!(
            "https://api.scryfall.com/cards/named?{kind}={}&format=image&version=normal",
            crate::render::percent(name)
        )
    };
    if crate::render::curl_to_file(&url("exact"), &part).is_err() {
        crate::render::curl_to_file(&url("fuzzy"), &part)
            .with_context(|| format!("Scryfall image for '{name}'"))?;
    }
    fs::rename(&part, &dst)?;
    Ok(dst)
}

/// Best available grid thumbnail for an editor-DB card: the cached print render's
/// preview JPEG when one exists (the cube's real face, errata and all — never
/// triggers a fresh render), else the Scryfall normal image.
pub fn thumb_for_card(
    editor_db: &Path,
    cache_dir: &Path,
    id: i64,
    eighth: bool,
) -> Result<PathBuf> {
    let db = Connection::open_with_flags(editor_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("opening {}", editor_db.display()))?;
    let name: String = db
        .query_row(
            "SELECT name FROM cube_cards WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .with_context(|| format!("card {id} not found"))?;
    let dir = crate::render::frame_card_dir(cache_dir, &name, eighth);
    if let Some(png) = crate::render::forefront_render(&dir) {
        if let Ok(preview) = crate::render::card_preview(&png, 480) {
            return Ok(preview);
        }
    }
    scryfall_normal(cache_dir, &name)
}
