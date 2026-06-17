#!/bin/bash
# scripts/guard-quality-baseline.sh
# PreToolUse hook (Edit/Write matcher) から呼ばれる。
# 品質基準そのものを緩める変更を deterministic に block する。
# AI 同士が結託して基準を下げることを防ぐメタレベル安全弁。
# 詳細: docs/DevProc_V2/review-guidelines/severity.md §3

set -euo pipefail

FILE_PATH="${1:-}"

if [ -z "$FILE_PATH" ]; then
  exit 0  # path がなければ何もしない
fi

# 品質基準ファイルへの書き込みを検出するパターン
PROTECTED_PATTERNS=(
  "^docs/DevProc_V2/[0-9]"             # 00-philosophy.md 〜 09-compiler-lens.md
  "^docs/DevProc_V2/README\.md$"
  "^docs/DevProc_V2/review-guidelines/"
  "^docs/DevProc_V2/bootstrap/CLAUDE.*\.md\.template$"
  "^\.trace-engine\.toml$"
  "^scripts/trace-check\.sh$"
  "^docs/perspectives/"                # 観点ナレッジベース（追加は OK だが削除/緩和は block）
)

for pattern in "${PROTECTED_PATTERNS[@]}"; do
  if echo "$FILE_PATH" | grep -qE "$pattern"; then
    echo "" >&2
    echo "═══ PreToolUse Guard: 品質基準ファイルへの書き込み ═══" >&2
    echo "  ファイル: $FILE_PATH" >&2
    echo "" >&2
    echo "  このファイルは品質基準を構成する成果物です（review-guidelines/severity.md §3）。" >&2
    echo "  AI による自動編集は禁止されています。" >&2
    echo "" >&2
    echo "  変更が本当に必要な場合は:" >&2
    echo "    1. 開発者が ADR で変更理由を記録する" >&2
    echo "    2. ADR ID を引用しつつ開発者自身が手で編集する" >&2
    echo "    3. 変更後に /advance で人間 Approve を経る" >&2
    echo "" >&2
    echo "═════════════════════════════════════════════════════" >&2
    echo "" >&2
    exit 2  # exit code 2 で Claude Code に block を伝える
  fi
done

exit 0
