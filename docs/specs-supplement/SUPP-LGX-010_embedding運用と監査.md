Document ID: SUPP-LGX-010

# SUPP-LGX-010: SPEC-LGX-010（embedding 運用・監査）実装補完資料

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-010 |
| 対象 SPEC | SPEC-LGX-010「embedding 運用・監査」 Version 0.2.1（Status: Accepted） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書の位置づけ**: 本文書は SPEC 本文の変更ではなく、**実装のための補完情報（参考資料）**である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。本文書の記載と SPEC 本文が矛盾する場合は SPEC 本文が優先する。[補完] は旧リポジトリ（legixy.old.p1 / traceability-engine.v3.chg_to_lexigy）から発見した根拠付きの補完内容、[要決定] は発見できなかった・または人間の判断が必要な論点である。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリ `legixy` には `docs/specs/` の SPEC 10 ファイルのみが存在する。SPEC-LGX-010 が参照する以下の文書は新リポジトリに無く、旧リポジトリで所在を確認した。

| # | 参照 ID / 文書 | SPEC 内参照箇所 | 必要な理由 | 所在（確認済みパス） | 状態 |
|---|---------------|----------------|-----------|---------------------|------|
| 1 | LGX-COMPAT-001（CLI/MCP 互換リファレンス v1.1.0） | ヘッダ表（親文書）、§2、REQ.01/02/03/04/05 | 凍結済み引数契約（§3 グローバルオプション、§4 #5〜#8 のサブコマンド定義、§7 順守チェックリスト）。実装の引数体系の正準 | `legixy.old.p1/docs/legixy_cli_compat_reference.md` | 発見 |
| 2 | NFR-LGX-001（非機能要件） | ヘッダ表、§2、REQ.01 | OBS.02（ログ stderr / 結果 stdout、L141）、OBS.05（終了コード 0/1/2、L144）。ほか OBS.04（エラーメッセージ日本語 primary）、PERF.07（WAL 必須）も実装に関与 | `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md` | 発見 |
| 3 | UC-LGX-010（トレーサビリティ健全性監査） | ヘッダ表、§2、REQ.04 | report の基本/代替フロー・事前事後条件・アクター定義 | `legixy.old.p1/docs/usecases/UC-LGX-010_トレーサビリティ健全性監査.md` | 発見 |
| 4 | UC-LGX-011（閾値キャリブレーション） | ヘッダ表、§2、REQ.05 | calibrate のフロー・「典型的な判断材料」（閾値チューニング指針） | `legixy.old.p1/docs/usecases/UC-LGX-011_閾値キャリブレーション.md` | 発見 |
| 5 | UC-LGX-012（snapshot）/ UC-LGX-013（drift） | ヘッダ表、§1.3、§2 | snapshot / drift の UC。SPEC 自身が「本 SPEC 受理後の UC フェーズで新規生成する」と予告 | **未生成**（旧リポジトリにも存在しない。usecases/ は UC-LGX-001〜011 のみ） | 未生成（プロセス上の予定どおり） |
| 6 | QSET-LGX-001 の回答（2026-06-07） | §1.1、§2、REQ.01/08 | 本 SPEC 新設の根拠（Q1: 未割当サブコマンドのオーナー → 選択肢 B = SPEC-LGX-010 新設、Q2: UC-010/011 の親 SPEC 確定） | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-001_legixy-全体要求.md` | 発見 |
| 7 | QSET-LGX-004 の回答（2026-06-07） | §2、REQ.01/03/04 | Q1: exit 2 = clap 構文層のみ、Q3: drift 二義性の書き分け、Q4: check（判定）/ report（計測）の責務境界 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-004_検証.md` | 発見 |
| 8 | QSET-LGX-006 の回答（2026-06-07） | §2、REQ.03/04/05/09 | Q2: calibrate パーセンタイル方式の正準化、Q4: 次元不一致の「集約 Warning + skip」（report/calibrate）と drift Error の対比 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-006_embeddingとドリフト検出.md` | 発見 |
| 9 | QSET-LGX-010 の回答（2026-06-08） | §2、REQ.02/03/05/07、変更履歴 v0.2.0 | Q1-a〜d（snapshot 系 4 点）、Q2-a〜c（drift 系 3 点）、Q3-a/b（calibrate 系 2 点）、Q4（UC-012/013 新規化）の確定根拠と「申し送り」（DD への指示を含む） | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-010_embedding運用と監査.md` | 発見 |
| 10 | GAP-LGX-185 | REQ.09、変更履歴 v0.2.1 | 非有限スコア（NaN/±Inf）要求の起源・敵対的精査の論拠（clamp が NaN を捕捉できない等） | `legixy.old.p1/docs/gap-analysis/GAP-LGX-185_スコアの特殊浮動小数点値の扱い.md`（closed 2026-06-10） | 発見 |
| 11 | GAP-LGX-186 | §1.3、REQ.03、§4 SCORE-INV-2 行 | model_version 不一致（同一次元）→ exit 1 要求の起源。「次元不一致 Error は補完的検出」への訂正根拠 | `legixy.old.p1/docs/gap-analysis/GAP-LGX-186_同一次元別モデルバージョンのベースライン妥当性.md`（closed 2026-06-10） | 発見 |
| 12 | SPP-LGX-010 / FCR-LGX-010 / SPP-LGX-001（次反復） | §1.3、変更履歴 v0.2.0 | 前段ループの差分案・受理判定。SPEC-LGX-001 網羅宣言「001〜011 → 001〜013」再改訂の処理予定 | `legixy.old.p1/docs/spec-patches/SPP-LGX-010_embedding運用と監査.md`、`legixy.old.p1/docs/frontend-pass/check-results/FCR-LGX-010_embedding運用と監査.md` | 発見 |
| 13 | LEGIXY-SPEC-001（基盤仕様 = 不変条件の定義元） | §4 不変条件表（CTX-INV-1, FB-INV-4, SCORE-INV-1/2, MCP-INV-1 ほか） | 不変条件の正準定義が無いと §4 の「関連/検証/実装」宣言を実装・検証できない（定義は §3 用語補完に転記） | `legixy.old.p1/docs/legixy_foundational_spec.md`（L225, L237, L244, L245, L251） | 発見 |
| 14 | ADR-LGX-007 | （明示参照なし。変更履歴 v0.2.1 の GAP クローズ根拠） | REQ.09 / model_version 照合ポリシーの判断記録（選択肢比較・残存リスク「baseline 側 model_version 未記録の旧データ → SPEC-LGX-008 で補完」） | `legixy.old.p1/docs/adr/ADR-LGX-007_nonfinite-score-and-model-version-policy.md` | 発見 |
| 15 | v3 実測の根拠ソース（`crates/te-cli/...`, `crates/te-embed/...`, `crates/te-db/...`） | §2 末尾、各 REQ 根拠欄 | 「v3 実測の正準化」の検証元。**注意: 旧実装リポジトリでは crate が `te-*` → `lx-*` に改名済み**。SPEC が引用するパスは現存リポジトリでは `crates/lx-cli/...` 等に読み替える（§4 対応表参照） | `traceability-engine.v3.chg_to_lexigy/crates/` | 発見（パス読み替え要） |
| 16 | DD（詳細設計）— snapshot スキーマ・snapshot_id 生成方式・タイブレーク・パーセンタイル式・bulk API シグネチャ等の「DD で凍結/確定」先 | REQ.02/05/06/07/08、ほか多数 | SPEC が多数の実装詳細を DD へ委任している | **未作成**（新旧リポジトリとも `docs/detailed-design/` は空。旧実装側コメントの「DD-LX-005 / DD-LX-007」は旧世代 DD で文書実体は確認できず） | 未生成（v3 実装を §2 で参考情報として補完） |

