Document ID: TS-LGX-012

# TS-LGX-012: ベースライン凍結管理（snapshot create / list / delete）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-012 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-012
**継承 TP**: TP-LGX-010（TP[SPEC] embedding 運用・監査、71 観点）, TP-LGX-022（TP[UC] UC-012 ベースライン凍結管理、26 観点）

> DD-LGX-012 §8 の「対応 TP」列は全行 TP-LGX-009 を引くが、TP-LGX-009 は MCP サーバ層（`ts-mcp`）専用の観点であり、snapshot 4 コマンドは MCP 非公開（TP-LGX-010 §2.9 F1 / MCP-INV-1）。本 TS の実質的な継承元は TP-LGX-010（SPEC）+ TP-LGX-022（UC）である（DD §8 の TP-009 表記は記載誤りとして §3 末尾で申し送る）。

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-012 の関数 / 型:

- DD-LGX-012 §3 `legixy_embed::snapshot::create(store: &EmbeddingStore, snapshot_id: &str, label: Option<&str>) -> Result<SnapshotCreateResult, SnapshotError>`
- DD-LGX-012 §3 `legixy_embed::snapshot::list(store: &EmbeddingStore) -> Result<Vec<SnapshotMeta>, SnapshotError>`
- DD-LGX-012 §3 `legixy_embed::snapshot::delete(store: &EmbeddingStore, snapshot_id: &str) -> Result<SnapshotDeleteResult, SnapshotError>`
- DD-LGX-012 §3 `legixy_embed::snapshot::resolve_label(store: &EmbeddingStore, label: &str) -> Result<LabelResolveResult, SnapshotError>`
- DD-LGX-012 §3 `legixy_embed::snapshot::generate_snapshot_id() -> String`
- DD-LGX-012 §3.1 `legixy_db::open_engine_db(project_root: &Path, access: DbAccess) -> Result<Connection, DbError>`（snapshot 系の DB パス解決と DbAccess::Read/Write 分岐）
- DD-LGX-012 §2 型: `SnapshotMeta` / `SnapshotCreateResult` / `SnapshotDeleteResult` / `LabelResolveResult`{Resolved,NotFound} / `SnapshotError`{Db,LabelNotFound,TransactionFailed} / `SnapshotAction`{Create,List,Delete}
- DD-LGX-012 §3.2 `legixy-cli` の snapshot コマンドハンドラ（text / `--json` 出力契約・exit code 変換）

委譲（本 TS 対象外）:
- 並行アクセス整合性（同時 create×2 の atomicity・create/delete interleaving・WAL ロック競合・busy_timeout）→ TP-LGX-010 §2.4 C1〜C4 は NFR-LGX-001 SEC.02 / REL.07 へ委譲（DD §7）。本 TS は単一スレッド・単一プロセスのケースのみ。
- 永続化障害回復（電源断 / ディスクフル / 権限エラーからの回復保証）→ NFR-LGX-001 REL.01/REL.06（TP-LGX-010 §2.2 E11 / §2.5 P4）。本 TS はトランザクションのロールバック（アトミック性）のみ検証。
- WAL モード性能予算（PERF.07）→ NFR-LGX-001。
- drift / report / calibrate コマンドの観点（B4〜B9, E2〜E6, E9, V4〜V7, I1, D1/D4/D5/D6/D7/D8 等）→ snapshot 範囲外。`resolve_label` を消費する drift の `--against snapshot:` 側挙動は TS-LGX-013（UC-013 drift）へ委譲。
- baseline 不変性 / TOCTOU / model_version 切替の意味的妥当性（S5/S6/V6/DF2 のうち drift 照合側）→ UC-LGX-013（drift）所有。本 TS は snapshot が content_hash / model_version 列を複製する事実（P3）のみ検証。
- `EmbeddingRow` の定義・`load_all` 等 EmbeddingStore 本体 API → DD-LGX-007 所有（本 DD は委譲）。本 TS は `&EmbeddingStore` を所与として snapshot API のみ検証。

本 TS は「snapshot create / list / delete / resolve_label が SPEC-010 REQ.02/REQ.06/REQ.07 と UC-012 フローを DD-012 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: 空ストア create（複製 0 件）→ node_count=0・非永続・Ok

- **観点出典**: TP-LGX-010 §2.1 B1（空ストア非永続+WARNING+exit 0）, TP-LGX-022 §2.2 AF2（2a 境界）
- **分類**: Unit
- **前提**: `EmbeddingStore` の `embeddings` テーブルが 0 行。`snapshot_id = "snap-..."`（呼出側生成）、`label = None`
- **入力**:
  ```
  create(&empty_store, "snap-018f-deadbeef", None)
  ```
- **期待**:
  ```
  Ok(SnapshotCreateResult { snapshot_id: "snap-018f-deadbeef", label: None, node_count: 0 })
  かつ embedding_snapshots テーブルに当該 snapshot_id の行が 1 件も書かれていない（非永続、DB 書込みなし）
  ```
- **境界条件**: node_count=0 = DB への永続なし（SPEC REQ.02 2a）。エラーではなく Ok。後続 list に現れない（AF2 二次検出と整合）

### ケース 2: create 正常系（node_count > 0）→ embedding_snapshots へ行複製

- **観点出典**: TP-LGX-010 §2.3 S1/S3, §2.5 P1/P3, TP-LGX-022 §2.1 BF1
- **分類**: Unit
- **前提**: `embeddings` テーブルに 3 行（node_id, embedding, embedding_dim, model_version, content_hash を持つ）。`snapshot_id = "snap-018f-a1b2c3d4"`, `label = Some("v0.3.0")`
- **入力**:
  ```
  create(&store_with_3_rows, "snap-018f-a1b2c3d4", Some("v0.3.0"))
  ```
