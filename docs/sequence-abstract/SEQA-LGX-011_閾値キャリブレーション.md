Document ID: SEQA-LGX-011

# SEQA-LGX-011: 閾値キャリブレーション のドメイン相互作用

**親 RBA**: RBA-LGX-011
**親 UC**: UC-LGX-011
**レイヤ**: 抽象側（ドメインレベル、言語非依存）

> **記述規律**: RBA-LGX-011 で識別したドメイン主語をレーンとして、UC-LGX-011 のフロー（基本/代替/例外）を時系列で展開する。メッセージは自然言語（ドメイン語彙）。関数名・API 名・引数型・言語固有同期機構は書かない（`04-iconix-layer.md` §4）。本 SEQA は UC ⇄ RBA ⇄ SEQA の Jacobson 流三者整合性を確定する。

---

## 1. UC text（並列配置）

UC-LGX-011 基本フロー（SEQA メッセージと 1:1 対応）:

```
1. アクターが legixy calibrate [--buckets N] [--recommend] [--json] を実行する
2. システムが embeddings テーブルから全件をロードする
3. システムが全ペア類似度を算出する（O(N²)）
4. 指定バケット数（既定 10、--buckets で調整可）のヒストグラムを生成する
5. text モード: ASCII ヒストグラム + 最小/最大/平均 + 現閾値一覧
6. --json モード: {"pairs": N, "min", "max", "mean", "distribution": [...], "thresholds": {...}}
7. exit 0 で終了
（代替 2a: embeddings が空 → INFO 出力 + exit 0）
（代替 1a: --buckets 0 指定 → エラー + exit 1）
（代替 3a: 全ペア算出失敗 → exit 1）
（代替 1b: --recommend 指定 → 推奨閾値追加出力）
（代替 3b: --recommend かつペア数 0 → stderr INFO、stdout 汚染なし）
```

## 2. 基本フロー（`calibrate`）

```mermaid
sequenceDiagram
    actor Actor as プロジェクトマネージャー / 設定管理者 / 設計者 / QA リード
    participant B1 as キャリブレーションコマンド受付窓口
    participant C0 as キャリブレーション統括処理
    participant C1 as 埋め込みロード処理
    participant Bstore as 埋め込みストア境界
    participant Evec as 埋め込みベクトル集合
    participant C2 as 全ペア類似度算出処理
    participant Epair as 全ペア類似度集合
    participant C3 as ヒストグラム生成処理
    participant Ehist as ヒストグラム
    participant Estat as 統計サマリ
    participant C5 as 結果整形処理
    participant Bcfg as 設定ファイル境界
    participant Ethresh as 現閾値設定
    participant Eresult as キャリブレーション結果
    participant B2 as キャリブレーション結果出力窓口

    Actor->>B1: キャリブレーションを要求する（バケット数・出力フォーマット指定）
    B1->>C0: キャリブレーションを統括する
    C0->>C1: 埋め込みベクトルをロードする
    C1->>Bstore: 埋め込みストアから全件を読む
    Bstore-->>C1: 全ノード埋め込みベクトル
    C1->>Evec: 埋め込みベクトル集合を確定する
    C0->>C2: 全ペア類似度を算出する
    C2->>Evec: 埋め込みベクトル集合を読む
    C2->>Epair: 全ペア類似度集合を生成する（非有限スコアのペアは算入しない）
    C0->>C3: ヒストグラムを生成する
    C3->>Epair: 全ペア類似度集合を読む
    C3->>Ehist: ヒストグラムを生成する（指定バケット数）
    C3->>Estat: 統計サマリを生成する（最小・最大・平均）
    C0->>C5: 結果を整形する
    C5->>Bcfg: 設定ファイルから現閾値を読む
    Bcfg-->>C5: 3 閾値の現在値
    C5->>Ethresh: 現閾値設定を確定する
    C5->>Ehist: ヒストグラムを読む
    C5->>Estat: 統計サマリを読む
    C5->>Ethresh: 現閾値設定を読む
    C5->>Eresult: キャリブレーション結果を生成する（指定フォーマット）
    C5->>B2: キャリブレーション結果を渡す
    B2-->>Actor: ヒストグラム + 統計 + 現閾値一覧 + 終了コード 0
```

