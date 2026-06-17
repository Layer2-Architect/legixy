Document ID: SEQA-LGX-012

# SEQA-LGX-012: ベースライン凍結管理 のドメイン相互作用

**親 RBA**: RBA-LGX-012
**親 UC**: UC-LGX-012
**レイヤ**: 抽象側（ドメインレベル、言語非依存）

> **記述規律**: RBA-LGX-012 で識別したドメイン主語をレーンとして、UC-LGX-012 のフロー（基本/代替/例外）を時系列で展開する。メッセージは自然言語（ドメイン語彙）。関数名・API 名・引数型・言語固有同期機構は書かない（`04-iconix-layer.md` §4）。本 SEQA は UC ⇄ RBA ⇄ SEQA の Jacobson 流三者整合性を確定する。

---

## 1. UC text（並列配置）

UC-LGX-012 基本フロー（SEQA メッセージと 1:1 対応）:

```
1. アクターが `legixy snapshot create --label <L>` を実行する（label は省略可）
2. システムが embeddings ストアの現行全行を単一トランザクションでスナップショット領域へ複製する
3. システムが一意な snapshot_id（snap- プレフィクス）を発行し、text / --json で返す
4. アクターが `legixy snapshot list` で凍結済ベースラインを確認する（snapshot_id / label / node_count / taken_at を taken_at 降順で一覧）
5. （後続利用）アクターが UC-LGX-013 のドリフト対比で --against snapshot:<L> の基準点として参照する
6. アクターが `legixy snapshot delete <snapshot_id | label:<L>>` で不要ベースラインを削除する
7. exit 0 で終了

代替:
- 1a. サブコマンド省略 → 使用法誤りとして exit 2
- 2a. 空ストアで create → WARNING(stderr) + exit 0（非永続）
- 4a. list 0 件 → 案内メッセージ / 空配列 + exit 0
- 6a. delete label:<L> で同一 label 複数存在 → taken_at 最新の 1 件へ決定論的に解決して削除
- 6b. delete snapshot_id 指定で該当 0 件 → text: WARNING(stderr)+exit 0 / --json: 警告なし+exit 0
- 6c. delete label:<L> で label 存在しない → ERROR(stderr) + exit 1（6b との非対称は意図的）
```

## 2. 基本フロー（`snapshot create` → `snapshot list` → `snapshot delete`）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C1 as 凍結処理
    participant Bembed as embeddings ストア境界
    participant Bsnap as スナップショット領域境界
    participant Eid as 凍結識別子
    participant Esnap as スナップショット行集合
    participant B2 as 結果出力窓口

    Note over Actor,B2: --- snapshot create ---
    Actor->>B1: スナップショット凍結を要求する（label 付与または省略）
    B1->>C0: スナップショット操作を統括する（サブコマンド: create）
    C0->>C1: 凍結処理を起動する
    C1->>Bembed: embeddings ストアの現行全行を読み取る
    Bembed-->>C1: 現行全行
    C1->>Bsnap: 現行全行を単一トランザクションでスナップショット領域へ複製する
    Bsnap-->>C1: 複製完了
    C1->>Esnap: スナップショット行集合を確定する（content_hash / model_version 含む）
    C1->>Eid: 一意な凍結識別子を発行する（snap- プレフィクス）
    C0->>B2: 凍結識別子と操作結果を渡す
    B2-->>Actor: snapshot_id（stdout）+ exit 0

    Note over Actor,B2: --- snapshot list ---
    Actor->>B1: ベースライン一覧を要求する
    B1->>C0: スナップショット操作を統括する（サブコマンド: list）
    C0->>C2: 一覧処理を起動する
    C2->>Bsnap: 全スナップショット行を読み取る
    Bsnap-->>C2: 全スナップショット行
    C2->>Elist: 一覧情報を組み立てる（taken_at 降順整列）
    C0->>B2: 一覧情報を渡す
    B2-->>Actor: 一覧（snapshot_id / label / node_count / taken_at）+ exit 0

    Note over Actor,B2: --- snapshot delete（snapshot_id 指定）---
    Actor->>B1: スナップショット削除を要求する（snapshot_id 指定）
    B1->>C0: スナップショット操作を統括する（サブコマンド: delete）
    C0->>C3: 削除処理を起動する（snapshot_id 指定）
    C3->>Bsnap: 対象スナップショット行を除去する
    Bsnap-->>C3: 削除完了
    C3->>Edel: 削除結果を確定する（対象識別子 / 削除行数）
    C0->>B2: 削除結果を渡す
    B2-->>Actor: 削除確認（stdout）+ exit 0
```

*参加者 C2（一覧処理）/ Elist（一覧情報）/ C3（削除処理）/ Edel（削除結果）は §3-4 でも再利用する。mermaid の構文制約上 participant 宣言は各ダイアグラム内で行う。*

## 3. 代替フロー

### 代替 1a: サブコマンド省略（`legixy snapshot` のみ）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット操作を要求する（サブコマンド省略）
    B1->>B2: 使用法誤りを通知する
    B2-->>Actor: 使用法案内（stderr）+ exit 2
    Note over B1,B2: サブコマンド省略は スナップショット統括処理 へ渡さずに境界で終端
```

