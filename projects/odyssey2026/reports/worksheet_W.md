# Worksheet W — Mono-White (oddysey2026)

**Target:** ~48 (24 creatures + 24 noncreatures). Singleton. Very low power, decision-dense.

## What this worksheet does
White is the cube's **enchantment-lock + pillowfort + life-as-resource** color. It supplies:
weak evasive/threshold bodies that *break stalls* rather than win races; Phantom +1/+1 damage-shields;
a Cleric prevention/lifegain sub-shell; tappers and tax that imitate Opposition on a budget; and the
mono-W half of the **Opalescence + Humility / Replenish** enchantment engine. No FIRE, no swingy bombs.

## Hard-rule note (creature pool boundary)
The curated creature pool `odysseyblock_creatures.tsv` is **Odyssey-block only** (ODY/TOR/JUD) — it
contains **zero Onslaught-block (ONS/LGN/SCG)** cards. So most creatures the brief *named*
(Aven Farseer, Mistral Charger, Battlefield Medic, the Soldier-flood lords, Glowrider, Whipcorder,
the ONS Clerics) are **not in the pool and cannot be used**. The Soldier weenie-FLOOD plan is not
buildable from this pool; only ODY-block Nomads/Clerics/Birds exist. All 24 creatures below are pulled
from the 25 white cards actually present in the pool (every one used except Silver Seraph). No
exceptions invoked — the pool covers the needs.

## Creatures (24) — archetype map
- **Birds / fliers (→ WX-birds, Soulcatchers' Aerie):** Suntail Hawk, Soulcatcher, Mystic Familiar,
  Battlewise Aven, Lieutenant Kirtar, Commander Eesha. Soulcatcher + Soulcatchers' Aerie is the small
  tribal build-around; fliers also feed the Aerie's death triggers.
- **Phantom shields (counters + break stalls):** Phantom Nomad, Phantom Flock — soak combat, pair with
  the cube's +1/+1-counter theme.
- **Threshold weenies (stall-breakers that grow/fly late):** Mystic Penitent, Mystic Visionary,
  Mystic Zealot, Vigilant Sentry, Valor (graveyard first-strike anthem), Wayward Angel (threshold flier
  top-end; turns black).
- **Tap / tax / go-wide (→ GW Opposition, WR tax):** Nomad Decoy (budget tapper), Beloved Chaplain
  (unblockable aura/equipment carrier + tax body), Pianna Nomad Captain (attack anthem), Blessed Orator
  (+0/+1 team toughness, durable wall-on-legs).
- **Clerics (→ WB lifegain/drain shell):** Master Apothecary, Militant Monk (tap-Cleric prevention
  engines), Ancestor's Chosen (lifegain payoff scaling off graveyard).
- **Enchantment recursion (→ WU enchantments / Replenish):** Auramancer returns an enchantment from GY.
- **Other utility:** Patrol Hound (discard outlet → threshold/madness/graveyard), Possessed Nomad
  (vigilant horror, breaks ground stalls).

## Noncreatures (24) — the white spine
- **Enchantment engine / honest combo:** Replenish (mass-reanimate enchantments — cast-and-spent,
  #16-legal), Humility (lock half; Opalescence lives in the UW colorshift section, NOT here),
  Starfield of Nyx (Replenish-on-a-stick), Open the Vaults considered but cut.
- **Fading removal (the missed theme):** Parallax Wave (depletion-counter creature exile; blink-with-
  Replenish is the marquee interaction). Parallax **Tide is blue** → maybeboard, not here.
- **Pillowfort / tax payoffs:** Sphere of Safety, Ghostly Prison (chosen over Propaganda — Propaganda is
  blue), Suppression Field (activated-ability tax — hoses Smokestack/lock engines), Rule of Law (chosen
  over Arcane Laboratory — that one is blue).
- **Enchantress shell:** Mesa Enchantress (the draw engine — no in-block equivalent, taken as a
  noncreature any-era include), Sigil of the Empty Throne (4/4-flier payoff), Soulcatchers' Aerie.
- **Lock protection / removal:** Seal of Cleansing + Aura of Silence (recur via Replenish for repeated
  Disenchant), Karmic Justice (punish breaking the lock), Enlightened Tutor (chosen over Idyllic Tutor —
  cheaper, lower-power, top-of-deck), Cage of Hands (#15 reusable tax-attack aura), Story Circle
  (color damage lock), Solitary Confinement (fog-lock; needs the card engine), Oblation (symmetric
  any-era removal — gives the target 2 cards, on-philosophy).
- **Life-as-resource (honest alt-win build-arounds):** Test of Endurance (50+ life), Near-Death
  Experience (be at exactly 1), Transcendence (survive at 0 / lose at 20 — perfect very-low-power,
  decision-dense ODY-block build-around), Earnest Fellowship (quasi-Humility combat/aura hoser).

## Honest-combo check
All win-cons are stapled to spells/permanents and must be assembled and maintained: Replenish is one-shot
and needs graveyard fill; the life-alt-wins demand precise life management; Sphere/Sigil need enchantment
density. Nothing here is a board-camping value engine (no Citadel/Storm/Bargain), so nothing is sent to
maybeboard for too_strong.

## Errata flags
**None needed in white.** The natural dead+errata candidate, **Battle of Wits** (200-card library is
impossible in a singleton cube), turned out to be **blue** (cube_color=U) → maybeboarded as off-color,
not errata'd here. No literal-blank mono-W build-around exists in the pool. Note: the cube's locked
**Opalescence>U/W** colorshift and **Enchanted Evening** errata are handled in the UW worksheet, not here.

## Notable cuts (full list in maybeboard)
Silver Seraph (WWW 8-drop anthem — too top-heavy, redundant with Valor/Blessed Orator anthems);
the entire ONS-block Soldier/Cleric suite (not in pool); Idyllic Tutor, Propaganda, Arcane Laboratory
(lost the "pick one" / are blue); Greater Auramancy & Sterling-style protection (redundant);
Axis of Mortality / Mirror-Universe pieces (slow, scrutiny); Open the Vaults (symmetric downside);
the dozens of weak ODY/ONS filler bodies and lifegain riders.
