Document ID: SUPP-LGX-008

# SUPP-LGX-008: SPEC-LGX-008（マイグレーション）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-008 |
| 対象 SPEC | SPEC-LGX-008_マイグレーション.md（Version 0.7.0, Status: Approved） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |
| 調査範囲 | legixy.old.p1/（以下 OLD）、traceability-engine.v3.chg_to_lexigy/（以下 V3） |
| 関連 | SUPP-LGX-000_参照文書インベントリ.md |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**

表記: **[補完]** = 旧文書・旧実装から根拠付きで補完できた事項。**[要決定]** = 根拠が見つからない／SPEC と旧資料が矛盾する等、人間の判断が必要な事項。

---

## §1 未解決参照（SPEC-LGX-008 が参照するが新リポジトリに存在しない文書）

新リポジトリ legixy には docs/specs/ の SPEC 10 ファイルのみが存在する。SPEC-LGX-008 が参照する以下の文書はすべて新リポジトリに不在である（計 **14 項目**）。

| # | 文書 ID | 参照箇所（SPEC-LGX-008 内） | 必要な理由 | 所在 |
|---|---------|------------------------------|------------|------|
| 1 | LGX-EXT-001（§4.3, §4.4, §8.2） | ヘッダ親文書、§2、REQ.01/03/04/05 | engine.db 自動マイグレーション解釈・後方互換・Phase 1 方針の根拠 | OLD/docs/legixy_subnode_spec_v0.2.1.md（ヘッダ ID 一致 v0.2.1） |
| 2 | LGX-COMPAT-001（§4 #1/#2, §6） | REQ.02a/06/07/13 | init / migrate の凍結引数契約、設定ファイル互換、exit code 規約 | OLD/docs/legixy_cli_compat_reference.md（ヘッダ ID 一致 v1.1.0） |
| 3 | NFR-LGX-001（COMPAT.04/05/11, REL.01/07/08, SEC.04/08, PERF.07, USE.02, OBS.02） | ヘッダ、REQ.01/02/02a/03/08/12 ほか | busy_timeout 値・WAL 要件・出力規約等の具体値 | OLD/docs/nfr/NFR-LGX-001_非機能要件.md |
| 4 | UC-LGX-009 | ヘッダ、REQ.07 根拠 | init / migrate の主・代替フロー（feedback.db / vectors.bin への言及あり、§2.1・§2.10 参照） | OLD/docs/usecases/UC-LGX-009_プロジェクト初期化とマイグレーション.md |
| 5 | LEGIXY-SPEC-001（不変条件定義） | §4 不変条件表（CTX-INV-2, STATE-INV-1/2, FB-INV-4 等） | §4 表の各不変条件の定義本文 | OLD/docs/legixy_foundational_spec.md §10（SUBNODE-INV-* は LGX-EXT-001 §7.2）。§3 に転記 |
| 6 | GAP-LGX-023/141/142/143/144/146/148/150/151/152/153/154/157/158/159/160（16 件） | REQ.02/02a/03/03a/07/08/09/11、変更履歴 | 各 REQ の確定経緯・裁定理由（実装判断時の背景） | OLD/docs/gap-analysis/（16 件すべて実在を確認） |
| 7 | ADR-LGX-011 | REQ.02（並行アクセス）、変更履歴 | migrate 並行排他リスク受容の決定記録 | OLD/docs/adr/ADR-LGX-011_migrate-concurrency-risk-acceptance.md（accepted 2026-06-10） |
| 8 | ADR（GAP-157 裁定の「整合判断は ADR に記録」） | REQ.06 裁定記録 | --from/--to PATH 正準化の決定記録 | OLD/docs/adr/ADR-LGX-001_migrate-from-to-path-canonical.md（accepted 2026-06-10） |
| 9 | QSET-LGX-008（Q1/Q2 回答） | REQ.07/13、変更履歴 0.5.0 | 単段 ICONIX 既定・設定二層構造の確定根拠 | OLD/docs/frontend-pass/questionnaires/QSET-LGX-008_マイグレーション.md |
| 10 | SPP-LGX-008 | 変更履歴 0.5.0 | 前段ループ反復 1 の SPEC 差分（承認済） | OLD/docs/spec-patches/SPP-LGX-008_マイグレーション.md |
| 11 | DD-LGX-007 §3.1.1 | REQ.07（init テンプレートの既定） | init が出力する完全 template の正準定義 | **本体不在**（OLD/docs/detailed-design/ は空）。前身 **DD-LX-007 §3.1.1** が V3/docs/detailed-design/DD-LX-007_プロジェクト初期化とマイグレーション.md に実在（§2.21 に内容転記） |
| 12 | TS-LGX-007（§Init.*） | REQ.07 検証方法 | init 直後 check の期待値・テストケース | **本体不在**。前身 **TS-LX-007**（T-INIT-001〜012, T-MIG-CFG/MATRIX 等）が V3/docs/test-specs/TS-LX-007_プロジェクト初期化とマイグレーション.md に実在 |
| 13 | VAL-LGX-001（Finding E-03, E-05） | REQ.11/12 根拠 | engine.db 配置条件・ID 互換性断絶の指摘原文 | **本体不在**（OLD/docs/validation/ は空）。前身 **VAL-LX-001** が V3/docs/validation/VAL-LX-001_外部照合記録.md に実在（E-03: 行 307、E-05: 行 333。いずれも「対応済」） |
| 14 | workflow_2026-04-20_init-template-spec.md | 変更履歴 0.4.0-draft | REQ.07 改訂（INIT Block）の経緯 | **所在不明**（OLD / V3 とも未発見。変更履歴のみの参照であり実装影響は小） |