### 代替 2a: 空ストアで create（WARNING + exit 0、非永続）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C1 as 凍結処理
    participant Bembed as embeddings ストア境界
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット凍結を要求する
    B1->>C0: スナップショット操作を統括する（サブコマンド: create）
    C0->>C1: 凍結処理を起動する
    C1->>Bembed: embeddings ストアの現行全行を読み取る
    Bembed-->>C1: 空（行 0 件）
    Note over C1,Bsnap: 複製行 0 件を検出 → スナップショット領域境界への書込みをスキップ（非永続）
    C0->>B2: 空ストア検出結果（snapshot_id 返却・node_count 0 / WARNING）を渡す
    B2-->>Actor: WARNING（stderr）+ snapshot_id / node_count:0（stdout or --json）+ exit 0
    Note over B2,Actor: 返却された snapshot_id は list に現れない（永続化なし）
```

### 代替 4a: list 0 件

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C2 as 一覧処理
    participant Bsnap as スナップショット領域境界
    participant Elist as 一覧情報
    participant B2 as 結果出力窓口

    Actor->>B1: ベースライン一覧を要求する
    B1->>C0: スナップショット操作を統括する（サブコマンド: list）
    C0->>C2: 一覧処理を起動する
    C2->>Bsnap: 全スナップショット行を読み取る
    Bsnap-->>C2: 空（0 件）
    C2->>Elist: 案内メッセージ相当の一覧情報を組み立てる（0 件）
    C0->>B2: 一覧情報（0 件）を渡す
    B2-->>Actor: 案内メッセージ（text）/ 空配列（json）+ exit 0
```

### 代替 6a: delete `label:<L>` で同一 label 複数存在

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant C4 as label 解決処理
    participant Bsnap as スナップショット領域境界
    participant Edel as 削除結果
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を要求する（label:<L> 指定）
    B1->>C0: スナップショット操作を統括する（サブコマンド: delete）
    C0->>C3: 削除処理を起動する（label:<L> 指定）
    C3->>C4: label 参照の解決を依頼する
    C4->>Bsnap: 対象 label の全スナップショット行を問い合わせる
    Bsnap-->>C4: 同一 label の複数行
    Note over C4: taken_at 最新の 1 件へ決定論的に解決する
    C4-->>C3: 解決済み凍結識別子
    C3->>Bsnap: 解決済み識別子の対象行を除去する
    Bsnap-->>C3: 削除完了
    C3->>Edel: 削除結果を確定する
    C0->>B2: 削除結果を渡す
    B2-->>Actor: 削除確認（stdout）+ exit 0
```

### 代替 6b: delete snapshot_id 指定で該当 0 件（非対称: exit 0）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant Bsnap as スナップショット領域境界
    participant Edel as 削除結果
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を要求する（snapshot_id 指定）
    B1->>C0: スナップショット操作を統括する（サブコマンド: delete）
    C0->>C3: 削除処理を起動する（snapshot_id 指定）
    C3->>Bsnap: 対象 snapshot_id の行を除去する
    Bsnap-->>C3: 0 行削除（該当なし）
    C3->>Edel: 削除結果を確定する（deleted_rows: 0）
    C0->>B2: 削除結果（deleted_rows: 0）を渡す
    B2-->>Actor: WARNING(stderr)+exit 0（text）/ {"snapshot_id","deleted_rows":0}(stdout)+exit 0（--json、WARNING なし）
    Note over B2,Actor: snapshot_id 不在は exit 0（label 不在 6c の exit 1 との非対称は意図的）
```

### 代替 6c: delete `label:<L>` で label 存在しない（非対称: exit 1）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant C4 as label 解決処理
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を要求する（label:<L> 指定）
    B1->>C0: スナップショット操作を統括する（サブコマンド: delete）
    C0->>C3: 削除処理を起動する（label:<L> 指定）
    C3->>C4: label 参照の解決を依頼する
    C4->>Bsnap: 対象 label の全スナップショット行を問い合わせる
    Bsnap-->>C4: 該当なし（0 件）
    Note over C4: label 解決失敗を確定する（解決済み識別子を返さない）
    C4-->>C3: label 解決失敗
    C3-->>C0: 削除処理失敗を通知する
    C0->>B2: ERROR（label 不在）を渡す
    B2-->>Actor: ERROR（stderr）+ exit 1
    Note over B2,Actor: label 誤り・project-root 誤りを WARNING で覆い隠さない（SPEC-LGX-010.REQ.02）
