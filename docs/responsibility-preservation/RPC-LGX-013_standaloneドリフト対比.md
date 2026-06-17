Document ID: RPC-LGX-013

# RPC-LGX-013: standalone ドリフト対比 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-013
**対象 RBA**: RBA-LGX-013
**対象 SEQA**: SEQA-LGX-013
**対象 RBD**: RBD-LGX-013
**対象 SEQD**: SEQD-LGX-013
**検査深度**: フル（§14.2: drift は ONNX モデル依存・exit コード非対称・SCORE-INV-2 違反検出を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

---

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

RBA-LGX-013（B×7 / C×8 / E×5）および SEQA-LGX-013 のレーン・メッセージから抽出。

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 対比コマンド受付窓口 | アクターからの `drift` 実行要求（artifact_id・--against・--json）を受け取る | UC-013 基本 Step 1 |
| AR-002 | RBA | Control | 対比統括処理 | モデル解決・成果物解決・embedding 生成・ベースライン解決・整合性検査・drift 算出・結果出力を協調させる | UC-013 基本 Step 1–7 |
| AR-003 | RBA | Boundary | モデル境界 | ONNX モデルを解決順序に従って供給する | UC-013 基本 Step 2 |
| AR-004 | RBA | Control | モデル解決処理 | モデル境界を解決順序（フラグ→LGX_MODELS_DIR→TE_MODELS_DIR→設定ファイル）に従って解決する。旧名解決時は stderr 案内し続行。全解決失敗・読込失敗は exit 1 | UC-013 基本 Step 2 / 代替 2a / 代替 2b |
| AR-005 | RBA | Entity | モデル解決設定 | 解決経路・モデル所在・モデル版数の解決済み値を保持する | UC-013 基本 Step 2 |
| AR-006 | RBA | Boundary | グラフ定義境界 | graph.toml から対象成果物の登録確認とファイルパスを供給する | UC-013 基本 Step 3 / 代替 3a / 代替 3b |
| AR-007 | RBA | Control | 成果物解決処理 | グラフ定義境界から対象成果物の登録確認とファイルパスを解決する。未登録は exit 1、現行ファイル欠落は exit 1 | UC-013 基本 Step 3 / 代替 3a / 代替 3b |
| AR-008 | RBA | Boundary | 現行ファイル境界 | 対象成果物の現行ファイル内容を供給する | UC-013 基本 Step 3 / 代替 3b |
| AR-009 | RBA | Entity | 成果物参照 | 登録確認済みの対象成果物識別子と現行ファイルパスを保持する | UC-013 基本 Step 3 |
| AR-010 | RBA | Control | embedding 生成処理 | 現行ファイル境界の内容を読み込み、解決済みモデルで embedding を生成する | UC-013 基本 Step 3 |
| AR-011 | RBA | Entity | 現行 embedding | 現行ファイル内容から生成した embedding 値（次元数・model_version）を保持する | UC-013 基本 Step 3 |
| AR-012 | RBA | Boundary | embeddings ストア境界 | 保存済み embedding 行（現行保存行 = デフォルトベースライン）を供給する | UC-013 基本 Step 4 / 代替 4a |
| AR-013 | RBA | Boundary | スナップショット境界 | 凍結済みスナップショットのベースライン行を供給する（--against snapshot:<token> 指定時） | UC-013 基本 Step 4 / 代替 4b / 代替 snapshot:label:<L> 失敗 |
| AR-014 | RBA | Control | ベースライン解決処理 | --against 省略・snapshot:<token>・snapshot:label:<L> の 3 形式を解決する。プレフィクス欠如は exit 1。label 複数一致は taken_at 最新 1 件へ決定論的解決。明示 label 失敗は exit 1。不在は exit 0（INFO） | UC-013 基本 Step 4 / 代替 1a / 4a / 4b / snapshot:label 失敗 |
| AR-015 | RBA | Entity | ベースライン embedding | embeddings ストアまたはスナップショットから取得したベースライン値（次元数・model_version・供給元情報）を保持する | UC-013 基本 Step 4 |
| AR-016 | RBA | Control | 整合性検査処理 | ベースラインと現行 embedding の次元数一致・model_version 完全一致を検査する。次元不一致は exit 1、model_version 不一致（SCORE-INV-2 違反）は exit 1 | UC-013 基本 Step 5 / 代替 5a / 5b |
| AR-017 | RBA | Control | drift 算出処理 | drift = 1.0 − cosine 類似度を算出する。非有限スコア（NaN/±Inf）は exit 1 | UC-013 基本 Step 6 / 代替 6a |
| AR-018 | RBA | Entity | 対比結果 | drift 値・baseline_available フラグ・baseline_source を含む集約済み対比情報を保持する | UC-013 基本 Step 6–7 / 代替 4a |
| AR-019 | RBA | Control | 対比結果集約処理 | 対比結果を集約し対比結果出力窓口へ渡す | UC-013 基本 Step 7 / 代替 4a |
| AR-020 | RBA | Boundary | 対比結果出力窓口 | drift 値（text / --json）を stdout に、ログ・診断を stderr に区別してアクターへ返す | UC-013 基本 Step 7 / 例外（--json 出力形式切替） |

