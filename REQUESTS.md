# Request log

Running log of user requests against the render/editor pipeline, newest first.
Status: ☐ todo · ◐ in progress · ☑ done

---

## ☑ 9. Maybeboard modules — launch / grid review / promote + cube trim
**Asked:** 2026-07-11. Three modes of operation for growing and shrinking a cube list:
(1) **Launch** a maybeboard module research from a Scryfall-ish query — `/maybe` UI with a
live preview over the whole card pool (`/api/pool`, in-cube cards auto-excluded), or
agent-driven via `mtgbrain edit maybe-launch --sql "SELECT name FROM cards WHERE …"`.
(2) **Review** it as a visual spoiler grid — colour sections → type → mana value, every grid
annotated with a sticky stats overlay (cards, creatures + token-makers = bodies vs
non-creature, per-colour counts, avg MV); click a card to cycle want → rejected → pending;
**Promote** adds every want to the cube (tagged `maybe:<module>`, appended to the source
list, decision `pending` so it enters the normal review pipeline).
(3) **Trim** (`/trim`) — the live cube in the same annotated grid, target 612 (MPC order),
✨ suggested cuts keep the bodies ratio at a chosen target, spread cuts across colours
(proportional cap), lowest cube-Elo first, lands never auto-suggested; Apply = soft-remove
(restorable from the 🗑 pile). Grid images: the card's own render preview when one exists,
else Scryfall `normal` cached on disk — fetches are serialised process-wide (a grid of
concurrent misses used to earn a 429 storm). Both editor instances (old-frame + 8ED) get
all three modes; modules live in each editor DB (`maybe_modules` / `maybe_cards`).

## ☑ 8. Import the real cardconjurer Level Up frame (replace synthesized leveler)
**Asked:** 2026-07-01. Replace our CSS-band leveler with the authentic levelers/regular frame,
custom-geometry approach (like devoid). Done: `card_template_8th_leveler_cc.html` with geometry
measured from the frame (art x.0767 y.1129 w.846 h.4419); level-up cost band + base P/T + two
tier bands mapped onto the frame's baked P/T pill banners. Only 1 leveler in the cube
(Hedron-Field Purists) — verified. `leveler_frame_8th` + `build_html_8th_leveler_cc`.

## ☑ 7. Devoid geometry polish + pip shadow direction
**Asked:** 2026-07-01. Devoid fixes: title/mana lowered onto the nameplate, type text down +
set symbol right, P/T anchored in the textbox bottom-right corner, Illus/legal dropped into the
bottom margin. Pip drop-shadow flipped to fall **left+bottom** (was right+bottom), a touch more
pronounced — across ALL 8th templates (applied cube-wide via the v9 re-render).

## ☑ 6. 8ED pips — inconsistent drop shadow (title vs textbox)
**Asked:** 2026-06-30. Name-box pips looked shadowed while textbox pips looked flat.
Root cause: pip SVGs have no baked shadow; the only shadow was a fixed `1px` `.ms-img`
filter, near-invisible at 2176px render scale and reading differently on the light title
bar vs white textbox. **Fix:** all 8th templates now use one em-scaled drop-shadow
(`0.035em 0.045em 0.01em rgba(0,0,0,0.5)`) so it scales with pip size and matches everywhere.
Bumped 8ED cache v7→v8; cube re-rendered (493, 0 errors).

## Note (2026-06-30): editor backend / clobbering
The running editor backends were the **pre-fix binary** and re-rendered (clobbered) corrected
`cards8/` PNGs on view (stale cache version → cache miss → old-logic render). Killed both;
relaunch on the rebuilt binary picks up all fixes AND the snappy preview path. Old-frame cube
was not clobbered (only the 8ED editor was browsed).

---

## ☑ 5. Colorless / Eldrazi (devoid) frame — modern M15 devoid frame + custom geometry
**Asked:** 2026-06-30. The tan/brown "colorless" 8ED frame (e.g. *Vexing Scuttler*) looks bad.
Use the modern M15 **devoid / Eldrazi** frame for colorless non-artifact cards.
**Done so far:** M15 devoid frames + PT downloaded to `assets/frames8/devoid/` (w/u/b/r/g/m/a/l
+ pt). Only 3 colorless cards in the cube (Vexing Scuttler, Endless One, Gaea's Will).
**Blocker (proven by a test render):** the M15 devoid frame uses **M15 geometry**; dropped into
the 8th template (`card_template_8th.html`) the title bar misaligns — a doubled/clipped title.
The 8th text positions (title 6.29%, art hole, type 57.2%, rules 62.77%) don't match M15.
**Plan:** give devoid colorless cards their **own template + geometry** (a `card_template_8th_devoid.html`
with M15 art-hole / title / type / textbox / PT positions), like the saga/pw/adventure special
layouts — then re-render the 3 cards. The frame-select hook (`devoid_frame_8th`) was wired then
backed out to avoid shipping a broken title; see the `TODO(devoid)` in `frame_file_8th`.

## ☑ 4. 8ED cube — MPC downloads / publish parity (BOTH)
**Asked:** 2026-06-30. The 8ED (`pointless-b-side`) cube has no export path: `render_selfhost`
+ `publish.sh` are hard-coded to the old-frame `cards/` dir; the 8ED cube renders to `cards8/`
which nothing reads. Wanted: **(a)** a local MPC bundle — stage `cards8/*/card8.png` as
`<Card-Name>.png` in `render-cache/mpc-8ed/`; **(b)** full publish parity — 8ED selfhost crops,
S3 sync, CubeCobra CSV.

## ☑ 3. Browser navigation is slow — make card preview snappy
**Asked:** 2026-06-30. Navigating cards in the editor re-downloads the full ~5 MB,
2176×2960 print PNG every time with **no HTTP caching**, displayed at 320px. Fix: serve a
downscaled (~760px) cached JPEG preview by default (full PNG via `?full=1`), add
`ETag` + `Cache-Control` + `304 Not Modified` to `/api/render` so revisits are instant.

## ☑ 2. Inline mana pips sit too low in text boxes
**Asked:** 2026-06-30. Inline mana symbols/costs in rules boxes were dropped below the text
mid-line (`vertical-align:-0.16em`/`-0.14em`). Bumped all body-text `.ms-img` to `-0.07em`
across every 8th template (title `.manacost` left at `-0.02em`). **Pending the batched
re-render** to take effect.

## ☑ 1. Glue punctuation to inline mana costs (no orphaned dots)
**Asked / done:** 2026-06-30. A mana pip followed by `. , ; : ! ?` could line-break, orphaning
the mark. `manaify()` now wraps the pip + trailing punctuation in a `white-space:nowrap` span.
Both cubes re-rendered; old-frame cube republished to S3. Commits `fc45de2`, `7eb3fc5`.
