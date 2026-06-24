# oddysey2026 — Creature-Count Analysis vs. Established Cubes

How creature-dense is this cube, and how does that compare to conventional cube construction?
Numbers are computed from the live editor DB (`data/cube_editor.sqlite`, active cards only,
`removed=0`), measured 2026-06-24. Type overrides were checked — none flip a card's
creature-ness — so the counts below are exact.

## This cube

| Bucket | Count | % of total (473) | % of nonland (424) |
|---|---:|---:|---:|
| **Creatures** | **173** | **36.6 %** | **40.8 %** |
| Noncreature spells | 251 | 53.1 % | 59.2 % |
| Lands | 49 | 10.4 % | — |
| **Total** | 473 | 100 % | — |

"Creatures" counts every card with `Creature` in its (effective) type line, including the few
Enchantment Creatures / Kindred bodies; all 173 are nonland.

### Creatures by colour

| W | U | B | R | G | Multicolor | Colorless |
|---:|---:|---:|---:|---:|---:|---:|
| 30 | 30 | **36** | 28 | 31 | 15 | 3 |

Black is the largest creature colour — the intended **Torment** homage (Torment was the
black-heavy middle set of the block), not an accident of curation.

## How we fare vs. established cubes

The standard cube-design rule of thumb (CubeCobra / Lucky Paper primers, most 360/450/540
singleton lists) is that **creatures make up roughly half of the nonland slots — typically
45–55 %**, with lands around **10–16 %** of the whole cube. Approximate reference points:

| Cube | Creatures (% of nonland) | Notes |
|---|---:|---|
| Typical "balanced" 360 singleton | ~50 % | the usual target |
| MTGO Vintage Cube (~540) | ~45–48 % | spell-rich but still creature-anchored |
| Aggro-leaning / peasant cubes | ~52–58 % | creatures carry the gameplan |
| **oddysey2026** | **40.8 %** | **deliberately creature-light** |

*(Established-cube figures are typical community ranges, not exact published stats — treat as a
calibration band, not a hard line.)*

**Verdict: we are creature-light by ~5–14 points, on purpose.** At 40.8 % of nonland (36.6 % of
the whole list) the cube sits *below* the conventional 45–55 % band. That is a direct, expected
consequence of the cube's hard rules:

- **Creatures come almost exclusively from the curated Odyssey-block pool** (ASSUMPTIONS.md);
  post-2004 creatures are added only rarely and with cause.
- The build-around identity is **spells, enchantments, artifacts, and lock pieces** — Enchanted
  Evening, Opalescence/Humility, Smokestack, the rituals, storm/coin-flip/alt-win engines — so
  noncreature permanents and spells (53 % of the list) intentionally outnumber bodies.

Two things this implies for play and for V2:

1. **Board presence is thin by design.** Combat is less central than in a typical cube; games
   lean on engines, symmetry-breakers, and alt-wins. That is the Odyssey "weird feel" we're
   chasing, but it raises the floor on each creature's job (see CREATURE-SUPPORT.md — every added
   body enables a specific gimmick rather than just filling a curve).
2. **Lands at 10.4 % are bang-on convention** — no fixing/ramp problem to flag here; the
   divergence from established cubes is entirely in the creature/noncreature split, not mana.

## Reproduce

```sh
sqlite3 data/cube_editor.sqlite "
SELECT COUNT(*) total,
       SUM(type LIKE '%Creature%') creatures,
       SUM(type LIKE '%Land%') lands,
       SUM(type NOT LIKE '%Land%') nonland
FROM cube_cards WHERE removed=0;"
```