- **期待**:
  ```
  Ok(SnapshotCreateResult { snapshot_id: "snap-018f-a1b2c3d4", label: Some("v0.3.0"), node_count: 3 })
  かつ embedding_snapshots に snapshot_id="snap-018f-a1b2c3d4" の行が 3 件（各 node_id ごと）。
  各行の content_hash / model_version / embedding / embedding_dim は元 embeddings 行と一致（SCORE-INV-1 複製）。
  taken_at は datetime('now') の UTC 秒精度文字列。
  ```
- **境界条件**: 複製は単一トランザクション。embeddings 本体行は不変（read-only、P1/P2）

### ケース 3: list 0 件 → `Vec::new()`

- **観点出典**: TP-LGX-010 §2.1 B2, TP-LGX-022 §2.2 AF3（4a 境界）
- **分類**: Unit
- **前提**: `embedding_snapshots` テーブルが 0 行（または DB 自体が空ストア相当）
- **入力**:
  ```
  list(&empty_snapshot_store)
  ```
- **期待**:
  ```
  Ok(Vec::<SnapshotMeta>::new())   // 長さ 0
  ```
- **境界条件**: 0 件 = 正常（エラーではない）。コマンド層が text 案内 / json `[]` を出力し exit 0

### ケース 4: list 複数件 → taken_at 降順 + snapshot_id DESC タイブレーク（property）

- **観点出典**: TP-LGX-010 §2.10 D3（list 安定降順 + 同規則タイブレーク）, TP-LGX-022 §2.6 R5
- **分類**: Property-based（proptest）
- **生成器**: 任意個（0..=20）の snapshot を `embedding_snapshots` に投入。各 snapshot は `(snapshot_id, label, taken_at, node 行数)` を持つ。
  - **生成器制約（必須、偽反例 shrink 防止）**: **同一 `snapshot_id` 内では `label` を一定**とする（1 つの snapshot は単一 label を持つ。DD §11 PRIMARY KEY `(snapshot_id, node_id)` と運用慣行に整合）。生成器は snapshot 単位で `(snapshot_id, label)` を 1 組決め、その snapshot に属する全行へ同一 label を割り当てる。`taken_at` は秒精度文字列で、複数 snapshot が同一秒を持つよう意図的に重複を含めて生成する（タイブレーク発火）。
- **不変条件**:
  ```
  list(&store) が返す Vec<SnapshotMeta> は
    ① MAX(taken_at) の降順
    ② 同一 taken_at 内では snapshot_id の降順
  で安定整列している（DD §11 SQL `ORDER BY MAX(taken_at) DESC, snapshot_id DESC` と一致）。
  同一入力に対し常に同一順序（決定論）。
  各要素 node_count == GROUP BY (snapshot_id, label) 単位の行数（DD §11 `GROUP BY snapshot_id, label, COUNT(*)`）。
  上記生成器制約（snapshot_id 内 label 一定）下では GROUP BY (snapshot_id, label) 群 ≡ GROUP BY snapshot_id 群となり、
  1 snapshot_id = 1 行に縮約される（snapshot_id ごとに複数 label が混在しないため、偽の分割行・偽反例 shrink が発生しない）。
  ```
- **反例ハンドリング**: shrink して最小の順序不一致例（同一秒タイブレーク違反等）を記録。生成器制約により「同一 snapshot_id に異 label を混入させたことによる GROUP BY 分割」起因の偽反例は構造的に排除される。

### ケース 5: list 同一秒タイブレークの具体例（決定論的順序）

- **観点出典**: TP-LGX-010 §2.10 D3, DD §11 SUPP-010 S-5
- **分類**: Unit
- **前提**: 同一 taken_at `"2026-06-13 10:00:00"` を持つ 2 スナップショット（`snap-...-aaaa` と `snap-...-bbbb`）+ より古い `"2026-06-13 09:00:00"` の `snap-...-cccc`
- **入力**:
  ```
  list(&store)
  ```
- **期待**:
  ```
  Vec の順序 = [ snap-...-bbbb, snap-...-aaaa, snap-...-cccc ]
  （同一秒は snapshot_id DESC → bbbb > aaaa、その後に古い cccc）
  ```
- **境界条件**: 秒精度衝突時の決定論。`list` と `resolve_label` が同一規則（taken_at DESC, snapshot_id DESC）

### ケース 6: list の label 表現（None / Some）

- **観点出典**: TP-LGX-010 §2.7 I3（label 非一意）, TP-LGX-022 §2.1 BF4, DD §3.2（text の `-` 表記）
- **分類**: Unit
- **前提**: label 付き snapshot（`Some("rc1")`）と label 無し snapshot（`--label` 未指定）が各 1 件
- **入力**:
  ```
  list(&store)
  ```
- **期待**:
  ```
  label 付き要素は SnapshotMeta.label == Some("rc1")
  label 無し要素は SnapshotMeta.label == None
  （コマンド層 text モードは None を "-" 表記。json は label: null）
  ```
- **境界条件**: `Option<String>` の None/Some 双方を SnapshotMeta が保持

### ケース 7: delete 成功（deleted_rows > 0）→ DB から該当行除去

- **観点出典**: TP-LGX-010 §2.3 S1, TP-LGX-022 §2.1 BF1/2.2 AF4, §2.4 AT3（不可逆）
- **分類**: Unit
- **前提**: snapshot_id `"snap-018f-a1b2c3d4"` が 3 行存在
- **入力**:
  ```
  delete(&store, "snap-018f-a1b2c3d4")
  ```
