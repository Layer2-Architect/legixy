#!/bin/bash
# scripts/extract-verdict.sh
# Stop hook / SubagentStop hook から呼ばれて、Reviewer subagent の最終出力から VERDICT を抽出する。
# 引数: $1 = Reviewer 出力 transcript path（Claude Code が hook 引数として渡す）
# 出力: .devproc/verdict.log に append、stderr に開発者向けサマリ
# 詳細: docs/DevProc_V2/review-guidelines/verdict-marker.md §9

set -euo pipefail

OUTPUT_FILE="${1:-/dev/stdin}"
VERDICT_LOG=".devproc/verdict.log"
ARCHIVE_DIR=".devproc/reviewer-output"

mkdir -p "$(dirname "$VERDICT_LOG")" "$ARCHIVE_DIR"

# Reviewer 出力を archive する（後で /advance から参照できるよう）
TIMESTAMP=$(date -u +"%Y-%m-%dT%H-%M-%SZ")
ARCHIVE_FILE="${ARCHIVE_DIR}/${TIMESTAMP}.md"
if [ -f "$OUTPUT_FILE" ]; then
  cp "$OUTPUT_FILE" "$ARCHIVE_FILE"
fi

# 末尾の VERDICT マーカーを取得（複数あれば最後を採用）
VERDICT=$(grep -oE "<!-- VERDICT:(APPROVE|REQUEST_CHANGES|NEEDS_HUMAN) -->" "$OUTPUT_FILE" 2>/dev/null | tail -1 || true)

if [ -z "$VERDICT" ]; then
  # fail-safe: 欠落は REQUEST_CHANGES として扱う
  VALUE="REQUEST_CHANGES"
  REASON="marker-missing"
else
  VALUE=$(echo "$VERDICT" | sed -E 's/<!-- VERDICT:(APPROVE|REQUEST_CHANGES|NEEDS_HUMAN) -->/\1/')
  REASON="explicit"
fi

# 形式: ISO8601 timestamp | git HEAD | VERDICT | reason | archive_path
ISO_TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
HEAD=$(git rev-parse --short HEAD 2>/dev/null || echo "no-git")

echo "${ISO_TS} | ${HEAD} | ${VALUE} | ${REASON} | ${ARCHIVE_FILE}" >> "$VERDICT_LOG"

# stderr に開発者向けに通知
echo "" >&2
echo "═══ AI Reviewer VERDICT ═══" >&2
echo "  判定: ${VALUE}" >&2
echo "  ログ: ${VERDICT_LOG}" >&2
echo "  詳細: ${ARCHIVE_FILE}" >&2
echo "═══════════════════════════" >&2
echo "" >&2

# REQUEST_CHANGES の場合は exit code 1（hook 側で次のアクション分岐用）
[ "$VALUE" = "REQUEST_CHANGES" ] && exit 1
exit 0
