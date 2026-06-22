# Odyssey Cube — Archetype Primer

This is the drafter's guide to the cube. The format is **drafted in colour pairs**, so the document is
organised that way: ten guilds, each a distinct engine with an **honest, distinct win condition**. The
per-colour "what each colour does" map up front exists to show the *overlaps* — the connective cards
(counters, discard, coin-flips, tokens, life-payment, fixing) that let pairs bleed into each other so
decks are never on rails.

Every card named below is verified against the cube's **design intent**, not just the raw list. A card
behaves by its **real Oracle text** *except* where this cube changes it, and the only changes are in
[`ERRATA-AND-COLORSHIFTS.md`](./ERRATA-AND-COLORSHIFTS.md). Two design decisions matter throughout and
are newer than some companion docs: **conventional Storm is cut** (Tendrils of Agony, Grapeshot, Empty
the Warrens are out — the black "build-your-own-Tendrils" is a *slow discard-drain* instead, never a
turn-three kill), and **Brain Freeze is errata'd to mill yourself only** (a self-mill enabler, not an
opponent clock). **Test of Endurance is also cut.** Where the older companion docs
([`COLOR-PAIRS.md`](./COLOR-PAIRS.md), [`PRIMER.md`](./PRIMER.md), [`COLOR-ROLES.md`](./COLOR-ROLES.md))
cite cards that aren't in the cube, those are flagged at the end under
[Fact-check & corrections](#fact-check--corrections).

---

## How a deck wins (the four families)

The cube is built so almost every pair has a real way to close. They sort into four families.

**A. Count to a number (alternate-win counters).** Only **Chance Encounter** (R — win at 10 luck
counters, errata'd to *enter with 2*) survives as a count-to-a-number win. Test of Endurance is cut, and
the blue/Simic "count to a win" payoffs the docs lean on (Azor's Elocutors, Simic Ascendancy, Darksteel
Reactor, Helix Pinnacle, Triskaidekaphile) **are not in the cube** — see the corrections section.

**B. Honest combo (a win stapled to a spell — principle #16).** With Storm cut, the spell-kills are:
**Ad Nauseam** → **Bond of Agony** (pay-life drain) or a deck-out; **High Tide** → **Stroke of Genius**
(deck the opponent, or draw yourself out); the **Leveler + Sundial of the Infinite** / **Laboratory
Maniac** / **Jace, Wielder of Mysteries** deck-out; **Mana Severance** + a draw into the same deck-out;
**Doomsday**; **Replenish** mass-return; **Mirror Universe** life-swap; **Sickening Dreams** discard-burst.
Each is cast and spent — answer it and the pilot is out the cards. The **slow discard-drain** that
replaces storm (Bond of Agony, Sickening Dreams, Shrieking Affliction, Gibbering Descent, Faith of the
Devoted) scales with cards spent, by design — a grind, never an explosive turn.

**C. Prison inevitability (lock them out, then a slow clock).** **Smokestack** (BG), **Opposition** (GW),
**Tangle Wire** (UG), **Ensnaring Bridge** (BR) behind the black hand-punishers, the **Teferi's Puzzle
Box** + **Phyrexian Tyranny** redraw-lock, **Winter Orb / Static Orb / Storage Matrix**, and the
**Opalescence + Humility** board-flatten the WU deck **Replenish**es out of.

**D. Just attack.** The deliberately weak creatures — gone wide (Squirrels, Goblin Soldiers, Birds),
grown (threshold beaters, madness fatties, Lhurgoyfs), or pointed (the pingers and burn) — close games
the locks slow down. In a low-power cube where board stalls are common, the cube is full of
stall-breakers so they crack in a decision-rich way.

---

## What each colour does (the overlap map)

The point of this section is the *overlap*: a pair is the union of two of these toolboxes, and the
shared rows are why guilds bleed into each other.

### White — the wall, the enchantment, the tax
White never races; it **outlasts**. Weak Birds/Clerics/Nomads wall the ground then chip over it.
- **Enchantress / Replenish engine:** **Mesa Enchantress** (the *only* repeatable enchantment-cast
  draw in-cube), **Enlightened Tutor**, **Academy Rector**, **Sterling Grove** (G/W), **Auramancer**,
  **Replenish**, **Starfield of Nyx**, **Serra's Sanctum**.
- **Pillowfort / tax:** **Ghostly Prison**, **Sphere of Safety** (scales with your enchantments),
  **Story Circle**, **Solitary Confinement**, **Rule of Law**, **Suppression Field**, **Glowrider**,
  **Aura of Silence**, **Cage of Hands**, **Teferi's Moat** (W/U).
- **Prevention / walls:** **Master Apothecary**, **Militant Monk**, **Blessed Orator**, the **Phantom
  Nomad/Flock** counter-shields, **Commander Eesha**, **Embolden**, **Moment's Peace** (G).
- **Removal:** **Oblivion Ring**, **Oblation**, **Kirtar's Wrath**, **Seal of Cleansing**, **Final
  Payment** (W/B).
- **Lock enchantments (gold + mono-W):** **Opalescence** (→U/W) + **Humility**, **Dovescape** (W/U),
  **Enchanted Evening** (W/U, errata'd), **Earnest Fellowship**.
- **Life floor:** the two cards that let you exist at ≤0 life — **Phyrexian Unlife** (but all damage then
  comes as infect, so 10 poison still kills you) and the mono-black **Pact Weapon** (an Equipment, no
  poison drawback). (No life-total alt-win remains — Test of Endurance is cut.)

White brings almost **no card draw and no rituals**: it taxes, prevents, recurs enchantments, gains life.

### Blue — the engine, the combo, the tempo
Blue is the **enabler**; it almost never wins by attacking.
- **Self-mill / loot:** **Cephalid Looter / Broker / Vandal / Sage / Scout**, **Careful Study**,
  **Mental Note**, **Thought Scour**, **Consider**, **Frantic Search**, **Compulsion**, **Brain Freeze**
  (errata: mill *yourself* only), **Stitcher's Supplier** (B), **False Memories**, **Narcomoeba** (off
  the mill).
- **High Tide free-spell package:** **High Tide**, **Snap**, **Reset**, **Turnabout**, **Frantic
  Search**, **Capsize**, **Lion's Eye Diamond** (colourless).
- **Deck-out / mill wins:** **Laboratory Maniac**, **Jace, Wielder of Mysteries**, **Mana Severance**,
  **Stroke of Genius**, **Sphinx's Tutelage**, **Glimpse the Unthinkable** (U/B), **Folio of Fancies**.
- **Counters / interaction:** **Exclude**, **Dismiss**, **Forbid**, **Rewind**, **Syncopate**,
  **Circular Logic** (Madness).
- **Islands-prison / control:** **Spreading Seas**, **Sea's Claim**, **Convincing Mirage**, **Aquitect's
  Will**, **Back to Basics**, **Standstill**, **Day's Undoing**, **Propaganda**.
- **Proliferate half:** **Steady Progress**, **Tezzeret's Gambit**, **Thrummingbird**, **Dreamtide
  Whale**, **Experimental Augury**.
- **Tutors / extra turns / payoffs:** **Merchant Scroll**, **Time Stretch**, **Stitch in Time** (U/R),
  **Epic Experiment** (U/R).

### Black — the deepest colour: fuel, disruption, kills
The most-loaded colour; the meeting point of half the cube's themes.
- **Rituals:** **Dark Ritual**, **Cabal Ritual** (threshold), **Culling the Weak**, **Songs of the
  Damned**, **Rain of Filth**, **Skirge Familiar**.
- **Discard outlets & payoffs:** **Putrid Imp**, **Sadistic Hypnotist**, **Bone Miser**, **Faith of the
  Devoted** (triggers on cycle **or discard**; no cyclers in cube → a pure **discard** drain), **Feast
  of Sanity**, **Nihilistic Glee**.
- **Graveyard fuel / recursion:** **Stitcher's Supplier**, **Darkblast** (Dredge), **Gravedigger**,
  **Cabal Surgeon**, **Shadow of the Grave**, **Ill-Gotten Gains**, the **Mortivore** / Lhurgoyf shell.
- **Tutors:** **Demonic Consultation**, **Tainted Pact**, **Infernal Tutor** (Hellbent), **Demonic
  Counsel** (Delirium), **Spoils of the Vault**, **Doomsday**.
- **Removal:** **Terror**, **Vendetta**, **Feed the Swarm**, **Seal of Doom**, **Chainer's Edict**,
  **Innocent Blood**, **Cabal Patriarch**, **Faceless Butcher**, **Font of Agonies**.
- **Discard-drain & life-as-resource (the storm replacement):** **Bond of Agony**, **Sickening Dreams**,
  **Shrieking Affliction**, **Gibbering Descent**, **Ad Nauseam**, **Greed**, **The Last Ride**,
  **Death's Shadow**, **Organ Grinder**.
- **Hand-size punishers (colour-shifted to black):** **Black Vise**, **The Rack**, **Iron Maiden**,
  **Ebony Owl Netsuke**, **Misers' Cage**, **Cursed Rack**.
- **Stax (gold):** **Smokestack** (→B/G), **Ensnaring Bridge** (→B/R), **Braids, Cabal Minion**,
  **Nether Void**, **Mindslicer**, **Sire of Insanity** (B/R), **Phyrexian Tyranny** (→U/B).

### Red — variance, the mana sink, aggression
Red owns the cube's two signatures, **coin-flips** and the **big-red mana sink**. (With Storm cut, red is
no longer a storm-payoff colour — Grapeshot and Empty the Warrens are out.)
- **Rituals / mana sink:** **Pyretic Ritual**, **Seething Song** (both instants), **Braid of Fire**
  (cumulative upkeep, +1 red per age counter), **Heart of Bogardan**, storage lands.
- **Tokens:** **Molten Birth** (doubles as a coin-flip engine), **Goblin Trenches** (R/W).
- **Coin-flip lane:** **Chance Encounter** (alt-win), **Game of Chaos**, **Fiery Gambit**, **Mana
  Clash**, **Impulsive Maneuvers**, **Molten Birth**, **Goblin Festival** (all paired with colourless
  **Krark's Thumb**).
- **Wheels / hand-refill:** **Wheel of Fortune**, **Reforge the Soul** (Miracle), **Change of Fortune**.
- **Discard outlets & madness:** **Faithless Looting**, **Cathartic Reunion**, **Minotaur Explorer**,
  **Frenetic Ogre**, **Glint-Horn Buccaneer**, **Fiery Temper**, **Violent Eruption**.
- **Burn / pingers / sinks:** **Grim Lavamancer**, **Spikeshot Elder**, **Chainflinger**, **Kamahl, Pit
  Fighter**, **Jeska, Warrior Adept**, **Flamewave Invoker**, **Lightning Storm**, **Magmatic Core**
  (creatures only), **Shower of Coals**, **Seal of Fire**.
- **Threshold / Lhurgoyf:** **Fledgling Dragon**, **Magnivore** (sorceries-matter).
- **Land/artifact interaction:** **Goblin Welder**, **Avalanche Riders**, **Dwarven Driller**, **Mine
  Layer**, **Ancient Grudge** (R/G), **Chaos Warp**.

### Green — the board, the ramp, the counters
Green is the **bodies and the mana**; it never wins on its own creatures — it *feeds* other colours' wins.
- **Ramp / fixing:** **Rampant Growth**, **Diligent Farmhand** (basic to the **battlefield**, not the
  yard), **Rites of Spring** (basics to *hand* — a filter/discard outlet, not battlefield ramp),
  **Werebear**, **Dryad of the Ilysian Grove**, **Prismatic Omen**, **Nylea's Presence**, **Cartographer**.
- **Token engines:** **Squirrel Nest**, **Druid's Call**, **Squirrel Wrangler**, **Nut Collector**,
  **Thallid**, **Tukatongue Thallid**, **Penumbra Bobcat**, **Saproling Burst** (Fading — tokens are
  temporary).
- **Flashback token spells:** **Chatter of the Squirrel**, **Acorn Harvest**, **Call of the Herd**,
  **Elephant Ambush**, **Grizzly Fate**, **Roar of the Wurm**.
- **Madness / threshold beaters:** **Wild Mongrel**, **Basking Rootwalla**, **Arrogant Wurm**, **Krosan
  Avenger**, **Seton's Scout**, **Centaur Chieftain**, **Terravore**, **Gorilla Titan**.
- **Counters:** **Quarry Hauler** (a one-shot ETB that adds *or* removes a counter on one permanent),
  **Chlorophant** and **Cocoon** (accumulators), **Travel Preparations** (G/W placer); the only true
  counter-*mover* in the cube is the colourless land **Nesting Grounds**. (**Muscle Burst** is a +X/+X
  pump spell, not a counter card.)
- **Removal / fog:** **Naturalize**, **Tranquility**, **Nantuko Calmer**, **Krosan Reclamation**,
  **Moment's Peace**.

---

## The ten colour pairs

Each pair leads with the **win-method investigation** — the concrete lines that actually close the game
with in-cube cards — then lists the cards and the overlaps. "Reliability" notes are honest: this is a
low-power cube, and most kills are slow and fragile by design.

### WU · Azorius — "Replenish Prison: Animate the Graveyard"
**Plan.** A patient enchantment-prison: tax and pillowfort behind **Ghostly Prison / Sphere of Safety /
Solitary Confinement / Dovescape** while blue's looters dump enchantments into the graveyard, then cast
**Replenish** to return all of them at once for a board-flooding blowout. Blue supplies the dig, the
counters to protect the combo turn, and the self-mill; white supplies the lock pieces, the enchantress
draw, and the tutors.

**How it wins.**
1. **Replenish blowout (the real win).** Get enchantments into your graveyard — **Enchanted Evening**'s
   errata (ETB mill 3 + return one), looting them away with **Careful Study / Cephalid Looter / Frantic
   Search / Brain Freeze** (self-mill), or just letting them die — then **Replenish** returns *every*
   enchantment card at once. With **Opalescence** (→U/W) or **Starfield of Nyx** live, each non-Aura
   enchantment is a creature with power/toughness equal to its mana value (**Ghostly Prison** is a 3/3,
   **Sphere of Safety** a 5/5) and you swing for lethal. *Fragile by design:* Replenish is a single
   sorcery; countered or Disenchanted in response and you've spent your graveyard for nothing. Opalescence
   does **not** animate itself, so you need a second enchantment to attack with. Protect the turn with
   **Dismiss / Rewind** (Exclude only counters creature spells, not a Disenchant aimed at the lock).
2. **Opalescence + Humility soft-lock.** Resolve **Opalescence** *first*, then **Humility** (it must have
   the later timestamp). Every real creature becomes a 1/1 with no abilities, and the animated
   enchantments are also 1/1s — a symmetric board-flatten that neutralises the opponent. *Caveats:* if
   Humility resolves first the animated enchantments are their mana value (3/3, 5/5, …) instead and the
   "everything is 1/1" lock fails; and Humility also switches off your own enchantress/prison abilities
   and shrinks **Cantivore**. It's a **stall, not a kill** — you still need a clock or to unlock and swing.
3. **Dovescape floor.** **Dovescape** counters every noncreature spell and hands the caster Birds. Stop
   casting noncreature spells yourself and win with an evasive body the opponent can't remove —
   **Iridescent Angel** (protection from each colour). *Symmetric:* it also counters your own Replenish
   and removal, so it's a "lock the door when ahead" fork, not a combo enabler.

**Cards.** *Engine:* **Mesa Enchantress**, **Enchanted Evening**, **Enlightened Tutor**, **Academy
Rector**, **Sterling Grove**, **Serra's Sanctum**. *Payoffs:* **Replenish**, **Opalescence**, **Starfield
of Nyx**, **Dance of the Manse**, **Humility**, **Iridescent Angel**, **Cantivore**. *Support:* **Ghostly
Prison**, **Sphere of Safety**, **Solitary Confinement**, **Teferi's Moat**, **Dovescape**, **Karmic
Justice**, **Aura of Silence**, **Dismiss / Rewind**, **Oblivion Ring**, and the W/U fixing (**Mystic
Gate**, **Skycloud Expanse**, **Calciform Pools**).

**Overlaps.** Shares the whole enchantress/pillowfort core with **GW** (Opposition tokens) and **WR**
(tax aggro); shares looters, counters and **Dance of the Manse** with **UB**/**UR**. Note **Opposition**
is colour-shifted to G/W — despite the docs' framing it is a **GW** card, not a WU payoff.

### UB · Dimir — "The Empty Library: Deck-Out and the Redraw Lock"
**Plan.** The cube's honest control-combo. Draw-go interaction and Cephalid self-mill buy time while you
assemble one of several fragile, telegraphed kill turns. UB has more real, distinct in-cube wins than
almost any pair.

**How it wins.**
1. **Leveler + Laboratory Maniac / Jace deck-out (the cleanest line).** With **Laboratory Maniac** or a
   **Jace, Wielder of Mysteries** static online, cast **Leveler** — a 10/10 Juggernaut whose body is
   irrelevant; its ETB exiles your entire library. Your next draw is replaced by "you win." If the
   payoff isn't online yet, activate **Sundial of the Infinite** **during your upkeep** ({1},{T}: end the
   turn) to skip your draw step and avoid decking out. Two cards, both in-cube, with a real safety valve.
2. **Teferi's Puzzle Box + Phyrexian Tyranny redraw-lock (the most reliable).** **Teferi's Puzzle Box**
   forces every player to bottom and redraw their whole hand each draw step; **Phyrexian Tyranny**
   (→U/B) makes each of those draws cost 2 life unless they pay {2}. Cash the forced *full* hands with
   the black punishers that scale with a big hand — **Black Vise**, **Iron Maiden**, **Ebony Owl
   Netsuke**, **Misers' Cage**. (The Rack and Cursed Rack are the *opposite*, empty-hand axis and do
   **not** belong to this full-hand plan.) It's an inevitability clock, not a one-shot.
3. **Ad Nauseam → Bond of Agony drain (or deck-out).** Cast **Ad Nauseam** at a safe life total, rebuild
   with **Dark / Cabal Ritual**, **Songs of the Damned**, **Lion's Eye Diamond**, then **Bond of Agony**
   (pay X, each opponent loses X) for the kill. With Storm cut, the drain is a single big X-spell, not a
   storm chain — realistically the *gas* Ad Nauseam buys plus one large Bond, or a pivot to the deck-out
   (Mana Severance shrinks your library so a draw decks you into Lab Maniac/Jace).
4. **High Tide → Stroke of Genius (deck the opponent, or yourself).** Chain net-free spells (**Snap**,
   **Frantic Search**, **Reset**, **Turnabout**, **Palinchron**) under **High Tide**, then **Stroke of
   Genius** for huge X targeting the opponent's short library to deck them, or yourself after Mana
   Severance into Lab Maniac/Jace. (**Brain Freeze**, errata'd to mill *yourself*, helps empty your own
   library for that self-deck-out — it no longer mills the opponent.) **High Tide only doubles Island
   mana**, so keep black a light splash.
5. **Glimpse / Sphinx's Tutelage opponent-mill.** With Brain Freeze no longer hitting the opponent, the
   dedicated mill plan is **Glimpse the Unthinkable** (mill ten) plus **Sphinx's Tutelage** turning your
   draws into mill, backed by **Ambassador Laquatus** and **Mindcrank** (errata'd to self-mill 1/upkeep;
   turns an opponent's life-loss into that opponent milling). Slow — best layered onto the Puzzle Box
   lock for the extra forced draws.

**Cards.** *Engine:* the Cephalid millers, **Careful Study / Frantic Search / Consider**, **Standstill**,
**Stitcher's Supplier**, **Brain Freeze** (self-mill), **Ad Nauseam**, **High Tide**, **Teferi's Puzzle
Box**. *Payoffs:* **Laboratory Maniac**, **Jace, Wielder of Mysteries**, **Bond of Agony**, **Phyrexian
Tyranny**, **Glimpse the Unthinkable**, **Sphinx's Tutelage**, **Stroke of Genius**. *Support:* **Mana
Severance**, **Sundial of the Infinite**, **Demonic Consultation / Tainted Pact**, **Infernal Tutor +
Lion's Eye Diamond**, **Doomsday**, the counterwall, the black hand-punishers, **Mindcrank**, and U/B
fixing (**Sunken Ruins**, **Darkwater Catacombs** — note these are *not* Islands).

**Overlaps.** Bleeds into **UR** (High Tide / free spells / Stroke of Genius), **BR** (rituals, the
hand-punishers + Ensnaring Bridge, the discard-drain), and **BG** (self-mill, black tutors). *Correction
to intent:* the docs cite "Ad Nauseam → **Lightning Storm**" as a UB finish — Lightning Storm is **red**
and uncastable in straight UB; the in-colour finishers are Bond of Agony, Lab Maniac, and Jace.

### BR · Rakdos — "Hellbent: Empty the Hand, Lock and Drain"
**Plan.** Dump your hand on purpose — it powers an empty-hand prison the same empty hand sustains, plus a
slow discard-drain. With conventional storm cut, the kill is **Bond of Agony** (a big X-spell off a
ritual turn) and the empty-hand drain package behind **Ensnaring Bridge** (→B/R). Creatures are weak
bodies that only enable, ping, or block.

**How it wins.**
1. **Hellbent prison + empty-hand drain (the primary plan).** Live under **Ensnaring Bridge** while you
   strip *their* hand (**Sire of Insanity**, **Mindslicer**, **Sadistic Hypnotist**, **Chain of Smog**,
   **Raven's Crime** with Retrace, the symmetric red wheels). With the opponent hellbent, **Shrieking
   Affliction** (lose 3 at ≤1 card) and **The Rack** drain them out, **Faith of the Devoted** drains 2
   per card *you* discard, and **Glint-Horn Buccaneer** (a 2/4) adds 1 per discard. Slow and the most
   resilient BR plan; a single **Naturalize / Tranquility / Ancient Grudge** on the enchantments unravels
   it, and a topdecking opponent refills to 1 card a turn (Shrieking Affliction carries it; The Rack
   stalls at 2 damage).
2. **Bond of Agony big-mana finisher.** After a ritual turn (**Cabal Ritual** at threshold, **Seething
   Song**, **Rain of Filth** sacrificing all lands, **Songs of the Damned** off a stocked yard), crack
   **Lion's Eye Diamond** in response to **Infernal Tutor** (Hellbent → fetch any card, e.g. Bond of
   Agony), then cast **Bond of Agony** for big X paying X life — each opponent loses X. The cleanest
   single-card kill the rituals can power; needs ~10+ mana to kill from 20, and you pay the life too, so
   it pairs with a fast/low plan or is the last spell.
3. **Sickening Dreams reach.** **Sickening Dreams** (additional cost: discard X cards) deals X to each
   creature *and* each player — a discard-fuelled symmetric sweeper/reach that clears their board and
   finishes a low opponent off the same empty-the-hand engine.

**Cards.** *Engine:* **Lion's Eye Diamond**, **Infernal Tutor**, the rituals, **Wheel of Fortune /
Reforge the Soul** (refuel + empty your hand). *Payoffs:* **Bond of Agony**, **Sickening Dreams**, **The
Rack**, **Shrieking Affliction**, **Gibbering Descent**, **Faith of the Devoted**, **Glint-Horn
Buccaneer**, **Black Vise**, **Spiteful Visions**. *Support:* **Ensnaring Bridge**, **Sire of Insanity**,
**Mindslicer**, **Sadistic Hypnotist**, **Chain of Smog**, **Raven's Crime**, the discard outlets,
**Fiery Temper**, and B/R fixing (**Graven Cairns**, **Shadowblood Ridge**, **Molten Slagheap**).

**Overlaps.** The ritual + **Lion's Eye Diamond + Infernal Tutor** package and the hand-punishers are
shared with mono-B and the **UB** redraw lock; the discard outlets and madness bodies overlap mono-B and
the R coin-flip lane. *Correction to intent:* a "Lab Maniac deck-out" is **not** a straight-BR plan —
Laboratory Maniac is blue, and black's deck-out enablers only mill *you* out; BR must kill with Bond of
Agony / the drain. The **One With Nothing + Shadow of the Grave** loop depends on a *proposed, unfinalised*
errata (each opponent loses 1 life per card discarded — number unconfirmed) and isn't a fixed finisher.

### BG · Golgari — "Soot and Ash: the Symmetric Sacrifice Grind"
**Plan.** Out-resource the table through symmetric sacrifice locks and recursion that refills faster than
the opponent can. Green supplies the disposable bodies, tokens, and lands the lock eats; black supplies
the outlets, the removal, the graveyard fuel, and the actual kills. There is **no clean kill** — every
win is a grind, by design.

**How it wins.**
1. **Smokestack fodder-lock.** Resolve **Smokestack** (→B/G). Each of your upkeeps add a soot counter;
   each player then sacrifices that many permanents. Out-feed it with token engines (**Squirrel Nest**,
   **Thallid**, flashback Squirrels/Elephants) and death-replacement bodies (**Nested Shambler**,
   **Tukatongue Thallid**, **Penumbra Bobcat**, **Blight Mound**), plus **Crucible of Worlds** to replay
   sacrificed lands so the *land* axis becomes one-sided. Assemble the fodder surplus *before* you turn
   soot on, or you lock yourself out too. Close with a surviving **Mortivore / Terravore** or **Organ
   Grinder**.
2. **Pernicious Deed + Squandered Resources (the most reliable swing).** With **Squandered Resources**
   out, sacrifice your lands for mana and activate **Pernicious Deed** for big X — destroying every
   artifact, creature, and enchantment of mana value ≤ X (it does **not** hit lands or planeswalkers,
   which is exactly why it combos with Squandered keeping your mana). A one-sided board reset; rebuild
   with **Gravedigger / Cabal Surgeon / Crucible / Cartographer** while they have nothing. It's a tempo
   blowout, not a kill — you still need a closer.
3. **Graveyard drain / Lhurgoyf beats.** Fuel the yard (**Stitcher's Supplier** mills 3 on ETB and
   death, **Darkblast** dredges, the sac engines dump creatures), then drain with **Organ Grinder** (3
   per activation) or attack with **Mortivore** (creature cards in all yards, regenerates) / **Terravore**
   (land cards in all yards — fed by fetch/Squandered/Smokestack land *deaths*, not by Diligent Farmhand,
   which puts its land on the battlefield). **Bond of Agony** off a big **Songs of the Damned** turn caps it.
4. **Braids, Cabal Minion.** A second sacrifice clock — only when you're already ahead, since it's
   symmetric. Best as a force-multiplier on the Smokestack plan.

**Cards.** *Engine:* **Smokestack**, **Pernicious Deed**, **Squandered Resources**, **Braids, Cabal
Minion**, **Crucible of Worlds**. *Payoffs:* **Organ Grinder**, **Mortivore**, **Terravore**, **Bond of
Agony**, **Blight Mound**. *Support:* the green token/fodder suite, **Innocent Blood / Chainer's Edict**,
**Cabal Patriarch**, the black removal suite, **Gravedigger / Cabal Surgeon / Cartographer**, **Stitcher's
Supplier / Darkblast**, **Quarry Hauler**, **Grave-Shell Scarab**, B/G fixing (**Llanowar Wastes**,
**Twilight Mire**, **Jungle Hollow**).

**Overlaps.** Shares the token/fodder package with **GW** (those bodies are Opposition tappers there) and
the graveyard/Lhurgoyf engine with mono-B and **RG/BG** madness. *Correction to intent:* the spec
justifies Smokestack→B/G via green "counter-movers" Powerful Broker / Forgotten Ancient / Maulfist
Revolutionary — **all three are absent**, and the cube's only true counter-*mover* is the colourless land
**Nesting Grounds**; the one-shot **Quarry Hauler** merely adds/removes a counter on a single permanent.
The real reason B/G works is that green makes the *sac fodder*, not that it moves soot counters.

### RG · Gruul — "The Cumulative-Upkeep Overload"
**Plan.** Ramp green mana plus a self-escalating red engine (**Braid of Fire**) and pour the surplus into
a fistful of small, repeatable overloads. *Reality check:* the docs' marquee single-button sink, **Comet
Storm**, is **not in the cube**, and Braid of Fire's mana is made at your upkeep and empties at end of
step with almost no instant-speed X-spells to dump it into — so the deck is honestly "big-mana value
burn," not "Braid overflow into one giant X-spell."

**How it wins.**
1. **Repeatable-pinger grind (the real default kill).** Stick repeatable damage — **Kamahl, Pit Fighter**
   ({T}: 3), **Chainflinger**, **Flamewave Invoker** ({7}{R}: 5 to a player), **Stormbind** (discard:
   2), **Grim Lavamancer**, **Spikeshot Elder**, **Barbarian Ring** (threshold) — and fire several per
   turn off big mana (Kamahl 3 + Chainflinger 1, or 2 at threshold + Stormbind 2 ≈ 6–7/turn). They
   double as removal so the opponent can't durdle behind blockers. This is how the deck actually closes;
   the burst finishers cap it.
2. **Domain Tribal Flames.** Ramp out lands, resolve **Dryad of the Ilysian Grove** / **Prismatic Omen**
   / **Nylea's Presence** so your lands have all five basic types, then **Tribal Flames** for 5 to the
   face. **Anarchist** recurs it from the yard. *Single point of failure:* strip the domain enabler and
   it's a 2-damage Shock.
3. **Heart of Bogardan self-detonation.** Pay **Heart of Bogardan**'s cumulative upkeep for a few turns
   (using ramp/Braid mana to grow age counters), then stop paying — it deals (2 × age) − 2 to a target
   player **and every creature they control**. Age 6 = 10 to the face plus a board wipe. On-theme and
   real when ramp is online; slow and removal-vulnerable otherwise.
4. **Lightning Storm land-dump.** From a flooded hand, **Lightning Storm** (base 3, +2 per discarded
   land) is instant-speed reach — realistically a 5–9 burst, not a solo kill (killing from 20 needs ~9
   spare lands). It is a charge-counter burn spell, *not* a Storm card, so it survives the storm cut.
5. **Chance Encounter (off-axis).** The coin-flip alt-win (errata-seeded to 2 luck counters) riding
   **Krark's Thumb** and the flip suite — a real but slow secondary, the **UR** lane bleeding into Gruul.

**Cards.** *Engine:* **Braid of Fire**, **Fungal Reaches** (the RG storage land), **Rampant Growth**,
**Dryad of the Ilysian Grove**, **Empowered Autogenerator** / **Astral Cornucopia** (ramp batteries —
they're proliferate *targets*, not sources). *Payoffs:* **Tribal Flames**, **Lightning Storm**, **Heart
of Bogardan**, **Magmatic Core** (damages *creatures only*, at your end step — board control, not
face-kill), **Kamahl**, **Chainflinger**, **Flamewave Invoker**, **Stormbind**, **Chance Encounter**.
*Support:* **Aether Rift** (reanimates discarded *creature cards* — e.g. **Arrogant Wurm**, but **not**
the sorcery **Roar of the Wurm**), domain enablers, **Werebear**, **Wild Mongrel**, **Magnivore**,
**Dire-Strain Rampage / Ancient Grudge / Chaos Warp**, **Crucible of Worlds**, RG fixing (**Mossfire
Valley**, **Fire-Lit Thicket**). Note **Mage-Ring Network** is colourless and only makes {C}; **Rites of
Spring** fills your *hand*, not the battlefield.

**Overlaps.** Coin-flip lane with **UR**; discard/madness and **Stormbind/Aether Rift** with **BR/BG**;
domain fixing and **Last Stand** with the 5-colour splash; storage/charge counters with **UG** proliferate.

### GW · Selesnya — "Opposition Swarm-Lock"
**Plan.** Flood the board with cheap green tokens, then deploy **Opposition** (→G/W) to convert that wide
board into a tap-down prison: each untapped creature taps one of their lands or blockers every turn,
denying mana *and* freezing defense while white anthems push the swarm through.

**How it wins.**
1. **Opposition tap-lock + anthem swarm.** Build a wide board (**Squirrel Nest** for a 1/1 each turn,
   flashback bodies, Saproling/Squirrel makers), resolve **Opposition**, then tap their lands on their
   turn and their blockers on yours. Pump with **Pianna, Nomad Captain** (attackers +1/+1 when she
   attacks), **Nut Collector** (threshold: Squirrels +2/+2), or **Mirari's Wake**, plus **Squirrel Mob**
   as a self-scaling beater (+1/+1 per other Squirrel). Opposition taps *lands as well as creatures*,
   which is what makes it a mana-denial prison rather than a blocker-tapper. *Fragility:* it taps out
   your own board, so you fold to a board wipe at the wrong moment; **Sterling Grove** gives Opposition
   shroud against targeted removal but not against sweepers.
2. **Anthem / token-explosion accelerant.** **Mirari's Wake** (+1/+1 and land-mana doubling, which
   re-buys flashback tokens like **Roar of the Wurm** and **Grizzly Fate**) plus **Travel Preparations**
   (+1/+1 counters on two creatures, flashback {1}{W}) and **Saproling Burst** (temporary — its tokens
   die when it leaves) push a pumped alpha strike through the tapped-down opponent. A layer on the lock,
   not a standalone kill.
3. **Nantuko Monastery closer.** Once threshold is online, animate **Nantuko Monastery** ({G}{W}: 4/4
   first strike) each turn — a sweeper-proof body (it's a land outside combat) that attacks into a
   defenseless opponent. Slow but resilient insurance after a wipe.

**Cards.** *Engine:* **Squirrel Nest**, **Squirrel Wrangler**, **Nut Collector**, **Thallid**, **Druid's
Call** (the tokens go to the enchanted creature's *controller*, so cast it on your own creature),
**Mesa Enchantress**. *Payoffs:* **Opposition**, **Pianna, Nomad Captain**, **Squirrel Mob**, **Mirari's
Wake**, **Travel Preparations**. *Support:* **Sterling Grove**, **Enlightened Tutor**, **Academy Rector**,
**Ghostly Prison**, **Sphere of Safety**, **Moment's Peace**, **Nantuko Monastery**, **By Gnome Means**
(mono-W incidental fodder), G/W fixing (**Sungrass Prairie**, **Wooded Bastion**, **Saltcrusted Steppe**).

**Overlaps.** Heavily overlaps **WU** (the enchantress/pillowfort core, and **Replenish** can mass-return
Opposition + anthems) and shares the token suite with **BG** (Smokestack fodder) and the go-wide half of
**WR** (Pianna, Goblin Trenches). *Correction to intent:* **Glory of Warfare** is **R/W** (Boros) and not
castable in pure GW; **Glittering Wish** is **absent**. The in-cube GW anthems are Pianna, Nut Collector,
and Mirari's Wake (with Squirrel Mob a self-scaling beater); the toolbox is Sterling Grove + Enlightened
Tutor + Academy Rector.

### WB · Orzhov — "The Mortal Coil: Life as Ammunition"
**Plan.** Treat life as a spendable resource: pay life and cards for fuel, deliberately crash your own
total behind a damage floor, and convert the gap into a kill — either drain it out (**Death Grasp**,
**Bond of Agony**) or flip it with a life-total swap (**Mirror Universe**). With Test of Endurance cut,
there is no longer a competing "gain to 50" axis — the deck is one coherent low-life plan.

**How it wins.**
1. **Death Grasp / Bond of Agony (the reliable, honest kill).** Ramp into a big X and point it at the
   face: **Death Grasp** ({X}{W}{B}) deals X *and* gains you X (reach plus a life buffer); **Bond of
   Agony** ({X}{B}, pay X life) makes each opponent lose X. Both are spells you commit and lose if
   answered — usually finishers after chip damage. **Organ Grinder** is a repeatable drain off a stocked
   yard.
2. **Mirror Universe life-swap (the spectacular finisher).** Self-drain with **Greed**, **Ad Nauseam**,
   **The Last Ride**, **Spoils of the Vault**, pain lands, and **Final Payment** (pay 5 life *or* sac —
   your choice) while protected. **Mirror Universe** ({6}, sac on your upkeep) then exchanges life totals
   with a higher opponent; **Soul Conduit** is the slower repeatable backup. *Critical rules point:* only
   **Phyrexian Unlife** or the mono-black **Pact Weapon** lets you survive at 0-or-less life — **Solitary
   Confinement** merely *prevents incoming damage* and does **not** stop you losing to life you *pay or
   lose*, so it buys assembly turns but is not a substitute for the floor. Under Unlife all damage is
   infect, so 10 poison still kills you; Pact Weapon has no such drawback. Mirror Universe is telegraphed
   a full turn and dies to artifact removal. **Death's Shadow** is a huge beater once you're low.
3. **Mindcrank mill-out (redundancy).** **Mindcrank** turns every drain (Bond, Organ Grinder, Death
   Grasp) into the opponent milling. A bonus lane, not the pair's identity — a big Bond usually just
   kills via life loss first.

**Cards.** *Engine:* **Greed**, **Ad Nauseam**, **Font of Agonies**, **The Last Ride**, **Organ
Grinder**, the black rituals. *Payoffs:* **Death Grasp**, **Bond of Agony**, **Mirror Universe**, **Soul
Conduit**, **Death's Shadow**, **Laquatus's Champion** (note its leave-trigger *refunds* the 6 life).
*Support:* **Phyrexian Unlife**, **Pact Weapon** (mono-B ≤0-life floor, no poison drawback), **Solitary
Confinement**, the prevention Clerics, **Final Payment**, **Doom Foretold**, the black removal suite,
**Mindcrank**, incidental lifegain (**Scoured Barrens**, **Ancestor's Chosen**) to buffer the pay-life
turns, W/B fixing (**Caves of Koilos**, **Fetid Heath**).

**Overlaps.** The pay-life / drain core is shared with **BR/UB** (Ad Nauseam, Bond of Agony, Organ
Grinder, the rituals, **Mindcrank**); **Death's Shadow / Phyrexian Unlife / Solitary Confinement** form a
low-life subtheme co-owned with any black self-drain shell; the white prison/Cleric core ties to
**WU/GW/WR**. *Correction to intent:* **Rain of Gore** is **B/R** (Rakdos) and *anti*-lifegain — it
fights this plan, not part of it; **Dark Tutelage**, **Tainted Remedy**, **Near-Death Experience**, and
**Transcendence** are all **absent**.

### UR · Izzet — "Tide & Tempest: Free Spells, Coin-Flips, and Burn"
**Plan.** The cube's spellslinger: chain cheap and net-free spells in one turn for velocity, then win on a
coin-flip alt-win, a burn finisher, or a blue deck-out. With Storm cut and Brain Freeze milling only
yourself, UR's payoffs are **Chance Encounter**, **Lightning Storm / Fevered Visions** burn, and **High
Tide → Stroke of Genius** — all amplified by **Mirari**.

**How it wins.**
1. **Chance Encounter coin-flip win.** **Chance Encounter** (errata-seeded to 2 luck counters) plus
   **Krark's Thumb** (flip two, ignore one ≈ 75% wins), fed by **Game of Chaos**, **Fiery Gambit**,
   **Molten Birth**, **Goblin Festival**, **Impulsive Maneuvers**, **Stitch in Time**. The cube's
   signature flip payoff and now one of UR's headline plans. *Not* a feeder: **Mana Clash** resolves on
   coin *faces*, not on winning a flip. The slowest, most draw-dependent plan, and Chance Encounter is
   naked to the cube's plentiful enchantment removal.
2. **High Tide → Stroke of Genius (deck-out).** Chain net-free/positive spells — **Snap** (needs a
   creature to bounce), **Frantic Search**, **Pyretic Ritual**, **Seething Song** (both *instants*) — to
   a big floating pool under **High Tide**, then **Stroke of Genius** for huge X to deck the opponent's
   short library, or to draw yourself out (after **Mana Severance**) into **Laboratory Maniac / Jace**.
   **Brain Freeze** (errata: mill yourself) accelerates emptying your own library for that line. **Mirari**
   copies one spell — a second Stroke — for {3}.
3. **Lightning Storm / Fevered Visions burn.** **Lightning Storm** (base 3, +2 per discarded land) is
   fat-hand reach after a wheel; **Fevered Visions** wheel-burn (an opponent with 4+ cards takes 2 each
   end step) plus pingers (**Spikeshot Elder**, **Grim Lavamancer**, **Chainflinger**) chip the last
   points. Neither is a Storm card, so both survive the cut.
4. **Epic Experiment burst.** **Epic Experiment** for X free-casts your rituals and cantrips off the top,
   chaining into a **Stroke of Genius** or **Lightning Storm** finish — a high-variance haymaker.

**Cards.** *Engine:* **High Tide**, **Pyretic Ritual**, **Seething Song**, **Snap**, **Frantic Search**,
**Reset** (opponent's turn only), **Turnabout**, **Lion's Eye Diamond**, **Brain Freeze** (self-mill),
**Mirari**, **Krark's Thumb**. *Payoffs:* **Chance Encounter**, **Stroke of Genius**, **Lightning
Storm**, **Fevered Visions**, **Epic Experiment**, **Stitch in Time**. *Support:* **Merchant Scroll**
(blue instants only), **Mystic Speculation**, **Capsize**, **Scrivener / Anarchist** rebuy, the
counterwall, the red pingers, **Magnivore / Fledgling Dragon / Cognivore** as a fair backup, **Cephalid
Coliseum / Barbarian Ring**, U/R fixing (**Cascade Bluffs**, **Shivan Reef**).

**Overlaps.** Shares the High Tide package and the deck-out finishers with **UB**; the coin-flip lane with
mono-R/**RG**; the wheel pieces and **Mirari** with the **UB** redraw lock. With storm gone, UR is the
*fair-tempo / coin-flip / burn* face of blue's velocity, where UB is its *control / deck-out* face.

### WR · Boros — "The One-Sided Clock (Tax-Wall Aggro)"
**Plan.** An asymmetric pillowfort: you keep attacking while the opponent can't. A cheap evasive/threshold
board chips in behind tax-walls that only point one way, and red burn supplies the reach to finish.
**No combo** — it wins by damage, and it's the most internally-honest plan in the cube (every cited
intent card is real and correctly coloured).

**How it wins.**
1. **Tax-wall beatdown (the real plan).** Cheap fliers/walls (**Suntail Hawk**, **Mystic Familiar**,
   **Battlewise Aven**) plus **Pianna** behind **Ghostly Prison** / **Sphere of Safety** (which scales
   with your enchantment count), **Cage of Hands** locking down their best blocker (bounce it for {1}{W}
   to re-lock or dodge removal), and **Glory of Warfare** (+2/+0 on your turn). Swing for ~4–6 turns
   while they're priced out of racing; burn covers the last points.
2. **Goblin Trenches grind.** **Goblin Trenches** ({1}{R}{W}: {2} + sac a land → two 1/1 Goblin
   Soldiers) turns flooded mana into an endless board; **Glory of Warfare** makes each a 3/1 on your
   turn for an alpha strike behind Ghostly Prison. Mana-hungry, but a real stall-breaker and it feeds
   Sphere of Safety's count.
3. **Burn reach.** **Spikeshot Elder** (deals damage equal to its power — a 3-damage gun under Glory),
   **Kamahl**, **Jeska**, **Grim Lavamancer**, **Seal of Fire**, **Flamewave Invoker**, **Barbarian
   Ring** cover the last several life the weak creatures can't.

**Cards.** *Engine:* **Ghostly Prison**, **Sphere of Safety**, **Cage of Hands**, **Glory of Warfare**,
**Goblin Trenches**. *Payoffs:* **Spikeshot Elder**, **Pianna**, **Fledgling Dragon**, **Kamahl**,
**Battlewise Aven**. *Support:* **Sphere of Resistance** (colourless tax, not white), **Suppression
Field**, **Aura of Silence**, **Story Circle**, **Karmic Justice**, **Mine Layer** (a 1/1 land-tax
disruptor), **Seal of Fire**, **Barbarian Ring**, **Airlift Chaplain** (mill 3 — the only real threshold
enabler in the colours), W/R fixing (**Rugged Prairie**, **Battlefield Forge**, **Wind-Scarred Crag**).

**Overlaps.** Shares the white pillowfort and **Cage of Hands** with **WU/WB**; the go-wide token plan
rhymes with **GW** Opposition; the burn/pingers with every red deck. *Honest weakness:* the threshold
beaters want a stocked graveyard WR can barely fill, so treat threshold as a bonus, not a plan.

### UG · Simic — "Proliferate Ramp-Lock"
**Plan.** A proliferate engine bolted to a soft mana-lock and a ramp-into-blue-payoff plan. *Reality
check, and it's a big one:* the docs bill UG as "count counters to an alternate win," but **every cited
payoff is absent** — Simic Ascendancy, Darksteel Reactor, Helix Pinnacle, and Forgotten Ancient are
**not in the cube**. There is **no count-to-a-number win for Simic here**. The real deck is a slow blue
control-combo wearing a green ramp/lock shell.

**How it wins.**
1. **Tangle Wire lock → ramp into a Blue deck-out.** Land **Tangle Wire** (→U/G) early and proliferate
   each turn (**Steady Progress**, **Tezzeret's Gambit**, **Contagion Engine**, **Throne of Geth**,
   **Karn's Bastion**, **Thrummingbird**) to add a fade counter back so it never expires — a sustained
   soft mana-lock (symmetric: it taps your stuff too). Behind it, ramp with **Empowered Autogenerator**
   and **Astral Cornucopia** (charge counters proliferate upward — but these are **mana engines, not win
   counters**), cast **Mana Severance** to exile your own land cards from your library, then **Stroke of
   Genius** on yourself to draw your thinned deck into a **Laboratory Maniac** / **Jace** win.
2. **Time Stretch / Folio of Fancies mana sink.** Proliferate the mana batteries (**Contagion Engine**
   proliferates *twice* per activation) to 8+ mana, then **Time Stretch** (two extra turns), **Folio of
   Fancies** mill, or a huge **Stroke of Genius** at the opponent. The most honest UG line — no creature
   needed, just mana plus one telegraphed blue spell.
3. **Contagion Engine soft-wipe (survival, not a win).** **Contagion Engine** seeds −1/−1 on each of a
   player's creatures, then proliferates them — a genuine asymmetric sweeper against this cube's small
   creatures. **Thrummingbird** chips and proliferates. This keeps you alive to reach the real finish;
   counting it as a "win" is dishonest.

**Cards.** *Engine:* the proliferate suite above, **Experimental Augury** (top three, rest to *bottom*,
then proliferate), **Nesting Grounds** (moves one counter — the in-cube counter-mover), **Contagion
Clasp**. *Payoffs:* **Tangle Wire**, **Stroke of Genius**, **Time Stretch**, **Folio of Fancies**,
**Empowered Autogenerator / Astral Cornucopia** (ramp). *Support:* **Mana Severance**, **Laboratory
Maniac**, **Jace, Wielder of Mysteries**, **Quarry Hauler**, **Chlorophant / Cocoon** (accumulators —
note proliferating Cocoon *delays* its payoff), **Naturalize / Ancient Grudge / Dire-Strain Rampage**
(the latter two are R/G and need red; Ancient Grudge hits artifacts only), U/G fixing (**Flooded Grove**,
**Yavimaya Coast**, **Thornwood Falls** — fixing lands, not storage lands).

**Overlaps.** The proliferate suite is cross-cube glue — it re-arms **Smokestack** soot (BG), feeds
storage-land mana (RG), and could advance any charge-counter win (but none exists). The blue half is
identical to the **UB** deck-out plan: UG is essentially *UB deck-out with green ramp + proliferate
instead of black tutors*. **Tangle Wire** sits beside the other colour-shifted stax signposts (Smokestack,
Opposition, Ensnaring Bridge). A drafter expecting an Ascendancy/Reactor/Pinnacle alt-win will find an
empty slot — the win is Blue's.

---

## The colourless spine (the shared toolbox)

These are colourless and splashable into any pair — the hardware where the locks and alternate wins that
*any* colour can run actually live.

- **Stax / prison:** **Winter Orb**, **Static Orb**, **Storage Matrix**, **Tangle Wire** (→U/G),
  **Smokestack** (→B/G), **Ensnaring Bridge** (→B/R), **Sphere of Resistance**, **Sphere of Law/Duty**
  (W), **Crawlspace**, **Damping Sphere**, **Defense Grid**, **Tsabo's Web**, **Possessed Portal**,
  **Spawning Pit**.
- **Wheel-lock / forced draw:** **Teferi's Puzzle Box**, **Anvil of Bogardan**, **Howling Mine**, **Font
  of Mythos**, **Otherworld Atlas**, **Temple Bell**, **Memory Jar**, **Mind's Eye** + the black
  hand-punishers + **Phyrexian Tyranny** (→U/B).
- **Proliferate / counters:** **Astral Cornucopia**, **Throne of Geth**, **Contagion Clasp**, **Contagion
  Engine**, **Karn's Bastion**, **Nesting Grounds**, **Steady Progress** (U), **Empowered Autogenerator**.
- **Lantern mill (errata'd to clock alone):** **Lantern of Insight**, **Pyxis of Pandemonium**, and
  **Mindcrank** each mill 1/upkeep in-cube; **Codex Shredder** and **Ghoulcaller's Bell** already mill
  1/turn (Ghoulcaller's Bell is *symmetric* — it mills you too; Codex Shredder can target the opponent).
- **Life-swap:** **Mirror Universe**, **Soul Conduit**.
- **Honest-combo enablers:** **Sundial of the Infinite** (end the turn during upkeep — the Leveler safety
  valve and a Stifle for exile-on-leave creatures), **Leveler** (10/10; exiles your own library),
  **Lion's Eye Diamond**, **Krark's Thumb**, **Krark-Clan Ironworks**.
- **Big-mana sinks / ramp rocks:** **Trading Post**, **Mage-Ring Network** (storage, {C} only),
  **Astral Cornucopia**, plus the low-power fixing rocks (**Mind Stone**, **Fellwar Stone**, **Prismatic
  Lens**, **Coldsteel Heart**, **Star Compass**, **Expedition Map**).

**Lands that build around.** The mana base is a second engine layer: **storage lands** (Calciform Pools,
Dreadship Reef, Molten Slagheap, Fungal Reaches, Saltcrusted Steppe — storage counters double as
proliferate targets), **threshold painlands** (**Cephalid Coliseum** self-mill, **Barbarian Ring**,
**Cabal Pit**), **Karn's Bastion** (proliferate) and **Nesting Grounds** (move a counter), the **wheel
land** **Mikokoro**, **hellbent** **Sea Gate Wreckage**, the **manlands** (**Nantuko Monastery**), and a
full allied+enemy filter/pain/taplife fixing suite.

---

## Cross-pair connective tissue (why decks aren't on rails)

- **Proliferate** (UG) supercharges every counter, re-arms fade counters (Tangle Wire), feeds storage
  lands, advances Chance Encounter's luck counters, and grows the −1/−1 and charge-counter axes.
- **Discard outlets** feed the discard-drain (Bond of Agony, Sickening Dreams, Faith of the Devoted),
  madness, hellbent, **Ensnaring Bridge**, and the Lhurgoyf/threshold bodies — all at once.
- **Coin-flips** feed Chance Encounter, Krark's Thumb, Stitch in Time, and the Game of Chaos / Fiery
  Gambit blowouts.
- **Tokens** feed Opposition, Smokestack fodder, Goblin Trenches, and the go-wide anthems.
- **Life payment** feeds Ad Nauseam, Death's Shadow, Bond of Agony, Death Grasp, and the Mirror Universe
  swap.
- **Wheels** (Wheel of Fortune, Reforge the Soul, Day's Undoing, Memory Jar, Teferi's Puzzle Box) refuel
  hands, force the hand-punishers' damage, and empty hands for hellbent.
- **The five colour-shifted signposts** are the open draft lanes: first-pick **Opposition** → GW tokens,
  **Smokestack** → BG sacrifice, **Ensnaring Bridge** → BR hellbent, **Tangle Wire** → UG proliferate,
  **Opalescence** → UW enchantments.

---

## Fact-check & corrections

This document is grounded in the cube's **design intent** plus the live list, with Oracle text verified
against Scryfall and changes limited to [`ERRATA-AND-COLORSHIFTS.md`](./ERRATA-AND-COLORSHIFTS.md). It
records the points where an earlier draft, or the companion docs, were wrong.

**Design decisions newer than some companion docs.**
- **Conventional Storm is cut.** **Tendrils of Agony**, **Grapeshot**, and **Empty the Warrens** are
  removed; the black "build-your-own-Tendrils" is the **slow discard-drain** instead (Bond of Agony,
  Sickening Dreams, Shrieking Affliction, Gibbering Descent, Faith of the Devoted) — payoffs that scale
  with cards spent, never a turn-three kill.
- **Brain Freeze is errata'd to mill yourself only** — a self-mill enabler (feeds threshold, the
  graveyard, and the Lab Maniac self-deck-out), **not** an opponent clock.
- **Test of Endurance is cut.** The only count-to-a-number alt-win left is **Chance Encounter** (R).

**Still-true myths to avoid.**
- **Faith of the Devoted keys on DISCARD.** It triggers on "whenever you cycle **or discard** a card,"
  and cycling is itself a discard. There are **zero cycling cards** in this cube, so in-cube it is a pure
  **discard** drain ({1} → opponent loses 2, you gain 2) — no errata needed.
- **The card is "Braid of Fire" (singular)**, not "Braids of Fire."
- **"One With Nothing + Shadow of the Grave"** is only a payoff under a *proposed, unfinalised* errata
  (each opponent loses 1 life per card discarded — number unconfirmed). As printed it just discards your
  hand; do not cite it as a fixed finisher.

**Rules points the lines depend on.** Opalescence + Humility yields 1/1s only if Opalescence resolves
first (else animated enchantments are their mana value); Sundial of the Infinite must be activated *in
your upkeep* to skip the lethal draw; Day's Undoing reshuffles graveyards and makes the opponent *harder*
to deck; Mana Clash resolves on coin faces and does **not** feed Chance Encounter; only Phyrexian Unlife
or Pact Weapon (not Solitary Confinement) lets you exist at ≤0 life; Pernicious Deed spares lands and
planeswalkers; Diligent Farmhand puts its basic on the battlefield, not in the yard; Magmatic Core hits
creatures only; Lightning Storm is a charge-counter burn spell, not a Storm card (it survives the cut).

**Colour-shifts (in-cube identity, overriding print).** Opposition → **G/W**; Tangle Wire → **U/G**;
Smokestack → **B/G**; Ensnaring Bridge → **B/R**; Opalescence → **U/W**; Phyrexian Tyranny → **U/B** (red
pip dropped); Black Vise, Iron Maiden, The Rack, Ebony Owl Netsuke, Misers' Cage, Cursed Rack → **black**.

**Cited by the companion docs but NOT in the cube** (do not build around these): Simic Ascendancy,
Darksteel Reactor, Helix Pinnacle, Azor's Elocutors, Sigil of the Empty Throne, Triskaidekaphile,
Near-Death Experience, Transcendence, Comet Storm, Goblin Bomb, Forgotten Ancient, Powerful Broker,
Maulfist Revolutionary, Cabal Archon, Dark Tutelage, Tainted Remedy, Collective Brutality, Burning
Inquiry, Worn Powerstone, Wild Growth, Utopia Sprawl, Carpet of Flowers, Glittering Wish, Quicksilver
Fountain, Celestial Dawn, Trap Digger, Glacial Chasm, Geier Reach, Parallax Wave, Parallax Inhibitor,
Sunken City, Shadow of Mortality, "Shadow of the Industry" (a mis-remembered name). Where these were
load-bearing in the intent, the pair sections name the real in-cube fallback or honestly flag the gap
(most notably: **Simic/UG has no count-to-a-number alt-win**, and **RG has no single big X-spell sink**).

**Mis-coloured in the companion docs.** Lightning Storm (red, not a UB finish), Glory of Warfare (R/W,
not pure GW), Rain of Gore (B/R anti-lifegain, not a WB lifegain card), Sphere of Resistance (colourless,
not white), Stormbind / Aether Rift (R/G, not B/R), Laquatus's Champion (mono-B, and its leave-trigger
refunds the life), By Gnome Means (mono-W, not green), Feed the Swarm (mono-B, not green).
