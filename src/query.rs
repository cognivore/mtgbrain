//! Query the card DB. The primary interface for an LLM agent: arbitrary
//! read-only SQL, FTS convenience search, schema introspection, card lookup.

use anyhow::{bail, Result};
use rusqlite::types::ValueRef;
use rusqlite::{params, Connection, OpenFlags, ToSql};
use serde_json::{Map, Value};
use std::path::Path;

use crate::{Format, OutputArgs};

const SEARCH_COLS: &[&str] = &["display_name", "mana_cost", "type", "edhrec_rank", "text"];

const CHEATSHEET: &str = r#"
==============================  mtgbrain query guide  ==========================

GRANULARITY
  One row per card FACE. Multi-face cards (transform/split/MDFC/adventure) share
  a `name` like "Fire // Ice" and get one row per face (face_name, side). Use
  `display_name` in output. To collapse faces to one row: GROUP BY name.

THE THREE WAYS TO FILTER (use whichever fits)
  1. Full-text (ranked, fast) -- best for "cards that do/say X":
       SELECT c.display_name, c.text FROM cards c JOIN cards_fts f ON f.rowid=c.id
       WHERE cards_fts MATCH 'counter NOT proliferate' ORDER BY rank;
     FTS syntax: tokenA tokenB (=AND) | 'tokenA OR tokenB' | '"exact phrase"'
       | prefix* | col:term  (cols: display_name name type text keywords).
     NOTE: the FTS tokenizer drops punctuation, so '+1/+1' is NOT FTS-searchable.
       For symbol/exact phrasing use LIKE on the raw `text` column (see #2).

  2. Substring on raw columns -- best for punctuation / exact phrasing:
       ... WHERE text LIKE '%+1/+1 counter%'
       ... WHERE keywords LIKE '%Flying%'

  3. Normalized membership tables -- best for exact set logic / counts:
       card_colors(card_id,color)  card_color_identity(card_id,color)
       card_types(card_id,type)    card_subtypes(card_id,subtype)
       card_supertypes(...)        card_keywords(card_id,keyword)
       card_produced_mana(card_id,mana)  card_legalities(card_id,format,status)
     e.g. mono-green Elves that can be a commander:
       SELECT c.display_name FROM cards c
       JOIN card_subtypes s ON s.card_id=c.id AND s.subtype='Elf'
       WHERE c.color_identity='G' AND c.can_be_commander=1;

KEY `cards` COLUMNS
  display_name name face_name side layout
  mana_cost mana_value(=cmc) face_mana_value
  colors color_count color_identity color_identity_count is_colorless is_multicolor
  type supertypes types subtypes
  power power_num  toughness toughness_num  loyalty loyalty_num  defense defense_num
    (*_num are NULL when the stat is '*'/'X'; use them for ranges like power_num>=7)
  text keywords produced_mana has_text
  num_printings printings  edhrec_rank  edhrec_saltiness
  cube_elo cube_count cube_pick_count cube_popularity   (CubeCobra; NULL if unrated)
  is_reserved is_funny is_game_changer
  can_be_commander can_be_brawl_commander can_be_oathbreaker
  hand life  scryfall_oracle_id  legalities(JSON)

"...AND DOESN'T SUCK"  (two independent quality signals)
  edhrec_rank = Commander popularity: LOWER = more played. Staples < 1000, good < ~5000.
    Filter `edhrec_rank IS NOT NULL AND edhrec_rank < 5000` and/or ORDER BY it.
  cube_elo = CubeCobra cube-draft power rating: HIGHER = better. ~1200 = default/unplayed,
    > 1500 = strong, > 1700 = bomb. cube_count = # of cubes including it (raw play signal).
    For cube/limited "doesn't suck", use `cube_elo IS NOT NULL ORDER BY cube_elo DESC`.
  The two disagree usefully: a coin-flip card can be cube-playable but EDHREC-obscure (or
  vice versa). Joke cards have is_funny=1; exclude with is_funny=0.

RULINGS (official judge clarifications): rulings(card_name,date,text),
  and rulings_fts MATCH '...'.

SEED EXAMPLES (CLI)
  mtgbrain sql "SELECT display_name,mana_cost,edhrec_rank,text FROM cards
    WHERE text LIKE '%flip a coin%' AND is_funny=0 AND edhrec_rank<6000
    ORDER BY edhrec_rank"
  mtgbrain search "counter remove OR move" --where "edhrec_rank IS NOT NULL" --order edhrec_rank
  mtgbrain sql "SELECT display_name,text FROM cards WHERE text LIKE '%charge counter%'"
===============================================================================
"#;

fn open_ro(db: &Path) -> Result<Connection> {
    if !db.exists() {
        bail!(
            "{} not found. Run `mtgbrain download` then `mtgbrain build`.",
            db.display()
        );
    }
    Ok(Connection::open_with_flags(
        db,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

/// Strip `-- line` and `/* block */` comments, respecting single-quoted string
/// literals (including `''` escapes), so a `;` inside a comment or string is not
/// mistaken for a statement separator.
fn strip_sql_comments(sql: &str) -> String {
    let b = sql.as_bytes();
    let mut out = String::with_capacity(sql.len());
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(c);
            if c == '\'' {
                if i + 1 < b.len() && b[i + 1] == b'\'' {
                    out.push('\'');
                    i += 2;
                    continue;
                }
                in_str = false;
            }
            i += 1;
        } else if c == '\'' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if c == '-' && i + 1 < b.len() && b[i + 1] == b'-' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if c == '/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            out.push(' ');
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn is_read_only(sql: &str) -> bool {
    let stripped = strip_sql_comments(sql);
    let s = stripped.trim().trim_end_matches(';').trim();
    if s.contains(';') {
        return false; // no multi-statement
    }
    let head = s.trim_start_matches('(').trim_start();
    let word = head.split_whitespace().next().unwrap_or("").to_lowercase();
    word == "select" || word == "with"
}

/// Run a SQL query and collect up to `limit` rows (0 = unlimited). Returns the
/// column names, the rows as JSON values, and whether more rows were available.
fn fetch(
    conn: &Connection,
    sql: &str,
    bind: &[&dyn ToSql],
    limit: usize,
) -> Result<(Vec<String>, Vec<Vec<Value>>, bool)> {
    let mut stmt = conn.prepare(sql)?;
    let ncol = stmt.column_count();
    let cols: Vec<String> = (0..ncol)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let mut rows = stmt.query(bind)?;
    let mut out: Vec<Vec<Value>> = Vec::new();
    let mut more = false;
    while let Some(row) = rows.next()? {
        if limit > 0 && out.len() == limit {
            more = true;
            break;
        }
        let mut vals = Vec::with_capacity(ncol);
        for i in 0..ncol {
            vals.push(value_ref_to_json(row.get_ref(i)?));
        }
        out.push(vals);
    }
    Ok((cols, out, more))
}

fn value_ref_to_json(v: ValueRef) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => Value::from(f),
        ValueRef::Text(t) => Value::from(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(_) => Value::from("<blob>"),
    }
}

/// Project rows down to the named columns (reordering / dropping as needed).
fn project(
    cols: &[String],
    rows: &[Vec<Value>],
    want: &[String],
) -> (Vec<String>, Vec<Vec<Value>>) {
    let idx: Vec<Option<usize>> = want
        .iter()
        .map(|w| cols.iter().position(|c| c == w))
        .collect();
    let new_rows = rows
        .iter()
        .map(|r| {
            idx.iter()
                .map(|oi| oi.map_or(Value::Null, |i| r[i].clone()))
                .collect()
        })
        .collect();
    (want.to_vec(), new_rows)
}

fn cell(v: &Value, width: usize, full: bool) -> String {
    let s = match v {
        Value::Null => String::new(),
        Value::String(s) => s.replace('\n', " / "),
        Value::Number(n) => match n.as_f64() {
            // whole-valued reals as ints (mana_value 1.0 -> "1"); else 2 decimals
            Some(f) if n.is_f64() && f.fract() == 0.0 => format!("{}", f as i64),
            Some(f) if n.is_f64() => format!("{f:.2}"),
            _ => n.to_string(),
        },
        other => other.to_string(),
    };
    if !full && width > 0 && s.chars().count() > width {
        let mut t: String = s.chars().take(width.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        s
    }
}

fn render(cols: &[String], rows: &[Vec<Value>], out: &OutputArgs) -> String {
    match out.fmt() {
        Format::Json => {
            let objs: Vec<Value> = rows
                .iter()
                .map(|r| {
                    let mut m = Map::new();
                    for (c, v) in cols.iter().zip(r) {
                        m.insert(c.clone(), v.clone());
                    }
                    Value::Object(m)
                })
                .collect();
            serde_json::to_string_pretty(&objs).unwrap_or_default()
        }
        Format::Csv => {
            let mut lines = vec![cols.join(",")];
            for r in rows {
                let row: Vec<String> = r
                    .iter()
                    .map(|v| {
                        let s = cell(v, 0, true).replace('\n', " ");
                        if s.contains(',') || s.contains('"') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s
                        }
                    })
                    .collect();
                lines.push(row.join(","));
            }
            lines.join("\n")
        }
        Format::Md => {
            let mut lines = vec![
                format!("| {} |", cols.join(" | ")),
                format!("| {} |", cols.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")),
            ];
            for r in rows {
                let row: Vec<String> = r
                    .iter()
                    .map(|v| cell(v, out.width, out.full).replace('|', "\\|"))
                    .collect();
                lines.push(format!("| {} |", row.join(" | ")));
            }
            lines.join("\n")
        }
        Format::Table => {
            let mut w: Vec<usize> = cols.iter().map(|c| c.chars().count()).collect();
            let body: Vec<Vec<String>> = rows
                .iter()
                .map(|r| {
                    r.iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let s = cell(v, out.width, out.full);
                            w[i] = w[i].max(s.chars().count());
                            s
                        })
                        .collect()
                })
                .collect();
            let pad = |s: &str, n: usize| {
                let len = s.chars().count();
                format!("{s}{}", " ".repeat(n.saturating_sub(len)))
            };
            let mut lines = vec![
                cols.iter()
                    .enumerate()
                    .map(|(i, c)| pad(c, w[i]))
                    .collect::<Vec<_>>()
                    .join("  "),
                w.iter().map(|n| "-".repeat(*n)).collect::<Vec<_>>().join("  "),
            ];
            for r in &body {
                lines.push(
                    r.iter()
                        .enumerate()
                        .map(|(i, s)| pad(s, w[i]))
                        .collect::<Vec<_>>()
                        .join("  "),
                );
            }
            lines.join("\n")
        }
    }
}

fn emit(cols: Vec<String>, rows: Vec<Vec<Value>>, more: bool, out: &OutputArgs) {
    let (cols, rows) = match &out.cols {
        Some(c) => {
            let want: Vec<String> = c.split(',').map(|s| s.trim().to_string()).collect();
            project(&cols, &rows, &want)
        }
        None => (cols, rows),
    };
    if rows.is_empty() {
        println!("(no rows)");
        return;
    }
    println!("{}", render(&cols, &rows, out));
    if more {
        println!(
            "\n... showing first {} rows; tighten the WHERE or raise --limit for more.",
            out.limit
        );
    }
}

pub fn run_sql(db: &Path, sql: &str, out: &OutputArgs) -> Result<()> {
    if !is_read_only(sql) {
        bail!("only a single read-only SELECT/WITH statement is allowed");
    }
    let conn = open_ro(db)?;
    let (cols, rows, more) = fetch(&conn, sql, &[], out.limit)?;
    emit(cols, rows, more, out);
    Ok(())
}

pub fn search(
    db: &Path,
    query: &str,
    where_clause: Option<&str>,
    order: &str,
    phrase: bool,
    out: &OutputArgs,
) -> Result<()> {
    let conn = open_ro(db)?;
    let cols: Vec<String> = match &out.cols {
        Some(c) => c.split(',').map(|s| s.trim().to_string()).collect(),
        None => SEARCH_COLS.iter().map(|s| (*s).to_string()).collect(),
    };
    let select = cols
        .iter()
        .map(|c| format!("c.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut sql = format!(
        "SELECT {select} FROM cards c JOIN cards_fts f ON f.rowid = c.id WHERE cards_fts MATCH ?1"
    );
    if let Some(w) = where_clause {
        sql.push_str(&format!(" AND ({w})"));
    }
    sql.push_str(&format!(" ORDER BY {order}"));
    if out.limit > 0 {
        sql.push_str(&format!(" LIMIT {}", out.limit)); // limit 0 = unlimited (no clause)
    }

    let q = if phrase {
        format!("\"{query}\"")
    } else {
        query.to_string()
    };
    let (cols, rows, _) = fetch(&conn, &sql, &[&q], 0)?;
    // Already capped by LIMIT in SQL; don't double-apply --cols here.
    let mut out2 = out.clone();
    out2.cols = None;
    emit(cols, rows, false, &out2);
    Ok(())
}

pub fn card(db: &Path, name: &str, out: &OutputArgs) -> Result<()> {
    let conn = open_ro(db)?;
    if matches!(out.fmt(), Format::Json) {
        let (cols, rows, _) = fetch(
            &conn,
            "SELECT * FROM cards WHERE name = ?1 OR display_name = ?1 OR name LIKE ?2 \
             ORDER BY name, face_index LIMIT 16",
            &[&name, &format!("%{name}%")],
            0,
        )?;
        emit(cols, rows, false, out);
        return Ok(());
    }

    let mut stmt = conn.prepare(
        "SELECT display_name, mana_cost, mana_value, type, power, toughness, loyalty, defense, \
         text, edhrec_rank, keywords, produced_mana, name \
         FROM cards WHERE name = ?1 OR display_name = ?1 OR name LIKE ?2 \
         ORDER BY name, face_index LIMIT 12",
    )?;
    let like = format!("%{name}%");
    let mut rows = stmt.query(params![name, like])?;
    let mut found_name: Option<String> = None;
    let mut any = false;
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    while let Some(r) = rows.next()? {
        let dn: String = r.get(0)?;
        found_name.get_or_insert(r.get::<_, String>(12)?);
        if !seen.insert(dn.clone()) {
            continue; // skip duplicate atomic entries for the same face
        }
        any = true;
        let cost: Option<String> = r.get(1)?;
        let mv: Option<f64> = r.get(2)?;
        let typ: Option<String> = r.get(3)?;
        let power: Option<String> = r.get(4)?;
        let tough: Option<String> = r.get(5)?;
        let loyalty: Option<String> = r.get(6)?;
        let defense: Option<String> = r.get(7)?;
        let text: Option<String> = r.get(8)?;
        let rank: Option<i64> = r.get(9)?;
        let kw: Option<String> = r.get(10)?;
        let makes: Option<String> = r.get(11)?;

        let cost_s = cost.map(|c| format!("  {c}")).unwrap_or_default();
        let mv_s = mv.map_or(String::new(), |v| {
            if v.fract() == 0.0 {
                format!("  (MV {})", v as i64)
            } else {
                format!("  (MV {v})")
            }
        });
        println!("\n{dn}{cost_s}{mv_s}");
        if let Some(t) = typ {
            println!("  {t}");
        }
        let mut stat = Vec::new();
        if let (Some(p), Some(t)) = (&power, &tough) {
            stat.push(format!("{p}/{t}"));
        }
        if let Some(l) = &loyalty {
            stat.push(format!("Loyalty {l}"));
        }
        if let Some(d) = &defense {
            stat.push(format!("Defense {d}"));
        }
        if !stat.is_empty() {
            println!("  {}", stat.join("  "));
        }
        if let Some(t) = text {
            for line in t.split('\n') {
                println!("    {line}");
            }
        }
        let mut extra = Vec::new();
        if let Some(r) = rank {
            extra.push(format!("EDHREC #{r}"));
        }
        if let Some(k) = kw {
            extra.push(format!("keywords: {k}"));
        }
        if let Some(m) = makes {
            extra.push(format!("makes: {m}"));
        }
        if !extra.is_empty() {
            println!("  [{}]", extra.join(" | "));
        }
    }
    if !any {
        println!("(no card found)");
        return Ok(());
    }

    if let Some(n) = found_name {
        if let Ok(mut rstmt) =
            conn.prepare("SELECT date, text FROM rulings WHERE card_name = ?1 ORDER BY date LIMIT 12")
        {
            let mut rr = rstmt.query(params![n])?;
            let mut printed = false;
            while let Some(r) = rr.next()? {
                if !printed {
                    println!("\n  Rulings:");
                    printed = true;
                }
                let date: Option<String> = r.get(0)?;
                let text: Option<String> = r.get(1)?;
                println!(
                    "    [{}] {}",
                    date.unwrap_or_default(),
                    text.unwrap_or_default()
                );
            }
        }
    }
    Ok(())
}

pub fn schema(db: &Path) -> Result<()> {
    let conn = open_ro(db)?;
    let mut info = conn.prepare("SELECT key, value FROM meta")?;
    let pairs: Vec<String> = info
        .query_map([], |r| Ok(format!("{}={}", r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .filter_map(Result::ok)
        .collect();
    println!("source: {}", pairs.join(", "));
    println!("{CHEATSHEET}");

    println!("TABLES & COLUMNS");
    let mut tstmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' \
         AND name NOT LIKE '%fts%' ORDER BY name",
    )?;
    let tables: Vec<String> = tstmt
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();
    for t in tables {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))?;
        let mut cstmt = conn.prepare(&format!("PRAGMA table_info('{t}')"))?;
        let cols: Vec<String> = cstmt
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(Result::ok)
            .collect();
        println!("\n  {t}  ({count} rows)");
        println!("    {}", cols.join(", "));
    }
    println!(
        "\n  FTS: cards_fts(display_name,name,type,text,keywords), rulings_fts(card_name,text)"
    );
    Ok(())
}