注: DD-LGX-007 / TS-LGX-007 / VAL-LGX-001 は LGX 系列としては未生成（SUPP-LGX-000 §2 と同旨）。前身 LX 系列文書（V3 リポジトリ）はリブランド前の内容であり、新リポジトリでの再生成（または正準化）には人間承認が必要。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 [要決定] 移行元 v0.1.0 DB の実体 — 「engine.db」か「feedback.db」か（最重要ギャップ）

- **論点:** REQ.01 は「v0.1.0 フォーマットの `engine.db` を検出した場合」と規定するが、**実在する v0.1.0 プロジェクトの DB は `.trace-engine/feedback.db`** である。
  - 実例: V3/old.source/.trace-engine/ には `feedback.db`（32KB, user_version=0）のみ存在し、engine.db は無い（実機確認済）。
  - UC-LGX-009 の migrate フロー手順 5 も「**feedback.db** を engine.db に移行（テーブル構造コピー）」と記述（OLD/docs/usecases/UC-LGX-009）。
  - v3 実装も `v01_root/.trace-engine/feedback.db` を読む（V3/crates/lx-mig/src/db.rs:37、不在時は Warning + skip）。
- **選択肢:**
  - A) SPEC の「engine.db」を「v0.1.0 の DB ファイル群（実体は feedback.db）」の総称と解釈し、実装は `feedback.db` を一次対象、`engine.db`（user_version=0 のもの）も存在すれば対象とする（v3 実装 + UC と整合、推奨）。
  - B) SPEC 文言どおり engine.db のみ対象とする（実在データを移行できず COMPAT.04 を満たさない恐れ）。
  - いずれにせよ REQ.01 の文言と実態の不一致は SPEC 変更（人間承認）候補として申し送るべき。

### 2.2 [補完] v0.1.0 DB（feedback.db）の正確なスキーマ

実機 DB（V3/old.source/.trace-engine/feedback.db、`PRAGMA user_version` = **0**）から抽出:

- `observations(id INTEGER PK AUTOINCREMENT, source TEXT NOT NULL, category TEXT NOT NULL, severity TEXT NOT NULL, message TEXT NOT NULL, related_ids TEXT NOT NULL DEFAULT '[]', context_json TEXT, status TEXT NOT NULL DEFAULT 'pending', created_at TEXT NOT NULL DEFAULT (datetime('now')))`
- `proposals(id INTEGER PK AUTOINCREMENT, observation_id INTEGER, kind TEXT NOT NULL, semantic_key TEXT NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL, action_json TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', decided_at TEXT, decided_reason TEXT, created_at TEXT ...)`
- `custom_edges(id INTEGER PK AUTOINCREMENT, source_glob TEXT NOT NULL, target_path TEXT NOT NULL, description TEXT, created_at TEXT ..., created_by TEXT NOT NULL DEFAULT 'manual', deleted_at TEXT)` — **v0.1.0 は from_id/to_id ではなく source_glob/target_path**（§2.6 参照）
- `layer_rules(id, path_glob, layer_name, specificity, priority, created_at)`
- `layer_documents(id, layer_name, document_path, deleted_at)`
- `context_log(id, input_files, input_command, resolved_targets, upstream_artifacts, custom_documents, layer_documents, created_at)` — **granularity カラムは無い**（実機 PRAGMA table_info で確認）。REQ.01 の「legixy で追加されたカラム（context_log.granularity）」と整合。

REQ.03a の「必須テーブルの欠落」検出対象は、v3 実装上はコピー対象テーブル（observations / proposals / custom_edges）の必須カラム検査が該当（V3/crates/lx-mig/src/db.rs:85-146）。

