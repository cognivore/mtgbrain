# pointless-b-side — the 8ED (modern-frame) BACK cube

The cube printed on the **back** of the odyssey2026 MPC cards. Rendered in the
**8th-Edition / modern frame** (vs odyssey2026's old frame), with **latest high-DPI art**.

## Files
- `cube_list.txt` — the card list (one name per line; blank / `#` lines ignored).
- `data/pointless-b-side.sqlite` — the editor DB (gitignored build artifact), seeded from the list.

## Workflow
```sh
just edit-8ed-seed          # seed data/pointless-b-side.sqlite from cube_list.txt
just edit-8ed               # serve the 8ED editor on http://127.0.0.1:49738
```
`edit-8ed` runs `edit serve --frame 8th`, so every render (UI preview, render card/all,
CSV image URLs) uses the 8th-Edition frame for this cube. The odyssey2026 front cube
(old frame, port 49737) is untouched — two parallel editors feed one double-faced order.

## Note
Double-faced cards (transform / MDFC / meld / adventure / split / flip — anything with `//`)
do not render in the single-face 8ED frame; they are flagged on seed so we can pick a face
or substitute per card.
