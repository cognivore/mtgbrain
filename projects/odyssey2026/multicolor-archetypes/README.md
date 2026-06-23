# Multicolor (gold) cards by archetype — research

Historic two-colour (gold) cards that **fit the cube's dream archetypes**, **work as printed (no errata needed)**,
and **hold the cube's very-low / decision-dense power line**. One report per guild archetype, produced by one
research subagent each, querying the mtgbrain card DB.

Method per file: filter to exactly two colours via `card_colors` (order-independent), scan oracle text for the
archetype's mechanics, judge era from the `printings` set codes (old-frame ≈ pre-2004 preferred), reject bombs,
and cross-check against `cube360/cube_list.txt` + `reports/worksheet_Multicolor.md` so existing cards aren't
re-counted. "First set (year)" is each card's debut; post-2004 picks are flagged.

## The reports

| Archetype | File | Best old-frame (pre-2004) anchor |
|---|---|---|
| {W}{U} Self-Mill Enchantments | [WU-self-mill-enchantments.md](WU-self-mill-enchantments.md) | Reviving Vapors (INV '00), Phantatog (ODY '01), Hanna (INV '00) |
| {U}{B} Discard Storm | [UB-discard-storm.md](UB-discard-storm.md) | Recoil (INV '00), Unfulfilled Desires (MIR '96), Nebuchadnezzar (LEG '94) |
| {B}{R} Hellbent Madness | [BR-hellbent-madness.md](BR-hellbent-madness.md) | Lim-Dûl's Paladin (ALL '96); keyword pool is Dissension ('06) |
| {B}{G} Stax | [BG-stax.md](BG-stax.md) | Dark Heart of the Wood (DRK '94); rest is Ravnica '05 |
| {W}{B} Lifeloss | [WB-lifeloss.md](WB-lifeloss.md) | Soul Link / Gerrard's Verdict / Martyrs' Tomb (all APC '01) |
| {U}{R} Laboratory Storm | [UR-laboratory-storm.md](UR-laboratory-storm.md) | Frenetic Efreet (MIR '96), Quicksilver Dagger + Prophetic Bolt (APC '01) |
| {U}{G} Proliferate | [UG-proliferate.md](UG-proliferate.md) | Malignant Growth (MIR '96) only; archetype is a 2005+ invention |
| {W}{R} Aggro | [WR-aggro.md](WR-aggro.md) | Goblin Legionnaire (APC '01); Boros identity is a 2005+ invention |
| {W}{G} Total Control | [WG-total-control.md](WG-total-control.md) | Reclamation (ICE '95), Dueling Grounds (INV '00), Hunting Grounds (JUD '02) |

## Cross-cutting findings

- **Era reality.** Gold cards were rare before Invasion (2000). For colour pairs whose *identity itself* was
  designed in Ravnica (2005) — notably **UG counters/proliferate**, **WR Boros aggro**, and the **Hellbent**
  keyword (Dissension '06) — there is essentially **no pre-2004 gold pool**. Those reports lean on the cleanest
  low-power Ravnica-era commons/uncommons, each flagged by set + year, so you can decide per the cube's
  "post-2004 creatures only rarely" rule.
- **Richest old-frame veins:** WB Orzhov lifeloss and WG Selesnya control both have real Apocalypse / Ice-age-era
  gold to draw on; WU and UB get solid Invasion-block enablers.
- **Creature caution:** WR aggro is almost entirely creatures, and most are 2005+ — that report flags every
  post-2004 body explicitly. BR is mostly Dissension creatures. Weigh these against the creature rule.
- **No errata required:** every "Top picks" card across the nine reports functions as printed at this power level;
  errata candidates (busted/oppressive lines) were pushed to each file's maybeboard with a reason.
- **Worksheet overlaps already excluded:** Psychatog (UB), Pernicious Deed / Smokestack / Squandered Resources (BG),
  Death Grasp / Final Payment (WB), Stitch in Time / Fevered Visions / Epic Experiment (UR), Sterling Grove /
  Phantom Nishoba / Last Stand (WG), Goblin Trenches / Glory of Warfare (WR) are noted as present, not new finds.

## Data note

One subagent flagged that **Repeal** and **Azor's Elocutors** (cited as existing WU payoffs) were **not found by
name in `cube_list.txt`** — worth a manual check of whether those are actually in the list or only in the worksheet plan.
