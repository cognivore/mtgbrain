# oddysey2026 — Creature-Support Pass (v2)

Support creatures from **outside** the `/odysseyblock` pool. Every slot is derived from *this
cube's* actual engine — the named keystones in the 359, not generic archetype roles. The job of
each creature is to **enable a specific gimmick the in-pool creatures can't reach**.

All verified in `data/mtg.sqlite` (cost, pip count, type, cube_elo). None already in the 359.
**Additions only.** Single pip preferred; double-pip flagged; no triple-pip. Mana sinks flagged
**non-tap (true sink, Braid-of-Fire-able)** vs **{T}**.

> v1 of this doc fell back on generic cube slots (spells-matter clocks, aristocrat drain,
> hatebears). This rebuild anchors to the engine cards: Opalescence/Sphere-count, deck-out,
> discard-*payoffs*, Smokestack symmetric-sac, Braid upkeep mana, Opposition bodies, **go-low +
> Mirror Universe**, **coin-flips**, prison-aggro, and proliferate + **fading** re-arm.

---

## WU — Enchantress Replenish Prison
**Engine cards:** Opalescence, Humility, Starfield of Nyx, Sphere of Safety, Replenish, Sterling
Grove, Enlightened Tutor. **Real need:** not card advantage — **enchantment *bodies*** (count for
Sphere of Safety; survive as real creatures under Starfield; stay enchantments under Opalescence;
become harmless 1/1s under Humility like everything else), plus **lock-piece tutoring/recursion.**

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Hopeful Eidolon | {W} | 1 | Enchantment Creature | enchantment body | counts for Sphere of Safety; bestow = aura *or* body; real under Starfield | 1 | 1220 |
| Nyxborn Shieldmate | {W} | 1 | Enchantment Creature | enchantment body/blocker | cheap enchantment-count + a wall that survives Humility | 1 | 1171 |
| Transcendent Envoy | {1}{W} | 2 | Enchantment Creature | enchantment body | enchantment-count flyer; discounts your Auras | 1 | 1160 |
| Heliod's Pilgrim | {2}{W} | 3 | Creature — Cleric | lock tutor | fetches Cage of Hands / Story Circle / the Aura lock pieces | 1 | 1213 |
| Monk Idealist | {2}{W} | 3 | Creature — Monk | recursion | mini-Replenish: rebuy an enchantment from yard | 1 | 1136 |
| Celestial Ancient | {3}{W}{W} | 5 | Creature — Elemental | enchant-cast payoff | each enchantment cast pumps the team; *double-pip, flag* | 2 | 1157 |

**Adds (6):** Hopeful Eidolon, Nyxborn Shieldmate, Transcendent Envoy, Heliod's Pilgrim, Monk Idealist, Celestial Ancient

## UB — Control-Combo / Deck-Out
**Engine cards:** Cephalid self-mill, Narcomoeba, Laboratory Maniac / Jace WoM (win from empty
library), Mana Severance, Sphinx's Tutelage / Glimpse the Unthinkable (mill *them*). **Real need:**
**self-mill that pays off** (threshold/Narcomoeba/yard), **mill-*them*-out bodies**, and
**graveyard-active** value — not generic card filtering.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Stitcher's Supplier | {B} | 1 | Creature — Zombie | self-mill fuel | mill 3 ETB + on death; feeds threshold / Narcomoeba / yard | 1 | 1293 |
| Hedron Crab | {U} | 1 | Creature — Crab | self-mill *or* mill-them | landfall mill 3 either player; fuels Lab Man **and** Tutelage plan | 1 | 1221 |
| Manic Scribe | {1}{U} | 2 | Creature — Wizard | mill-them + tax | ETB mill 3; Delirium clock on their upkeep; deck-out support | 1 | 1224 |
| Jace's Phantasm | {U} | 1 | Creature — Illusion | mill-them payoff | 5/5 flyer once they have 10+ in yard; a *clock* on the mill plan | 1 | 1182 |
| Wonder | {3}{U} | 4 | Creature — Incarnation | yard-active evasion | from the yard, your weak creatures all gain flying; rewards self-mill | 1 | 1285 |
| Wall of Lost Thoughts | {1}{U} | 2 | Creature — Wall | mill body / blocker | ETB mill 4 (self or them); stalls + fuels both mill plans | 1 | 1257 |

