//! Append-only event log for GenAI art decisions.
//!
//! This is the **source of truth** for everything we do per card in the GenAI
//! flow — every art-direction (re)generation, every prompt edit, every image
//! generated (or failed), every choice and un-choice — so a decision is never
//! lost and the whole history can be time-travelled by querying. The rendered
//! `art.json`/`art.png` next to each card are just a *materialised view* of the
//! latest `chosen` event.
//!
//! Stored in `<cache>/genai.sqlite` (WAL, so the async full-pass worker and the
//! review server can both append concurrently). Rows are only ever inserted —
//! never updated or deleted.

use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

/// Event kinds (the `action` column).
pub const DIRECTION_SET: &str = "direction_set"; // art direction generated or edited (prompt = text)
pub const ART_GENERATED: &str = "art_generated"; // one artist's image produced (artist, art_hash)
pub const ART_FAILED: &str = "art_failed"; // generation failed (artist, detail = error)
pub const CHOSEN: &str = "chosen"; // user picked an artist's art (artist, art_hash)
pub const UNCHOSEN: &str = "unchosen"; // user cleared the choice

/// Open (and migrate) the genai event store.
pub fn open(cache_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(cache_dir)?;
    let db = Connection::open(cache_dir.join("genai.sqlite"))?;
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA busy_timeout=10000;
         CREATE TABLE IF NOT EXISTS events (
             id        INTEGER PRIMARY KEY AUTOINCREMENT,
             ts        TEXT NOT NULL DEFAULT (datetime('now')),
             card_id   INTEGER NOT NULL,
             card_name TEXT NOT NULL,
             action    TEXT NOT NULL,
             artist    TEXT,
             art_hash  TEXT,
             prompt    TEXT,        -- the art direction in force at the time (for reprompt/time-travel)
             detail    TEXT         -- JSON misc (errors, source event id on revert, etc.)
         );
         CREATE INDEX IF NOT EXISTS idx_events_card ON events(card_id);
         CREATE INDEX IF NOT EXISTS idx_events_action ON events(action);",
    )?;
    Ok(db)
}

/// Append one event. Append-only — this is the only write.
#[allow(clippy::too_many_arguments)]
pub fn log(
    db: &Connection,
    card_id: i64,
    card_name: &str,
    action: &str,
    artist: Option<&str>,
    art_hash: Option<&str>,
    prompt: Option<&str>,
    detail: Option<&str>,
) -> Result<()> {
    db.execute(
        "INSERT INTO events (card_id, card_name, action, artist, art_hash, prompt, detail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![card_id, card_name, action, artist, art_hash, prompt, detail],
    )?;
    Ok(())
}

/// The current (latest) art direction for a card, if any was ever set.
pub fn current_direction(db: &Connection, card_id: i64) -> Result<Option<String>> {
    Ok(db
        .query_row(
            "SELECT prompt FROM events WHERE card_id=?1 AND action=?2 ORDER BY id DESC LIMIT 1",
            params![card_id, DIRECTION_SET],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}

/// The current choice `(artist, art_hash)` — the latest `chosen` unless a later
/// `unchosen` cleared it.
pub fn current_choice(db: &Connection, card_id: i64) -> Result<Option<(String, String)>> {
    let row = db
        .query_row(
            "SELECT action, artist, art_hash FROM events
               WHERE card_id=?1 AND action IN (?2, ?3) ORDER BY id DESC LIMIT 1",
            params![card_id, CHOSEN, UNCHOSEN],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?;
    Ok(match row {
        Some((a, Some(artist), Some(hash))) if a == CHOSEN => Some((artist, hash)),
        _ => None,
    })
}

/// Full ordered history for a card (for the review UI / time-travel).
pub fn history(db: &Connection, card_id: i64) -> Result<Vec<Value>> {
    let mut stmt = db.prepare(
        "SELECT id, ts, action, artist, art_hash, prompt, detail
           FROM events WHERE card_id=?1 ORDER BY id",
    )?;
    let rows = stmt
        .query_map(params![card_id], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "ts": r.get::<_, String>(1)?,
                "action": r.get::<_, String>(2)?,
                "artist": r.get::<_, Option<String>>(3)?,
                "art_hash": r.get::<_, Option<String>>(4)?,
                "prompt": r.get::<_, Option<String>>(5)?,
                "detail": r.get::<_, Option<String>>(6)?,
            }))
        })?
        .filter_map(std::result::Result::ok)
        .collect();
    Ok(rows)
}

/// The `prompt` of a specific past event (for "reprompt this old direction").
pub fn prompt_of(db: &Connection, event_id: i64) -> Result<Option<String>> {
    Ok(db
        .query_row("SELECT prompt FROM events WHERE id=?1", params![event_id], |r| {
            r.get::<_, Option<String>>(0)
        })
        .optional()?
        .flatten())
}

/// Per-card rollup for the review dashboard: how many arts generated, whether a
/// direction exists, and the current choice.
pub fn status(db: &Connection) -> Result<std::collections::HashMap<i64, Value>> {
    let mut out = std::collections::HashMap::new();
    let mut stmt = db.prepare(
        "SELECT card_id,
                SUM(action='art_generated'),
                MAX(action='direction_set'),
                COUNT(*)
           FROM events GROUP BY card_id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, Option<i64>>(1)?.unwrap_or(0),
            r.get::<_, Option<i64>>(2)?.unwrap_or(0),
            r.get::<_, i64>(3)?,
        ))
    })?;
    for row in rows.flatten() {
        let (card_id, generated, has_dir, total) = row;
        let choice = current_choice(db, card_id)?;
        out.insert(
            card_id,
            json!({
                "generated": generated,
                "has_direction": has_dir != 0,
                "events": total,
                "chosen_artist": choice.map(|(a, _)| a),
            }),
        );
    }
    Ok(out)
}
