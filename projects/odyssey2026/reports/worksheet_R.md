# Worksheet R — MONO-RED (oddysey2026)

Target ~46. Result: 24 creatures + 22 noncreatures = **46**.

## What this worksheet does

Mono-red is the cube's **big-red mana-sink** and **honest-storm / push-your-luck**
engine. It does NOT play Modern burn or swingy dragons (FIRE-free per spec). Three
threads, all incremental and decision-dense:

1. **Cumulative-upkeep "age counter" mana sink** (spec #1). Headliner **Braid of Fire**
   (`{1}{R}` "Cumulative upkeep — Add {R}") floods late-game mana that the curve has
   nowhere else to spend. Sinks: **Comet Storm** (multikicker X), **Savage Firecat**
   ({3}{R}{R} 0/0 with seven +1/+1 counters, sheds one per land tapped — a *land-tax
   counter sink* that rewards NOT spending mana), **Chainflinger** / **Grim Lavamancer**
   repeatable pings, **Kamahl, Pit Fighter**. Companion age-counter time bombs
   **Magmatic Core** and **Heart of Bogardan** double as proliferate targets (feeds U/R,
   B/x proliferate) and as Smokestack-shift fuel.
2. **Honest storm** (spec #16 — win-cons stapled to a spell, cast and spent):
   **Grapeshot** (1 dmg/prior spell), **Lightning Storm** (Ad Nauseam Line-B finish —
   discard drawn lands to scale, also the Modern-AdNauseam kill alongside Lab Maniac, per
   spec #4), **Empty the Warrens** (slower/fairer Goblin-token kill). Enablers:
   **Pyretic Ritual**, **Seething Song**, **Wheel of Fortune**, **Reforge the Soul**,
   **Faithless Looting**. Feeds U/R and B/R storm.
3. **Coin-flip / push-your-luck** (spec's "go nuts" lane): **Goblin Bomb** (5 fuse → 20
   to a player — a counter+flip near-win), **Game of Chaos**, **Fiery Gambit**,
   **Mana Clash**, **Goblin Festival**, **Molten Birth**, **Impulsive Maneuvers**.
   **Chance Encounter** (win at 10 luck counters) is the alt-win anchor.

Creature base is deliberately weak ODY-block bodies that *break stalls* through
threshold, first strike, reach (flyer), and sacrifice/discard outlets that recycle into
the graveyard-storm and madness threads.

## Guild archetypes fed

- **R/G** — big-red / Braid mana-sink, X-spells, age counters.
- **U/R** — spells-matter / storm (rituals + Grapeshot + wheels).
- **B/R** — storm / discard-outlet graveyard value; Ensnaring Bridge (B/R gold shift)
  behind which the red beats poke through.
- **Coin-flip** macro-theme across the cube (Chance Encounter / Goblin Bomb).

## Key cards & combos

- **Braid of Fire + Comet Storm / Savage Firecat** — pour age-counter mana into an X-sink
  or a counter body the instant before it would burn off.
- **Storm line**: ritual → ritual → wheel → **Grapeshot / Lightning Storm**; Empty the
  Warrens as the grindy backup that leaves a board.
- **Discard outlets → madness / graveyard storm**: Minotaur Explorer, Frenetic Ogre,
  Dwarven Strike Force, Pitchstone Wall, Burning Inquiry, Faithless Looting, Cathartic
  Reunion, Change of Fortune feed One-With-Nothing graveyard-storm (cross-sheet) and turn
  on **threshold** for Possessed Barbarian / Fledgling Dragon / Chainflinger / Barbarian
  Ring.
- **Bomb Squad** — fuse-counter removal that also doubles as a slow saboteur clock; pairs
  with the proliferate package.

## Notable cuts / maybeboard

- **Mirror March** — flagged `too_strong` in source (its modern "flip until you lose"
  wording is a copy *engine*, not a spent payoff). → **maybeboard**.
- **Underworld Breach** — recursion value engine, board-camping, source-flagged strong;
  against spec #16. → **maybeboard**.
- **Mine Layer (ODY)** and **Crazed Firecat (TOR)** — named in the brief and *are* genuine
  Odyssey-block reds, but they are **NOT in the curated `odysseyblock_creatures.tsv`**
  (they live only in the broader `block_creatures.tsv`). HARD RULE forbids pulling
  creatures from outside the curated pool, so they are **maybeboard** with that note, not
  cube. Goblin Traprunner is Onslaught — out of block entirely.
- Curve-redundant ODY-block beaters cut to keep the count honest: Barbarian Outcast
  (needs Swamps), Ember Beast (can't act alone), Halberdier, Longhorn Firebeast,
  Dwarven Driller (redundant land-hate), Pardic Firecat, Pardic Collaborator, Balthor the
  Stout (Barbarian lord — thin tribe), Spark Mage, Anger (graveyard-haste better in a
  reanimator shell). All in maybeboard.
- **Smoke** (red Winter Orb) — stax piece; sits better as a B/G Smokestack-adjacent gold
  consideration; maybeboard.
- Removed from red entirely (locked gold shift): **Ensnaring Bridge → B/R sheet**.

## Errata flags

- **Chance Encounter** — `dead=true, errata=true`. A literal-blank build-around if you
  never flip. **Errata proposal**: *"Also: at the beginning of your upkeep, you may flip a
  coin; if you win, put a luck counter on it. Whenever you cast a spell with {X} in its
  cost, flip a coin; if you win, put a luck counter on it."* — gives it self-driven
  counters tied to the coin-flip + big-red X-sink threads so it advances on its own.
