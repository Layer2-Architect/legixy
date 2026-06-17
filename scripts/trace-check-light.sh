#!/usr/bin/env bash
# trace-check-light.sh — PostToolUse 用の軽量チェック（DevProc_V4.1）
#
# 編集直後のファイル 1 件に対して、Document ID 行の有無だけを早期検出する軽量版。
# 重い検証（traceability-engine check --formal / 各種ゲート）は scripts/trace-check.sh が担う。
# フックからは `|| true` 付きで呼ばれるため、ここでの非ゼロ終了はブロックしない（警告のみ）。
#
# Usage: bash scripts/trace-check-light.sh <file-path>

set -uo pipefail

FILE="${1:-}"

# 対象外: 空・非 .md・DevProc 本体・テンプレート・隠しディレクトリ配下
[[ -z "$FILE" ]] && exit 0
[[ "$FILE" != *.md ]] && exit 0
[[ "$FILE" == *"docs/DevProc_V4/"* ]] && exit 0
[[ "$FILE" == *"/templates/"* ]] && exit 0
[[ ! -f "$FILE" ]] && exit 0

# typecode-id を持つべき成果物ディレクトリ配下のみ検査
case "$FILE" in
  */docs/specs/*|*/docs/usecases/*|*/docs/nfr/*|*/docs/test-perspectives/*|\
  */docs/gap-analysis/*|*/docs/robustness-abstract/*|*/docs/sequence-abstract/*|\
  */docs/robustness-detail/*|*/docs/sequence-detail/*|*/docs/detailed-design/*|\
  */docs/test-specs/*|*/docs/acceptance-tests/*|*/docs/adr/*|*/docs/validation/*|\
  */docs/responsibility-preservation/*|*/docs/frontend-pass/*|*/docs/spec-patches/*)
    if ! grep -qE '^Document ID:' "$FILE"; then
      printf "\033[33m[trace-check-light] 警告: %s に 'Document ID:' 行がありません（ハードルール 4）\033[0m\n" "$FILE" >&2
      exit 1
    fi
    ;;
esac

exit 0