**Adds (6):** Stitcher's Supplier, Hedron Crab, Manic Scribe, Jace's Phantasm, Wonder, Wall of Lost Thoughts

## BR — Hellbent Discard / Threshold (the grind — per NOTES-FOR-V2)
**Engine cards:** Faith of the Devoted, the rituals, Ensnaring Bridge, threshold beaters.
**Real need:** the missing half is **discard *payoffs*** (the outlets exist in-pool). Threshold
*bodies* come from the pool; outside-pool fills the "when you discard, X happens" engine.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Bone Miser | {4}{B} | 5 | Creature — Zombie | discard payoff engine | discard a creature→Zombie, land→{B}{B}, other→draw; the payoff | 1 | 1257 |
| Glint-Horn Buccaneer | {1}{R}{R} | 3 | Creature — Pirate | discard payoff + outlet | each discard pings them; loots; clock *and* enabler; *double-pip* | 2 | 1256 |
| Asylum Visitor | {1}{B} | 2 | Creature — Wizard | discard payoff | draws when anyone discards; madness itself | 1 | 1272 |
| Putrid Imp | {B} | 1 | Creature — Imp | free outlet + threshold | "Discard a card: gain flying" — free repeatable outlet; threshold flyer | 1 | 1277 |
| Olivia's Dragoon | {1}{B} | 2 | Creature — Berserker | free outlet | discard-to-fly outlet on a body; fuels hellbent/madness | 1 | 1231 |
| Bloodrage Brawler | {1}{R} | 2 | Creature — Minotaur | outlet beater | discard 2 → 4/3; outlet stapled to a threshold clock | 1 | 1226 |

**Adds (6):** Bone Miser, Glint-Horn Buccaneer, Asylum Visitor, Putrid Imp, Olivia's Dragoon, Bloodrage Brawler

## BG — Smokestack Symmetric Sacrifice
**Engine cards:** Smokestack, Braids Cabal Minion, Pernicious Deed, Squandered Resources.
**Real need:** **win by losing less** — bodies that **come back** or **leave a body** when the
symmetric tax saces them, plus token fodder. *Not* a Blood Artist drain pile.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Reassembling Skeleton | {1}{B} | 2 | Creature — Skeleton | recurring fodder | **{1}{B}: return from yard** — feeds Smokestack every upkeep | 1 | 1276 |
| Gravecrawler | {B} | 1 | Creature — Zombie | recurring fodder | recast from yard with a Zombie; near-free sac chaff | 1 | 1337 |
| Young Wolf | {G} | 1 | Creature — Wolf | undying fodder | sacked → comes back bigger; you lose *less* than they do | 1 | 1196 |
| Butcher Ghoul | {1}{B} | 2 | Creature — Zombie | undying fodder | same, on-colour; survives Deed/Smokestack once for free | 1 | 1229 |
| Safehold Elite | {1}{G/W} | 2 | Creature — Elf | persist fodder | persist body that shrugs the symmetric sac; castable off {G} | (hybrid) | 1305 |
| Utopia Mycon | {G} | 1 | Creature — Fungus | token fodder + mana | Saproling each cycle as sac chaff; sac it for mana too | 1 | 1240 |

**Adds (6):** Reassembling Skeleton, Gravecrawler, Young Wolf, Butcher Ghoul, Safehold Elite, Utopia Mycon

