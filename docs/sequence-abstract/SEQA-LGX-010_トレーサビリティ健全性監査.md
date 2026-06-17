Document ID: SEQA-LGX-010

# SEQA-LGX-010: トレーサビリティ健全性監査 のドメイン相互作用

**親 RBA**: RBA-LGX-010
**親 UC**: UC-LGX-010
**レイヤ**: 抽象側（ドメインレベル、言語非依存）

> **記述規律**: RBA-LGX-010 で識別したドメイン主語をレーンとして、UC-LGX-010 のフロー（基本/代替/例外）を時系列で展開する。メッセージは自然言語（ドメイン語彙）。関数名・API 名・引数型・言語固有同期機構は書かない（`04-iconix-layer.md` §4）。本 SEQA は UC ⇄ RBA ⇄ SEQA の Jacobson 流三者整合性を確定する。

---

## 1. UC text（並列配置）

UC-LGX-010 基本フロー（SEQA メッセージと 1:1 対応）:

```
1. アクターが `legixy report [--json]` を実行する
2. システムが graph.toml をパースし、embeddings テーブルから全件をロードする
   （サブ: .legixy.toml の link_candidate_threshold を解決する）
3. システムが以下を算出する:
   a. 全エッジの cosine 類似度
   b. 非エッジペアで類似度 ≥ link_candidate_threshold のリンク漏れ候補
4. text モード: 人間可読な階層表示（=== Traceability Report: All Links === + 各行類似度 + === Link Candidates === + 候補一覧 + 統計サマリ）
5. --json モード: {"links": [...], "candidates": [...], "summary": {...}} の構造化 JSON
6. exit 0 で終了
（代替 2a: embeddings テーブルが空 → INFO 出力して exit 0。代替 3a: 算出失敗 → exit 1）
```

## 2. 基本フロー（`report [--json]`）

```mermaid
sequenceDiagram
    actor Actor as プロジェクトリード / QA リード / 設計者・実装者 / CI システム
    participant B1 as 監査コマンド受付窓口
    participant C0 as 監査統括処理
    participant C1 as 設定解決処理
    participant Bcfg as 設定ファイル境界
    participant Ecfg as 監査設定
    participant C2 as グラフ定義取得処理
    participant Bgraph as グラフ定義境界
    participant Eedge as グラフエッジ集合
    participant C3 as ベクトルロード処理
    participant Bvec as ベクトルストア境界
    participant Evec as 保存済みベクトル集合
    participant C4 as エッジスコア算出処理
    participant Escore as エッジスコア集合
    participant C5 as リンク漏れ候補算出処理
    participant Ecand as リンク漏れ候補集合
    participant C6 as 監査報告生成処理
    participant Ereport as 監査報告
    participant B2 as 監査結果出力窓口

    Actor->>B1: 監査を要求する（report [--json]）
    B1->>C0: 監査を統括する
    C0->>C1: 設定を解決する
    C1->>Bcfg: 設定を読む
    Bcfg-->>C1: 設定内容（link_candidate_threshold 含む）
    C1->>Ecfg: 監査設定を確定する
    C0->>C2: グラフ定義を取得する
    C2->>Bgraph: 全エッジ定義を読む
    Bgraph-->>C2: エッジ定義（Chain / Custom / ParentChild）
    C2->>Eedge: グラフエッジ集合を確定する
    C0->>C3: 全件ベクトルをロードする
    C3->>Bvec: embeddings テーブルから全件を読む
    Bvec-->>C3: 保存済み全件ベクトル
    C3->>Evec: 保存済みベクトル集合を確定する
    C0->>C4: エッジスコアを算出する
    C4->>Eedge: 全エッジ定義を参照する
    C4->>Evec: 両端点のベクトルを参照する
    C4->>Ecfg: 監査設定を参照する
    C4->>Escore: エッジスコア集合を生成する（スキップエッジ・集約警告を含む）
    C0->>C5: リンク漏れ候補を算出する
    C5->>Evec: 全件ベクトルを参照する
    C5->>Ecfg: 閾値設定を参照する
    C5->>Ecand: リンク漏れ候補集合を生成する
    C0->>C6: 監査報告を生成する
    C6->>Escore: エッジスコア集合を参照する
    C6->>Ecand: リンク漏れ候補集合を参照する
    C6->>Ereport: 監査報告を作る（text モードまたは JSON モード + 統計サマリ）
    C6->>B2: 監査報告を渡す
    B2-->>Actor: 監査報告（stdout）+ 診断情報（stderr）+ 終了コード 0
```

## 3. 代替フロー

### 代替 2a: embeddings テーブルが空

```mermaid
sequenceDiagram
    participant C0 as 監査統括処理
    participant C3 as ベクトルロード処理
    participant Bvec as ベクトルストア境界
    participant C7 as 空ストア通知処理
    participant B2 as 監査結果出力窓口

    C0->>C3: 全件ベクトルをロードする
    C3->>Bvec: embeddings テーブルから全件を読む
    Bvec-->>C3: 空（エントリなし）
    C3->>C7: 空状態を通知する
    C7->>B2: 案内情報を渡す（INFO: ベクトルストアが空。embed --all を実行してください）
    B2-->>Actor: 案内情報（stdout）+ 終了コード 0
    Note over C0,B2: 算出処理は起動しない（空状態で早期終了）
```

