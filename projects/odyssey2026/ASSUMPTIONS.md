# oddysey2026 — interpretation, assumptions & locked decisions

Companion to [`SPEC.md`](./SPEC.md) (the verbatim brief). This file records how the spec is being
turned into a 360, every judgement call, and the open questions. Revise here, not in SPEC.md.

## How this is being built
- **Mode:** ULTRACODE (multi-agent workflows) + `/loop` (self-paced overnight iteration).
- **Pipeline:** (1) verbatim spec → `SPEC.md` ✓ · (2) DB-driven pool generation (workflow) →
  `research/pool_*.json` · (3) synthesis → `cube360/` 360 master + per-colour worksheet CSVs ·
  (4) `PRIMER.md` + `COLOR-PAIRS.md` + `COLOR-ROLES.md` + `ERRATA-AND-COLORSHIFTS.md`.
- **DEFERRED to a later "afternoon task"** (user's words): *prove it works* — define functional
  slots, show which card fills each slot, and build example 40-card decks per archetype. Tracked,
  not done now.

## Interpretation of the hard rules
- **Creatures:** Odyssey block (ODY/TOR/JUD, 205 creatures) core + Onslaught block (ONS/LGN/SCG)
  gap-fillers (435 total with Onslaught). Creatures from any other set only as a flagged
  `Out-exception` with a structural reason. Sanctioned exceptions so far: **Narcomoeba**
  (self-mill payoff), **Laboratory Maniac** (Ad Nauseam / mill finish — "never Thassa's Oracle"),
  **Death's Shadow** (#11 life-loss payoff). Each must earn its slot.
- **Non-creatures:** any era; best low-power fit (this is where the locks / alt-wins / fixing come from).
- **Honest combo (#16):** wins are commitments *stapled to a spell* (Ad Nauseam yes; Bolas's Citadel
  / Thousand Year Storm no). Board-camping value engines are flagged `too_strong` and cut.
- **#12 / #15:** any card that would be a literal blank is flagged `errata_needed` with a concrete
  proposal (ETB effect / mana sink) rather than silently included or dropped.
- **Singleton (#17).**

## Locked colour-shifts (decided by DB data, not vibes)
| Card | → Colours | Guild | Why (data) |
|---|---|---|---|
| **Opposition** | G/W | Selesnya | Biggest turn-4 creature flood: W 50 cheap creatures + G 14 token-makers (next colour ≤2). |
| **Tanglewire** (Tangle Wire) | U/G | Simic | Most proliferate: U 17 + G 11 *non-creature* sources (+10 colourless). |
| **Smokestack** | B/G | Golgari | In-cube counter-movers are green (Powerful Broker, Quarry Hauler, Forgotten Ancient, Maulfist); black removes counters / sacrifices. |
| **Ensnaring Bridge** | B/R | Rakdos | Discard-your-hand / hellbent: R 68 + B 41 (madness B 31 + R 21). |
| **Opalescence** | U/W | Azorius | Per spec #10. |

## The 10-guild archetype map
- **WU Azorius** — Enchantmentress / Replenish: Opalescence+Humility lock, Enchanted Evening (errata), Parallax *fading*.
- **UB Dimir** — Honest control-combo: Ad Nauseam, High Tide, Lab Maniac, Teferi's Puzzle Box wheel-lock, Sundial.
- **BR Rakdos** — Hellbent graveyard-storm + Ensnaring Bridge prison: One With Nothing, Shadow of the Grave, rituals, Tendrils.
- **BG Golgari** — Sacrifice + counters: Smokestack, −1/−1 plague, counter-movers, aristocrats.
- **RG Gruul** — Big-red mana sink: Braids of Fire, cumulative-upkeep/age, *fading*, coin-flip Chance Encounter, X-spells.
- **GW Selesnya** — Token flood + tap-down: Opposition, Squirrels/Saprolings/Soldiers, Phantom shields, Soulcatchers' Aerie.
- **WB Orzhov** — Life-as-resource: Mirror Universe, Death's Shadow, pay-life, Clerics lifegain, Azor's Elocutors.
- **UR Izzet** — Spells / storm-lite: coin-flip (Krark), Lightning Storm Ad-Nauseam finish, free-spell tempo, Sundial.
- **WR Boros** — Aggro tax / hatebears-lite: Cage of Hands, Trap Digger, soldiers, taxes behind a clock.
- **UG Simic** — Proliferate / counters / fading: Tanglewire, charge/tower alt-wins, Simic Ascendancy, ramp.

## Target distribution (≈360 — converges in synthesis)
Lands ~36 · Colorless artifacts ~44 · Mono W/U/B/R/G ~38 each (~190) · Gold ~8/guild (~80). Block has
only **9 gold creatures**, so the gold section is mostly colourshifted artifacts + gold enchantments/spells.

## Errata captured so far
- **Enchanted Evening** (#10): ETB *also* mill top 3 and return an enchantment card to hand.
- **One With Nothing** (#2): flagged — likely kept as an enabler paired with **Shadow of the Grave**
  (discard hand → return it all → card-neutral storm fuel); user to confirm any further buff.
- **Lantern pieces** (#15): flagged `errata_needed` so each does something alone (e.g. mill 1/turn).
- Full list will live in `ERRATA-AND-COLORSHIFTS.md`.

## Corrections applied (2026-06-20, mid-run, from user)
1. **Creatures come ONLY from the curated `/odysseyblock` cube** — 243 creatures extracted to
   `pool/odysseyblock_creatures.tsv`. The earlier wide DB pull (`pool/block_creatures.tsv`, 435) is
   abandoned for creature selection. This resolves the old "Onslaught gap-fillers?" question: the
   creature pool is exactly whatever `/odysseyblock` contains.
2. **Maybeboard is a hard rule** — every considered-but-cut card is parked on the maybeboard with a
   reason; nothing is discarded.
3. **Lands are NOT block-gated** — non-creatures are any-era; the manabase is a light fixing layer.
4. **Project dir reconciled** to `projects/odyssey2026/` (folded `cube/odyssey` → `legacy/`).
5. **Discoveries written up as reports** under `reports/` (one per worksheet) — not just card dumps.
6. Alan thinking disabled per request.

## Open questions (async — these will NOT block the run; sensible defaults assumed)
1. **Onslaught creatures** — resolved: pool = the `/odysseyblock` cube's creatures, full stop.
2. **Spreadsheet format** — per-colour CSV worksheets are canonical; also want a single `.xlsx`? *(Assumed: CSV; xlsx if trivial.)*
3. **Gold density** — OK with ~8 gold/guild (mostly colourshifted artifacts + gold enchantments)?
4. Any cards to **force-include / force-exclude** beyond the spec?
