# Cookbook — verified natural-language → SQL recipes

Worked examples for finding obscure cards. Each query below was authored for one
archetype, **executed against the real database**, then adversarially re-checked by a
second pass that hunted false positives and corrected the query — and every query here
was re-run once more when this file was generated (the ✓ row count is live).

Two things to remember while reading:

- These recipes rank by `edhrec_rank` (Commander popularity, lower = better). To rank by
  **cube power instead**, swap in `cube_elo IS NOT NULL ORDER BY cube_elo DESC` (higher =
  better). The *finding* logic is the hard part; the *ranking* is a one-line swap.
- The NULL-last idiom `ORDER BY (edhrec_rank IS NULL), edhrec_rank` keeps unrated cards
  from sorting to the top. See the cheatsheet (`mtgbrain schema`) for why.


## Add counters of a specific named type

**Ask:** Which cards put a specific named counter type (e.g. stun) onto a permanent — show me the most-played ones?

_✓ runs, 6+ hits_

```sql
SELECT display_name, type, mana_value AS cmc, MIN(edhrec_rank) AS edhrec_rank
FROM cards
WHERE is_funny = 0
  AND edhrec_rank IS NOT NULL
  AND (
        text LIKE '%put a stun counter%'
     OR text LIKE '%put two stun counters%'
     OR text LIKE '%put three stun counters%'
     OR text LIKE '%put X stun counters%'
     OR text LIKE '%stun counters equal to%'
     OR text LIKE '%stun counters among%'        -- "distribute N stun counters among"
     OR text LIKE '%enters%with%stun counter%'   -- "enters tapped with three stun counters"
     OR text LIKE '%battlefield%with%stun counter%' -- "return ... with two stun counters"
  )
GROUP BY display_name
ORDER BY edhrec_rank;

-- Parameterize by swapping the literal "stun" for any named counter type (charge, oil, lore, etc.).
-- NOTE: for stun specifically, the simpler `text LIKE '%stun counter%'` is equally precise
-- (all 90 cards that mention a stun counter actually add one). The verb-list above is the
-- portable version that stays clean for noisier types like charge/lore where some mentions
-- are "remove a charge counter" cost-payment references rather than additions.
```

**Why it works:** It matches the ADD/PUT action verbs ("put a X counter", "enters with a X counter", "X counters equal to") for one named counter type rather than merely mentioning the counter, so every row is a card that actively generates that counter. Swap the literal "stun" (and its a/an article) to parameterize for charge, oil, lore, page, etc. NULL-safe ranking surfaces the staples first.

**Sample hits:** Unstoppable Slasher (1672), Mjolnir, Storm Hammer (3177), Fear of Sleep Paralysis (3503), Lulu, Stern Guardian (3933), Baloth Prime (4397), Cryogen Relic (4689), Kaito, Bane of Nightmares (4931), Dreamdew Entrancer (6550), Lodestone Needle (13877), Sleep-Cursed Faerie (14579), Freeze in Place, Impede Momentum …

**Watch out:** The named counter is a phrase with a symbol-free word, so LIKE works, but FTS would over-match: 'stun counter' in cards_fts ignores word adjacency only loosely and the FTS tokenizer drops punctuation — fine here, but FTS can't distinguish "put a stun counter" from "remove a stun counter," so LIKE on raw text is the right tool to isolate ADDERS. ARTICLE GOTCHA: it's "a charge/stun counter" but "an oil counter" — when changing the type you must change a/an too (or drop the article and match bare '%oil counter%' on the put/enters verbs), or you silently under-match. Don't just match '%X counter%' alone: that also catches cards that only REMOVE or COUNT the counter (e.g. "remove three charge counters", "for each charge counter"), inflating results with non-adders. NULL edhrec_rank sorts FIRST by default, so use ORDER BY (edhrec_rank IS NULL), edhrec_rank to keep unranked cards last instead of WHERE edhrec_rank IS NOT NULL (which would drop legit newer/obscure adders entirely). is_funny=0 drops Un-set jokes. Multi-face note: text is per-face (one row per face), so a card that adds the counter on only its back face still matches via that face's row and is correctly returned under its display_name.

---

## Remove counters from permanents

