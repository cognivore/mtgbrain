# pointless-b-side — special layouts to render (split / adventure / prepare)

These are SINGLE physical faces (the MPC back is the odyssey2026 card) but have a
two-part printed layout. Front-face data is already seeded (lookup_card front-face
fallback), so they render the **front face cleanly today**; the work below adds the
second part. All second-part data is in `mtg.sqlite` (`face_index = 1`) and in the cached
Scryfall `__latest.json` (`card_faces[]`).

## Cards affected
| Card | Layout | Front (renders now) | Second part (TODO) |
|---|---|---|---|
| Bind // Liberate | split | Bind — {1}{G} Instant | Liberate — {1}{W} Instant |
| Spiritcall Enthusiast // Scrollboost | prepare | Cat Cleric 3/3 | Scrollboost — {1}{W} Sorcery |
| Harmonized Trio // Brainstorm | prepare | Merfolk Bard 1/1 | Brainstorm — Sorcery |
| Blazing Firesinger // Seething Song | prepare | Dwarf Bard 2/3 | Seething Song — Sorcery |
| Inspired Skypainter // Maestro's Gift | prepare | Lizard Wizard 2/2 | Maestro's Gift |
| Lorehold Archivist // Restore Relic | prepare | Dwarf Artificer 3/2 | Restore Relic |
| Value Town // Take a Trip… | adventure | Land — Town | Take a Trip — sorcery (adventure) |

## Build plan (per layout)
**Detection + data:** in `render_card_8th`, look the card up in the source DB by `name`/face;
if `layout in (adventure, split, flip, prepare-equivalent)` or there are two faces, load
**both** faces' (name, mana_cost, type, oracle_text) from `mtg.sqlite` (or `card_faces[]` in
`render-cache/art/<name>__latest.json`). Pass a `Vec<Face>` into the builder.

**Adventure / Prepare** (cardconjurer `packAdventure.js`): main creature frame unchanged; overlay
an **adventure sub-frame in the LEFT HALF of the rules box** — its own pinline + name + cost +
type bar + rules. cardconjurer 8th-style adventure assets: `img/frames/8th/…` + the adventure
pinline. Sub-box geometry from packAdventure (left ~0.10, top ~0.628, width ~0.40, the creature
rules reflow into the right half). Prepare ≈ adventure (sub-box = the "prepared" spell); no
cardconjurer pack yet, so adapt the adventure sub-box.

**Split** (cardconjurer `packAftermath.js` / split): render PORTRAIT but rotate the content 90°,
two stacked half-cards (each: mini title bar + cost + type + rules), read by turning the card.
Bind half on one side, Liberate on the other. New template `card_template_8th_split.html`.

## Status
- Front faces render correctly **now** (functional proxies).
- Full two-part layouts = the next focused pass; each is its own template + geometry + visual
  iteration loop (render → crop → compare → tune), same workflow that locked the base 8ED frame.