### 2.3 [補完] 移行先（legixy）engine.db のスキーマ・PRAGMA・user_version

V3/crates/lx-db/src/schema.rs（CREATE TABLE 群）および connection.rs:15-32 より:

- テーブル: `nodes`, `edges`, `embeddings`, `scores`, `observations`, `proposals`, `custom_edges(from_id, to_id, reason, UNIQUE(from_id,to_id))`, `layer_rules`, `layer_documents`, `context_log`（granularity 含む）, `embedding_snapshots`
- PRAGMA: `journal_mode=WAL`（NFR PERF.07）、`synchronous=NORMAL`（同）、`busy_timeout=5000`（**REL.07 の上限 5000ms** と一致）、`foreign_keys=ON`
- `PRAGMA user_version = 3` を明示設定（V3/crates/lx-mig/src/initializer.rs:137。REQ.09 の一次根拠と一致）
- DB 配置: `<project-root>/.trace-engine/engine.db`（v3）。legixy では SPEC REQ.07 により `.legixy/engine.db`（ディレクトリ名のリブランドが必要 — §2.20 注記）

### 2.4 [要決定] v0.1.0 設定スキーマの変種 — `[id.chain]` 単数形と `[id.chains]`（multi-area）の併存

- **論点:** REQ.03 は「`[id.chain]` の `order` が欠落・不正な場合は破損として Error」と規定する。しかし**実在する v0.1.0 設定**（V3/old.source/.trace-engine.toml）は multi-area モードであり、`[id.chain]` を持たず **`[id.chains]`（複数形・area 別 order）+ `[id.areas]`** を使用する。同ファイル行 25 に「`[id.chain]` は multi-area モード時は `[id.chains]` が優先されるため省略」と明記がある。
  ```toml
  [id.areas]
  CTX = { label = "Context Resolution", chain = "rust" }
  ...
  [id.chains]
  rust = { order = ["SPEC", "UC", "DD", "TS", "TC", "SRC"] }
  ```
- このまま REQ.03 を字義どおり実装すると、**実在の v0.1.0 プロジェクトが「破損」判定で Error になる**（REQ.03 自身の「v0.1.0 が許容していた構造の幅を migrate 側で狭めない」という方針と矛盾）。
- 参考: v3 実装は `id.chain.order` 欠落時に既定 `["UC","RB","SEQ","DD","TS","TC","SRC"]` へフォールバックし（V3/crates/lx-mig/src/matrix.rs:213-245）、Error にしない（= SPEC 0.7.0 の規定とも異なる）。
- **選択肢:** A) `[id.chains]`+`[id.areas]` を正規の v0.1.0 変種として抽出規則に含める（DD で構文細目を定義）／B) multi-area は非対応とし明確な診断付き Error／C) REQ.03 の「破損」判定を「`[id.chain]` も `[id.chains]` も無い場合」に限定する SPEC 改訂を提案。

### 2.5 [補完] matrix.md の抽出規則の細目（v3 実装の正準化候補）

V3/crates/lx-mig/src/matrix.rs:53-155 より:

- `|` で始まり `|` で終わる行のみを表として扱う。表行が 3 行未満（ヘッダ+区切り+データ最低 1）なら空 matrix（REQ.03 の空入力経路と整合）。
- 1 行目 = ヘッダ（typecode 列名）、2 行目 = 区切り、3 行目以降 = データ。
- 各データ行の**第 1 列（TE-001 等の行 ID）はノード化しない**。
- セル値 `-` または空文字は「不在」（None）。「概要/summary」ヘッダの最終列は summary として保持。
- ノード生成: chain order の typecode 列・independent の typecode 列・未宣言ヘッダ列（ヘッダ名を typecode と推定）の present セルをノード化。ID 重複は初出のみ（push_node_unique）。
- chain エッジ生成: order 配列の**隣接ペア（windows(2)）で両セルとも present の場合のみ** `kind="chain"` を生成。`-` セルは chain を中断する（飛び越しエッジは作らない）。
- independent 列はノードのみ生成、エッジは生成しない。
- 既定値（order/independent 未設定時）: order `["UC","RB","SEQ","DD","TS","TC","SRC"]`、independent `["SPEC","NFR","VAL"]`（matrix.rs:214-221）。ただし SPEC 0.7.0 では order 欠落は Error（§2.4 の論点）。
- v0.1.0 matrix.md 実例: V3/old.source/docs/traceability/matrix.md（`| # | SPEC | UC | DD | TS | TC | SRC | 概要 |` 形式）。

### 2.6 [要決定] custom_edges 継承の意味論 — source_glob はノード ID ではない

