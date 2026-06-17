Document ID: SEQD-LGX-012

# SEQD-LGX-012: ベースライン凍結管理 のクラス間メッセージング

**親 RBD**: RBD-LGX-012
**親 SEQA**: SEQA-LGX-012 / **親 UC**: UC-LGX-012
**レイヤ**: 具体側（クラス図レベル、言語非依存）

> **記述規律**: RBD-LGX-012 で識別したクラスをレーンとして、操作呼び出しの時系列を描く。**操作呼び出しは操作名（人間の言語）**。関数名・引数具体型・戻り型・言語固有同期機構は書かない（DD で確定）。本 SEQD は **Behavior Allocation**（どのクラスがどの操作を担うか）を確定する。
>
> **ハードルール 10**: 命名規則に従う関数呼び出し・言語固有のジェネリック型・並行修飾子・モジュール識別子が混入したら違反。`scripts/trace-check.sh` [5/5] が検出する。本ファイルは禁止トークンを literal で引用しない（記述的に書く）。

---

## 1. 基本フロー（`snapshot create` → `snapshot list` → `snapshot delete`）

### 1-1. snapshot create

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

    Actor->>B1: スナップショット凍結を受け付ける(ラベル参照)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C1: 凍結を実行する(ラベル参照)
    C1->>Bembed: ストアの存在を確認する()
    Bembed-->>C1: 真偽
    C1->>Bembed: 現行全行を読み取る()
    Bembed-->>C1: 行集合
    C1->>C1: 空ストアを検出する(行集合)
    C1->>Bsnap: 現行全行を単一トランザクションで複製する(行集合)
    Bsnap-->>C1: 複製結果
    C1->>Eid: 識別子を取り出す()
    Eid-->>C1: 識別子
    C1->>Esnap: 行集合の件数を取り出す()
    Esnap-->>C1: 数値
    C1-->>C0: 凍結操作結果
    C0->>B2: 操作結果を出力する(凍結識別子, 出力形式種別)
    B2-->>Actor: 凍結識別子（標準出力）+ 終了コード 0
```

### 1-2. snapshot list

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C2 as 一覧処理
    participant Bsnap as スナップショット領域境界
    participant Elist as 一覧情報
    participant B2 as 結果出力窓口

    Actor->>B1: ベースライン一覧要求を受け付ける()
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C2: 一覧を組み立てる()
    C2->>Bsnap: 全スナップショット行を読み取る()
    Bsnap-->>C2: 行集合
    C2->>Elist: 整列済み行を取り出す()
    Elist-->>C2: スナップショット行のコレクション
    C2->>Elist: 件数を取り出す()
    Elist-->>C2: 数値
    C2-->>C0: 一覧情報
    C0->>B2: 一覧を出力する(一覧情報, 出力形式種別)
    B2-->>Actor: 一覧（標準出力・凍結日時降順）+ 終了コード 0
```

### 1-3. snapshot delete（凍結識別子直接指定）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant Bsnap as スナップショット領域境界
    participant Edel as 削除結果
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を受け付ける(削除対象識別)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C3: 削除を実行する(削除対象識別)
    C3->>Bsnap: 対象行を除去する(凍結識別子)
    Bsnap-->>C3: 削除結果
    C3->>Edel: 削除行数を取り出す()
    Edel-->>C3: 数値
    C3->>C3: 削除行数を確認する(削除結果)
    C3-->>C0: 削除操作結果
    C0->>B2: 削除確認を出力する(削除結果, 出力形式種別)
    B2-->>Actor: 削除確認（標準出力）+ 終了コード 0
```

## 2. 代替フロー

### 代替 1a: サブコマンド省略（使用法誤り）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット凍結を受け付ける(ラベル参照)
    Note over B1: サブコマンド種別が未指定
    B1->>B1: 使用法誤りを返す()
    B1->>B2: 診断メッセージを出力する(診断メッセージ, 診断種別)
    B2-->>Actor: 使用法案内（標準エラー出力）+ 終了コード 2
    Note over B1,B2: スナップショット統括処理へは渡さず境界で終端する
```