全 20 AR が UC ステップに紐づく（UC ステップに紐づかない AR なし → 構造翻訳が情報を加えていない、§9 分解(b) 候補なし）。SEQA-LGX-013 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

---

## 2. Concrete Responsibilities

RBD-LGX-013（B×7 / C×8 / E×5）および SEQD-LGX-013 の操作呼び出しから抽出。

| CR-ID | Source | Class | Operation | Responsibility | Message/Section |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 対比コマンド受付窓口 | 対比要求を受け付ける | アクター境界で drift 実行要求を受理 | Actor→B1 |
| CR-002 | RBD/SEQD | 対比統括処理 | 対比を統括する | 各処理を順に依頼し対比フロー全体を協調 | B1→C0 |
| CR-003 | RBD/SEQD | モデル境界 | モデルを解決順序に従って読み込む / モデルの存在を確認する | ONNX モデル供給（解決順序に従う） | C1→Bmodel |
| CR-004 | RBD/SEQD | モデル解決処理 | モデルを解決する / 旧名解決を案内する | 解決順序ロジック・旧名案内 | C0→C1 |
| CR-005 | RBD/SEQD | モデル解決設定 | 解決済みモデルを参照する | モデル解決経路・所在・版数を保持し参照を返す | C1→Emsetting |
| CR-006 | RBD/SEQD | グラフ定義境界 | 成果物の登録を確認する / ファイルパスを読み込む | graph.toml から登録確認とファイルパス供給 | C2→Bgraph |
| CR-007 | RBD/SEQD | 成果物解決処理 | 成果物を解決する | 登録確認とパス解決の協調。未登録・欠落は exit 1 | C0→C2 |
| CR-008 | RBD/SEQD | 現行ファイル境界 | 現行ファイル内容を読み込む / ファイルの存在を確認する | 現行ファイル内容・存在の供給 | C2→Bfile, C3→Bfile |
| CR-009 | RBD | 成果物参照 | （属性保持） | 成果物識別子・現行ファイルパスを保持 | C2→Eartifact |
| CR-010 | RBD/SEQD | 埋め込み生成処理 | 埋め込みを生成する | 現行ファイル内容とモデルから embedding を生成 | C0→C3 |
| CR-011 | RBD | 現行埋め込み | （属性保持） | embedding ベクトル・次元数・モデル版数を保持 | C3→Ecurrent |
| CR-012 | RBD/SEQD | embeddingsストア境界 | 現行保存行を読み込む / 保存行の存在を確認する | 現行保存行（デフォルトベースライン）供給 | C4→Bembed |
| CR-013 | RBD/SEQD | スナップショット境界 | ラベルで照合する / スナップショット識別子で照合する / 該当行の存在を確認する | スナップショット行（ラベル/識別子照合）供給 | C4→Bsnap |
| CR-014 | RBD/SEQD | ベースライン解決処理 | ベースラインを解決する / プレフィクスを検証する / ラベルを解決する / 最新行を選択する | 3 形式解決・プレフィクス検証・label 複数一致時の最新選択。プレフィクス欠如・明示 label 失敗は exit 1、不在は exit 0 | C0→C4 |
| CR-015 | RBD | ベースライン埋め込み | （属性保持） | embedding ベクトル・次元数・モデル版数・供給元情報を保持 | C4→Ebaseline |
| CR-016 | RBD/SEQD | 整合性検査処理 | 整合性を検査する / 次元数を照合する / モデル版数を照合する | 次元数・model_version 照合。不一致は exit 1 | C0→C5 |
| CR-017 | RBD/SEQD | ドリフト算出処理 | ドリフトを算出する / コサイン類似度を計算する / スコアの有限性を確認する | drift = 1.0 − cosine 類似度の算出。非有限スコアは exit 1 | C0→C6 |
| CR-018 | RBD | 対比結果 | （属性保持） | drift 値・ベースライン利用可否・供給元を保持 | C6→Eresult |
| CR-019 | RBD/SEQD | 対比結果集約処理 | 結果を集約する | 対比結果を集約し対比報告を構成して出力窓口へ渡す | C0→C7 |
| CR-020 | RBD/SEQD | 対比結果出力窓口 | 対比報告を出力する / ログを出力する | drift 値（stdout）とログ・診断（stderr）を区別して出力。--json 形式も対応 | C7→B2 |

