#!/usr/bin/env bash
# trace-check.sh — DevProc_V2 フレームワーク統合検証スクリプト
#
# 1. traceability-engine check --formal （第 1 層: 形式検証）
#    + F2 escalation: [WARNING] ChainIntegrity（チェーン断裂）を grep して FAIL 化
#      （エンジンは ChainIntegrity を Warning 扱いで exit=0 を返すため、配送/機能チェーンの
#       孤児・未連結が素通りする。ラッパでゲート化する。DevProc_V4.1 §7/§12, 08-gates）
# 2. traceability-engine check          （第 1 層 + 第 2 層 semantic、ONNX モデル必須）
# 3. SPEC レベル TDD ループ固有のゲート:
#    - red 状態の TP がない
#    - open 状態の GAP がない
# 4. 前段ループゲート（ハードルール 9）:
#    - 各 SPEC に対して最新の FCR を見つけ、frontend_status = ACCEPTED であること
#    - SPEC ファイルに `**前段スキップ**: ADR-...` がある場合は pass 扱い
# 5. レイヤ汚染検査（ハードルール 10）:
#    - 具体側 RBD/SEQD に言語固有要素が混入していないこと
#    - 関数名(snake_case/camelCase 呼び出し)、型表記(Result<T,E>, Vec<T>)、
#      修飾子(pub, async)、crate 識別子(tokio::, std::)を検出
# 6. 契約適合ゲート（配送層、DevProc_V4.1 §12 / P-1）:
#    - ①ビルド → ②TC[DLV] 実バイナリ E2E（TC-CLI-001=cli.rs / TC-MCP-001=ts-mcp e2e）pass
#    - 凍結境界契約（LGX-COMPAT-001 §3）への適合を実バイナリで検証する。機能チェーン GREEN は
#      契約適合を意味しない（defect-root-cause 2026-06-14 の真因）ため独立ゲートとして強制。
#
# CI で `bash scripts/trace-check.sh` として呼び出す想定。失敗（exit != 0）で
# リリースブロック。CLAUDE.md ハードルール 2（GAP がクローズしないうちに
# 次フェーズへ進まない）およびハードルール 9（前段 ACCEPTED 未経由の SPEC で
# 下流着手禁止）の機械検証。
#
# Usage:
#   bash scripts/trace-check.sh                   # 第 1 層 + 第 2 層 + ゲート + 契約適合
#   bash scripts/trace-check.sh --no-semantic     # 第 2 層を明示的にスキップ（高速 CI 用）
#   bash scripts/trace-check.sh --no-conformance  # 契約適合ゲート（cargo build/test）をスキップ
#   bash scripts/trace-check.sh --strict          # ONNX モデルが無い場合を FAIL 扱い
#                                                 （デフォルトは WARN 扱いでスキップ）

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

NO_SEMANTIC=0
NO_CONFORMANCE=0
STRICT=0
for arg in "$@"; do
  case "$arg" in
    --no-semantic)    NO_SEMANTIC=1 ;;
    --no-conformance) NO_CONFORMANCE=1 ;;
    --strict)         STRICT=1 ;;
    -h|--help)
      awk 'NR>1 && /^#/{sub(/^# ?/, ""); print; next} NR>1{exit}' "$0"
      exit 0
      ;;
    *)
      echo "不明なオプション: $arg" >&2
      exit 2
      ;;
  esac
done