補足: SPEC-LGX-001 / 004 / 006 への参照は新リポジトリ `docs/specs/` に存在するため解決済み（例: SPEC-LGX-006.REQ.10 の model_version 複合キー・完全一致判定、REQ.11 bulk API は `docs/specs/SPEC-LGX-006_embeddingとドリフト検出.md` L190 以降に現存）。また DevProc 用語の定義元 `docs/DevProc_V4/` 相当は新リポジトリでは `docs/DevPorc/`（ディレクトリ名が typo の可能性。要確認）に存在する。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 共通（4 コマンド横断）

**C-1 [補完] CLI 引数体系（clap 構成）の具体形**
SPEC は LGX-COMPAT-001 §3/§4 #5〜#8 を参照するのみ。正準は:
- グローバル: `--project-root <PATH>`（既定 `.`）/ `--json`（flag）/ `--models-dir <PATH>` / `-h, --help` / `-V, --version`。**サブコマンドの前に指定**（LGX-COMPAT-001 §3）
- `drift <artifact_id> [--against <snapshot:LABEL|snapshot:ID>]`（#5）、`report`（フラグなし、#6）、`calibrate [--buckets <N>]（既定 10）[--recommend]`（#7）、`snapshot create [--label <L>] / list / delete <target>`（#8、target は snapshot_id または `label:<LABEL>`）
- v3 実装: `traceability-engine.v3.chg_to_lexigy/crates/lx-cli/src/main.rs` L35-43（グローバル）、L97-110（Drift/Report/Calibrate）、L250-263（SnapshotAction enum: Create{label}/List/Delete{target}）。clap derive でサブコマンド欠落・未知フラグ・型不正は自動的に exit 2（REQ.01 と整合）

**C-2 [補完] 閾値の数値既定と設定ファイル**
SPEC §1.2 は「閾値の数値既定（→ NFR-LGX-001 / `.legixy.toml`）」をスコープ外とするが、NFR-LGX-001 に数値は無い。正準は LGX-COMPAT-001 §6 の実測スキーマ:
```toml
[semantic]
model = "paraphrase-multilingual-MiniLM-L12-v2"
similarity_threshold = 0.4
drift_threshold = 0.3
link_candidate_threshold = 0.7
```
設定ファイル探索は `.legixy.toml` → `.trace-engine.toml`（旧名フォールバック、SPEC-LGX-008.REQ.13 = 新リポジトリ `docs/specs/SPEC-LGX-008_マイグレーション.md` L217-233）。calibrate の `thresholds`（現在値）はこの 3 キーをそのまま出力する（v3: `crates/lx-cli/src/commands/calibrate.rs` L88-92）。

**C-3 [補完] engine.db の物理パス**
SPEC-LGX-010 は「engine.db」とのみ言う。legixy の正準パスは **`.legixy/engine.db`**（新リポジトリ SPEC-LGX-008、L144/L149「`.legixy/engine.db` を初期スキーマで作成」「既存判定の対象 … `.legixy/engine.db`」）。v3 実測は `<project_root>/.trace-engine/engine.db`（`crates/lx-db/src/connection.rs` L16-20。**ディレクトリと DB を自動作成 + WAL/NORMAL/busy_timeout 5000/foreign_keys PRAGMA + スキーマ初期化**。REQ.07【v3 差分】はこの自動作成を読取系で禁止する点に注意）。