## 3. 代替フロー

### 代替 1b: `--recommend` 指定（推奨閾値を追加出力）

```mermaid
sequenceDiagram
    participant C0 as キャリブレーション統括処理
    participant C4 as 推奨閾値算出処理
    participant Epair as 全ペア類似度集合
    participant Erec as 推奨閾値
    participant C5 as 結果整形処理
    participant Eresult as キャリブレーション結果
    participant B2 as キャリブレーション結果出力窓口

    C0->>C4: 推奨閾値を算出する（--recommend 指定時）
    C4->>Epair: 全ペア類似度集合を読む
    C4->>Erec: 推奨閾値を算出する（パーセンタイル方式: p25 / 1.0−p90 / p75）
    C0->>C5: 結果を整形する（推奨閾値を含む）
    C5->>Erec: 推奨閾値を読む
    C5->>Eresult: キャリブレーション結果を生成する（推奨閾値含む）
    C5->>B2: キャリブレーション結果を渡す
    B2-->>Actor: ヒストグラム + 統計 + 現閾値 + 推奨閾値 + 終了コード 0
    Note over B2: 代替 3b: ペア数 0 の場合は推奨閾値を算出せず<br/>stderr に INFO を出力（stdout は汚染しない）
```

### 代替 2a: embeddings が空（早期終了）

```mermaid
sequenceDiagram
    participant C0 as キャリブレーション統括処理
    participant C1 as 埋め込みロード処理
    participant Bstore as 埋め込みストア境界
    participant Evec as 埋め込みベクトル集合
    participant B2 as キャリブレーション結果出力窓口

    C0->>C1: 埋め込みベクトルをロードする
    C1->>Bstore: 埋め込みストアから全件を読む
    Bstore-->>C1: 空（エントリなし）
    C1->>Evec: 埋め込みベクトル集合を空として確定する
    C1->>C0: 早期終了を通知する（空ストア）
    C0->>B2: INFO 通知を渡す（embed --all の実行を促す）
    B2-->>Actor: INFO: ベクトルストアが空です + 終了コード 0
    Note over C0,B2: 全ペア算出・ヒストグラム生成・推奨閾値算出はスキップ
```

### 代替 1a: `--buckets 0` 指定（入力契約違反）

```mermaid
sequenceDiagram
    actor Actor as プロジェクトマネージャー / 設定管理者 / 設計者 / QA リード
    participant B1 as キャリブレーションコマンド受付窓口
    participant C0 as キャリブレーション統括処理
    participant B2 as キャリブレーション結果出力窓口

    Actor->>B1: キャリブレーションを要求する（--buckets 0 指定）
    B1->>C0: オプション検証でエラーを通知する（バケット数 0 は無効）
    C0->>B2: エラー報告を渡す
    B2-->>Actor: エラーメッセージ + 終了コード 1
```

## 4. 例外フロー

### 例外: 全ペア算出失敗（代替 3a）

```mermaid
sequenceDiagram
    participant C0 as キャリブレーション統括処理
    participant C2 as 全ペア類似度算出処理
    participant Evec as 埋め込みベクトル集合
    participant B2 as キャリブレーション結果出力窓口

    C0->>C2: 全ペア類似度を算出する
    C2->>Evec: 埋め込みベクトル集合を読む
    C2->>C0: 算出失敗を通知する（エラーコンテキスト付き）
    C0->>B2: エラー報告を渡す
    B2-->>Actor: エラーメッセージ + 終了コード 1
```

### 例外: `--recommend` かつペア数 0（代替 3b、stdout 保護）

```mermaid
sequenceDiagram
    participant C0 as キャリブレーション統括処理
    participant C4 as 推奨閾値算出処理
    participant Epair as 全ペア類似度集合
    participant B2 as キャリブレーション結果出力窓口

    C0->>C4: 推奨閾値を算出する（--recommend 指定）
    C4->>Epair: 全ペア類似度集合を読む（ペア数 0）
    C4->>C0: 算出不能通知を生成する（ペア数 0）
    C0->>B2: 通常の出力（推奨閾値なし）と stderr INFO を渡す
    B2-->>Actor: ヒストグラム + 統計 + 現閾値（stdout）<br/>+ INFO: ペア数 0 のため推奨値は算出されません（stderr）
    Note over B2: --json の stdout を汚染しない（OBS.02 準拠）
```

