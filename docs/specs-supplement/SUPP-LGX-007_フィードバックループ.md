Document ID: SUPP-LGX-007

# SUPP-LGX-007: SPEC-LGX-007（フィードバックループ）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-007 |
| 対象 SPEC | SPEC-LGX-007 フィードバックループ（Version 0.5.0, 2026-06-10 Approved） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
>
> 補完内容の根拠は主に旧文書群（`legixy.old.p1/docs/`）と旧実装
> （`traceability-engine.v3.chg_to_lexigy/`、以下「v3 実装」。crate 名は te-* → lx-* にリネーム済み）に求めた。
> [補完] = 旧資産から具体的内容を確認できたもの。[要決定] = 旧資産に存在しない・矛盾がある・人間判断が必要なもの。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリ内で解決できる参照は SPEC-LGX-001 / SPEC-LGX-003.REQ.07, REQ.19 / SPEC-LGX-004.REQ.04 / SPEC-LGX-009.REQ.02（いずれも `docs/specs/` に存在）のみ。以下は**未解決**。

| # | 参照 ID | SPEC 内の参照箇所 | 何のために必要か | 所在（旧リポジトリで発見） |
|---|---------|------------------|----------------|--------------------------|
| 1 | LEGIXY-SPEC-001 §2, §10 | ヘッダ表（親文書）、§2、REQ.01/06/07、REQ.11、§4 | FB-INV-1〜5 / MCP-INV-1〜4 / STATE-INV-1/2 の正準定義（§3 に転記）と製品概要 | `legixy.old.p1/docs/legixy_foundational_spec.md` |
| 2 | NFR-LGX-001（MAINT.05, OBS.01, SEC.02/04/05/08, REL.07/08） | ヘッダ表、REQ.01/05/10/11 | busy_timeout 上限（5000 ms）、マスキング義務、単独開発者前提、テスト不可侵、engine.db 配置条件 | `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md` |
| 3 | UC-LGX-008 | ヘッダ表（対応 UC） | 基本フロー・代替フロー（claim release）・事前事後条件 | `legixy.old.p1/docs/usecases/UC-LGX-008_フィードバックループ.md` |
| 4 | LGX-COMPAT-001 §3, §4, §4.1, §5 | REQ.01（凍結契約） | CLI 引数の凍結契約（observe 位置引数、CATEGORY 3 値、全 19 サブコマンド表、exit 2/1 規約、MCP→CLI マッピング） | `legixy.old.p1/docs/legixy_cli_compat_reference.md` |
| 5 | LGX-EXT-001 §4.3 | REQ.08（根拠） | engine.db スキーマ変更（サブノード ID 格納可能化）。※同節は DB 配置を `.legixy/engine.db` と記載し v3 実装（`.trace-engine/`）と矛盾 → §2-4 [要決定] | `legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md` |
| 6 | CLAUDE.md 絶対ルール 1・5、MCP ツール使用ルール | §2、REQ.01/02/03/05/10 | 「テストコード不可侵」「承認権限の制限（Proposal の承認・却下は人間のみ）」の規範文。新リポジトリに CLAUDE.md 自体が無い | `traceability-engine.v3.chg_to_lexigy/CLAUDE.md` §絶対ルール（1, 5） |
| 7 | GAP-LGX-121/122/124/126/127/129/135/139/140 | REQ.01/05/06/08/09、§4、§6 | 各境界決定の経緯・選択肢。SPEC に結論は反映済みだが追跡性のため | `legixy.old.p1/docs/gap-analysis/GAP-LGX-1xx_*.md`（9 件すべて存在） |
| 8 | QSET-LGX-007（Q1/Q2 回答 2026-06-07） | REQ.01/09 | dedup キー・semantic_key・CATEGORY 凍結の根拠回答全文 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-007_フィードバックループ.md` |
| 9 | SPP-LGX-007 | §6（v0.4.0） | v0.3.0→0.4.0 差分の承認記録 | `legixy.old.p1/docs/spec-patches/SPP-LGX-007_フィードバックループ.md` |
| 10 | ADR（「ADR に記録」: REQ.05/06/09、ADR-LGX-005 明示参照） | REQ.05/06/08/09 | GAP-126→ADR-LGX-005（破損時保護）、GAP-140→ADR-LGX-006（人間のみ二層強制）、GAP-139→ADR-LGX-004（可用性優先） | `legixy.old.p1/docs/adr/ADR-LGX-004/005/006_*.md` |
| 11 | DD（詳細設計。REQ.09「破損の検出契機・診断メッセージの詳細は DD で確定」「typestate か実行時検証かは DD に委譲」） | REQ.06/09 | legixy 用 DD は**未作成**（`legixy.old.p1/docs/detailed-design/` は空）。参考として v3 の DD-LX-006 が存在 | 参考: `traceability-engine.v3.chg_to_lexigy/docs/detailed-design/DD-LX-006_フィードバックループ.md` |
| 12 | 「v0.1.0 の feedback/analyze/proposals/approve/reject 実装」（慣例仕様） | §2、REQ.02〜04, 08, 09 | 慣例仕様の実体。v3 実装（§4 参照）が現存の最新形 | `traceability-engine.v3.chg_to_lexigy/crates/lx-feedback/` ほか |

**運用上の推奨**: 上記 1〜10 を新リポジトリへ転写（または正準コピー）するか、参照解決ルールを SPEC-LGX-001 側で定めるかは人間判断（[要決定]）。本文書は当面の参照先パスを示すに留める。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2-1. observations テーブルの具体スキーマ（REQ.08）— [補完]

SPEC は「v0.1.0 スキーマを継承」とのみ規定。実体は v3 実装 `crates/lx-db/src/schema.rs:71-85`:

```sql
CREATE TABLE IF NOT EXISTS observations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    related_ids TEXT NOT NULL DEFAULT '[]',   -- 正準化済み JSON 配列（distinct→昇順→JSON 文字列化）
    context_json TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_obs_dedup
    ON observations (category, related_ids)
    WHERE status IN ('pending', 'analyzing');   -- REQ.11 の dedup キーを DB レベルで強制（partial UNIQUE INDEX）