red()    { printf "\033[31m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }
green()  { printf "\033[32m%s\033[0m\n" "$*"; }

FAIL=0
WARN=0

ONNX_MODEL_PATH="models/paraphrase-multilingual-MiniLM-L12-v2/model.onnx"

# ---------- [1/6] 第 1 層 ----------
echo "[1/6] traceability-engine check --formal（第 1 層: 形式検証 + ChainIntegrity escalate）"
FORMAL_OUT=$(traceability-engine check --formal 2>&1)
formal_exit=$?
printf '%s\n' "$FORMAL_OUT"
if [[ $formal_exit -ne 0 ]]; then
  red "  公式 check --formal が失敗しました（exit=$formal_exit）"
  FAIL=$((FAIL+1))
fi

# F2 escalation（DevProc_V4.1 §7/§12, 08-gates 契約適合ゲート）:
#   ChainIntegrity は Severity::Warning であり、エンジンの exit は ERROR 数のみで決まる。
#   よって配送軸/機能軸のチェーン断裂（孤児・未連結）を検出しても check --formal は exit=0 を返す
#   ＝「検出」だけではゲートにならない。ここで [WARNING] ChainIntegrity 行を grep し、
#   1 件でもあれば明示的に FAIL へ escalate する（再現: DevProc_V4.1/spikes/multi-area-2026-06-14/）。
CHAIN_WARN=$(printf '%s\n' "$FORMAL_OUT" | grep -E '\[WARNING\][[:space:]]*ChainIntegrity' || true)
if [[ -n "$CHAIN_WARN" ]]; then
  chain_warn_n=$(printf '%s\n' "$CHAIN_WARN" | grep -c . )
  red "  ChainIntegrity WARNING を $chain_warn_n 件検出（チェーン断裂＝孤児/未連結）。exit=0 でも escalate して FAIL:"
  while IFS= read -r line; do red "    $line"; done <<< "$CHAIN_WARN"
  FAIL=$((FAIL+1))
fi

# ---------- [2/6] 第 2 層 semantic ----------
echo ""
echo "[2/6] traceability-engine check（第 2 層: semantic）"

if [[ $NO_SEMANTIC -eq 1 ]]; then
  yellow "  --no-semantic 指定によりスキップ"
elif [[ ! -f "$ONNX_MODEL_PATH" ]]; then
  if [[ $STRICT -eq 1 ]]; then
    red "  ONNX モデル ($ONNX_MODEL_PATH) が見つかりません（--strict なので FAIL）"
    FAIL=$((FAIL+1))
  else
    yellow "  ONNX モデル ($ONNX_MODEL_PATH) が見つからないためスキップ"
    yellow "  semantic 検証を有効化するには huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 から"
    yellow "  model.onnx と tokenizer.json を $(dirname "$ONNX_MODEL_PATH")/ に配置してください"
    WARN=$((WARN+1))
  fi
else
  if ! traceability-engine check; then
    fail_msg=$?
    red "  公式 check（第 2 層含む）が失敗しました（exit=$fail_msg）"
    FAIL=$((FAIL+1))
  fi
fi

# ---------- [3/6] SPEC レベル TDD ゲート ----------
echo ""
echo "[3/6] SPEC レベル TDD ループのゲート"

RED_TPS=$(grep -l "^\*\*ステータス\*\*: red" docs/test-perspectives/*.md 2>/dev/null || true)
if [[ -n "$RED_TPS" ]]; then
  while IFS= read -r f; do red "  red TP: $f"; FAIL=$((FAIL+1)); done <<< "$RED_TPS"
fi

OPEN_GAPS=$(grep -l "^\*\*ステータス\*\*: open$" docs/gap-analysis/*.md 2>/dev/null || true)
if [[ -n "$OPEN_GAPS" ]]; then
  while IFS= read -r f; do red "  open GAP: $f"; FAIL=$((FAIL+1)); done <<< "$OPEN_GAPS"
fi

# ---------- [4/6] 前段ループゲート（ハードルール 9）----------
echo ""
echo "[4/6] 前段ループゲート（各 SPEC の最新 FCR が ACCEPTED か）"

FCR_DIR="docs/frontend-pass/check-results"
SPEC_DIR="docs/specs"

if [[ ! -d "$SPEC_DIR" ]]; then
  yellow "  $SPEC_DIR が存在しないためスキップ（新規プロジェクト初期化前）"
else
  SPEC_FILES=$(ls "$SPEC_DIR"/*.md 2>/dev/null || true)
  if [[ -z "$SPEC_FILES" ]]; then
    yellow "  SPEC ファイルが 0 件、スキップ"
  else
    while IFS= read -r spec_file; do
      # SPEC ID をファイル名から抽出（例: SPEC-VNS-001_xxx.md → SPEC-VNS-001）
      spec_basename=$(basename "$spec_file" .md)
      spec_id=$(echo "$spec_basename" | sed -E 's/^(SPEC-[A-Z]+-[0-9]+).*$/\1/')

      if [[ -z "$spec_id" || "$spec_id" == "$spec_basename" ]]; then
        # ID 形式が認識できない場合は warn
        yellow "  SPEC ID 形式が認識できない: $spec_file"
        WARN=$((WARN+1))
        continue
      fi

      # 前段スキップ ADR の参照があるか確認
      if grep -qE "^\*\*前段スキップ\*\*:\s*ADR-" "$spec_file" 2>/dev/null; then
        green "  $spec_id: 前段スキップ（ADR 参照あり、pass 扱い）"
        continue
      fi

      # 対応する FCR を探す（FCR-<AREA>-NNN.md で **対象 SPEC**: <spec_id> を持つもの）
      if [[ ! -d "$FCR_DIR" ]]; then
        red "  $spec_id: FCR ディレクトリ ($FCR_DIR) が存在しない。前段ループ未実施。"
        FAIL=$((FAIL+1))
        continue
      fi

      MATCHING_FCRS=$(grep -l "^\*\*対象 SPEC\*\*:\s*$spec_id\b" "$FCR_DIR"/*.md 2>/dev/null || true)

      if [[ -z "$MATCHING_FCRS" ]]; then
        red "  $spec_id: FCR が 0 件。前段ループ未実施。"
        FAIL=$((FAIL+1))
        continue
      fi

      # FCR ID の連番が最大のものを「最新」とみなす
      latest_fcr=$(echo "$MATCHING_FCRS" | xargs -n1 basename 2>/dev/null | sort -V | tail -n1)
      latest_fcr_path="$FCR_DIR/$latest_fcr"

      status=$(grep -E "^\*\*frontend_status\*\*:" "$latest_fcr_path" 2>/dev/null | head -n1 | sed -E 's/^\*\*frontend_status\*\*:\s*//' | awk '{print $1}')

      case "$status" in
        ACCEPTED)
          # green "  $spec_id: ACCEPTED ($latest_fcr)"
          ;;
        NEEDS_QUESTIONNAIRE)
          red "  $spec_id: 最新 FCR ($latest_fcr) が NEEDS_QUESTIONNAIRE。前段ループを継続"
          FAIL=$((FAIL+1))
          ;;
        *)
          red "  $spec_id: 最新 FCR ($latest_fcr) の frontend_status が不明 ('$status')"
          FAIL=$((FAIL+1))
          ;;
      esac
    done <<< "$SPEC_FILES"
  fi
