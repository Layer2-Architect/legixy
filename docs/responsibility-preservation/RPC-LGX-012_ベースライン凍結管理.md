Document ID: RPC-LGX-012

# RPC-LGX-012: ベースライン凍結管理 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-012
**対象 RBA**: RBA-LGX-012
**対象 SEQA**: SEQA-LGX-012
**対象 RBD**: RBD-LGX-012
**対象 SEQD**: SEQD-LGX-012
**検査深度**: フル（§14.2: snapshot ライフサイクル全体を担う中クリティカリティ UC。代替フロー多岐・非対称終了コード契約あり）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | スナップショットコマンド受付窓口 | アクターからのスナップショット操作要求（create / list / delete）を受け取る。サブコマンド省略は exit 2 で終端 | UC-012 Step 1, 4, 6 / 代替 1a |
| AR-002 | RBA | Control | スナップショット統括処理 | サブコマンド種別を判定し、凍結処理 / 一覧処理 / 削除処理のいずれかへ振り分ける | UC-012 Step 1, 4, 6 / 全代替・例外 |
| AR-003 | RBA | Control | 凍結処理 | embeddings ストア境界から現行全行を単一トランザクションでスナップショット領域境界へ複製し、一意な凍結識別子を発行する。空ストア時は非永続・WARNING | UC-012 Step 2, 3 / 代替 2a / 例外（トランザクション失敗） |
| AR-004 | RBA | Boundary | embeddings ストア境界 | 凍結対象となる現行 embedding 全行を供給する。engine.db 不在は空ストア相当 | UC-012 Step 2 / 代替 2a |
| AR-005 | RBA | Boundary | スナップショット領域境界 | 凍結済みスナップショット行の永続供給元・書込先（engine.db 内の分離領域） | UC-012 Step 2, 4, 6 / 代替 6a, 6b, 6c |
| AR-006 | RBA | Entity | 凍結識別子 | `snap-` プレフィクスを持つ一意な識別子を保持し取り出す | UC-012 Step 3 |
| AR-007 | RBA | Entity | スナップショット行集合 | snapshot_id / label / node_count / taken_at / content_hash / model_version を含む複製済み行集合を保持する | UC-012 Step 2 / 関連不変条件 SCORE-INV-1 |
| AR-008 | RBA | Control | 一覧処理 | スナップショット領域境界から全スナップショット行を読み取り、taken_at 降順で整列して一覧情報を組み立てる。0 件時は案内メッセージ相当 | UC-012 Step 4 / 代替 4a |
| AR-009 | RBA | Entity | 一覧情報 | snapshot_id / label / node_count / taken_at を持つ taken_at 降順整列済み表現を保持し提供する | UC-012 Step 4 / 代替 4a |
| AR-010 | RBA | Control | 削除処理 | 削除対象（凍結識別子または label 参照）を受け取り、label 参照時は label 解決処理に委譲し、対象行をスナップショット領域境界から除去する | UC-012 Step 6 / 代替 6a, 6b |
| AR-011 | RBA | Control | label 解決処理 | `label:<L>` 形式を taken_at 最新の 1 件へ決定論的に解決する。label 不在は ERROR + exit 1 | UC-012 代替 6a, 6c |
| AR-012 | RBA | Entity | 削除結果 | 削除操作の結果（対象識別子 / 削除行数）を保持する | UC-012 Step 6 / 代替 6a, 6b |
| AR-013 | RBA | Boundary | 結果出力窓口 | 操作結果（snapshot_id / 一覧 / 削除確認）を stdout に、診断メッセージ（INFO / WARNING / ERROR）を stderr に区別してアクターへ返す | UC-012 Step 3, 4, 7 / 全代替 |