## RG — Big-Red Mana Sink (Braid of Fire)
**Engine cards:** Braid of Fire (R each upkeep), storage lands, Mage-Ring Network, Comet Storm,
Magmatic Core. **Real need:** **non-tap, instant-speed mana sinks** that eat the upkeep mana —
firebreathers and X-pumps usable any time. Not ramp dorks.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Flamewave Invoker | {2}{R} | 3 | Creature — Goblin | repeatable burn sink | **{7}{R}: 5 dmg (non-tap)** — dumps surplus upkeep mana | 1 | 1121 |
| Feral Hydra | {X}{G} | 1 | Creature — Hydra | X-sink (instant) | **{3}: +1/+1 any time (non-tap)** — pure instant-speed sink | 1 | 1165 |
| Hungering Hydra | {X}{G} | 1 | Creature — Hydra | X mana sink | dump mana on ETB; grows when damaged; can't be ganged | 1 | 1279 |
| Furnace Whelp | {2}{R}{R} | 4 | Creature — Dragon | firebreathing sink | **{R}: +1/0 (non-tap)** flyer; *double-pip, flag* | 2 | 1116 |
| Apocalypse Hydra | {X}{R}{G} | 2 | Creature — Hydra | X-sink + burn sink | huge X body **and {1}{R}: ping (non-tap)**; *double-pip, flag* | 2 | 1156 |
| Flameblast Dragon | {4}{R}{R} | 6 | Creature — Dragon | attack burn sink | **{X}{R} on attack → X dmg**; mana-into-reach; *double-pip, flag* | 2 | 1214 |

**Adds (6):** Flamewave Invoker, Feral Hydra, Hungering Hydra, Furnace Whelp, Apocalypse Hydra, Flameblast Dragon

## GW — Opposition Tokens
**Engine cards:** Opposition (tap their lands/blockers with *your untapped* creatures), Glory of
Warfare, the Squirrel/Saproling makers. **Real need:** **maximum untapped bodies** to fuel the
tap-down, plus a go-wide closer. Token *quantity* is the gimmick.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Imperious Perfect | {2}{G} | 3 | Creature — Elf | token maker + anthem | **{G},{T}: 1/1 ({T})** + lord; steady bodies to tap with Opposition | 1 | 1320 |
| Tendershoot Dryad | {4}{G} | 5 | Creature — Dryad | token engine + anthem | a Saproling *every upkeep* — endless Opposition fuel | 1 | 1355 |
| Yavimaya Sapherd | {2}{G} | 3 | Creature — Fungus | token body | body + Saproling in one card; two things to tap | 1 | 1201 |
| Jade Mage | {1}{G} | 2 | Creature — Shaman | token mana-sink | **{2}{G}: Saproling (non-tap)** — surplus mana → more bodies | 1 | 1207 |
| Tukatongue Thallid | {G} | 1 | Creature — Fungus | cheap body + token | one-drop that leaves a body; early Opposition fuel | 1 | 1154 |
| Champion of Lambholt | {1}{G}{G} | 3 | Creature — Warrior | go-wide closer | a tapped-down opponent + wide board = unblockable kill; *double-pip* | 2 | 1294 |

**Adds (6):** Imperious Perfect, Tendershoot Dryad, Yavimaya Sapherd, Jade Mage, Tukatongue Thallid, Champion of Lambholt

## WB — Life as a Resource (go LOW, then swap)
**Engine cards:** **Mirror Universe / Soul Conduit** (swap totals), **Death's Shadow**, Bond of
Agony, Greed, **Rain of Gore / Tainted Remedy** (lifegain → loss). **Real need:** bodies that
**spend life** and **thrive while low** — *never* lifegain (Rain of Gore would punish your own).
This is the section v1 got backwards.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Wall of Blood | {2}{B} | 3 | Creature — Wall | pay-life finisher | **Pay 1 life: +1/+1** — dump your life into a swing, then Mirror-swap | 1 | 1193 |
| Dark Confidant | {1}{B} | 2 | Creature — Wizard | pay-life draw | card advantage that *pushes you low*; *top-of-band, flag* | 1 | 1370 |
| Blood Scrivener | {1}{B} | 2 | Creature — Zombie | pay-life draw | empty-hand: draw 2, lose 1 — hellbent + go-low | 1 | 1229 |
| Scourge of the Skyclaves | {1}{B} | 2 | Creature — Demon | low-life threat | grows as life drops; kicker halves both totals to set up the swap | 1 | 1231 |
| Carnophage | {B} | 1 | Creature — Zombie | go-low aggro | 2/2 for {B} that bleeds you each upkeep; intentional life-spend | 1 | 1179 |
| Flesh Reaver | {1}{B} | 2 | Creature — Horror | go-low beater | 4/4 for 2 that damages *you* — fast clock that drives you to Shadow range | 1 | 1114 |

