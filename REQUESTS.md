# Request log

Running log of user requests against the render/editor pipeline, newest first.
Status: ☐ todo · ◐ in progress · ☑ done

---

## ☐ 5. Colorless / Eldrazi (devoid) frame is ugly — use modern cardconjurer colorless frame
**Asked:** 2026-06-30. The tan/brown "colorless" 8ED frame (e.g. *Vexing Scuttler*, an
Eldrazi with Emerge) looks bad. Pull the **devoid / Eldrazi colourless frame** from the more
modern cardconjurer asset set and use it for colorless/devoid (non-artifact) cards.
**Scope:** download the modern colorless frame asset; wire `frame_file_8th` (and PT box) to
select it for colorless non-artifact cards; re-render affected cards.

## ☐ 4. 8ED cube — MPC downloads / publish parity (BOTH)
**Asked:** 2026-06-30. The 8ED (`pointless-b-side`) cube has no export path: `render_selfhost`
+ `publish.sh` are hard-coded to the old-frame `cards/` dir; the 8ED cube renders to `cards8/`
which nothing reads. Wanted: **(a)** a local MPC bundle — stage `cards8/*/card8.png` as
`<Card-Name>.png` in `render-cache/mpc-8ed/`; **(b)** full publish parity — 8ED selfhost crops,
S3 sync, CubeCobra CSV.

## ☐ 3. Browser navigation is slow — make card preview snappy
**Asked:** 2026-06-30. Navigating cards in the editor re-downloads the full ~5 MB,
2176×2960 print PNG every time with **no HTTP caching**, displayed at 320px. Fix: serve a
downscaled (~760px) cached JPEG preview by default (full PNG via `?full=1`), add
`ETag` + `Cache-Control` + `304 Not Modified` to `/api/render` so revisits are instant.

## ◐ 2. Inline mana pips sit too low in text boxes
**Asked:** 2026-06-30. Inline mana symbols/costs in rules boxes were dropped below the text
mid-line (`vertical-align:-0.16em`/`-0.14em`). Bumped all body-text `.ms-img` to `-0.07em`
across every 8th template (title `.manacost` left at `-0.02em`). **Pending the batched
re-render** to take effect.

## ☑ 1. Glue punctuation to inline mana costs (no orphaned dots)
**Asked / done:** 2026-06-30. A mana pip followed by `. , ; : ! ?` could line-break, orphaning
the mark. `manaify()` now wraps the pip + trailing punctuation in a `white-space:nowrap` span.
Both cubes re-rendered; old-frame cube republished to S3. Commits `fc45de2`, `7eb3fc5`.
