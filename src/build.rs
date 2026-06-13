//! Build the queryable SQLite DB from MTGJSON's `AtomicCards.json`.
//!
//! One row per card FACE in `cards`. Array fields are stored three ways so any
//! query style works: a readable joined column (for LIKE), normalized child
//! tables (for exact membership / JOINs), and inside the FTS5 index (for ranked
//! full-text search). Numeric *_num columns parse "*"/"X" stats to NULL.

use std::collections::HashSet;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;

use crate::model::{AtomicFile, Card};

const COLOR_ORDER: [&str; 6] = ["W", "U", "B", "R", "G", "C"];

const SCHEMA: &str = r"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);

-- One row per card FACE. Single-face cards have face_name = NULL.
CREATE TABLE cards (
    id                      INTEGER PRIMARY KEY,
    name                    TEXT NOT NULL,   -- full card name, e.g. 'Fire // Ice'
    face_name               TEXT,            -- this face's name (NULL if single-face)
    display_name            TEXT NOT NULL,   -- face_name if present else name (use in output)
    side                    TEXT,            -- 'a' / 'b' / ... for multi-face cards
    face_index              INTEGER,         -- 0-based index of this face within the card
    num_faces               INTEGER,
    layout                  TEXT,            -- normal, transform, split, modal_dfc, adventure, ...

    mana_cost               TEXT,            -- e.g. '{2}{U}{U}'
    mana_value              REAL,            -- a.k.a. converted mana cost (whole card)
    face_mana_value         REAL,

    colors                  TEXT,            -- WUBRG-sorted, e.g. 'U' or 'U, R'
    color_count             INTEGER,
    color_identity          TEXT,            -- WUBRG-sorted commander color identity
    color_identity_count    INTEGER,
    is_colorless            INTEGER,
    is_multicolor           INTEGER,

    type                    TEXT,            -- full type line
    supertypes              TEXT,
    types                   TEXT,
    subtypes                TEXT,

    power                   TEXT,            -- raw (may be '*', '1+*')
    power_num               REAL,            -- parsed, NULL if non-numeric
    toughness               TEXT,
    toughness_num           REAL,
    loyalty                 TEXT,
    loyalty_num             REAL,
    defense                 TEXT,
    defense_num             REAL,

    text                    TEXT,            -- oracle text (the main thing to search)
    keywords                TEXT,            -- 'Flying, Vigilance'
    produced_mana           TEXT,            -- 'G, W'
    has_text                INTEGER,

    num_printings           INTEGER,         -- reprint/popularity proxy
    printings               TEXT,            -- comma-joined set codes
    edhrec_rank             INTEGER,         -- lower = more played; NULL if unranked
    edhrec_saltiness        REAL,

    is_reserved             INTEGER,
    is_funny                INTEGER,         -- Un-set / acorn / silver-border
    is_game_changer         INTEGER,         -- Commander 'Game Changer' list
    can_be_commander        INTEGER,
    can_be_brawl_commander  INTEGER,
    can_be_oathbreaker      INTEGER,

    hand                    TEXT,            -- Vanguard starting-hand modifier (e.g. '+1')
    life                    TEXT,            -- Vanguard starting-life modifier

    scryfall_oracle_id      TEXT,
    legalities              TEXT             -- JSON: {format: status}
);

CREATE TABLE card_colors          (card_id INTEGER, color     TEXT);
CREATE TABLE card_color_identity  (card_id INTEGER, color     TEXT);
CREATE TABLE card_types           (card_id INTEGER, type      TEXT);
CREATE TABLE card_subtypes        (card_id INTEGER, subtype   TEXT);
CREATE TABLE card_supertypes      (card_id INTEGER, supertype TEXT);
CREATE TABLE card_keywords        (card_id INTEGER, keyword   TEXT);
CREATE TABLE card_produced_mana   (card_id INTEGER, mana      TEXT);
CREATE TABLE card_legalities      (card_id INTEGER, format    TEXT, status TEXT);

CREATE VIRTUAL TABLE cards_fts USING fts5(
    display_name, name, type, text, keywords,
    content='', tokenize='unicode61 remove_diacritics 2'
);
";

const INDEXES: &str = r"
CREATE INDEX idx_cards_name          ON cards(name);
CREATE INDEX idx_cards_display_name  ON cards(display_name);
CREATE INDEX idx_cards_mana_value    ON cards(mana_value);
CREATE INDEX idx_cards_edhrec_rank   ON cards(edhrec_rank);
CREATE INDEX idx_cards_power_num     ON cards(power_num);
CREATE INDEX idx_cards_toughness_num ON cards(toughness_num);
CREATE INDEX idx_cards_loyalty_num   ON cards(loyalty_num);

