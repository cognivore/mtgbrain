# oddysey2026 — Worksheet U (Mono-Blue)

Target ~48. Built per HARD RULES: creatures **only** from `pool/odysseyblock_creatures.tsv`
(Odyssey-block ODY/TOR/JUD), non-creatures from `pool/master_candidates.tsv` `cube_color='U'`.

## What this worksheet does

Mono-blue is the **engine room** of the cube's slow, honest combo decks. It does four
interlocking things, all low-power and decision-dense:

1. **High Tide prison/ritual** — `High Tide` doubles Island mana; free/refunding spells
   (`Frantic Search`, `Snap`, `Turnabout`, `Reset`, `Capsize`) chain into a single big
   payoff (`Stroke of Genius`) instead of a board-camping value engine. Cast and spent.
2. **Lands-become-Islands** — `Spreading Seas`, `Sea's Claim`, `Convincing Mirage`,
   `Aquitect's Will` turn the table's lands into Islands: own-ramp for High Tide **and**
   colour-screw denial. This is the namesake "turn lands into Islands" archetype (spec #3).
3. **Self-mill → deck-out win** — Cephalid loot/mill bodies (`Cephalid Vandal` shred,
   `Cephalid Looter`, `Cephalid Broker`, `Cephalid Sage` threshold, `Cephalid Scout`)
   fill the yard and thin the library; `Mana Severance` strips lands; the win is the next
   draw from an empty library via **`Laboratory Maniac`** (EXCEPTION creature, deck-out —
   *never* Thassa's Oracle) backed up by **`Jace, Wielder of Mysteries`**.
4. **Mill-storm / mill-out the opponent** — `Brain Freeze` (storm), `Sphinx's Tutelage`
   convert your card-draw into an opposing deck-out; `Scalpelexis` and `Ambassador Laquatus`
   add evasive/repeatable mill.

## Guild archetypes it feeds

- **UB Dimir (Ad Nauseam / control-combo)** — Lab Maniac + Jace deck-out line, `Standstill`
  soft-lock, `Day's Undoing` honest wheel, cantrips/tutors to assemble. Hand-size punishers
  (Black Vise / Iron Maiden, colourshifted-B in other sheets) want `Prosperity`-style draws.
- **UG Simic (proliferate / fading)** — `Steady Progress`, `Tezzeret's Gambit` accelerate
  charge/age/fading counters; cross-refs `Tangle Wire` (locked U/G) and `Parallax Tide`.
- **UR Izzet (spells / storm-lite)** — free spells + buyback cantrips (`Mystic Speculation`,
  `Capsize`) pad storm for `Brain Freeze`; `Mana Short` clears the finish turn.
- **UW Azorius (Replenish/enchantress)** — light support only here; the enchantment-recursion
  package (Attunement, Seal, Hatching Plans, Opalescence) lives on the W/gold sheets.

## Key cards & combos

- **High Tide line:** `High Tide` → `Frantic Search`/`Snap`/`Reset`/`Turnabout` net mana →
  `Stroke of Genius` for a huge draw or to deck the opponent. `Capsize` w/ buyback is the
  grindy soft-lock back-up (bounce a land each turn).
- **Deck-out win:** thin with self-mill + `Mana Severance`, draw into empty library →
  `Laboratory Maniac` / `Jace, Wielder of Mysteries` static converts the loss into a win.
- **Mill-out win:** `Sphinx's Tutelage` + any big draw (`Stroke of Genius`) or `Brain Freeze`
  storm decks the opponent — honest, draw-fuelled, no infinite engine.
- **Alt-win build-around:** `Triskaidekaphile` (exactly 13 cards in hand) — has a built-in
  `{3}{U}: Draw` mana sink, so it is **not** a dead card.

## Errata / flags

- **No literal-blank cards in the cube list** → no `dead`+`errata` flags this sheet.
  `Triskaidekaphile` self-sustains via its draw ability; `Mana Severance` is a real effect.
- **One EXCEPTION creature:** `Laboratory Maniac` (`source=exception`, structural reason:
  deck-out win condition; the only sanctioned blue creature outside the pool).
- **No colourshifts originate here** (the locked blue/gold shifts — Black Vise, Iron Maiden,
  The Rack, Phyrexian Tyranny, etc. — are resolved on the B/gold worksheets).

## Notable cuts (→ maybeboard, nothing discarded)

- **`Cephalid Illusionist`** — named in the brief but **not in the odysseyblock pool**
  (Judgment, present only in master_candidates); pool-locked out as a creature.
- **`too_strong` (board-camping value engines):** `Mind Over Matter`, `Time Spiral`,
  `Palinchron` — infinite/format-warping; sent to maybeboard.
- **One-X-draw rule:** kept `Stroke of Genius`; `Braingeyser` and `Blue Sun's Zenith` cut.
- **Free-spell degeneracy:** `Gitaxian Probe` cut (highest elo, pure 0-mana cantrip) to hold
  the low-power line; on-theme free spells that untap lands (`Snap`, `Frantic Search`) kept.
- **Out-of-pool free-mana / morph bodies** (`Peregrine Drake`, `Great Whale`, `Cloud of
  Faeries`, the Riptide/Mistform wizards, `Harbinger of the Seas`, `Wonder`) — strong Islands
  / ramp creatures but outside the curated pool; maybeboarded.
- **Off-colour wheels** (`Windfall`, `Tolarian Winds`) live on UB/UR sheets, not mono-U.
- **`Battle of Wits`** — honest alt-win but needs a 200-card deck; undraftable in a singleton
  cube, so maybeboarded rather than errata'd.

## Counts

20 odysseyblock mono-U creatures + 1 exception (Lab Maniac) = 21 creature cards; 28
non-creatures (incl. Jace PW). Total **49**.
