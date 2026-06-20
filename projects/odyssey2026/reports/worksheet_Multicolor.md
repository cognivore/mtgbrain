# oddysey2026 — MULTICOLOR / GOLD worksheet

32 singleton gold cards. Gives each of the 10 guilds a build-around identity anchor while staying
VERY low power and decision-dense. Gold here is mostly colour-shifted lock artifacts + gold
enchantments/spells, because the Odyssey-block creature pool contributes only **4** gold creatures.

## What this worksheet does
Anchors the 5 **locked colour-shift** lock pieces in their data-chosen guilds and layers on honest,
slow build-arounds: alternate-win permanents, symmetric draw/lifegain punishers, sac/counter
engines, and a small toolbox. No FIRE, no swingy bombs. Every alt-win is stapled to a removable
permanent or a cast-and-spent spell.

## Locked colour-shifts (all flags.colorshift set)
| Card | Guild | Lock it provides |
|---|---|---|
| **Opposition** → G/W | Selesnya | Tap-down lock fed by a wide token board (W flood + G tokens). |
| **Tangle Wire** → U/G | Simic | Fading soft-lock; the FADE counters are a proliferate target. |
| **Smokestack** → B/G | Golgari | Symmetric sacrifice engine on soot counters; counter-mover home. |
| **Ensnaring Bridge** → B/R | Rakdos | Hellbent prison; empty your hand to shut off attackers. |
| **Opalescence** → U/W | Azorius | Animates enchantments; half of the Opalescence+Humility lock. |

## Guild identity coverage (every guild gets >=2 gold)
- **WU Azorius (6):** Opalescence, Enchanted Evening (errata), Teferi's Moat, Dovescape, Azor's
  Elocutors (filibuster alt-win), Iridescent Angel. Enchantmentress / Replenish / pillowfort.
- **UB Dimir (4):** Glimpse the Unthinkable (mill payoff), Phyrexian Tyranny (>U/B draw-tax),
  Soul Manipulation (modal counter/regrow), Shadowmage Infiltrator. Control-combo + deck-out.
- **BR Rakdos (4):** Ensnaring Bridge, Spiteful Visions (wheel→clock), Rain of Gore (anti-lifegain),
  Vampiric Dragon. Hellbent / punisher.
- **BG Golgari (3):** Smokestack, Squandered Resources (burst-mana sink), Pernicious Deed
  (symmetric sac-reset). Sacrifice + counters.
- **RG Gruul (2):** Stormbind (repeatable {2}+discard sink → 2 dmg, hellbent), Aether Rift (random
  upkeep discard → reanimate). Big-red mana/graveyard sink.
- **GW Selesnya (4):** Opposition, Sterling Grove (enchantment tutor + shroud), Glittering Wish
  (gold toolbox), Phantom Nishoba. Token flood + tap-down + enchantress glue.
- **WB Orzhov (2):** Final Payment (pay-5-life removal), Death Grasp (X dmg + gain X). Life-as-resource.
- **UR Izzet (3):** Fevered Visions (wheel-punisher), Stitch in Time (coin-flip extra turn),
  Epic Experiment (cast-and-spent free-spell payoff). Spells / coin-flip / storm-lite.
- **WR Boros (2):** Goblin Trenches (sac-land token sink), Glory of Warfare (asymmetric anthem).
  Token grind / aggro-tax.
- **UG Simic (2):** Tangle Wire, Simic Ascendancy (+1/+1 growth → 20 = alt-win). Proliferate / counters.

## Honest alternate wins (stapled, beatable)
- **Azor's Elocutors** — 5 filibuster counters = win, on a removable creature, resets on damage.
- **Simic Ascendancy** — 20 growth counters = win, fed by +1/+1 counters; slow, removable.
- **Epic Experiment** — not a win-con but the model honest payoff: cast spells off the top, fully spent.

## Errata flags
- **Enchanted Evening** (errata=true): ETB also mill top 3 and return an enchantment card to hand
  (per spec #10) so it isn't a do-nothing the turn it lands.

## Notable cuts (full list in maybeboard)
- **Karlov / Vizkopa Guildmage / Cliffhaven Vampire / Vizkopa Confessor / Corpsejack Menace** —
  all CREATURES not in the Odyssey pool; hard-rule violation. Cut to maybeboard. WB and BG gold
  identity therefore leans on noncreature spells, not these payoffs.
- **Mind Funeral** — second UB mill; kept Glimpse the Unthinkable as the cleaner payoff ("keep best").
- **Debtors' Knell / Assemble the Legion** — board-camping value engines (flagged too_strong).
- **Pyrrhic Revival / Death Frenzy / Powerstone Minefield / Dralnu's Crusade** — fun but redundant
  or too swingy for their guild slot.

## Exceptions used
- **Azor's Elocutors** — source="exception", flags.exception=true. WU filibuster alt-win; no such
  creature exists in the Odyssey pool, and it is exactly the honest-alt-win-on-a-permanent the
  brief wants. 1 exception only.