**C-4 [要決定] DB 不在判定の対象と旧ディレクトリ（`.trace-engine/`）フォールバック**
REQ.07 の「engine.db 不在時」を実装するには不在判定の対象パスが必要。SPEC-LGX-008 は設定ファイルの旧名フォールバックは規定するが、**DB ディレクトリ `.trace-engine/engine.db` を legixy が読むか**は明文がない。論点:
- 案 A: `.legixy/engine.db` のみを見る（旧 DB は `migrate` で取り込む前提。シンプル）
- 案 B: `.legixy/engine.db` → `.trace-engine/engine.db` の順でフォールバック読み（既存 v3 プロジェクトの snapshot/report を再 migrate なしで読める。設定ファイルの旧名フォールバック方針と同型）
- いずれも SPEC-LGX-008 側の所管に踏み込むため人間判断が必要（互換制約: legixy CLI は traceability-engine.v3 バイナリと実行時引数互換 — DB パスは引数契約外だが運用互換に影響）

**C-5 [補完] v0.1.0 DB 検出ゲート（autodetect）との接続**
v3 では全コマンドが実行前に `autodetect::gate(project_root, Access)` を通る（`crates/lx-cli/src/autodetect.rs`。Read = check/context/report/calibrate/drift 等、Write = init/migrate/embed 等。snapshot create/delete は Write、list は Read — `crates/lx-cli/src/commands/snapshot.rs` L24/L77/L122）。SPEC-LGX-008.REQ.01（新リポジトリ L47）は「読み取り系コマンドが v0.1.0 engine.db を検出したら Error で `migrate` を促す」と規定しており、本 SPEC の 4 コマンドもこのゲートの対象になる。REQ.07 の「DB 不在 → 空ストア相当」と「v0.1.0 DB 検出 → Error」は別ケースとして実装すること。

**C-6 [補完] 非有限スコア（REQ.09）の防御層は新規実装**
v3 に NaN/±Inf の検査は存在しない。v3 の `cosine_similarity`（`crates/lx-embed/src/drift.rs` L24-43）はゼロノルム時に 0.0 を返す（NaN を一次的に防ぐ）が、SPEC-LGX-006 v0.7.0 ではゼロベクトルは skip + 集約 Warning・cosine 値域 [-1,1] clamp に再定義済み（新リポジトリ SPEC-LGX-006 REQ.04）。実装指針: consumer 側（本 SPEC の 3 読取コマンド）で `f32::is_finite()` 判定を行い、calibrate/report は次元不一致 skip と同経路で skip、drift は exit 1。`--json` のシリアライズ前に非有限値が残らないこと（serde_json は f32 NaN/Inf を null にするか Error にするが、それに依存せず明示検査する）。

**C-7 [要決定] 集約 Warning の文言と `--json` 時の warning 表現**
REQ.04【v3 差分】は「`--json` 時の warning 表現（任意フィールド）は DD で確定」と委任。v3 には集約 Warning 自体が無い（無言 skip: `crates/lx-embed/src/similarity.rs` L76-86, L124, L155）ため参考実装なし。論点: (a) stderr 文言の形式（skip 件数 + 代表理由。例「WARNING: N ペアをスキップしました（端点 embedding 不在 X / 次元不一致 Y / 非有限スコア Z）」）、(b) `--json` の stdout に `"warnings": [...]` 等の任意フィールドを足すか（snapshot create 空ストア時の `warning` フィールドと整合させるか）、(c) SPEC-LGX-006 REQ.04 の bulk API 側 Warning との出力責務分担（API が Warning 情報を返しコマンド層が出力するのが REQ.08「再実装しない」と整合的）。

**C-8 [要決定] 診断メッセージの言語**
NFR-LGX-001.OBS.04 は「エラーメッセージ: 日本語（primary）」。v3 実測は混在（drift INFO は英語 `drift not computable (no baseline embedding; run \`embed --all\` first)`〔`crates/lx-cli/src/commands/drift.rs` L116-124〕、calibrate/report INFO・snapshot WARNING は日本語〔`calibrate.rs` L98、`report.rs` L80、`snapshot.rs` L47/L133/L154〕、snapshot create の JSON `warning` は英語〔`snapshot.rs` L43〕）。日本語へ統一するか v3 文言を維持するかの方針が必要（文言は引数契約外なので互換上は自由だが、`--json` の `warning` フィールド値を機械処理している consumer があれば破壊的）。

### 2.2 snapshot（REQ.02 / REQ.07 関連）

**S-1 [補完] スナップショット領域のテーブル構造（REQ.07「DD で定義」の参考）**
v3 実測（`crates/lx-db/src/schema.rs` L166-179）:
```sql
CREATE TABLE IF NOT EXISTS embedding_snapshots (
    snapshot_id TEXT NOT NULL,
    label TEXT NULL,
    node_id TEXT NOT NULL,
    embedding BLOB NOT NULL,          -- f32 little-endian 直列化（store.rs L298-313）
    embedding_dim INTEGER NOT NULL,
    model_version TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    taken_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (snapshot_id, node_id)
);
CREATE INDEX IF NOT EXISTS idx_snapshots_label ON embedding_snapshots (label);
```
行複製方式: `INSERT INTO embedding_snapshots ... SELECT ?1, ?2, node_id, embedding, embedding_dim, model_version, content_hash FROM embeddings` を単一トランザクションで実行（`crates/lx-embed/src/store.rs` L196-207）。§4 SCORE-INV-1 行の「content_hash / model_version を含む行を複製」はこの SELECT 列に対応する。空ストア時に行 0 件 = 非永続（REQ.02 / QSET-LGX-010 Q1-a の実装的根拠）。

**S-2 [補完] taken_at の形式と精度**
SQLite `datetime('now')` = UTC の `YYYY-MM-DD HH:MM:SS`（**秒精度**）。同一秒内に複数 snapshot を create すると taken_at が衝突するため、REQ.02/REQ.06 の「同時刻タイブレーク」は理論上の話ではなく**現実に発生する**（テストでも再現容易）。DD でタイムゾーン・精度を変える場合は list 出力の `taken_at` フィールド形式が変わる点に注意（v3 互換の観点では文字列形式維持が安全）。

