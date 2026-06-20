# oddysey2026 — Counter Cards Discovery

Every counter-related card in the cube pool (Odyssey block + Onslaught/Legions/Scourge gap-fillers), **explained verbatim**: the oracle text is quoted exactly as pulled from MTGJSON. Pure counter*spells* (Counter target spell…) are excluded — this is about counters-on-permanents. Each entry: name · mana cost · type · the verbatim rules text · a short design note.

> Sets shown in `[brackets]` after the type are the printings on record; the in-pool ones are ODY/TOR/JUD (Odyssey block) and ONS/LGN/SCG (Onslaught block).


## 1. Counter-based win conditions (the spine — 'errata'd stronger' candidates)

The reason the cube exists. Chance Encounter is the one true counter win-con printed in the block; the rest are out-of-block imports listed as reskins/errata fuel for *additional* win conditions. All 'count to a number' and reward the cube's counter-manipulation.


### Chance Encounter — {2}{R}{R}
*Enchantment*  `[ODY]`

> Whenever you win a coin flip, put a luck counter on this enchantment.
> At the beginning of your upkeep, if this enchantment has ten or more luck counters on it, you win the game.

**[ODY · in-block]** The cube's anchor red win-con. Buff ideas if you errata: lower the threshold (e.g. 7), also gain a luck counter when you *lose* a flip, or let it be proliferated. Already decision-rich (when to force flips, what to flip with).

### Goblin Bomb — {1}{R}
*Enchantment*  `[out of block]`

> At the beginning of your upkeep, you may flip a coin. If you win the flip, put a fuse counter on this enchantment. If you lose the flip, remove a fuse counter from this enchantment.
> Remove five fuse counters from this enchantment and sacrifice it: It deals 20 damage to target player or planeswalker.

**[import]** Coin-flip + *fuse* near-win (5 fuse → 20 damage). Reinforces the flip theme with a counter payoff; a natural errata target to make a real alt-win.

### Darksteel Reactor — {4}
*Artifact*  `[out of block]`