---

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・アクターから drift 実行要求を受け取る責務が CR-001「対比要求を受け付ける」として保存 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Control・協調統括責務が CR-002「対比を統括する」として保存。SEQD §1 で各処理への順次依頼を確認 | GREEN |
| AR-003 | CR-003 | preserved | 同一 Boundary・解決順序に従ったモデル読込責務が CR-003「モデルを解決順序に従って読み込む / モデルの存在を確認する」として保存 | GREEN |
| AR-004 | CR-004 | preserved | 同一 Control・モデル解決処理の責務（解決順序ロジック・旧名案内・exit 1）が CR-004「モデルを解決する / 旧名解決を案内する」として保存 | GREEN |
| AR-005 | CR-005 | preserved | 同一 Entity・解決済み値の保持責務が CR-005「解決済みモデルを参照する」（属性: 解決経路・モデル所在・モデル版数）として保存 | GREEN |
| AR-006 | CR-006 | preserved | 同一 Boundary・登録確認とファイルパス供給責務が CR-006「成果物の登録を確認する / ファイルパスを読み込む」として保存 | GREEN |
| AR-007 | CR-007 | preserved | 同一 Control・成果物解決（未登録 exit 1・欠落 exit 1）責務が CR-007「成果物を解決する」として保存 | GREEN |
| AR-008 | CR-008 | preserved | 同一 Boundary・現行ファイル内容供給責務が CR-008「現行ファイル内容を読み込む / ファイルの存在を確認する」として保存。代替 3b の存在確認も同一クラス内操作として保存 | GREEN |
| AR-009 | CR-009 | preserved | 同一 Entity・成果物識別子とファイルパスの保持責務が CR-009（属性: 成果物識別子・現行ファイルパス）として保存。RBD §1 に属性明示 | GREEN |
| AR-010 | CR-010 | preserved | 同一 Control・embedding 生成責務が CR-010「埋め込みを生成する」として保存。クラス名は「埋め込み生成処理」（embedding→埋め込みの表記揺れ、意味同一。RBD §4 mapping で明示確認） | GREEN |
| AR-011 | CR-011 | preserved | 同一 Entity・embedding ベクトル・次元数・モデル版数の保持責務が CR-011（属性: 埋め込みベクトル・次元数・モデル版数）として保存 | GREEN |
| AR-012 | CR-012 | preserved | 同一 Boundary・現行保存行供給責務が CR-012「現行保存行を読み込む / 保存行の存在を確認する」として保存 | GREEN |
| AR-013 | CR-013 | preserved | 同一 Boundary・スナップショット行供給責務が CR-013「ラベルで照合する / スナップショット識別子で照合する / 該当行の存在を確認する」として保存 | GREEN |
| AR-014 | CR-014 | preserved | 同一 Control・ベースライン解決処理の責務が CR-014「ベースラインを解決する / プレフィクスを検証する / ラベルを解決する / 最新行を選択する」として保存。RBD §1 で操作が同一クラス内に識別済み（新規クラスの発見なし、RBD §4 明記）。操作の細分化は多形式解決ロジックの具体化であり UC に記述済みの 3 形式・決定論的選択・exit 規則から逸脱しない | GREEN |
| AR-015 | CR-015 | preserved | 同一 Entity・ベースライン値（embedding ベクトル・次元数・モデル版数・供給元情報）の保持責務が CR-015（属性: 埋め込みベクトル・次元数・モデル版数・供給元情報、<<persistent>>）として保存 | GREEN |
| AR-016 | CR-016 | preserved | 同一 Control・次元数一致・model_version 完全一致検査（SCORE-INV-2 / exit 1）責務が CR-016「整合性を検査する / 次元数を照合する / モデル版数を照合する」として保存 | GREEN |
| AR-017 | CR-017 | preserved | 同一 Control・drift 算出（非有限スコア exit 1）責務が CR-017「ドリフトを算出する / コサイン類似度を計算する / スコアの有限性を確認する」として保存。クラス名「ドリフト算出処理」（drift→ドリフトの表記揺れ、意味同一。RBD §4 mapping で明示確認） | GREEN |
| AR-018 | CR-018 | preserved | 同一 Entity・drift 値・baseline_available・baseline_source の保持責務が CR-018（属性: ドリフト値・ベースライン利用可否・ベースライン供給元）として保存 | GREEN |
| AR-019 | CR-019 | preserved | 同一 Control・集約と出力窓口への受け渡し責務が CR-019「結果を集約する」として保存 | GREEN |
| AR-020 | CR-020 | preserved | 同一 Boundary・stdout/stderr 分離出力（text / --json 両対応）責務が CR-020「対比報告を出力する / ログを出力する」として保存。--json 形式切替は SEQD §3 例外フローで出力形式種別引数として保存 | GREEN |