### 代替 2a: 空ストアで create（警告・非永続）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C1 as 凍結処理
    participant Bembed as embeddings ストア境界
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット凍結を受け付ける(ラベル参照)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C1: 凍結を実行する(ラベル参照)
    C1->>Bembed: 現行全行を読み取る()
    Bembed-->>C1: 行集合（0 件）
    C1->>C1: 空ストアを検出する(行集合)
    Note over C1,Bsnap: 0 件を検出 → スナップショット領域境界への書込みをスキップ（非永続）
    C1->>C1: 永続化をスキップする()
    C1-->>C0: 凍結操作結果（ノード数 0・警告付き）
    C0->>B2: 診断メッセージを出力する(診断メッセージ, 診断種別)
    C0->>B2: 操作結果を出力する(凍結識別子, 出力形式種別)
    B2-->>Actor: 警告（標準エラー出力）+ 凍結識別子・ノード数 0（標準出力または診断形式）+ 終了コード 0
    Note over B2,Actor: 返却された凍結識別子は一覧に現れない（永続化なし）
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

    Actor->>B1: ベースライン一覧要求を受け付ける()
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C2: 一覧を組み立てる()
    C2->>Bsnap: 全スナップショット行を読み取る()
    Bsnap-->>C2: 行集合（0 件）
    C2->>C2: 取得件数 0 件を処理する()
    C2->>Elist: 件数を取り出す()
    Elist-->>C2: 数値（0）
    C2-->>C0: 一覧情報（0 件）
    C0->>B2: 一覧を出力する(一覧情報, 出力形式種別)
    B2-->>Actor: 案内メッセージ（テキスト形式）/ 空のコレクション（診断形式）+ 終了コード 0
```

### 代替 6a: delete `ラベル参照` で同一 label 複数存在

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

    Actor->>B1: スナップショット削除を受け付ける(削除対象識別)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C3: 削除を実行する(削除対象識別)
    C3->>C4: ラベル参照を解決する(ラベル参照)
    C4->>Bsnap: 対象 label の全行を問い合わせる(ラベル参照)
    Bsnap-->>C4: 行集合（同一 label 複数件）
    C4->>C4: 最新一件へ決定論的に解決する(行集合)
    C4-->>C3: 解決済み凍結識別子
    C3->>Bsnap: 対象行を除去する(凍結識別子)
    Bsnap-->>C3: 削除結果
    C3->>Edel: 削除行数を取り出す()
    Edel-->>C3: 数値
    C3->>C3: 削除行数を確認する(削除結果)
    C3-->>C0: 削除操作結果
    C0->>B2: 削除確認を出力する(削除結果, 出力形式種別)
    B2-->>Actor: 削除確認（標準出力）+ 終了コード 0
```

### 代替 6b: delete 凍結識別子指定で該当 0 件（終了コード 0）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant Bsnap as スナップショット領域境界
    participant Edel as 削除結果
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を受け付ける(削除対象識別)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C3: 削除を実行する(削除対象識別)
    C3->>Bsnap: 対象行を除去する(凍結識別子)
    Bsnap-->>C3: 削除結果（0 行削除）
    C3->>Edel: 削除行数を取り出す()
    Edel-->>C3: 数値（0）
    C3->>C3: 削除行数を確認する(削除結果)
    C3-->>C0: 削除操作結果（0 件削除）
    C0->>B2: 削除確認を出力する(削除結果, 出力形式種別)
    B2-->>Actor: 警告（標準エラー出力）+ 終了コード 0（テキスト形式）/ 削除件数 0 のみ（診断形式・警告なし）
    Note over B2,Actor: 凍結識別子不在は終了コード 0（ラベル不在 6c の終了コード 1 との非対称は意図的）
