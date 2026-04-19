#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMO_DIR="$ROOT/studio/agent-exporter-promo"

cd "$PROMO_DIR"

pnpm install --frozen-lockfile
pnpm poster
pnpm render