20 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-013 §4 が新規クラスなしを確認済み。

---

## 4. Role Fitness Check（§5.2）

### Boundary

- **Finding**: 各 Boundary クラス（対比コマンド受付窓口・モデル境界・グラフ定義境界・現行ファイル境界・embeddingsストア境界・スナップショット境界・対比結果出力窓口）は境界操作（読込・供給・存在確認・出力）のみ保持。モデル境界が解決順序ロジックを担う逸脱なし（解決順序ロジックはモデル解決処理 Control が担う）。対比結果出力窓口が --json 形式切替を内部で処理することは出力形式の変換であり Boundary 操作の範疇（制御ロジックの越権なし）。Boundary overreach なし。
- **Severity**: なし / 原因の所在: — / Required action: なし

### Control

- **Finding**: 各 Control（対比統括処理・モデル解決処理・成果物解決処理・埋め込み生成処理・ベースライン解決処理・整合性検査処理・ドリフト算出処理・対比結果集約処理）は担当ロジックに限定。対比統括処理は協調のみで drift 算出を自ら担わない（Control leakage なし）。ベースライン解決処理は 3 形式解決ロジックを一手に担うが UC に明示された複雑な分岐であり Service blob 化ではない（責務名と処理が一致、万能化なし）。ドリフト算出処理内の `コサイン類似度を計算する` / `スコアの有限性を確認する` は同一 Control 内の操作細分化であり、他クラスへの越権なし（self-call として SEQD に明示）。
- **Severity**: なし / 原因の所在: — / Required action: なし

### Entity