> Indestructible (Effects that say "destroy" don't destroy this artifact.)
> At the beginning of your upkeep, you may put a charge counter on this artifact.
> When this artifact has twenty or more charge counters on it, you win the game.

**[import]** *charge* → 20 = win, indestructible. The cleanest 'count-to-a-win' artifact for a counters cube; pairs with any charge/proliferate support.

### Helix Pinnacle — {G}
*Enchantment*  `[out of block]`

> Shroud (This enchantment can't be the target of spells or abilities.)
> {X}: Put X tower counters on this enchantment.
> At the beginning of your upkeep, if there are 100 or more tower counters on this enchantment, you win the game.

**[import]** *tower* → 100 = win, shrouded. The slow-burn green win-con — fits low power because it's glacial without ramp.

### Azor's Elocutors — {3}{W/U}{W/U}
*Creature — Human Advisor*  `[out of block]`

> At the beginning of your upkeep, put a filibuster counter on this creature. Then if this creature has five or more filibuster counters on it, you win the game.
> Whenever a source deals damage to you, remove a filibuster counter from this creature.

**[import]** *filibuster* → 5 = win, but taking damage removes one. Interactive and on-band; rewards a defensive (cleric/lifegain) shell.

### Simic Ascendancy — {G}{U}
*Enchantment*  `[out of block]`

> {1}{G}{U}: Put a +1/+1 counter on target creature you control.
> Whenever one or more +1/+1 counters are put on a creature you control, put that many growth counters on this enchantment.
> At the beginning of your upkeep, if this enchantment has twenty or more growth counters on it, you win the game.

**[import]** *growth* → 20 = win. Turns any +1/+1 theme into a clock — converts the block's many +1/+1 cards into a wincon.


## 2. Funky utility counters in the pool (non-+1/+1, non-combat)

The genuinely weird counter types the block shipped — exactly the 'fun, scales, not a +1/+1 pile' direction.


### Bomb Squad — {3}{R}
*Creature — Dwarf*  `[ODY]`

> {T}: Put a fuse counter on target creature.
> At the beginning of your upkeep, put a fuse counter on each creature with a fuse counter on it.
> Whenever a creature has four or more fuse counters on it, remove all fuse counters from it and destroy it. That creature deals 4 damage to its controller.

*fuse* — ticking time-bomb removal; fuses double each upkeep, at 4 → destroy + 4 to its controller. Counter-movers can shove an opponent's creature to 4 for on-demand removal. High decision density.

### Cephalid Vandal — {1}{U}
*Creature — Octopus Rogue*  `[TOR]`

> At the beginning of your upkeep, put a shred counter on this creature. Then mill a card for each shred counter on this creature.

*shred* — self-accelerating self-mill (1, 3, 6, 10…). Fuels graveyard payoffs; remove shred counters to avoid decking. Commit-and-manage.

### Delaying Shield — {3}{W}
*Enchantment*  `[ODY]`

> If damage would be dealt to you, put that many delay counters on this enchantment instead.
> At the beginning of your upkeep, remove all delay counters from this enchantment. For each delay counter removed this way, you lose 1 life unless you pay {1}{W}.

*delay* — bank incoming damage as counters, then each upkeep pay {1}{W} each or lose 1 life each. A mana-vs-life puzzle that buys time for the slow win-cons.

### Force Bubble — {2}{W}{W}
*Enchantment*  `[SCG]`

> If damage would be dealt to you, put that many depletion counters on this enchantment instead.
> When there are four or more depletion counters on this enchantment, sacrifice it.
> At the beginning of each end step, remove all depletion counters from this enchantment.

*depletion* — prevents ALL damage to you while it has depletion counters, banking that damage as counters; pops (and dumps it) when empty. A fog-bank engine.

### Decree of Silence — {6}{U}{U}
*Enchantment*  `[SCG]`

> Whenever an opponent casts a spell, counter that spell and put a depletion counter on this enchantment. If there are three or more depletion counters on this enchantment, sacrifice it.
> Cycling {4}{U}{U} ({4}{U}{U}, Discard this card: Draw a card.)
> When you cycle this card, you may counter target spell.

*depletion* — hard-counters opponents' spells, shedding a depletion counter each time (cycling gives a free counter). A heavy control lock — gate by power for this band.

### Aurification — {2}{W}{W}
*Enchantment*  `[ONS]`

> Whenever a creature deals damage to you, put a gold counter on it.
> Each creature with a gold counter on it is a Wall in addition to its other creature types and has defender. (Those creatures can't attack.)
> When this enchantment leaves the battlefield, remove all gold counters from all creatures.

*gold* — creatures that damage you get gold counters and become 0/4 Walls. Janky soft pillow-fort.

### Mine Layer — {3}{R}
*Creature — Dwarf*  `[ODY]`

> {1}{R}, {T}: Put a mine counter on target land.
> Whenever a land with a mine counter on it becomes tapped, destroy it.
> When this creature leaves the battlefield, remove all mine counters from all lands.

*mine* — turn lands into a minefield; land denial. Niche, slow, very low power.

### Trap Digger — {3}{W}
*Creature — Human Soldier*  `[SCG]`

> {2}{W}, {T}: Put a trap counter on target land you control.
> Sacrifice a land with a trap counter on it: This creature deals 3 damage to target attacking creature without flying.

*trap* — stockpile trap counters on your lands, sac for 2 damage. Slow repeatable reach for a white/soldier shell.


## 3. Affliction counters (-1/-1 & plague)

Slow, attrition-flavoured removal counters — low power, decision-heavy targeting.


### Traveling Plague — {3}{B}{B}
*Enchantment — Aura*  `[ODY]`

> Enchant creature
> At the beginning of each upkeep, put a plague counter on this Aura.
> Enchanted creature gets -1/-1 for each plague counter on this Aura.
> When enchanted creature leaves the battlefield, that creature's controller returns this Aura from its owner's graveyard to the battlefield.

*plague* — an aura that grows a -1/-1 each upkeep and hops to a new creature when its host dies. Spreading attrition.

### Withering Hex — {B}
*Enchantment — Aura*  `[ONS]`

> Enchant creature
> Whenever a player cycles a card, put a plague counter on this Aura.
> Enchanted creature gets -1/-1 for each plague counter on this Aura.

*plague* — cycle-triggered -1/-1 growth (Onslaught cycling payoff).

### Shambling Swarm — {1}{B}{B}{B}
*Creature — Horror*  `[TOR]`

> When this creature dies, distribute three -1/-1 counters among one, two, or three target creatures. For each -1/-1 counter you put on a creature this way, remove a -1/-1 counter from that creature at the beginning of the next end step.

*-1/-1* — on death, distribute three -1/-1 counters; flexible removal/shrink.

### Consumptive Goo — {B}{B}
*Creature — Ooze*  `[SCG]`

> {2}{B}{B}: Target creature gets -1/-1 until end of turn. Put a +1/+1 counter on this creature.

*-1/-1 + +1/+1* — pay {2}{B}{B} to shrink a creature and grow itself; a black mana sink.


## 4. Birds (and a note on Clerics)

The bird tribal core that uses counters. (Onslaught **Clerics** are a lifegain tribe with almost no counter cards — they support life-total win-cons like the Elocutors rather than counter engines, so the counter overlap lives on the **Bird** side. Hydromorph Gull is an Elemental Bird but a counter-*spell*, not a counter-permanent card, so it's noted only here.)


### Soulcatcher — {1}{W}
*Creature — Bird Soldier*  `[ODY]`

> Flying
> Whenever a creature with flying dies, put a +1/+1 counter on this creature.

**Bird** — grows whenever ANY flyer dies. Core of the bird package.

### Soulcatchers' Aerie — {1}{W}
*Enchantment*  `[JUD]`

> Whenever a Bird is put into your graveyard from the battlefield, put a feather counter on this enchantment.
> Bird creatures get +1/+1 for each feather counter on this enchantment.

**Bird** — *feather* counters as Birds die → anthem all your Birds. The build-around you wanted; it needs Bird density, which this cube intentionally supplies.

### Aven Farseer — {1}{W}
*Creature — Bird Soldier*  `[SCG]`

> Flying
> Whenever a permanent is turned face up, put a +1/+1 counter on this creature.

**Bird** — grows (+1/+1) whenever a permanent is turned face up (morph synergy from Onslaught).

### Aven Warhawk — {4}{W}
*Creature — Bird Soldier*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Bird and/or Soldier card you reveal in your hand.)
> Flying

**Bird** — *Amplify 1* (enters bigger per Bird revealed) plus a morph payoff; bridges Birds and the Amplify counter theme.


## 5. The Phantom 'shield counter' cycle

The block's best idea: +1/+1 counters used as removable damage SHIELDS. Each prevents ALL damage from a source by shedding one counter — and only one per source, so they soak gang-blocks and multi-source pings. Renewable interactive walls; counter-movers (Powerful Broker / Quarry Hauler) and any proliferate re-arm them. This is the *welcome* use of +1/+1 for this cube.


### Phantom Nomad — {1}{W}
*Creature — Spirit Nomad*  `[JUD]`

> This creature enters with two +1/+1 counters on it.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.

### Phantom Tiger — {2}{G}
*Creature — Cat Spirit*  `[JUD]`

> This creature enters with two +1/+1 counters on it.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.

### Phantom Nantuko — {2}{G}
*Creature — Insect Spirit*  `[JUD]`

> Trample
> This creature enters with two +1/+1 counters on it.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.
> {T}: Put a +1/+1 counter on this creature.

### Phantom Centaur — {2}{G}{G}
*Creature — Centaur Spirit*  `[JUD]`

> Protection from black
> This creature enters with three +1/+1 counters on it.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.

### Phantom Flock — {3}{W}{W}
*Creature — Bird Soldier Spirit*  `[JUD]`

> Flying
> This creature enters with three +1/+1 counters on it.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.

### Phantom Nishoba — {5}{G}{W}
*Creature — Cat Beast Spirit*  `[JUD]`

> Trample
> This creature enters with seven +1/+1 counters on it.
> Whenever this creature deals damage, you gain that much life.
> If damage would be dealt to this creature, prevent that damage. Remove a +1/+1 counter from this creature.


## 6. Amplify — tribal scaling +1/+1 (Onslaught)

Amplify creatures enter with N +1/+1 counters for each shared-type card you reveal from hand, so they scale with tribal density (Birds, Soldiers, Zombies, Beasts). They reward committing to a tribe and pair with the counter-mover/Phantom shells.


### Canopy Crawler — {3}{G}
*Creature — Beast*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Beast card you reveal in your hand.)
> {T}: Target creature gets +1/+1 until end of turn for each +1/+1 counter on this creature.

### Daru Stinger — {3}{W}
*Creature — Soldier*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Soldier card you reveal in your hand.)
> {T}: This creature deals damage equal to the number of +1/+1 counters on it to target attacking or blocking creature.

### Embalmed Brawler — {2}{B}
*Creature — Zombie*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Zombie card you reveal in your hand.)
> Whenever this creature attacks or blocks, you lose 1 life for each +1/+1 counter on it.