**S-3 [補完] snapshot_id の v3 生成方式（DD 凍結対象の参考）**
`snap-{epoch_ms の 13 桁 16 進}-{8 桁 16 進乱数}`（`crates/lx-cli/src/commands/snapshot.rs` L161-183）。乱数は `Instant`/`SystemTime` のナノ秒 XOR による擬似乱数（`rand_seed()`、外部 crate 不使用）。

**S-4 [要決定] snapshot_id 生成方式の DD 凍結内容**
SPEC は `snap-` プレフィクス + 一意性のみ凍結し生成方式を DD へ委任（QSET-LGX-010 Q1-b で「将来 UUID 化等の自由を残す」と明言）。v3 の `rand_seed()` はエントロピーが弱く、同一ミリ秒内の連続生成で衝突しうる（PRIMARY KEY(snapshot_id, node_id) 違反 → create 失敗）。論点: (a) v3 方式踏襲、(b) UUID v4 / ULID 採用、(c) 衝突時リトライの要否。一意性は SPEC 要求なので、DD では衝突防止の根拠を明記すべき。

**S-5 [要決定] 同時刻 taken_at のタイブレーク規則（DD 確定指示）**
v3 は `ORDER BY taken_at DESC LIMIT 1`（label 解決、`store.rs` L240-251）/ `ORDER BY MAX(taken_at) DESC`（list、`store.rs` L254-259）のみで、同時刻時の順序は SQLite の実装依存 = **非決定**。QSET-LGX-010 Q1-c の申し送り: 「label 解決と list 降順安定出力の**両方に同一規則**（例: snapshot_id の全順序による安定タイブレーク）」。推奨案: `ORDER BY taken_at DESC, snapshot_id DESC`（両 SQL に適用）。最終決定は DD（人間承認は不要だが REQ.02 の指示〔同一規則〕への適合を DD レビューで確認すること）。

**S-6 [補完] `snapshot list` の集計・出力形式**
v3 実測: `SELECT snapshot_id, label, COUNT(*), MAX(taken_at) FROM embedding_snapshots GROUP BY snapshot_id, label ORDER BY MAX(taken_at) DESC`（`store.rs` L254-274）。node_count = 行数、taken_at = MAX(taken_at)。`--json` は `[{snapshot_id, label, node_count, taken_at}]` の配列（pretty print、`snapshot.rs` L87-97。label 無しは null）。text は 0 件時「（スナップショットはありません。`snapshot create` で作成してください）」、1 件以上はヘッダ行 + ハイフン罫線 80 桁 + `{:<32} {:<20} {:>6} {}` の固定幅表（label 無しは `-` 表示、`snapshot.rs` L98-117）。

**S-7 [補完] `snapshot create` 正常時の出力スキーマ（SPEC 未明示）**
REQ.02 は空ストア時の `--json` のみ規定。v3 実測の正常時: `--json` は `{"snapshot_id", "label", "node_count"}`（`warning` なし、`taken_at` なし。`snapshot.rs` L56-64）、text は `snapshot created:` + snapshot_id / label（指定時のみ）/ nodes の 3-4 行（L66-72）。delete 成功時 text は「snapshot '{id}' を削除しました（{n} 行）」、`--json` は件数によらず `{"snapshot_id", "deleted_rows"}`（L145-157）。

**S-8 [要決定] 空ストア create の warning 文言**
REQ.02 は「`warning` 文言には『ストアが空のため永続化されません』を含める（DD で確定）」と指示。v3 実測文言は `"embeddings table is empty; nothing copied. Run \`embed --all\` first."`（JSON、英語）/ text は「WARNING: embeddings テーブルが空です。`embed --all` を先に実行してください」+「snapshot_id = {} は作成されましたが、ノード行は 0 件です」（`snapshot.rs` L43-51）であり、**「永続化されません」の語が無い** → 文言の新規設計が必要（C-8 の言語方針と合わせて DD で確定）。

**S-9 [補完] delete の実装ロジック**
v3 実測（`snapshot.rs` L121-158）: `label:` プレフィクスを strip → `resolve_snapshot_id_by_label`（S-5 の SQL）。解決失敗は ERROR（stderr）+ `std::process::exit(1)`。解決成功または snapshot_id 直接指定なら `DELETE FROM embedding_snapshots WHERE snapshot_id = ?`（単一 Tx、`store.rs` L277-285）。該当 0 行: text = WARNING（stderr）+ exit 0、`--json` = `{"snapshot_id", "deleted_rows": 0}` のみで WARNING 無し（REQ.02 の正準化と一致確認済み）。

### 2.3 drift（REQ.03 関連）

**D-1 [補完] モデル解決の v3 実装と「設定ファイル」解決の実体**
`crates/lx-cli/src/model_dir.rs` L40-72: 優先順 (1) `--models-dir`、(2) 環境変数 `TE_MODELS_DIR`（**空文字列は未設定扱い**）、(3) `<project_root>/models/<config.semantic.model>`（= SPEC の「設定ファイル」解決の実体。`.legixy.toml` の `semantic.model` 名からの既定パス構築であり、models_dir を直接指定する設定キーは存在しない）。相対パスは project_root 基準で絶対化。`model.onnx` / `tokenizer.json` のファイル検査は `Embedder::new` に委譲。失敗時は探索パス一覧を `NotFound::tried` に列挙してエラーメッセージに含める（REQ.03「試行内容を stderr に通知」の実装根拠）。`LGX_MODELS_DIR`（新名）と旧名使用時の stderr Info は **v3 に存在しない新規実装**【v3 差分】。