全 13 AR が UC ステップ（または UC の代替・例外フロー）に紐づく。UC ステップに紐づかない AR なし → 構造翻訳が情報を加えていない（§9 分解(b) 候補なし）。SEQA-LGX-012 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | SEQD Message / SEQD section |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | スナップショットコマンド受付窓口 | スナップショット凍結を受け付ける(ラベル参照) / ベースライン一覧要求を受け付ける() / スナップショット削除を受け付ける(削除対象識別) / 使用法誤りを返す() | アクター境界で操作要求を受理。サブコマンド省略時は境界で終端 | 基本フロー §1 / 代替 1a |
| CR-002 | RBD/SEQD | スナップショット統括処理 | スナップショット操作を統括する(操作要求) / サブコマンド種別を判定する(操作要求) | サブコマンド種別を判定し各処理へ委譲する | 基本フロー §1 / 全代替 |
| CR-003 | RBD/SEQD | 凍結処理 | 凍結を実行する(ラベル参照) / 空ストアを検出する(行集合) / 永続化をスキップする() | 読取・複製・識別子発行・空ストア検出・非永続スキップ | 基本 §1-1 / 代替 2a / 例外 §3 |
| CR-004 | RBD/SEQD | embeddings ストア境界 | 現行全行を読み取る() / ストアの存在を確認する() | 現行 embedding 全行の供給と存在確認 | 基本 §1-1 / 代替 2a / 例外 §3 |
| CR-005 | RBD/SEQD | スナップショット領域境界 | 全スナップショット行を読み取る() / 対象 label の全行を問い合わせる(ラベル参照) / 現行全行を単一トランザクションで複製する(行集合) / 対象行を除去する(凍結識別子) | 読取・label 問合せ・複製・除去の永続操作 | 基本 §1-1〜1-3 / 代替 6a, 6b, 6c / 例外 §3 |
| CR-006 | RBD/SEQD | 凍結識別子 | 識別子を取り出す() | 識別子値（snap- プレフィクス）を提供する | 基本 §1-1（C1→Eid） |
| CR-007 | RBD/SEQD | スナップショット行集合 | 行集合の件数を取り出す() | 複製行数（node_count）を保持し提供する | 基本 §1-1（C1→Esnap） |
| CR-008 | RBD/SEQD | 一覧処理 | 一覧を組み立てる() / 取得件数 0 件を処理する() | 全行読取・降順整列・0 件処理・一覧情報組立 | 基本 §1-2 / 代替 4a |
| CR-009 | RBD/SEQD | 一覧情報 | 整列済み行を取り出す() / 件数を取り出す() | 整列済み行集合と件数を提供する | 基本 §1-2（C2→Elist） / 代替 4a |
| CR-010 | RBD/SEQD | 削除処理 | 削除を実行する(削除対象識別) / 削除行数を確認する(削除結果) | label 参照委譲・対象行除去・削除行数確認 | 基本 §1-3 / 代替 6a, 6b |
| CR-011 | RBD/SEQD | label 解決処理 | ラベル参照を解決する(ラベル参照) / 最新一件へ決定論的に解決する(行集合) / ラベル解決失敗を確定する() | taken_at 最新解決・label 不在時の解決失敗確定 | 代替 6a / 代替 6c |
| CR-012 | RBD/SEQD | 削除結果 | 削除行数を取り出す() | 対象識別子・削除行数を保持し提供する | 基本 §1-3（C3→Edel） / 代替 6a, 6b |
| CR-013 | RBD/SEQD | 結果出力窓口 | 操作結果を出力する(凍結識別子, 出力形式種別) / 一覧を出力する(一覧情報, 出力形式種別) / 削除確認を出力する(削除結果, 出力形式種別) / 診断メッセージを出力する(診断メッセージ, 診断種別) | stdout へ操作結果・stderr へ診断メッセージの区別出力 | 基本 §1-1〜1-3 / 全代替 |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001（スナップショットコマンド受付窓口） | CR-001 | preserved | 同一 Boundary。create/list/delete 三操作受付 + 使用法誤り終端（代替 1a）を CR-001 の 4 操作に 1:1 識別。役割逸脱なし | GREEN |
| AR-002（スナップショット統括処理） | CR-002 | preserved | 同一 Control。統括・サブコマンド判定の 2 操作を識別。全代替フローで統括処理が中継点として機能 | GREEN |
| AR-003（凍結処理） | CR-003 | preserved | 同一 Control。凍結実行・空ストア検出・非永続スキップの 3 操作を識別。例外（トランザクション失敗）も凍結処理内で処理 | GREEN |
| AR-004（embeddings ストア境界） | CR-004 | preserved | 同一 Boundary。現行全行読取 + ストア存在確認の 2 操作を識別。RBD で「ストアの存在を確認する」が追加されているが、SEQA §2（代替 2a）の「engine.db 不在は空ストア相当」を操作化したものでありUC/SPEC 範囲内 | GREEN |
| AR-005（スナップショット領域境界） | CR-005 | preserved | 同一 Boundary。全行読取・label 問合せ・トランザクション複製・対象行除去の 4 操作を識別。RBA の「読取・複製・問合せ・除去」と完全対応 | GREEN |
| AR-006（凍結識別子） | CR-006 | preserved | 同一 Entity。`識別子を取り出す()` という自身データ操作のみ保持。RBD で識別子値・プレフィクス種別の属性が識別されたがドメイン概念の具体化であり湧出なし | GREEN |
| AR-007（スナップショット行集合） | CR-007 | preserved | 同一 Entity。`行集合の件数を取り出す()` を識別。属性（スナップショット識別子 / ラベル / ノード数 / 凍結日時 / 内容ハッシュ / モデルバージョン）は SCORE-INV-1 の具体化 | GREEN |
| AR-008（一覧処理） | CR-008 | preserved | 同一 Control。全行読取・降順整列・0 件処理（代替 4a）の責務が `一覧を組み立てる()` / `取得件数 0 件を処理する()` に保存 | GREEN |
| AR-009（一覧情報） | CR-009 | preserved | 同一 Entity。`整列済み行を取り出す()` / `件数を取り出す()` の 2 操作を識別。自身データの提供に留まり overreach なし | GREEN |
| AR-010（削除処理） | CR-010 | preserved | 同一 Control。`削除を実行する()` / `削除行数を確認する()` の 2 操作を識別。代替 6b（削除行数 0 の検出）が `削除行数を確認する()` に保存 | GREEN |
| AR-011（label 解決処理） | CR-011 | preserved | 同一 Control。`ラベル参照を解決する()` / `最新一件へ決定論的に解決する()` / `ラベル解決失敗を確定する()` の 3 操作を識別。代替 6a（複数存在→最新優先）と 6c（不在→exit 1）の非対称性が操作レベルで完全保存 | GREEN |
| AR-012（削除結果） | CR-012 | preserved | 同一 Entity。`削除行数を取り出す()` の自身データ操作のみ。対象識別子・削除行数の属性は RBA ドメイン概念の具体化 | GREEN |
| AR-013（結果出力窓口） | CR-013 | preserved | 同一 Boundary。stdout への操作結果（3 操作）/ stderr への診断メッセージ（1 操作）の計 4 操作を識別。6b の text/json 非対称（WARNING あり/なし）が `診断メッセージを出力する(診断種別)` の診断種別パラメータに吸収されている | GREEN |

13 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-012 §4 mapping が新規クラスなしを確認済み。

**追加確認（SEQD で操作追加があるか）**: SEQD §1-1 で `C0->>C0: サブコマンド種別を判定する(操作要求)` が内部呼び出しとして現れるが、これは AR-002 の「サブコマンド種別を判定する」責務の時系列実装であり invented ではない。SEQD §1-1 の `C1->>C1: 空ストアを検出する(行集合)` も AR-003 の「空ストア時は非永続」責務の実装。いずれも UC ステップ錨着済み。

## 4. Role Fitness Check（§5.2）

### Boundary

- **スナップショットコマンド受付窓口**: アクター境界操作（受付 3 + 使用法誤り終端 1）のみ保持。代替 1a の `使用法誤りを返す()` は境界内完結（SEQD §2 で B1→B1 の内部呼出し → B2 出力）であり Boundary overreach なし。
- **embeddings ストア境界**: 現行全行読取 + 存在確認の 2 操作のみ。データ加工なし。Boundary overreach なし。
- **スナップショット領域境界**: 読取・label 問合せ・複製・除去の 4 操作のみ。ビジネスロジック（taken_at 最新解決、非永続判定）は Control に委譲。Boundary overreach なし。
- **結果出力窓口**: stdout/stderr 振り分け出力のみ。6b の text/json 分岐は `出力形式種別` パラメータで吸収されており、出力形式判定が境界内に留まる（Boundary 固有責務の範囲内）。
- Finding: Boundary overreach なし / Severity: なし / Required action: なし