```

## 4. 例外フロー

### 例外: スナップショット領域書込み失敗（create トランザクション失敗）

```mermaid
sequenceDiagram
    participant C0 as スナップショット統括処理
    participant C1 as 凍結処理
    participant Bembed as embeddings ストア境界
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    C0->>C1: 凍結処理を起動する
    C1->>Bembed: embeddings ストアの現行全行を読み取る
    Bembed-->>C1: 現行全行
    C1->>Bsnap: 単一トランザクションでスナップショット領域へ複製する
    Bsnap-->>C1: トランザクション失敗（部分書込みなし）
    Note over C1,Bsnap: トランザクション失敗時は原子的にロールバック（スナップショット行集合は不完全状態で残らない）
    C1-->>C0: 凍結失敗を通知する
    C0->>B2: ERROR を渡す
    B2-->>Actor: ERROR（stderr）+ exit 1
```

## 5. 並行性（概念レベル）

`snapshot create` / `snapshot list` / `snapshot delete` はいずれも単一の操作要求として受け付ける。`create` は単一トランザクション複製という UC 明示の制約により原子的に完了する。ドメインレベルで並行に発生する事象はない（各サブコマンドは統括処理の協調下で逐次進む）。並行アクセス時の整合性（複数アクターが同一 label に同時 delete を行う場合等）は NFR-LGX-001 の射程であり本 SEQA 範囲外。

## 6. 整合性確認

- [x] 各メッセージがドメイン語彙で書かれている（関数名・API 名・型なし）
- [x] レーンが RBA-LGX-012 の主語と一致する（スナップショットコマンド受付窓口 / embeddings ストア境界 / スナップショット領域境界 / 結果出力窓口 / スナップショット統括処理 / 凍結処理 / 一覧処理 / 削除処理 / label 解決処理 / 凍結識別子 / スナップショット行集合 / 一覧情報 / 削除結果）
- [x] UC-LGX-012 の基本（Step1-7）/ 代替（1a/2a/4a/6a/6b/6c）/ 例外（create トランザクション失敗）フローを網羅
- [x] Noun-Verb ルール遵守（Actor⇄Boundary / Boundary⇄Control / Control⇄Control / Control⇄Entity のみ。Boundary 同士・Entity 同士・Boundary→Entity・Actor→内部 の直接通信なし）

## 7. コントローラ責務と実行操作の整合（§4.4）

| Control レーン | 概念名が示す責務 | 実行する操作 | 整合 |
|---|---|---|---|
| スナップショット統括処理 | スナップショット操作の統括・サブコマンド振分け | 操作要求を受けサブコマンド種別を判定し凍結/一覧/削除処理のいずれかを起動する | ✓ |
| 凍結処理 | embeddings 全行の単一トランザクション複製・凍結識別子発行 | embeddings ストア境界を読みスナップショット領域境界へ複製し凍結識別子を発行する。空ストア時は非永続 | ✓ |
| 一覧処理 | スナップショット行集合の読取・taken_at 降順整列・一覧情報組立 | スナップショット領域境界を読み一覧情報を組み立てる。0 件時は案内メッセージ相当を組み立てる | ✓ |
| 削除処理 | 対象識別子の削除・削除結果の確定 | label 参照時は label 解決処理に委譲し、解決済み識別子または直接 snapshot_id でスナップショット領域境界の対象行を除去する | ✓ |
| label 解決処理 | label 参照の taken_at 最新優先解決・label 不在時の ERROR 確定 | スナップショット領域境界に問い合わせ最新 1 件へ決定論的に解決する。label 不在時は解決失敗を返す | ✓ |

余剰操作なし（各操作が UC ステップに対応）。Control 間メッセージ（統括 → 各処理、削除処理 → label 解決処理）が UC の振る舞いを実現。

## 8. Jacobson 流三者整合性（UC ⇄ RBA ⇄ SEQA、§11.1）— 確定

| 検査 | 確認内容 | 結果 |
|---|---|---|
| UC ⇄ RBA | UC-LGX-012 の全ステップが RBA-LGX-012 フローに 1:1 対応（RBA-LGX-012 §5） | ✓ |
| RBA ⇄ SEQA | RBA-LGX-012 の主語（B/C/E）が本 SEQA のレーンと一致（13 主語すべて一致）、Noun-Verb ルールが SEQA でも保持（§6） | ✓ |
| UC ⇄ SEQA | UC text 並列配置（§1）、UC-LGX-012 の基本 Step1-7 / 代替 1a・2a・4a・6a・6b・6c / 例外（トランザクション失敗）が §2-4 で網羅 | ✓ |

6b（snapshot_id 不在=exit 0）と 6c（label 不在=exit 1）の非対称性が SEQA 代替フローで明示的に可視化されており、UC の意図（SPEC-LGX-010.REQ.02）と整合する。

**これにより RBA-LGX-012 §8 の Jacobson 三者整合性「保留」が解消される。**

## 9. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版。UC-LGX-012 / RBA-LGX-012 の時系列展開。基本（create/list/delete）/ 代替（1a/2a/4a/6a/6b/6c）/ 例外（create トランザクション失敗）を網羅。6b(exit 0) と 6c(exit 1) の非対称を SEQA で明示。Jacobson 流三者整合性を確定（RBA-012 §8 保留解消）。Control 責務⇄操作の整合（§4.4）確認 |