- **Finding**: 各 Entity（モデル解決設定・成果物参照・現行埋め込み・ベースライン埋め込み・対比結果）は自身のデータ保持のみ。`モデル解決設定` が `解決済みモデルを参照する` 操作を持つが、これは自身の保持データへのアクセサであり Entity anemia / overreach なし。`成果物参照` / `現行埋め込み` / `対比結果` は属性保持のみ（SEQD で登録渡しのみ）。`ベースライン埋め込み` は <<persistent>> を示すが保持データへのアクセサを持たず Entity anemia の疑いあり → ただし具体クラスとしての操作はドリフト算出処理 Control から参照される依存関係として表現されており、SEQD §1 / §2 の各代替フローで参照が実現されている。概念型属性（埋め込みベクトル・次元数・モデル版数・供給元情報）は DD で操作化される前提。アニミア状態は具体層の成果物として許容範囲内。
- **Severity**: なし / 原因の所在: — / Required action: なし

---

## 5. Sequential Execution Check（§5.3）

### 基本フロー（UC-013 Step 1–7）

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Step 1（drift 実行要求） | Actor→対比コマンド受付窓口→対比統括処理 | 対比要求を受け付ける→対比を統括する | Yes | |
| Step 2（ONNX モデル解決） | 対比統括処理→モデル解決処理→モデル境界→モデル解決設定 | モデルを解決する→モデルを解決順序に従って読み込む→解決済みモデルを参照する | Yes | |
| Step 3（成果物解決・embedding 生成） | 成果物解決処理→グラフ定義境界→成果物参照 / embedding生成処理→現行ファイル境界→モデル解決設定→現行embedding | 成果物を解決する→登録確認→ファイルパス取得→存在確認→成果物参照 / 埋め込みを生成する→ファイル内容読込→モデル参照→現行埋め込み | Yes | SEQD §1 で C2 が Bgraph・Bfile を経由し Eartifact を確定、C3 が Bfile・Emsetting から Ecurrent を生成 |
| Step 4（ベースライン選択） | ベースライン解決処理→embeddingsストア境界→ベースラインembedding | ベースラインを解決する→現行保存行を読み込む→ベースライン埋め込み | Yes | |
| Step 5（次元数・model_version 検査） | 整合性検査処理→現行embedding / ベースラインembedding | 整合性を検査する→次元数を照合する→モデル版数を照合する | Yes | |
| Step 6（drift 算出） | drift算出処理→現行embedding / ベースラインembedding→対比結果 | ドリフトを算出する→コサイン類似度を計算する→スコアの有限性を確認する→対比結果 | Yes | |
| Step 7（出力・exit 0） | 対比結果集約処理→対比結果→対比結果出力窓口 | 結果を集約する→対比報告を出力する→stdout + exit 0 | Yes | |

### 代替フロー

