# oddysey2026 — Creature-Support Pass

Support creatures from **outside** the `/odysseyblock` pool, one set per archetype, filling
functional slots the in-pool creatures miss. All verified in `data/mtg.sqlite` (exact cost, pip
count, type, cube_elo). None are already in the 359-card list. **Additions only — no cuts.**

Constraints honoured: single colored pip preferred (double-pip flagged, no triple); on-band
(all under ~1450 cube_elo, favouring lesser-known); mana sinks flagged **non-tap = true sink**
(usable on Braid-of-Fire upkeep mana) vs **{T} = once/turn**; native colours, no flavour-fail shifts.

> Context: the canonical 359 dropped several alt-win anchors (Simic Ascendancy, Helix Pinnacle,
> Azor's Elocutors, Near-Death Experience, Transcendence, Sigil of the Empty Throne,
> Triskaidekaphile). UG/UW lean harder on the *engine* now, so these picks weight engine bodies.

---

## WU — Enchantress Replenish Prison
Slots: enchantress **draw body**, **aura tutor body**, **enchantment-recursion body**, an
**enchantment-creature** that plays with Opalescence/Humility.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Kor Spiritdancer | {1}{W} | 2 | Creature — Kor Wizard | enchantress draw body | draws when you cast an Aura on it; cheap engine | 1 | 1248 |
| Sram, Senior Edificer | {1}{W} | 2 | Legendary Creature — Dwarf | enchantress draw body | draws on *any* Aura cast (not just on itself) | 1 | 1293 |
| Heliod's Pilgrim | {2}{W} | 3 | Creature — Human Cleric | aura tutor body | ETB tutor an Aura (Cage of Hands, the lock pieces) | 1 | 1213 |
| Monk Idealist | {2}{W} | 3 | Creature — Human Monk | recursion body | ETB return an enchantment from yard; mini-Replenish | 1 | 1136 |
| Transcendent Envoy | {1}{W} | 2 | Enchantment Creature — Griffin | enchantment-creature | a body under Starfield/Opalescence; reduces Aura costs | 1 | 1160 |

**Adds (5):** Kor Spiritdancer, Sram Senior Edificer, Heliod's Pilgrim, Monk Idealist, Transcendent Envoy

## UB — Control-Combo / Deck-Out
Slots: **evasive repeatable-draw sink**, **dig body** for the combo spells, **self-mill body**
(Cephalid/Narcomoeba/threshold fuel), redundant **mill bodies**.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Spectral Sailor | {U} | 1 | Creature — Spirit Pirate | evasive draw sink | flash flyer, **{3}{U}: draw (non-tap true sink)** | 1 | 1319 |
| Augur of Bolas | {1}{U} | 2 | Creature — Merfolk Wizard | dig body | ETB dig 3 for the instant/sorcery half of the combo | 1 | 1307 |
| Merfolk Looter | {1}{U} | 2 | Creature — Merfolk Rogue | filter/self-mill | **{T}: loop loot ({T} once/turn)**; feeds yard | 1 | 1306 |
| Wall of Lost Thoughts | {1}{U} | 2 | Creature — Wall | mill body / blocker | ETB mill 4 (self or them); stalls + fuels deck-out | 1 | 1257 |
| Hedron Crab | {U} | 1 | Creature — Crab | self-mill engine | landfall mill 3; threshold + deck-out fuel | 1 | 1221 |
| Stitcher's Supplier | {B} | 1 | Creature — Zombie | self-mill body | mill 3 on ETB + on death; on-colour fodder | 1 | 1293 |

**Adds (6):** Spectral Sailor, Augur of Bolas, Merfolk Looter, Wall of Lost Thoughts, Hedron Crab, Stitcher's Supplier

## BR — Hellbent Discard / Threshold (the slow grind, per NOTES-FOR-V2)
Slots: **repeatable discard outlets** (the gap; threshold bodies come from the pool), a
**discard payoff body**, a **discard-ritual** outlet for hellbent.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Putrid Imp | {B} | 1 | Creature — Zombie Imp | free discard outlet | **"Discard a card: gain flying" (free, repeatable)**; threshold flyer | 1 | 1277 |
| Olivia's Dragoon | {1}{B} | 2 | Creature — Vampire Berserker | free discard outlet | discard-to-fly outlet on a body; feeds madness/hellbent | 1 | 1231 |
| Bloodrage Brawler | {1}{R} | 2 | Creature — Minotaur | discard outlet beater | discard 2 on ETB → 4/3+; outlet stapled to a clock | 1 | 1226 |
| Asylum Visitor | {1}{B} | 2 | Creature — Vampire Wizard | discard payoff | draws when a player discards; has madness itself | 1 | 1272 |
| Skirge Familiar | {4}{B} | 5 | Creature — Phyrexian Imp | discard-ritual outlet | sac cards for {B}; free discard outlet + hellbent ritual | 1 | 1197 |

**Adds (5):** Putrid Imp, Olivia's Dragoon, Bloodrage Brawler, Asylum Visitor, Skirge Familiar

## BG — Smokestack Sacrifice + Attrition
Slots: **free sac outlets** (non-tap, to out-sacrifice Smokestack), **recurring fodder**,
**self-replacing fodder**, **utility sac fodder**.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Carrion Feeder | {B} | 1 | Creature — Zombie | free sac outlet | **"Sacrifice a creature" (non-tap, at will)**; grows | 1 | 1254 |
| Viscera Seer | {B} | 1 | Creature — Vampire Wizard | free sac outlet + scry | non-tap sac for selection; smooths the grind | 1 | 1259 |
| Reassembling Skeleton | {1}{B} | 2 | Creature — Skeleton | recurring fodder | **{1}{B}: return from yard** — endless Smokestack fuel | 1 | 1276 |
| Doomed Dissenter | {B} | 1 | Creature — Human | self-replacing fodder | dies → 2/2 Zombie; two sacs for one card | 1 | 1276 |
| Spore Frog | {G} | 1 | Creature — Frog | recurring sac fog | sac to fog; loops with Gravedigger/recursion | 1 | 1226 |
| Sakura-Tribe Elder | {1}{G} | 2 | Creature — Snake Shaman | sac-for-ramp fodder | block then sac → land; fodder that ramps | 1 | 1369 |

**Adds (6):** Carrion Feeder, Viscera Seer, Reassembling Skeleton, Doomed Dissenter, Spore Frog, Sakura-Tribe Elder

## RG — Big-Red Mana Sink
Slots: **non-tap mana-sink bodies** (Braid-of-Fire upkeep mana), **ramp bodies**, a
**grindy land-fetch/cycler**.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Flamewave Invoker | {2}{R} | 3 | Creature — Goblin Mutant | repeatable burn sink | **{7}{R}: 5 dmg (non-tap true sink)**; dumps surplus mana | 1 | 1121 |
| Feral Hydra | {X}{G} | 1 | Creature — Hydra | X-sink + counters | **{3}: +1/+1, instant-speed, any player (non-tap)**; UG crossover | 1 | 1165 |
| Furnace Whelp | {2}{R}{R} | 4 | Creature — Dragon | firebreathing sink | **{R}: +1/0 (non-tap true sink)** flyer; *double-pip, flag* | 2 | 1116 |
| Farhaven Elf | {2}{G} | 3 | Creature — Elf Druid | ramp body | ETB fetch a land; fixes + accelerates the engine | 1 | 1373 |
| Quirion Ranger | {G} | 1 | Creature — Elf Ranger | ramp/untap utility | free land-untap each turn (extra mana / Braid value) | 1 | 1251 |
| Krosan Tusker | {5}{G}{G} | 7 | Creature — Boar | land-fetch cycler | **cycling {2}{G}: land + card**; flood insurance; *double-pip via cycle* | 2 | 1309 |

**Adds (6):** Flamewave Invoker, Feral Hydra, Furnace Whelp, Farhaven Elf, Quirion Ranger, Krosan Tusker

## GW — Opposition Tokens
Slots: **token-maker bodies** (untapped bodies to tap with Opposition), and a **go-wide payoff**.

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Imperious Perfect | {2}{G} | 3 | Creature — Elf Warrior | token maker + anthem | **{G},{T}: 1/1 Elf ({T} once/turn)** + lord; bodies to tap | 1 | 1320 |
| Tendershoot Dryad | {4}{G} | 5 | Creature — Dryad | token engine + anthem | a Saproling every upkeep; endless Opposition fuel | 1 | 1355 |
| Thallid | {G} | 1 | Creature — Fungus | slow token maker | spore-counter Saprolings; cheap recurring bodies | 1 | 1214 |
| Tukatongue Thallid | {G} | 1 | Creature — Fungus | fodder + death token | one-drop body that leaves a body | 1 | 1154 |
| Champion of Lambholt | {1}{G}{G} | 3 | Creature — Human Warrior | go-wide payoff | a wide board makes your team unblockable; *double-pip, flag* | 2 | 1294 |

