---
name: spec-change
description: 仕様変更イベント。既存 SPEC / UC を意図的に変更する場合の 8 ステップフローを実行する。詳細は 10-modification-events.md §4.2。
argument-hint: <SPEC-ID-or-UC-ID>
---

仕様変更イベント (`spec-change`) のフローを起動する。`https://github.com/OWNER/spec-compiling-pipeline/blob/v1.0.0/ja/10-modification-events.md` §4.2 の 8 ステップに従う。

## 引数

`$ARGUMENTS` には変更対象の ID が入る (例: `SPEC-VNS-005`、`UC-VNS-012`)。

## 実行手順

### Step 1: イベントログへの記録

```bash
PARENT_CYCLE=$(tail -1 .devproc/event-log.txt 2>/dev/null | awk -F'|' '{print $5}' | tr -d ' ' || echo "")
if [ -n "$PARENT_CYCLE" ]; then
  CYCLE=$((PARENT_CYCLE + 1))
  PARENT_LABEL="cycle-${PARENT_CYCLE}"
else
  CYCLE=1
  PARENT_LABEL="-"
fi

mkdir -p .devproc
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "${TIMESTAMP} | spec-change | ${ARGUMENTS} | parent: ${PARENT_LABEL} | cycle: ${CYCLE} | pending | -" >> .devproc/event-log.txt
```

### Step 2: 変更対象ノードの確定

仕様変更は常に開発者が直接ノードを指定するため、確定的 (`defect-fix` のような embedding 検索は不要)。

```bash
TARGET="$ARGUMENTS"
# 存在確認
if ! traceability-engine node-exists "$TARGET" 2>/dev/null; then
  echo "✗ ノード $TARGET が graph.toml に存在しません"
  exit 1
fi
```

### Step 3: impact で下流影響範囲の特定

```bash
traceability-engine impact "$TARGET" --max-depth 10 > .devproc/event-output/${TIMESTAMP}-impact.txt
```

影響範囲が広い場合 (たとえば 20 ノード以上)、変更のスコープが過大であることを警告。

### Step 4: 影響範囲のレビュー (Reviewer subagent)

Reviewer subagent を spawn する。subagent に以下を渡す:

- イベント種別: `spec-change`
- 変更対象: `$TARGET`
- 影響範囲: Step 3 の出力
- 変更内容の説明 (開発者が用意した変更案、あれば)

Reviewer は以下の観点で変更内容と影響範囲を評価する:

- `[Trace]`: 下流追従の必要性
- `[Consistency]`: 既存連鎖との整合性
- `[Layer]`: レイヤ汚染の可能性
- `[Spec-TDD]`: TP / GAP への影響

末尾に VERDICT マーカーを出力。SPEC 変更はハードルール 1 により、Reviewer の VERDICT は最良でも `NEEDS_HUMAN` 止まり。

### Step 5: 人間承認 (ハードルール 1)

SPEC / UC 変更は人間判断必須。開発者が以下を確認:

- 変更内容そのもの
- 影響範囲の妥当性
- 下流追従修正の量
- ADR 起票の必要性 (重大な変更の場合)

承認する場合、開発者が:

```bash
git tag spec-change-approved-${TIMESTAMP}
```

を打つ。tag を打つ操作が承認の不可逆な意思表示 (PR ボタンの代替)。

### Step 6: 変更の適用

承認後、対象 SPEC / UC を修正する。

修正規律:

- 変更履歴を SPEC / UC 文書内の「改訂履歴」セクションに追記
- ADR-NNN を起票する場合は `docs/adr/` 配下に新規作成
- commit message に `spec-change:` プレフィックスと、`Recurrence:` フッタを付ける

### Step 7: インクリメンタル再構築

`10-modification-events.md` §5 のフローに従い、下流影響範囲を選択的に再生成する。

具体的には Step 3 の `impact` 出力から、影響を受ける UC / RB / SEQ / DD / TS / TC / SRC を順次:

1. Author subagent で再生成
2. AI Reviewer で評価
3. APPROVE / REQUEST_CHANGES / NEEDS_HUMAN の判定
4. NEEDS_HUMAN は開発者承認を待つ

### Step 8: ゲート再評価

影響を受けたゲート (08-gates.md の §3〜§12) を再実行する。

```bash
# 影響を受けたフェーズの phase tag を確認
LAST_TAG=$(git describe --tags --abbrev=0)
# 必要に応じて、影響範囲のゲートを順次再起動
/advance <stage>  # 影響範囲の各フェーズに対して
```

### 完了処理

イベントログを更新:

```bash
sed -i "$ s|pending|${FINAL_VERDICT}|" .devproc/event-log.txt
```

サイクル数の確認:

```bash
bash scripts/count-fix-cycle.sh
```

## 注意

- 仕様変更は **常に人間承認必須** (ハードルール 1)。AI Reviewer は `APPROVE` を返せず、最良でも `NEEDS_HUMAN`
- 影響範囲が広い場合 (20 ノード以上目安) は、変更のスコープを見直すこと。小さく分けるか、ADR で大規模変更の理由を記録すること
- 仕様変更は実装着手後の SPEC 改変であり、ハードルール 6 に対する例外 (本ハードルールは「気まぐれな変更」を禁止しており、設計判断に基づく変更は許容される)
- 3 サイクル超過した場合、設計レベルでの見直しを検討する