- **期待**:
  ```
  Ok(SnapshotDeleteResult { snapshot_id: "snap-018f-a1b2c3d4", deleted_rows: 3 })
  かつ delete 後 embedding_snapshots に当該 snapshot_id の行が 0 件。
  他 snapshot_id の行 / embeddings 本体行は不変（read-only on embeddings）。
  ```
- **境界条件**: delete は単一トランザクション。embeddings テーブルは変更しない

### ケース 8: delete 該当 0 件（6b）→ Ok(deleted_rows=0)・エラー非発生

- **観点出典**: TP-LGX-010 §2.1 B3, TP-LGX-022 §2.2 AF4/2.3 EF4
- **分類**: Unit
- **前提**: 指定 snapshot_id `"snap-does-not-exist"` が DB に存在しない
- **入力**:
  ```
  delete(&store, "snap-does-not-exist")
  ```
- **期待**:
  ```
  Ok(SnapshotDeleteResult { snapshot_id: "snap-does-not-exist", deleted_rows: 0 })
  // SnapshotError ではない。コマンド層が WARNING を stderr に出力し exit 0。json は { "snapshot_id", "deleted_rows": 0 }（WARNING なし）
  ```
- **境界条件**: deleted_rows=0 は Ok（exit 0 パス）。DB 不在 ID の delete は失敗ではない（SPEC REQ.02 6b）

### ケース 9: resolve_label 同一 label 複数存在（6a）→ taken_at DESC + snapshot_id DESC で 1 件

- **観点出典**: TP-LGX-010 §2.7 I3, §2.10 D3, TP-LGX-022 §2.6 R2, DD §11 resolve SQL
- **分類**: Unit
- **前提**: label `"release"` を持つ snapshot が 3 件: `(taken_at="2026-06-13 12:00:00", id="snap-...-zz")`, `(taken_at="2026-06-13 12:00:00", id="snap-...-yy")`, `(taken_at="2026-06-13 08:00:00", id="snap-...-old")`
- **入力**:
  ```
  resolve_label(&store, "release")
  ```
- **期待**:
  ```
  Ok(LabelResolveResult::Resolved("snap-...-zz"))
  // taken_at 最新の 2 件のうち snapshot_id DESC で zz > yy → zz を 1 件決定論的に解決
  ```
- **境界条件**: label 非一意でも 1 件に決定論的解決（SPEC REQ.02 6a / REQ.06）。delete の `label:<L>` 解決と drift `--against snapshot:<L>` が同一規則（R2）

### ケース 10: resolve_label label 不在（6c）→ NotFound

- **観点出典**: TP-LGX-010 §2.2 E7, TP-LGX-022 §2.3 EF1（GENUINE: 不在 label = exit 1）
- **分類**: Unit
- **前提**: label `"nonexistent"` を持つ snapshot が 0 件
- **入力**:
  ```
  resolve_label(&store, "nonexistent")
  ```
- **期待**:
  ```
  Ok(LabelResolveResult::NotFound)
  // resolve_label 自体は Err ではない（DB エラーのみ Err）。
  // delete コマンド層が NotFound を受けて SnapshotError::LabelNotFound へ昇格 → ERROR(stderr) + exit 1
  ```
- **境界条件**: label 不在（delete → exit 1）と snapshot_id 不在（delete → exit 0、ケース 8）の分岐。LabelResolveResult::NotFound は delete 経路で exit 1、drift 曖昧形式経路では exit 0（DD §2.2 注記、drift 側は TS-013）

### ケース 11: generate_snapshot_id の形式（`snap-` プレフィクス・13+8 桁 16 進）

- **観点出典**: TP-LGX-010 §2.6 V3（snap- 凍結・不透明トークン）, TP-LGX-022 §2.6 R1, DD §3（生成式）
- **分類**: Unit
- **前提**: なし
- **入力**:
  ```
  generate_snapshot_id()
  ```
- **期待**:
  ```
  戻り値が正規表現 ^snap-[0-9a-f]{13}-[0-9a-f]{8}$ に一致
  （snap- プレフィクス + epoch_ms 13 桁 16 進 + "-" + 8 桁 16 進乱数）
  ```
- **境界条件**: `snap-` プレフィクスは SPEC 凍結（外部契約）、内部 13+8 桁形式は DD 凍結。一意性保証はなし（衝突は create Tx 失敗で検出）

### ケース 12: generate_snapshot_id の非衝突傾向（property）

- **観点出典**: TP-LGX-010 §2.10 D2（create 非決定性）, DD §7（衝突は TransactionFailed で検出）
- **分類**: Property-based（proptest）
- **生成器**: `generate_snapshot_id()` を N 回（例 1000 回）連続呼出した結果集合
- **不変条件**:
  ```
  全戻り値が ^snap-[0-9a-f]{13}-[0-9a-f]{8}$ に一致し、
  異なる呼出は（同一プロセス内で）原則異なる文字列を返す傾向（epoch_ms + 乱数）。
  ※ 一意性は保証契約ではない。同一ミリ秒衝突は許容され create 側 PRIMARY KEY 制約で検出される（ケース 14 へ）
  ```
- **反例ハンドリング**: 衝突が出ても fail ではなく、衝突が create トランザクションで検出されることをケース 14 で担保する旨を記録

### ケース 13: create → list → delete の E2E ライフサイクル