**D-2 [要決定] 解決順の意味論 — チェーンフォールバックか優先ソース選択か**
v3 は「上位ソースが**指定されていれば**それを使い、指定先が不在なら**即エラー**（下位へフォールバックしない）」（`model_dir.rs` L45-52: `--models-dir` 不在 → `InvalidDir` エラー、env 指定先不在も同様）。SPEC REQ.03 の「`--models-dir` ＞ `LGX_MODELS_DIR` ＞ `TE_MODELS_DIR` ＞ 設定ファイル の順で解決する。全解決失敗…は実行エラー」は両解釈可能。v3 正準化の方針に従えば「優先ソース選択（指定済みソースの不在は即エラー）」だが、`LGX_MODELS_DIR`/`TE_MODELS_DIR` **両方設定時に LGX 優先**という新規定があるため、「LGX 指定先が不在のとき TE へフォールバックするか」は新規の状況で v3 正準が存在しない → 人間判断（推奨: 不在は即エラー。沈黙フォールバックは誤ったモデルでの drift 算出を招く）。

**D-3 [補完] `--against` 解決の実装ロジック**
v3 実測（`crates/lx-cli/src/commands/drift.rs` L50-69）: `snapshot:` プレフィクス必須（欠落は `anyhow::bail!` → exit 1。アプリ層判定なので exit 2 ではない — REQ.03 と一致）。token から `label:` を strip（`snapshot:label:<L>` 明示形式の受理）し、まず `resolve_snapshot_id_by_label` で label 解決、解決できなければ**そのトークンを snapshot_id とみなす**。snapshot に当該 node の行が無ければ `Ok(None)` → INFO（stderr）+ exit 0 + `--json {"artifact_id", "drift": null, "baseline_available": false}`（L106-125。INFO は snapshot 指定時「snapshot '{id}' has no row for this node」、省略時「no baseline embedding; run `embed --all` first」の 2 文言）。

**D-4 [要決定] `snapshot:label:<L>` 明示形式で label が不在の場合の挙動**
SPEC REQ.03 は明示判別形式を「label として解決する」とのみ規定。v3 は明示形式でも label 解決失敗時にトークンを snapshot_id 扱いし、行不在 → INFO + exit 0 になる（D-3）。一方 `snapshot delete label:<L>` の label 解決失敗は ERROR + exit 1（REQ.02）。**明示的に label と宣言した名前参照の解決失敗を exit 0 に落とすのは delete との非対称**であり、QSET-LGX-010 Q1-d の論法（名前解決失敗を覆い隠さない）とも緊張がある。選択肢: (a) v3 正準化（exit 0、REQ.03「スナップショットに当該行なし = exit 0」の一形態とみなす）、(b) 明示 label: 形式に限り解決失敗を exit 1（delete と整合だが【v3 差分】になり SPEC 改訂 = 人間承認が必要）。実装前に確認推奨。

**D-5 [補完] model_version 照合（GAP-LGX-186）は新規実装**
v3 の `compute_node_drift_at` / `compute_node_drift_against_snapshot`（`crates/lx-embed/src/orchestrator.rs` L206-251）に model_version 照合は**存在しない**。実装材料: baseline 側は `embeddings.model_version` / `embedding_snapshots.model_version` 列（`EmbeddingRow.model_version`、`store.rs` L138-161 / L210-238 でロード済み）、現行側は `Embedder` の model_version（v3 では `config.semantic.model` 文字列、`drift.rs` L44。legixy では SPEC-LGX-006.REQ.10 の複合キー〔モデル名 + ONNX 内容ハッシュ + 前処理プロファイル + 次元〕に変わる）。照合は完全一致（SPEC-LGX-006.REQ.10）。不一致時 exit 1 + stderr ERROR（両 version 文字列を提示するのが OBS.04 の趣旨に適う）。残存リスク: 旧 DB に model_version 未記録の行がある場合の扱いは SPEC-LGX-008（マイグレーション）所管（ADR-LGX-007 §4）。

**D-6 [要決定] model_version 照合の適用範囲（`--against` 省略時を含むか）**
REQ.03 の文言は「**ベースライン**保存時の model_version と現行 model_version が異なる場合」で、`--against` 省略時（embeddings 現行保存行がベースライン）も文言上は含まれる。GAP-LGX-186 / ADR-LGX-007 の議論は snapshot 文脈中心。embeddings 行はモデル切替後の `embed --all` 未実行時に旧 model_version を持ちうるため、省略時にも照合を適用するのが SCORE-INV-2 に忠実（推奨）だが、SPEC の意図確認を推奨（適用すると「モデル切替直後・再 embed 前の drift は常に exit 1」という運用挙動になる — これは REQ.03 の原則「壊れた状態を隠さない」に合致）。

**D-7 [補完] 「現行ファイル内容」の読込仕様（サブノード対応）**
`read_current_content_for_node`（`orchestrator.rs` L145-175、REQ.03 が引用する関数）: サブノードは**親ドキュメントのファイル**を読み `content_range` で切り出す（embed 時と同一ロジック。ISSUE-003 BUG-3 fix — これが無いと無変更でも drift > 0 になる）。`read_to_string` の Err 伝播が「現行ファイル欠落 → exit 1」の実装点。content_range 範囲外は clamp、UTF-8 境界不正は全文フォールバック。

