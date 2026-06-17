Document ID: RPC-LGX-011

# RPC-LGX-011: 閾値キャリブレーション chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-011
**対象 RBA**: RBA-LGX-011
**対象 SEQA**: SEQA-LGX-011
**対象 RBD**: RBD-LGX-011
**対象 SEQD**: SEQD-LGX-011
**検査深度**: フル（§14.2: 読取専用分析処理だが calibrate コマンドは閾値チューニング判断の根拠を提供する重要な観測チャネル）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | キャリブレーションコマンド受付窓口 | `calibrate` 実行要求とオプション（`--buckets` / `--recommend` / `--json`）を受け取る | UC-011 基本 Step 1 |
| AR-002 | RBA | Boundary | キャリブレーションコマンド受付窓口 | `--buckets 0` 指定時にオプション検証でエラーを検出する | UC-011 代替 1a |
| AR-003 | RBA | Control | キャリブレーション統括処理 | 実行要求を受け、埋め込みロード・全ペア類似度算出・ヒストグラム生成・推奨閾値算出（オプション）・結果整形・出力を協調させる | UC-011 基本 Step 1–7 |
| AR-004 | RBA/SEQA | Control | キャリブレーション統括処理 | 空ストア検出時の早期終了を判定し、全ペア算出・ヒストグラム生成・推奨閾値算出をスキップする | UC-011 代替 2a |
| AR-005 | RBA | Control | 埋め込みロード処理 | 埋め込みストア境界から全ノードの埋め込みベクトルをロードする | UC-011 基本 Step 2 |
| AR-006 | RBA/SEQA | Control | 埋め込みロード処理 | 空ストアを検出し統括処理へ早期終了を通知する | UC-011 代替 2a |
| AR-007 | RBA | Boundary | 埋め込みストア境界 | `engine.db` の embeddings テーブルから全ノード埋め込みベクトルを供給する（読取専用） | UC-011 基本 Step 2 |
| AR-008 | RBA | Entity | 埋め込みベクトル集合 | ロード済みの全ノード埋め込みベクトルを保持する（空も許容） | UC-011 基本 Step 2, 3 |
| AR-009 | RBA | Control | 全ペア類似度算出処理 | ロード済み埋め込みベクトルから全ペアのコサイン類似度を算出する（非有限スコアのペアは算入しない） | UC-011 基本 Step 3 |
| AR-010 | RBA/SEQA | Control | 全ペア類似度算出処理 | 全ペア算出失敗時に統括処理へエラーを通知する | UC-011 代替 3a |
| AR-011 | RBA | Entity | 全ペア類似度集合 | 全ノードペアのコサイン類似度値の集まりを保持する | UC-011 基本 Step 3, 4 |
| AR-012 | RBA | Control | ヒストグラム生成処理 | 全ペア類似度集合と指定バケット数から度数分布（ヒストグラム）と統計値（最小・最大・平均）を生成する | UC-011 基本 Step 4 |
| AR-013 | RBA | Entity | ヒストグラム | バケット別の度数分布（バケット数は実行オプションで指定）を保持する | UC-011 基本 Step 4, 5, 6 |
| AR-014 | RBA | Entity | 統計サマリ | 全ペア類似度の最小・最大・平均値を保持する | UC-011 基本 Step 4, 5, 6 |
| AR-015 | RBA | Control | 推奨閾値算出処理 | 全ペア類似度集合のパーセンタイル方式（p25 / 1.0−p90 / p75）に基づき 3 閾値の推奨値を算出する（`--recommend` 指定時のみ） | UC-011 代替 1b |
| AR-016 | RBA/SEQA | Control | 推奨閾値算出処理 | ペア数 0 の場合は推奨閾値を算出せず算出不能通知を生成する | UC-011 代替 3b |
| AR-017 | RBA | Entity | 推奨閾値 | パーセンタイル方式で算出した 3 閾値の推奨値を保持する（`--recommend` 指定時のみ存在） | UC-011 代替 1b |
| AR-018 | RBA | Control | 結果整形処理 | ヒストグラム・統計値・現閾値設定・推奨閾値（指定時）を指定フォーマット（text / JSON）に整形し、キャリブレーション結果出力窓口へ渡す | UC-011 基本 Step 5, 6 |
| AR-019 | RBA | Boundary | 設定ファイル境界 | `.legixy.toml` の 3 閾値（`similarity_threshold` / `drift_threshold` / `link_candidate_threshold`）の現在値を供給する | UC-011 基本 Step 5 |
| AR-020 | RBA | Entity | 現閾値設定 | `.legixy.toml` から供給された 3 閾値の現在値を保持する | UC-011 基本 Step 5 |
| AR-021 | RBA | Entity | キャリブレーション結果 | ヒストグラム・統計サマリ・現閾値設定・推奨閾値（指定時）を束ねた出力全体を保持する | UC-011 基本 Step 5, 6, 7 |
| AR-022 | RBA | Boundary | キャリブレーション結果出力窓口 | ヒストグラム・統計サマリ・閾値一覧・推奨閾値をアクターへ返す（text モードは stdout / ログ・情報通知・エラーは stderr） | UC-011 基本 Step 5, 6, 7 / 代替 2a, 3b |

