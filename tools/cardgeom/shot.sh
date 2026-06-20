#!/usr/bin/env bash
# shot.sh HTML PNG — screenshot a card HTML at full bleed size with headless Chrome.
set -euo pipefail
HTML="$1"; PNG="$2"
CHROME="${MTGBRAIN_CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
# Flags mirror src/render.rs::compose so the preview matches the real pipeline
# (virtual-time-budget lets fonts.ready + the auto-fit JS run before capture).
"$CHROME" --headless --disable-gpu --hide-scrollbars \
  --no-default-browser-check --no-first-run \
  --force-device-scale-factor=1 \
  --run-all-compositor-stages-before-draw \
  --virtual-time-budget=15000 \
  --window-size=2176,2960 \
  --screenshot="$PNG" "file://$HTML" 2>/dev/null
echo "$PNG"