**D-8 [補完] 出力フォーマット**
text 正常時: `{artifact_id}: drift = {:.4}`（小数 4 桁）+ snapshot 指定時は `  baseline: snapshot {id}` 行（`drift.rs` L98-103）。`--json` 正常時は REQ.03 記載の 4 フィールド（drift は f32 生値をシリアライズ、丸めなし。`baseline_source` は `"snapshot:<解決済み snapshot_id>"` — label で指定しても **id に解決した値**が入る）。graph.toml 不在ノードは `EmbedError::NodeNotFound` → ERROR + exit 1（「`graph.toml` に登録されているか確認してください」、L127-133）。

### 2.4 report（REQ.04 関連）

**R-1 [補完] bulk API の利用と skip 条件**
v3 実測: `compute_edge_scores`（`crates/lx-embed/src/similarity.rs` L66-96）= graph.toml 全エッジに対し、端点いずれかの embedding 不在（L76-83）または `a.dim != b.dim || a.dim == 0`（L84-86）を skip して cosine 算出。`compute_link_candidates`（L104-141）= 非エッジペア（既存エッジは**無向**で除外 — from↔to 両方向を HashSet 登録、L111-117）のうち score ≥ `config.semantic.link_candidate_threshold` を抽出。SPEC-LGX-006.REQ.11 の操作名・シグネチャは DD 凍結だが v3 の関数名（`compute_edge_scores` / `compute_link_candidates` / `compute_all_pair_scores` / `histogram` / `EmbeddingStore::load_all`）が参考になる。空ストア判定は `load_all().is_empty()`（`report.rs` L62-83）。

**R-2 [補完] 出力順序**
v3 実測: links は `graph.edges()` の**挿入順**（= graph.toml 記載順、`similarity.rs` L65 コメント）、candidates は node_id 昇順ペア (i < j) 順（`load_all` の `ORDER BY node_id ASC`、`store.rs` L163-170 に由来）。REQ.06 は「走査・出力順序は SPEC-LGX-006 REQ.11 の決定性保証に従う」とするのみなので、**順序の正準は DD で凍結**すること（v3 順序の踏襲が安全。graph.toml が同一なら挿入順も決定的であり REQ.06 と矛盾しない）。

**R-3 [補完] summary の計算定義**
v3 実測（`report.rs` L165-186）: `total_links` = 算出対象エッジ数（skip 後 — REQ.04 と一致）、`total_candidates` = 候補数、min/max/mean は **links の score のみ**が対象（candidates は統計に含めない）。links 0 件時は 3 統計とも null（None）。kind 文字列は `"chain" / "custom" / "parent_child"`（L157-163）。

**R-4 [補完] text モードのフォーマット**
v3 実測（`report.rs` L120-153）: `=== Traceability Report: All Links ===` 見出し + `  {from} -> {to} : score={:.4} ({kind})` 行、`=== Link Candidates ===` 見出し + `  {from} <-> {to} : score={:.4}` 行（0 件時「  リンク漏れ候補なし」）、末尾に `  合計: {n} リンク, {m} 候補` + `  リンク類似度統計: min=... / max=... / mean=...`（小数 4 桁）。`--json` は serde_json pretty print。

**R-5 [補完→新規実装] スキップの集約 Warning【v3 差分】**
v3 は無言 skip（R-1 の continue 箇所）。skip 件数・理由を bulk API から返す経路（戻り値拡張 or Warning 構造体）が新規に必要。REQ.08 の「ロジックを再実装しない」原則上、skip 計数は bulk API 側（SPEC-LGX-006 REQ.04 の集約 Warning と共通機構）で行い、コマンド層は出力のみが整合的。文言・JSON 表現は C-7 [要決定] 参照。

### 2.5 calibrate（REQ.05 関連）

**K-1 [補完] ヒストグラムの算出式**
v3 実測（`similarity.rs` L221-244）: `bucket_width = 1.0 / N`、`clamped = score.clamp(0.0, 1.0)`、`idx = clamped >= 1.0 ? N-1 : min(floor(clamped / bucket_width), N-1)`。バケットは `{low: i*width, high: low+width, count}`。テスト（L339-380）が境界挙動（1.0 は末尾、域外 clamp、空入力で全バケット 0 件の N 配列）を固定している。min/max/mean は clamp **前**の生値（`calibrate.rs` L110-118: 全スコアをソートし first/last/算術平均 — REQ.05 と一致）。

**K-2 [補完] パーセンタイル算出式（DD 凍結対象、QSET-LGX-010 Q3-a）**
v3 実測（`calibrate.rs` L216-242 `compute_recommended`）: 昇順ソート済み配列に対し `idx = round((n-1) * frac)` の nearest-rank 変種、`sorted[min(idx, n-1)]`。p10/p25/p50/p75/p90 を算出し、`similarity_threshold = p25`、`drift_threshold = 1.0 - p90`、`link_candidate_threshold = p75`（SPEC の写像と一致）。REQ.05 検証方法の「既知分布 fixture に対する推奨値一致テスト（nearest-rank 式前提）」はこの式を前提とする。

**K-3 [補完] `--json` の実測スキーマ（SPEC 記載との差分あり）**
v3 実測（`calibrate.rs` L16-54）: SPEC 記載の `{pairs, min, max, mean, distribution[], thresholds{}}` に加え、`--recommend` 時の `recommended_thresholds` には **p10/p25/p50/p75/p90 と `note` 文字列**が含まれる（`note = "Phase 1 ノード単位 embedding ベース。Phase 2 サブノード化後に再算出推奨"`）。SPEC REQ.05 は「参考情報として p10〜p90 を併記する」と規定（→ JSON にパーセンタイル群を含めるのは SPEC 整合）。空ストア時は pairs=0・min/max/mean=null・distribution=[]・thresholds は現在値・recommended_thresholds キー自体なし（`#[serde(skip_serializing_if)]`、L38-39, L80-101）。