全 22 AR が UC ステップまたは代替フロー（1a/1b/2a/3a/3b）に紐づく。UC ステップに紐づかない AR なし → 構造翻訳が情報を加えていない。SEQA-LGX-011 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | キャリブレーションコマンド受付窓口 | キャリブレーション要求を受け付ける | アクター境界でオプション付き実行要求を受理 | Actor→B1 |
| CR-002 | RBD/SEQD | キャリブレーションコマンド受付窓口 | オプションを検証する | バケット数 0 などの入力契約違反を検出 | B1→B1 |
| CR-003 | RBD/SEQD | キャリブレーション統括処理 | キャリブレーションを統括する | 各処理を順に協調、フロー全体を制御 | B1→C0 |
| CR-004 | RBD/SEQD | キャリブレーション統括処理 | 早期終了を判定する | 空ストア時に以降の処理をスキップ | C0→C0 |
| CR-005 | RBD/SEQD | 埋め込みロード処理 | 埋め込みベクトルをロードする | 埋め込みストア境界から全件を読み込む | C0→C1 |
| CR-006 | RBD/SEQD | 埋め込みロード処理 | 空ストアを検出する | 空ストア状態を統括処理へ通知 | C1→C0 |
| CR-007 | RBD/SEQD | 埋め込みストア境界 | 全件を読み出す | engine.db の全ノード埋め込みベクトルを供給 | C1→Bstore |
| CR-008 | RBD/SEQD | 埋め込みストア境界 | ストアが空かを確認する | 空判定を埋め込みロード処理へ供給 | C1→Bstore |
| CR-009 | RBD/SEQD | 埋め込みベクトル集合 | 件数を取り出す | 保持するベクトル件数を返す | C1→Evec, C2→Evec |
| CR-010 | RBD/SEQD | 全ペア類似度算出処理 | 全ペア類似度を算出する | 埋め込みベクトル集合から全ペア類似度集合を生成 | C0→C2 |
| CR-011 | RBD/SEQD | 全ペア類似度算出処理 | 非有限スコアのペアを除外する | 算出中の非有限値スコアをフィルタ | C2→C2 |
| CR-012 | RBD/SEQD | 全ペア類似度集合 | ペア数を取り出す | 保持するペア数を返す | C2→Epair, C4→Epair |
| CR-013 | RBD/SEQD | 全ペア類似度集合 | スコア一覧を取り出す | 類似度スコアの集合を返す | C3→Epair, C4→Epair |
| CR-014 | RBD/SEQD | ヒストグラム生成処理 | ヒストグラムを生成する | 全ペア類似度集合と指定バケット数から度数分布を生成 | C0→C3 |
| CR-015 | RBD/SEQD | ヒストグラム生成処理 | 統計サマリを生成する | 全ペア類似度集合から最小・最大・平均を生成 | C3→C3 |
| CR-016 | RBD/SEQD | ヒストグラム | バケット一覧を取り出す | 保持するバケットのコレクションを返す | C3→Ehist |
| CR-017 | RBD/SEQD | 統計サマリ | 統計値を取り出す | 保持する統計値セットを返す | C3→Estat |
| CR-018 | RBD/SEQD | 推奨閾値算出処理 | 推奨閾値を算出する | パーセンタイル方式で 3 閾値推奨値を算出 | C0→C4 |
| CR-019 | RBD/SEQD | 推奨閾値算出処理 | ペア数ゼロを検出する | ペア数 0 時に算出不能通知を生成 | C4→C4 |
| CR-020 | RBD/SEQD | 推奨閾値 | 推奨値一覧を取り出す | 保持する推奨閾値コレクションを返す | C4→Erec |
| CR-021 | RBD/SEQD | 結果整形処理 | キャリブレーション結果を生成する | ヒストグラム・統計サマリ・現閾値・フォーマット種別から結果を生成 | C0→C5 |
| CR-022 | RBD/SEQD | 結果整形処理 | 推奨閾値を結果に加える | 推奨閾値指定時にキャリブレーション結果へ付加 | C0→C5（代替 1b） |
| CR-023 | RBD/SEQD | 設定ファイル境界 | 閾値設定を読み出す | .legixy.toml の 3 閾値現在値を供給 | C5→Bcfg |
| CR-024 | RBD/SEQD | 現閾値設定 | 閾値一覧を取り出す | 保持する 3 閾値のコレクションを返す | C5→Ethresh |
| CR-025 | RBD/SEQD | キャリブレーション結果 | 結果を取り出す | 保持する結果表現（フォーマット指定済）を返す | C5→Eresult |
| CR-026 | RBD/SEQD | キャリブレーション結果出力窓口 | 結果を標準出力へ渡す | ヒストグラム・統計・閾値をアクターへ出力 | C0→B2 |
| CR-027 | RBD/SEQD | キャリブレーション結果出力窓口 | 情報通知を標準エラー出力へ渡す | 空ストア・ペア数 0 の INFO を stderr へ出力 | C0→B2（代替 2a, 3b） |
| CR-028 | RBD/SEQD | キャリブレーション結果出力窓口 | エラーを標準エラー出力へ渡す | バケット数 0 / 全ペア算出失敗のエラーを stderr へ出力 | C0→B2（代替 1a, 3a） |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・アクター境界での実行要求受理操作 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Boundary・バケット数 0 を含む入力契約違反の検出操作 | GREEN |
| AR-003 | CR-003 | preserved | 統括 Control。各処理の協調・フロー制御 | GREEN |
| AR-004 | CR-004 | preserved | 早期終了判定操作として具体化。スキップ対象（全ペア算出/ヒストグラム生成/推奨閾値算出）を判断する責務が統括処理クラス内に保存 | GREEN |
| AR-005 | CR-005 | preserved | 埋め込みロード Control。埋め込みストア境界から全件ロード操作 | GREEN |
| AR-006 | CR-006 | preserved | 空ストア検出操作として具体化。統括処理への通知責務が保存 | GREEN |
| AR-007 | CR-007, CR-008 | split | 抽象側「全ノード埋め込みベクトルを供給する」が具体側で「全件を読み出す」（通常時）と「ストアが空かを確認する」（空確認）に分割。RBD-LGX-011 §4・SEQD-LGX-011 代替 2a に明示。空ストアの境界チェックを Boundary 操作として具体化した妥当な分割 | GREEN（justified split） |
| AR-008 | CR-009 | preserved | Entity が件数取り出し操作で自身のデータを提供。ベクトル集合の保持責務は保存 | GREEN |
| AR-009 | CR-010, CR-011 | split | 抽象側「全ペアのコサイン類似度を算出する（非有限スコアのペアは算入しない）」が具体側で「全ペア類似度を算出する」と「非有限スコアのペアを除外する」に分割。RBD §1 Control クラスの操作識別に明示。非有限除外を独立操作として可視化した妥当な分割 | GREEN（justified split） |
| AR-010 | CR-010 | preserved | 算出失敗時の通知責務は全ペア類似度算出処理が失敗結果を返す形で統括処理へ伝搬（SEQD §3 例外フロー：全ペア算出失敗）。算出失敗の検出と通知は同クラスに保持 | GREEN |
| AR-011 | CR-012, CR-013 | split | 抽象側「全ノードペアのコサイン類似度値の集まりを保持する」が具体側で「ペア数を取り出す」と「スコア一覧を取り出す」の 2 操作を持つ Entity として具体化。RBD §1 Entity クラスに明示。異なる Consumer（ヒストグラム生成処理・推奨閾値算出処理・統括処理）が異なる側面にアクセスするための妥当な操作分割 | GREEN（justified split） |
| AR-012 | CR-014, CR-015 | split | 抽象側「度数分布（ヒストグラム）と統計値（最小・最大・平均）を生成する」が具体側で「ヒストグラムを生成する」と「統計サマリを生成する」に分割。RBD §1 Control クラスに明示。ヒストグラムと統計サマリは別 Entity に格納されるため独立操作として具体化した妥当な分割 | GREEN（justified split） |
| AR-013 | CR-016 | preserved | ヒストグラム Entity がバケット一覧を取り出す操作で自身のデータを保持 | GREEN |
| AR-014 | CR-017 | preserved | 統計サマリ Entity が統計値を取り出す操作で自身のデータを保持 | GREEN |
| AR-015 | CR-018 | preserved | 推奨閾値算出 Control。パーセンタイル方式算出操作（--recommend 時のみ） | GREEN |
| AR-016 | CR-019 | preserved | ペア数ゼロ検出操作として具体化。算出不能通知の生成責務が保存 | GREEN |
| AR-017 | CR-020 | preserved | 推奨閾値 Entity が推奨値一覧を取り出す操作で自身のデータを保持 | GREEN |
| AR-018 | CR-021, CR-022 | split | 抽象側「ヒストグラム・統計値・現閾値設定・推奨閾値（指定時）を指定フォーマット（text / JSON）に整形し渡す」が具体側で「キャリブレーション結果を生成する」（基本・JSON）と「推奨閾値を結果に加える」（--recommend 付加）に分割。RBD §1 Control クラス・SEQD §2 代替 1b に明示。推奨閾値の有無とフォーマット指定の直交性を操作で表現した妥当な分割（RBA §6 Object Discovery「結果整形処理の独立」と整合） | GREEN（justified split） |
| AR-019 | CR-023 | preserved | 設定ファイル境界が閾値設定を読み出す操作で .legixy.toml を供給 | GREEN |
| AR-020 | CR-024 | preserved | 現閾値設定 Entity が閾値一覧を取り出す操作で自身のデータを保持 | GREEN |
| AR-021 | CR-025 | preserved | キャリブレーション結果 Entity が結果を取り出す操作で束ねたデータを保持 | GREEN |
| AR-022 | CR-026, CR-027, CR-028 | split | 抽象側「ヒストグラム・統計サマリ・閾値一覧・推奨閾値をアクターへ返す（stdout / stderr 分離）」が具体側で「結果を標準出力へ渡す」「情報通知を標準エラー出力へ渡す」「エラーを標準エラー出力へ渡す」の 3 操作に分割。RBD §1 Boundary クラス・SEQD §3 各フローに明示。stdout / stderr / エラー stderr の三経路が明確に異なる観測要件（NFR-LGX-001.OBS.02）に対応した妥当な分割 | GREEN（justified split） |