**Adds (5):** Imperious Perfect, Tendershoot Dryad, Thallid, Tukatongue Thallid, Champion of Lambholt

## WB — Life as a Resource
Slots: **death-drain bodies** (life swing), **lifegain→drain converters**, **burst-life** combo
enabler (Mirror Universe / pay-life setup).

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Blood Artist | {1}{B} | 2 | Creature — Vampire | death-drain engine | drain 1 on *any* death; turns the board into life-swing | 1 | 1330 |
| Zulaport Cutthroat | {1}{B} | 2 | Creature — Human Rogue | death-drain engine | your-deaths version; redundant with Blood Artist | 1 | 1287 |
| Marauding Blight-Priest | {2}{B} | 3 | Creature — Vampire Cleric | lifegain→drain | each lifegain drains them 1; pairs the lifegain suite | 1 | 1183 |
| Epicure of Blood | {4}{B} | 5 | Creature — Vampire | lifegain→drain (big) | same, scaled; closes via clerics' incidental gain | 1 | 1133 |
| Children of Korlis | {W} | 1 | Creature — Human Cleric | burst-life enabler | regain life spent this turn; sets up Mirror Universe / Ad Naus | 1 | 1175 |
| Bloodthirsty Aerialist | {1}{B}{B} | 3 | Creature — Vampire Rogue | lifegain payoff clock | grows + flies on lifegain; *double-pip, flag* | 2 | 1179 |

**Adds (6):** Blood Artist, Zulaport Cutthroat, Marauding Blight-Priest, Epicure of Blood, Children of Korlis, Bloodthirsty Aerialist

## UR — Spells & Coin-Flips
Slots: **spell→token clocks** (the cube lacks bodies that turn spell velocity into a board) and
**spell-count evasive clocks**. (Azure Mage already fills the draw mana-sink slot.)

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Young Pyromancer | {1}{R} | 2 | Creature — Human Shaman | spell→token clock | 1/1 per instant/sorcery; *top-of-band, flag* | 1 | 1395 |
| Murmuring Mystic | {3}{U} | 4 | Creature — Human Wizard | spell→token wall/clock | 1/1 flyer per instant/sorcery; *top-of-band, flag* | 1 | 1410 |
| Talrand, Sky Summoner | {2}{U}{U} | 4 | Legendary Creature — Merfolk | spell→token clock | 2/2 Drake per instant/sorcery; *double-pip, flag* | 2 | 1322 |
| Enigma Drake | {1}{U}{R} | 3 | Creature — Drake | spell-count clock | flyer, power = instants/sorceries in yard; *double-pip* | 2 | 1197 |
| Nivix Cyclops | {1}{U}{R} | 3 | Creature — Cyclops | prowess-style clock | huge swing the turn you chain spells; *double-pip* | 2 | 1162 |
| Niblis of Frost | {2}{U}{U} | 4 | Creature — Spirit | prowess flyer + tapper | taps a blocker per spell; *double-pip, flag* | 2 | 1266 |

**Adds (6):** Young Pyromancer, Murmuring Mystic, Talrand Sky Summoner, Enigma Drake, Nivix Cyclops, Niblis of Frost

## WR — Tax Aggro
Slots: **tax-on-a-body** (Sphere/Thalia effects stapled to a clock), and **mana-sink one-drops**
(aggro that uses surplus mana).

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Vryn Wingmare | {2}{W} | 3 | Creature — Pegasus | tax body (flyer) | noncreature spells cost {1} more, on an evasive clock | 1 | 1203 |
| Leonin Arbiter | {1}{W} | 2 | Creature — Cat Cleric | tax body (search) | taxes fetch/tutor; hoses the combo decks | 1 | 1255 |
| Glowrider | {2}{W} | 3 | Creature — Human Cleric | tax body (Sphere) | Sphere of Resistance on a 2/1; redundant tax | 1 | 1190 |
| Figure of Destiny | {R/W} | 1 | Creature — Kithkin | mana-sink one-drop | **non-tap level-up sink**; castable off one colour | (hybrid) | 1337 |
| Student of Warfare | {W} | 1 | Creature — Human Knight | mana-sink one-drop | level-up aggro; pours surplus mana into a clock | 1 | 1285 |
| Adanto Vanguard | {1}{W} | 2 | Creature — Vampire Soldier | resilient clock | indestructible attacker; grinds through removal/blocks | 1 | 1255 |

**Adds (6):** Vryn Wingmare, Leonin Arbiter, Glowrider, Figure of Destiny, Student of Warfare, Adanto Vanguard

