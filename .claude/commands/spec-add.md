---
name: spec-add
description: 仕様追加イベント。既存システムに新規 SPEC を追加する場合のフロー (前段ループ + 衝突検査 + 接続点特定 + 新規連鎖) を実行する。詳細は 10-modification-events.md §4.3。
argument-hint: <description-or-file-path>
---

仕様追加イベント (`spec-add`) のフローを起動する。`https://github.com/OWNER/spec-compiling-pipeline/blob/v1.0.0/ja/10-modification-events.md` §4.3 の 7 ステップに従う。

## 引数

`$ARGUMENTS` には以下のいずれかが入る:

- 新機能の説明文 (例: `"履歴の選択的エクスポート機能"`)
- 既に起草済みの SPEC ファイルパス (例: `docs/specs/SPEC-VNS-015_draft.md`)

## 実行手順

### Step 1: イベントログへの記録

```bash
mkdir -p .devproc
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "${TIMESTAMP} | spec-add | ${ARGUMENTS} | parent: - | cycle: 1 | pending | -" >> .devproc/event-log.txt
```

`spec-add` は通常、親イベントを持たない (修正フローの再帰から発火することは稀)。

### Step 2: SPEC の起票

引数が説明文の場合、`templates/SPEC-template.md` からコピーして新規 SPEC を起草する:

```bash
NEW_ID="SPEC-<AREA>-$(next-spec-id)"
cp templates/SPEC-template.md docs/specs/${NEW_ID}_<short-name>.md
```

引数がファイルパスの場合、それを起点とする。

### Step 3: 前段ループの実行

`03a-frontend-pass.md` の前段ループを実行する:

```
Raw SPEC → QSET → SPP → FCR → Accepted SPEC
```

具体的には:

1. Raw SPEC に対する Critic ペアによる質問群 (QSET) を生成
2. 開発者が質問群に回答 → Spec Patches (SPP) を作成
3. SPP を SPEC に統合
4. Front-end Check Report (FCR) を生成
5. `frontend_status: ACCEPTED` を確定

この間、SPEC は `frontend_status: DRAFT` で graph.toml には追加しない。

### Step 4: 既存 SPEC との衝突検査 (Reviewer subagent)

Reviewer subagent を spawn する。subagent に以下を渡す:

- イベント種別: `spec-add`
- 新 SPEC 内容: Step 3 の Accepted SPEC
- 既存 SPEC 群: `docs/specs/` 配下の全 SPEC

Reviewer は以下の観点で衝突検査:

- `[Consistency]`: 既存 SPEC と矛盾する記述がないか
- `[Layer]`: 既存ドメイン用語との一致 (新しい用語を導入していないか)
- `[Coverage]`: 既存 SPEC で既に網羅されている領域への重複追加でないか
- `[Spec-TDD]`: TP / GAP との関係

末尾に VERDICT マーカー。新規 SPEC は人間判断必須のため、最良でも `NEEDS_HUMAN`。

### Step 5: 接続点の特定

新 SPEC が既存連鎖のどの UC / RB と接続するかを embedding で推定:

```bash
traceability-engine suggest-connections --source "$NEW_SPEC_ID" --target-types "UC,RB"
```

候補が出る場合は、開発者が選択して確定する。候補がない場合は、新規の独立した連鎖として扱う。

### Step 6: 人間承認 (ハードルール 1)

SPEC 追加は人間判断必須。開発者が以下を確認:

- 新 SPEC の内容
- 既存との衝突がないこと (Step 4 の Reviewer findings)
- 接続点の妥当性 (Step 5 の選択結果)

承認する場合、`frontend_status: ACCEPTED` を確定し、graph.toml に新 SPEC ノードを追加する:

```bash
git tag spec-add-accepted-${TIMESTAMP}
```

### Step 7: 新規連鎖の構築

新 SPEC が確定したら、通常の chain (`SPEC → UC → RB → SEQ → DD → TS → TC → SRC`) を起動する。

これは `/advance spec-to-uc` 以降の通常フローに移行する。`spec-add` イベントとしては Step 7 でフローを終了し、以降は新規作成と同じ扱い。

### 完了処理

```bash
sed -i "$ s|pending|${FINAL_VERDICT}|" .devproc/event-log.txt
```

## 注意

- 仕様追加は **常に人間承認必須** (ハードルール 1)
- 既存 SPEC との衝突は Reviewer が見逃すパターンがあるため、開発者が改めて目視確認すること
- 接続点が複数候補に拮抗する場合、ADR で選択理由を記録すること
- Step 3 の前段ループは Raw SPEC が大きい場合に時間がかかる。短く分割するか、複数の `spec-add` イベントに分けることを検討
- 新規連鎖の構築 (Step 7) は通常の chain ゲートに従う。`spec-add` 固有のフローは Step 6 までで終了