**Adds (6):** Wall of Blood, Dark Confidant, Blood Scrivener, Scourge of the Skyclaves, Carnophage, Flesh Reaver

## UR — Spells & **Coin-Flips**
**Engine cards:** **Krark's Thumb** (re-flip), Stitch in Time, Fiery Gambit, Game of Chaos,
Goblin Festival, Chance Encounter, Molten Birth, Impulsive Maneuvers. **Real need:** **coin-flip
*creatures*** — with Krark's Thumb in the cube these go from gimmick to engine. Not a third
spells-matter token-maker.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Krark, the Thumbless | {1}{R} | 2 | Legendary — Wizard | flip-payoff (spells) | flip to **copy/return** each spell; absurd with Krark's Thumb | 1 | 1200 |
| Zndrsplt, Eye of Wisdom | {4}{U} | 5 | Legendary — Homunculus | flip → card draw | win a flip → draw; the coin-flip card engine | 1 | 1200 |
| Okaun, Eye of Chaos | {4}{R} | 5 | Legendary — Cyclops | flip → clock | win a flip → doubles + huge swing; pairs Zndrsplt | 1 | 1200 |
| Goblin Archaeologist | {1}{R} | 2 | Creature — Goblin | flip utility | flip to blow up an artifact + untap; cheap flipper | 1 | 1133 |
| Frenetic Efreet | {1}{U}{R} | 3 | Creature — Efreet | flip protection | {0}: flip to phase out (dodge removal/combat); *double-pip* | 2 | 1121 |
| Karplusan Minotaur | {2}{R}{R} | 4 | Creature — Minotaur | flip burn clock | cumulative-upkeep flip → 2 dmg; coin-flip beater; *double-pip* | 2 | 1193 |

**Adds (6):** Krark the Thumbless, Zndrsplt Eye of Wisdom, Okaun Eye of Chaos, Goblin Archaeologist, Frenetic Efreet, Karplusan Minotaur

## WR — Tax / Prison Aggro
**Engine cards:** Cage of Hands, Ghostly Prison, Sphere of Safety, Suppression Field, Sphere of
Resistance, Goblin Trenches. **Real need:** **tax-on-a-body** + **mana-sink clocks** that race
behind your own pillowfort, and a way to **rebuy the Aura lock**.

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Vryn Wingmare | {2}{W} | 3 | Creature — Pegasus | tax body (flyer) | noncreature spells cost {1} more, on an evasive clock | 1 | 1203 |
| Leonin Arbiter | {1}{W} | 2 | Creature — Cat | tax body (search) | taxes tutors/fetches; hoses the combo decks | 1 | 1255 |
| Glowrider | {2}{W} | 3 | Creature — Cleric | tax body (Sphere) | a second Sphere of Resistance on a 2/1 | 1 | 1190 |
| Kor Skyfisher | {1}{W} | 2 | Creature — Soldier | lock rebuy | bounce **Cage of Hands / Seal** to re-trigger the tax-lock; evasive | 1 | 1298 |
| Figure of Destiny | {R/W} | 1 | Creature — Kithkin | mana-sink clock | **non-tap level-up sink**; castable off one colour | (hybrid) | 1337 |
| Student of Warfare | {W} | 1 | Creature — Knight | mana-sink clock | level-up one-drop; pours surplus mana into the race | 1 | 1285 |

**Adds (6):** Vryn Wingmare, Leonin Arbiter, Glowrider, Kor Skyfisher, Figure of Destiny, Student of Warfare

## UG — Proliferate & **Fading** Counters
**Engine cards:** Steady Progress, Contagion Clasp/Engine, Throne of Geth, Karn's Bastion,
Darksteel Reactor, **Tangle Wire** (fade soft-lock you re-arm with proliferate). **Real need:**
**proliferate bodies**, **fading/vanishing creatures** that proliferate keeps alive, and a
**counter payoff** (Simic Ascendancy/Helix were cut — payoff must live on a body now).

