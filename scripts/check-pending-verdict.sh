#!/bin/bash
# scripts/check-pending-verdict.sh
# Stop hook（メインセッション終了時）から呼ばれる。
# 最新 VERDICT が REQUEST_CHANGES のまま残っていたら開発者に警告する。

set -euo pipefail

VERDICT_LOG=".devproc/verdict.log"

if [ ! -f "$VERDICT_LOG" ]; then
  exit 0
fi

LATEST=$(tail -1 "$VERDICT_LOG" 2>/dev/null | awk -F'|' '{print $3}' | tr -d ' ' || echo "")

case "$LATEST" in
  REQUEST_CHANGES)
    echo "" >&2
    echo "─── ⚠ 未解決の REQUEST_CHANGES が残っています ───" >&2
    echo "  最新 VERDICT: REQUEST_CHANGES" >&2
    echo "  詳細を確認: tail -1 ${VERDICT_LOG}" >&2
    echo "  Reviewer 出力: $(tail -1 "$VERDICT_LOG" | awk -F'|' '{print $5}' | tr -d ' ')" >&2
    echo "─────────────────────────────────────────────" >&2
    ;;
  NEEDS_HUMAN)
    echo "" >&2
    echo "─── △ NEEDS_HUMAN: 開発者の明示操作待ち ───" >&2
    echo "  次に進むには phase tag を打ってください。" >&2
    echo "  例: git tag v1-<stage>-green" >&2
    echo "──────────────────────────────────────────" >&2
    ;;
  *)
    # APPROVE / NO_VERDICT は警告不要
    ;;
esac

exit 0