## 4. 例外フロー

### 例外 3a: エッジスコア算出 / リンク漏れ候補算出の失敗

```mermaid
sequenceDiagram
    participant C0 as 監査統括処理
    participant C4 as エッジスコア算出処理
    participant C5 as リンク漏れ候補算出処理
    participant B2 as 監査結果出力窓口

    C0->>C4: エッジスコアを算出する
    C4-->>C0: 算出失敗（エラーコンテキスト付き）
    Note over C4,C0: リンク漏れ候補算出処理も同様に失敗しうる
    C0->>B2: 算出失敗を報告する（エラーコンテキスト付き）
    B2-->>Actor: エラー情報（stderr）+ 終了コード 1
```

## 5. 並行性（概念レベル）

`report` は読取専用の計測処理であり、ドメインレベルで並行に発生する事象はない。設定解決・グラフ定義取得・ベクトルロード・エッジスコア算出・リンク漏れ候補算出は監査統括処理の協調下で逐次進む。STATE-INV-1（engine.db / graph.toml は不変）により書込み競合も発生しない。

## 6. 整合性確認

- [x] 各メッセージがドメイン語彙で書かれている（関数名・API 名・型なし）
- [x] レーンが RBA-LGX-010 の主語と一致する（Boundary 5 / Control 8 / Entity 6 のうち SEQA に現れる主語はすべて RBA に存在する。クラス名混入なし）
- [x] UC-LGX-010 の基本（Step 1–6）/ 代替（2a・3a）フローを網羅
- [x] Noun-Verb ルール遵守（Actor⇄Boundary / Boundary⇄Control / Control⇄Control / Control⇄Entity のみ。Boundary 同士・Entity 同士・Boundary→Entity・Actor→内部 の直接通信なし）

## 7. コントローラ責務と実行操作の整合（§4.4）

| Control レーン | 概念名が示す責務 | 実行する操作 | 整合 |
|---|---|---|---|
| 監査統括処理 | 監査フロー全体の協調 | 設定解決・グラフ定義取得・ベクトルロード・エッジスコア算出・リンク漏れ候補算出・監査報告生成を順に依頼 | ✓ |
| 設定解決処理 | 設定の解決 | 設定ファイル境界を読み監査設定を確定 | ✓（報告生成等の越権なし） |
| グラフ定義取得処理 | グラフ定義の取得 | グラフ定義境界を読みグラフエッジ集合を確定 | ✓ |
| ベクトルロード処理 | 全件ベクトルのロード・空状態の検知 | ベクトルストア境界を読み保存済みベクトル集合を確定。空時は空ストア通知処理を起動 | ✓ |
| エッジスコア算出処理 | 既定義エッジの cosine 類似度算出 | グラフエッジ集合・保存済みベクトル集合・監査設定を参照しエッジスコア集合を生成（スキップ・集約警告含む） | ✓ |
| リンク漏れ候補算出処理 | 非エッジペアからのリンク漏れ候補算出 | 保存済みベクトル集合・監査設定を参照しリンク漏れ候補集合を生成 | ✓ |
| 監査報告生成処理 | 監査報告の生成と出力渡し | エッジスコア集合・リンク漏れ候補集合を参照し監査報告を作成（text/JSON + 統計サマリ）し監査結果出力窓口へ渡す | ✓ |
| 空ストア通知処理 | 空ストア時の案内情報生成 | 案内情報を監査結果出力窓口へ渡す | ✓ |

余剰操作なし（各操作が UC ステップに対応）。Control 間メッセージ（統括 → 各処理）が UC の振る舞いを実現。

## 8. Jacobson 流三者整合性（UC ⇄ RBA ⇄ SEQA、§11.1）— 確定

| 検査 | 確認内容 | 結果 |
|---|---|---|
| UC ⇄ RBA | UC-LGX-010 各ステップが RBA-LGX-010 フローに 1:1 対応（RBA-010 §5） | ✓ |
| RBA ⇄ SEQA | RBA-LGX-010 の主語（Boundary 5 / Control 8 / Entity 6）が本 SEQA のレーンと一致。Noun-Verb ルールが SEQA でも保持（§6）。RBA に無い新規主語の追加なし | ✓ |
| UC ⇄ SEQA | UC text 並列配置（§1）、各 UC ステップが SEQA メッセージと対応（基本/代替/例外を §2–4 で網羅） | ✓ |

3 者が同じ振る舞いを動的に表現していることを確認。**これにより RBA-LGX-010 §8 の Jacobson 流三者整合性「保留」が解消される。**

## 9. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版。UC-LGX-010 / RBA-LGX-010 の時系列展開。基本（report [--json]）/ 代替（空ストア）/ 例外（算出失敗）を網羅。Jacobson 流三者整合性を確定（RBA-010 §8 保留解消）。Control 責務⇄操作の整合（§4.4）確認 |