CREATE INDEX idx_colors_card   ON card_colors(card_id);
CREATE INDEX idx_colors_color  ON card_colors(color);
CREATE INDEX idx_ci_card       ON card_color_identity(card_id);
CREATE INDEX idx_ci_color      ON card_color_identity(color);
CREATE INDEX idx_types_card    ON card_types(card_id);
CREATE INDEX idx_types_type    ON card_types(type);
CREATE INDEX idx_subtypes_card ON card_subtypes(card_id);
CREATE INDEX idx_subtypes_sub  ON card_subtypes(subtype);
CREATE INDEX idx_supertypes_card ON card_supertypes(card_id);
CREATE INDEX idx_supertypes_sup  ON card_supertypes(supertype);
CREATE INDEX idx_keywords_card ON card_keywords(card_id);
CREATE INDEX idx_keywords_kw   ON card_keywords(keyword);
CREATE INDEX idx_pmana_card    ON card_produced_mana(card_id);
CREATE INDEX idx_pmana_mana    ON card_produced_mana(mana);
CREATE INDEX idx_legal_card    ON card_legalities(card_id);
CREATE INDEX idx_legal_fmt     ON card_legalities(format, status);
";

fn color_key(c: &str) -> usize {
    COLOR_ORDER.iter().position(|&x| x == c).unwrap_or(99)
}

fn sort_colors(arr: &[String]) -> Vec<String> {
    let mut v = arr.to_vec();
    v.sort_by_key(|c| color_key(c));
    v
}

fn join_opt(arr: &[String]) -> Option<String> {
    if arr.is_empty() {
        None
    } else {
        Some(arr.join(", "))
    }
}

fn num(s: &Option<String>) -> Option<f64> {
    s.as_ref().and_then(|v| v.parse::<f64>().ok())
}

fn slice(o: &Option<Vec<String>>) -> &[String] {
    o.as_deref().unwrap_or(&[])
}