```

### 代替 6c: delete `ラベル参照` で label 存在しない（終了コード 1）

```mermaid
sequenceDiagram
    actor Actor as 運用者 / 設定管理者 / 設計者 / QA リード
    participant B1 as スナップショットコマンド受付窓口
    participant C0 as スナップショット統括処理
    participant C3 as 削除処理
    participant C4 as label 解決処理
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    Actor->>B1: スナップショット削除を受け付ける(削除対象識別)
    B1->>C0: スナップショット操作を統括する(操作要求)
    C0->>C0: サブコマンド種別を判定する(操作要求)
    C0->>C3: 削除を実行する(削除対象識別)
    C3->>C4: ラベル参照を解決する(ラベル参照)
    C4->>Bsnap: 対象 label の全行を問い合わせる(ラベル参照)
    Bsnap-->>C4: 行集合（0 件）
    C4->>C4: ラベル解決失敗を確定する()
    C4-->>C3: 解決失敗
    C3-->>C0: 削除操作結果（解決失敗）
    C0->>B2: 診断メッセージを出力する(診断メッセージ, 診断種別)
    B2-->>Actor: エラー（標準エラー出力）+ 終了コード 1
    Note over B2,Actor: ラベル誤り・プロジェクトルート誤りを警告で覆い隠さない（SPEC-LGX-010.REQ.02）
```

## 3. 例外フロー

### 例外: スナップショット領域書込み失敗（create トランザクション失敗）

```mermaid
sequenceDiagram
    participant C0 as スナップショット統括処理
    participant C1 as 凍結処理
    participant Bembed as embeddings ストア境界
    participant Bsnap as スナップショット領域境界
    participant B2 as 結果出力窓口

    C0->>C1: 凍結を実行する(ラベル参照)
    C1->>Bembed: 現行全行を読み取る()
    Bembed-->>C1: 行集合
    C1->>Bsnap: 現行全行を単一トランザクションで複製する(行集合)
    Bsnap-->>C1: 複製結果（トランザクション失敗）
    Note over C1,Bsnap: トランザクション失敗時は原子的にロールバック（行集合は不完全状態で残らない）
    C1-->>C0: 凍結操作結果（失敗）
    C0->>B2: 診断メッセージを出力する(診断メッセージ, 診断種別)
    B2-->>Actor: エラー（標準エラー出力）+ 終了コード 1