**K-4 [要決定] `note` フィールドと text の ISSUE-005 注記の扱い**
v3 の `note`（K-3）および text モード末尾の「テンプレ寄与」注記（`calibrate.rs` L208-210。背景は ISSUE-005: ノード単位 embedding ではテンプレ相似がスコアを底上げする）は Phase 1 時代の文言。legixy はサブノード embedding 既定（SPEC-LGX-006 REQ.09/12）なので文言が陳腐化している。選択肢: (a) note 削除（JSON 消費側がフィールド存在に依存していなければ互換安全）、(b) 文言更新して維持。SPEC のスキーマ凍結範囲（note は SPEC 非記載 = 任意フィールド）の解釈確認を含め DD で確定。

**K-5 [補完] text モードのフォーマット**
v3 実測（`calibrate.rs` L149-211）: `=== Similarity Distribution ===` + ペア数 + 最小/最大/平均（4 桁）+ 罫線付きヒストグラム表（`[{:.2}, {:.2})   | {:>4} | {bar}`、バーは `#` で最大カウントを 40 文字に正規化）+ 現閾値 3 行（2 桁）+ `--recommend` 時は推奨値 3 行（現在値と算出根拠パーセンタイルを併記）+ percentiles 1 行。

**K-6 [補完] `--buckets 0` のエラー**
v3 実測: `anyhow::bail!("--buckets は 1 以上を指定してください")` → exit 1（`calibrate.rs` L64-67）。clap 層の型不正（非整数・負数で usize パース失敗）は exit 2 で、REQ.01 の分類と一致。pairs=0 + `--recommend` 時の INFO（REQ.05【v3 差分】「ペア数 0 のため推奨値は算出されません」）は新規実装（v3 は `recommend && !score_vec.is_empty()` で無言省略、L121-125）。

---

## §3 用語・前提の補完

| 用語 | SPEC 内での扱い | 補完（定義と出典） |
|------|----------------|-------------------|
| CTX-INV-1 | §4 で参照のみ | 「決定論保証: 同じ入力に対して常に同じコンテキスト結果を返す」（legixy_foundational_spec.md L225） |
| FB-INV-4 | §4 / REQ.07 で参照のみ | 「DB 不在時安全性: DB がなくてもグラフ上流は正常に返される」（同 L237） |
| SCORE-INV-1 | §4 / REQ.06 で参照のみ | 「ハッシュ一致保証: ノードのハッシュが一致するスコアのみ fresh とする」（同 L244） |
| SCORE-INV-2 | §4 / REQ.03 で参照のみ | 「モデルバージョン一致: 現在のモデルバージョンと一致するスコアのみ有効」（同 L245） |
| MCP-INV-1 | REQ.01 で参照のみ | 「Agent Surface 限定: MCP は compile_context, observe, get_compile_audit の 3 ツールのみ」（同 L251）。4 コマンドを MCP に**追加しない**ことがこの不変条件の維持 |
| Admin Surface / Agent Surface | REQ.01 で前提 | SPEC-LGX-001 REQ.08（新リポジトリに現存）の Surface 分離。Admin = 人間運用者向け CLI、Agent = MCP 3 ツール |
| 前段ループ / QSET / SPP / FCR / TP / GAP / DD / ハードルール 9 | §1/§5 で前提 | DevProc_V4.1 の成果物タイプ・プロセス用語。定義は新リポジトリ `docs/DevPorc/`（02-typecodes.md, 03a-frontend-pass.md 等。**ディレクトリ名 DevPorc は typo の疑い**）および旧 `legixy.old.p1/CLAUDE.md`。ハードルール 9 = 「SPEC は FCR ACCEPTED 到達まで TP[SPEC]/UC 着手禁止」 |
| 「v3 実測」「v0.4.0-alpha4」 | §1.1/§2 で前提 | 旧 `traceability-engine` バイナリの実測挙動。検証元ソースは `traceability-engine.v3.chg_to_lexigy/`。**SPEC が引用する `crates/te-cli/...` 等のパスは現リポジトリでは `crates/lx-cli/...` に改名済み**（§4 対応表） |
| bulk similarity API | REQ.08 で前提 | SPEC-LGX-006.REQ.11（新リポジトリに現存）。「全エッジスコア算出 / リンク候補抽出 / 全ペアスコア算出 / ヒストグラム集計 / 決定論的全件ロード」の 5 操作。v3 実装名は §2 R-1 参照 |
| model_version（完全一致判定） | REQ.03 で参照 | SPEC-LGX-006.REQ.10（新リポジトリ L190-193 ほか）: モデル名 + ONNX 内容ハッシュ + 前処理プロファイル + 次元の複合キー、文字列完全一致。v3 実測ではモデル名文字列のみだった点に注意（D-5） |
| drift 値域 [0.0, 2.0] | REQ.03 で定義 | cosine 値域 [-1, 1]（SPEC-LGX-006 REQ.04、GAP-LGX-105 対応で確定）から `1.0 − cosine` ∈ [0, 2]。calibrate の clamp [0,1] は「負の cosine は正常出力」を前提とする（SPEC-LGX-006 REQ.04 が明文化） |
| link_candidate_threshold 等 3 閾値 | §1.2 でスコープ外宣言 | 既定値 0.4 / 0.3 / 0.7、設定キーは `[semantic]`（§2 C-2、LGX-COMPAT-001 §6） |
| graph.toml / embeddings ストア / engine.db | 各 REQ で前提 | graph.toml = `config.graph.file`（既定 `docs/traceability/graph.toml`）。embeddings ストア = engine.db の `embeddings` テーブル（スキーマは `lx-db/src/schema.rs` L37-49）。engine.db = `.legixy/engine.db`（§2 C-3） |
| 「3 軸比較」 | REQ.01/REQ.07 の採用根拠 | 整合性・堅牢性・実現性の 3 観点比較。QSET-LGX-010 の各回答が実例（前段ループの定型判断手法） |

