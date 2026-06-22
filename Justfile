# mtgbrain — task runner. Run inside `nix develop` / direnv (deps come from the flake).

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# One-shot: fetch MTGJSON + CubeCobra Elo + build the SQLite DB into ./data.
setup:
    cargo run --release -- setup

# Fetch MTGJSON AtomicCards into ./data.
download:
    cargo run --release -- download

# Fetch CubeCobra card Elo ratings into ./data/cubecobra_elo.jsonl.
cubecobra:
    cargo run --release -- cubecobra

# (Re)build ./data/mtg.sqlite from downloaded JSON.
build-db:
    cargo run --release -- build

# Reproducible sandboxed build -> ./result/bin/mtgbrain
nix-build:
    nix build .#default
    @echo "built: ./result/bin/mtgbrain"

# Print schema + cheatsheet (start here when authoring queries).
schema:
    cargo run --release -- schema

# Run a SQL query, e.g.  just sql "SELECT display_name FROM cards LIMIT 5"
sql QUERY:
    cargo run --release -- sql "{{QUERY}}"

# Full-text search, e.g.  just search "flip a coin"
search QUERY:
    cargo run --release -- search "{{QUERY}}"

# Show one card,  e.g.  just card "Krark, the Thumbless"
card NAME:
    cargo run --release -- card "{{NAME}}"

# Build the cube-editor DB from projects/odyssey2026/cube360/cube_list.txt.
edit-seed:
    cargo run --release -- edit seed --force

# Launch the local cube-editor web UI — UNIFIED: always loads the rageveil keys + live
# MPCfill art + GenAI gallery, so "Generate" works out of the box (degrades to no-art if
# rageveil is unavailable — keys just come back empty). Weird port avoids clashes.
edit PORT="49737":
    # Free the port first: kill any lingering mtgbrain editor on it so boot never fails
    # with "Address already in use". (Leading '-' = ignore errors on this line.)
    -for p in $(lsof -tiTCP:{{PORT}} -sTCP:LISTEN 2>/dev/null); do ps -o comm= -p $p 2>/dev/null | grep -q mtgbrain && kill $p 2>/dev/null; done; sleep 0.4
    # Keys are best-effort: '|| true' so a locked/absent rageveil degrades to no-art, never bricks boot.
    ANTHROPIC_API_KEY="$({{mpc_key}} 2>/dev/null | head -1 || true)" \
    OPENAI_API_KEY="$({{openai_key}} 2>/dev/null | head -1 || true)" \
      cargo run --release -- edit serve --port {{PORT}} --art-backend https://mpcfill.com

# Recompute color_identity from each card's effective (override-applied) mana cost
# + rules-text mana symbols, and write it back. Add --dry-run to preview.
recolor *FLAGS:
    cargo run --release -- edit recolor {{FLAGS}}

# Export the CubeCobra bulk-import CSV (custom S3 image URLs + recomputed colours)
# -> data/odyssey2026_cubecobra.csv. Paste into the cube's "Replace from CSV".
cubecobra-csv *FLAGS:
    cargo run --release -- edit cubecobra-csv {{FLAGS}}

# Download old-frame render assets (cardconjurer frames, old fonts, foil, mana font) -> ./assets.
render-assets:
    cargo run --release -- render assets

# Render one card to a print-ready old-frame PNG by editor-DB id, e.g. just render-card 0
render-card ID:
    cargo run --release -- render card {{ID}}

# Render every card (normal); add foil with: cargo run --release -- render all --foil
render-all:
    cargo run --release -- render all

# Build the Scryfall-format "selfhost" image (cropped, rounded, 745x1040) for every
# card from its forefront MPC render. No Chrome/network needed.
selfhost:
    cargo run --release -- render selfhost

# Claude key (artwork match + crop) pulled from rageveil at run time.
mpc_key := "rageveil show geosurge.ai/api.anthropic.com/onehr-cellvm/pool"

# OpenAI key (GenAI art) from rageveil.
openai_key := "rageveil show platform.openai.com/api"

# `edit-art` is now identical to `edit` (the editor is unified — always keyed). Kept as an
# alias for muscle memory; forwards the PORT argument.
alias edit-art := edit

# Render ONE card with MPCfill high-DPI art + foil, e.g. just render-mpc 2
render-mpc ID:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
      cargo run --release -- render card {{ID}} --art-backend https://mpcfill.com --foil --force

# Render the fixed regression set (fast, Scryfall art) — see RENDER-REGRESSION.md.
render-regression:
    db="data/cube_editor.sqlite"; \
    for n in "Propaganda" "Spikeshot Elder" "Glint-Horn Buccaneer" "Possessed Aven" "Cabal Pit" "Wild Mongrel"; do \
      id=$(sqlite3 "$db" "SELECT id FROM cube_cards WHERE name='$n';"); \
      [ -n "$id" ] && cargo run --release -- render card "$id" --force || echo "skip $n"; \
    done; \
    echo "regression set -> render-cache/cards/<Card>/  (check RENDER-REGRESSION.md)"

# Render the WHOLE cube with MPCfill high-DPI art + foil (long; ~hours, ~390 Claude calls).
render-cube:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
      cargo run --release -- render all --art-backend https://mpcfill.com --foil

# FULL pipeline: render the cube, then build all selfhost images, then publish.
# `*FLAGS` are forwarded to `render all` (e.g. `--force`, `--foil`).
render-publish *FLAGS:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
      cargo run --release -- render all --art-backend https://mpcfill.com {{FLAGS}}
    cargo run --release -- render selfhost
    just publish-selfhost

# Upload selfhost.png files to the bucket. TODO(jm): fill in once the mechanism is
# confirmed — rclone remote vs aws s3 sync (see `cubecobra-csv --base`). Until then
# this is a no-op that prints what it WOULD sync so nothing silently breaks.
publish-selfhost:
    @echo "TODO: upload render-cache/cards/*/selfhost.png -> s3://social-doma-dev-media/odyssey2026/"
    @echo "  ($(find render-cache/cards -name selfhost.png | wc -l | tr -d ' ') images ready)"

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets

# Async FULL PASS: generate GenAI galleries for all flagged cards (keys via rageveil).
# Idempotent/resumable; review + choose in the editor's Review GenAI tab.
genai-pass:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
    OPENAI_API_KEY="$({{openai_key}} | head -1)" \
      cargo run --release -- render genai-pass --art-backend https://mpcfill.com
