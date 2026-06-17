#!/bin/bash
# scripts/count-fix-cycle.sh
# 修正イベントの再帰サイクル数をチェックする。
# 10-modification-events.md §6 の 3 サイクル超過警告を実装。
#
# 動作:
#   - .devproc/event-log.txt の末尾エントリのサイクル数を取得
#   - 3 を超えていたら警告メッセージを stderr に出力
#   - サイクル数を stdout に返す (スクリプトからの利用用)

set -euo pipefail

EVENT_LOG=".devproc/event-log.txt"
THRESHOLD=3

if [ ! -f "$EVENT_LOG" ]; then
  echo "0"
  exit 0
fi

# 末尾エントリの cycle 列を抽出 (フォーマット: ts | event | trigger | parent | cycle: N | verdict | archive)
LATEST_CYCLE=$(tail -1 "$EVENT_LOG" | awk -F'|' '{print $5}' | sed -E 's/.*cycle:[ ]*([0-9]+).*/\1/' | tr -d ' ')

if [ -z "$LATEST_CYCLE" ] || ! echo "$LATEST_CYCLE" | grep -qE "^[0-9]+$"; then
  echo "0"
  exit 0
fi

echo "$LATEST_CYCLE"

if [ "$LATEST_CYCLE" -gt "$THRESHOLD" ]; then
  echo "" >&2
  echo "═══ ⚠ 修正フロー ${LATEST_CYCLE} サイクル目です (閾値: ${THRESHOLD}) ═══" >&2
  echo "" >&2
  echo "  これは個別修正で対処すべき問題ではなく、設計レベルの見直しが必要なシグナルです。" >&2
  echo "" >&2
  echo "  推奨アクション:" >&2
  echo "    1. 現在の修正サイクルを停止する" >&2
  echo "    2. これまで修正されたノード群を点検する:" >&2
  echo "       cat ${EVENT_LOG}" >&2
  echo "    3. ADR を起票して設計判断の根本的な見直しを記録する:" >&2
  echo "       docs/adr/ADR-NNN_<topic>.md を作成" >&2
  echo "    4. 必要なら SPEC レベルでの再設計を検討する" >&2
  echo "" >&2
  echo "  継続する場合は ADR にスキップ理由を明記してください。" >&2
  echo "" >&2
  echo "════════════════════════════════════════════════════" >&2
  echo "" >&2

  exit 1  # 非ゼロ exit code で hook 側に警告を伝える
fi

exit 0