- **論点:** REQ.03 は「custom エッジは `custom_edges` テーブルから継承」と規定。v0.1.0 の custom_edges は `(source_glob, target_path, description)`（§2.2）であり、値は**パス glob / ファイルパス**であって成果物 ID ではない。
- v3 実装は値をそのまま `edges.from`/`edges.to` に流し込む（V3/crates/lx-mig/src/matrix.rs:157-189: source_glob→from, target_path→to, kind="custom"）。この場合、生成 graph.toml に**ノードとして存在しない from/to を持つ custom エッジ**が入り得て、REQ.03a の「出力の妥当性保証」（CTX-INV-2 保全）や REQ.11 の dangling 参照防止方針と緊張関係にある。
- **選択肢:** A) パス→ノード ID の解決を行い、解決不能エッジは REQ.11 のマッピング不可 ID と同じ扱い（既定 abort／opt-in 継続時は除外）に統一／B) v3 同様そのまま転記し check 側で検出させる／C) custom_edges のうち `deleted_at IS NOT NULL` の行の扱い（v3 は WHERE 句なしで全行継承）も併せて定義。

### 2.7 [補完] 新 ID（SHA-256 ベース）の生成規則（REQ.11）

V3/crates/lx-mig/src/id_map.rs:59-94 より:

- ハッシュ入力: `"{path}\n{type_code}"`（ノードのパスと typecode を改行連結）
- 新 ID 形式: `{type_code}-{area}-{sha256 16進の先頭 seq_digits 桁}`（area / seq_digits は `[id]` 設定から取得）
- 同一実行内で新 ID が衝突した場合は桁数を 1 ずつ伸ばして再試行し、Confidence を High→Warn に落とす
- REQ.11 の全単射検証（旧 ID 重複 / 新 ID 衝突 / graph 全体一意性）は v3 実装では桁伸長で回避しており **Error にしない**。SPEC 0.7.0（GAP-LGX-159）では Error が正準のため、**v3 実装をそのまま流用してはならない**（新規実装では検証を Error 化し、--dry-run でも実施）。

### 2.8 [補完] migration-id-map.toml のスキーマと配置

- スキーマ（V3/crates/lx-mig/src/id_map.rs、TOML シリアライズ）: `mappings` 配列、各要素 `{ old_id, new_id, confidence }`（confidence: High/Warn）。
- 配置: **SPEC REQ.11 は `.legixy/migration-id-map.toml`**。v3 実装は `docs/traceability/migration-id-map.toml`（V3/crates/lx-cli/src/commands/migrate.rs:136）。**新実装は SPEC の配置が正準**（v3 と差異がある点に注意。deploy/manual.md §12 の手順記述も旧配置前提なので転用時に修正が必要）。

### 2.9 [補完] フィードバックデータ（observations / proposals）の移行規則

SPEC REQ.01 は「既存データは保持」とのみ規定。具体規則は v3 実装（V3/crates/lx-mig/src/db.rs:29-146, 211-297）が参考になる:

- v3 側 engine.db を open（スキーマ初期化込み）→ user_version=3 設定 → **単一トランザクション内で** feedback.db からテーブルコピー → commit（REQ.02 の SQLite トランザクション使用と整合）。
- observations: 必須カラム `source, category, severity, message, related_ids` 検査の上、`INSERT OR IGNORE`（冪等性 = REQ.02 の再実行可能と整合）。
- proposals: 既知カラムのみコピー（v0.1.0 に無い `payload, resolved_at, resolution_reason` は既定値）。
- custom_edges: `(source_glob,target_path,description)` または `(from_id,to_id,reason)` の両スキーマ変種を検出して対応（§2.6 の論点あり）。

### 2.10 [要決定] vectors.bin（v0.1.0 埋め込みデータ）の扱い

- SPEC-LGX-008 は vectors.bin に**一切言及しない**。一方 UC-LGX-009 の migrate フロー手順 6 は「vectors.bin があれば embeddings テーブルにインポート」と記述し、v3 実装は「Phase 2 延期（ImportStrategy::Skip）+ Warning」（V3/crates/lx-mig/src/db.rs:53-60）。また config 移行は `[semantic].vector_store` キーを削除する（§2.11）。
- **選択肢:** A) v3 同様 Skip + Warning（embedding は移行後 `embed --all` で再生成。REQ.05 の Phase 1 方針と整合的、推奨）／B) UC どおりインポートを実装。UC と SPEC の不整合として申し送り推奨。

### 2.11 [補完] .legixy.toml 移行（REQ.04）で追加・削除されるセクションの全集合

SPEC REQ.04 は `[graph]` 追加のみ明記。v3 実装（V3/crates/lx-mig/src/config.rs:36-99）は次を行う（既存キーは保持）:

