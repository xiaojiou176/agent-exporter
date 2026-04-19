#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMO_DIR="$ROOT/studio/agent-exporter-promo"

if command -v say >/dev/null 2>&1 && command -v ffmpeg >/dev/null 2>&1; then
  "$ROOT/scripts/build_promo_voiceover.sh"
fi

cd "$PROMO_DIR"

pnpm install --frozen-lockfile
pnpm poster
pnpm social-card
pnpm render
pnpm vertical
pnpm vertical-poster