| 代替 | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 1a（プレフィクス欠如 exit 1） | ベースライン解決処理→形式不正→exit 1 | プレフィクスを検証する→形式不正として解決失敗→ログを出力する→exit 1 | Yes | UC: exit 1（exit 2 ではない）が SEQD §2 代替 1a で終了コード 1 として保存 |
| 2a（モデル解決失敗 exit 1） | モデル解決処理→全経路失敗→exit 1 | モデルの存在を確認する→解決失敗→stderr 試行内容診断 + exit 1 | Yes | |
| 2b（旧名 TE_MODELS_DIR → stderr 案内し続行） | モデル解決処理→旧名経路で確定→stderr 案内→続行 | モデルを解決順序に従って読み込む（旧名経路）→ログを出力する（Info stderr）→解決済みモデルを参照する（続行） | Yes | UC: 両変数同時設定時は LGX_MODELS_DIR 優先。SEQD の処理続行は保存されている |
| 3a（未登録 exit 1） | 成果物解決処理→未登録→exit 1 | 成果物の登録を確認する→未登録→ERROR stderr + exit 1 | Yes | |
| 3b（現行ファイル欠落 exit 1） | 成果物解決処理→ファイル欠落→exit 1（4a exit 0 との非対称は意図的） | 登録確認→ファイルパス取得→ファイルの存在を確認する→欠落→exit 1 | Yes | UC 「壊れた状態を隠さない」という意図がSEQD「壊れた状態」注記で保存 |
| 4a（ベースライン不在 exit 0 + INFO + baseline_available: false） | ベースライン解決処理→不在（正常ライフサイクル）→exit 0 | 保存行の存在を確認する→不在→baseline_available: false + INFO stderr + exit 0 | Yes | exit 0 が維持されている（exit 1 への誤変換なし）。--json 時の `{"drift": null}` は出力窓口の形式切替として保存 |
| 4b（同一 label 複数 → taken_at 最新 1 件） | ベースライン解決処理→同一 label 複数→最新 1 件確定 | プレフィクスを検証する→ラベルを解決する→ラベルで照合する→保存済み埋め込み行のコレクション→最新行を選択する→ベースライン埋め込み（最新 1 件） | Yes | UC「UC-LGX-012 の delete と同一規則」。SEQD で決定論的選択が保存 |
| 5a（次元数不一致 exit 1） | 整合性検査処理→次元不一致→exit 1 | 次元数を照合する（両側）→次元数不一致として検査失敗→exit 1 | Yes | |
| 5b（model_version 不一致 exit 1、SCORE-INV-2） | 整合性検査処理→次元一致・model_version 不一致→exit 1 | 次元数一致確認→モデル版数を照合する（両側）→モデル版数不一致として検査失敗→exit 1 | Yes | SEQD §2 代替 5b で次元数チェック後に model_version チェックを行う二段構成が保存。UC の非対称性（5a: 計算不能 / 5b: SCORE-INV-2）の根拠が維持されている |
| 6a（非有限スコア exit 1） | drift算出処理→非有限スコア→exit 1 | コサイン類似度を計算する→スコアの有限性を確認する→非有限スコアとして算出失敗→exit 1 | Yes | |
| snapshot:label:<L> 明示形式失敗（exit 1） | ベースライン解決処理→明示 label 解決失敗→exit 1 | ラベルで照合する→該当なし→スナップショット識別子で照合する→該当なし→明示ラベル解決失敗として通知→exit 1 | Yes | SEQD §2 代替「明示ラベル形式指定で解決失敗」でラベル照合→識別子照合→失敗の二段試行が保存 |

### 例外フロー

| 例外 | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| --json 出力形式切替 | 対比結果出力窓口がアクターへ返す形式の切替（フロー分岐なし） | 対比報告を出力する（出力形式種別引数で内部変換）→フロー分岐なし | Yes | SEQA §4・SEQD §3 の両方で「別フローを要さない」と明示。出力窓口の Boundary 責務内で処理 |

全 UC フローが SEQA / SEQD 上で責務の不整合なく実行可能。

---

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。新規クラスの発見なし、RBD §4 確認済み）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None（クラス名の表記揺れ「embedding生成処理」→「埋め込み生成処理」、「drift算出処理」→「ドリフト算出処理」は RBD §4 の mapping で明示されており意味変質ではない）
- **Ambiguous Mappings**: None

---

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---:|---:|
| Total abstract responsibilities | 20 |
| Preserved | 20 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 20 |
| Invention rate（監視用） | 0% |

---

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（解消済）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

---

## 9. Required Changes

- なし（保存失敗なし）

---

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC 基本・代替・例外フロー実行可能）。クラス名の表記揺れ（embedding→埋め込み / drift→ドリフト）は RBD §4 の 1:1 mapping で明示されており意味変質ではない。ベースライン解決処理の操作細分化（プレフィクスを検証する / ラベルを解決する / 最新行を選択する）は UC に記述済みの 3 形式・決定論的選択・exit 規則の具体化であり justified（新規クラスの発見なし）。抽象責務集合（UC 錨着）が具体責務集合へ 1:1 で保存されている。

<!-- VERDICT:APPROVE -->