22 AR のうち preserved 14、justified split 8（split 根拠はいずれも RBD §1 操作識別・SEQD フロー・RBA §6 Object Discovery に明示）。lost / shifted / mutated / ambiguous / 未正当化 invented なし。

## 4. Role Fitness Check（§5.2）

### Boundary
- Finding: キャリブレーションコマンド受付窓口（アクター境界）・埋め込みストア境界（外部 DB 境界）・設定ファイル境界（外部ファイル境界）・キャリブレーション結果出力窓口（出力境界）はいずれも境界操作のみ保持。受付窓口の「オプションを検証する」はアクター→システム境界での入力契約チェックであり Boundary overreach なし（RBA §6 Object Discovery で確認済）。
- Severity: なし / 原因の所在: — / Required action: なし

### Control
- Finding: キャリブレーション統括処理は協調・早期終了判定のみ（データ保持なし）。埋め込みロード処理・全ペア類似度算出処理・ヒストグラム生成処理・推奨閾値算出処理・結果整形処理はそれぞれ単一の処理責務に集中。万能化（Service blob）なし・Control leakage なし。各 Control の操作が担う処理と概念名が一致（ヒストグラム生成処理が推奨閾値を算出しない等、RBA §6 確認済）。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity
- Finding: 埋め込みベクトル集合・全ペア類似度集合・ヒストグラム・統計サマリ・現閾値設定・推奨閾値・キャリブレーション結果はいずれも自身のデータ操作（取り出し系）のみ保持。Entity anemia（操作なし）なし。Entity overreach（他 Entity の操作・処理ロジック混入）なし。各 Entity の属性が UC ドメイン概念に沿っている（Data Fitness 問題なし）。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 基本 Step 1（`calibrate` 実行） | Actor→受付窓口→統括処理 | キャリブレーション要求を受け付ける→オプションを検証する→キャリブレーションを統括する | Yes | |
| 基本 Step 2（embeddings テーブルから全件ロード） | 統括→埋め込みロード→埋め込みストア境界→埋め込みベクトル集合 | 埋め込みベクトルをロードする→全件を読み出す→件数を取り出す | Yes | |
| 基本 Step 3（全ペア類似度算出、O(N²)） | 統括→全ペア類似度算出→埋め込みベクトル集合→全ペア類似度集合 | 全ペア類似度を算出する→非有限スコアのペアを除外する→ペア数を取り出す | Yes | |
| 基本 Step 4（ヒストグラム生成） | 統括→ヒストグラム生成→全ペア類似度集合→ヒストグラム / 統計サマリ | ヒストグラムを生成する→統計サマリを生成する→バケット一覧を取り出す→統計値を取り出す | Yes | |
| 基本 Step 5（text モード出力） | 統括→結果整形→設定ファイル境界→現閾値設定→キャリブレーション結果 | キャリブレーション結果を生成する→閾値設定を読み出す→閾値一覧を取り出す→結果を取り出す | Yes | |
| 基本 Step 6（`--json` モード出力） | 結果整形処理がフォーマット種別に従い JSON 形式でキャリブレーション結果を生成 | キャリブレーション結果を生成する（フォーマット種別=JSON） | Yes | |
| 基本 Step 7（exit 0 で終了） | 統括処理が正常終了を確定 | 結果を標準出力へ渡す→終了状態種別(成功) | Yes | |

