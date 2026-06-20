# Worksheet G — MONO-GREEN (oddysey2026)

Target ~44. Result: 26 creatures + 18 noncreatures = **44**.

(23 of the 26 creatures are from the curated `odysseyblock_creatures.tsv`; 3 are
`source="exception"`, justified below. All 18 noncreatures are `source="any-era"`
mono-green build-around / token / ramp pieces.)

## What this worksheet does

Mono-green is the cube's **token-flood + +1/+1-counter + slow-ramp** color. No FIRE,
no swingy bombs (Overrun / Crush of Wurms cut). Four incremental, decision-dense threads:

1. **Token flood for Opposition (>G/W) and go-wide sac.** A deep squirrel/saproling/
   elephant token base: **Squirrel Nest** + **Nut Collector** + **Squirrel Wrangler**
   (land→squirrels), **Saproling Burst** (Fading 7 — a *fade-counter* engine, proliferate
   payoff), and a flashback token suite (**Chatter of the Squirrel**, **Acorn Harvest**,
   **Call of the Herd**, **Elephant Ambush**, **Grizzly Fate**). These bodies tap under
   **Opposition** and feed B/G sacrifice.
2. **+1/+1 counter / proliferate spine (feeds U/G).** Counter-makers and counter-movers:
   **Forgotten Ancient** (the slow counter-engine 0/3), **Quarry Hauler** (ETB counter
   doubler), **Chlorophant** (upkeep grower), **Nantuko Cultivator** (discard lands →
   counters + cards), **Travel Preparations** (two counters, W flashback bridge), plus
   **Saproling Burst**'s fade counters and **Helix Pinnacle**'s tower counters as
   proliferate targets.
3. **Honest alt-win (cast & spent, no board-camping value).** **Helix Pinnacle** —
   pure mana sink, `{X}: tower counters`, win at 100. It generates no cards and no board,
   so it is the honest green alt-win the spec wants (vs. a value engine).
4. **Slow ramp / fixing (feeds R/G big-mana).** **Wild Growth**, **Utopia Sprawl**,
   **Rampant Growth**, **Carpet of Flowers** (anti-blue), plus dorks **Diligent Farmhand**,
   **Werebear**, and land-recur **Cartographer** — fuel for Helix Pinnacle and the big
   flashback fatty **Roar of the Wurm**.

Creature base is intentionally weak ODY-block bodies that **break stalls** via threshold
(Werebear/Krosan Avenger/Seton's Scout/Possessed Centaur/Centaur Chieftain), madness
(Basking Rootwalla / Arrogant Wurm + Wild Mongrel as the discard outlet), reach blockers
(Anurid Swarmsnapper), and combat-math punishers (Rabid Elephant).

## Guild archetypes fed

- **G/W** — Opposition + tokens (squirrel/saproling flood; **Phantom Nishoba** is the
  in-pool G/W shield bridge; Travel Preparations' W flashback).
- **U/G** — proliferate / +1/+1 counters (Forgotten Ancient, Quarry Hauler, Saproling
  Burst fade counters, Helix tower counters).
- **R/G** — ramp / big-mana (Wild Growth, Utopia Sprawl, Carpet, dorks → Helix / Roar).
- **B/G** — sacrifice (token fodder; Diligent Farmhand & Squirrel Wrangler sac outlets;
  Possessed Centaur turns black at threshold).

## Key cards & combos

- **Opposition (>G/W) + Squirrel Nest / Nut Collector / Saproling Burst** — turn a wide,
  cheap token board into a soft lock by tapping the opponent's lands and creatures.
- **Helix Pinnacle + ramp** (Wild Growth / Utopia Sprawl / Carpet) — dump flooded mana
  into tower counters as the honest, slow win that nothing else green provides.
- **Forgotten Ancient / Quarry Hauler + Saproling Burst** — move/double counters and
  proliferate fade counters; the U/G counter glue.
- **Wild Mongrel (discard outlet) + madness** (Basking Rootwalla, Arrogant Wurm) and
  flashback token spells — every discard recycles into board or madness value.
- **Moment's Peace** — repeatable fog (flashback) that buys the slow ramp/token plans time
  without ending the game.

## Notable cuts / maybeboard

- **Prompt-named creatures NOT in the curated pool** — **Aquamoeba** (it's blue anyway),
  **Krosan Wayfarer**, **Ironshell Beetle**, **Phantom Tiger / Phantom Centaur /
  Phantom Nantuko** are genuine ODY-block but are absent from
  `odysseyblock_creatures.tsv`. HARD RULE forbids non-pool creatures except the sparing
  exception budget (spent on Forgotten Ancient / Squirrel Wrangler / Quarry Hauler). All
  → **maybeboard** with that note. The shield-counter mechanic is represented in-pool by
  **Phantom Nishoba** (G/W).
- **Overrun**, **Crush of Wurms** — swingy alpha-strike / triple-6/6 bomb, against the
  "no swingy bombs / no FIRE" spec. → **maybeboard** (`too_strong`).
- **Verdant Succession**, **Bearscape**, **Holistic Wisdom** — recursion / board-camping
  value engines (and Verdant whiffs in singleton since it fetches a same-named copy). →
  **maybeboard**.
- **Epic Struggle** (win at 20 creatures) — second alt-win, but 20 bodies in a 40-card
  singleton deck is unrealistic; Helix Pinnacle covers the green alt-win. → **maybeboard**.
- **Choke** (master-listed mono-G) — anti-Island prison/sideboard hate, off-theme for a
  build-around worksheet. → **maybeboard**.
- Ramp/fatty redundancy trimmed for count: **Far Wanderings**, **Deep Reconnaissance**,
  **Krosan Tusker**, **Beast Attack**, **Elephant Guide**, **Ground Seal**,
  **Living Wish** (cube has no outside-the-game zone). All → **maybeboard**.

## Errata flags

- **Helix Pinnacle** — `dead=true, errata=true` (honoring the master-candidates flags). As
  printed it is a near-blank durdle that almost never reaches 100 and does nothing else.
  **Errata proposal**: *"When Helix Pinnacle enters, put four tower counters on it. At the
  beginning of your upkeep, put a tower counter on it for each Forest you control."* —
  gives it a self-driven clock tied to the green ramp/devotion plan so it advances without
  all-in mana dumps, keeping it an honest (still slow) alt-win rather than a dead card.