## UG — Proliferate & Fading Counters
Slots: **proliferate-on-a-body**, a **proliferate mana-sink**, a **counter-producer/fetcher**,
a **counter-mover** — engine bodies (alt-win counters now live on artifacts/Darksteel Reactor).

| Name | Cost | MV | Type | Slot | Why | pips | cube_elo |
|---|---|---|---|---|---|---|---|
| Thrummingbird | {1}{U} | 2 | Creature — Bird | proliferate clock | evasive: proliferate on combat damage each turn | 1 | 1236 |
| Viral Drake | {3}{U} | 4 | Creature — Drake | proliferate mana-sink | **{3}{U}: proliferate (non-tap true sink)** flyer | 1 | 1175 |
| Evolution Sage | {2}{G} | 3 | Creature — Elf Druid | proliferate engine | proliferate on every landfall; ramps the counters | 1 | 1275 |
| Fertilid | {2}{G} | 3 | Creature — Elemental | counter-producer/ramp | counters that fetch lands; a proliferate target itself | 1 | 1303 |
| Pollenbright Druid | {1}{G} | 2 | Creature — Elf Druid | counter seeder | ETB put a +1/+1 counter; cheap proliferate enabler | 1 | 1219 |
| Plaxcaster Frogling | {1}{G}{U} | 3 | Creature — Frog Mutant | counter-mover sink | **{2}: move a +1/+1 counter (non-tap)** + shroud; *double-pip* | 2 | 1132 |

**Adds (6):** Thrummingbird, Viral Drake, Evolution Sage, Fertilid, Pollenbright Druid, Plaxcaster Frogling

---

## Flat add-list (57 creatures)

```
1 Kor Spiritdancer
1 Sram, Senior Edificer
1 Heliod's Pilgrim
1 Monk Idealist
1 Transcendent Envoy
1 Spectral Sailor
1 Augur of Bolas
1 Merfolk Looter
1 Wall of Lost Thoughts
1 Hedron Crab
1 Stitcher's Supplier
1 Putrid Imp
1 Olivia's Dragoon
1 Bloodrage Brawler
1 Asylum Visitor
1 Skirge Familiar
1 Carrion Feeder
1 Viscera Seer
1 Reassembling Skeleton
1 Doomed Dissenter
1 Spore Frog
1 Sakura-Tribe Elder
1 Flamewave Invoker
1 Feral Hydra
1 Furnace Whelp
1 Farhaven Elf
1 Quirion Ranger
1 Krosan Tusker
1 Imperious Perfect
1 Tendershoot Dryad
1 Thallid
1 Tukatongue Thallid
1 Champion of Lambholt
1 Blood Artist
1 Zulaport Cutthroat
1 Marauding Blight-Priest
1 Epicure of Blood
1 Children of Korlis
1 Bloodthirsty Aerialist
1 Young Pyromancer
1 Murmuring Mystic
1 Talrand, Sky Summoner
1 Enigma Drake
1 Nivix Cyclops
1 Niblis of Frost
1 Vryn Wingmare
1 Leonin Arbiter
1 Glowrider
1 Figure of Destiny
1 Student of Warfare
1 Adanto Vanguard
1 Thrummingbird
1 Viral Drake
1 Evolution Sage
1 Fertilid
1 Pollenbright Druid
1 Plaxcaster Frogling
```

**Total creatures added: 57** (≤60). WU 5 · UB 6 · BR 5 · BG 6 · RG 6 · GW 5 · WB 6 · UR 6 · WR 6 · UG 6.

### Notes / flags
- **Pip purity:** all single-pip except 8 deliberate double-pips, each flagged in-table
  (Furnace Whelp, Champion of Lambholt, Bloodthirsty Aerialist, Talrand, Enigma Drake, Nivix
  Cyclops, Niblis of Frost, Plaxcaster Frogling). UR is the pip-heaviest pair by nature; Figure of
  Destiny is hybrid (castable off one colour).
- **True mana sinks (non-tap, Braid-of-Fire-able):** Spectral Sailor, Flamewave Invoker, Feral
  Hydra, Furnace Whelp, Viral Drake, Figure of Destiny, Student of Warfare, Plaxcaster Frogling.
  **{T} once/turn:** Merfolk Looter, Imperious Perfect.
- **Top-of-band watch (>~1390):** Young Pyromancer (1395), Murmuring Mystic (1410). Kept because
  the cube has *no* in-pool body that converts spell velocity to a board — they are the UR payoff.
  Cut first if power creeps.