- 追加: `[graph] file = "docs/traceability/graph.toml"`、`[contextual_retrieval] enabled = false`、`[migration] auto = false`、`[freshness] enabled = true, method = "mtime"`（いずれも未存在時のみ）
- 削除: `[semantic].vector_store`（v0.1.0 専用キー）
- `[matrix]` は残置（REQ.04 の「graph.toml から matrix.md を生成する設定」へ意味変更）
- 注: `[migration]` の自動追記は **migrate 時のみ**。init では生成しない（DD-LX-007 §3.1.1 末尾の注記）。REQ.01 の opt-in 自動実行フラグの読み取りは V3/crates/lx-cli/src/autodetect.rs:112-122（`migration.auto`、既定 false）参照。

### 2.12 [補完]+[要決定] 退避命名 — v3 実装は SPEC 0.7.0 非準拠（流用禁止）

- [補完] v3 実装（V3/crates/lx-mig/src/backup.rs:32-115）は**固定名 `.bak`** に退避し、既存 `.bak` があればそれを `.bak.{unix秒}` へ玉突き退避する。SPEC REQ.02a は**固定 `.bak` を明示的に禁止**し、直接 `<元名>.bak.{unix epoch 秒}` とする。**v3 のコードをそのまま移植してはならない**。退避対象 3 点（設定 toml / graph.toml / engine.db）の選定は v3 が参考になる。
- [要決定] REQ.02a の「同一秒内衝突時は連番サフィックス」の具体形式が未定（候補: `<名>.bak.{epoch}.1`, `.2`, …）。DD で確定する。

### 2.13 [補完] 非 DB ファイルの atomic 書込 — v3 実装に該当機構なし（新規実装必須）

v3 実装は graph.toml / id-map / 設定を `std::fs::write` で直接書く（temp+fsync+rename なし）。SPEC REQ.02（GAP-LGX-152）の「`.tmp.{unix epoch 秒}` へ全量書き出し → fsync → rename(2)」および「DB コミット先行 → 平文確定」の順序制御は**新規実装が必要**で、旧実装に参考コードは無い。SPEC-LGX-002.REQ.13（refresh-subnodes）と方式統一のこと。

### 2.14 [補完] exit code 規約

LGX-COMPAT-001（OLD/docs/legixy_cli_compat_reference.md）のグローバル規約: **使用法誤り exit 2／実行時失敗 exit 1／成功 0**。clap は引数エラーで既定 exit 2 を返すため `--from` 省略時 exit 2（REQ.06 検証方法と整合）。v3 の migrate 実行時エラーは anyhow::bail! 経由の exit 1（V3/crates/lx-cli/src/commands/migrate.rs:43,53 ほか）。

### 2.15 [要決定] REQ.11 マッピング不可 ID の「継続フラグ」名称

SPEC は「継続フラグの名称は DD で定義する」とし、DD-LGX-007 は不在。凍結契約（LGX-COMPAT-001 §4 #2）は migrate の引数を `--from/--to/--dry-run/--format` と定めるため、**追加フラグが凍結契約に抵触しないかの確認も必要**（凍結は「既存引数の意味変更禁止」であり追加可否の解釈は人間判断）。候補: `--skip-unmapped` / `--allow-unmapped`（既定 abort は SPEC どおり）。

### 2.16 [要決定] `--format json` の成功サマリ・レポートのスキーマ

REQ.08 は「スキーマは DD」とし DD-LGX-007 不在。v3 の OutputFormat（markdown|json）実装はあるがサマリ項目は SPEC 0.7.0 で拡張されている（生成/更新ファイル一覧・書換 ID 件数・id-map 参照・バックアップ場所）。最低限のキー集合（例: `files_written[]`, `ids_rewritten_count`, `id_map_path`, `backups[]`, `warnings[]`）を DD で確定する必要がある。

### 2.17 [要決定] ネットワーク FS 判定方法（REQ.12、Step 2）

判定アルゴリズムは SPEC・旧資料とも未規定。Linux では `statfs(2)` の f_type（NFS_SUPER_MAGIC, SMB_SUPER_MAGIC, CIFS_MAGIC_NUMBER 等）による判定が候補。誤検知許容（Warning で継続）は SPEC 規定済。Step 2（Docker 配布、NFR COMPAT.11）まで実装を遅延できる。

### 2.18 [要決定] REQ.13「一度だけ Info を出力」の解釈

legixy は STATE-INV-1（ステートレス、§3 参照）により永続状態を持てないため、「一度だけ」を実行回数で追跡できない。**「1 コマンド実行につき 1 回」の意と解釈するのが整合的**（複数箇所で設定を読んでも重複出力しない）。恒久的に 1 回のみとするなら状態保持が必要で STATE-INV-1 と衝突する。解釈確定は人間判断。