---

## §4 旧実装からの参考情報

### 4.1 crate 対応表（SPEC 引用パスの読み替え）

旧実装リポジトリ: `traceability-engine.v3.chg_to_lexigy/`（自己呼称は途中改名の「lexigy」。crate prefix は `lx-`）

| SPEC / COMPAT 上の名称 | 現存パス |
|------------------------|----------|
| te-cli | `crates/lx-cli/` |
| te-embed | `crates/lx-embed/` |
| te-db | `crates/lx-db/` |
| te-core / te-graph / ほか | `crates/lx-core/` `crates/lx-graph/` 等（lx-check, lx-ctx, lx-feedback, lx-nav, lx-mig） |

注: legixy の crate 分割は SPEC で凍結されない（QSET-LGX-001 Q3 回答: crate 名は DD で確定し SPEC では抽象化）。

### 4.2 本 SPEC 関連の主要ファイル

| 役割 | パス（旧実装リポジトリ基準） |
|------|------------------------------|
| snapshot コマンド | `crates/lx-cli/src/commands/snapshot.rs`（create/list/delete、snapshot_id 生成） |
| drift コマンド | `crates/lx-cli/src/commands/drift.rs`（--against 解決、出力、exit code） |
| report コマンド | `crates/lx-cli/src/commands/report.rs`（links/candidates/summary、text/JSON） |
| calibrate コマンド | `crates/lx-cli/src/commands/calibrate.rs`（histogram、--recommend パーセンタイル） |
| CLI 定義（clap） | `crates/lx-cli/src/main.rs`（グローバルオプション L35-43、SnapshotAction L250-263） |
| モデルディレクトリ解決 | `crates/lx-cli/src/model_dir.rs`（TE_MODELS_DIR、探索パス列挙エラー） |
| v0.1.0 DB ゲート | `crates/lx-cli/src/autodetect.rs`（Read/Write アクセス分類） |
| bulk similarity API | `crates/lx-embed/src/similarity.rs`（compute_edge_scores / compute_link_candidates / compute_all_pair_scores / histogram + 境界テスト） |
| drift 計算・現行内容読込 | `crates/lx-embed/src/orchestrator.rs`（compute_node_drift_at / compute_node_drift_against_snapshot / read_current_content_for_node） |
| cosine / DriftCalculator | `crates/lx-embed/src/drift.rs` |
| EmbeddingStore（snapshot API 含む） | `crates/lx-embed/src/store.rs`（create/list/delete/resolve_snapshot_id_by_label/load_snapshot_embedding、f32⇄bytes 直列化） |
| DB スキーマ（embedding_snapshots） | `crates/lx-db/src/schema.rs` L163-179 |
| DB 接続（自動作成 — REQ.07 で禁止される旧挙動） | `crates/lx-db/src/connection.rs` |

### 4.3 関連 issue（要求の起源・既知問題）

| Issue | 内容 | 本 SPEC との関係 |
|-------|------|------------------|
| `issues/ISSUE-002_drift-baseline-management.md` | drift baseline が embed --all で上書きされ定点観測不能 → embedding_snapshots テーブルと snapshot コマンド群を導入 | REQ.02/REQ.03 の機能起源。スキーマ案・コマンド案の原文 |
| `issues/ISSUE-004_calibrate-threshold-recommendation.md` | calibrate に推奨閾値自動算出（--recommend）を追加 | REQ.05 パーセンタイル方式の起源 |
| `issues/ISSUE-005_template-similarity-noise-floor.md` | ノード単位 embedding のテンプレ相似で類似度が底上げ（vnstudio 実測 mean≈0.68） | calibrate text の注記文言（§2 K-4）と閾値解釈の背景。legixy ではサブノード embedding（SPEC-LGX-006 REQ.09）が対策 |
| `issues/ISSUE-001_semantic-id-redefinition-detection.md` / `ISSUE-003_vector-store-config-stale.md` | 粒度問題 / BUG-2（スキーマ migration 順序）・BUG-3（drift がサブノード content_range を無視 → 無変更で drift>0） | BUG-3 fix は REQ.03「現行ファイル内容から生成」の実装上の前提（§2 D-7）。BUG-2 は schema initialize の ALTER 順序制約 |

### 4.4 その他

- `deploy/manual.md`（旧実装リポジトリ）: 運用マニュアル。drift/snapshot の運用手順の参考。
- ONNX モデル: `models/paraphrase-multilingual-MiniLM-L12-v2/`（旧開発リポジトリに配置実績。`model.onnx` + `tokenizer.json` が必須ファイル — `model_dir.rs` のエラーメッセージより）。
- 旧リポジトリ docs（`legixy.old.p1/docs/`）には本 SPEC の検証材料となる `test-perspectives/TP-LGX-010`（GAP-185/186 の親）、`acceptance-tests/`、`traceability/graph.toml` も存在する。新リポジトリへの文書移管範囲は人間判断事項。

---

## 集計

- §1 未解決参照: 16 件（発見 13 件、未生成 3 件 = UC-LGX-012/013、DD、※#12 は 3 文書を 1 行に集約）
- §2 [補完]: 26 件（C-1/2/3/5/6、S-1/2/3/6/7/9、D-1/3/5/7/8、R-1/2/3/4/5、K-1/2/3/5/6）
- §2 [要決定]: 10 件（C-4/7/8、S-4/5/8、D-2/4/6、K-4）

（本文書は AI 生成・非正準。人間査読の上、SPEC 改訂が必要な項目は SPP / spec-change-proposals 経由で処理すること）
