# Worksheet C — Colorless (artifacts) — discovery report

**Scope:** all colourless artifacts for oddysey2026. This is the cube's shared toolbox: the
mana-denial / stax locks, the honest alternate-win combos that break stalls, the wheel + hand
engines that feed the puzzle-box punishers, and the fixing rocks that let every guild run a
build-around. Source: `master_candidates.tsv` rows with `cube_color='C'`, plus **Mind's Eye**
pulled from the DB (named in the brief, not yet in the TSV) and **Leveler** kept as an
`exception` artifact-creature. **46 cards in, 6 to maybeboard.**

> The 4 colourshifted artifacts — **Smokestack** (B/G), **Tangle Wire** (U/G),
> **Ensnaring Bridge** (B/R), **Opposition** (G/W) — are GOLD now and live on the gold
> worksheets, NOT here. The puzzle-box hand punishers (**Black Vise, Iron Maiden, Ebony Owl
> Netsuke, Misers' Cage, Cursed Rack**) colourshift to **B**; **The Rack** colourshifts to **B**
> too, so it leaves this worksheet for mono-black.

## What this worksheet does (functional buckets)

1. **Mana-denial / stax (10):** Winter Orb, Static Orb, Storage Matrix, Sphere of Resistance,
   Crawlspace, Tsabo's Web, Cursed Totem, Damping Sphere, Defense Grid, Possessed Portal. The
   symmetric speed-bumps that make slow build-arounds survivable and turn every untap/cast into a
   decision. Trinisphere is the one piece cut for being too strong (3-mana floor hard-locks a
   low-curve cube) → maybeboard, `too_strong`.
2. **Counter alt-wins & proliferate engines (7):** Darksteel Reactor (count-to-20 win),
   Contagion Clasp, Contagion Engine, Throne of Geth, Spawning Pit, Astral Cornucopia, Empowered
   Autogenerator. The colourless half of the proliferate / charge-counter theme that the Simic
   (Tangle Wire fades) and big-red mana-sink shells lean on.
3. **Wheel / draw enablers (8):** Teferi's Puzzle Box (the #14 wheel-lock keystone), Anvil of
   Bogardan, Memory Jar, Howling Mine, Temple Bell, Font of Mythos, Otherworld Atlas, Mind's Eye.
   These over-fill hands so the B-side puzzle-box punishers (Black Vise / Owl Netsuke) actually
   fire — Temple Bell/Memory Jar let you flood on YOUR upkeep before the opponent can dump.
4. **Lantern win #15 (5):** Codex Shredder, Ghoulcaller's Bell, Lantern of Insight, Mindcrank,
   Pyxis of Pandemonium. Selective mill behind Ensnaring Bridge. Three need errata so they're a
   real clock alone (see below).
5. **Life-swap finishers (3):** Mirror Universe (named anchor), Soul Conduit (repeatable backup),
   Possessed Portal counts under stax. Pairs with the WB/BR life-loss shell (Death's Shadow).
6. **Singleton specials:** Sundial of the Infinite (#13 — Stifle the Leveler draw-step loss),
   Leveler (`exception` artifact-creature — empties own library for the deck-out / Sundial line),
   Krark's Thumb (coin-flip linchpin), Cursed Scroll (hellbent burn), Lion's Eye Diamond (ritual +
   discard for the Infernal-Tutor / hellbent lines), Trading Post & Crucible of Worlds (grindy
   mana-sinks for big-red).
7. **Fixing rocks (8):** Mind Stone, Fellwar Stone, Prismatic Lens, Coldsteel Heart, Worn
   Powerstone, Star Compass, Pristine Talisman, plus Expedition Map (utility-land tutor). All
   low-power, period-appropriate, no fast-mana jump — Worn Powerstone feeds the big-red sinks
   without breaking the curve.

## Guild archetypes fed

- **UB Dimir control-combo:** wheel-lock (Puzzle Box + Mind's Eye + Howling Mine), Sundial,
  Leveler deck-out, life rocks.
- **UG Simic / BG Golgari proliferate:** Reactor / Contagion / Throne / Spawning Pit / Autogenerator
  + Astral Cornucopia as colourless proliferate targets.
- **BR Rakdos hellbent:** Cursed Scroll, Lion's Eye Diamond, Anvil/Memory Jar discard fuel behind
  the (gold) Ensnaring Bridge.
- **Mono-B puzzle-box punisher shell:** every wheel engine here is the enabler; the punishers
  themselves sit in black.
- **WB/BR life-loss:** Mirror Universe + Soul Conduit swap a tanked life total back.
- **RG big-red mana-sink:** Worn Powerstone, Astral Cornucopia, Empowered Autogenerator, Trading
  Post, Crucible of Worlds.
- **Lantern (any control shell):** the 5-piece mill suite behind a bridge / fateseal.

## Errata flags (cards that would otherwise be blanks — principle #12 / #15)

- **Darksteel Reactor** (`dead`+`errata`): solo it only ticks 1/turn to 20. Errata: *enters with 3
  charge counters; whenever you proliferate, put an extra charge counter on it.* Ties it to the
  proliferate theme so it's a real clock, not a 20-turn timer.
- **Krark's Thumb** (`dead`+`errata`): does nothing without coin-flips. Errata: *{2}, {T}: flip a
  coin* (gives the deck a flip source on the linchpin itself) so it isn't dead in a flip-light hand.
- **Lantern of Insight** (`dead`+`errata`): pure information without a mill partner. Errata:
  *at the beginning of your upkeep, mill 1.* Real lantern clock alone.
- **Mindcrank** (`dead`+`errata`): needs an external life-loss source. Errata: *{T}: target player
  mills 1* (self-acting mill that still snowballs off any life loss).
- **Pyxis of Pandemonium** (`dead`+`errata`): symmetric exile that may never resolve. Errata:
  *its {T} ability also mills each player 1* so the exile clock advances every turn on its own.

`Leveler` is flagged `exception` (artifact-creature from outside the curated creature pool, kept
solely for the structural deck-out / Sundial interaction, not as a beater).

## Notable cuts (→ maybeboard, nothing discarded)

- **Trinisphere** — `too_strong`; 3-mana floor hard-locks this deliberately low-curve cube.
- **The Rack** — colourshifts to **B** with the rest of the puzzle-box punisher family.
- **Quicksilver Fountain** — Islands-lock / High-Tide prison engine; belongs to the U/UB Islands
  theme, not the colourless core.
- **Bottled Cloister** — hand-hiding for Bridge, but redundant with the wheel package and slow.
- **Containment Construct** — artifact-CREATURE discard-recursion engine; not a named include and
  the creature pool is reserved.
- **Parallax Inhibitor** — niche fade-refueler; only live alongside a heavy Parallax/fading build,
  would need errata to not be a blank.
