# oddysey2026 — Errata & Colour-shifts

Every card the cube changes from its printed form, and **why**. Three buckets: (1) colour-shifts
(printed colourless/one-colour → gold or a new colour), (2) errata (rules-text changes — spec-mandated
or proposed to de-blank a build-around per spec #12/#15), (3) cards that need manual handling (not in
the DB, or mis-named in the spec). Decisions are data-driven where the data exists — see the metric in
each row.

---

## 1. Colour-shifts

### 1a. The five marquee artifacts/enchantments (spec #5–#8, #10)

| Card | Printed | → In-cube | Guild | Decided by (DB metric) |
|---|---|---|---|---|
| **Opposition** | U | **G/W** | Selesnya | Biggest turn-4 creature flood: White = 50 cheap (≤3 MV) creatures + Green = 14 token-makers in pool; next colour ≤2 token-makers. GW is where the board gets wide enough to tap the opponent out. |
| **Tanglewire** (Tangle Wire) | C | **U/G** | Simic | Most proliferate support: U 17 + G 11 *non-creature* proliferate sources (+10 colourless). Its fade counters live where counters live. |
| **Smokestack** | C | **B/G** | Golgari | The cube's counter-*movers* are all green (Powerful Broker, Quarry Hauler, Forgotten Ancient, Maulfist Revolutionary); black supplies counter-*removal* + the sacrifice fodder the soot engine eats. |
| **Ensnaring Bridge** | C | **B/R** | Rakdos | The colours that most want to empty their hand (hellbent): R 68 + B 41 discard/hellbent cards (madness B 31, R 21). Bridge + empty hand = their board can't swing; you win behind it. |
| **Opalescence** | W | **U/W** | Azorius | Spec #10 — anchors the UW enchantments deck. With Humility, every creature (incl. animated enchantments) is a 1/1 with no abilities = symmetric soft-lock. |

> **Why gold matters here:** making these gold is a *power brake*, not a buff. A two-colour
> requirement means the strongest stax/prison pieces cost a real deckbuilding commitment and can't
> splash into every deck — exactly the "decks are not on rails, but they pay for their power" feel.

### 1b. Teferi's Puzzle Box hand-punishers → Black (spec #14)

The Puzzle Box wheel-lock needs a colour home. Its punishers are colourless artifacts that play as
black "you drew/held cards, lose life" effects, so they shift to **B** (keeping Puzzle Box itself,
Anvil, Memory Jar colourless as the symmetric engine):

| Card | Printed | → | Note |
|---|---|---|---|
| Black Vise | C | **B** | The turn-1 clock vs a flooded hand. |
| Iron Maiden | C | **B** | Black Vise for multiplayer; in 1v1 a near-dup → one is maybeboard. |
| The Rack | C | **B** | The empty-hand axis (pairs with Ensnaring Bridge / discard). |
| Ebony Owl Netsuke | C | **B** | 4 damage when an opponent holds 7+ — Puzzle Box / Font of Mythos payoff. |
| Misers' Cage | C | **B** | Softer (5+ cards) variant; surplus. |
| Cursed Rack | C | **B** | Caps a hand at 4 — forces discards into The Rack range. |

### 1c. Other shifts

- **Phyrexian Tyranny** {U}{B}{R} → **U/B** (drop the red pip): a symmetric "each draw costs 2 life
  or {2}" tax that belongs to the Dimir wheel-lock; the red is vestigial.

---

## 2. Errata

