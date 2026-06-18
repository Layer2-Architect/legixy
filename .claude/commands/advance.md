---
name: advance
description: 現在地のゲートを判定して次フェーズに進めるか確認する。引数で stage 名を指定（spec-to-uc, uc-to-rba, iconix-abstract, iconix-concrete, dd-to-ts, ts-to-tc-red, tc-red-to-src, src-to-tc-green, release）。
argument-hint: <stage>
---

ローカル運用での AI レビュアゲートを起動する。`https://github.com/OWNER/spec-compiling-pipeline/blob/v1.0.0/ja/08-gates.md` §17 のローカル運用フローに従う。

## 引数

`$ARGUMENTS` に stage 名が入る。受け付ける値:

- `spec-to-uc` — SPEC → UC ゲート
- `uc-to-rba` — UC → RBA ゲート
- `iconix-abstract` — RBA → SEQA（抽象層 GREEN 確定）
- `iconix-concrete` — RBD → SEQD → DD（具体層 + DD）
- `dd-to-ts` — DD → TS ゲート
- `ts-to-tc-red` — TS → TC[RED] ゲート
- `tc-red-to-src` — TC[RED] → SRC ゲート
- `src-to-tc-green` — SRC → TC[GREEN] ゲート
- `release` — リリースゲート（AT 通過）

## 実行手順

以下を順に実行する:

### Step 1: diff スコープの確定

直近の phase tag からの差分を確認する:

```bash
PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -n "$PREV_TAG" ]; then
  git diff "${PREV_TAG}..HEAD" --stat
else
  echo "phase tag なし。リポジトリ全体をレビュー対象とします。"
  git diff --stat HEAD~10..HEAD 2>/dev/null || git ls-files
fi
```

### Step 2: 機械検証層の実行

```bash
bash scripts/trace-check.sh
```

失敗した場合はそこで停止。Author に指示を出して修正後に再 `/advance` する。

### Step 3: Reviewer subagent の起動

機械検証が pass したら、Reviewer subagent を spawn する。subagent には以下を渡す:

- stage 名（`$ARGUMENTS`）
- diff スコープ（Step 1 の出力）
- レビュー対象成果物の path 一覧

Reviewer は `https://github.com/OWNER/spec-compiling-pipeline/blob/v1.0.0/ja/review-guidelines/perspectives.md` の 9 観点を順次チェックし、末尾に `<!-- VERDICT:APPROVE -->` / `<!-- VERDICT:REQUEST_CHANGES -->` / `<!-- VERDICT:NEEDS_HUMAN -->` のいずれかを出力する。

Reviewer 完了時に `SubagentStop` hook が `scripts/extract-verdict.sh` を走らせて `.devproc/verdict.log` に append する。

### Step 4: 最新 VERDICT の読み取りと提案

```bash
VERDICT=$(bash scripts/check-latest-verdict.sh)
case "$VERDICT" in
  APPROVE)
    echo "✓ APPROVE — 次フェーズに進めます。"
    echo "推奨 tag: v<N>-${stage}-green"
    echo "次のコマンド: git tag v<N>-${stage}-green"
    ;;
  REQUEST_CHANGES)
    echo "✗ REQUEST_CHANGES — 指摘を読んで Author に再指示してください。"
    echo "詳細: tail -50 .devproc/reviewer-output/$(ls -t .devproc/reviewer-output/ | head -1)"
    ;;
  NEEDS_HUMAN)
    echo "△ NEEDS_HUMAN — AI Approve 不可領域。開発者の判断で tag を打つかどうかを決めてください。"
    echo "Stage: ${stage}"
    echo "対象が SPEC / UC / リリースの場合はハードルール 1 により人間判断必須。"
    ;;
esac
```

## 注意

- `/advance` は **読み取り専用**。tag を打つ操作は開発者が手動で行う（自動 tag 付与は採用しない、`08-gates.md` §17 参照）
- VERDICT が `REQUEST_CHANGES` の場合、Author に再指示する前に **指摘内容を読んで** どの観点・どの severity かを把握する
- `NEEDS_HUMAN` は「ハードルール 1 で AI Approve 不可」のサイン。SPEC / UC / リリース判断・品質基準変更がこれに該当
- ローカル運用では tag を打つ瞬間が PR ボタン押下の代替。tag を打たないと次フェーズに進めない規律を保つ
