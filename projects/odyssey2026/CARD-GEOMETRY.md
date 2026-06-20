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

## Ported verbatim from cardconjurer `packSeventh.js`

The renderer's bounds, font sizes, shadows and **mana symbols** are now taken
directly from cardconjurer's Seventh pack (`~/Github/cardconjurer`, the source
the frame PNGs/fonts already come from). All values are fractions of the FACE;
font px = fraction × FACE_H (2800); the FACE sits inside the 2176×2960 bleed.

| Element | cardconjurer value |
|---|---|
| Title | x 0.1134, y 0.0481, w 0.7734, h 0.041, size 0.041, goudy, white |
| Mana | x 0.1067, y 0.0539, w 0.8174, **size 72/1638**, right-aligned |
| Type | x 0.1074, y 0.5486, w 0.7852, h 0.0543, size 0.032 |
| Rules | x 0.128, y 0.6067, w 0.744, h 0.2724, size 0.0358, **top-aligned** |
| P/T | x 0.8074, y 0.9043, w 0.1367, size 0.0429 |
| Illus / Wizards | y 1908/2100 size 0.0172 / y 1940/2100 size 0.0143 |

- **Shadow** (title/type/PT): sharp black, **no blur**, offset (0.002·W, 0.0015·H)
  — cardconjurer applies `shadowOffsetX/Y` with `shadowBlur` unset.
- **Pips are cardconjurer's own SVGs** (`img/manaSymbols/*.svg`, copied to
  `assets/mana/`): each is a complete pip (coloured disk + glyph baked in, e.g.
  `w.svg` pale-yellow circle + sun, `3.svg` grey-tan circle + numeral). Rendered
  as `<img class=ms-img>` at **0.78em** of the surrounding font (cardconjurer
  draws each at `textSize·0.78`). This makes pips match cardconjurer pixel-for-pixel
  (the andrewgioia mana font's sun is fatter and was the prior mismatch).

## Typography rules (locked in)

- Title / type / P/T: **Goudy/MPlantin, white fill + tight black outline** (the engraved old
  look; readable on the beige nameplate). NOT a big drop-shadow (that doubled the title).
- Rules: MPlantin black, vertically centered, reminder text (parentheticals) **italic**.
- **Mana pips are an icon font and must never be italic** — the `<i class="ms">` tag and the
  `.rules i { italic }` rule would skew them (the Propaganda `{2}` bug). Pinned with
  `.ms, .rules .ms, .titlerow .ms { font-style: normal }`. See RENDER-REGRESSION.md.
- **Title shadow is faint** (`text-shadow:1px 2px 2px rgba(0,0,0,0.32)`). A heavy shadow
  (the old `3px 4px 3px / 0.55`) drags the title's optical centre down ("text too low") and
  reads as doubled — verified with `tools/cardgeom` against the real 7ED Serra Angel scan.
- **Cost-row generic/colourless mana is a BARE numeral — no disk** (real old-frame look). The
  disk is stripped only under `.manacost` (`.manacost .ms-0…ms-x { background:none; … }`), so
  inline `{2}` in *rules* text keeps its disk (correct for body text).
- Coloured cost pips: a **pale grey-green disk** (`#c1c3b1`) ~title height (`ms-cost` 1.12em)
  with the symbol scaled to **0.82** inside it → a pale ring around the glyph (the andrewgioia
  sun is fatter than the real 7ED sun; the ring + faint `0.22` shadow tames the "too heavy"
  look). Tune these against a real scan with `tools/cardgeom/card-compare`.
- Bottom: `Illus. <artist>` then `NOT FOR SALE · ™ & © <year> Wizards of the Coast, Inc.`