### 2a. Spec-mandated
- **Enchanted Evening** (#10) — ETB *also* mill the top 3 cards of your library, then return an
  enchantment card from your graveyard to your hand. (Turns the "all permanents are enchantments"
  symmetric-Armageddon enabler into a self-feeding engine for Replenish/Opalescence.)

### 2b. De-blanking build-arounds (spec #12 / #15 — "make it do something on its own")
These are kept because the cube *wants* the build-around, but as printed they can do nothing without
a partner, so each gains a small floor:

| Card | As printed | Proposed errata |
|---|---|---|
| **One With Nothing** | Discard your hand. (literally nothing else) | *…then each opponent loses 1 life for each card discarded this way.* Loops with Shadow of the Grave for a real storm/drain payoff. **(needs your sign-off on the exact number)** |
| **Lantern of Insight** | Reveal tops; no clock alone | *At the beginning of your upkeep, each player mills 1.* Becomes a true Lantern-control clock (#15). |
| **Pyxis of Pandemonium** | Exile tops face-down; pays off only on death | *…also mill 1 each upkeep.* A real inevitability clock. |
| **Mindcrank** | Needs a life-loss source | *…also mills you 1 each upkeep* so it acts alone. |
| **Chance Encounter** | Does nothing until you flip | *Enters with two luck counters.* (Seeds the count so a flip deck isn't drawing a blank.) |
| **Darksteel Reactor** | Inert without charge-counter support | *Enters with a charge counter; {2}: put a charge counter on it.* Built-in slow clock + mana sink. |
| **Helix Pinnacle** | Pure mana sink, can whiff | (Fine as-is — it *is* the mana sink; no errata, just slow.) |
| **Krark's Thumb** | Dead without coin-flips | (Fine — the cube supplies the flip density; no errata.) |
| **Parallax Inhibitor** | Only re-arms fade counters | *Enters with a fade counter and {T}: add a fade counter to target permanent.* Ties into Tangle Wire / Parallax. |
| **Battle of Wits** | Unfieldable in singleton (needs 200-card deck) | **Replace the wording** with a cube-legal alt-win, e.g. *"At the beginning of your upkeep, if you have 40+ cards in your library and 7+ card types in your graveyard, you win."* **(needs your final wording)** |

> Cards explicitly **not** errata'd because they already act alone (verified): **Codex Shredder** and
> **Ghoulcaller's Bell** already mill 1/turn; The Rack/Cursed Rack do nothing into an empty hand but
> that's the *opposite axis* by design, not a blank.

### 2c. Self-mill / theme errata (design decision)
- **Brain Freeze** — errata'd so it can only target **yourself**: *"You mill three cards. Storm."* It is
  a **self-mill enabler** (fuels threshold, Cabal Ritual threshold, Songs of the Damned, and the Lab
  Maniac self-deck-out), **not** a clock on opponents. (Its printed text targets any player; the cube
  removes the opponent-mill mode now that conventional Storm is cut — see §6.)

---

## 3. Manual handling (not in DB / mis-named)

- **Braids of Fire** — *not present in `mtg.sqlite`* (Coldsnap). Added by hand. Card text:
  `{R} Enchantment — Cumulative upkeep—Add {R}.` Each upkeep it gains an age counter and you add that
  many red mana (1, 2, 3, …) which you sink into instants / X-spells / activated abilities that turn —
  the cube's signature **big-red mana-sink** engine (spec #1). Pairs with Mage-Ring Network, storage
  lands, X-spells, and proliferate (more age counters = more mana).
- **"Shadow of the Industry"** (spec #11) — **does not exist** in any data source; almost certainly a
  mis-remembered name. The life-loss deck instead runs **Death's Shadow** (the true anchor) +
  **Shadow of Mortality** (the nearest real card: a huge body that costs less the lower your life).
  **Please confirm** which card you meant, if any.
- **Sunken City** — its real text is a blue-creature anthem with cumulative upkeep, **not** an
  Islands-maker. The High-Tide "Islands-matter" plan uses real enablers instead — Spreading Seas,
  Sea's Claim, Convincing Mirage, Celestial Dawn, Quicksilver Fountain. (Sunken City available via
  errata "Lands you control are Islands" if you specifically want it; otherwise maybeboard.)

---

## 4. Sanctioned out-of-block creature exceptions (spec #9)

Creatures come from the curated `/odysseyblock` pool. These few non-pool creatures are allowed because
they are *structurally necessary* and have no in-pool equivalent — each flagged `exception` in the sheet:

| Creature | Why it's necessary |
|---|---|
| **Narcomoeba** | Self-mill payoff that hits the battlefield from the graveyard (spec #9 names it). |
| **Laboratory Maniac** | The deck-out win for Ad Nauseam / High Tide — "never Thassa's Oracle" (#4). |
| **Death's Shadow** | The life-loss payoff (#11); nothing in-pool scales with low life. |
| **Leveler** | Empties your own library for the Lab-Maniac / Sundial line (#13); an artifact creature. |

(Kept *very* sparing — every other creature is from the curated pool.)

---

## 5. Flagged too-strong → maybeboard (philosophy #16 / low-power)
Not deleted — parked on the maybeboard with a reason, in case you want to raise the band later:
Yawgmoth's Bargain, Necropotence, Channel, Demonic Tutor, Palinchron, Time Spiral, Mind Over Matter,
Bolas's Citadel, Thousand Year Storm, Trinisphere (format-warping floor), Exquisite Blood (+ Sanguine
Bond = infinite loop), Solemnity (anti-synergy: shuts off the cube's own fade/counters).

---

## 6. Cut by design (off-theme, not too-strong)
Removed from the cube because they cut against the intended feel, not because of raw power. These leave
slots to backfill (see `cube360/cube_list.txt`).

| Card | Why cut | What replaces it |
|---|---|---|
| **Tendrils of Agony** | Conventional Storm is out — no turn-3 storm kills. | The slow **discard-drain** (Bond of Agony, Sickening Dreams, Shrieking Affliction, Gibbering Descent, Faith of the Devoted). |
| **Grapeshot** | Storm cut. | UR closes on coin-flips (Chance Encounter), Lightning Storm/Fevered Visions burn, and the High Tide → Stroke of Genius deck-out. |
| **Empty the Warrens** | Storm cut. | Go-wide lives in GW (Opposition) / WR (Goblin Trenches); RG tokens via Molten Birth. |
| **Test of Endurance** | "Gain to 50" alt-win is cringe / off-theme; it also split WB into two anti-synergistic halves. | WB is now a single low-life plan (Mirror Universe swap + Death Grasp / Bond of Agony). The only count-to-a-number alt-win left is **Chance Encounter**. |

> **Brain Freeze stays** (errata'd to self-mill only — see §2c); it is not a storm payoff in this cube.
> **Lightning Storm stays** — it is a charge-counter burn spell, not a Storm card.