- **観点出典**: TP-LGX-010 §2.3 S1, TP-LGX-022 §2.1 BF1/BF3
- **分類**: Integration
- **前提**: embeddings 2 行を持つ実 DB ファイル（一時ディレクトリの `.legixy/engine.db`）
- **入力**:
  ```
  let id = generate_snapshot_id();
  create(&store, &id, Some("e2e"));   // → node_count: 2
  list(&store);                        // → 当該 id を 1 件含む
  delete(&store, &id);                 // → deleted_rows: 2
  list(&store);                        // → 当該 id を含まない
  ```
- **期待**:
  ```
  create: Ok(node_count == 2)
  1 回目 list: 当該 id を含む長さ 1（label Some("e2e"), node_count 2）
  delete: Ok(deleted_rows == 2)
  2 回目 list: 当該 id を含まない（長さ 0）
  各段で embeddings 本体行は 2 のまま不変
  ```
- **境界条件**: 凍結→一覧→削除の事後条件連鎖（BF1）。delete 後の不在が list で観察可能（AT3 不可逆）

### ケース 14: create トランザクション失敗 → 部分行残存なし（ロールバック）

- **観点出典**: TP-LGX-010 §2.2 E11/2.5 P1, TP-LGX-022 §2.3 EF2/EF3, DD §6（tx.commit 失敗 → TransactionFailed）
- **分類**: Integration
- **前提**: 既存 snapshot_id と同一 ID で再 create を試行し `PRIMARY KEY(snapshot_id, node_id)` 違反を誘発（または書込み途中で commit が失敗する状況をモック）。元 embeddings は 3 行
- **入力**:
  ```
  create(&store, "snap-collide-existing", None)   // 既に同一 id の行が存在
  ```
- **期待**:
  ```
  Err(SnapshotError::TransactionFailed(_))   // または Db(_)（PRIMARY KEY 違反由来）
  かつ ロールバックにより当該 create で部分的に書き込まれた行は 0 件（SQLite が ROLLBACK 自動保証）。
  既存行・embeddings 本体行は不変。コマンド層は exit 1。
  ```
- **境界条件**: トランザクション失敗時のアトミック性（all-or-nothing）。部分スナップショット非残存（EF3 atomicity）

### ケース 15: engine.db 不在で list（DbAccess::Read）→ 空配列・DB 非作成・exit 0

- **観点出典**: TP-LGX-010 §2.3 S2（DB 不在 ≡ 空ストア、非作成）, TP-LGX-022 §2.6 R4, DD §3.1（FB-INV-4 / DbAccess::Read）
- **分類**: Integration
- **前提**: `<project_root>/.legixy/engine.db` も `<project_root>/.trace-engine/engine.db` も不在
- **入力**:
  ```
  open_engine_db(project_root, DbAccess::Read) 経由で list を実行
  ```
- **期待**:
  ```
  list → Ok(Vec::new())   // 空配列、exit 0
  かつ project_root 配下に engine.db / .legixy/ ディレクトリが新規作成されていない（読取系は非作成、FB-INV-4）
  ```
- **境界条件**: 読取系（list）は DB 不在で空ストア相当・非破壊。create の DB 初期化非対称（ケース 16）と対比

### ケース 16: engine.db 不在で create（DbAccess::Write）→ `.legixy/` 作成 + スキーマ初期化 + exit 0

- **観点出典**: TP-LGX-010 §2.3 S3（create のみ DB 初期化）, TP-LGX-022 §2.6 R4, DD §3.1（DbAccess::Write）
- **分類**: Integration
- **前提**: `<project_root>/.legixy/engine.db` 不在。ただし embeddings は空（DB 新規作成直後）
- **入力**:
  ```
  open_engine_db(project_root, DbAccess::Write) 経由で create(&store, &id, None) を実行
  ```
- **期待**:
  ```
  <project_root>/.legixy/engine.db が新規作成され、embedding_snapshots テーブルがスキーマ初期化される（§11 CREATE TABLE）。
  embeddings が空のため create は node_count: 0 を返し snapshot 行は永続しない（ケース 1 と整合）。exit 0。
  ```
- **境界条件**: 書込み系（create）のみ正準パス `.legixy/engine.db` を新規作成 + スキーマ初期化（SPEC REQ.07）。read-only 系との非対称

### ケース 17: snapshot サブコマンド省略 → exit 2（clap 構文層）

- **観点出典**: TP-LGX-010 §2.2 E8, TP-LGX-022 §2.2 AF1/AF5（1a 排他）, DD §2.2（SnapshotAction 未指定）
- **分類**: Contract（CLI）
- **前提**: `snapshot` をサブコマンド（create/list/delete）なしで起動
- **入力**:
  ```
  legixy snapshot          # SnapshotAction 未指定
  ```
- **期待**:
  ```
  exit code == 2（clap が自動付与。SnapshotError 経路ではない）
  ```
- **境界条件**: exit 2 は clap 構文層限定（サブコマンド省略・型不正・未知フラグ）。値の意味的不正（DB 失敗・label 不在）は exit 1

### ケース 18: exit code 契約 0/1/2（snapshot 全分岐）

- **観点出典**: TP-LGX-010 §2.2 E1（exit 3 分類）, §2.9 F2, TP-LGX-022 §2.6 R3, DD §2.3 終了コード規約
- **分類**: Contract（CLI）
- **前提**: (a) 正常 create / list / delete 成功, (b) 空ストア create, (c) list 0 件, (d) delete 該当 0 件（snapshot_id 不在）, (e) delete `label:<L>` 解決失敗, (f) サブコマンド省略
- **入力**:
  ```
  各シナリオを CLI ディスパッチで実行
  ```