### 2.19 [要決定] 「既に legixy の場合は no-op」（REQ.06）の出力と exit code

未規定。整合的な候補: exit 0 + stderr に Info（「既に legixy 形式、変更なし」）+ stdout に空サマリ（REQ.08 の変更サマリ枠組みで件数 0 を報告）。REQ.03 の空入力（exit 0 + Info）との診断メッセージ区別も必要。

### 2.20 [補完] init の生成物 — v3 実装との差分一覧

v3 init（V3/crates/lx-mig/src/initializer.rs:35-149）と SPEC REQ.07 の差分。**新実装は SPEC が正準**:

| 項目 | v3 実装 | SPEC-LGX-008 REQ.07 |
|------|---------|---------------------|
| 設定ファイル名 | `.trace-engine.toml` | `.legixy.toml`（REQ.13: init は旧名で生成しない） |
| DB ディレクトリ | `.trace-engine/` | `.legixy/` |
| 作成ディレクトリ | 11 個（ICONIX 8 + docs/validation + docs/traceability + .trace-engine） | ICONIX 8 ディレクトリ（docs/specs, usecases, robustness, sequence, detailed-design, test-specs, tests, src）。※graph.toml/engine.db 生成のため docs/traceability/ と .legixy/ の作成は実装上必然。**docs/validation/ は SPEC に無い**（生成しない） |
| matrix.md | ヘッダのみ生成（initializer.rs:116-122） | REQ.07 に記載なし（UC-LGX-009 は空テンプレート生成と記述 — 生成の要否は DD 確定事項） |
| 既存判定 | `.trace-engine.toml` の存在のみ | legixy 管理生成物 4 種（`.legixy.toml` / `.trace-engine.toml` / graph.toml / engine.db）— GAP-LGX-143 |
| --force 退避 | 固定 `.bak`（copy） | REQ.02a 命名（`.bak.{epoch}`） |
| その他 | `.trace-engine/.gitignore`（WAL/bak 除外、initializer.rs:125-132）、graph.toml 空テンプレート、user_version=3 設定 | `.legixy/.gitignore` 相当は SPEC 未言及だが UC-LGX-009 に「.legixy/ ディレクトリ（.gitignore 付き）」とあり踏襲推奨 |

### 2.21 [補完] init テンプレートの完全内容（DD-LGX-007 §3.1.1 の前身）

正準参照先: V3/docs/detailed-design/DD-LX-007_プロジェクト初期化とマイグレーション.md §3.1.1（行 295〜）および実装 V3/crates/lx-core/src/config/loader.rs:352-438（generate_template）。要点:

- `[project] name`、`[graph] file = "docs/traceability/graph.toml"`、`[matrix] format/file/section`
- `[id] pattern = "{type}-{area}-{seq}"`, `area`, `seq_digits = 3`
- `[id.chain] order = ["UC","RB","SEQ","DD","TS","TC","SRC"]`, `independent = ["SPEC","NFR","VAL"]`
- `[id.document_id] pattern = "Document ID:"`（literal prefix 一致。`{id}` プレースホルダを含めると Warning — DD-LX-007 行 322-324）
- `[id.types.SPEC/UC/RB/SEQ/DD/TS/TC/SRC]` 8 セクション（dir / ext / file_pattern = "prefix"|"contains"。TC は tests/ + .rs + contains、SRC は src/ + .rs + contains）
- `[semantic] enabled = false`（model 等の既定値付き）、`[contextual_retrieval] enabled = false`、`[freshness] enabled = true, method = "mtime"`
- `[migration]` セクションは**生成しない**（migrate 時のみ追加）

SPEC REQ.07 が明示するのは「8 typecode の `[id.types.*]` + `[id.document_id]` を含む完全 template」のみであり、上記のその他セクションの既定値は v3 準拠が妥当（DD 再生成時に正準化）。

### 2.22 [補完] REQ.09 バージョン検出 — v3 実装の対応状況と注意

- 一次根拠 `PRAGMA user_version >= 3`、二次判定 `context_log.granularity` カラム有無: V3/crates/lx-cli/src/autodetect.rs:97-110 に実装あり（SPEC の引用どおり）。
- 設定側判定: v3 は `[graph]` **または** `[contextual_retrieval]` の有無で判定（autodetect.rs:77-78）。SPEC は `[graph]` のみ規定 — 実装時は SPEC 準拠とし、`[contextual_retrieval]` を補助とするかは DD 判断。
- マーカ欠落 → v0.1.0: v3 と一致（autodetect.rs:87-93）。
- **矛盾特徴 → Error**: autodetect.rs は矛盾を「部分的 v3 痕跡 = V01」と扱い Error にしない。一方 migrate 経路の lx-mig には VersionMismatch エラーがあり（V3/crates/lx-mig/src/config.rs:127-162、TS-LX-007 T-MIG-CFG-004「version 不整合（config vs db）で VersionMismatch」）、SPEC 0.7.0 の Error 規定はこちらが前身。**autodetect 側の挙動を流用すると SPEC 違反になる**点に注意。