fi

# ---------- [5/6] レイヤ汚染検査（ハードルール 10）----------
echo ""
echo "[5/6] レイヤ汚染検査（具体側 RBD/SEQD に言語固有要素なし）"

RBD_DIR="docs/robustness-detail"
SEQD_DIR="docs/sequence-detail"

# レイヤ汚染検出パターン(grep -E 正規表現)
# 検出される場合は DD に移動する違反
# 各パターンは (パターン名|正規表現|説明) で扱う

check_layer_pollution() {
  local target_file="$1"
  local violations=0

  # 1. snake_case 関数呼び出し: foo_bar(, place_order(
  #    ただし mermaid のラベルや日本語混じりは検出しない
  local snake_hits
  snake_hits=$(grep -nE '\b[a-z][a-z0-9_]*[a-z0-9]\(' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--|^\s*\*|^\s*-' | \
    grep -vE '\b(participant|note|loop|alt|opt|par|else|end|activate|deactivate|graph)\b' || true)
  if [[ -n "$snake_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [snake_case 関数呼び出し] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$snake_hits"
  fi

  # 2. camelCase 関数呼び出し: placeOrder(, getUserName(
  local camel_hits
  camel_hits=$(grep -nE '\b[a-z][a-zA-Z0-9]*[A-Z][a-zA-Z0-9]*\(' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--|^\s*\*|^\s*-' || true)
  if [[ -n "$camel_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [camelCase 関数呼び出し] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$camel_hits"
  fi

  # 3. ジェネリック型表記: Result<T,E>, Vec<T>, Arc<Mutex<T>>, Option<T>
  local generic_hits
  generic_hits=$(grep -nE '\b(Result|Option|Vec|Arc|Box|Rc|Mutex|RwLock|HashMap|BTreeMap)<' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--' || true)
  if [[ -n "$generic_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [言語固有ジェネリック型] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$generic_hits"
  fi

  # 4. async/await キーワード
  local async_hits
  async_hits=$(grep -nE '\b(async\s+fn|async\s+function|await\b)' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--' || true)
  if [[ -n "$async_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [async/await 機構] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$async_hits"
  fi

  # 5. Rust 修飾子: pub, pub(crate)
  local rust_mod_hits
  rust_mod_hits=$(grep -nE '\bpub(\(crate\)|\(super\))?\s+' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--' || true)
  if [[ -n "$rust_mod_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [Rust 修飾子] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$rust_mod_hits"
  fi

  # 6. crate/module 経由呼び出し: tokio::, std::, sqlx::
  local module_hits
  module_hits=$(grep -nE '\b[a-z][a-z_]+::' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--' || true)
  if [[ -n "$module_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [crate/module 識別子] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$module_hits"
  fi

  # 7. import / use 文
  local import_hits
  import_hits=$(grep -nE '^(\s*)(import\s|use\s|from\s+\w+\s+import|require\s*\()' "$target_file" 2>/dev/null | \
    grep -vE '^\s*<!--' || true)
  if [[ -n "$import_hits" ]]; then
    while IFS= read -r line; do
      printf "\033[31m    [import/use 文] %s: %s\033[0m\n" "$target_file" "$line" >&2
      violations=$((violations+1))
    done <<< "$import_hits"
  fi

  echo "$violations"
}

TOTAL_POLLUTION=0
for layer_dir in "$RBD_DIR" "$SEQD_DIR"; do
  if [[ ! -d "$layer_dir" ]]; then
    yellow "  $layer_dir が存在しないためスキップ"
    continue
  fi
  layer_files=$(ls "$layer_dir"/*.md 2>/dev/null || true)
  if [[ -z "$layer_files" ]]; then
    continue
  fi
  while IFS= read -r f; do
    v=$(check_layer_pollution "$f")
    TOTAL_POLLUTION=$((TOTAL_POLLUTION+v))
  done <<< "$layer_files"
done

if [[ $TOTAL_POLLUTION -gt 0 ]]; then
  red "  レイヤ汚染を $TOTAL_POLLUTION 件検出。違反は DD に移動するか、概念表現に書き直す。"
  FAIL=$((FAIL+1))
fi

# ---------- [6/6] 契約適合ゲート（配送層、DevProc_V4.1 §12 / P-1） ----------
echo ""
echo "[6/6] 契約適合ゲート（実バイナリ E2E: TC-CLI-001 / TC-MCP-001）"

if [[ $NO_CONFORMANCE -eq 1 ]]; then
  yellow "  --no-conformance 指定によりスキップ"
elif ! command -v cargo >/dev/null 2>&1; then
  yellow "  cargo が見つからないためスキップ（契約適合ゲートは Rust ツールチェーン必須）"
  WARN=$((WARN+1))
else
  # ① ビルド（実行ファイル legixy = 契約サーフェス）。
  if ! cargo build -p legixy-cli >/dev/null 2>&1; then
    red "  legixy-cli ビルド失敗（契約サーフェスがビルドできない）"
    FAIL=$((FAIL+1))
  else
    # ② TC[DLV] 実バイナリ E2E（TC-CLI-001 = crates/legixy-cli/tests/cli.rs）。
    #    LGX-COMPAT-001 §3 の凍結契約（19 サブコマンド・終了コード・グローバルフラグ・config）への適合を
    #    実 spawn で検証する（機能チェーン GREEN とは独立した契約適合の経路）。
    if cargo test -p legixy-cli --test cli >/dev/null 2>&1; then
      green "  TC-CLI-001（CLI 契約適合 E2E）pass"
    else
      red "  TC-CLI-001（CLI 契約適合 E2E）失敗 — 契約サーフェスが凍結契約に適合していない"
      FAIL=$((FAIL+1))
    fi
  fi
  # ③ TC-MCP-001（ts-mcp 実バイナリ E2E）。node/npm + 依存導入済みのときのみ。
  if [[ -d ts-mcp/node_modules ]] && command -v npm >/dev/null 2>&1; then
    if (cd ts-mcp && npm test >/dev/null 2>&1); then
      green "  TC-MCP-001（MCP 契約適合 E2E）pass"
    else
      red "  TC-MCP-001（MCP 契約適合 E2E）失敗"
      FAIL=$((FAIL+1))
    fi
  else
    yellow "  ts-mcp/node_modules 不在 or npm なし → TC-MCP-001 スキップ（npm ci 後に有効）"
    WARN=$((WARN+1))
  fi
fi

# ---------- サマリ ----------
echo ""
echo "======================================"
TP_GREEN=$(grep -l "^\*\*ステータス\*\*: green" docs/test-perspectives/*.md 2>/dev/null | wc -l)
TP_RED=$(grep -l "^\*\*ステータス\*\*: red"   docs/test-perspectives/*.md 2>/dev/null | wc -l)
GAP_CL=$(grep -l "^\*\*ステータス\*\*: closed" docs/gap-analysis/*.md 2>/dev/null | wc -l)
GAP_OP=$(grep -l "^\*\*ステータス\*\*: open$"  docs/gap-analysis/*.md 2>/dev/null | wc -l)
ADR_CT=$(ls docs/adr/*.md 2>/dev/null | wc -l)
SPEC_CT=$(ls docs/specs/*.md 2>/dev/null | wc -l)
QSET_CT=$(ls docs/frontend-pass/questionnaires/*.md 2>/dev/null | wc -l)
SPP_CT=$(ls docs/spec-patches/*.md 2>/dev/null | wc -l)
FCR_ACC=$(grep -l "^\*\*frontend_status\*\*:\s*ACCEPTED" docs/frontend-pass/check-results/*.md 2>/dev/null | wc -l)
FCR_NQ=$(grep -l "^\*\*frontend_status\*\*:\s*NEEDS_QUESTIONNAIRE" docs/frontend-pass/check-results/*.md 2>/dev/null | wc -l)
RBA_CT=$(ls docs/robustness-abstract/*.md 2>/dev/null | wc -l)
SEQA_CT=$(ls docs/sequence-abstract/*.md 2>/dev/null | wc -l)
RBD_CT=$(ls docs/robustness-detail/*.md 2>/dev/null | wc -l)
SEQD_CT=$(ls docs/sequence-detail/*.md 2>/dev/null | wc -l)

echo " SPEC: $SPEC_CT, QSET: $QSET_CT, SPP: $SPP_CT, FCR: ACCEPTED=$FCR_ACC NEEDS_Q=$FCR_NQ"
echo " RBA: $RBA_CT, SEQA: $SEQA_CT, RBD: $RBD_CT, SEQD: $SEQD_CT, レイヤ汚染: $TOTAL_POLLUTION"
echo " TP: green=$TP_GREEN red=$TP_RED, GAP: closed=$GAP_CL open=$GAP_OP, ADR: $ADR_CT"
echo "======================================"

if [[ $FAIL -gt 0 ]]; then
  red "FAIL: $FAIL 件"
  exit 1
fi

if [[ $WARN -gt 0 ]]; then
  yellow "PASS（警告 $WARN 件あり）"
else
  green "PASS"
fi
exit 0