```

- SPEC 本文に登場しないカラム: `source`（`manual` / `auto:{category}` / `drift:contextual_retrieval`）、`severity`（`error`/`warning`/`info`、observe の `--severity` 既定 `info`）、`context_json`（下記 2-8）。
- SPEC REQ.08 の「related_node_id フィールド」という表現は v3 実体では `related_ids`（JSON 配列）である。乖離は字句のみで意味は同じ（複数 ID 格納可・サブノード ID 可）。
- 並行 INSERT のレースは UNIQUE 制約違反 → SELECT-then-skip フォールバックで吸収（`crates/lx-feedback/src/recorder.rs:34-98`）。REQ.11 の「内部リトライで吸収」の実装機構。

### 2-2. proposals テーブルの具体スキーマ（REQ.09）— [補完] + [要決定]

[補完] v3 実体 `crates/lx-db/src/schema.rs:99-130`:

```sql
CREATE TABLE IF NOT EXISTS proposals (
    id,                                  -- affinity なし（後述）
    observation_id INTEGER,
    kind TEXT NOT NULL DEFAULT '',
    semantic_key TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    action_json TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    decided_at TEXT,
    decided_reason TEXT,
    payload TEXT, resolved_at TEXT, resolution_reason TEXT,   -- 旧テスト互換の遺産列
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_proposals_semantic_key ON proposals (semantic_key, status);
-- id が NULL の INSERT に rowid を転写するトリガ proposals_fill_id_from_rowid あり
```

- SPEC の `approved_by` / `approved_at` / `reject_reason` に直接対応する列は v3 に**存在しない**。v3 の対応物は `decided_at`（approve/reject 共用）・`decided_reason`（reject 理由、approve 時は NULL）。`approved_by` 相当は無い（単独開発者前提で常に人間）。
- [要決定] 列名の選択: (a) v3 列名（decided_at/decided_reason）を継承し SPEC の語を論理名とみなす / (b) SPEC の語通りの列（approved_by, approved_at, reject_reason）で新設計。engine.db は再生成可能キャッシュ扱いの例外データを含むため、v3 DB をそのまま読む後方互換が必要なら (a)、不要（migrate で変換）なら (b)。
- [要決定] v3 の遺産列（`payload`/`resolved_at`/`resolution_reason`、id の affinity なし + トリガ）は旧テスト TC-LX-002 互換のための吸収であり、legixy 新実装で引き継ぐ必然性は無い。`id INTEGER PRIMARY KEY AUTOINCREMENT` への純化を推奨（v3 DD-LX-006 観察事項 9 でも MIG 連携での純化が提起済み）。
- [補完] proposal の status は SPEC 上 3 値（pending/approved/rejected）だが、v3 enum には `Skipped` が存在する（`crates/lx-feedback/src/analyzer.rs:11-16`）。DB 上に 'skipped' proposal 行が作られる経路は v3 に無く、enum の余剰。legixy では 3 値で実装してよい。

### 2-3. context_log テーブルと記録ペイロード形式（REQ.06/07）— [補完]

[補完] v3 実体 `crates/lx-db/src/schema.rs:155-161` と書込側 `crates/lx-ctx/src/audit_logger.rs:25-66`:

```sql
CREATE TABLE IF NOT EXISTS context_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id TEXT NOT NULL,          -- 解決された先頭 artifact_id（解決不能時は空文字）
    granularity TEXT NULL,            -- "document" / "subnode"
    payload TEXT NOT NULL,            -- 下記 JSON
    created_at TEXT DEFAULT (datetime('now'))
);
```

payload JSON（audit_logger.rs:45-55）:
`{"command", "target_files"（区切りは / に正規化）, "targets"（解決済み ID 配列）, "granularity", "upstream_count", "layer_count", "additional_count", "custom_count", "unresolved_count"}`

- SPEC REQ.06 の「タイムスタンプ、target_files、返却ノード、granularity」はこの形で充足される。書込責務は CTX 側（SPEC-LGX-003）で、FB 側は読取専用（`crates/lx-feedback/src/audit.rs` ContextAuditReader: `recent(limit)` / `by_target(target_id, limit)`、id 降順）。
- ベストエフォート書込（失敗時 stderr に `audit log write failed (best-effort)` を出し本体は Ok）は GAP-LGX-139 の確定方針と一致しており、v3 実装をそのまま参考にできる。

### 2-4. engine.db の配置パスと接続設定 — [補完] + [要決定]

[補完] v3 実体 `crates/lx-db/src/connection.rs:15-33`: `{project_root}/.trace-engine/engine.db`。接続時 PRAGMA: `journal_mode=WAL` / `synchronous=NORMAL` / `busy_timeout=5000`（NFR REL.07 の暫定値） / `foreign_keys=ON`。ディレクトリ不在時は作成、スキーマは `CREATE TABLE IF NOT EXISTS` で冪等初期化。NFR REL.08: ネットワーク共有上の配置禁止（起動時検出で Warning）。

[要決定] **ディレクトリ名の矛盾**: LGX-EXT-001 §4.3 は `.legixy/engine.db`、v3 実装と LGX-COMPAT-001 §6 は `.trace-engine/`（設定ファイルも `.trace-engine.toml`）。CLI 実行時引数互換の制約（traceability-engine.v3 バイナリとの互換）を踏まえると:
- 案 A: `.trace-engine/` を維持（v3 バイナリと同一プロジェクトで相互運用可能。リブランドが不徹底）
- 案 B: `.legixy/` へ移行 + migrate で旧パスから移動（リブランド徹底。v3 バイナリとの併用不可）
- 案 C: `.legixy/` 優先 + `.trace-engine/` フォールバック読取

### 2-5. feedback コマンドのアルゴリズム（REQ.02）— [補完]

SPEC は一文のみ。v3 実体 `crates/lx-feedback/src/observer.rs:74-135`（AutoObserver）+ `crates/lx-cli/src/commands/feedback.rs`:

1. check 全体を実行（`run_all_from_paths`）して CheckReport を得る
2. フィルタ規則: `severity == Ok` 除外 / `FileExistence × Error` 除外 / `DocumentId × Warning` 除外 / 既知カテゴリ以外除外
3. カテゴリ写像: ChainIntegrity→`chain_integrity`、LinkCandidate→`link_candidate`、Drift→`drift`、OrphanFile→`orphan_file`、SemanticSimilarity→`semantic_similarity`（その他の check カテゴリは Observation 化しない）
4. `source = "auto:{category}"`、related_ids は昇順ソート、ObservationRecorder::record で dedup INSERT
5. stdout: `feedback: {N} created, {M} skipped`

補助経路: embedding の `ContextualRetrievalFailed` を drift Observation 化する `drift_from_embed_error`（observer.rs:142-157）。message は **必ず `mask_api_key` を通す**（NFR SEC.05 のマスキング義務）。

**注意**: feedback が生成する category（chain_integrity 等 5 種）は REQ.01 で凍結された observe の 3 値（compile_miss/review_correction/manual_note）とは**別の集合**である。REQ.01 の凍結は observe（Agent/CLI 手動経路）の入力契約であり、observations テーブルの category 値域全体ではない。実装時に混同しないこと。

### 2-6. analyze コマンドのアルゴリズム（REQ.03）— [補完] + [要決定]

[補完] v3 実体 `crates/lx-feedback/src/analyzer.rs:67-324`（SPEC REQ.09 が根拠として実測参照する箇所）:

1. **Pessimistic Claim**: 単一トランザクションで `UPDATE observations SET status='analyzing' WHERE status='pending'` → analyzing 行を SELECT
2. カテゴリ→proposal kind 変換: `chain_integrity`→`add_chain_entry`（action: missing_id = related_ids 先頭）/ `link_candidate`→`add_link`（related_ids ソート後の先頭 2 つ）/ `drift`→`update_doc`（changed_id = 先頭、review_targets = 全 related_ids）/ `orphan_file`→`add_link`（target_id 空）。`semantic_similarity`・observe 3 値（compile_miss 等）・未知カテゴリは Proposal 化対象外
3. semantic_key 生成（analyzer.rs:276-312）: SPEC REQ.09 の 3 形式に加え v3 には `add_custom_edge:{from_id}:{to_id}` と fallback `unknown:{kind}` が存在
4. `status='pending'` の同一 semantic_key があれば INSERT 抑止（FB-INV-5）
5. INSERT 列: observation_id, kind, semantic_key, title（`"{kind}: {message}"`）, description（message）, action_json, status='pending'
6. Observation の事後状態遷移とエラー時 Claim Release（pending へ戻す）
7. stdout: `analyze: {N} proposals generated`

[要決定] **observation 状態モデルの SPEC/v3 差分**（実装上最大の分岐点）:
- SPEC REQ.08（GAP-LGX-129 で確定）: `pending / analyzing / resolved` の 3 値。analyze 取込→analyzing、**対応 proposal の approve→resolved**、reject または proposal 未生成→pending（再分析対象に戻る）
- v3 実装: `pending / analyzing / proposed / skipped`（`crates/lx-feedback/src/observer.rs:13-18`）で、analyze 完了時に proposed/skipped へ即遷移し、approve/reject は observation に**波及しない**
- SPEC が正準であり v3 から挙動変更が必要。実装には未規定の決定が残る:
  1. approve→resolved の連動には proposal→observation の逆参照（observation_id 列で可）と approve トランザクション内での observation UPDATE が必要（FB-INV-2 の原子性境界に observation 更新を含めるか）
  2. 「proposal 未生成→pending」の場合、Proposal 化対象外カテゴリ（semantic_similarity、observe 3 値）の observation は **analyze のたびに pending へ戻り永久に再 claim される**。v3 の skipped はこの死蔵問題への手当だった。選択肢: (a) SPEC 字義通り（毎回再分析、無限ループ容認）/(b) resolved 扱いにする（SPEC の resolved 定義から逸脱）/(c) SPEC 改訂で skipped 相当の終端を追加（人間承認必要）
  3. semantic_key 重複で INSERT 抑止された observation の遷移先（SPEC 字義では「proposal 未生成→pending」だが、既存 pending proposal が対応中とみなして analyzing 維持・既存 proposal への紐付けも考えうる）

### 2-7. approve / reject の副作用と実装機構（REQ.05/09）— [補完] + [要決定]

[補完] v3 実体 `crates/lx-feedback/src/manager.rs`:
- approve: 単一トランザクション内で (1) proposal を SELECT、(2) `status != 'pending'` なら `InvalidProposalStatus` エラー、(3) `kind == "add_custom_edge"` のときのみ action_json から from_id/to_id/reason を取り出し `custom_edges` に INSERT、(4) `UPDATE proposals SET status='approved', decided_at=datetime('now')`。**それ以外の kind（add_chain_entry/add_link/update_doc）は記録のみで副作用なし**（graph.toml の自動更新はしない — 修正は人間が行う。REQ.10 と整合）
- reject: reason 空文字列は `EmptyRejectReason` エラー。`UPDATE ... SET status='rejected', decided_at, decided_reason=reason`
- CLI 層（`crates/lx-cli/src/commands/approve.rs` / `reject.rs`）: 実行時に stderr へ `[WARNING] approve / reject は人間の承認コマンドです（CLAUDE.md 絶対ルール 5）` を出力。stdout は `approved proposal id={id}` / `rejected proposal id={id}`

[要決定] SPEC REQ.09 は競合解決を **CAS（`UPDATE ... WHERE status='pending'` の更新行数判定）** と規定するが、v3 は fetch-then-update（approve はトランザクション内、**reject はトランザクション外**）であり CAS になっていない。legixy では SPEC 通り CAS で実装する（v3 踏襲不可）。その際の詳細は DD 確定事項: 更新行数 0 のとき「対象不在」と「終端状態への再操作」を区別する診断（SELECT で事後判別）を行うか、exit 1 のメッセージをどう分けるか。
[要決定] reject の `--reason` 空白のみ（trim 後 0 文字）拒否は SPEC v0.5.0 の新規定（GAP-LGX-124）。v3 は `is_empty()` のみで空白のみ文字列を通す。trim 判定を追加実装すること（v3 差分）。observe の message 空白のみ拒否（GAP-LGX-121）も同様に v3 に存在しない新規実装。

### 2-8. observe コマンドの全引数と context_json 形式（REQ.01）— [補完]

SPEC は category/message/related_id しか記述しない。v3 実体（`crates/lx-cli/src/main.rs:196-220`、`commands/observe.rs`、LGX-COMPAT-001 §4 #14）:

- 位置引数: `<category> <message>`（v0.1.0 の `--category`/`--message` フラグは v3 で廃止済み — 凍結契約）
- フラグ: `--severity <S>`（既定 info）、`--related-id <ID>`（0 個以上繰返し）、`--target-file <PATH>`（0 個以上繰返し）、`--missing-doc <ID>`、`--source-glob <GLOB>`
- `--target-file`/`--missing-doc`/`--source-glob` のいずれか指定時、`context_json = {"target_files": [...], "missing_doc": ..., "source_glob": ...}` として格納。未指定なら NULL
- `source = "manual"` 固定
- **stdout 契約（互換上重要）**: `observation: id={N} skipped={true|false}` の単一行。MCP 層が正規表現 `^observation:\s*id=(\d+)\s+skipped=(true|false)` でパースする（`ts-mcp/src/tools/observe.ts:18-29`）。形式変更は MCP 層を壊す
- CLI 層の category 3 値検証（不正値 exit 2）は SPEC v0.4.0 の【v3 差分】であり v3 には**存在しない**（`category: String` 無検証）。clap の `ValueEnum` 相当で新規実装する

### 2-9. audit コマンド（get_compile_audit の下位層、REQ.07）— [補完] + [要決定]

[補完] v3 実体 `crates/lx-cli/src/commands/audit.rs`: `audit [--limit <N>]`、limit は 1..=50・既定 10、範囲外は bail（exit 1）。出力は ContextLogEntry（id, target_id, granularity, payload, created_at）の JSON 配列（pretty）。MCP `get_compile_audit` は zod `limit?: int 1..50` を受け `audit --limit N` へ転送、結果を Markdown 整形し `_meta["anthropic/maxResultSizeChars"]=500000` を付与（`ts-mcp/src/tools/get-compile-audit.ts`）。

[要決定] **v3 の MCP 整形コードと Rust 出力の不整合（バグ疑い）**: `get-compile-audit.ts:15-27` の `formatAuditEntry` は `e.input_files` / `e.input_command` を参照するが、Rust 側 ContextLogEntry に該当フィールドは無い（実際は `target_id`/`payload`(JSON 内に target_files/command)/`created_at`）。v3 では Files が常に undefined 相当で表示されていた可能性が高い。legixy 実装時にどちらを契約とするか決定が必要: (a) Rust 出力（target_id/payload）に合わせ MCP 整形を修正（推奨。payload を parse して target_files/command を表示）/(b) Rust 側出力に input_files/input_command を追加。MCP-INV-2（忠実な転送）の観点では (a) が自然。

### 2-10. exit code の全体規約 — [補完]

LGX-COMPAT-001 §3（v1.0.1 追記）: 使用法誤り（引数パーサ層の構文エラー）= exit 2（clap 既定の契約化）、受理済み引数の意味的不正・実行時失敗 = exit 1。SPEC-LGX-007 内の各 exit 指定（category 不正 exit 2、message/--reason 空 exit 1、終端 proposal 再操作 exit 1、CAS 敗者 exit 1、DB 破損 exit 1）はこの規約の適用。v3 の FeedbackError（`crates/lx-feedback/src/error.rs`: ProposalNotFound / InvalidProposalStatus / EmptyRejectReason / InvalidObservationCategory / Db / Json / Io / AnalyzeFailed）は anyhow 経由で exit 1 に落ちる。proposal 不在（ProposalNotFound）の exit code は SPEC 未規定だが規約上 exit 1 でよい。

### 2-11. DB 破損の検出契機と診断（REQ.09、GAP-LGX-126 / ADR-LGX-005）— [要決定]

SPEC 自身が「破損の検出契機・診断メッセージの詳細は DD で確定する」と明示委譲。v3 に検出実装は無い（rusqlite エラーの素通し。`Connection::open` は破損ファイルでも成功し得る）。DD 起草時の論点:
- 検出契機の選択肢: (a) 接続直後に `PRAGMA integrity_check`（確実だが毎回コスト）/(b) `PRAGMA quick_check` /(c) 操作中の SQLITE_CORRUPT / NotADatabase エラーを捕捉して破損と判定（遅延検出・低コスト）/(d) 書込系コマンドのみ (a)、読取系は (c)
- 「不在」（正常 — 新規作成して続行）と「破損」（exit 1）の判別: ファイル不在 = 不在、存在するが open/PRAGMA 失敗 = 破損、が素直
- 診断メッセージに復旧手順（バックアップ復元・手動削除の案内）を含めるか

### 2-12. 並行 observe の「内部リトライ」具体策（REQ.11）— [補完]

v3 実体: (1) `busy_timeout=5000`（NFR REL.07 暫定値、無限リトライ禁止）でロック競合を吸収、(2) dedup は事前 SELECT → INSERT → UNIQUE 制約違反時に SELECT-then-skip フォールバック（`crates/lx-feedback/src/recorder.rs:48-78`）の二段構え。「BUSY を Agent に返さない」は busy_timeout 内で解決する限り成立し、超過時は失敗として返す（NFR REL.07。完全な無 BUSY 保証ではない点に注意）。distinct 化（GAP-LGX-122 追加分）は v3 recorder に**無い**（sort のみ）。legixy では sort 前に dedup を追加すること（v3 差分）。

### 2-13. REQ.10 の検証手段「パイプラインフックの role 別書き込み制御」— [要決定]

s1/s2 の role 識別は `$env:PIPE_ROLE`（NFR SEC.08）前提の開発プロセス側機構であり、legixy 製品コードの要件ではない。新リポジトリにはパイプライン設定・フックがまだ存在しないため、検証環境（DevProc V4.1 のフック群）をどう持ち込むかは人間判断。製品実装としての作業は不要という理解でよいかの確認を推奨。

### 2-14. グローバルオプションとの組合せ — [補完]

LGX-COMPAT-001 §3: 全コマンド共通で `--project-root <PATH>`（既定 `.`）、`--json` が存在。MCP 層は常に `<bin> --project-root <root> <subcommand...>` 形式で起動する。v3 の feedback/analyze/approve/reject は `--json` 時の専用出力を持たない（プレーン println のみ）。`--json` 指定時の出力スキーマは SPEC/COMPAT とも未規定 — 互換上は v3 同様プレーン出力のままでも破壊にならない（[補完]、ただし整備するなら要決定）。

---

## §3 用語・前提の補完

| 用語 | 定義（出典） |
|------|-------------|
| Admin Surface / Agent Surface | 人間が CLI で操作する面 / Agent が MCP ツール経由でのみ操作する面。MCP は `compile_context`・`observe`・`get_compile_audit` の 3 ツールのみ公開（MCP-INV-1、LEGIXY-SPEC-001 §10.4、LGX-COMPAT-001 §5） |
| FB-INV-1〜5 | 1: Observation 冪等性（同内容は重複生成されない）/ 2: Proposal 承認原子性（1 トランザクション）/ 3: 承認前不変性（pending proposal は context 結果に影響しない）/ 4: DB 不在時安全性（DB がなくてもグラフ上流は正常に返る）/ 5: Proposal 重複排除（同一 semantic_key の pending は最大 1 つ）。出典: legixy_foundational_spec.md §10.2 |
| MCP-INV-1〜4 | 1: Agent Surface 限定（3 ツールのみ）/ 2: 忠実な転送（CLI 出力の加工なし）/ 3: MCP 経由でも Observation 重複排除が機能 / 4: compile_context 全呼出しの記録。出典: 同 §10.4 |
| STATE-INV-1/2 | 1: ステートレス性 — 永続状態は graph.toml（Git 管理）と engine.db（再生成可能キャッシュ）のみ（observations/proposals は ADR-LGX-005 によりこの「キャッシュ」扱いの明示的例外）/ 2: graph.toml 変更は Git commit 経由。出典: 同 §10.5 |
| v0.1.0 | 旧製品 traceability-engine の初期バージョン。SPEC の「v0.1.0 継承」は実質的に v3 実装（0.4.0-alpha4 系、§4 参照）で確認するのが現実的 |
| 前段ループ反復 1 | DevProc V4.1 フロントエンドパス（QSET 発行→人間回答→SPP 承認→SPEC 改訂）の第 1 反復。QSET-LGX-007 / SPP-LGX-007 が成果物 |
| Pessimistic Claim | analyze 冒頭で pending 全件を analyzing に一括 UPDATE してから処理する排他パターン。失敗時は pending へ戻す（Claim Release）。出典: v3 DD-LX-006 §3.3 |
| semantic_key / related_ids 正準形 | SPEC REQ.09 / REQ.11 に正準定義あり。生成手順（distinct→昇順 sort→JSON 文字列化）は両者で共有 |
| s1 / s2 | DevProc V4.1 の役割コード（s1=設計者、s2=実装者）。テストコード不可侵（絶対ルール 1）の主体区分 |
| ハードルール 1 / 7 | DevProc V4.1 の開発プロセス規則（1: SPEC 変更は人間承認、7: 凍結境界の変更は次バージョン SPEC 改訂）。新リポジトリでは `docs/DevPorc/` に vendored（ディレクトリ名は "DevPorc" と綴られている点に注意） |
| CATEGORY 3 値と feedback 生成カテゴリの関係 | 凍結 3 値（compile_miss/review_correction/manual_note）= observe 入力契約。feedback 自動生成は別系列（chain_integrity/link_candidate/drift/orphan_file/semantic_similarity）。proposal kind への変換規則を持つのは後者のみ（§2-5/2-6） |
| custom_edges | approve の副作用先テーブル（from_id, to_id, reason、UNIQUE(from_id,to_id)）。schema.rs:132-139 |
| engine.db | プロジェクトローカルの SQLite。配置は §2-4 参照（名称矛盾あり） |

---

## §4 旧実装からの参考情報（v3 実装の該当箇所）

ベース: `traceability-engine.v3.chg_to_lexigy/`（crate 接頭辞は te-* から lx-* へリネーム済み。QSET/SPEC が引く `te-feedback/src/analyzer.rs:275-312` 等の行番号は `crates/lx-feedback/src/analyzer.rs` の 276-312 にほぼ対応）。

| 関心事 | パス |
|--------|------|
| FB ブロック本体（crate: lx-feedback） | `crates/lx-feedback/src/observer.rs`（AutoObserver・状態 enum・drift_from_embed_error）/ `recorder.rs`（dedup INSERT）/ `analyzer.rs`（Pessimistic Claim・semantic_key）/ `manager.rs`（approve/reject）/ `audit.rs`（ContextAuditReader）/ `error.rs` / `cli.rs`（FeedbackCli ファサード） |
| DB スキーマ・接続（crate: lx-db） | `crates/lx-db/src/schema.rs`（observations:71-85, proposals:99-130, context_log:155-161, custom_edges:132-139）/ `connection.rs`（open_engine_db、PRAGMA） |
| CLI コマンド（crate: lx-cli） | `crates/lx-cli/src/main.rs:178-247`（引数定義）、`commands/{feedback,observe,audit,analyze,proposals,approve,reject}.rs`（stdout 契約・limit 検証・WARNING 出力） |
| context_log 書込側（crate: lx-ctx） | `crates/lx-ctx/src/audit_logger.rs`（payload 形式・ベストエフォート方針） |
| MCP 層（ts-mcp） | `ts-mcp/src/tools/observe.ts`（zod enum・stdout パーサ）/ `tools/get-compile-audit.ts`（limit zod・_meta・※整形フィールド不整合 §2-9）/ `server.ts` / `types.ts` |
| v3 詳細設計（DD の参考） | `docs/detailed-design/DD-LX-006_フィードバックループ.md`（アルゴリズム §3、型 §4、エラー §7、並行性 §8、観察事項 §9 — 特に観察事項 4「approve による graph.toml 自動更新は将来課題」、観察事項 9「proposals schema 純化」） |
| 旧テスト | `old.source/tests/TC-FB-001_feedback_test.rs` |
| 規範文書 | `CLAUDE.md`（絶対ルール 1〜8、MCP ツール使用ルール、ID 体系） |
| 利用者向けマニュアル | `deploy/manual.md`（CLI 利用手順の実例） |

**v3 から意図的に変える点（SPEC v0.5.0 が確定済みの v3 差分）の要約**: CLI 層 category 検証（exit 2）/ message・--reason の trim 空拒否（exit 1）/ related_ids の distinct 化 / proposal 操作の CAS 化 / observation 状態モデルの 3 値化（proposed・skipped 廃止 — ただし §2-6 の未決事項あり）/ DB 破損時 exit 1。

---

## 集計

- 未解決参照: 12 件（§1）
- [補完]: 11 件（§2-1, 2-2(一部), 2-3, 2-4(一部), 2-5, 2-6(一部), 2-7(一部), 2-8, 2-9(一部), 2-10, 2-12, 2-14）
- [要決定]: 9 件（§1 転写方針, §2-2 列名と遺産列, §2-4 ディレクトリ名, §2-6 observation 状態モデルの残課題, §2-7 CAS 詳細, §2-9 audit フィールド不整合, §2-11 破損検出契機, §2-13 REQ.10 検証環境, §2-14 --json 出力）