pub fn build(data_dir: &Path, db_path: &Path, with_rulings: bool) -> Result<()> {
    let t0 = Instant::now();
    let json_path = data_dir.join("AtomicCards.json");
    if !json_path.exists() {
        bail!(
            "{} not found. Run `mtgbrain download` first.",
            json_path.display()
        );
    }

    println!("Loading {} ...", json_path.display());
    let reader = std::io::BufReader::new(std::fs::File::open(&json_path)?);
    let parsed: AtomicFile =
        serde_json::from_reader(reader).context("parsing AtomicCards.json")?;
    println!(
        "  {} card names (MTGJSON {})",
        parsed.data.len(),
        parsed.meta.version
    );

    if db_path.exists() {
        std::fs::remove_file(db_path)?;
    }
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;")?;
    conn.execute_batch(SCHEMA)?;
    if with_rulings {
        conn.execute_batch(
            "CREATE TABLE rulings (card_name TEXT, date TEXT, text TEXT);
             CREATE VIRTUAL TABLE rulings_fts USING fts5(
                 card_name, text, content='', tokenize='unicode61 remove_diacritics 2');",
        )?;
    }

    conn.execute(
        "INSERT INTO meta VALUES ('mtgjson_version', ?1), ('mtgjson_date', ?2), ('source', 'AtomicCards')",
        params![parsed.meta.version, parsed.meta.date],
    )?;

    let tx = conn.transaction()?;
    let mut card_faces: i64 = 0;
    let mut ruling_count: i64 = 0;
    {
        let mut ins_card = tx.prepare(
            "INSERT INTO cards VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,\
             ?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,\
             ?37,?38,?39,?40,?41,?42,?43,?44,?45,?46,?47)",
        )?;
        let mut ins_fts = tx.prepare(
            "INSERT INTO cards_fts(rowid, display_name, name, type, text, keywords) VALUES (?,?,?,?,?,?)",
        )?;
        let mut ins_color = tx.prepare("INSERT INTO card_colors VALUES (?,?)")?;
        let mut ins_ci = tx.prepare("INSERT INTO card_color_identity VALUES (?,?)")?;
        let mut ins_type = tx.prepare("INSERT INTO card_types VALUES (?,?)")?;
        let mut ins_sub = tx.prepare("INSERT INTO card_subtypes VALUES (?,?)")?;
        let mut ins_super = tx.prepare("INSERT INTO card_supertypes VALUES (?,?)")?;
        let mut ins_kw = tx.prepare("INSERT INTO card_keywords VALUES (?,?)")?;
        let mut ins_pm = tx.prepare("INSERT INTO card_produced_mana VALUES (?,?)")?;
        let mut ins_legal = tx.prepare("INSERT INTO card_legalities VALUES (?,?,?)")?;
        let mut ins_ruling = if with_rulings {
            Some((
                tx.prepare("INSERT INTO rulings VALUES (?,?,?)")?,
                tx.prepare("INSERT INTO rulings_fts(rowid, card_name, text) VALUES (?,?,?)")?,
            ))
        } else {
            None
        };

        let mut cid: i64 = 0;

        for (name, faces) in &parsed.data {
            let num_faces = faces.len() as i64;
            for (fi, c) in faces.iter().enumerate() {
                cid += 1;
                card_faces = cid;

                let colors = sort_colors(slice(&c.colors));
                let ci = sort_colors(slice(&c.color_identity));
                let types = slice(&c.types);
                let subtypes = slice(&c.subtypes);
                let supertypes = slice(&c.supertypes);
                let keywords = slice(&c.keywords);
                let produced = sort_colors(slice(&c.produced_mana));
                let printings = slice(&c.printings);

                let display_name = c.face_name.clone().unwrap_or_else(|| name.clone());
                let colors_s = join_opt(&colors);
                let ci_s = join_opt(&ci);
                let supertypes_s = join_opt(supertypes);
                let types_s = join_opt(types);
                let subtypes_s = join_opt(subtypes);
                let keywords_s = join_opt(keywords);
                let produced_s = join_opt(&produced);
                let printings_s = join_opt(printings);
                let has_text = c.text.as_deref().is_some_and(|t| !t.trim().is_empty());
                let legal_json = c
                    .legalities
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap_or_default());
                let lead = c.leadership_skills.as_ref();
                let can_cmd = lead.and_then(|l| l.commander).unwrap_or(false);
                let can_brawl = lead.and_then(|l| l.brawl).unwrap_or(false);
                let can_oath = lead.and_then(|l| l.oathbreaker).unwrap_or(false);
                let scryfall = c
                    .identifiers
                    .as_ref()
                    .and_then(|i| i.scryfall_oracle_id.as_deref());

                ins_card.execute(params![
                    cid,
                    name,
                    c.face_name,
                    display_name,
                    c.side,
                    fi as i64,
                    num_faces,
                    c.layout,
                    c.mana_cost,
                    c.mana_value,
                    c.face_mana_value,
                    colors_s,
                    colors.len() as i64,
                    ci_s,
                    ci.len() as i64,
                    colors.is_empty(),
                    colors.len() >= 2,
                    c.type_line,
                    supertypes_s,
                    types_s,
                    subtypes_s,
                    c.power,
                    num(&c.power),
                    c.toughness,
                    num(&c.toughness),
                    c.loyalty,
                    num(&c.loyalty),
                    c.defense,
                    num(&c.defense),
                    c.text,
                    keywords_s,
                    produced_s,
                    has_text,
                    printings.len() as i64,
                    printings_s,
                    c.edhrec_rank,
                    c.edhrec_saltiness,
                    c.is_reserved.unwrap_or(false),
                    c.is_funny.unwrap_or(false),
                    c.is_game_changer.unwrap_or(false),
                    can_cmd,
                    can_brawl,
                    can_oath,
                    c.hand,
                    c.life,
                    scryfall,
                    legal_json,
                ])?;

                ins_fts.execute(params![
                    cid,
                    display_name,
                    name,
                    c.type_line.as_deref().unwrap_or(""),
                    c.text.as_deref().unwrap_or(""),
                    keywords_s.as_deref().unwrap_or(""),
                ])?;

                for v in &colors {
                    ins_color.execute(params![cid, v])?;
                }
                for v in &ci {
                    ins_ci.execute(params![cid, v])?;
                }
                for v in types {
                    ins_type.execute(params![cid, v])?;
                }
                for v in subtypes {
                    ins_sub.execute(params![cid, v])?;
                }
                for v in supertypes {
                    ins_super.execute(params![cid, v])?;
                }
                for v in keywords {
                    ins_kw.execute(params![cid, v])?;
                }
                for v in &produced {
                    ins_pm.execute(params![cid, v])?;
                }
                if let Some(m) = &c.legalities {
                    for (fmt, status) in m {
                        ins_legal.execute(params![cid, fmt, status])?;
                    }
                }
            }

            // Rulings belong to the card (per name), but MTGJSON may attach them to
            // any face (split cards repeat them, some put them only on the back).
            // Gather across ALL faces and dedupe by (date, text) content.
            if let Some((ins_r, ins_rf)) = ins_ruling.as_mut() {
                let mut seen: HashSet<(&str, &str)> = HashSet::new();
                for c in faces {
                    for r in slice_rulings(c) {
                        let date = r.date.as_deref().unwrap_or("");
                        let text = r.text.as_deref().unwrap_or("");
                        if seen.insert((date, text)) {
                            ruling_count += 1;
                            ins_r.execute(params![name, r.date, r.text])?;
                            ins_rf.execute(params![ruling_count, name, text])?;
                        }
                    }
                }
            }
        }
    }
    tx.commit()?;

    println!("  inserted {card_faces} card faces; creating indexes ...");
    conn.execute_batch(INDEXES)?;
    if with_rulings {
        conn.execute_batch("CREATE INDEX idx_rulings_name ON rulings(card_name);")?;
    }

    let cube_matched = apply_cube(&mut conn, data_dir)?;
    if cube_matched > 0 {
        println!("  CubeCobra Elo: matched {cube_matched} card faces");
    }

    conn.execute_batch("VACUUM;")?;

    let names: i64 = conn.query_row("SELECT COUNT(DISTINCT name) FROM cards", [], |r| r.get(0))?;
    let ranked: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE edhrec_rank IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    let size = std::fs::metadata(db_path)?.len();
    println!(
        "Done in {:.1}s -> {} ({:.1} MB)",
        t0.elapsed().as_secs_f64(),
        db_path.display(),
        size as f64 / 1e6
    );
    println!("  {card_faces} card faces / {names} distinct names; {ranked} with EDHREC rank");
    if with_rulings {
        println!("  rulings: {ruling_count}");
    }
    println!("Next: mtgbrain schema");
    Ok(())
}