- **期待**:
  ```
  (a) → 0    (b) 空ストア create → 0    (c) list 0 件 → 0    (d) delete 0 件 → 0
  (e) delete label 不在 → 1（SnapshotError::LabelNotFound）
  (f) サブコマンド省略 → 2（clap）
  ```
- **境界条件**: exit 0=結果が空/正常、exit 1=実行時失敗・label 不在、exit 2=構文層（LGX-COMPAT-001 凍結契約）

### ケース 19: `--json` 出力スキーマ（create / list / delete・空ストア warning フィールド含む）

- **観点出典**: TP-LGX-010 §2.9 F3（各 json キー明示）, TP-LGX-022 §2.5 DF1, DD §3.2 出力形式表
- **分類**: Contract（CLI）
- **前提**: 各操作を `--json` 付きで実行
- **入力 / 期待**:
  ```
  create 成功(node_count>0)      → stdout JSON: {"snapshot_id", "label", "node_count"}
  create 空ストア(node_count=0)  → stdout JSON: {"snapshot_id", "label", "node_count": 0,
                                      "warning": "ストアが空のため永続化されません。`embed --all` を先に実行してください"}
  list(>=1 件)                   → stdout JSON 配列: [{"snapshot_id","label","node_count","taken_at"}, ...]（pretty print）
  list(0 件)                     → stdout JSON: []
  delete 成功(deleted_rows>0)    → stdout JSON: {"snapshot_id", "deleted_rows": n}
  delete 該当 0 件               → stdout JSON: {"snapshot_id", "deleted_rows": 0}（WARNING なし）
  delete label 解決失敗          → exit 1、stdout に JSON 出力なし
  ```
- **境界条件**: `--json` 時も診断は stderr、結果 JSON は stdout（機械可読性維持、L2/DF1）。空ストア warning は JSON フィールドとして埋め込み（v3 英語文言廃止・日本語文言、SUPP-010 S-8）

### ケース 20: 出力先分離（結果=stdout / WARNING・ERROR・INFO=stderr）

- **観点出典**: TP-LGX-010 §2.8 L1（診断=stderr、結果=stdout）, TP-LGX-022 §2.3 EF4/2.5 DF1, NFR-LGX-001 OBS.02
- **分類**: Contract（CLI）
- **前提**: (a) 空ストア create（WARNING）, (b) delete 該当 0 件（WARNING）, (c) delete label 解決失敗（ERROR）
- **入力**:
  ```
  各シナリオを text モードで実行し stdout / stderr を分離取得
  ```
- **期待**:
  ```
  (a) stderr に WARNING + snapshot_id + nodes=0、stdout に結果（snapshot_id）
  (b) stderr に WARNING、stdout に結果なし（exit 0）
  (c) stderr に ERROR（"label '...' に該当するスナップショットがありません"）、stdout 空、exit 1
  全ケースで結果は stdout、INFO/WARNING/ERROR は stderr に分離
  ```
- **境界条件**: stdout/stderr チャネル分離（OBS.02【v3 差分】）。warning/error 文言は日本語 primary（OBS.04）

### ケース 21: snapshot 操作の embeddings 本体 read-only 不変（property）

- **観点出典**: TP-LGX-010 §2.5 P1/P2（embeddings 本体不変・読取系非破壊）, TP-LGX-022 §2.4 AT4/2.5 DF3, DD §3 read-only 不変条件
- **分類**: Property/Integration
- **前提**: 任意の embeddings 本体行集合（成功・失敗いずれの create/list/delete 操作後も）
- **入力**:
  ```
  操作前後の embeddings テーブル内容（全 EmbeddingRow）のハッシュ
  ```
- **不変条件**:
  ```
  create / list / delete / resolve_label のいかなる操作（成功・Err・空ストア・0 件いずれも）の前後で
  embeddings テーブルの行集合は不変（バイト一致）。
  graph.toml・成果物ファイルも不変。
  ```
- **反例ハンドリング**: shrink して embeddings を変更した最小操作列を記録
- **境界条件**: snapshot 系は `embeddings` を変更しない（DD §3 全関数の read-only 不変条件）。borrow（`&EmbeddingStore`）による保証

### ケース 22: SnapshotError の exit code 変換（コマンド層）

- **観点出典**: TP-LGX-010 §2.2 E1/E7, DD §6（公開境界のエラー伝播）
- **分類**: Unit
- **前提**: コマンド層が各 `SnapshotError` バリアントを受け取る
- **入力**:
  ```
  SnapshotError::Db(_)               // DB open / クエリ失敗
  SnapshotError::LabelNotFound{..}   // delete label 不在
  SnapshotError::TransactionFailed(_) // create/delete commit 失敗
  ```
- **期待**:
  ```
  3 バリアントいずれも exit 1 へ変換される（SnapshotError 全体 = exit 1）。
  clap 構文層誤りのみ exit 2（SnapshotError 経路外）。
  ```
- **境界条件**: SnapshotError → exit 1 の一意対応（DD §2.3）。Display 文言は日本語（OBS.04）

### ケース 23: delete by `label:<L>` 成功の主経路（resolve → delete → deleted_rows > 0）E2E

- **観点出典**: TP-LGX-010 §2.7 I2（delete target 二形態）, §2.3 S1, TP-LGX-022 §2.2 AF4（6a label 形態）, §2.1 BF1, DD §3.2（delete `label:LABEL` 形態のディスパッチ。v3 `snapshot.rs` run_delete の `strip_prefix("label:")` → resolve → delete 配線）
- **分類**: Integration（E2E、コマンド層 run_delete 配線を含む）
- **前提**: 同一 label `"release"` を持つ snapshot が 2 件存在する実 DB（一時ディレクトリの `.legixy/engine.db`）:
  - `(taken_at="2026-06-13 12:00:00", id="snap-018f-rel-newer", 3 行)`
  - `(taken_at="2026-06-13 08:00:00", id="snap-018f-rel-older", 2 行)`

  さらに無関係な別 label snapshot `(id="snap-018f-other", label="rc1", 4 行)` と embeddings 本体行を含む（混入検査用）。