### Feral Throwback — {4}{G}{G}
*Creature — Beast*  `[LGN]`

> Amplify 2 (As this creature enters, put two +1/+1 counters on it for each Beast card you reveal in your hand.)
> Provoke (Whenever this creature attacks, you may have target creature defending player controls untap and block it if able.)

### Ghastly Remains — {B}{B}{B}
*Creature — Zombie*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Zombie card you reveal in your hand.)
> At the beginning of your upkeep, if this card is in your graveyard, you may pay {B}{B}{B}. If you do, return it to your hand.

### Glowering Rogon — {5}{G}
*Creature — Beast*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Beast card you reveal in your hand.)

### Kilnmouth Dragon — {5}{R}{R}
*Creature — Dragon*  `[LGN]`

> Amplify 3 (As this creature enters, put three +1/+1 counters on it for each Dragon card you reveal in your hand.)
> Flying
> {T}: This creature deals damage equal to the number of +1/+1 counters on it to any target.

### Zombie Brute — {6}{B}
*Creature — Zombie*  `[LGN]`

> Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Zombie card you reveal in your hand.)
> Trample


## 7. Other +1/+1 growth & counter granters in the pool

The rest of the block's counter creatures and enablers — tribal/conditional growth and effects that hand out counters (feed the Phantoms, Amplify, and Simic Ascendancy).


