#!/bin/bash
# scripts/check-latest-verdict.sh
# /advance slash command から呼ばれる。最新 VERDICT を読み取って状態を stdout に返す。
# 戻り値: APPROVE / REQUEST_CHANGES / NEEDS_HUMAN / NO_VERDICT のいずれか
# 詳細: docs/DevProc_V2/review-guidelines/verdict-marker.md §9

set -euo pipefail

VERDICT_LOG=".devproc/verdict.log"

if [ ! -f "$VERDICT_LOG" ]; then
  echo "NO_VERDICT"
  exit 0
fi

# 最新エントリの 3 列目（VERDICT 値）を取得
LATEST=$(tail -1 "$VERDICT_LOG" | awk -F'|' '{print $3}' | tr -d ' ')

if [ -z "$LATEST" ]; then
  echo "NO_VERDICT"
  exit 0
fi

echo "$LATEST"
