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

# Launch the local cube-editor web UI (deliberately weird port to avoid clashes).
edit PORT="49737":
    cargo run --release -- edit serve --port {{PORT}}

# Download old-frame render assets (cardconjurer frames, old fonts, foil, mana font) -> ./assets.
render-assets:
    cargo run --release -- render assets

# Render one card to a print-ready old-frame PNG by editor-DB id, e.g. just render-card 0
render-card ID:
    cargo run --release -- render card {{ID}}

# Render every card (normal); add foil with: cargo run --release -- render all --foil
render-all:
    cargo run --release -- render all

# Claude key (artwork match + crop) pulled from rageveil at run time.
mpc_key := "rageveil show geosurge.ai/api.anthropic.com/onehr-cellvm/pool"

# Render ONE card with MPCfill high-DPI art + foil, e.g. just render-mpc 2
render-mpc ID:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
      cargo run --release -- render card {{ID}} --art-backend https://mpcfill.com --foil --force

# Render the WHOLE cube with MPCfill high-DPI art + foil (long; ~hours, ~390 Claude calls).
render-cube:
    ANTHROPIC_API_KEY="$({{mpc_key}} | head -1)" \
      cargo run --release -- render all --art-backend https://mpcfill.com --foil

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets
