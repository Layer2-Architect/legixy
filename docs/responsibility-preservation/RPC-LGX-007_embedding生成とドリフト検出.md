Document ID: RPC-LGX-007

# RPC-LGX-007: embedding 生成とドリフト検出 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-007
**対象 RBA**: RBA-LGX-007
**対象 SEQA**: SEQA-LGX-007
**対象 RBD**: RBD-LGX-007
**対象 SEQD**: SEQD-LGX-007
**検査深度**: フル（§14.2: embedding 生成・ハッシュ不変条件・ONNX モデル依存を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

---

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | embedding コマンド受付窓口 | embedding 生成要求（`embed [--all] [--subnodes]`）を受け取る | UC-007 基本 1 |
| AR-002 | RBA | Control | embedding 生成統括処理 | ノード走査・ハッシュ照合・前処理・推論・格納を協調させ、部分失敗時は後続ノードを継続する | UC-007 基本 1–4、例外全般 |
| AR-003 | RBA | Control | ノード一覧取得処理 | グラフ定義境界から対象ノード一覧を確定する | UC-007 基本 2 |
| AR-004 | RBA | Boundary | グラフ定義境界 | `graph.toml` から対象ノード一覧を供給する | UC-007 基本 2 |
| AR-005 | RBA | Entity | 対象ノード一覧 | 確定した処理対象ノードの集合を保持する | UC-007 基本 2 |
| AR-006 | RBA | Control | ハッシュ照合処理 | 成果物コンテンツの SHA-256 を計算し、格納境界の既存ハッシュと比較して スキップ・再生成・未生成 の 3 状態を確定する（SCORE-INV-1 起点）。`--all` 時はスキップ判定を省略する | UC-007 基本 3a–3b / 代替 3b |
| AR-007 | RBA | Boundary | 成果物ファイル境界 | 各成果物ファイル本文を供給する | UC-007 基本 3a |
| AR-008 | RBA | Entity | ノードコンテンツ | 成果物ファイルから供給された本文テキスト（正規化前の入力）を保持する | UC-007 基本 3a–3c |
| AR-009 | RBA | Entity | ハッシュ照合結果 | コンテンツハッシュと既存ハッシュの比較結果（3 状態）を保持する | UC-007 基本 3b / 代替 3b |
| AR-010 | RBA | Control | 前処理適用処理 | スキップ対象外のノードコンテンツに正規化（空テキスト判定・トークン上限超過切り捨て）を適用し前処理済みコンテンツを生成する | UC-007 基本 3c |
| AR-011 | RBA | Entity | 前処理済みコンテンツ | 正規化・切り捨て処理を経た embedding 入力テキストを保持する | UC-007 基本 3c–3d |
| AR-012 | RBA | Control | 推論処理 | ONNX モデル境界を通じてトークン化・推論・Mean Pooling・L2 正規化を実行し embedding ベクトルを生成する。モデル不在時は ERROR を生成結果集計に記録する | UC-007 基本 3d / 代替 2a |
| AR-013 | RBA | Boundary | ONNX モデル境界 | `model.onnx` + `tokenizer.json` を供給する（不在は起動不能） | UC-007 基本 3d / 代替 2a |
| AR-014 | RBA | Entity | embedding ベクトル | 推論処理が生成した意味表現ベクトル（モデルバージョン・コンテンツハッシュを伴う）を保持する | UC-007 基本 3d–3e |
| AR-015 | RBA | Control | embedding 格納処理 | embedding ベクトル・モデル版情報・コンテンツハッシュを embedding 格納境界に書き込む（ノード単位トランザクション）。SCORE-INV-2 を実現する | UC-007 基本 3e / 事後条件 |
| AR-016 | RBA | Boundary | embedding 格納境界 | 既存ハッシュを参照し、生成済み embedding を格納する（`engine.db`） | UC-007 基本 3b, 3e / 事後条件 |
| AR-017 | RBA | Control | サブノード走査処理 | `--subnodes` 指定時にノードのコンテンツ範囲を切り出し、サブノード単位で embedding 生成統括処理に委譲する | UC-007 基本 4 |
| AR-018 | RBA | Entity | 生成結果集計 | 処理全体の件数（生成・スキップ・失敗・エラー詳細）を保持する | UC-007 基本 3b, 3e / 事後条件 |
| AR-019 | RBA | Boundary | 処理結果出力窓口 | 生成件数・スキップ件数・失敗件数・エラー詳細をアクターへ返す | UC-007 基本フロー末尾 / 代替 2a |

全 AR（19 件）が UC ステップに紐づく。SEQA-LGX-007 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない（SEQA §8 にて Jacobson 三者整合性確定済み）。

---

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | embedding コマンド受付窓口（SEQD: 埋め込みコマンド受付窓口） | 生成要求を受け付ける | アクター境界で生成要求を受理し、フラグ種別を統括処理へ渡す | Actor→B受付 |
| CR-002 | RBD/SEQD | embedding 生成統括処理（SEQD: 埋め込み生成統括処理） | 生成フローを統括する / 部分失敗を継続判定する | フロー協調・部分失敗継続 | B受付→C統括、全体統括 |
| CR-003 | RBD/SEQD | ノード一覧取得処理 | 対象ノード一覧を確定する | グラフ定義境界から対象ノード一覧を確定 | C統括→C一覧→Bgraph |
| CR-004 | RBD/SEQD | グラフ定義境界 | グラフ定義を読み込む | グラフ定義供給 | C一覧→Bgraph |
| CR-005 | RBD/SEQD | 対象ノード一覧 | ノードを取り出す | ノードのコレクション保持・取り出し | C一覧→Eノード一覧 |
| CR-006 | RBD/SEQD | ハッシュ照合処理 | ハッシュを照合する | 3 状態照合（SCORE-INV-1 起点）・強制再生成フラグ対応 | C統括→Cハッシュ |
| CR-007 | RBD/SEQD | 成果物ファイル境界 | 成果物本文を読み込む / 成果物の存在を確認する | ファイル本文・存在確認供給 | Cハッシュ→Bファイル |
| CR-008 | RBD/SEQD | ノードコンテンツ | ノードコンテンツを保持する / ノードコンテンツを参照する | 本文テキスト・ハッシュ値保持 | Cハッシュ→Eコンテンツ |
| CR-009 | RBD/SEQD | ハッシュ照合結果 | ハッシュ照合結果を確定する / スキップを判定する | 3 状態（スキップ・再生成・未生成）保持 | Cハッシュ→E照合結果 |
| CR-010 | RBD/SEQD | 前処理適用処理 | 前処理を適用する | 正規化・切り捨て・空テキスト判定適用 | C統括→C前処理 |
| CR-011 | RBD/SEQD | 前処理済みコンテンツ | 前処理済みコンテンツを確定する / 前処理済みコンテンツを参照する | 正規化済みテキスト・切り捨て済みフラグ保持 | C前処理→E前処理済 |
| CR-012 | RBD/SEQD | 推論処理 | 推論を実行する | ONNX モデル参照・トークン化・推論・プーリング・正規化・モデル不在 ERROR | C統括→C推論 |
| CR-013 | RBD/SEQD | ONNX モデル境界（RBD）/ 推論モデル境界（SEQD） | モデルを参照する / モデルの存在を確認する | モデル内容・存在確認供給 | C推論→Bモデル |
| CR-014 | RBD/SEQD | embedding ベクトル（SEQD: 埋め込みベクトル） | 埋め込みベクトルを生成する / 埋め込みベクトルを参照する | ベクトル値・モデルバージョン・コンテンツハッシュ保持 | C推論→Eベクトル |
| CR-015 | RBD/SEQD | embedding 格納処理（SEQD: 埋め込み格納処理） | 埋め込みを格納する | ベクトル・ハッシュ・バージョンの永続化（SCORE-INV-2） | C統括→C格納 |
| CR-016 | RBD/SEQD | embedding 格納境界（SEQD: 埋め込み格納境界） | 既存ハッシュを参照する / ベクトルとハッシュとバージョンを書き込む | ハッシュ参照・embedding 書込供給 | Cハッシュ/C格納→B格納 |
| CR-017 | RBD/SEQD | サブノード走査処理 | サブノードを走査する / コンテンツ範囲を切り出す | コンテンツ範囲切り出し・統括処理への委譲 | C統括→Cサブ |
| CR-018 | RBD/SEQD | 生成結果集計 | 件数を加算する / エラーを記録する | 件数（生成・スキップ・失敗）・エラー詳細保持 | C統括/Cハッシュ/C推論→E集計 |
| CR-019 | RBD/SEQD | 処理結果出力窓口 | 処理結果集計を出力する | 件数・エラー詳細をアクターへ返す | C統括→B出力→Actor |

RBD §4 で明示されているとおり「新規クラスの発見なし（RBA-007 主語と 1:1）」。

---

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・受付操作・フラグ渡し | GREEN |
| AR-002 | CR-002 | preserved | 統括 Control。協調・部分失敗継続を操作化 | GREEN |
| AR-003 | CR-003 | preserved | — | GREEN |
| AR-004 | CR-004 | preserved | — | GREEN |
| AR-005 | CR-005 | preserved | ノードのコレクション保持・取り出し操作を Entity 自身に保持 | GREEN |
| AR-006 | CR-006 | preserved | 3 状態照合・強制再生成フラグ対応・スキップ省略を操作化 | GREEN |
| AR-007 | CR-007 | preserved | ファイル本文・存在確認の 2 操作を識別 | GREEN |
| AR-008 | CR-008 | preserved | 本文テキスト・ハッシュ値保持を Entity 自身の操作に保持 | GREEN |
| AR-009 | CR-009 | preserved | 3 状態保持・スキップ判定を Entity 自身の操作に保持 | GREEN |
| AR-010 | CR-010 | preserved | — | GREEN |
| AR-011 | CR-011 | preserved | 切り捨て済みフラグ属性を追加（RBA §6 Object Discovery 内）。SPEC-LGX-006.REQ.01 範囲内 | GREEN |
| AR-012 | CR-012 | preserved | モデル不在 ERROR 生成を推論処理 Control に保持（SEQD §3 例外 2a） | GREEN |
| AR-013 | CR-013 | **ambiguous** | RBA/SEQA/RBD は「ONNX モデル境界」、SEQD §1 のレーン名は「推論モデル境界」。責務（モデル参照・存在確認）は保存されているが、SEQD と RBD のクラス名が不一致。「ONNX」という固有名詞を「推論」に一般化した命名変更か、SEQD のみ別クラスを想定したものかが SEQD 内では不明 | YELLOW |
| AR-014 | CR-014 | preserved | ベクトル値・モデルバージョン・コンテンツハッシュ保持（SCORE-INV-2 に寄与） | GREEN |
| AR-015 | CR-015 | preserved | ベクトル・ハッシュ・バージョン同時書込・SCORE-INV-2 実現 | GREEN |
| AR-016 | CR-016 | preserved | ハッシュ参照・書込 2 操作を識別 | GREEN |
| AR-017 | CR-017 | **ambiguous** | RBD §1 では「コンテンツ範囲を切り出す」はサブノード走査処理（Control）の操作。しかし SEQD §2（代替 4）では `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す(ノード識別子)` として ノードコンテンツ（Entity）に送っている。SEQD §6 Behavior Allocation 表は「コンテンツ範囲を切り出す → サブノード走査処理」と記録しており、§2 のメッセージ方向と矛盾する。操作がどちらのクラスに帰属するかが SEQD 内で不整合 | YELLOW |
| AR-018 | CR-018 | preserved | 件数加算・エラー記録を Entity 自身の操作に保持 | GREEN |
| AR-019 | CR-019 | preserved | — | GREEN |

19 AR 中 2 件（AR-013、AR-017）が `ambiguous`。lost / mutated / shifted / invented なし。split / merge なし。

---

## 4. Role Fitness Check（§5.2）

### Boundary

- **Finding**: 各 Boundary クラス（受付窓口・グラフ定義境界・成果物ファイル境界・ONNX/推論モデル境界・embedding 格納境界・出力窓口）は境界操作のみ保持。Boundary overreach なし。
  - ただし AR-013/CR-013 で指摘した通り、RBD と SEQD で「ONNX モデル境界」vs「推論モデル境界」のクラス名の不一致がある。責務の boundary 性は維持されている（外部ファイル境界として正当）。
- Severity: Minor（命名不整合）/ 原因の所在: 具体側（SEQD）/ Required action: SEQD のレーン名を RBD のクラス名「ONNX モデル境界」に統一する

### Control

- **Finding**: 各 Control（統括/ノード一覧取得/ハッシュ照合/前処理適用/推論/格納/サブノード走査）は調停・処理に留まり、データ保持なし。Service blob 化なし。Control leakage なし。
  - embedding 生成統括処理は協調のみ（万能化していない）。サブノード走査処理の「コンテンツ範囲を切り出す」操作について、SEQD §2 でメッセージが Eコンテンツ に向かっているが、SEQD §6 Behavior Allocation は正しく Cサブ に帰属させている。SEQD のシーケンス図と Behavior Allocation 表の間で帰属が矛盾している（ambiguous として記録）。
- Severity: Minor（SEQD シーケンス図の表記誤り）/ 原因の所在: 具体側（SEQD §2 図）/ Required action: SEQD §2 の `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す` を、Cサブ 自身が実行する自己メッセージまたは明示的な説明に修正する

### Entity

- **Finding**: 各 Entity（対象ノード一覧・ノードコンテンツ・ハッシュ照合結果・前処理済みコンテンツ・embedding ベクトル・生成結果集計）は自身のデータ操作のみ保持。Entity anemia なし（各 Entity が適切な操作を持つ）、Entity overreach なし。
  - ノードコンテンツの「コンテンツ範囲を切り出す」受信（SEQD §2）は Role Fitness の観点からは問題（Entity が Control の操作を担う構造）であるが、SEQD §6 Behavior Allocation では正しく修正されているため、シーケンス図の表記誤りと判断する。
- Severity: Minor（SEQD §2 の表記のみ）/ 原因の所在: 具体側（SEQD §2 図）/ Required action: §4 Control 節の Required action と同一

---

## 5. Sequential Execution Check（§5.3）

### Basic Flow

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 基本 1（embed 実行） | Actor→B受付→C統括 | 生成要求を受け付ける→生成フローを統括する | Yes | |
| 基本 2（graph.toml から全ノード取得） | C統括→C一覧→Bgraph→Eノード一覧 | 対象ノード一覧を確定する→グラフ定義を読み込む | Yes | |
| 基本 3a（SHA-256 計算） | C統括→Cハッシュ→Bファイル→Eコンテンツ | ハッシュを照合する→成果物本文を読み込む→ノードコンテンツを保持する | Yes | |
| 基本 3b（既存ハッシュ比較・スキップ判定、SCORE-INV-1） | Cハッシュ→B格納→E照合結果（スキップ分岐） | 既存ハッシュを参照する→ハッシュ照合結果を確定する→スキップ分岐 | Yes | |
| 基本 3c（前処理適用） | C統括→C前処理→Eコンテンツ→E前処理済 | 前処理を適用する→ノードコンテンツを参照する→前処理済みコンテンツを確定する | Yes | |
| 基本 3d（ONNX モデルで embedding 生成） | C統括→C推論→Bモデル/E前処理済→Eベクトル | 推論を実行する→モデルを参照する→前処理済みコンテンツを参照する→埋め込みベクトルを生成する | Yes | SEQD レーン「推論モデル境界」は RBD の「ONNX モデル境界」と同一。責務は保存 |
| 基本 3e（embeddings テーブルに格納、SCORE-INV-2） | C統括→C格納→Eベクトル→B格納 | 埋め込みを格納する→埋め込みベクトルを参照する→ベクトルとハッシュとバージョンを書き込む | Yes | |
| 基本 4（--subnodes サブノード処理） | C統括→Cサブ→Eノード一覧→C統括 | サブノードを走査する→ノードを取り出す→生成フローを統括する（委譲） | Yes（一部 ambiguous） | SEQD §2 の `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す` はシーケンス図表記が不明確（§4 Control 節参照）だが、責務の流れは SEQD §6 に従えば実行可能 |
| 事後条件（embeddings 更新・モデルバージョン記録） | C格納→B格納（モデル版情報同時書込） | ベクトルとハッシュとバージョンを書き込む | Yes | |

### Alternative Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 3b（--all 強制再生成） | Cハッシュ→E照合結果（全再生成確定） | ハッシュを照合する(強制再生成フラグ)→ハッシュ照合結果を確定する(再生成固定) | Yes | SEQD §2 でスキップ判定省略を明示 |
| 代替 4（--subnodes） | C統括→Cサブ→Eノード一覧→C統括 | サブノードを走査する→ノードを取り出す→コンテンツ範囲を切り出す→生成フローを統括する | Yes（シーケンス図表記は ambiguous、§4 参照） | 委譲後は基本フロー再生成パスを実行 |

### Exception Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 2a（ONNX モデル不在 ERROR） | C推論→Bモデル（不在）→E集計→B出力 | C推論→Bモデル: モデルの存在を確認する→不在→E集計: エラーを記録する→B出力: ERROR 報告 | Yes | SEQD §3 例外 2a で「推論モデル境界」名を使用。責務は保存。モデル不在は即時終了 |
| 例外（一部ノード読込失敗 部分失敗継続） | C統括→Cハッシュ→Bファイル（失敗）→E集計→継続 | ハッシュを照合する→成果物本文を読み込む（失敗）→エラーを記録する→部分失敗を継続判定する | Yes | SEQD §3 例外フロー、SEQD §5 失敗伝搬で明示 |

全 UC フローが SEQA / SEQD 上で責務の不整合なく実行可能（ambiguous 2 件はいずれも責務の流れ自体は正しく、SEQD §6 Behavior Allocation 表で整合している）。

---

## 6. Mismatches

### Lost Responsibilities
None

### Invented Responsibilities
None（具体側に抽象側根拠のない責務なし）

### Shifted Responsibilities
None

### Mutated Responsibilities
None

### Ambiguous Mappings

**AMB-001（AR-013 ↔ CR-013）**: ONNX モデル境界の命名不一致
- 抽象側（RBA/SEQA）および具体側 RBD は「ONNX モデル境界」を使用。
- SEQD §1 のレーン alias は「推論モデル境界」、§3 例外 2a も「推論モデル境界」を使用。
- SEQD §6 Behavior Allocation は `モデルを参照する / モデルの存在を確認する | 推論モデル境界` と記録。
- 責務（モデル内容・存在確認の供給）は一致しているが、クラス名が RBD と SEQD で齟齬している。
- **原因の所在**: 具体側（SEQD のみ）。RBD の「ONNX モデル境界」という名称を SEQD が「推論モデル境界」と変えた。RBA/SEQA/RBD で一貫した名称を SEQD が逸脱。
- **解消方法**: SEQD の「推論モデル境界」を「ONNX モデル境界」に統一する（RBD との整合）。REQUEST_CHANGES で AI 自律修正可能。

**AMB-002（AR-017 ↔ CR-017）**: サブノード走査処理の「コンテンツ範囲を切り出す」操作帰属の不整合
- RBD §1 では「コンテンツ範囲を切り出す」はサブノード走査処理（Control）の操作として定義。
- SEQD §6 Behavior Allocation も「コンテンツ範囲を切り出す → サブノード走査処理」と記録（Control への正しい帰属）。
- しかし SEQD §2（代替 4）のシーケンス図では `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す(ノード識別子)` とノードコンテンツ（Entity）宛にメッセージを送っており、§6 の帰属と矛盾する。
- **原因の所在**: 具体側（SEQD §2 図のみ）。SEQD §6 の Behavior Allocation 表は正しい。シーケンス図の描き方が誤りで、`Cサブ` が自分で切り出す（自己操作）ことを `Eコンテンツ` への依頼として誤って書いた。
- **解消方法**: SEQD §2 の `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す(ノード識別子)` + `Eコンテンツ-->>Cサブ: 本文テキスト（サブノード範囲）` を、`Cサブ->>Eコンテンツ: ノードコンテンツを参照する(ノード識別子)` + `Eコンテンツ-->>Cサブ: 本文テキスト（サブノード範囲）` に修正し、`コンテンツ範囲を切り出す` は Cサブ の内部操作として Note over 等で表現する。REQUEST_CHANGES で AI 自律修正可能。

---

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 19 |
| Preserved | 17 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 2 |
| Preservation rate（監視用） | 89.5%（17/19。ambiguous 2 件は分子に含めない） |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 19 |
| Invention rate（監視用） | 0% |

---

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（AMB-001/AMB-002 は SEQD-LGX-007 修正で解消、review-fix loop 2026-06-13）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし（責務の配置は正しい。SEQD §2 のシーケンス図表記は誤りだが §6 で正しく修正されている）
- [x] UC 基本/代替/例外フローが具体側で実行可能（SEQD §6 に従えば）

**絶対条件ゲート: GREEN**（AMB-001/AMB-002 は SEQD-LGX-007 修正で解消済、review-fix loop 2026-06-13。下記 §10 参照）

---

## 9. Required Changes

### RC-001（AMB-001 解消）

- **対象**: SEQD-LGX-007 §1・§2・§3
- **内容**: 「推論モデル境界」という participant alias / レーン名を「ONNX モデル境界」に統一する
- **箇所**:
  - SEQD §1 基本フロー: `participant Bモデル as 推論モデル境界` → `participant Bモデル as ONNX モデル境界`
  - SEQD §2 代替 3b: 明示的な `Bモデル` 参照がある場合は同様に修正
  - SEQD §3 例外 2a: `participant Bモデル as 推論モデル境界` → `participant Bモデル as ONNX モデル境界`
  - SEQD §6 Behavior Allocation: `推論モデル境界` → `ONNX モデル境界`
- **原因の所在**: 具体側（SEQD）。UC は正しい。
- **対応 VERDICT**: REQUEST_CHANGES（AI 自律修正）

### RC-002（AMB-002 解消）

- **対象**: SEQD-LGX-007 §2（代替 4: --subnodes サブノード走査）
- **内容**: サブノード走査のシーケンス図で `Cサブ->>Eコンテンツ: コンテンツ範囲を切り出す(ノード識別子)` を修正し、切り出し操作が `サブノード走査処理`（Cサブ）に帰属することを明示する
- **修正後イメージ**:
  ```mermaid
  Cサブ->>Eコンテンツ: ノードコンテンツを参照する(ノード識別子)
  Eコンテンツ-->>Cサブ: 本文テキスト（全体）
  Note over Cサブ: コンテンツ範囲を切り出す（サブノード範囲を特定）
  Cサブ->>C統括: 生成フローを統括する(サブノード単位フラグ種別)
  ```
- **原因の所在**: 具体側（SEQD §2 図）。SEQD §6 Behavior Allocation は既に正しい帰属を記録している。UC は正しい。
- **対応 VERDICT**: REQUEST_CHANGES（AI 自律修正）

---

## 10. Verdict（§9 規律）

初回検査で ambiguous 2 件（AMB-001: ONNX モデル境界の命名不整合、AMB-002: コンテンツ範囲を切り出す操作帰属の SEQD 内不整合）を検出した。いずれも具体側（SEQD）の表記・命名誤りで、抽象責務の本質（モデル供給・サブノード走査）は保存されており UC/RBA/SEQA/RBD は正しい（§9.1 分解 (a) 具体側逸脱）。

**review-fix loop（2026-06-13）で解消**: §9.2 規律（ambiguous は人間にエスカレートせず AI が SEQD を修正）に従い SEQD-LGX-007 を修正 — RC-001: participant / Behavior Allocation の「推論モデル境界」を「ONNX モデル境界」へ統一（RBD-007 と一致）、RC-002: §2 サブノード走査図を「ノードコンテンツを参照する」+ 切り出しは Control 自身処理の Note へ修正。再検査で全 19 AR preserved、ambiguous=0、絶対条件ゲート GREEN。

<!-- VERDICT:APPROVE -->