```

## 4. 並行性（概念レベル）

`snapshot create` / `snapshot list` / `snapshot delete` はいずれも単一の操作要求として逐次受け付ける。`create` は UC 明示の単一トランザクション複製制約により原子的に完了する。スナップショット統括処理の協調下で各サブコマンドは逐次進む。並行アクセス時の整合性（複数アクターが同一 label に同時削除を行う場合等）は NFR-LGX-001 の射程であり本 SEQD 範囲外。具体的な並行機構は DD で確定する。

## 5. 失敗伝搬

- 各操作の戻り値は「結果」概念（成功 / 失敗 + 理由）で表現。具体的なエラー型は DD で確定。
- create トランザクション失敗は凍結処理が凍結操作結果（失敗）として統括処理へ伝搬し、最終的に終了コード 1 となる。
- label 解決失敗（代替 6c）は label 解決処理 → 削除処理 → 統括処理と伝搬し、終了コード 1 となる。
- 凍結識別子不在（代替 6b）・空ストア（代替 2a）はいずれも終了コード 0。削除行数 0 と label 不在の非対称は SPEC-LGX-010.REQ.02 の意図的設計であり伝搬路を別経路にする。
- サブコマンド省略（代替 1a）はスナップショットコマンド受付窓口が境界で終端し、終了コード 2 となる（統括処理へは渡さない）。

## 6. Behavior Allocation（操作のクラス帰属）

各操作は一つのクラスに帰属する（複数クラスへの分散なし）。Boundary=境界操作のみ / Control=複数 Entity の協調 / Entity=自身のデータ操作。

| 操作 | 帰属クラス | 役割 | 妥当性 |
|---|---|---|---|
| スナップショット凍結を受け付ける / ベースライン一覧要求を受け付ける / スナップショット削除を受け付ける / 使用法誤りを返す | スナップショットコマンド受付窓口 | Boundary（アクター境界） | ✓ 境界操作のみ |
| 現行全行を読み取る / ストアの存在を確認する | embeddings ストア境界 | Boundary（外部ストア境界） | ✓ |
| 全スナップショット行を読み取る / 対象 label の全行を問い合わせる / 現行全行を単一トランザクションで複製する / 対象行を除去する | スナップショット領域境界 | Boundary（外部永続域境界） | ✓ |
| 操作結果を出力する / 一覧を出力する / 削除確認を出力する / 診断メッセージを出力する | 結果出力窓口 | Boundary（出力境界） | ✓ |
| スナップショット操作を統括する / サブコマンド種別を判定する | スナップショット統括処理 | Control（協調） | ✓ |
| 凍結を実行する / 空ストアを検出する / 永続化をスキップする | 凍結処理 | Control | ✓ |
| 一覧を組み立てる / 取得件数 0 件を処理する | 一覧処理 | Control | ✓ |
| 削除を実行する / 削除行数を確認する | 削除処理 | Control | ✓ |
| ラベル参照を解決する / 最新一件へ決定論的に解決する / ラベル解決失敗を確定する | label 解決処理 | Control | ✓ |
| 識別子を取り出す | 凍結識別子 | Entity（自身のデータ） | ✓ |
| 行集合の件数を取り出す | スナップショット行集合 | Entity（自身のデータ） | ✓ |
| 整列済み行を取り出す / 件数を取り出す | 一覧情報 | Entity（自身のデータ） | ✓ |
| 削除行数を取り出す | 削除結果 | Entity（自身のデータ） | ✓ |

割り当てに迷う操作なし。各操作が UC ステップ / SEQA メッセージに対応（余剰操作なし）。

## 7. 整合性確認

- [x] レーンが RBD-LGX-012 のクラスと一致する（スナップショットコマンド受付窓口 / embeddings ストア境界 / スナップショット領域境界 / 結果出力窓口 / スナップショット統括処理 / 凍結処理 / 一覧処理 / 削除処理 / label 解決処理 / 凍結識別子 / スナップショット行集合 / 一覧情報 / 削除結果）
- [x] 操作呼び出しが RBD-LGX-012 で識別した操作と対応する
- [x] 命名規則に従う関数名が混入していない（操作名は日本語）
- [x] 言語固有の引数型・戻り型が混入していない（概念型のみ）
- [x] 言語固有同期機構の表記が混入していない
- [x] Boundary 同士の直接通信なし（Control 経由でのみ連携）
- [x] Entity 同士の直接通信なし（Control 経由でのみ読み書き）
- [x] Actor → Control / Entity 直結なし（アクターはスナップショットコマンド受付窓口 Boundary のみと通信）
- [x] UC-LGX-012 の基本（create/list/delete）/ 代替（1a/2a/4a/6a/6b/6c）/ 例外（create トランザクション失敗）フローを網羅
- [x] 6b（凍結識別子不在 = 終了コード 0）と 6c（ラベル不在 = 終了コード 1）の非対称が SEQD でも明示されている

## 8. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版。RBD-LGX-012 のクラスをレーンに操作呼び出し時系列を展開。基本（create/list/delete）/ 代替（1a/2a/4a/6a/6b/6c）/ 例外（create トランザクション失敗）を網羅。Behavior Allocation（13 クラス・操作のクラス帰属）を確定。6b(終了コード 0) と 6c(終了コード 1) の非対称を SEQD でも明示。言語要素なし |