### Alternative Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 1a（--buckets 0 → exit 1） | 受付窓口がオプション検証でエラーを統括処理へ通知 | オプションを検証する→エラーを標準エラー出力へ渡す→終了状態種別(失敗) | Yes | |
| 代替 1b（--recommend → 推奨閾値追加出力） | 統括→推奨閾値算出→全ペア類似度集合→推奨閾値→結果整形→キャリブレーション結果→出力窓口 | 推奨閾値を算出する→ペア数ゼロを検出する→スコア一覧を取り出す→推奨値一覧を取り出す→推奨閾値を結果に加える→結果を取り出す | Yes | |
| 代替 2a（embeddings が空 → INFO + exit 0） | 埋め込みロード→空ストア検出→統括処理が早期終了判定→出力窓口へ INFO | 全件を読み出す(空)→ストアが空かを確認する→空ストアを検出する→早期終了を判定する→情報通知を標準エラー出力へ渡す | Yes | stdout 汚染なし確認 |
| 代替 3a（全ペア算出失敗 → exit 1） | 全ペア算出処理が失敗通知→統括処理→出力窓口へエラー | 全ペア類似度を算出する→結果(算出失敗)→エラーを標準エラー出力へ渡す | Yes | |
| 代替 3b（--recommend かつペア数 0 → stderr INFO、stdout 保護） | 推奨閾値算出処理がペア数 0 を検出→算出不能通知→統括処理→出力窓口（stdout は汚染しない） | ペア数ゼロを検出する→ペア数を取り出す(0)→結果(算出不能)→結果を標準出力へ渡す(推奨閾値なし) + 情報通知を標準エラー出力へ渡す | Yes | OBS.02 準拠確認 |