fn slice_rulings(c: &Card) -> &[crate::model::Ruling] {
    c.rulings.as_deref().unwrap_or(&[])
}

#[derive(Deserialize)]
struct CubeRec {
    oracle_id: Option<String>,
    name_lower: Option<String>,
    elo: Option<f64>,
    cube_count: Option<i64>,
    pick_count: Option<i64>,
    popularity: Option<f64>,
}

/// Add the cube_* columns (always, so queries can reference them) and populate
/// them from cubecobra_elo.jsonl when present. Joins on Scryfall oracle_id, with
/// a name fallback. Returns how many card faces got an Elo rating.
fn apply_cube(conn: &mut Connection, data_dir: &Path) -> Result<i64> {
    conn.execute_batch(
        "ALTER TABLE cards ADD COLUMN cube_elo REAL;
         ALTER TABLE cards ADD COLUMN cube_count INTEGER;
         ALTER TABLE cards ADD COLUMN cube_pick_count INTEGER;
         ALTER TABLE cards ADD COLUMN cube_popularity REAL;",
    )?;

    let path = data_dir.join("cubecobra_elo.jsonl");
    if !path.exists() {
        return Ok(0);
    }

    conn.execute_batch(
        "CREATE TEMP TABLE cube(
            oracle_id TEXT, name_lower TEXT, elo REAL,
            cube_count INTEGER, pick_count INTEGER, popularity REAL);",
    )?;

    let reader = std::io::BufReader::new(std::fs::File::open(&path)?);
    let tx = conn.transaction()?;
    {
        let mut ins = tx.prepare("INSERT INTO cube VALUES (?,?,?,?,?,?)")?;
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let Ok(r) = serde_json::from_str::<CubeRec>(&line) else {
                continue;
            };
            ins.execute(params![
                r.oracle_id,
                r.name_lower,
                r.elo,
                r.cube_count,
                r.pick_count,
                r.popularity
            ])?;
        }
    }
    tx.commit()?;

    // UPDATE ... FROM (SQLite >= 3.33): match on oracle_id first, then by name.
    conn.execute_batch(
        "CREATE INDEX tmp_cube_oid  ON cube(oracle_id);
         CREATE INDEX tmp_cube_name ON cube(name_lower);
         UPDATE cards SET cube_elo=c.elo, cube_count=c.cube_count,
                          cube_pick_count=c.pick_count, cube_popularity=c.popularity
           FROM cube c
          WHERE c.oracle_id IS NOT NULL AND c.oracle_id = cards.scryfall_oracle_id;
         UPDATE cards SET cube_elo=c.elo, cube_count=c.cube_count,
                          cube_pick_count=c.pick_count, cube_popularity=c.popularity
           FROM cube c
          WHERE cards.cube_elo IS NULL AND c.name_lower = lower(cards.name);
         CREATE INDEX idx_cards_cube_elo   ON cards(cube_elo);
         CREATE INDEX idx_cards_cube_count ON cards(cube_count);
         DROP TABLE cube;",
    )?;

    conn.query_row("SELECT COUNT(*) FROM cards WHERE cube_elo IS NOT NULL", [], |r| {
        r.get(0)
    })
    .map_err(Into::into)
}
