---
name: defect-fix
description: 不具合修正イベント。SRC レベルでの不具合検出から、上流ドキュメントの不備検査、修正、インクリメンタル再構築までの 10 ステップフローを実行する。詳細は 10-modification-events.md §4.1。
argument-hint: <SRC-ID-or-symptom-description>
---

不具合修正イベント (`defect-fix`) のフロー全体を起動する。`https://github.com/Layer2-Architect/SPEC-compiling-pipeline/blob/v1.0.0/ja/10-modification-events.md` §4.1 の 10 ステップに従う。

## 引数

`$ARGUMENTS` には以下のいずれかが入る:

- 確定的な起点 (例: `SRC-VNS-042`、`DD-VNS-005`)
- 不確定な症状記述 (例: `"保存ボタンを押しても history に記録されない"`)

## 実行手順

### Step 1: 不具合の検出記録

イベントログに新規エントリを追加する。サイクル数の初期値は親イベントがあれば +1、なければ 1:

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
echo "${TIMESTAMP} | defect-fix | ${ARGUMENTS} | parent: ${PARENT_LABEL} | cycle: ${CYCLE} | pending | -" >> .devproc/event-log.txt
```

### Step 2: 起点 SRC の特定

`$ARGUMENTS` の形式により分岐:

- **確定的な ID** (正規表現で `^[A-Z]+-[A-Z]+-\d+$` にマッチ): そのまま Step 3 へ
- **症状記述**: traceability-engine で embedding 検索を実行

```bash
if echo "$ARGUMENTS" | grep -qE "^[A-Z]+-[A-Z]+-[0-9]+$"; then
  TARGET="$ARGUMENTS"
else
  # 候補を 5 件まで列挙
  CANDIDATES=$(traceability-engine semantic-search --query "$ARGUMENTS" --limit 5 2>/dev/null || true)
  echo "起点 SRC 候補:"
  echo "$CANDIDATES"
  echo ""
  echo "確信度が拮抗しています。どの候補を起点としますか?"
  # 開発者の選択を待つ (このコマンド内では先に進めない)
  exit 0
fi
```

### Step 3: trace --upstream で関連ドキュメント収集

```bash
traceability-engine trace --upstream "$TARGET" --max-depth 10 > .devproc/event-output/${TIMESTAMP}-upstream.txt
```

連鎖が長すぎる (max-depth に到達) 場合は警告を出力。

### Step 4: 上流ドキュメントの不備検査 (Reviewer subagent)

Reviewer subagent を spawn する。subagent に以下を渡す:

- イベント種別: `defect-fix`
- 起点ノード: `$TARGET`
- 探索範囲: Step 3 の出力
- 不具合の症状: `$ARGUMENTS`

Reviewer は以下の観点で範囲全体を順次検査する:

- `[Consistency]`: 三者整合性違反が不具合原因か
- `[Coverage]`: 未テスト観点が不具合原因か
- `[Doc]`: DD と SRC の乖離が不具合原因か
- `[Layer]`: レイヤ汚染が不具合原因か
- `[AI-Antipattern]` §G: 不具合修正フロー特有の罠 (浅い読解、過剰修正など)

末尾に VERDICT マーカーを出力。`SubagentStop` hook が `extract-verdict.sh` で `.devproc/verdict.log` に追記する。

### Step 5: 原因の選別

Reviewer の検出した不備のうち、本当に当該不具合の原因と判断できるものを開発者が選別する。

`.devproc/event-output/{timestamp}/` 配下に出力された Reviewer findings を読み、開発者が:

- `findings/accepted/` ディレクトリに移動 → 修正対象として確定
- `findings/rejected/` ディレクトリに移動 → false positive として除外
- `findings/deferred/` ディレクトリに移動 → 別イベントで対処

### Step 6: ドキュメント修正

`accepted/` 配下の各 findings に対し、ドキュメント修正を実行する。

原則:

- **SRC を直接 patch しない**: 上流ドキュメントを修正し、Step 8 のインクリメンタル再構築で下流を更新する
- **修正は 1 つずつ commit**: 各修正に「Recurrence:」フッタを付ける (pre-commit hook で強制)
- **API 凍結部分の DD は人間判断必須**: NEEDS_HUMAN VERDICT を確認

### Step 7: impact で再構築範囲の特定

```bash
for fixed_doc in $(git diff --name-only HEAD~1); do
  traceability-engine impact "$(infer-id-from-path $fixed_doc)" --max-depth 5
done > .devproc/event-output/${TIMESTAMP}-rebuild-range.txt
```

### Step 8: インクリメンタル再構築

10-modification-events.md §5 に従う。`scripts/incremental-rebuild.sh` があれば呼び出し、なければ手動で:

1. 影響範囲の各下流ノードを順次再評価
2. 変更前の上流に依存していた部分を identify
3. 当該部分を Author subagent で再生成
4. 各再生成ノードに対し AI Reviewer 発火
5. APPROVE になるまで反復

### Step 9: traceability-engine check

```bash
traceability-engine check
```

`check` が新たな不整合を検出した場合は再帰イベント発火 (Step 1 に戻る、ただし `cycle` は +1)。

### Step 10: [Recurrence] 観点による再発防止

`review-guidelines/perspectives.md` の [Recurrence] 観点を発火する。以下のいずれかを選択し、commit message の footer に明記:

```
Recurrence: trace-check に追加
Recurrence: perspectives 昇格
Recurrence: ガイドライン追加
Recurrence: ADR で例外として記録
Recurrence: 何もしない (<理由>)
```

pre-commit hook が `fix:` で始まる commit に `Recurrence:` ヘッダが存在することを検証する。

### 完了処理

イベントログの末尾エントリを更新:

```bash
# pending → APPROVE / REQUEST_CHANGES / NEEDS_HUMAN
sed -i "$ s|pending|${FINAL_VERDICT}|" .devproc/event-log.txt
sed -i "$ s|| -$|| .devproc/event-output/${TIMESTAMP}|" .devproc/event-log.txt
```

サイクル数を確認:

```bash
bash scripts/count-fix-cycle.sh
```

3 サイクルを超過していたら警告メッセージを表示する (10-modification-events.md §6.2)。

## 注意

- 本フロー全体は **読み取り専用ではない**。Step 6 でドキュメントを修正する
- ハードルール 6「仕様書とテストコードは実装着手後に変更しない」と矛盾しない (§4.1.2 参照)
- 3 サイクル超過した場合、個別修正を継続するのではなく ADR を起票して設計レベルで見直す
- Reviewer の検出した不備を全て自動修正しない。Step 5 で開発者が選別する規律を守る
