# cardgeom — pixel-by-pixel card geometry analysis

Standalone Rust tool (own `[workspace]`, so it does not touch the parent
`mtgbrain` crate) for verifying the old-frame ("Seventh"/7ED) renderer against
**real Scryfall scans**, instead of eyeballing.

Why it exists: a render canvas has MPC **bleed** around the 2.5×3.5 face; a real
scan does not. Everything is normalised to **fractions of the face** (bleed
cropped) so a 745×1040 scan and a 2000×2800 face compare directly.

## Build

```sh
cd tools/cardgeom && cargo build --release   # or: nix develop -c cargo build --release
```

Uses only the same `image`/`clap`/`serde` deps as the parent, so the repo
`flake.nix` dev shell already covers it.

## Programs

- **`card-measure`** — geometry of one image as JSON (title/mana extents, shadow
  "dark load"). `--render` crops bleed; `--scan` (default) treats the whole image
  as the face. `--annotate out.png` draws the detected boxes.
- **`card-compare --ref <scan> --render <render>`** — aligns both faces and prints
  per-element deltas + a plain verdict (title too low / pips too big / too
  shadowed). Exits non-zero on mismatch → usable as a regression gate.
- **`card-crop IMG OUT x0 y0 x1 y1 [--render]`** — 3× magnify a face-fraction
  region (bleed-aware) for eyeballing pips/title.
- **`card-preview`** + **`shot.sh`** — fill the *live* `src/card_template.html`
  for one card and screenshot it with headless Chrome **without rebuilding the
  main crate** (fast template/geometry iteration). `shot.sh` mirrors the Chrome
  flags in `src/render.rs::compose` so the preview matches the real pipeline.

## Example (the Serra Angel regression)

```sh
B=target/release
$B/card-compare \
  --ref ../../render-cache/cards/Serra_Angel/scryfall/7ed.png \
  --render ../../render-cache/cards/Serra_Angel/serra_FIXED.png
```

## Caveats / metric honesty

- The **mana "dark load"** metric is dominated by the andrewgioia sun-glyph black
  area (fatter than the real 7ED sun), so it stays ~2.5× even when the box-shadow
  is near-zero — it flags *glyph weight*, not just shadow. Judge pips with
  `card-crop` too.
- The **pip-height** metric picks up the dark frame border lines inside the mana
  band; trust it only when pips clearly dominate. Title metrics are reliable.
- Search bands stop just above the art window (y≈0.097); pushing lower lets dark
  art clouds corrupt the readings.