- **入力**:
  ```
  // コマンド層 run_delete の "label:LABEL" 形態（snapshot_id 直指定ではない主経路）
  legixy snapshot delete "label:release"
  // 配線: target "label:release" → strip_prefix("label:") → "release"
  //       → resolve_label(&store, "release") → Resolved("snap-018f-rel-newer")
  //       → delete(&store, "snap-018f-rel-newer")
  ```
- **期待**:
  ```
  resolve_label("release") = Resolved("snap-018f-rel-newer")
    （taken_at 最新 = 12:00:00 の 1 件。ケース 9 の解決規則 taken_at DESC + snapshot_id DESC を再利用）
  delete("snap-018f-rel-newer") = Ok(SnapshotDeleteResult { snapshot_id: "snap-018f-rel-newer", deleted_rows: 3 })
    → deleted_rows == 3 > 0（成功主経路）
  delete 後の embedding_snapshots:
    - "snap-018f-rel-newer" の行 0 件（全行除去）
    - "snap-018f-rel-older"（同 label の旧 snapshot）の 2 行は不変（resolve が最新 1 件のみ対象、巻き添え削除なし）
    - "snap-018f-other"（別 label）の 4 行は不変
    - embeddings 本体テーブルは全行不変（read-only）
  コマンド層は exit 0、text モードで `snapshot 'snap-018f-rel-newer' を削除しました（3 行）` を stdout 出力。
  ```