### Exception Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 全ペア算出失敗（代替 3a と同一） | 算出処理が失敗通知→統括処理が出力窓口へエラー伝搬 | 結果(算出失敗)→エラーを標準エラー出力へ渡す(エラーメッセージ) | Yes | |
| --recommend かつペア数 0（代替 3b と同一） | 推奨閾値算出処理がペア数 0 検出→通常出力 + stderr INFO | ペア数ゼロを検出する→ペア数を取り出す(0)→結果(算出不能)→stdout + stderr 分離出力 | Yes | |

全 UC フロー（基本 Step 1–7 / 代替 1a/1b/2a/3a/3b / 例外）が SEQA / SEQD 上で責務の不整合なく実行可能。

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。CR-001〜CR-028 の全操作が AR または justified split の具体化として説明可能）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None
- **Ambiguous Mappings**: None

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 22 |
| Preserved | 14 |
| Justified split | 8 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100%（(14+8)/22） |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 28 |
| Invention rate（監視用） | 0% |

> split 展開により CR(28) > AR(22)。追加 CR はすべて justified split の内訳（分割後の具体操作）であり、抽象側根拠なき湧出ではない。

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0（全 8 件の split に RBD §1・SEQD フロー・RBA §6 の根拠が明示されている）
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

## 9. Required Changes

- なし（保存失敗なし）

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC フロー実行可能）。抽象責務集合（UC 錨着 22 AR）が具体責務集合（28 CR）へ保存されている。8 件の justified split はいずれも RBD §1 操作識別・SEQD フロー・RBA §6 Object Discovery に根拠を持ち、抽象責務を消失させず具体操作として展開している。

<!-- VERDICT:APPROVE -->
