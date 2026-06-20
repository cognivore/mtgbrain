# mtgbrain

Download the entire Magic: The Gathering card corpus from [MTGJSON](https://mtgjson.com)
into a **queryable SQLite database** designed for an LLM agent to drive with diverse SQL —
so a human can ask in natural language for obscure cards ("ones that move counters", "coin
flips that don't suck", "things that add charge counters") and the agent can translate that
into a precise query.

Rust binary, all dependencies provisioned by a Nix flake, `direnv`-friendly.

```
you (natural language)  ->  LLM agent  ->  mtgbrain sql/search  ->  SQLite (cards + FTS5)
```

## Quick start

```sh
direnv allow            # or: nix develop     (provisions cargo, rustc, curl, sqlite, just)
just setup              # download MTGJSON + CubeCobra Elo + build ./data/mtg.sqlite
                        # (the JSON build is seconds; the CubeCobra fetch adds a few minutes —
                        #  skip it with `mtgbrain setup --no-cube`)
just schema             # print the schema + query cheatsheet (read this first)

# then query:
just search "flip a coin"
just sql "SELECT display_name, text FROM cards WHERE text LIKE '%move a counter%' LIMIT 10"
```

Without `just`, call the binary directly (`cargo run --release --` in the dev shell, or the
built `./result/bin/mtgbrain` after `nix build`):

```sh
mtgbrain setup
mtgbrain schema
mtgbrain search "counter remove OR move" --where "edhrec_rank IS NOT NULL" --order edhrec_rank
```

## The data

[MTGJSON `AtomicCards`](https://mtgjson.com/data-models/card/card-atomic/) — one logical card
per name, deduplicated across printings (the right granularity for "find me cards that do X").
At the time of writing that's **~34,600 distinct cards / ~35,500 faces**, with EDHREC
popularity ranks, keywords, produced mana, format legalities, and ~77,000 official rulings.
The build is fully reproducible: re-run `mtgbrain setup` to refresh to the latest MTGJSON.

## The commands

| command | what it does |
| --- | --- |
| `mtgbrain download` | fetch `AtomicCards.json` into `./data` (delegates the download to `curl`) |
| `mtgbrain cubecobra` | fetch CubeCobra card Elo ratings into `./data/cubecobra_elo.jsonl` |
| `mtgbrain build` | parse the JSON into `./data/mtg.sqlite` (FTS5 indexes, normalized tables, cube Elo join) |
| `mtgbrain setup` | `download` + `cubecobra` + `build` in one step (`--no-cube` to skip Elo) |
| `mtgbrain schema` | print the schema and a query cheatsheet — **the agent's starting point** |
| `mtgbrain sql "<SELECT…>"` | run a single read-only `SELECT`/`WITH` and print the result |
| `mtgbrain search "<fts>"` | ranked full-text search over name/type/oracle text |
| `mtgbrain card "<name>"` | show every face + rulings for one card |
| `mtgbrain edit seed` | build a cube-editor DB from a card-name list, snapshotting all fields |
| `mtgbrain edit serve` | serve a local card-by-card review/errata UI |

Output: `-f table` (default), `--json`, `--md`, `--csv`. Plus `--limit N`, `--full` (don't
truncate text), `--width N`, `--cols a,b,c`. The DB is opened **read-only**, and `sql`
accepts only a single `SELECT`/`WITH`, so the agent cannot mutate or drop anything.

## Schema, at a glance

**One row per card _face_** in `cards`. Multi-face cards (transform / split / MDFC /
adventure) share a `name` like `"Fire // Ice"` and get one row per face (`face_name`, `side`,
`face_index`). Use `display_name` in output; `GROUP BY name` to collapse faces.

Selected `cards` columns (full list via `mtgbrain schema`):

- identity: `display_name name face_name layout`
- cost/color: `mana_cost mana_value(=cmc) colors color_count color_identity is_colorless is_multicolor`
- type: `type supertypes types subtypes`
- stats: `power power_num  toughness toughness_num  loyalty loyalty_num  defense defense_num`
  — the `*_num` columns are `NULL` when the printed value is `*`/`X`, so range filters like
  `power_num >= 7` just work.
- text: `text keywords produced_mana has_text`
- quality: `edhrec_rank edhrec_saltiness` (Commander popularity) and `cube_elo cube_count
  cube_pick_count cube_popularity` (CubeCobra cube-draft signal; `NULL` if unrated)
- meta: `num_printings printings  is_reserved is_funny is_game_changer  can_be_commander
  can_be_brawl_commander can_be_oathbreaker  legalities`

Plus FTS (`cards_fts`, `rulings_fts`) and normalized membership tables for exact set logic:
`card_colors, card_color_identity, card_types, card_subtypes, card_supertypes, card_keywords,
card_produced_mana, card_legalities` and `rulings(card_name, date, text)`.

## Three ways to query (pick what fits)

**1. Full-text search** — ranked, fast; best for "cards that _do_ X":

```sql
SELECT c.display_name, c.text
FROM cards c JOIN cards_fts f ON f.rowid = c.id
WHERE cards_fts MATCH 'counter NOT proliferate'
ORDER BY rank;
```

FTS syntax: `a b` (AND), `'a OR b'`, `'"exact phrase"'`, `prefix*`, `col:term` (columns:
`display_name name type text keywords`). **The FTS tokenizer drops punctuation, so `+1/+1` is
not FTS-searchable** — use `LIKE` for symbols.

**2. `LIKE` on raw columns** — best for punctuation / exact phrasing:

```sql
SELECT display_name, text FROM cards WHERE text LIKE '%+1/+1 counter%';
```

**3. Normalized membership tables** — best for exact set logic and joins:

```sql
-- mono-green Elves that can be a commander
SELECT c.display_name, c.mana_cost
FROM cards c JOIN card_subtypes s ON s.card_id = c.id AND s.subtype = 'Elf'
WHERE c.color_identity = 'G' AND c.can_be_commander = 1
ORDER BY c.edhrec_rank;
```

### "…and doesn't suck"

There are **two independent quality signals**, and they disagree in useful ways:

- `edhrec_rank` — Commander popularity, **lower = more played** (staples `< 1000`, good
  `< ~5000`). `NULL` for unranked cards.
- `cube_elo` — CubeCobra's cube-draft power rating, **higher = better** (`~1200` = default /
  unplayed, `> 1500` strong, `> 1700` a bomb). `cube_count` is how many of ~25k cubes run the
  card — a blunt but robust "is it actually played" signal. `NULL` if the card is unrated.

A coin-flip or counter card can be a cube all-star while EDHREC-obscure, or vice versa — so
pick the axis that matches the format you care about. Gotchas worth burning in:

- `edhrec_rank` and `cube_elo` can be `NULL`, and **`NULL` sorts first** under `ORDER BY
  edhrec_rank` (and you usually want it last for `cube_elo DESC` too). Add `WHERE … IS NOT
  NULL`, or sort with `ORDER BY (edhrec_rank IS NULL), edhrec_rank`.
- Joke/Un-set cards have `is_funny = 1`; exclude them with `is_funny = 0`.

## The motivating examples

```sh
# coin flips that don't suck
mtgbrain sql "SELECT display_name, mana_cost, edhrec_rank, text FROM cards
  WHERE text LIKE '%flip a coin%' AND is_funny = 0
    AND edhrec_rank IS NOT NULL AND edhrec_rank < 6000
  ORDER BY edhrec_rank"
# -> Mana Crypt, Krark the Thumbless, Mirror March, Tavern Scoundrel, ...

# cards that remove or move counters (ranked by how played they are)
mtgbrain search "counter move OR remove OR proliferate" \
  --where "is_funny = 0 AND edhrec_rank IS NOT NULL" --order edhrec_rank
# -> Karn's Bastion, Forgotten Ancient, Evolution Sage, Walking Ballista, Nesting Grounds, ...

# cards that add a specific counter type (e.g. charge counters)
mtgbrain sql "SELECT display_name, text FROM cards
  WHERE text LIKE '%charge counter%' AND text LIKE '%put%counter%'
  ORDER BY edhrec_rank"

# same idea, ranked for CUBE instead of Commander: coin flips that are cube-playable
mtgbrain sql "SELECT display_name, cube_elo, cube_count, edhrec_rank FROM cards
  WHERE text LIKE '%flip a coin%' AND is_funny = 0 AND cube_elo IS NOT NULL
  ORDER BY cube_elo DESC LIMIT 10"
```

More worked, **verified** examples (each query was run against the real DB and its results
checked) live in [`COOKBOOK.md`](./COOKBOOK.md).

## For the LLM agent driving this

1. Run `mtgbrain schema` once to load the table list + cheatsheet.
2. Prefer `search` for fuzzy "does/says X" questions; reach for `sql` + `LIKE` for exact
   phrasing or punctuation; use the normalized tables for exact set membership.
3. Use `--json` when you want to parse results programmatically.
4. When a human wants "good" cards, filter `edhrec_rank IS NOT NULL` and order by it.

## Project layout

```
flake.nix          devShell (cargo/rustc/curl/sqlite/just) + package output
nix/package.nix    reproducible build; wraps curl onto the binary's PATH
.envrc             `use flake` (direnv)
Justfile           run-at-will recipes (setup, sql, search, card, nix-build, ...)
Cargo.toml         single Rust crate, stdlib-light deps
src/
  main.rs          clap CLI
  model.rs         serde model for MTGJSON AtomicCards
  download.rs      curl + flate2 fetch/inflate
  build.rs         JSON -> SQLite schema + FTS5
  query.rs         schema / sql / search / card + table/json/md/csv rendering
data/              AtomicCards.json, cubecobra_elo.jsonl, mtg.sqlite (gitignored)
COOKBOOK.md        20 verified natural-language -> SQL recipes for obscure archetypes
```

## Cube editor (oddysey2026)

A tiny **local web app** to walk a cube list one card at a time, accept each card or attach an
**errata**, and toggle a per-card **"GenAI new art in the style of Odyssey-block artists?"** flag.
It needs `data/mtg.sqlite` built first (`just setup`).

```sh
just edit-seed          # build data/cube_editor.sqlite from projects/odyssey2026/cube360/cube_list.txt
just edit               # serve the UI at http://127.0.0.1:49737  (override: just edit 50001)

# or directly:
mtgbrain edit seed --list projects/odyssey2026/cube360/cube_list.txt --force
mtgbrain edit serve --port 49737
```

The seeder snapshots every field from `mtg.sqlite` into a separate editor DB (your decisions never
touch the source data) and applies two **pre-sets** (a starting point — review each card by hand):

- **Pre-approved** (`decision = accepted`): Odyssey-block creatures only — a Creature printed in
  ODY / TOR / JUD. Nothing else.
- **GenAI-art flag pre-set on**: every *modern-frame* card — one with no printing in any
  pre-8th-Edition (pre-2003 "old frame") set. Old-frame cards keep their original art. Nothing else.

The UI: filter (all / pending / accepted / errata / genai) + name search in the sidebar; the detail
pane shows the full card, a live **card-image preview** (fetched from the [Scryfall](https://scryfall.com/docs/api)
open API by name), and an errata box. Keys: `a` accept · `e` errata · `g` toggle GenAI art ·
`j`/`k` next/prev · `n` next pending. Everything persists immediately to `data/cube_editor.sqlite`
(table `cube_cards` — query/export it with `sqlite3` like any other DB). Port is deliberately high
(`49737`) to avoid clashes; change with `--port`.

## Rebuilding / updating

`mtgbrain setup --force` re-downloads the latest MTGJSON and rebuilds the DB. The build drops
and recreates `mtg.sqlite` from scratch each time, so it always matches the current JSON.

Re-running `mtgbrain edit seed --force` rebuilds the editor DB from the list and **discards saved
decisions**; omit `--force` to protect an in-progress review.

## License

MIT.
