# oddysey2026 — Primer

**What if the properly-old, 90s / early-2000s Magic ideas had a backbone of 2026 game design?**

A 360-card singleton cube. No FIRE, no Dragon-Magic, no swingy bombs — *pure incremental "small"
games* with a lot of different **lock pieces** and **honest alternate win conditions**, fought over
by the deliberately **weak creatures** of Odyssey block. Very low power; very high decision density.

---

## The two halves of the cube

1. **The creatures are weak, and that's the point.** Every creature is drawn from the curated
   `/odysseyblock` cube (243 cards) — threshold beaters, madness one-drops, Phantom damage-shields,
   Cephalid self-millers, Squirrel/Saproling tokens, dwarves and clerics. None of them ends the game
   on its own. Their job is to **break stalls**, **block**, **chip**, and **enable** — to be the
   board that the lock pieces and combos are played around. (A handful of structurally necessary
   non-block creatures are sanctioned exceptions: **Narcomoeba, Laboratory Maniac, Death's Shadow,
   Leveler** — see [`ERRATA-AND-COLORSHIFTS.md`](./ERRATA-AND-COLORSHIFTS.md).)

2. **The non-creatures win the game.** Drawn from *any* era, these are the spine: the prison pieces,
   the counters-that-count-to-a-win, the storm kills, the wheel-locks, the life-swaps. They are how
   you actually close — and there are **a dozen different ways to do it**.

So a game is: durdle out an incremental board of small creatures while you assemble *your* way to
win, and disrupt *theirs*. Whoever's engine survives the other's lock pieces takes it.

---

## How you win (the spine)

The cube is built so that almost every colour pair has a real, distinct win condition. They sort
into four families:

