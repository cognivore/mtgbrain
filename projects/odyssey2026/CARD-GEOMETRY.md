# Old-frame (1997–2003) card geometry

Measured from **real Scryfall card scans** of Odyssey-block printings (745×1040 PNG,
full card incl. its thin black border). Reference scans are cached under
`render-cache/cards/<Card>/scryfall/<id>.png` (e.g. Cephalid Looter [ODY], Wild Mongrel
[ODY], Spark Mage [ODY], Possessed Aven [TOR], Cabal Pit [ODY], Propaganda [7ED]).

All values are **fractions of the full card** (0..1). Because a Scryfall full-card scan is
the 2.5×3.5 card *including* its black border, these fractions map directly onto our render
**FACE** box (the 2.5×3.5 area we center inside the 2176×2960 / 800-DPI-with-bleed canvas).

## Measured spec (real ODY card, cross-checked)

| Element | x | y | w | h | notes |
|---|---|---|---|---|---|
| Title text | 0.105 | 0.045 | ~0.50 | 0.045 | cap-height ≈ **0.030** of card height |
| Mana cost (pip cluster) | 0.84 | 0.045 | 0.115 | 0.040 | right-aligned, pips ≈ title height |
| **Art window** (inner edge) | 0.105 | 0.095 | 0.795 | 0.475 | bottom ≈ 0.570 |
| Type line | 0.105 | 0.575 | ~0.60 | 0.045 | cap ≈ **0.028** |
| Set symbol | 0.80 | 0.575 | 0.10 | 0.045 | right of type, vertically centered |
| **Text box** (inner edge) | 0.105 | 0.625 | 0.795 | 0.265 | bottom ≈ 0.890 |
| Rules text | inset ~0.02 inside the box | | | | cap ≈ **0.030**; vertically centered when short |
| Power/Toughness | 0.83 | 0.945 | 0.105 | 0.040 | bottom-right |
| Illus. line | 0.27 | 0.918 | ~0.46 | 0.030 | centered |
| Copyright line | 0.25 | 0.948 | ~0.50 | 0.025 | centered, below Illus. |

Key proportions: art window is ~**0.80 wide** and ~**0.475 tall**, starting at x≈0.105 / y≈0.095;
title and mana share the top band at y≈0.045; the type band sits at y≈0.575; the rules box fills
y≈0.625–0.890; P/T and the two credit lines live in the bottom border (y≈0.92–0.95).

## What we actually render onto

We composite onto **cardconjurer's "Seventh" (2001/7th-edition) frame PNGs**, the only
freely-available transparent old-frame art. Our element bounds are cardconjurer's own
`packSeventh.js` values, which are measured against *that* frame and land within ~2–3% of the
real-ODY spec above. The **art window is dictated by the frame PNG's transparent hole**
(cardconjurer's window ≈ x0.12 y0.099 w0.767 h0.443), so the art layer is sized to that, not to
the real-ODY 0.795×0.475 — otherwise the illustration would spill past the printed frame edge.

**Pixel-exact real-ODY geometry is not achievable with these assets** — it would require
transparent *real* Odyssey frame PNGs (the real scans have art/text baked in). The cardconjurer
recreation is the faithful approximation; the deltas are sub-3% and not visually objectionable
(see the blue Propaganda render).

## Typography rules (locked in)

- Title / type / P/T: **Goudy/MPlantin, white fill + tight black outline** (the engraved old
  look; readable on the beige nameplate). NOT a big drop-shadow (that doubled the title).
- Rules: MPlantin black, vertically centered, reminder text (parentheticals) **italic**.
- **Mana pips are an icon font and must never be italic** — the `<i class="ms">` tag and the
  `.rules i { italic }` rule would skew them (the Propaganda `{2}` bug). Pinned with
  `.ms, .rules .ms, .titlerow .ms { font-style: normal }`. See RENDER-REGRESSION.md.
- Pips sized ~**title height** (`ms-cost` ≈ 1.18em of the mana text), generic pips grey with the
  classic drop-shadow.
- Bottom: `Illus. <artist>` then `NOT FOR SALE · ™ & © <year> Wizards of the Coast, Inc.`