### Crazed Firecat — {5}{R}{R}
*Creature — Elemental Cat*  `[TOR]`

> When this creature enters, flip a coin until you lose a flip. Put a +1/+1 counter on this creature for each flip you won.

**coin flip → +1/+1** — flip until you lose, enters with that many +1/+1. Directly ties the flip theme to counters.

### Forgotten Ancient — {3}{G}
*Creature — Elemental*  `[SCG]`

> Whenever a player casts a spell, you may put a +1/+1 counter on this creature.
> At the beginning of your upkeep, you may move any number of +1/+1 counters from this creature onto other creatures.

**[ONS]** grows on every spell cast and MOVES its counters each upkeep — a counter-distribution engine. Powerful; gate by power or use as a marquee build-around.

### Elvish Vanguard — {1}{G}
*Creature — Elf Warrior*  `[ONS]`

> Whenever another Elf enters, put a +1/+1 counter on this creature.

### Stag Beetle — {3}{G}{G}
*Creature — Insect*  `[ONS]`

> This creature enters with X +1/+1 counters on it, where X is the number of other creatures on the battlefield.

### Kurgadon — {4}{G}
*Creature — Beast*  `[SCG]`

> Whenever you cast a creature spell with mana value 6 or greater, put three +1/+1 counters on this creature.

### Chlorophant — {G}{G}{G}
*Creature — Elemental*  `[ODY]`

> At the beginning of your upkeep, you may put a +1/+1 counter on this creature.
> Threshold — As long as there are seven or more cards in your graveyard, this creature has "At the beginning of your upkeep, you may put another +1/+1 counter on this creature."

### Repentant Vampire — {3}{B}{B}
*Creature — Vampire*  `[ODY]`

> Flying
> Whenever a creature dealt damage by this creature this turn dies, put a +1/+1 counter on this creature.
> Threshold — As long as there are seven or more cards in your graveyard, this creature is white and has "{T}: Destroy target black creature."

### Vampiric Dragon — {6}{B}{R}
*Creature — Vampire Dragon*  `[ODY]`

> Flying
> Whenever a creature dealt damage by this creature this turn dies, put a +1/+1 counter on this creature.
> {1}{R}: This creature deals 1 damage to target creature.

### Savage Firecat — {3}{R}{R}
*Creature — Elemental Cat*  `[ODY]`

> Trample
> This creature enters with seven +1/+1 counters on it.
> Whenever you tap a land for mana, remove a +1/+1 counter from this creature.

### Nantuko Cultivator — {3}{G}
*Creature — Insect Druid*  `[TOR]`

> When this creature enters, you may discard any number of land cards. Put that many +1/+1 counters on this creature and draw that many cards.

### Ironshell Beetle — {1}{G}
*Creature — Insect*  `[JUD]`

> When this creature enters, put a +1/+1 counter on target creature.

+1/+1 granter on a body — cheap enabler for the +1/+1 / Phantom / Amplify webs.

### Unspeakable Symbol — {1}{B}{B}
*Enchantment*  `[SCG]`

> Pay 3 life: Put a +1/+1 counter on target creature.

### Invigorating Boon — {1}{G}
*Enchantment*  `[ONS]`

> Whenever a player cycles a card, you may put a +1/+1 counter on target creature.

### Decree of Savagery — {7}{G}{G}
*Instant*  `[SCG]`

> Put four +1/+1 counters on each creature you control.
> Cycling {4}{G}{G} ({4}{G}{G}, Discard this card: Draw a card.)
> When you cycle this card, you may put four +1/+1 counters on target creature.

mass +1/+1 (four on each of your creatures) with a cycling mode — a tribal payoff.

### Ivy Elemental — {X}{G}
*Creature — Elemental*  `[ODY]`

> This creature enters with X +1/+1 counters on it.

### Junk Golem — {4}
*Artifact Creature — Golem*  `[ODY]`

> This creature enters with three +1/+1 counters on it.
> At the beginning of your upkeep, sacrifice this creature unless you remove a +1/+1 counter from it.
> {1}, Discard a card: Put a +1/+1 counter on this creature.

### Riptide Replicator — {X}{4}
*Artifact*  `[ONS]`

> As this artifact enters, choose a color and a creature type.
> This artifact enters with X charge counters on it.
> {4}, {T}: Create an X/X creature token of the chosen color and type, where X is the number of charge counters on this artifact.

### Forcemage Advocate — {1}{G}
*Creature — Centaur Shaman*  `[JUD]`

> {T}: Return target card from an opponent's graveyard to their hand. Put a +1/+1 counter on target creature.
