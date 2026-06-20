# oddysey2026

A redesign of **Odyssey block as the designers wanted it to be, not as they shipped it** — a
low-power but decision-intensive cube where the half-finished counter / alternate-win themes of
2001–2003 are made to actually work (including win conditions **errata'd to be stronger**).

## Card pool
- **Core:** Odyssey block — *Odyssey* (ODY), *Torment* (TOR), *Judgment* (JUD).
- **Gap-fillers:** selected creatures from Onslaught block — *Onslaught* (ONS), *Legions* (LGN),
  *Scourge* (SCG) — chosen to support the tribal themes the core lacks.

## Themes
- **Tribal:** **Birds** and **Clerics** (Onslaught-era tribes) — e.g. the Aven / Soulcatcher
  bird package; Soulcatchers' Aerie is a wanted build-around.
- **Counters matter:** funky, non-+1/+1 counters that scale (fuse, shred, delay, plague,
  depletion, gold, mine, trap, luck) plus the interactive "shield counter" Phantom cycle.
- **Alternate win conditions:** the spine of the cube; several are intended to be buffed/errata'd
  so they're real win cons rather than cute do-nothings.

## Power level
Deliberately low (the Odyssey-block common/uncommon band) but **fun = lots of meaningful
decisions**. Reference: the `odysseyblock` CubeCobra cube.

## Files
- [`counter-cards-discovery.md`](./counter-cards-discovery.md) — every counter-related card in the
  pool, **explained verbatim** (oracle text quoted exactly), grouped by counter type and theme,
  with design notes. The oracle text is pulled directly from `data/mtg.sqlite` via the `mtgbrain`
  tool in this repo, so it is exact — e.g. `mtgbrain card "Bomb Squad"`.