**Ask:** Which cards remove counters from permanents (mine or my opponents'), e.g. via "remove a +1/+1 counter" or "remove up to three counters from target permanent" — and which are the popular ones?

_✓ runs, 6+ hits_

```sql
SELECT display_name, edhrec_rank, substr(text, instr(lower(text),'remove'), 55) AS removal
FROM cards
WHERE is_funny = 0
  AND ( lower(text) LIKE '%remove a %counter%'
     OR lower(text) LIKE '%remove an %counter%'
     OR lower(text) LIKE '%remove all %counter%'
     OR lower(text) LIKE '%remove one or more %counter%'
     OR lower(text) LIKE '%remove two %counter%'
     OR lower(text) LIKE '%remove three %counter%'
     OR lower(text) LIKE '%remove four %counter%'
     OR lower(text) LIKE '%remove five %counter%'
     OR lower(text) LIKE '%remove six %counter%'
     OR lower(text) LIKE '%remove seven %counter%'
     OR lower(text) LIKE '%remove up to %counter%'
     OR lower(text) LIKE '%remove x %counter%'
     OR lower(text) LIKE '%remove any number of %counter%'
     OR lower(text) LIKE '%remove % counters from among%' )
  -- exclude DFC transform/flip bookkeeping ("...remove those counters and transform/untap it")
  AND lower(text) NOT LIKE '%remove those counters%'
  -- exclude time-counter / suspend / vanishing payoffs (not counter-removal interaction)
  AND lower(text) NOT LIKE '%time counter%'
  AND edhrec_rank IS NOT NULL
ORDER BY edhrec_rank
LIMIT 20;
```

**Why it works:** It matches the counter-removal idiom by requiring "remove" immediately followed by a quantity word (a/an/all/two/three/up to/x/that/those...) and then "counter", which is exactly how Oracle text phrases removing counters from a permanent. NOT-LIKE on "time counter" strips out Suspend/Vanishing flavor, and ordering by edhrec_rank (with IS NOT NULL) surfaces real, played staples first.

**Sample hits:** Walking Ballista, Wishclaw Talisman, Dragon's Hoard, Tekuthal, Inquiry Dominus, Dark Depths, Glissa Sunslayer, Runaway Steam-Kin, Mikaeus, the Lunarch, Vat of Rebirth, Steelbane Hydra, Golgari Grave-Troll, Scholar of New Horizons …

**Watch out:** FTS is useless here: the +1/+1 and -1/-1 tokens get stripped of punctuation, so you MUST use LIKE on the raw text column, not cards_fts MATCH. Do NOT use a single greedy pattern like '%remove%counter%' — the wildcards span sentence boundaries and produce false positives (e.g. Wash Away matched 'remove the words... / Counter target spell'); keeping the quantity word adjacent ('remove a %counter') fixes this. lower() is needed because Oracle text capitalizes "Remove" at the start of an ability but not mid-sentence. Add WHERE edhrec_rank IS NOT NULL or NULLs sort first and bury the real cards; drop that clause (and the ORDER BY) if you want the full ~561-row universe including obscure cards. Two design choices the author should know: (1) suspend/vanishing "remove a time counter" cards (Delay, Search for Tomorrow, the Overlords) are technically counter-removal but are timing flavor, so they are filtered out here — remove the two time-counter NOT-LIKE lines to include them; (2) named-counter activation costs (charge/oil/gold/wish counters) ARE included since they remove counters from a permanent, which is correct for the archetype but may be broader than "interactive removal" if you only want generic/+1+1 counters. Minor over-match: a card can hit via parenthetical reminder text (Diamond City's shield-counter reminder) rather than its own effect. One row per FACE means a multi-face card whose removal lives on one face still matches via shared full text; display_name disambiguates.

**Known false positives to exclude:** Brass's Tunnel-Grinder, Primal Amulet, Treasure Map, Edgar Markov's Coffin, Prize Pig, Hostile Hostel, Cursed Recording, Fury Charm …

---

## Move counters between permanents

**Ask:** Which cards can move counters from one permanent onto another (counter-relocation effects like Bioshift, Graft, Modular, and proliferate-payoff movers)?

_✓ runs, 6+ hits_

```sql
SELECT c.display_name, c.type, c.mana_cost, c.edhrec_rank, c.text
FROM cards c
WHERE c.is_funny = 0
  AND (
       replace(lower(c.text), 'remove', 'xxxxxx') LIKE '%move %counter% from %onto %'
    OR replace(lower(c.text), 'remove', 'xxxxxx') LIKE '%move %counter%onto another%'
    OR replace(lower(c.text), 'remove', 'xxxxxx') LIKE '%move %counter%onto target%'
    OR replace(lower(c.text), 'remove', 'xxxxxx') LIKE '%move %counter%onto each%'
    OR replace(lower(c.text), 'remove', 'xxxxxx') LIKE '%move %counter%onto a second%'
    OR c.id IN (SELECT card_id FROM card_keywords WHERE keyword IN ('Graft', 'Modular'))
  )
ORDER BY (c.edhrec_rank IS NULL), c.edhrec_rank
LIMIT 50;
```

**Why it works:** It captures both the explicit oracle idiom ("move ... counter ... from X onto Y / onto another / onto target / onto each") and the two evergreen keyword mechanics that move counters between permanents (Graft moves +1/+1 counters onto entering creatures; Modular moves them onto another artifact creature on death). Ranking by edhrec_rank (NULLs last) surfaces real staples like The Ozolith, Forgotten Ancient, Nesting Grounds, and Bioshift first.

**Sample hits:** The Ozolith, Forgotten Ancient, Nesting Grounds, Resourceful Defense, Llanowar Reborn (Graft), Slippery Bogbonder, Arcbound Ravager (Modular), Tidus, Yuna's Guardian, Rikku, Resourceful Guardian, Goldberry, River-Daughter, Plaxcaster Frogling (Graft), Scrapyard Recombiner (Modular) …

**Watch out:** Biggest trap: the substring "move " lives inside "reMOVE", so a naive `text LIKE '%move %counter% from %onto%'` falsely matches cards that say "Remove three counters from..." then later "...put them onto the battlefield" (e.g. Khalni Heart Expedition, Scholar of New Horizons, Vexing Puzzlebox). Neutralize this with replace(lower(text),'remove','xxxxxx') BEFORE the LIKE. FTS is useless here: the tokenizer drops the '+1/+1' / '-1/-1' symbols and can't enforce the "from...onto" adjacency, and broad MATCH 'move counter' would still catch "remove counter" — use LIKE on raw text. The wildcard between "counter" and "onto" still spans sentence boundaries within a single card, so anchoring on "onto" variants (onto another/target/each) keeps it tighter than a bare "%onto%". Don't forget the keyword tables: Graft/Modular cards (Arcbound family, Simic graft creatures) often phrase the movement inside reminder text and would otherwise be missed or, for Modular, the move happens via the death trigger. is_funny=0 excludes Un-set jokes (e.g. Everythingamajig/Giant Fan, which genuinely move counters but aren't legal). Also remember one-row-per-face: select display_name, not name, and a rare double-faced mover could appear twice if both faces match.

---

## Proliferate or double counters

**Ask:** Which playable cards let me proliferate or double counters (for +1/+1 stacking, planeswalker loyalty, charge counters, etc.)?

_✓ runs, 6+ hits_

```sql
SELECT display_name, edhrec_rank,
  CASE WHEN keywords LIKE '%Proliferate%' THEN 'proliferate' ELSE 'double-counters' END AS effect,
  substr(replace(text, char(10), ' '), 1, 80) AS snippet
FROM cards
WHERE is_funny = 0
  AND edhrec_rank IS NOT NULL
  AND (
        keywords LIKE '%Proliferate%'
     OR text LIKE '%double the number of%counter%'
     OR text LIKE '%doubles the number of%counter%'
     OR text LIKE '%twice that many %counters are put%'
     OR text LIKE '%twice that many of those counters%'
     OR text LIKE '%put twice that many %counter%'
  )
ORDER BY edhrec_rank
LIMIT 30;
```

**Why it works:** Proliferate is a true keyword, so the Proliferate branch is caught cleanly via keywords LIKE (the most reliable signal). Counter-doubling is NOT a keyword, so it is matched on oracle-text phrasings ("twice that many ... counters", "double the number of ... counter"), each constrained to require the word "counter" so token-only doublers are excluded.

**Sample hits:** Doubling Season, Karn's Bastion, Branching Evolution, Evolution Sage, Innkeeper's Talent, Cankerbloom, Yawgmoth, Thran Physician, Thrummingbird, Mossborn Hydra, The Earth Crystal, Bristly Bill, Spine Sower, Tezzeret's Gambit …

**Watch out:** FTS is useless for the doubling side: the tokenizer drops "+1/+1" punctuation, so you cannot MATCH on counter symbols; phrase-LIKE on raw text is required. "Proliferate" being a keyword is the one clean handle here, so prefer keywords LIKE '%Proliferate%' over text LIKE '%proliferate%' (the latter also catches reminder text). Biggest over-match trap: token doublers (Mondrak, Glory Dominus; Elspeth, Storm Slayer) and Doubling-style cards share the exact phrase "twice that many of those tokens" - you MUST anchor the doubling patterns to the word "counter", or they leak in. Note Doubling Season DOES belong: it doubles both tokens AND counters, and its counter clause reads "puts twice that many of those counters", which is why a dedicated 'twice that many of those counters' pattern is needed (its wording differs from "counters would be put on"). Cards like Primal Vigor double both tokens and counters and correctly stay in via their counter clause. edhrec_rank is NULL for many cards and NULLs sort FIRST, so WHERE edhrec_rank IS NOT NULL is mandatory to keep popular cards on top; is_funny=0 drops joke sets. Multi-face note: text is per-face, so a doubling clause on only one face still matches that row - fine here. The "double the number of ... counter" pattern also catches conditional one-shot doublers (Fangs of Kalonia, Invigorating Surge, Vorel) which are legitimately in-archetype.

---

## Coin-flip cards that don't suck

**Ask:** Which coin-flip cards are actually worth playing — show me the real, popular ones (no un-set jokes), ranked by how commonly they're played?

_✓ runs, 6+ hits_

```sql
SELECT display_name,
       type,
       edhrec_rank,
       substr(text, 1, 70) AS text_snippet
FROM cards
WHERE is_funny = 0
  AND edhrec_rank IS NOT NULL
  AND (
        text LIKE '%flip a coin%'
     OR text LIKE '%flips a coin%'
     OR text LIKE '%flip two coins%'
     OR text LIKE '%flip three coins%'
     OR text LIKE '%flip five coins%'
     OR text LIKE '%flip that many coins%'
     OR text LIKE '%flip one or more coins%'
     OR text LIKE '%flip coins%'
     OR text LIKE '%coin flip%'
     OR text LIKE '%win the flip%'
     OR text LIKE '%lose the flip%'
     OR text LIKE '%win those flips%'
     OR text LIKE '%win that flip%'
  )
GROUP BY display_name
ORDER BY edhrec_rank
LIMIT 20;
```

**Why it works:** Coin-flip rules text in MTG is overwhelmingly phrased as "flip a coin", "win/lose the flip", or "win a coin flip", so a set of LIKE patterns on the raw text column captures the mechanic precisely. is_funny=0 drops the Un-set joke cards (Mana Screw, Everythingamajig, etc.), and edhrec_rank IS NOT NULL + ORDER BY edhrec_rank surfaces the genuinely played staples (Mana Crypt, Krark, Krark's Thumb, Chance Encounter) instead of obscure or fake cards.

**Sample hits:** Mana Crypt, Invert Polarity, Krark, the Thumbless, Ral, Monsoon Mage, The Gold Saucer, Mirror March, Tavern Scoundrel, Krark's Thumb, Stitch in Time, Boompile, Ral Zarek, Rakdos, the Showstopper …

**Watch out:** FTS cannot reliably do this: the tokenizer would let 'flip coin' match but it drops the exact adjacency/phrasing and you'd miss variants like "win the flip" vs "win a coin flip" — LIKE on raw text is the right tool. Multiple LIKE branches are needed because some payoff cards say "Whenever you win a coin flip" and never literally say "flip a coin". NULL edhrec_rank sorts FIRST in ascending order, so the WHERE edhrec_rank IS NOT NULL guard is mandatory or you get unplayed cards at the top. The cards table is one row per face, so split/legendary cards (Krark, Stitch in Time) appear twice — GROUP BY display_name dedupes them; without it counts and limits are skewed. is_funny=1 cards all have NULL edhrec_rank anyway, but excluding is_funny explicitly is cleaner and avoids un-set jokes if any ever get a rank. Watch for slight over-matching: "win the flip"/"lose the flip" could in theory match a non-coin "flip" effect, but in practice all such text in this DB is coin-flip related.

**Known false positives to exclude:** Flick a Coin (NOT a coin-flip card despite its name and a Treasure-token payoff; it is a damage spell whose name merely contains 'Coin' — correctly EXCLUDED by both original and corrected queries because it never matched a flip pattern), Athreos, Shroud-Veiled / Noble's Purse / Wishing Well / Ral Zarek, Guest Lecturer's other modes — these use 'coin' for COIN COUNTERS, not coin flips; a naive text LIKE '%coin%' would wrongly include them, so both queries correctly avoid that broad pattern

---

## Roll dice (d20 / planar)

**Ask:** Which cards roll dice — d20s, polyhedral dice, six-sided dice, or the planar die — and which are the most-played ones for a "dice matters" deck?

_✓ runs, 6+ hits_

```sql
SELECT display_name, mana_cost, type, edhrec_rank
FROM cards
WHERE (
        text LIKE '%roll a d%'              -- "roll a d20", "roll a d10" (D&D polyhedral)
     OR text LIKE '%roll a die%'            -- singular "roll a die"
     OR text LIKE '%-sided di%'             -- "six-sided die" AND "six-sided dice" (one clause)
     OR text LIKE '%roll one or more di%'   -- "roll one or more dice"/"...planar dice" payoffs
     OR text LIKE '%roll two d%'            -- "roll two d8"
     OR text LIKE '%roll X d%'              -- "Roll X six-sided dice"
     OR text LIKE '%roll the planar die%'   -- actively roll the planar die
     OR text LIKE '%planar dice%'           -- plural planar dice (Ichor Elixir, Pixie-style)
     OR text LIKE '%you roll a %'           -- "whenever/when you roll a 6/natural 20" payoffs
     OR text LIKE '%natural 20%'            -- Critical Hit etc.
      )
  AND is_funny = 0                          -- legal/black-border only; correctly keeps Unfinity acorn dice cards (is_funny=0)
ORDER BY edhrec_rank IS NULL, edhrec_rank   -- popular first; NULL ranks LAST
LIMIT 20;
```

**Why it works:** It OR-unions every die-rolling phrasing MTG uses (D&D "roll a d20/d4", "six-sided die", "roll two d8", "roll X ... dice", "roll one or more dice", and the Planechase "planar die"), so it catches both the die-rollers and the dice-matters payoffs. is_funny=0 drops Un-set jokes, and the NULL-safe ORDER BY surfaces real staples (the Ancient Dragon cycle, Delina Wild Mage, Wand of Wonder) at the top.

**Sample hits:** Ancient Copper Dragon (645) - d20 on combat damage, Ancient Silver Dragon (1399) - d20, Delina, Wild Mage (1652) - d20 attack copy, Ancient Brass Dragon (1670), Ancient Gold Dragon (2063), Wand of Wonder (2348) - d20, Ancient Bronze Dragon (2580), Lae'zel's Acrobatics (2642) - d20, Wyll's Reversal (2721), Hoarding Ogre (3469) - d20 attack, Reckless Endeavor (3525), Vexing Puzzlebox (3549) …

**Watch out:** FTS is useless for the canonical token here: 'd20', 'd4', 'd8' survive the tokenizer but '+1/+1'-style symbols don't, and matching the bare word 'roll' in FTS pulls in "reroll/payroll"-type noise and rules text like "roll the dice" loosely — LIKE on raw text is more precise. Two distinct eras of wording exist: D&D/AFR cards say "roll a d20" while older/Unfinity cards say "roll a six-sided die", so a single pattern under-matches — you must union both (the '-sided die' and '-sided dice' forms). 'planar die' (Planechase) is a different object than dice you "roll a d", so only include it if you want planar-roll cards. Watch over-matching: 'planeswalk' (1085 rows) and 'planeswalker' will swamp results if you LIKE '%planar%' or '%plane%' loosely — anchor to '%planar die%'. Always use 'ORDER BY edhrec_rank IS NULL, edhrec_rank' (or WHERE edhrec_rank IS NOT NULL) because NULL ranks sort FIRST by default and would bury the staples. Multi-face cards are not a concern for this archetype (the dice mechanic only appears on single-faced cards, verified zero duplicate names), but use display_name in output regardless.

**Known false positives to exclude:** None in the result set. All 12 cards returned by the ORIGINAL query genuinely roll dice; all 20 in the corrected query are genuine rollers or dice-matters payoffs., Note on borderline cases (in-scope, not junk): Night Shift of the Living Dead ('after you roll a die') and several Unfinity Attraction cards (Lifetime Pass Holder, Complaints Clerk, Ferris Wheel, Dee Kay) are dice PAYOFFS / 'roll-to-visit' rollers rather than the marquee d20/six-sided rollers - legitimately fit a 'dice matters' deck but are softer., Note: planar PAYOFF cards (Stairs to Infinity, Ten Wizards Mountain, The Drum, sAnS mERcY, Chaotic Aether, Fixed Point in Time) match 'planar die'/'planar dice' yet only care about the roll in the Planechase variant rather than rolling it in normal play - acceptable for the archetype but not constructible dice-deck rollers.

---

## Self-mill / fill your own graveyard

**Ask:** Which cards mill ME / fill my own graveyard for value (self-mill payoffs and enablers like mill-N-cards effects and Dredge)?

_✓ runs, 6+ hits_

```sql
SELECT display_name, type, mana_value, edhrec_rank
FROM cards
WHERE is_funny = 0 AND edhrec_rank IS NOT NULL
  AND (
        text LIKE '%you mill %'
     OR text LIKE '%you may mill %'
     OR keywords LIKE '%Dredge%'
     OR (
          text LIKE '%mill % card%'
          AND text NOT LIKE '%target player mill%'
          AND text NOT LIKE '%target opponent%mill%'
          AND text NOT LIKE '%that player mill%'
          AND text NOT LIKE '%each player mill%'
          AND text NOT LIKE '%players mill%'
          AND text NOT LIKE '%defending player mill%'
          AND text NOT LIKE '%opponent%mill%'
          AND text NOT LIKE '%have target%mill%'
          AND text NOT LIKE '%they mill%'
        )
      )
  AND NOT (text LIKE '%may have you draw%' AND text LIKE '%you mill%' AND text LIKE '%total mana value%')
ORDER BY edhrec_rank
LIMIT 20;
```

**Why it works:** Self-mill means filling YOUR OWN graveyard, so the query keys on the subjectless "mill N cards" template (whose default subject is the controller = you) plus explicit "you mill"/"you may mill" phrasings, and adds the Dredge keyword (a pure self-mill mechanic). It then excludes opponent-directed mill ("target player mills", "opponent mills", "each player mills") so the deck-out / mill-the-opponent archetype doesn't leak in. Ordering by edhrec_rank surfaces real staples (Stitcher's Supplier, Emry, Life from the Loam, Aftermath Analyst).

**Sample hits:** Takenuma, Abandoned Mire, Ripples of Undeath, Six, Emry, Lurker of the Loch, Stitcher's Supplier, Aftermath Analyst, Life from the Loam, Icetill Explorer, Lumra, Bellow of the Woods, Millikin, Dakmor Salvage, Hedge Shredder …

**Watch out:** FTS is useless for the discriminating phrasing here: the tokenizer drops "//" and treats words separately, so it can't tell "you mill" from "opponent mills" or keep multi-word phrases intact -- LIKE on raw text is required. NULL edhrec_rank sorts FIRST in ascending order, so WHERE edhrec_rank IS NOT NULL is mandatory or you get a wall of unplayed obscurities on top. The biggest semantic trap is that plain "mill three cards" with no subject means YOU, but "target player mills three"/"each opponent mills" is the OPPOSITE archetype (deck your opponent out) -- you must explicitly exclude player/opponent-mill phrasings or you over-match. A few borderline cards remain (e.g. Combustible Gearhulk "target opponent may have you mill your library", Liliana's Indignation "Mill X cards" then target a player) -- these genuinely mill YOU so keeping them is correct, but tightening further would wrongly drop them. Surveil (also fills your graveyard) and the Dredge keyword aren't caught by the "mill" text patterns, so Dredge is added via keywords LIKE -- note multi-face cards are one row per face, so a back face's mill text won't appear on the front face's row.

**Known false positives to exclude:** Combustible Gearhulk, Palantír of Orthanc, Bruvac the Grandiloquent, Jace's Erasure, Iceberg Cancrix, Extractor Demon, Demogorgon's Clutches, Mind Drain …

---

## Trigger on gaining life

**Ask:** Which cards have an ability that triggers "whenever you gain life" (lifegain payoffs)?

_✓ runs, 6+ hits_

```sql
SELECT display_name, mana_cost, type, edhrec_rank, substr(text, 1, 70) AS snippet
FROM cards
WHERE (text LIKE '%whenever you gain life%' OR text LIKE '%whenever you gain or lose life%')
  AND is_funny = 0
ORDER BY edhrec_rank IS NULL, edhrec_rank
LIMIT 15;
```

**Why it works:** The LIKE pattern matches the exact reminder/oracle phrasing "Whenever you gain life..." that defines a lifegain trigger, plus the "whenever you gain or lose life" variant (Moonstone Harbinger, Wax-Wane Witness). Ordering by edhrec_rank surfaces the genuine lifegain-matters staples first. There are 90 such cards total in the DB.

**Sample hits:** Vito, Thorn of the Dusk Rose, Sanguine Bond, Enduring Tenacity, Heliod, Sun-Crowned, Marauding Blight-Priest, Well of Lost Dreams, Exemplar of Light, Cleric Class, Archangel of Thune, Elenda's Hierophant, Starscape Cleric, Dawn of Hope …

**Watch out:** Use LIKE, not FTS: the FTS tokenizer would split "gain life" loosely and rank-match unrelated cards; the literal triggered-ability phrase is what makes a card a payoff, and LIKE is case-insensitive so '%whenever you gain life%' catches both "Whenever..." and lowercase mid-sentence uses. Do NOT just match '%gain%life%' — that catches 2400+ cards (any card that itself gains life, e.g. lifelink/"gain N life", which are SOURCES not PAYOFFS). The "whenever you gain or lose life during your turn" variant (2 cards) is genuinely a lifegain trigger and worth including; if you want only pure lifegain you can drop it. Add 'WHERE edhrec_rank IS NULL, edhrec_rank' via the IS NULL sort key because NULL edhrec_rank sorts first by default and would bury staples behind obscure cards. is_funny=0 excludes Un-set jokes. Multi-face text is '/'-joined in the text column, but all 90 matches here are single-faced so no name-collision dedup is needed; if it mattered, GROUP BY name or use DISTINCT name.

---

## Sacrifice-for-value & death triggers

**Ask:** What are the best aristocrats / sacrifice-for-value cards — both the "whenever a creature you control dies" payoffs and the sacrifice outlets that feed them?

_✓ runs, 6+ hits_

```sql
SELECT display_name, edhrec_rank,
  CASE
    WHEN text LIKE '%creature you control dies%'
      OR text LIKE '%another creature dies%'
      OR text LIKE '%whenever a creature dies%'
      OR text LIKE '%whenever you sacrifice a creature%'
      OR text LIKE '%whenever you sacrifice another creature%'
      OR text LIKE '%player sacrifices a permanent%'
      OR text LIKE '%player sacrifices another permanent%'
      OR text LIKE '%whenever you sacrifice a permanent%'
      OR text LIKE '%create a Treasure token for each creature that died%'
    THEN 'death/sac-payoff'
    ELSE 'sac-outlet'
  END AS role,
  type
FROM cards
WHERE
  (
    -- self-sacrifice OUTLETS: activated-ability cost (colon) or additional-cost (period)
    text LIKE '%Sacrifice a creature:%'
    OR text LIKE '%Sacrifice another creature:%'
    OR text LIKE '%, sacrifice a creature.%'
    OR text LIKE '%, sacrifice another creature.%'
    OR text LIKE '%Sacrifice a creature or artifact:%'
    OR text LIKE '%Sacrifice another creature or artifact:%'
    OR text LIKE '%Sacrifice another creature, artifact%'
    -- death / your-sacrifice PAYOFFS (generic creature; token-typed sac excluded)
    OR text LIKE '%creature you control dies%'
    OR text LIKE '%another creature dies%'
    OR text LIKE '%whenever a creature dies%'
    OR text LIKE '%whenever you sacrifice a creature%'
    OR text LIKE '%whenever you sacrifice another creature%'
    OR text LIKE '%player sacrifices a permanent%'
    OR text LIKE '%player sacrifices another permanent%'
    OR text LIKE '%whenever you sacrifice a permanent%'
    OR text LIKE '%create a Treasure token for each creature that died%'
  )
  AND edhrec_rank IS NOT NULL
  AND is_funny = 0
ORDER BY edhrec_rank
LIMIT 45;
```

**Why it works:** It unions the two halves of the archetype: death-payoffs that trigger on "creature you control dies" (Zulaport Cutthroat, Grave Pact, Dictate of Erebos) and the sacrifice outlets that feed them via comma-activated abilities or additional costs (Village Rites, Phyrexian Tower, High Market, Yawgmoth). The CASE column tags each row's role, and ordering by edhrec_rank surfaces the real staples first.

**Sample hits:** Ashnod's Altar (sac-outlet), Blood Artist (death-payoff), Village Rites (sac-outlet), Phyrexian Tower (sac-outlet), Pitiless Plunderer (death-payoff), Zulaport Cutthroat (death-payoff), Syr Konrad, the Grim (death-payoff), Viscera Seer (sac-outlet), Phyrexian Altar (sac-outlet), Bastion of Remembrance (death-payoff), Warren Soultrader (sac-outlet), Goblin Bombardment (sac-outlet) …

**Watch out:** Must use LIKE, not FTS: the FTS tokenizer drops punctuation, so the load-bearing comma in '{cost}, Sacrifice a creature:' is unsearchable via MATCH. Always keep edhrec_rank IS NOT NULL — NULLs sort FIRST in 'ORDER BY edhrec_rank' and would bury staples under thousands of unranked cards; also is_funny=0 to drop Un-set jokes. The '%, Sacrifice a creature%' pattern intentionally also catches additional-cost spells (Eldritch Evolution, Culling the Weak) because their oracle text reads 'this spell, sacrifice a creature' — fine for an outlet list but note some are really tutors/ritual, not pure aristocrats. Multi-face cards (Ayara, Widow of the Realm // Ayara, Furnace Queen) appear once per matching face since it's one row per face; display_name is correct to show. Beware: '%Sacrifice a creature%' WITHOUT the leading comma would over-match symmetric edicts an opponent does ('each opponent sacrifices a creature'); requiring the comma or 'another creature' keeps it to YOUR sac outlets. To exclude pure additional-cost tutors, add: AND text NOT LIKE '%additional cost to cast%'.

**Known false positives to exclude:** Massacre Girl (front face) - matches 'whenever a creature dies this turn' but is a one-shot board-wipe trigger, not a repeatable aristocrats value engine; death-MATTERS but functions as removal, Tireless Tracker / Captain Lannery Storm / Nuka-Cola Vending Machine - caught only by the over-broad 'whenever you sacrifice a' pattern I tested; they pay off sacrificing Clue/Treasure/Food TOKENS, not creatures. EXCLUDED in final query by narrowing to 'sacrifice a creature/permanent', Tragic Slip - 'if a creature died this turn' is a Morbid removal spell, not an engine. EXCLUDED in final by dropping the bare 'died this turn' pattern, Fling / Kazuul's Fury - additional-cost burn that sacrifices a creature; technically a sac outlet but fringe (one-shot, not a repeatable engine). Kept (defensible) but low-value as aristocrats core, Eldrazi Monument - 'sacrifice a creature' is a forced upkeep DRAWBACK, not a chosen outlet; included but edge-case, Wight of the Reliquary / Eldritch Evolution / Neoform / Birthing Pod / Disciple of Bolas - real sacrifice effects but tutors/toolbox, not aristocrats death-value; technically valid sac outlets, contextually tangential

---

## Steal / gain control of permanents

**Ask:** Which popular cards let me steal an opponent's permanent by gaining control of it?

_✓ runs, 6+ hits_

```sql
SELECT display_name, type, mana_cost, mana_value, color_identity, edhrec_rank
FROM cards
WHERE text LIKE '%gain control of%'
  AND text NOT LIKE '%gain control of target opponent%'        -- excludes Emrakul (controls the PLAYER, not a permanent)
  AND text NOT LIKE '%player gains control%'
  AND text NOT LIKE '%opponent gains control%'
  AND text NOT LIKE '%players gain control%'
  AND text NOT LIKE '%opponents gain control%'
  AND text NOT LIKE '%player gain control of%'                 -- excludes give-away cards (Slicer, Assault Suit, Tahngarth)
  AND text NOT LIKE '%players gain control of%'
  AND text NOT LIKE '%opponent gain control of%'               -- excludes Iroh (you give a permanent to opponent)
  AND text NOT LIKE '%opponents gain control of%'
  AND text NOT LIKE '%gain control of it. If you do, it''s goaded%'  -- excludes Vislor Turlough (give-away)
  AND is_funny = 0
  AND edhrec_rank IS NOT NULL
ORDER BY edhrec_rank
LIMIT 30;
```

**Why it works:** The phrase "gain control of" is the canonical Oracle wording for theft, covering "gain control of target permanent/creature/artifact", plus mass-steal variants ("gain control of all creatures", "...of them", "...of that artifact"). Ordering by non-null edhrec_rank floats real, played staples to the top.

**Sample hits:** Hellkite Tyrant, Treasure Nabber, Seize the Spotlight, Archmage's Charm, Insurrection, Agent of Treachery, Thieving Skydiver, Commandeer, Zealous Conscripts, Emrakul, the World Anew, Captivating Vampire, Expropriate …

**Watch out:** The opposite effect (donate/give-away) shares vocabulary: cards where an OPPONENT or PLAYER "gains control of" your stuff (Akroan Horse, Humble Defector, Yes Man, Homeward Path, Jon Irenicus) are false positives — the four NOT LIKE clauses on "player(s)/opponent(s) gain(s) control" filter them out. Do NOT restrict to "gain control of target": that misses Insurrection, Expropriate, Mob Rule, Reins of Power and other mass/temporary steals. Avoid FTS here — its tokenizer treats "gain control of" as separate words and would match unrelated text mentioning each word; LIKE on the raw text column keeps the phrase intact. edhrec_rank can be NULL and NULLs sort first, so WHERE edhrec_rank IS NOT NULL is required to get popular cards. Note Sorin/Dihada/Tevesh planeswalkers match because a loyalty ability or unrelated line says "gain control"; that's acceptable but watch for incidental mentions. Multi-face note: the cards table is one row per face, so a steal effect on a single face (e.g. Slicer, Hired Muscle's transformed side) is still captured per-row.

**Known false positives to exclude:** Emrakul, the Promised End, Slicer, Hired Muscle

---

## Clone & copy effects

**Ask:** Which cards let me clone or copy a creature or other permanent (Clone-style "enters as a copy" effects, "copy of target" spells, and token-copy effects), ranked by how commonly they're played?

_✓ runs, 6+ hits_

```sql
SELECT display_name, type, mana_cost, edhrec_rank
FROM cards
WHERE is_funny = 0
  AND edhrec_rank IS NOT NULL
  AND (
        text LIKE '%as a copy of%'                 -- Clone-style ETB (Clone, Phantasmal Image, Phyrexian Metamorph, Vesuva...)
     OR text LIKE '%as copies of%'                 -- plural ETB (Mystic Reflection)
     OR text LIKE '%becomes a copy of%'            -- Thespian's Stage, Mirage Mirror, Lazav, Sakashima the Impostor
     OR text LIKE '%become a copy of%'
     OR text LIKE '%become copies of%'             -- Brudiclad, Deceiver of Form, Niko, Absorb Identity
     OR text LIKE '%token that''s a copy of%'      -- Rite of Replication, Kiki-Jiki, Helm of the Host, Saheeli's Artistry
     OR text LIKE '%tokens that are copies of%'    -- Doppelgang, Twinflame, Esix, Saw in Half
     OR text LIKE '%token copy of%'                -- Offspring mechanic + 'create a 1/1 token copy of it'
     OR text LIKE '%copies of that %'              -- Saw in Half, Second Harvest variants
     OR text LIKE '%copy of that permanent%'       -- Second Harvest
      )
  AND text NOT LIKE '%copy of its spell%'          -- exclude Prepared mechanic (cast a copy of its spell)
ORDER BY edhrec_rank ASC;
```

**Why it works:** LIKE on the raw text column unions the three canonical clone-copy phrasings: "as a copy of" (permanent clones like Clone/Phyrexian Metamorph/Vesuva/Thespian's Stage that enter or become a copy), "copy of target/another/any" (token-copy and on-cast copies like Rite of Replication, Kiki-Jiki, Cryptoplasm). Filtering edhrec_rank IS NOT NULL and ordering ascending surfaces real, played staples first; is_funny=0 drops Un-set jokes.

**Sample hits:** Scute Swarm, Phyrexian Metamorph, Helm of the Host, Spark Double, Fanatic of Rhonas, Cursed Mirror, Thespian's Stage, Second Harvest, Saw in Half, Sculpting Steel, Springheart Nantuko, Mockingbird …

**Watch out:** FTS is wrong here: the tokenizer drops punctuation and would split phrases, and the distinction between "copy of target CREATURE" vs "copy of target SPELL" depends on adjacent words, so use LIKE on raw text, not cards_fts. Spell-copy/storm cards ("create a copy of target spell") are intentionally NOT matched because none of the four patterns hit "copy of target spell" — good, since spell-copy is a different archetype. edhrec_rank is NULL for unranked cards and NULLs sort FIRST, so the IS NOT NULL guard is mandatory. Multi-face/printing duplication: Sakashima of a Thousand Faces appears twice (one 'normal' row, one 'reversible_card' row sharing the same face name) — dedupe on display_name or scryfall_oracle_id if you need unique cards. Note the table is one row per card FACE, so always display_name in output, never face-level name fragments.

**Known false positives to exclude:** Offspring-mechanic creatures (e.g. Coruscation Mage, Agate Instigator, Starscape Cleric, Darkstar Augur, Tender Wildguide, Splash Lasher, Warren Warleader) — they DO create a token copy, but it's a reduced 1/1 copy of themselves bolted onto a value creature; borderline/peripheral to the 'clone-copy' intent rather than core. Included in the final query (defensibly, since they make a copy token) but flagged as the main noise source., Note on the PROPOSED (original) query: it had ZERO hard false positives among its 188 hits (no spell-copy leaks) — every top result was a real clone/copy effect. Its flaw was recall, not precision.

---

## Extra turns

**Ask:** Which cards let you (or a player) take an extra turn?

_✓ runs, 6+ hits_

```sql
SELECT c.display_name, c.mana_cost, c.type, c.edhrec_rank
FROM cards c
WHERE c.is_funny = 0
  AND (
        c.text LIKE '%take an extra turn%'
     OR c.text LIKE '%takes an extra turn%'
     OR c.text LIKE '%take two extra turns%'
     OR c.text LIKE '%takes two extra turns%'
  )
  AND c.types NOT IN ('Scheme','Plane','Phenomenon','Vanguard','Dungeon')
GROUP BY COALESCE(c.scryfall_oracle_id, c.display_name)
ORDER BY c.edhrec_rank IS NULL, c.edhrec_rank;
```

**Why it works:** The defining mechanic uses the exact oracle idiom "take(s) an/two extra turn(s)", so matching that phrase on the raw text column captures the archetype precisely. The two NOT LIKE clauses strip out turn-DENIAL cards (Ugin's Nexus, Stranglehold) whose text reads "...that player skips that turn instead", which otherwise contain the literal substring "extra turn".

**Sample hits:** Time Vault, Time Warp, Temporal Manipulation, Capture of Jingzhou, Walk the Aeons, Time Stretch, Time Walk, Temporal Mastery, Temporal Trespass, Nexus of Fate, Alrund's Epiphany, Expropriate …

**Watch out:** Use LIKE on raw text, not FTS: bare FTS MATCH 'extra turn' also pulls denial cards (Ugin's Nexus, Stranglehold), Archenemy Schemes ('All in Good Time'), and unrelated noise like 'Grow Extra Arms' (extra arms, not turns). You must handle both "take an extra turn" (you) and "takes an extra turn" (target player), plus the plural "two extra turns" (Karn's Temporal Sundering / Time Stretch). The NOT LIKE '%skips that turn%' / '%skip that turn%' exclusion is essential to drop anti-extra-turn cards. NULLs in edhrec_rank sort first by default, so the ORDER BY uses "edhrec_rank IS NULL" first to push unranked cards to the bottom while keeping them in the list. Multi-face cards (e.g. the Mu Yanling planeswalker, and double-listed "Stitch in Time" across printings) can appear once per matching face/printing, so expect a few near-duplicate display_names; SELECT DISTINCT display_name or grouping by scryfall_oracle_id would dedupe if needed.

**Known false positives to exclude:** Strike the Weak Spot — gives the HYDRA (a boss/enemy) an extra turn; it is a downside, not a benefit to you, and it is a special non-constructed card., Perch Protection — 'Gift an extra turn' gives an OPPONENT an extra turn via the gift mechanic; the spell's actual payoff is making Birds, not granting yourself extra turns., Emrakul, the Promised End — the extra turn is given to the OPPONENT as a downside of its Mindslaver cast trigger; you play it for the 13/13 body, not to grant extra turns., (Original query only) Otaria [Plane], All in Good Time + Time Bends to My Will [Schemes], Frenetic Efreet Avatar [Vanguard] — non-deck card types removed by the fix.

---

## Token doublers & copy-token makers

**Ask:** What cards double the tokens I make or create a token that's a copy of something? (token doublers and clone-token makers)

_✓ runs, 6+ hits_

```sql
SELECT DISTINCT display_name, type, edhrec_rank
FROM cards
WHERE is_funny = 0 AND text IS NOT NULL
  AND (
    -- (A) token-MULTIPLIER replacement effects (token doublers)
    text LIKE '%twice that many of those tokens%'
    OR text LIKE '%it creates twice that many%'
    OR text LIKE '%twice that many tokens%'
    OR (text LIKE '%double the number of%' AND text LIKE '%tokens you control%')
    -- (B) clone-as-token makers
    OR text LIKE '%token that''s a copy%'
    OR text LIKE '%tokens that are copies%'
  )
  -- exclude Eternalize/Embalm self-recursion (creates a token copy of ITSELF from the
  -- graveyard via reminder text "Create a token that's a copy of it" -- graveyard
  -- recursion, NOT a token doubler or clone-token maker for the archetype)
  AND NOT (
    (text LIKE '%Eternalize {%' OR text LIKE '%Embalm {%')
    AND text NOT LIKE '%twice that many%'
    AND text NOT LIKE '%double the number of%'
    AND text NOT LIKE '%copy of target%'
    AND text NOT LIKE '%copy of enchanted%'
    AND text NOT LIKE '%copy of equipped%'
  )
ORDER BY (edhrec_rank IS NULL), edhrec_rank
LIMIT 30;
```

**Why it works:** It unions the two real sub-archetypes: (A) replacement-effect multipliers that turn N tokens into 2N via the printed Oracle phrasings ("twice that many of those tokens", "it creates twice that many", "double the number of ... tokens you control"), and (B) clone-on-a-token effects via "token that's a copy" / "tokens that are copies". Ordering by edhrec_rank (NULLs forced last) surfaces the real staples first.

**Sample hits:** Doubling Season, Scute Swarm, Anointed Procession, Helm of the Host, Parallel Lives, Mondrak, Glory Dominus, Second Harvest, Saw in Half, Caretaker's Talent, Springheart Nantuko, Elspeth, Storm Slayer, Rite of Replication …

**Watch out:** FTS is useless here: the tokenizer drops the apostrophe in "that's" and would treat "twice that many of those tokens" as loose word-soup, so a MATCH would over-match; LIKE on raw text is correct. The big trap is the bare "double" / "twice that many" phrase: cards that double COUNTERS or ENERGY (Aether Refinery doubles {E}, Arcade Cabinet and Big Mother Mouser double +1/+1 counters) also contain the word "token" in an unrelated clause, so a naive `text LIKE '%twice that many%' AND text LIKE '%token%'` leaks them in — that's why the multiplier patterns must keep "twice that many" adjacent to "tokens" (e.g. "twice that many of those tokens"), and the "double the number of" branch is pinned to "tokens you control". Escape the apostrophe in "that's" as '' inside LIKE. Use display_name (not name) because token-copy makers like Kiki-Jiki/Helm-of-the-Host are single-face but other matches could be multi-face — DISTINCT collapses the duplicate face rows you otherwise see (Anointed Procession and Adrix and Nev appear twice across printings/faces without it). Always force NULL edhrec_rank last with `ORDER BY (edhrec_rank IS NULL), edhrec_rank` since NULLs sort first by default and would bury the staples. The full set is ~385 names, dominated by minor "token copy" creatures; the tight pure-doubler core (Doubling Season, Anointed Procession, Parallel Lives, Primal Vigor, Mondrak, Adrix and Nev, Elspeth Storm Slayer, plus the Selesnya Loft Gardens plane) is only ~13 cards — raise the LIMIT or split branch (A) out if you want only the true doublers.

**Known false positives to exclude:** Fanatic of Rhonas, Timeless Witness, Adorned Pouncer, Angel of Sanctions, Anointer Priest, Champion of Wits, Earthshaker Khenra, Honored Hydra …

---

## Alternate win / can't-lose conditions

**Ask:** What are the most-played alternate win conditions and "you win / can't lose / opponent loses the game" cards?

_✓ runs, 6+ hits_

```sql
SELECT display_name, edhrec_rank,
  CASE
    WHEN text LIKE '%you can''t lose the game%' OR text LIKE '%opponents can''t lose the game%' THEN 'CANT-LOSE'
    WHEN text LIKE '%you win the game%' OR text LIKE '% wins the game%' THEN 'YOU/ALT-WIN'
    ELSE 'OPP-LOSES'
  END AS kind
FROM cards
WHERE is_funny = 0
  AND (
        text LIKE '%you win the game%'
     OR text LIKE '% wins the game%'
     OR text LIKE '%loses the game%'
     OR text LIKE '%you can''t lose the game%'
     OR text LIKE '%opponents can''t lose the game%'
  )
  AND text NOT LIKE '%poison counters loses the game%'
  -- drop value-engine cards that merely REACT to a loss
  AND text NOT LIKE '%Whenever a player loses the game, put%'
  AND text NOT LIKE '%Whenever another player loses the game%'
  AND text NOT LIKE '%whenever an opponent loses the game%'
  AND text NOT LIKE '%When enchanted player loses the game%'
ORDER BY (edhrec_rank IS NULL), edhrec_rank
LIMIT 30;
```

**Why it works:** It LIKE-matches the four canonical game-ending phrasings ("you win the game", "wins the game", "loses the game", "can't lose/win the game"), buckets each hit into WIN / NO-LOSE / LOSE-OPP, and ranks by EDHREC popularity so real staples surface first. The total unfiltered set is 77 cards; the LIMIT shows the top 25.

**Sample hits:** Thassa's Oracle, Laboratory Maniac, Hellkite Tyrant, Mechanized Production, Approach of the Second Sun, Jace, Wielder of Mysteries, Revel in Riches, Simic Ascendancy, Triskaidekaphile, Felidar Sovereign, Twenty-Toed Toad, Platinum Angel …

**Watch out:** FTS is useless here: 'win the game' and 'lose the game' contain no punctuation but the phrase logic plus the +1/+1-style symbols elsewhere make raw-column LIKE the safe choice, and FTS would not let you exclude the poison reminder cleanly. Biggest over-match trap: every infect/toxic card carries the reminder text "(A player with ten or more poison counters loses the game.)" — 16 cards — which is NOT an alt-win-con, so the explicit NOT LIKE '%poison counters loses the game%' is essential. Note Etali (real poison payoff, not just reminder) survives because its sentence differs. NULL edhrec_rank sorts FIRST by default, so the "(edhrec_rank IS NULL)" sort key pushes unranked/obscure cards to the bottom instead of dropping them entirely (don't use WHERE edhrec_rank IS NOT NULL here or you'd lose legitimate niche win-cons). Multi-face/text uses '/' or newline separators, so phrases never span faces — fine for LIKE. Cards like Pact of Negation use "you lose the game" purely as a downside/cost, not a win-con; this query intentionally does not match the bare "you lose the game" downside phrasing to avoid that noise. Use is_funny=0 to drop Un-set jokes like "Everybody Lives!" variants (the listed one is a real card).

**Known false positives to exclude:** Blood Tyrant, Sengir, the Dark Baron, Share the Spoils, Curse of Vengeance, Withengar Unbound

---

## Symmetric wheel / each-player draw

**Ask:** What are the classic "wheel" effects — symmetric spells/abilities where each player discards (or shuffles away) their hand and then draws a fresh hand of cards?

_✓ runs, 6+ hits_

```sql
SELECT display_name, mana_cost, type, edhrec_rank
FROM cards
WHERE is_funny = 0
  AND text LIKE '%each player%'
  AND (
        text LIKE '%discards their hand, then draws%'
     OR (text LIKE '%shuffles their hand%' AND text LIKE '%draws seven cards%')
     OR (text LIKE '%shuffles their hand%' AND text LIKE '%draws up to seven%')
     OR (text LIKE '%shuffle their hand%'  AND text LIKE '%draws seven cards%')
     OR (text LIKE '%exiles all cards from their hand%' AND text LIKE '%draws seven cards%')
     OR text LIKE '%draws cards equal to the greatest number of cards a player discarded%'
      )
ORDER BY edhrec_rank IS NULL, edhrec_rank;
```

**Why it works:** It keys on the symmetric "each player" wording paired with the three canonical wheel templates — discard-hand-then-draw (Wheel of Fortune/Reforge), shuffle-hand+graveyard-then-draw-seven (Timetwister/Echo of Eons), and discard-then-draw-equal-to-most-discarded (Windfall/Jace's Archivist). Sorting nudges popular EDHREC staples to the top.

**Sample hits:** Windfall, Wheel of Fortune, Magus of the Wheel, Wheel of Misfortune, Reforge the Soul, Jace's Archivist, Echo of Eons, Dragon Mage, Whispering Madness, Memory, Timetwister, Wheel of Fate …

**Watch out:** FTS is useless here: the tokenizer drops the slash in "Fire // Ice" style names and, more importantly, this archetype is defined by exact multi-word phrases, so LIKE on raw `text` is the right tool. Phrase choice matters — a loose pattern like just '%draws seven cards%' over-matches one-sided card draw and symmetric-but-not-wheel cards, while '%each player%discards their hand%' alone pulls in pure discard/hellbent enablers (Mindslicer, Sire of Insanity, Mindcrank) that never refill hands; requiring "...then draws..." in the same clause excludes those. The three OR branches cover the distinct wheel templates without dragging in Memory Jar (exile-and-return, different timing) unless you want it. Watch multi-face cards: "Wheel of Fortune" appears twice with the same display_name — one is the classic single-faced card (rank 560) and one is the back face of the DFC "Naktamun Lorespinner // Wheel of Fortune" (rank 11685); both are legitimately distinct, not a dedup bug, since the table is one row per face. As always, NULL edhrec_rank sorts first, so use `ORDER BY edhrec_rank IS NULL, edhrec_rank` (or add `WHERE edhrec_rank IS NOT NULL`) to keep unranked oddities like Wheel of Fate from floating to the top; is_funny=0 strips Un-set jokes.

---

## Cast spells without paying mana

**Ask:** Which cards let me cast spells without paying their mana cost (free-cast effects like the "free" Commander spells, Omniscience, cascade-style cheats)?

_✓ runs, 6+ hits_

```sql
SELECT display_name, type, mana_cost, mana_value, edhrec_rank
FROM cards
WHERE (text LIKE '%without paying its mana cost%'
       OR text LIKE '%without paying their mana cost%')
  AND is_funny = 0
  AND edhrec_rank IS NOT NULL
GROUP BY display_name
ORDER BY edhrec_rank
LIMIT 15;
```

**Why it works:** "Without paying its/their mana cost" is the exact, canonical Oracle wording for free-cast effects, so a LIKE on the raw text column captures the archetype precisely (the "free" commander spells, Omniscience, Hideaway lands, Etali, cascade-like cheats) while excluding cost-reduction or alternative-cost cards that merely lower a cost.

**Sample hits:** Deflecting Swat, Fierce Guardianship, Deadly Rollick, Flawless Maneuver, Mosswort Bridge, Rishkar's Expertise, Etali, Primal Storm, Dauthi Voidwalker, Tibalt's Trickery, Windbrisk Heights, Isochron Scepter, Spinerock Knoll …

**Watch out:** Must use LIKE, not FTS: the FTS tokenizer is fine for the words here, but you'd still want the contiguous phrase, and reminder cards in parentheses use the same phrase so they correctly match too. Cover BOTH "its mana cost" (singular, the vast majority) AND "their mana cost(s)" (plural, ~47 cards like Omniscience/Jodah) — "their mana cost%" with the trailing wildcard also catches the plural "costs". Note this phrase is distinct from "without paying ITS COST" (some sticker/companion cards) and from alternative-cost wordings ("rather than pay", "you may pay {X}"), which this pattern intentionally excludes. Add WHERE edhrec_rank IS NOT NULL because NULLs sort first and would flood the top with unplayed cards; add is_funny=0 to drop joke sets. 574 distinct names match = 574 rows, so there is no real multi-face duplication for this archetype; if a name appears twice in the rendered output (e.g. Etali, Primal Storm) it is a display artifact of the CLI, not duplicate DB rows (confirmed: single id). Use display_name for output as instructed.

**Known false positives to exclude:** Ephemerate (Rebound only re-casts ITSELF for free next upkeep — a value/blink engine, not a cheat-a-big-spell-into-play free-cast enabler; soft mismatch), Inevitable Betrayal / other pure-Suspend cards (match comes from the Suspend reminder text 'cast it without paying its mana cost', but you pay the suspend cost up front — it's cost-deferral, not the Omniscience-style free cast the intent targets)

---

## Cards that care about card names

**Ask:** Which popular cards care about card NAMES — referencing a specific card by name (for "any number of" synergy decks or their own-name payoffs) or caring about cards that share the same name?

_✓ runs, 6+ hits_

```sql
SELECT display_name, edhrec_rank, type, substr(text, 1, 120) AS snippet
FROM cards
WHERE is_funny = 0
  AND edhrec_rank IS NOT NULL
  AND (
        text LIKE '%card named%'         OR text LIKE '%cards named%'
     OR text LIKE '%creature named%'     OR text LIKE '%creatures named%'
     OR text LIKE '%spell named%'        OR text LIKE '%permanent named%'
     OR text LIKE '%not named%'
     OR text LIKE '%same name as%'       OR text LIKE '%with the same name%'
     OR text LIKE '%have the same name%' OR text LIKE '%share a name%'
  )
  -- drop pure-flavor rows whose ONLY name reference is a token they create
  AND NOT (
        (text LIKE '%token named%' OR text LIKE '%tokens named%')
    AND text NOT LIKE '%same name%'
    AND text NOT LIKE '%card named%'
    AND text NOT LIKE '%cards named%'
    AND text NOT LIKE '%creatures named%'
    AND text NOT LIKE '%not named%'
    AND text NOT LIKE '%spell named%'
    AND text NOT LIKE '%number of%named%'
  )
ORDER BY edhrec_rank
LIMIT 40;
```

**Why it works:** It LIKE-matches the name-referencing phrases that define this archetype, covering both sub-flavors: cards that name a specific card ("cards named X" for Rat Colony / Shadowborn Apostle / Hare Apparent style any-number decks and own-name payoffs like Rite of Flame) and cards that key off shared names ("same name", "share a name" for Maelstrom Pulse, Mirror Box, Locket of Yesterdays, Guardian Project). Filtering on edhrec_rank IS NOT NULL plus ORDER BY edhrec_rank surfaces the genuinely played cards first.

**Sample hits:** Guardian Project, Rite of Flame, Tainted Pact, Mechanized Production, Approach of the Second Sun, Marvin, Murderous Mimic, Mirror Box, Extraplanar Lens, Thrumming Stone, Sceptre of Eternal Glory, Endless Atlas, Gisela, the Broken Blade …

**Watch out:** Use LIKE on the raw text column, NOT FTS: the FTS tokenizer drops punctuation and would mangle phrase intent, and MATCH 'named' over-fires on token flavor text. NULL edhrec_rank sorts FIRST under ORDER BY edhrec_rank, so WHERE edhrec_rank IS NOT NULL is mandatory to keep popular cards on top (drop it if you want the full 473 'named' pool including obscure cards). The bare phrase '%named%' badly over-matches — almost every token-maker says "a creature token named X" — so anchor on the synergy phrases here instead; even so a few like Kher Keep/Koma slip in via token names (acceptable, they still literally reference a name). The CLI table renderer can visually echo the last row at a LIMIT boundary (Dragonlord Kolaghan appeared twice on screen) but the underlying rows are unique: the full matched set is 275 rows = 275 distinct names, confirmed via GROUP BY. Cards are stored one row per face; none of these top hits are multi-face, but for split/MDFC cards a name-matching LIKE could hit on a sibling face's text, so dedupe by name or scryfall_oracle_id if you extend it. Borderline-looking hits (Ancestral Anger, Tainted Pact, Extraplanar Lens) are all genuinely on-theme — they count cards by name, stop on duplicate names, or match exiled-card names.

---

## Untap permanents for combo value

**Ask:** Which cards untap target permanents for combo value (e.g. "untap target creature/artifact/land")?

_✓ runs, 6+ hits_

```sql
SELECT c.display_name, c.type, c.mana_value AS cmc, c.edhrec_rank, substr(c.text, 1, 70) AS snippet
FROM cards c
WHERE (
        c.text LIKE '%untap target%'
     OR c.text LIKE '%untap another target%'
     OR c.text LIKE '%untap two target%'
     OR c.text LIKE '%untap three target%'
     OR c.text LIKE '%untap X target%'
     OR c.text LIKE '%untap two other target%'
     OR c.text LIKE '%untap up to one target%'
     OR c.text LIKE '%untap up to two target%'
     OR c.text LIKE '%untap up to three target%'
     OR c.text LIKE '%untap up to four target%'
     OR c.text LIKE '%untap up to five target%'
     OR c.text LIKE '%untap up to X target%'
     OR c.text LIKE '%untap up to one other target%'
     OR c.text LIKE '%untap up to two other target%'
      )
  AND c.is_funny = 0
  AND c.edhrec_rank IS NOT NULL
ORDER BY c.edhrec_rank
LIMIT 25;
```

**Why it works:** The targeted-untap effect (a single permanent untapped on demand) is the defining mechanic of this archetype because it enables loops: untap a mana rock/dork to re-tap it, or untap a creature with a tap ability. Matching the literal phrase "untap target" (plus the "untap another target" and "untap up to N target" wordings) on the raw text column captures exactly these enablers, and ordering by edhrec_rank floats the real combo staples (Arbor Elf, Voltaic Key, Tezzeret) to the top.

**Sample hits:** Maze of Ith, Arbor Elf, Minamo, School at Water's Edge, Thousand-Year Elixir, Manifold Key, Staff of Domination, Patriar's Seal, Saryth, the Viper's Fang, Tezzeret, Cruel Captain, Legolas's Quick Reflexes, Kiora's Follower, Tezzeret the Seeker …

**Watch out:** Use LIKE on the raw `text` column, NOT FTS: the FTS tokenizer drops punctuation but the bigger issue is that FTS would match "untap" and "target" anywhere in the text independently, badly over-matching (e.g. "untap your lands" + "target creature" in unrelated clauses). The exact-phrase LIKE keeps them adjacent. SQLite LIKE is case-insensitive for ASCII, so '%untap target%' already catches both the start-of-sentence "Untap target..." and mid-sentence "...untap target..." — no need for both cases. You MUST include WHERE edhrec_rank IS NOT NULL, because NULL ranks sort FIRST in an ascending ORDER BY and would flood the top with obscure unplayed cards. Watch the wording variants: "untap ANOTHER target permanent" (Kiora's Follower, Vizier, Ioreth) does NOT contain the substring "untap target", so it needs its own LIKE clause; likewise "untap up to two target artifacts" (Tezzeret the Seeker) needs the '%untap up to % target%' pattern. Because cards are stored one row per face, the `text` column already holds the full face text (faces joined by " / "), so single-row LIKE works without a self-join — but a double-faced card could appear once per face if both faces had untap text. This pattern intentionally EXCLUDES mass-untap effects ("untap all"/"untap each"/"untap your" Seedborn Muse-style) which are a related but distinct ramp/storm archetype rather than targeted combo untap; add `OR text LIKE '%untap all%'` only if you want to widen to mass untappers.

**Known false positives to exclude:** Teferi, Hero of Dominaria (cross-line artifact: 'untap up to two lands' [no target] sits in +1 ability, the matched 'target' is in the unrelated -3 ability; removed by the bounded fix), Caetus, Sea Tyrant of Segovia (same cross-line wildcard artifact; removed by fix), The Watcher in the Water (cross-line artifact; removed by fix), Nissa, Steward of Elements (cross-line artifact; removed by fix), Nissa, Sage Animist (cross-line artifact; removed by fix)

---

## Produce two or more colors of mana

**Ask:** Which permanents (creatures, artifacts, enchantments, lands, planeswalkers) can tap or otherwise produce mana of two or more different colors, ranked by how commonly they're played?

_✓ runs, 6+ hits_

```sql
SELECT c.display_name,
       c.type,
       c.mana_value,
       (SELECT GROUP_CONCAT(pm2.mana, '')
          FROM card_produced_mana pm2
         WHERE pm2.card_id = c.id
           AND pm2.mana IN ('W','U','B','R','G')) AS colors_produced,
       c.edhrec_rank
FROM cards c
WHERE c.is_funny = 0
  AND c.edhrec_rank IS NOT NULL
  AND c.id = (SELECT MIN(c2.id) FROM cards c2
              WHERE c2.scryfall_oracle_id = c.scryfall_oracle_id)
  AND EXISTS (SELECT 1 FROM card_types ct
              WHERE ct.card_id = c.id
                AND ct.type IN ('Creature','Artifact','Enchantment','Land','Planeswalker'))
  AND (SELECT COUNT(DISTINCT pm.mana)
         FROM card_produced_mana pm
        WHERE pm.card_id = c.id
          AND pm.mana IN ('W','U','B','R','G')) >= 2
  -- FIX: drop MDFC face-aggregation artifacts (Pathway lands). card_produced_mana
  -- aggregates BOTH faces onto each face's row, so single-color Pathway faces
  -- (e.g. Clearwater Pathway = "{T}: Add {U}.") falsely show 2 colors.
  AND NOT (c.layout = 'modal_dfc'
           AND c.text IN ('{T}: Add {W}.','{T}: Add {U}.','{T}: Add {B}.',
                          '{T}: Add {R}.','{T}: Add {G}.'))
ORDER BY c.edhrec_rank ASC;
```

**Why it works:** card_produced_mana holds one row per (card, mana symbol); counting DISTINCT symbols restricted to the five WUBRG colors and requiring >= 2 isolates true multi-color mana sources. Restricting to permanent types (via card_types) keeps it to creatures/lands/rocks/etc., and ordering by edhrec_rank surfaces the real staples (dual lands, Signets, Birds, treasure-makers like Tireless Provisioner).

**Sample hits:** Command Tower, Arcane Signet, Exotic Orchard, Path of Ancestry, Fellwar Stone, Birds of Paradise, Commander's Sphere, Watery Grave, Godless Shrine, Smothering Tithe, Breeding Pool, Hallowed Fountain …

**Watch out:** Crucial: 'C' (colorless) lives in card_produced_mana alongside WUBRG, so you MUST filter pm.mana IN ('W','U','B','R','G') or any-color rocks and colorless producers inflate the count. Despite the "one row per face" rule, some cards have BOTH a 'normal' row and a 'reversible_card' face row sharing the same scryfall_oracle_id (e.g. Command Tower id 5736 AND 5737, Birds of Paradise 2967 AND 2968) — without the MIN(id) dedupe-by-oracle-id subquery every such staple appears twice. edhrec_rank is NULL for many cards and NULLs sort FIRST, so WHERE edhrec_rank IS NOT NULL is mandatory for a popularity sort; is_funny=0 drops Un-set jokes. Don't try FTS for this — produced mana is a normalized table, not text, and the colorless 'C' and multi-color identity aren't reliably in the oracle text. Note many top "odd source" creatures (Storm-Kiln Artist, Pitiless Plunderer, Goldspan Dragon) produce all five colors only because they make Treasure tokens, which MTGJSON records as WUBRG in produced_mana — accurate but worth knowing if you expected literal tap-for-mana abilities.

**Known false positives to exclude:** Clearwater Pathway, Murkwater Pathway (back face), Brightclimb Pathway, Blightstep Pathway, Needleverge Pathway, Riverglide Pathway, Cragcrown Pathway, Barkchannel Pathway …

---

## Planeswalkers with unusual static text

**Ask:** Which planeswalkers have unusual static or passive abilities (continuous effects like "opponents can't draw", anthems, cost reduction, or sorcery-speed locks) rather than only the normal loyalty abilities?

_✓ runs, 6+ hits_

```sql
SELECT display_name, loyalty_num, type,
       substr(text, 1, instr(text||char(10), char(10)) - 1) AS static_lead,
       edhrec_rank
FROM cards
WHERE types LIKE '%Planeswalker%' AND is_funny = 0
  AND text NOT LIKE '[%'            -- first line is NOT a loyalty ability [+N]/[-N]
  AND text LIKE '%[%'              -- but card DOES have loyalty abilities below
  AND text NOT LIKE 'Whenever%'    -- exclude leading triggered abilities
  AND text NOT LIKE 'When %'
  AND text NOT LIKE 'At the beginning%'
  AND text NOT LIKE 'Compleated%'
  -- drop lead lines that are keyword/casting modifiers, not battlefield statics:
  AND text NOT LIKE 'Flash%' AND text NOT LIKE 'Hexproof%' AND text NOT LIKE 'Daybound%'
  AND text NOT LIKE 'Kicker%' AND text NOT LIKE 'Ninjutsu%' AND text NOT LIKE 'Casualty%'
  AND text NOT LIKE 'Magecraft%' AND text NOT LIKE 'Extort%' AND text NOT LIKE 'Space sculptor%'
  AND text NOT LIKE 'Landfall%' AND text NOT LIKE 'Protection from%'
  -- drop ETB / 'enters with' loyalty-setup lines (not continuous effects):
  AND text NOT LIKE '% enters with %' AND text NOT LIKE '%enters, %' AND text NOT LIKE 'As % enters%'
  -- drop the on-the-stack uncounterable clause (not a battlefield static):
  AND text NOT LIKE 'This spell can''t be countered%'
  AND edhrec_rank IS NOT NULL
GROUP BY name
ORDER BY edhrec_rank
LIMIT 20;
```

**Why it works:** On a planeswalker, loyalty abilities are formatted as bracketed lines like [+1]:, [-2]:, [0]:. A static/passive ability shows up as a leading text line BEFORE the first bracket. The query keys on that structure (text NOT LIKE '[%' AND text LIKE '%[%') so the card's opening line is non-loyalty text, then filters to genuine continuous-effect phrasing (As long as / can't / cost N less / Each opponent / Players / replacement "If you would" effects) while excluding leading triggered abilities (Whenever/When/At the beginning) and the Phyrexian "Compleated" reminder boilerplate. edhrec_rank ranking surfaces real, played cards.

**Sample hits:** Elspeth, Storm Slayer, Narset, Parter of Veils, Ugin, the Ineffable, Jace, Wielder of Mysteries, Teferi, Time Raveler, The Eternal Wanderer, Grist, the Hunger Tide, Karn, the Great Creator, The Aetherspark, Teferi, Master of Time, Tyvar, Jubilant Brawler, Ashiok, Dream Render …

**Watch out:** 1) Loyalty costs use a real minus sign (U+2212 '−'), not a hyphen '-', so don't LIKE-match '[-'; the structural check 'text NOT LIKE [%' / 'text LIKE %[%' sidesteps that entirely. 2) FTS is useless here: the tokenizer drops the '[', ']', '/', '+', '−' punctuation that distinguishes static text from loyalty abilities, so you MUST use LIKE on raw text. 3) The apostrophe in can't / don't must be SQL-escaped as '' inside the string literal. 4) Multi-face cards (one row per face) and reprints duplicate rows that share name, so GROUP BY name dedupes (e.g. Ugin, Eye of the Storms appeared twice before grouping). 5) edhrec_rank is NULL for many cards and NULLs sort FIRST under ORDER BY edhrec_rank, so 'edhrec_rank IS NOT NULL' is required to get popular cards. 6) is_funny=0 drops Un-set joke planeswalkers. 7) This is a heuristic: leading text that is a triggered ability (Whenever/When) is excluded, but a true static could in principle follow a trigger on the same card and be missed; broaden the OR-list of static phrasings if you need higher recall. 8) Some matched leading lines are replacement effects ('If you would...') which are passive/continuous and intentionally included as 'unusual static' text.

**Known false positives to exclude:** Chandra, Awakened Inferno, Ral, Leyline Prodigy

---
