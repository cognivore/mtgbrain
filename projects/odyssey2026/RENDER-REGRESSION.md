# Render regression set

A fixed set of tricky cards to eyeball after any change to the renderer
(`src/render.rs`, `src/card_template.html`). Render them all with:

```sh
just render-regression          # fast: Scryfall art, no MPCfill/Claude
```

Outputs land in `render-cache/cards/<Card>/<hash>.png`. Open them and check each
assertion below. (Geometry spec: see CARD-GEOMETRY.md.)

| Card | Guards against |
|---|---|
| **Propaganda** | inline `{2}` in rules must be **UPRIGHT**, not synthetic-italic (the `<i class="ms">` / `.rules i` skew bug, 2026-06). Pip is a grey circle. |
| **Spikeshot Elder** | `{T}` tap symbol renders as the tap icon; activated-ability mana `{1}{R}{R}` pips inline and upright. |
| **Glint-Horn Buccaneer** | longer rules text auto-fits the box; inline `{1}{R}` cost pip; reminder/keyword text not mangled. |
| **Possessed Aven** | multicolour (TOR) → multi/gold frame; inline mana upright. |
| **Cabal Pit** | land frame; ability text + `{T}`/`{B}` pips. |
| **Wild Mongrel** | `{G}` discard ability pip; short rules vertically centered. |

### Always-check invariants (every card)
1. **Mana pips are never italic** (cost row *and* inline in rules).
2. **Title is not doubled** — white fill + tight black outline, single crisp image.
3. Pips ≈ title height; generic pips grey with drop-shadow.
4. Title / type / rules / P/T sit in their boxes; nothing clips the frame.
5. Bottom shows `Illus. <artist>` and `NOT FOR SALE · ™ & © <year> Wizards…`, both legible, not crammed.
6. Foil variant (`--foil`) = a single pure-white 7ED falling star, nothing else (no sheen/tint).
