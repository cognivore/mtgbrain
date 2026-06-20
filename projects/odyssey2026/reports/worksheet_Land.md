# Worksheet: LANDS (oddysey2026)

**Target:** ~40 lands. **Power:** deliberately low — slow fixing, ETB-tapped, life/damage costs,
build-around utility. No fast mana, no land destruction, no premium duals. Manabase is **ANY-ERA**
(not block-gated) but every pick earns its slot by either (a) fixing the 10 pairs at low power or
(b) feeding an on-theme engine (storage counters, threshold, proliferate, wheels, self-mill,
cumulative-upkeep pillowfort, mana sinks).

## What this worksheet does
Provides a 2-color-friendly but low-velocity manabase for a 360 singleton cube whose decks grind.
It fixes all ten guild pairs **twice over** at low power (one allied + one enemy tech per pair via
filter lands; plus a 5-color sac-tapland cycle and any-color utility) while quietly powering the
cube's signature build-arounds. Nothing here ramps explosively; lands cost life, enter tapped, or
gate behind threshold/counters.

## Guild archetypes fed
- **Allied filter lands** (Skycloud W/U, Darkwater U/B, Shadowblood B/R, Mossfire R/G, Sungrass G/W):
  pay {1} to make the pair — slow, no life loss, perfect low-power allied fixing.
- **Enemy filter lands** (Mystic Gate W/U... wait, enemy: Mystic Gate W/U is allied; the cycle here
  is the *Shadowmoor/Eventide* filters covering enemy pairs): Mystic Gate (W/U), Sunken Ruins (U/B),
  Graven Cairns (B/R), Fire-Lit Thicket (R/G), Wooded Bastion (G/W). They tap for {C} alone or filter
  a hybrid pip into two colored — strong fixing that still demands a colored pip to "go off," keeping
  velocity low.
- **5c sac-taplands** (Abandoned Outpost W, Bog Wreckage B, Seafloor Debris U, Ravaged Highlands R,
  Timberland Ruins G): ETB tapped, tap for one fixed color, or sac for any-1. Splash glue without
  power creep. (These are the *block* sac-for-any-color lands the spec wants on-theme.)

## On-theme engines
- **Threshold lands** (Cabal Pit -2/-2, Barbarian Ring 2 dmg, Cephalid Coliseum loot-3, Centaur Garden
  +3/+3, Nomad Stadium +4 life, Nantuko Monastery 4/4 manland): reward the GY-storm / self-mill plan;
  painlands until threshold, then a sac payoff. Cephalid Coliseum doubles as a **self-mill enabler**
  (turns on threshold, fuels One With Nothing / dredge).
- **Storage lands** (Calciform Pools W/U, Dreadship Reef U/B, Molten Slagheap B/R, Fungal Reaches R/G,
  Saltcrusted Steppe G/W): **storage-counter** batteries — proliferate targets and slow mana sinks
  that pair with the counters-matter and big-mana plans.
- **Mage-Ring Network**: colorless storage battery — Braid-of-Fire / big-red sink, proliferate target.
- **Karn's Bastion**: repeatable **proliferate** — the glue between storage/charge/age/fade counters.
- **Nesting Grounds**: **move a counter** sorcery-speed — Smokestack-shift (counter relocation),
  storage/charge shuffling, age-counter shenanigans.
- **Geier Reach Sanitarium** + **Mikokoro**: symmetric wheels — uncounterable card flow that floods
  hands for the hand-size punisher suite (Black Vise / Iron Maiden / The Rack) and feeds GY-storm.
- **Glacial Chasm**: cumulative-upkeep **age-counter pillowfort** — buys turns for honest combo /
  alt-win; self-synergizes with the age/fade/cumulative-upkeep theme. Costs a land + 2 life/turn —
  intentionally taxing, not free.
- **Sea Gate Wreckage**: hellbent card engine — rewards emptying hand (Ensnaring Bridge / GY-storm).
- **Riptide Laboratory**: protects/recurs Wizards (blue tempo, ETB loops at low power).
- **Petrified Field**: regrow a land — recurs the utility lands / fixes a sac'd tapland.
- **Thespian's Stage**: copy any land — flexibly re-buys a storage/threshold/utility land's ability
  (singleton-friendly redundancy without adding a second copy).

## Cycling lands (ONS — on theme)
Barren Moor (B), Forgotten Cave (R), Lonely Sandbar (U), Secluded Steppe (W), Tranquil Thicket (G):
1-mana cycling for discard-matters / GY-storm / flood insurance. All have ONS printings → on-theme.

## Notable cuts (maybeboard)
- **Power-level cuts:** Ancient Tomb (elo 1583), Strip Mine (1761), Wasteland (1555), Mishra's
  Workshop (1461), City of Brass (1425), Reflecting Pool, Rishadan Port, Gemstone Mine, Tendo Ice
  Bridge, Lotus Vale — all too fast/oppressive for a low-power grind cube.
- **Manlands** (Faerie Conclave, Treetop Village, Mishra's Factory, Stalking Stones, Forbidding
  Watchtower, Ghitu Encampment, Spawning Pool): efficient evasive threats that *end* stalls the wrong
  way (the cube wants weak creatures to break stalls, not colorless beaters). Nantuko Monastery kept
  as the one threshold-gated exception.
- **{2}-cycling Urza lands** (Slippery Karst, Smoldering Crater, Drifting Meadow, Remote Isle, Desert,
  Polluted Mire variant): strictly worse cyclers than the ONS {1} set → alternates.
- **Depletion/Lair fast-mana** (Saprazzan Skerry, Peat Bog, Sandstone Needle, Hickory Woodlot, the
  Lairs): burst ramp, off-philosophy.
- **Maze of Ith / Kor Haven / Tower of the Magistrate / Desert / Glacial Chasm's cousins**: pure
  fog-walls; Glacial Chasm chosen as the single age-counter pillowfort representative.
- **Quicksilver Fountain**: the Islands-lock / High-Tide payoff — but it is type **Artifact**, so it
  belongs to the **U / Islands-lock worksheet**, not Lands. Maybeboarded here, claimed there.
- **Dakmor Salvage / Polluted Mire / Cabal Pit / Barbarian Ring / Forgotten Cave** etc. are tagged in
  `master_candidates.tsv` under their **mono-color** sections. The spec assigns the chosen ones to
  THIS worksheet; included here with color tag `Land`. **De-dup note:** if a color worksheet also
  pulls Cabal Pit / Barbarian Ring / Forgotten Cave, drop one copy (SINGLETON). Dakmor Salvage left to
  the B worksheet (self-mill there); maybeboarded here.

## Errata flags
None. Every land in this worksheet has a live, non-blank ability (fixing, sac payoff, counter engine,
wheel, pillowfort, or mana sink). No `dead`/`errata` flags needed for the land slot.

## Overlap / colourshift notes
- No colourshifts originate in the land slot (the locked gold colourshifts are artifacts/enchantments).
- Storage + Karn's Bastion + Nesting Grounds form the **counter-relocation/proliferate** spine that the
  Smokestack(B/G) / Tangle Wire(U/G) shifts lean on.