- **境界条件**: これは delete 対称化（成功側、AF4 6a / I2 label 形態）の**唯一の主経路**。ケース 7（delete 成功）は snapshot_id 直指定のみ、ケース 9（resolve_label）は resolve 単体 Unit のみで、`label:<L>` 形態 delete の resolve → delete 合成主経路（v3 run_delete 配線）はこのケースが唯一束縛する。同一 label の旧 snapshot が巻き添え削除されない（resolve が 1 件決定 → その snapshot_id のみ delete）ことが I2/AF4 の事後条件。delete `label:<L>` 不在は exit 1（ケース 10/18/20、失敗側）と対称。

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース / 委譲先 |
|---|---|---|
| TP-010 §2.1 B1 空ストア create | 境界値 | ケース 1, 16, 19 |
| TP-010 §2.1 B2 list 0 件 | 境界値 | ケース 3, 19 |
| TP-010 §2.1 B3 delete 0 行 | 境界値 | ケース 8, 18, 19 |
| TP-010 §2.1 B4〜B9 calibrate/drift/report 値域・特殊値 | 境界値 | snapshot 範囲外 → TS-LGX-013（drift）/ calibrate・report TS へ委譲 |
| TP-010 §2.1 B10 snapshot 保持上限 | 境界値 | 無制限・手動管理（v3 正準、GAP-187 削除）。件数上限なし = list/delete で管理（ケース 13） |
| TP-010 §2.2 E1 exit 3 分類 | エラー | ケース 17, 18, 22 |
| TP-010 §2.2 E2〜E6/E9 drift エラー | エラー | snapshot 範囲外 → TS-LGX-013 へ委譲 |
| TP-010 §2.2 E7 label 解決失敗 exit 1 | エラー | ケース 10, 18, 20, 22 |
| TP-010 §2.2 E8 サブコマンド省略 exit 2 | エラー | ケース 17, 18 |
| TP-010 §2.2 E10 集約 Warning（report/calibrate）| エラー | snapshot 範囲外 → report/calibrate TS へ委譲 |
| TP-010 §2.2 E11 create Tx 中断回復 | エラー | ケース 14（ロールバック）+ NFR-LGX-001 REL.01/06 へ委譲（障害回復本体） |
| TP-010 §2.2 E12 部分成功 | エラー | snapshot は単一対象 = N/A（report/calibrate 側） |
| TP-010 §2.3 S1 create→list→delete ライフサイクル | 状態 | ケース 13 |
| TP-010 §2.3 S2 DB 不在 ≡ 空ストア・非作成 | 状態 | ケース 15 |
| TP-010 §2.3 S3 create のみ DB 初期化 | 状態 | ケース 16 |
| TP-010 §2.3 S4 誤削除復旧フロー | 状態 | ケース 13（delete 後 list 確認で観察可能）+ UC-012 アクター責務 |
| TP-010 §2.3 S5 baseline 不変性 | 状態 | ケース 2（content_hash/model_version 複製）+ UC-013（再 embed 後の凍結照合）へ委譲 |
| TP-010 §2.3 S6 TOCTOU | 状態 | NFR-LGX-001 SEC.02/REL.07 へ委譲（並行制御） |
| TP-010 §2.4 C1〜C4 並行性 | 並行 | NFR-LGX-001 SEC.02/REL.07 へ委譲（DD §7） |
| TP-010 §2.5 P1 create 単一 Tx・本体不変 | 永続化 | ケース 2, 14, 21 |
| TP-010 §2.5 P2 読取系+delete 非破壊 | 永続化 | ケース 7, 21 |
| TP-010 §2.5 P3 content_hash/model_version 複製 | 永続化 | ケース 2 |
| TP-010 §2.5 P4 障害回復保証 | 永続化 | ケース 14（ロールバック）+ NFR-LGX-001 REL.01/06 へ委譲 |
| TP-010 §2.5 P5 テーブル構造 | 永続化 | ケース 2, 16（DD §11 スキーマで具体化） |
| TP-010 §2.6 V1/V2 引数体系・グローバル 3 オプション | 互換 | ケース 17, 18, 19（snapshot 系の引数契約。`--project-root`/`--json` は本 TS、`--models-dir` は drift 主管） |
| TP-010 §2.6 V3 snap- プレフィクス凍結 | 互換 | ケース 11, 12 |
| TP-010 §2.6 V4〜V7 drift モデル解決順 | 互換 | snapshot 範囲外 → TS-LGX-013 へ委譲 |
| TP-010 §2.7 I1 drift 3 形式 | 入力 | snapshot 範囲外 → TS-LGX-013 へ委譲 |
| TP-010 §2.7 I2 delete target 二形式 | 入力 | ケース 23（label:<L> 形態 delete 成功主経路: resolve → delete → deleted_rows>0）, 9（label resolve 単体）, 7/8（snapshot_id 直指定）, 10（label 不在 exit 1） |
| TP-010 §2.7 I3 label 非一意・最新解決 | 入力 | ケース 9, 6 |
| TP-010 §2.7 I4 構文層/意味層分離 | 入力 | ケース 17（構文 exit 2）, 10/18（意味 exit 1）|
| TP-010 §2.7 I5 label 文字列境界 | 入力 | clap/DD 入力検証詳細（GAP-184 削除）。`label: Option<String>` は任意文字列受理（ケース 6） |
| TP-010 §2.8 L1 診断=stderr/結果=stdout | 観測 | ケース 20 |
| TP-010 §2.8 L2 json INFO 併出 | 観測 | ケース 19, 20 |
| TP-010 §2.8 L3 drift 試行内容通知 | 観測 | snapshot 範囲外 → TS-LGX-013 へ委譲 |
| TP-010 §2.8 L4 運用操作監査証跡 | 観測 | スコープ外（GAP-182 削除、操作監査ログは feature-addition）|
| TP-010 §2.8 L5 WARNING/INFO 文言ローカライズ | 観測 | ケース 19, 20（日本語文言 "ストアが空のため…"。NFR OBS.04 所有） |
| TP-010 §2.8 L6 機密非混入 | 観測 | NFR-LGX-001 SEC.05 へ委譲（embedding 本体非ログ化） |
| TP-010 §2.9 F1 MCP 非公開 | 境界 API | snapshot は MCP 非公開（MCP-INV-1）= 本 TS の CLI ケースが MCP 不在を前提（ケース 17〜20） |
| TP-010 §2.9 F2 CLI 引数契約一致 | 境界 API | ケース 17, 18, 19（LGX-COMPAT-001 §4 #8） |
| TP-010 §2.9 F3 json スキーマ | 境界 API | ケース 19 |
| TP-010 §2.10 D1 読取系決定性 | 決定性 | ケース 4（list property）。drift/report/calibrate は TS-LGX-013 等へ委譲 |
| TP-010 §2.10 D2 create 非決定性 | 決定性 | ケース 12（snapshot_id 時刻+乱数依存） |
| TP-010 §2.10 D3 list 降順安定+タイブレーク | 決定性 | ケース 4, 5, 9（resolve も同規則） |
| TP-010 §2.10 D4〜D8 report/check/drift 責務 | 領域 | snapshot 範囲外 → report/check/drift TS へ委譲 |
| TP-022 §2.1 BF1 ステップ連鎖整合 | UC フロー | ケース 13, 2, 7 |
| TP-022 §2.1 BF2 段階区分の観察可能性 | UC フロー | ケース 17（構文）, 9/10（意味）, 18 |
| TP-022 §2.1 BF3 成功時事後条件観察可能性 | UC フロー | ケース 13（list で観察）, 7 |
| TP-022 §2.1 BF4 snapshot_id 出力契約 | UC フロー | ケース 11, 19（text/json）|
| TP-022 §2.2 AF1 サブコマンド分岐網羅 | UC フロー | ケース 13（create/list/delete）, 17（省略） |
| TP-022 §2.2 AF2 create 境界 case（空ストア）| UC フロー | ケース 1, 16, 19 |
| TP-022 §2.2 AF3 list 境界 case（0 件）| UC フロー | ケース 3, 19 |
| TP-022 §2.2 AF4 delete target 形態 | UC フロー | ケース 7, 8（snapshot_id）, 23（label:<L> 成功主経路: resolve→delete）, 9（label resolve 単体）|
| TP-022 §2.2 AF5 代替フロー exit 収束 | UC フロー | ケース 18（2a/4a/6b=0、1a=2）|
| TP-022 §2.2 AF6 遷移条件の明示 | UC フロー | ケース 1, 3, 8, 9（各発火条件）|
| TP-022 §2.3 EF1 delete label 解決失敗 exit 1 | UC フロー | ケース 10, 18, 20, 22（GAP-291 解消観点）|
| TP-022 §2.3 EF2 create 書込み失敗パス | UC フロー | ケース 14（GAP-292 解消観点）|
| TP-022 §2.3 EF3 エラー時 atomicity | UC フロー | ケース 14, 21 |
| TP-022 §2.3 EF4 severity 区分・出力先 | UC フロー | ケース 20 |
| TP-022 §2.3 EF5 非永続見逃し二次検出 | UC フロー | ケース 1 + 13（list 不在で検出）|
| TP-022 §2.4 AT1 アクター権限一貫性 | アクター | 全アクター同一 snapshot 権限（権限差なし）= ケース 13 が代表（権限分岐テスト不要）|
| TP-022 §2.4 AT2 責任境界（実行 vs 判断）| アクター | UC-012 アクター責務記述（システム=機械的実行）。テスト対象は機械的実行のみ = ケース全般 |
| TP-022 §2.4 AT3 不可逆操作の事前確認 | アクター | ケース 13（delete 後 list で不在確認）|
| TP-022 §2.4 AT4 読取専用の並行整合性 | アクター | ケース 21（read-only）+ 並行は NFR へ委譲 |
| TP-022 §2.5 DF1 stdout/stderr 分離 | データフロー | ケース 19, 20 |
| TP-022 §2.5 DF2 スナップショットデータ寿命 | データフロー | ケース 2（content_hash/model_version 複製）+ UC-013 drift 参照へ委譲 |
| TP-022 §2.5 DF3 非破壊性データフロー | データフロー | ケース 15, 21 |
| TP-022 §2.6 R1 snapshot_id 不透明トークン性 | 領域 | ケース 11, 12 |
| TP-022 §2.6 R2 label 解決規則一貫性 | 領域 | ケース 9（drift と同規則）|
| TP-022 §2.6 R3 終了コード契約一致 | 領域 | ケース 18 |
| TP-022 §2.6 R4 DB 不在時挙動 | 領域 | ケース 15, 16 |
| TP-022 §2.6 R5 list 決定性 vs create 非決定性 | 領域 | ケース 4/5（list 決定性）, 12（create 非決定性）|

