#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMO_DIR="$ROOT/studio/agent-exporter-promo"
VOICEOVER_DIR="$PROMO_DIR/voiceover"
PUBLIC_DIR="$PROMO_DIR/public"
MEDIA_DIR="$ROOT/docs/assets/media"

if ! command -v say >/dev/null 2>&1; then
  echo "say is required to refresh promo voiceover assets on macOS." >&2
  exit 1
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "ffmpeg is required to encode promo voiceover assets." >&2
  exit 1
fi

mkdir -p "$PUBLIC_DIR" "$MEDIA_DIR"

build_voiceover() {
  local name="$1"
  local rate="$2"
  local aiff_file="/tmp/agent-exporter-$name-voiceover.aiff"
  local text_file="$VOICEOVER_DIR/$name.txt"
  local public_output="$PUBLIC_DIR/agent-exporter-promo-$name-voiceover.m4a"
  local media_output="$MEDIA_DIR/agent-exporter-promo-$name-voiceover.m4a"

  say -v Samantha -r "$rate" -o "$aiff_file" -f "$text_file"
  ffmpeg -y -i "$aiff_file" -c:a aac -b:a 128k "$public_output" >/dev/null 2>&1
  cp "$public_output" "$media_output"
  rm -f "$aiff_file"
}

build_voiceover "landscape" 185
build_voiceover "vertical" 170