| Name | Cost | MV | Type | Slot | Why (engine) | pips | elo |
|---|---|---|---|---|---|---|---|
| Thrummingbird | {1}{U} | 2 | Creature — Bird | proliferate clock | evasive: proliferate on combat damage — re-arms Tangle Wire too | 1 | 1236 |
| Viral Drake | {3}{U} | 4 | Creature — Drake | proliferate sink | **{3}{U}: proliferate (non-tap)** flyer; repeatable | 1 | 1175 |
| Blastoderm | {2}{G}{G} | 4 | Creature — Beast | fading payoff | fading 3 + shroud → **proliferate makes it permanent**; *double-pip* | 2 | 1243 |
| Chronozoa | {3}{U} | 4 | Creature — Illusion | vanishing payoff | proliferate stalls its clock; dies → two more; fade-counter synergy | 1 | 1210 |
| Sage of Hours | {1}{U} | 2 | Creature — Wizard | counter alt-win body | remove counters → **extra turns**; the counter payoff the cut alt-wins left open | 1 | 1241 |
| Spike Feeder | {1}{G}{G} | 3 | Creature — Spike | counter-mover | seeds + moves +1/+1 counters; a proliferate target; *double-pip* | 2 | 1242 |

**Adds (6):** Thrummingbird, Viral Drake, Blastoderm, Chronozoa, Sage of Hours, Spike Feeder

---

## Flat add-list (60 creatures)

```
1 Hopeful Eidolon
1 Nyxborn Shieldmate
1 Transcendent Envoy
1 Heliod's Pilgrim
1 Monk Idealist
1 Celestial Ancient
1 Stitcher's Supplier
1 Hedron Crab
1 Manic Scribe
1 Jace's Phantasm
1 Wonder
1 Wall of Lost Thoughts
1 Bone Miser
1 Glint-Horn Buccaneer
1 Asylum Visitor
1 Putrid Imp
1 Olivia's Dragoon
1 Bloodrage Brawler
1 Reassembling Skeleton
1 Gravecrawler
1 Young Wolf
1 Butcher Ghoul
1 Safehold Elite
1 Utopia Mycon
1 Flamewave Invoker
1 Feral Hydra
1 Hungering Hydra
1 Furnace Whelp
1 Apocalypse Hydra
1 Flameblast Dragon
1 Imperious Perfect
1 Tendershoot Dryad
1 Yavimaya Sapherd
1 Jade Mage
1 Tukatongue Thallid
1 Champion of Lambholt
1 Wall of Blood
1 Dark Confidant
1 Blood Scrivener
1 Scourge of the Skyclaves
1 Carnophage
1 Flesh Reaver
1 Krark, the Thumbless
1 Zndrsplt, Eye of Wisdom
1 Okaun, Eye of Chaos
1 Goblin Archaeologist
1 Frenetic Efreet
1 Karplusan Minotaur
1 Vryn Wingmare
1 Leonin Arbiter
1 Glowrider
1 Kor Skyfisher
1 Figure of Destiny
1 Student of Warfare
1 Thrummingbird
1 Viral Drake
1 Blastoderm
1 Chronozoa
1 Sage of Hours
1 Spike Feeder
```

**Total creatures added: 60** (= cap). 6 per pair.

### Flags
- **Double-pip (flagged, deliberate):** Celestial Ancient, Glint-Horn Buccaneer, Furnace Whelp,
  Apocalypse Hydra, Flameblast Dragon, Champion of Lambholt, Frenetic Efreet, Karplusan Minotaur,
  Blastoderm, Spike Feeder. No triple-pip (Sigil Captain and Mijae Djinn were cut for {G}{W}{W} /
  {R}{R}{R}). Hybrids castable off one colour: Safehold Elite ({1}{G/W}), Figure of Destiny ({R/W}).
- **True non-tap mana sinks (Braid-of-Fire-able):** Flamewave Invoker, Feral Hydra, Furnace Whelp,
  Apocalypse Hydra, Viral Drake, Jade Mage, Figure of Destiny, Student of Warfare, Wall of Blood.
- **Top-of-band watch (>~1340):** Gravecrawler (1337), Tendershoot Dryad (1355), Dark Confidant
  (1370). Each kept for an engine the cube has no in-pool answer to (free recurring fodder /
  per-turn token engine / pay-life-draw-and-go-low). Cut first if power creeps.
- **Singleton-safe:** every card appears once; no cross-pair collisions.
