#!/usr/bin/env bash
# Post the PPU implementation issues. Dry run unless called with --yes.
set -euo pipefail
cd "$(dirname "$0")"

REPO="${REPO:-NoxHarmonium/sirc}"
RUN=0
[[ "${1:-}" == "--yes" ]] && RUN=1

run() {
  if [[ $RUN == 1 ]]; then
    "$@"
  else
    printf 'DRY RUN:'
    printf ' %q' "$@"
    printf '\n'
  fi
}

run gh label create ppu --repo "$REPO" --description "Video device (PPU)" --color 1d76db || true

declare -A NUM
while IFS=$'\t' read -r key file milestone title; do
  echo "== $key: $title"
  if [[ $RUN == 1 ]]; then
    # Drop the markdown H1 and the blank line after it: GitHub shows the title separately
    url=$(tail -n +3 "$file" | gh issue create --repo "$REPO" --title "$key: $title" --label ppu --body-file -)
    NUM[$key]="${url##*/}"
    echo "   $url"
  else
    printf 'DRY RUN: tail -n +3 %q | gh issue create --repo %q --title %q --label ppu --body-file -\n' "$file" "$REPO" "$key: $title"
    NUM[$key]="<$key>"
  fi
done < issues.tsv

# Tracking issue: add the real issue numbers to the task list so GitHub tracks them
body=$(cat 00-tracking-issue.md | tail -n +3)
for key in "${!NUM[@]}"; do
  body="${body//\*\*$key\*\*/#${NUM[$key]} **$key**}"
done
if [[ $RUN == 1 ]]; then
  printf '%s\n' "$body" | gh issue create --repo "$REPO" --title "PPU: implement the video device (tracking)" --label ppu --body-file -
else
  printf 'DRY RUN: gh issue create --repo %q --title %q --label ppu --body-file - (tracking body, %d lines)\n' \
    "$REPO" "PPU: implement the video device (tracking)" "$(printf '%s\n' "$body" | wc -l)"
fi