**A. Count to a number (alternate win counters)**
- **Chance Encounter** (R) — 10 luck counters from winning coin-flips.
- **Darksteel Reactor** (C) — 20 charge counters (errata'd to seed + grow with proliferate).
- **Simic Ascendancy** (UG) — 20 growth counters off +1/+1 effects.
- **Helix Pinnacle** (G) — 100 tower counters (errata'd to advance itself); the slow green sink.
- **Azor's Elocutors** (UW) — 5 filibuster counters, reset when you take damage.
- **Triskaidekaphile** (U) — exactly 13 cards in hand. **Test of Endurance** (W) — 50 life.
  **Near-Death Experience** (W) — be at exactly 1 life.

**B. Honest combo (a win *stapled to a spell*, cast and spent — principle #16)**
- **Ad Nauseam** (B) → draw a chunk or your whole library → **Tendrils of Agony** / **Grapeshot**
  (storm) *or* **Lightning Storm** / **Laboratory Maniac** / **Jace, Wielder of Mysteries** (deck-out).
  *Never Thassa's Oracle.*
- **High Tide** (U) → free spells (**Snap**, **Frantic Search**, **Reset**, **Turnabout**) → **Brain
  Freeze** mill or **Stroke of Genius**.
- **Doomsday** (B), **Leveler + Sundial of the Infinite** (C, #13), **One With Nothing + Shadow of
  the Grave** (B, errata) — all one-shot, all fragile.

**C. Prison inevitability (lock them out, then a slow clock)**
- **Smokestack** (BG) sacrifice lock · **Opposition** (GW) tap-down · **Tangle Wire** (UG) fading
  soft-lock · **Ensnaring Bridge** (BR) + **Black Vise / The Rack / Lantern** mill behind it ·
  **Winter Orb / Static Orb / Storage Matrix** (C) · **Teferi's Puzzle Box** (C) + hand-punishers ·
  **Opalescence + Humility** (UW) turning the board into a 1/1 stalemate you then **Replenish** out of.

**D. Just attack.** The weak creatures, gone wide (Squirrels, tokens, Birds) or grown (threshold,
madness, **Mirror Universe** after self-damage with **Death's Shadow**), close games the locks slow down.

The full crossover is the fun: proliferate (UG) accelerates the count-to-a-win *and* the fade
counters *and* the storage lands; discard outlets feed graveyard-storm *and* madness *and* hellbent
*and* Ensnaring Bridge; coin-flips feed Chance Encounter *and* Goblin Bomb *and* Krark's Thumb.

---

## The ten colour pairs (every pair has one — none are blank)

| Pair | Archetype | Wins by |
|---|---|---|
| **WU** Azorius | Enchantress / Replenish prison | Opalescence + Humility lock → **Replenish**; Sigil of the Empty Throne; Test of Endurance |
| **UB** Dimir | Honest control-combo / deck-out | **Ad Nauseam** → Lab Maniac / Jace / Lightning Storm; High Tide → Brain Freeze; Doomsday |
| **BR** Rakdos | Hellbent graveyard-storm + Bridge prison | **Tendrils / Grapeshot** storm; or **Ensnaring Bridge** + The Rack / Black Vise / Lantern |
| **BG** Golgari | Smokestack sacrifice + attrition | **Smokestack** grind; Pernicious Deed resets; Cabal Archon drain |
| **RG** Gruul | Big-red mana sink | **Braids of Fire** → Comet Storm / Magmatic Core / fat threshold beats |
| **GW** Selesnya | Opposition tokens | Flood bodies → **Opposition** tap-down → swarm |
| **WB** Orzhov | Life as a resource | **Mirror Universe** swap; Death's Shadow; drain; life-total alt-wins |
| **UR** Izzet | Spells & coin-flips | Storm (Brain Freeze / Empty the Warrens); Stitch in Time turns; **Krark's Thumb** flips |
| **WR** Boros | Tax aggro | A clock behind Cage of Hands / Ghostly Prison / Goblin Trenches |
| **UG** Simic | Proliferate & fading counters | **Simic Ascendancy** / Darksteel Reactor / Helix Pinnacle; **Tangle Wire** soft-lock |

Each pair is anchored either by a colour-shifted gold signpost (Opposition, Tangle Wire, Smokestack,
Ensnaring Bridge, Opalescence) or a named keystone (Ad Nauseam, High Tide, Braids of Fire, Mirror
Universe). Full card lists and how each plays: [`COLOR-PAIRS.md`](./COLOR-PAIRS.md).

## The honest-combo rule (principle #16)

Combo is allowed, but only the *fragile, telegraphed* kind. A win must be a **commitment stapled to
a spell** — cast it, spend it, and if it's answered you're out the cards. That's why **Ad Nauseam**
is in and **Bolas's Citadel** / **Thousand Year Storm** / **Yawgmoth's Bargain** are not: those camp
the board giving repeated free value, which is exactly the "storming off with training wheels" the
cube refuses. Board-camping engines are flagged `too_strong` and parked on the maybeboard.

The same rule de-blanks the do-nothings: a card that would be a literal blank (One With Nothing,
the Lantern pieces, the bare alt-win counters) gets a small **errata** so it does *something* on its
own — an ETB, a built-in clock, a mana sink. See `ERRATA-AND-COLORSHIFTS.md`.

---

## The colour-shifted signposts

Five iconic artifacts/enchantments were **colour-shifted by the cube's own data** to become gold
build-around signposts — making the strongest stax a real deckbuilding commitment, not a free splash:

| Opposition → **GW** | Tangle Wire → **UG** | Smokestack → **BG** | Ensnaring Bridge → **BR** | Opalescence → **UW** |
|---|---|---|---|---|

(Why each landed where it did is in `ERRATA-AND-COLORSHIFTS.md` — every shift is justified by a DB
metric: token density, proliferate density, counter-mover colours, discard-your-hand colours.)

---

## How it plays / the metagame

It's a slow, grindy, **rock-paper-scissors** of four macro-decks:
- **Prison-control** (WU enchantress, mono-stax) beats combo and durdle, loses to fast tax-aggro.
- **Combo** (UB Ad Nauseam, BR storm, U High Tide) beats durdle and slow control, folds to disruption
  + the prison pieces + a fast clock.
- **Tax-aggro / tokens** (WR, GW Opposition) beats combo and control, grinds out vs midrange.
- **Midrange attrition / counters** (BG sacrifice, UG counters, WB life, RG mana-sink) grinds
  everyone and wins the long game.

Because the creatures are weak, **board stalls happen** — and the cube is *full* of stall-breakers
(threshold fliers, trample, Rabid Elephant, tappers, Opposition, alt-wins) so the stalls always crack
in a decision-rich way rather than a topdeck war.

---

## Drafting it

- The **colour-shifted golds** are open signposts: first-pick Opposition and you're the GW tokens
  drafter; Smokestack, you're BG sacrifice; etc.
- **Counters, discard outlets, and coin-flips are connective tissue** — they're good in several decks,
  so they wheel less but flex more. Read what's open.
- Every two-colour lane has a build-around payoff and the enablers to support it; the cube is wide
  enough that **two drafters can share a guild** without starving (esp. UB, BR, the counters web).
- Low power means **curve and consistency win**, not raw bombs. Mana rocks and the fixing lands matter.

---

## Files
- [`cube360/cube_master.csv`](./cube360/cube_master.csv) — the 360, every column + flags.
- [`cube360/by-color/`](./cube360/by-color/) — one worksheet per colour (W/U/B/R/G/Multicolor/Colorless/Land).
- [`cube360/maybeboard.csv`](./cube360/maybeboard.csv) — every card considered but cut, with reasons.
- [`COLOR-PAIRS.md`](./COLOR-PAIRS.md) · [`COLOR-ROLES.md`](./COLOR-ROLES.md) — the ten guilds and the five colours.
- [`ERRATA-AND-COLORSHIFTS.md`](./ERRATA-AND-COLORSHIFTS.md) — every change from printed cards.
- [`reports/`](./reports/) — per-worksheet discovery reports.
- **Deferred (afternoon task):** functional-slot decklists + example 40-card decks proving each archetype.