---

## §3 用語・前提の補完

| 用語 | 定義（[補完] 根拠付き） |
|------|------------------------|
| v0.1.0 | 前身ツール traceability-engine の初版データ形式世代。設定 `.trace-engine.toml`（matrix 形式）、`docs/traceability/matrix.md`、`.trace-engine/feedback.db`（user_version=0）で構成。実例: V3/old.source/ 一式 |
| legixy（バージョン名として） | 本 SPEC 群が定義する新世代形式（旧称 v3）。graph.toml + `.legixy/engine.db`（user_version=3）+ `.legixy.toml`。CLI 実行時引数は traceability-engine.v3 バイナリと互換維持の制約（LGX-COMPAT-001） |
| matrix 形式 | v0.1.0 のトレーサビリティ表現: Markdown 表（matrix.md）の各行が成果物チェーン 1 本。`[matrix]` 設定（format/file/section）で所在を指定。legixy 移行後は graph.toml が一次データとなり matrix.md は読み取り専用の派生ビュー（NFR COMPAT.05、LGX-EXT-001 §4.4） |
| chain / independent / typecode | `[id.chain].order` は typecode（UC, RB 等の成果物種別）の連鎖順序。隣接 typecode 間に chain エッジを張る。independent はチェーンに参加しない typecode（ノードのみ）。根拠: V3/crates/lx-mig/src/matrix.rs、OLD/.trace-engine.toml |
| 凍結契約 | LGX-COMPAT-001 §4 に列挙された CLI 引数シグネチャの不変保証（init: `[--force]`、migrate: `--from <PATH>` 必須 / `[--to <PATH>]` / `[--dry-run]` / `[--format markdown|json]`）。変更には人間承認（ハードルール 7） |
| CTX-INV-2 | グラフ整合性: 返される成果物はグラフ定義と矛盾しない（OLD/docs/legixy_foundational_spec.md §10.1） |
| STATE-INV-1 | ステートレス性: legixy は永続的な独自状態を持たず、graph.toml（Git 管理）と engine.db（再生成可能キャッシュ）のみを扱う（同 §10.5） |
| STATE-INV-2 | グラフ不変性: graph.toml の変更は全て Git commit を経由する（同 §10.5）。migrate は commit を自動実行せず、ユーザの確認・commit までが完了（SPEC §4） |
| FB-INV-4 | DB 不在時安全性: DB がなくてもグラフ上流は正常に返される（同 §10.2） |
| SUBNODE-INV-1〜6 | 親存在 / パス整合 / ID 一意 / DAG / 自動生成 ID の決定論性 / 明示 ID（`s:` 接頭辞）と自動生成 ID（16 進ハッシュ）の形式区別（OLD/docs/legixy_subnode_spec_v0.2.1.md §7.2） |
| busy_timeout（REL.07） | SQLite ロック待機上限 5000ms、超過で失敗（無限リトライ禁止）。OLD/docs/nfr/NFR-LGX-001 REL.07、実装 V3/crates/lx-db/src/connection.rs |
| 二層区別（設定ファイル） | REQ.04/13 の対象は「成果物 legixy が生成・読む設定」。本リポジトリの開発運用が旧バイナリで読む `.trace-engine.toml` は DevProc 側の別レイヤで対象外（QSET-LGX-008 Q2 回答 2026-06-07） |
| DevProc 二段化 | RBA/RBD・SEQA/SEQD の抽象/具体分離。本リポジトリの開発運用が設定を上書きして採用する別レイヤであり、init の既定生成物ではない（QSET-LGX-008 Q1 回答 = 選択肢 A、SPP-LGX-008） |
| Confidence（id-map） | 旧→新 ID 対応の確信度。v3 実装では High（衝突なし）/ Warn（桁伸長で回避）の 2 値（V3/crates/lx-mig/src/id_map.rs）。SPEC 0.7.0 では衝突は Error のため意味の再定義が必要（§2.7） |
| GAP / ADR / QSET / SPP / FCR | DevProc_V4.1 の成果物種別: ギャップ分析 / アーキテクチャ決定記録 / 前段ループ質問票 / 仕様パッチ（承認済差分）/ 前段チェック結果。SPEC-LGX-008 の各 REQ の確定経緯はこれらに記録されている（§1 #6〜#10） |