### Control

- **スナップショット統括処理**: サブコマンド判定と各処理への委譲のみ。データ保持なし。Service blob 化なし（統括処理は協調のみ）。
- **凍結処理**: 読取依頼→複製依頼→識別子発行→空ストア検出→非永続スキップ。ストア操作は Boundary に委譲。凍結識別子の「発行」は凍結処理の正当な責務（SEQD で C1→Eid の関係）。Control leakage なし。
- **一覧処理**: 全行読取依頼→降順整列→0 件処理→一覧情報組立。整列ロジックは一覧処理固有。Service blob 化なし。
- **削除処理**: label 参照判定→label 解決委譲→対象行除去→削除行数確認。Control leakage なし。
- **label 解決処理**: taken_at 最新解決（決定論的）と解決失敗確定に特化。単一責務。Control leakage なし。
- Finding: Service blob / Control leakage なし / Severity: なし / Required action: なし

### Entity

- **凍結識別子**: 識別子値の提供のみ。Entity anemia 懸念を確認 → `識別子を取り出す()` 1 操作はドメイン的に適切（不透明トークンを外部に返す）。
- **スナップショット行集合**: `行集合の件数を取り出す()` 1 操作のみ。件数以外の属性（content_hash / model_version 等）はスナップショット領域境界経由でのみ読み書き。Entity anemia 懸念あり → ただし本ドメインは「永続領域から読み取った複製データ」であり、件数のみ公開する設計は §3.5 の「概念領域の汚染なし」要件の観点では正当。UC 不変条件 SCORE-INV-1 の content_hash / model_version は永続化契約（Boundary が担保）であり Entity 操作として公開する必然性はない。
- **一覧情報**: `整列済み行を取り出す()` / `件数を取り出す()` の 2 操作。自身の整列済みデータ操作に留まる。Entity overreach なし。
- **削除結果**: `削除行数を取り出す()` 1 操作。自身データ操作のみ。
- Finding: Entity anemia / overreach なし（スナップショット行集合の件数のみ公開は正当）/ Severity: なし / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Step 1（`snapshot create` 実行） | Actor→受付窓口→統括→凍結処理 | スナップショット凍結を受け付ける → 統括する → 種別判定 → 凍結を実行する | Yes | |
| Step 2（embeddings 全行を単一トランザクションで複製） | 凍結処理→ストア境界→スナップショット領域境界→スナップショット行集合 | ストアの存在を確認する → 現行全行を読み取る → 空ストアを検出する → 現行全行を単一トランザクションで複製する | Yes | 存在確認→読取→複製の順序正常 |
| Step 3（snapshot_id 発行・返却） | 凍結処理→凍結識別子→結果出力窓口→Actor | 識別子を取り出す → 行集合の件数を取り出す → 操作結果を出力する | Yes | |
| Step 4（`snapshot list` 確認） | 統括→一覧処理→スナップショット領域境界→一覧情報→出力窓口 | 一覧を組み立てる → 全スナップショット行を読み取る → 整列済み行を取り出す → 件数を取り出す → 一覧を出力する | Yes | |
| Step 5（UC-LGX-013 参照）| スナップショット領域境界が参照元 | （本 UC 範囲外） | Yes | 後続利用の注記のみ |
| Step 6（`snapshot delete` 実行） | 統括→削除処理→スナップショット領域境界→削除結果→出力窓口 | 削除を実行する → 対象行を除去する → 削除行数を取り出す → 削除行数を確認する → 削除確認を出力する | Yes | |
| Step 7（exit 0） | 結果出力窓口→Actor（exit 0） | Actor: 終了コード 0 | Yes | |