> 継承 TP（TP-LGX-010 全 71 観点 + TP-LGX-022 全 26 観点）はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（漏れ 0、人間ゲート判断対象）。drift / report / calibrate コマンドの観点（B4〜B9, E2〜E6/E9/E10, V4〜V7, I1, L3, D4〜D8）は SPEC-010 の別コマンド責務であり snapshot 範囲外として TS-LGX-013（drift）/ report・calibrate TS へ委譲、並行性・障害回復（C1〜C4, S6, P4 本体, L4, L6）は NFR-LGX-001 へ委譲する。本 TS は snapshot create / list / delete / resolve_label / generate_snapshot_id の finding・exit・決定論整列・read-only 不変・トランザクション原子性に集中する。
>
> **DD §8 の TP 引用に関する申し送り**: DD-LGX-012 §8「対応 TP」列は全行を TP-LGX-009（MCP サーバ）と記載しているが、snapshot 4 コマンドは MCP 非公開（MCP-INV-1 / TP-010 F1）であり TP-LGX-009 の観点（zod スキーマ・MCP error マッピング等）は本 feature と無関係。本 TS の実質的継承元は TP-LGX-010（SPEC）+ TP-LGX-022（UC）。DD §8 の TP-009 表記は記載誤りと判断し、TP-009 観点は本 TS に継承しない（DD 修正は上流 /defect-fix 経路の対象。HR6 により実装着手後は変更不可だが、本 TS の継承元は割当指示の TP-010+TP-022 に従う）。

## 4. テスト技法選択

- 同値分割: create の node_count（0 / >0）、list の件数（0 / >=1）、delete の deleted_rows（0 / >0）、resolve_label の解決（Resolved / NotFound）、delete target 形態（snapshot_id 直指定 = ケース 7 / `label:<L>` 形態成功 = ケース 23 / `label:<L>` 不在 = ケース 10）。各等価クラスに代表ケース。
- 境界値分析: 空ストア（0 件）= ケース 1/3、同一秒タイブレーク境界 = ケース 5、DB 不在境界（Read 非作成 / Write 作成）= ケース 15/16。
- Property-based: list の決定論整列（ケース 4）、generate_snapshot_id 形式・非衝突傾向（ケース 12）、embeddings read-only 不変（ケース 21）。
- 状態遷移: create → list → delete → list のライフサイクル状態機械（ケース 13）。トランザクション失敗時のロールバック状態（ケース 14）。

## 5. テスト基盤

- 言語: Rust（CLI 本体・legixy-embed / legixy-db crate）
- フレームワーク: cargo test
- Property-based: proptest（ケース 4, 12, 21）
- モック: 原則なし。DB は一時ディレクトリの実 SQLite ファイル（tempfile）を使用。トランザクション失敗誘発（ケース 14）は PRIMARY KEY 違反の実衝突または commit 失敗注入で再現。CLI exit code 検証（ケース 17〜20）は `assert_cmd` 等でプロセス起動。

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1, 2, 16 | TC-LGX-012（create 系） | legixy-embed/src/snapshot/mod.rs（#[cfg(test)]）|
| ケース 3, 4, 5, 6 | TC-LGX-012（list 系） | legixy-embed/src/snapshot/mod.rs |
| ケース 7, 8 | TC-LGX-012（delete 系） | legixy-embed/src/snapshot/mod.rs |
| ケース 9, 10 | TC-LGX-012（resolve_label 系） | legixy-embed/src/snapshot/mod.rs |
| ケース 11, 12 | TC-LGX-012（generate_snapshot_id） | legixy-embed/src/snapshot/mod.rs |
| ケース 13, 14, 15 | TC-LGX-012（E2E / 永続化） | legixy-embed/tests/snapshot_integration.rs |
| ケース 23 | TC-LGX-012（delete by label:<L> 成功 E2E、run_delete 配線） | legixy-cli/tests/snapshot_cli.rs |
| ケース 21 | TC-LGX-012（read-only property） | legixy-embed/tests/snapshot_integration.rs |
| ケース 17, 18, 19, 20, 22 | TC-LGX-012（CLI contract） | legixy-cli/tests/snapshot_cli.rs |