## 5. 並行性（概念レベル）

`calibrate` は読み取り専用の分析処理であり、ドメインレベルで並行に発生する事象はない。埋め込みロード → 全ペア算出 → ヒストグラム生成 → 推奨閾値算出（オプション）→ 結果整形 の各処理はキャリブレーション統括処理の協調下で逐次進む。SCORE-INV-1（決定性保証: 同一 embeddings → 同一ヒストグラム）がこの逐次性と整合する。

## 6. 整合性確認

- [x] 各メッセージがドメイン語彙で書かれている（関数名・API 名・型なし）
- [x] レーンが RBA-LGX-011 の主語と一致する（クラス名混入なし）
- [x] UC-LGX-011 の基本（Step1-7）/ 代替（1a/1b/2a/3a/3b）/ 例外（全ペア算出失敗・ペア数 0 推奨）フローを網羅
- [x] Noun-Verb ルール遵守（Actor⇄Boundary / Boundary⇄Control / Control⇄Control / Control⇄Entity のみ。Boundary 同士・Entity 同士・Boundary→Entity・Actor→内部 の直接通信なし）

## 7. コントローラ責務と実行操作の整合（§4.4）

| Control レーン | 概念名が示す責務 | 実行する操作 | 整合 |
|---|---|---|---|
| キャリブレーション統括処理 | キャリブレーションフロー全体の協調・早期終了判断 | 各処理を順に依頼、早期終了通知受信・結果整形依頼 | ✓ |
| 埋め込みロード処理 | embeddings テーブルから全件ロード・空ストア検出 | 埋め込みストア境界を読み埋め込みベクトル集合を確定、空時は早期終了通知 | ✓（ヒストグラム生成等の越権なし） |
| 全ペア類似度算出処理 | 全ペアコサイン類似度の算出（非有限スコア除外） | 埋め込みベクトル集合を読み全ペア類似度集合を生成 | ✓ |
| ヒストグラム生成処理 | 度数分布と統計値の生成 | 全ペア類似度集合を読みヒストグラムと統計サマリを生成 | ✓（推奨閾値算出は行わない） |
| 推奨閾値算出処理 | パーセンタイル方式による推奨閾値算出（--recommend 時） | 全ペア類似度集合を読み推奨閾値を算出、ペア数 0 時は算出不能通知 | ✓ |
| 結果整形処理 | 全出力要素の整形・キャリブレーション結果生成 | 設定ファイル境界から現閾値設定を取得、ヒストグラム/統計サマリ/推奨閾値（指定時）を読み結果を生成 | ✓ |

余剰操作なし（各操作が UC ステップに対応）。Control 間メッセージ（統括 → 各処理）が UC の振る舞いを実現。

## 8. Jacobson 流三者整合性（UC ⇄ RBA ⇄ SEQA、§11.1）— 確定

| 検査 | 確認内容 | 結果 |
|---|---|---|
| UC ⇄ RBA | UC-LGX-011 各ステップが RBA-LGX-011 フローに 1:1 対応（RBA §5） | ✓ |
| RBA ⇄ SEQA | RBA-LGX-011 の主語（B4/C6/E7）が本 SEQA のレーンと一致、Noun-Verb ルールが SEQA でも保持（§6） | ✓ |
| UC ⇄ SEQA | UC text 並列配置（§1）、各 UC ステップが SEQA メッセージと対応（基本/代替/例外を §2-4 で網羅） | ✓ |

3 者が同じ振る舞いを動的に表現していることを確認。**これにより RBA-LGX-011 §8 の Jacobson 流三者整合性「保留」が解消される。**

## 9. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版。UC-LGX-011 / RBA-LGX-011 の時系列展開。基本（calibrate）/ 代替（1a --buckets 0 / 1b --recommend / 2a 空ストア早期終了 / 3a 全ペア算出失敗 / 3b --recommend かつペア数 0）/ 例外（全ペア算出失敗・ペア数 0 推奨）を網羅。Jacobson 流三者整合性を確定（RBA-LGX-011 §8 保留解消）。Control 責務⇄操作の整合（§4.4）確認 |