### Alternative Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 1a（サブコマンド省略→exit 2） | 受付窓口→出力窓口（統括処理経由せず） | 使用法誤りを返す() → 診断メッセージを出力する → exit 2 | Yes | SEQD で B1→B1 内部呼出し → B2 出力として正確に再現 |
| 代替 2a（空ストア create→WARNING+exit 0 非永続） | 凍結処理→ストア境界（0件）→非永続スキップ→出力窓口 | 現行全行を読み取る(0件) → 空ストアを検出する → 永続化をスキップする → 診断メッセージ+操作結果を出力する | Yes | WARNING は診断メッセージ、返却 snapshot_id は操作結果、非対称出力が正確に保存 |
| 代替 4a（list 0件→案内+exit 0） | 一覧処理→スナップショット領域境界（0件）→案内 | 一覧を組み立てる → 全スナップショット行を読み取る(0件) → 取得件数0件を処理する → 件数を取り出す(0) → 一覧を出力する | Yes | |
| 代替 6a（delete label:<L> 複数存在→最新解決） | 削除処理→label解決処理→スナップショット領域境界（複数件）→最新解決→対象行除去 | ラベル参照を解決する → 対象labelの全行を問い合わせる(複数) → 最新一件へ決定論的に解決する → 解決済み凍結識別子返却 → 対象行を除去する | Yes | |
| 代替 6b（delete snapshot_id 0件→WARNING+exit 0 / json 警告なし） | 削除処理→スナップショット領域境界（0行削除）→削除行数確認→出力窓口 | 対象行を除去する(0行) → 削除行数を取り出す(0) → 削除行数を確認する → 削除確認を出力する（text: WARNING+exit0 / json: 0件のみ+exit0） | Yes | 非対称出力が `診断種別` パラメータで吸収されている |
| 代替 6c（delete label:<L> 不在→ERROR+exit 1） | label解決処理→解決失敗→削除処理失敗→統括→出力窓口（ERROR+exit 1） | ラベル参照を解決する → 対象labelの全行を問い合わせる(0件) → ラベル解決失敗を確定する → 解決失敗返却 → 削除操作結果（解決失敗） → 診断メッセージを出力する → ERROR+exit 1 | Yes | 6b(exit 0)との非対称が SEQD §2 でも明示 |

### Exception Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外: create トランザクション失敗（原子的ロールバック） | 凍結処理→スナップショット領域境界（失敗）→原子ロールバック→凍結失敗→出力窓口（ERROR+exit 1） | 凍結を実行する → 現行全行を読み取る → 現行全行を単一トランザクションで複製する(失敗) → 凍結操作結果（失敗） → 診断メッセージを出力する → ERROR+exit 1 | Yes | SEQD §3 で「トランザクション失敗時は原子的にロールバック」が Note として明示 |

全 UC フローが SEQA / SEQD 上で責務の不整合なく実行可能。

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。RBD で追加された「ストアの存在を確認する()」は SEQA §2 / UC REQ.07「engine.db 不在時は空ストア相当」の操作化であり UC 範囲内）
- **Shifted Responsibilities**: None（B/C/E 役割が具体側で移動していない）
- **Mutated Responsibilities**: None（代替 6b text/json 出力非対称は `診断種別` パラメータで吸収されており意味変質なし）
- **Ambiguous Mappings**: None

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 13 |
| Preserved | 13 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 13 |
| Invention rate（監視用） | 0% |

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（解消済）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

## 9. Required Changes

- なし（保存失敗なし）

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC の基本 Step 1-7・代替 1a/2a/4a/6a/6b/6c・例外（create トランザクション失敗）の全フローが具体側で実行可能）。

特記: 代替 6b（snapshot_id 不在=exit 0）と代替 6c（label 不在=exit 1）の意図的非対称が、SEQD §2 の代替 6b/6c 両フローで明示的に維持されていることを確認した。RBD §4 mapping が新規クラスなしを明示（抽象主語 13 に対し具体クラス 13 の 1:1）。

<!-- VERDICT:APPROVE -->
