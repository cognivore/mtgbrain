# Worksheet B — Mono-Black (oddysey2026)

**Target ~52 · Built 53 (23 creatures + 30 non-creatures) · SINGLETON · very low power.**

Black is the deepest colour: it carries the cube's *graveyard-storm*, *hellbent / empty-hand*,
*life-as-resource*, *discard*, and *aristocrats sac* engines, plus the colour-shifted **puzzle-box
hand-punisher** package. Everything here is slow build-around or honest spell-stapled reach — no
board-camping value engines (those are flagged `too_strong` and parked on the maybeboard).

## Hard-rule reconciliation (creatures)
The curated `pool/odysseyblock_creatures.tsv` contains **only 23 mono-black creatures** — a
hand-picked Odyssey-block subset, *not* the full block. Several creatures the worksheet brief names
(Rotlung Reanimator, Cabal Archon, Cabal Torturer, Putrid Imp, Zombie Cannibal, Consumptive Goo) are
**Onslaught-block** and are **absent from the curated pool** (verified: 0 hits in the TSV; they only
appear in `master_candidates.tsv` tagged `ONS-block`/`ODY-block`). Per the overriding HARD RULE +
ASSUMPTIONS correction #1 ("creatures come ONLY from the curated /odysseyblock cube"), and since the
only sanctioned creature exceptions are Narcomoeba / Lab Maniac / Death's Shadow, **this worksheet
takes 0 creature exceptions** and uses all 23 available mono-B pool creatures. The wished-for
Onslaught creatures are routed to the maybeboard with the reason "not in curated pool."

Net effect: the creature count lands at 23 (pool-capped, not 26); non-creatures carry the extra
slots so the worksheet still totals ~52.

## The 23 creatures (all from the pool)
On-theme low-power bodies that break stalls late:
- **Threshold beaters** (vanilla-until-GY-online): Boneshard Slasher, Childhood Horror,
  Treacherous Werewolf, Treacherous Vampire, Masked Gorgon, Repentant Vampire, Gloomdrifter (a
  threshold -2/-2 sweep flyer).
- **GY-fuel -X/-X removal on a stick** (decision-dense, self-mill payoff): Painbringer, Cabal
  Patriarch, Cabal Trainee (free self-sac -2/-0 trick).
- **Mass-of-graveyard scalers**: Mortivore, Sutured Ghoul.
- **Sac fodder / sac outlets / aristocrats glue**: Cabal Trainee, Crypt Creeper, Carrion Rats,
  Carrion Wurm (GY-hate attackers), Cabal Surgeon (rebuy fodder).
- **ETB disruption / removal**: Mesmeric Fiend (hand exile), Faceless Butcher (creature exile),
  Laquatus's Champion (honest 6-life drain stapled to a body), Gravedigger (recursion).
- **Mana-sink threats**: Nantuko Shade, Organ Grinder (GY-storm drain finisher).
- **Symmetric stax body**: Braids, Cabal Minion (feeds BG Smokestack / sac).

## The 30 non-creatures
- **Rituals (combo/storm fuel):** Dark Ritual, Cabal Ritual, Songs of the Damned, Culling the Weak.
- **Discard:** Raven's Crime (retrace, recurring), Chain of Smog (copyable), Collective Brutality
  (modal escalate), Sickening Dreams (discard-X sweeper + drain).
- **Hellbent / empty-hand payoffs:** Nihilistic Glee, Shrieking Affliction, Quest for the Nihil Stone.
- **Wheels / storm kills:** Ill-Gotten Gains (wheel + GY recursion), Tendrils of Agony (storm drain).
- **Tutors (build the combo turn):** Infernal Tutor (hellbent), Tainted Pact (library-thinning),
  Demonic Consultation (dig + self-mill toward deck-out), Spoils of the Vault (life-cost dig),
  Doomsday (alt-win deck-stack, honest one-shot).
- **Life-as-resource:** Greed, Dark Tutelage, Bond of Agony (X-life burn), Font of Agonies (pay-life
  removal payoff).
- **Graveyard-storm enabler pair:** One With Nothing + Shadow of the Grave (discard hand → return
  it → card-neutral storm fuel).
- **Puzzle-box hand punishers — colour-shifted to B** (the whole package lives here): Black Vise,
  Iron Maiden, The Rack, Ebony Owl Netsuke, Misers' Cage, Cursed Rack (the hand-size lock).

## Key combos & lines
- **BR / mono-B graveyard storm:** Putrid Imp/discard outlets fill the yard → Songs of the Damned /
  Cabal Ritual go big → Tendrils of Agony or Organ Grinder for reach; One With Nothing + Shadow of
  the Grave loops the hand for storm count.
- **Hellbent shell:** dump the hand (One With Nothing, discard outlets) → Infernal Tutor finds the
  kill, Nihilistic Glee draws, Shrieking Affliction / Quest for the Nihil Stone bleed the *opponent*
  for keeping a full hand — both halves of the empty-hand axis covered.
- **The hand-size lock (alt-win clock):** Cursed Rack caps an opponent at 4 cards; Black Vise / Iron
  Maiden / The Rack / Ebony Owl / Misers' Cage punish either side of that line — an honest prison
  clock behind Ensnaring Bridge (BR).
- **Life-as-resource (WB):** Greed / Dark Tutelage draw on life; Bond of Agony / Font of Agonies
  spend it; feeds Death's Shadow / Mirror Universe in the WB worksheet.
- **BG sac:** Braids + Cabal Trainee/Patriarch sac engine; Cabal Surgeon rebuys fodder.

## Errata flags
- **One With Nothing** — `dead=true` + `errata=true`. Literal "discard your hand" does nothing on its
  own. **Errata proposal:** *"As an additional effect, each opponent loses 1 life for each card you
  discarded this way."* Turns the enabler into a real graveyard-storm payoff while staying low-power.

## Notable cuts → maybeboard (nothing discarded)
- **Onslaught creatures not in the curated pool:** Rotlung Reanimator, Cabal Archon, Cabal Torturer,
  Putrid Imp, Zombie Cannibal, Consumptive Goo — all wanted by the brief but pool-illegal here.
- **`too_strong` (board-camping value engines):** Yawgmoth's Will (one-shot GY replay, judge-power),
  Necropotence (camping card-advantage engine), Phyrexian Arena (passive value engine). All to
  maybeboard per HARD RULE #16.
- **Belongs to a dedicated/other worksheet:** Ad Nauseam, Dark Petition (UB Ad-Nauseam combo core).
- **Marginal / needs a dedicated shell:** Bitter Ordeal (gravestorm needs a sac engine), Lich (4×B,
  too fragile/narrow), Dark Prophecy / Waste Not (aristocrats & wheel payoffs better in BG/BR).