---

## §4 旧実装からの参考情報

### 4.1 クレート対応（SPEC 中の te-* 参照は lx-* に改名済）

SPEC-LGX-008 本文が引用する `crates/te-cli/src/main.rs:66-73`、`crates/te-mig/src/initializer.rs:56-68, 137`、`crates/te-cli/src/autodetect.rs:99` は、現 V3 リポジトリでは **te-→lx-** に改名されている（行番号はほぼ一致を確認済）。

| SPEC-LGX-008 の関心事 | 該当 crate / ファイル（V3 = traceability-engine.v3.chg_to_lexigy） |
|----------------------|------------------------------------------------------|
| migrate / init の CLI 定義（REQ.06/07） | V3/crates/lx-cli/src/main.rs:59-82（Init: force、Migrate: from/to/dry_run/format） |
| migrate コマンド本体・レポート出力 | V3/crates/lx-cli/src/commands/migrate.rs |
| バージョン自動検出（REQ.09） | V3/crates/lx-cli/src/autodetect.rs:69-122 ＋ V3/crates/lx-mig/src/config.rs:127-162（VersionMismatch） |
| init 実装（REQ.07） | V3/crates/lx-mig/src/initializer.rs:35-149、テンプレート本体 V3/crates/lx-core/src/config/loader.rs:352-438 |
| DB 移行（REQ.01/02/03a） | V3/crates/lx-mig/src/db.rs:29-146, 211-355 |
| matrix → graph 変換（REQ.03） | V3/crates/lx-mig/src/matrix.rs:53-245 |
| 設定移行（REQ.04） | V3/crates/lx-mig/src/config.rs:36-99 |
| id-map 生成（REQ.11） | V3/crates/lx-mig/src/id_map.rs:59-157 |
| バックアップ（REQ.02a — ただし非準拠、§2.12） | V3/crates/lx-mig/src/backup.rs:32-115 |
| engine.db スキーマ・PRAGMA（REQ.01/12） | V3/crates/lx-db/src/schema.rs、connection.rs:15-32 |
| エラー型 | V3/crates/lx-mig/src/error.rs（AlreadyExists, SchemaIncompatible, VersionMismatch, BackupFailed 等） |

### 4.2 前身ドキュメント（LGX 系列再生成の種文書）

- DD-LX-007: V3/docs/detailed-design/DD-LX-007_プロジェクト初期化とマイグレーション.md（§3.1.1 init template 全文、§3.2 migrate アルゴリズム、§2.1 REQ 実装箇所表）
- TS-LX-007: V3/docs/test-specs/TS-LX-007_プロジェクト初期化とマイグレーション.md（T-INIT-001〜012、T-MIG-CFG-001〜004、T-MIG-MATRIX-001〜004 ほか — REQ.07 検証方法「TS-LGX-007 §Init.*」の前身）
- VAL-LX-001: V3/docs/validation/VAL-LX-001_外部照合記録.md（Finding E-03 = 行 307、E-05 = 行 333）
- 利用手順: V3/deploy/manual.md §6.1（init）、§6.2 / §12（migrate 手順と処理内容。ただし id-map パスは旧配置 — §2.8 注記）

### 4.3 v0.1.0 実データ（マイグレーションテストの fixture 候補）

- v0.1.0 設定（multi-area 変種）: V3/old.source/.trace-engine.toml（`[id.chains]`/`[id.areas]` 使用 — §2.4 の根拠）
- v0.1.0 DB: V3/old.source/.trace-engine/feedback.db（user_version=0、§2.2 のスキーマ実物）
- v0.1.0 matrix.md: V3/old.source/docs/traceability/matrix.md
- 移行後 graph.toml の実例: V3/docs/traceability/graph.toml（`[[nodes]]` id/type/path、`[[edges]]` from/to/kind 形式。冒頭コメント「migrated from v0.1.0」）
- 単一 chain 形式の v0.1.0 風設定: V3/.trace-engine.toml（`[id.chain]` 単数形の実例）

### 4.4 集計

- §1 未解決参照: **14 項目**（うち実体発見 11、本体不在・前身のみ 3〔DD-LGX-007 / TS-LGX-007 / VAL-LGX-001〕、完全未発見 1〔workflow_2026-04-20〕）
- §2 [補完]: 13 件（2.2, 2.3, 2.5, 2.7, 2.8, 2.9, 2.11, 2.12 前段, 2.13, 2.14, 2.20, 2.21, 2.22）
- §2 [要決定]: 10 件（2.1, 2.4, 2.6, 2.10, 2.12 連番形式, 2.15, 2.16, 2.17, 2.18, 2.19）

（以上）
