Document ID: SUPP-LGX-005

# SUPP-LGX-005: SPEC-LGX-005（グラフ走査）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-005 |
| 対象 SPEC | SPEC-LGX-005 グラフ走査 v0.4.0（Approved, 2026-06-10） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
>
> 補完の根拠は主に旧文書群（`legixy.old.p1/docs/`）と旧実装（`traceability-engine.v3.chg_to_lexigy/`、crate 名は te-* → lx-* に改名済）から実地に採取した。SPEC 中の `crates/te-graph/...` 等の v3 ソース参照は、旧実装リポジトリでは `crates/lx-graph/...` に対応する。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリ（legixy）には docs/specs/ の SPEC 10 件のみが存在する。SPEC-LGX-005 が参照する以下の文書は新リポジトリに存在しない。所在を確認できたものはパスを記す。

| # | 文書 ID | SPEC 内の参照箇所 | 必要な理由 | 所在（確認済） |
|---|---------|------------------|-----------|----------------|
| 1 | UC-LGX-005（逆方向探索） | ヘッダ表「対応 UC」、§2、REQ.01/02/04/07/09 根拠 | investigate の基本フロー・代替フロー（drift 閾値、embedding 未生成時）の定義 | `legixy.old.p1/docs/usecases/UC-LGX-005_逆方向探索.md` |
| 2 | UC-LGX-006（順方向探索） | 同上 | impact の基本フロー・代替フロー（max-depth 未指定、存在しない起点）の定義 | `legixy.old.p1/docs/usecases/UC-LGX-006_順方向探索.md` |
| 3 | LEGIXY-SPEC-001（基盤仕様） | §1.1、REQ.03 根拠（§10 CTX-INV-1） | 不変条件 CTX-INV-1〜4 の正準定義（§10）、impact/investigate のエンジン機能位置づけ（§4, §5） | `legixy.old.p1/docs/legixy_foundational_spec.md`（CTX-INV 表は L225 付近 §10.1） |
| 4 | LGX-EXT-001（サブノード仕様 v0.2.1） | §1.1、REQ.10 根拠（§6.1） | MCP-INV-1（Agent Surface 限定 = MCP は compile_context / observe / get_compile_audit の 3 ツールのみ）、SUBNODE-INV-4（DAG 制約、§7.1） | `legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md`（§6.1 は L505 以降、SUBNODE-INV-4 は L615） |
| 5 | LGX-COMPAT-001（CLI 互換リファレンス v1.1.0） | REQ.04「互換対象 (d) 既定値」、REQ.09「引数体系 (a)〜(f)」 | 互換対象の定義 (a)サブコマンド名 (b)位置引数 (c)フラグ名と値 (d)既定値 (e)終了コード (f)MCP 3 ツール。§4 #11/#12 が `impact <start> [--max-depth <N>]` / `investigate <start> [--max-depth <N>]` を凍結 | `legixy.old.p1/docs/legixy_cli_compat_reference.md` |
| 6 | NFR-LGX-001（非機能要件） | ヘッダ表「対応 NFR」NFR-LGX-001.REL.05 | REL.05 = 「BFS 走査決定性（CTX-INV-1）: 同一グラフ・同一起点から常に同一 visited 順。検証 TS-LGX-001 T-GT-005」（L157）。stderr Info の出力先規律は OBS.02（結果は stdout、ログは stderr、L141） | `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md` |
| 7 | GAP-LGX-081 | REQ.01「方向の一般則（GAP-LGX-081 対応）」 | custom エッジ方向セマンティクスの裁定経緯（closed 2026-06-10、双方向特例なし） | `legixy.old.p1/docs/gap-analysis/GAP-LGX-081_custom-エッジの方向セマンティクス.md` |
| 8 | GAP-LGX-085 | REQ.04「打ち切りの可観測性（GAP-LGX-085 対応）」 | stderr Info 採用の裁定経緯と DD への truncated フラグ申し送り | `legixy.old.p1/docs/gap-analysis/GAP-LGX-085_max_depth打ち切りの可観測性.md` |
| 9 | QSET-LGX-005 | REQ.04/REQ.09 根拠「QSET-LGX-005 Q1/Q2 回答」 | max_depth 既定 = 無制限（Q1）、--json 機能化（Q2）の開発者回答原文と v3 実測根拠 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-005_グラフ走査.md` |
| 10 | SPP-LGX-005 | §5 変更履歴 0.3.0 行 | v0.2.0→v0.3.0 差分の承認記録（承認 2026-06-07） | `legixy.old.p1/docs/spec-patches/SPP-LGX-005_グラフ走査.md` |
| 11 | TS-LGX-001（T-GT-001〜005） | 各 REQ の検証方法 | 走査テストの具体内容。**legixy 版 TS-LGX-001 は未作成**（legixy.old.p1/docs/test-specs/ は空）。旧 v3 の `TS-LX-001` §6（L183-208）に T-GT-001〜005 の同名テストが存在し、内容を §2 に転記 | `traceability-engine.v3.chg_to_lexigy/docs/test-specs/TS-LX-001_グラフ基盤.md` §6 |
| 12 | 対応 DD（§1.2「実装詳細は対応 DD で定義」、REQ.09「JSON スキーマは DD 凍結対象」） | §1.2、REQ.04、REQ.09 | **legixy 版 DD は未作成**（legixy.old.p1/docs/detailed-design/ は空）。前身は旧 v3 の `DD-LX-004_グラフ走査.md`（クレート構成・公開 API・アルゴリズム・Reporter を規定） | `traceability-engine.v3.chg_to_lexigy/docs/detailed-design/DD-LX-004_グラフ走査.md` |

**新リポジトリ内で解決できる参照（問題なし）:** SPEC-LGX-001（REQ.08 Admin/Agent Surface 分離、§7.1 変更承認）、SPEC-LGX-002（REQ.04 エッジ 3 種別、REQ.08 IndexMap 決定論）、SPEC-LGX-003.REQ.05（CTX-INV-3 所有）、SPEC-LGX-004 REQ.08、SPEC-LGX-010 REQ.01 は docs/specs/ に存在する。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### §2.1 走査アルゴリズムの詳細

**[補完] BFS の具体構造と max_depth 判定**
根拠: `traceability-engine.v3.chg_to_lexigy/crates/lx-graph/src/traversal.rs`（全 89 行、DD-LX-001 §3.9 由来）

- 単起点 BFS。`VecDeque<(NodeId, usize)>` キューに (ノード, 深度) を積む。起点は深度 0 で visited / depth_map に即時登録。
- `max_depth = Some(limit)` のとき、キューから取り出したノードの `depth >= limit` なら**そのノードの隣接展開をスキップ**する（traversal.rs:53-58。SPEC 引用の te-graph/src/traversal.rs:54-58 に対応）。つまり深度 limit のノード自体は返すが、その先を辿らない。
- 順方向は `graph.outgoing(&current)`、逆方向は `graph.incoming(&current)` のエッジ index 列を使い、Forward は `edge.to`、Reverse は `edge.from` を次ノードとする（REQ.01 の from→to 一般則と一致）。
- エッジ種別（chain/custom/parent_child）による絞り込みは行わない = 全種別を辿る（REQ.01/02/08 と一致）。

**[補完] 決定論性の実現機構**
根拠: 同 traversal.rs + `lx-graph/src/model.rs:63,129-137`

- visited 判定自体は `HashSet` だが、**隣接展開順はノード格納 `IndexMap<NodeId, Node>`（graph.toml 記載順）と、エッジ配列に対する outgoing/incoming index リスト（エッジ定義順）で決まる**ため、同一入力で常に同一順となる。REQ.03 の「IndexMap の挿入順」はこの機構を指す（SPEC-LGX-002.REQ.08 が順序保持 TOML パーサを要求）。
- visited 出力順・depth_map 挿入順は発見順（BFS 順）。

**[補完] 循環グラフでの停止保証（REQ.06）**
根拠: traversal.rs:75（`visited.insert` が false なら enqueue しない）、旧 TS-LX-004 T-MT-006「循環グラフでも無限ループせず visited が有限集合になる」

**[補完] 多起点走査のマージ規則（ライブラリ層）**
根拠: `lx-nav/src/multi_traverser.rs`（DD-LX-004 §3.1）

- 起点ごとに単起点 BFS を実行し、visited は「start_ids の入力順 × 各起点の BFS 順」でマージ（既出ノードは追加しない）。
- 複数起点から到達できるノードの depth は **min(depth)** で記録（`lx-nav/src/result.rs:22-26`）。
- 存在しない起点は読み飛ばして他起点を継続（REQ.05 のライブラリ層実装。旧 TS-LX-004 T-MT-005）。
- 注: v3 CLI は位置引数 1 個（`<start>`）のため CLI からは常に単起点。UC-LGX-005/006 の `<start_ids>`（複数形）はライブラリ API 層の語彙であり、**CLI で複数起点を受理する変更は LGX-COMPAT-001 互換対象 (b) 違反となるため不可**。

**[補完] edges_traversed の意味（REQ.09「走査で使用したエッジの情報」）**
根拠: traversal.rs:71-84

- v3 は**初訪問を生んだエッジのみ**（BFS spanning tree）を `(from, to)` グラフ向きペアで記録する。既訪問ノードに到達するエッジは記録しない（コードコメント「Skipping keeps edges_traversed aligned with the spanning tree of first-visits only」）。
- 逆方向走査でもペアはグラフ定義の (from, to) 向きのまま。
- なお v3 の Text / JSON 出力には edges_traversed は**含まれない**（内部結果型のみ保持）。REQ.09 の「使用したエッジの情報」を出力へ含めるか否かは DD 凍結対象 → [要決定]（§2.3 参照）。

### §2.2 結果データ構造

**[補完] v3 の結果型（DD の出発点として）**
根拠: `lx-graph/src/traversal.rs:6-11`、`lx-nav/src/result.rs`（DD-LX-004 §4）

- `TraversalResult { visited: Vec<NodeId>, edges_traversed: Vec<(NodeId,NodeId)>, depth_map: HashMap<NodeId,usize> }`（lx-graph 層）
- `VisitedNode { id, type_code: String, path: String, depth: usize }`
- `MultiTraversalResult { visited: Vec<VisitedNode>, edges_traversed: Vec<(NodeId,NodeId)>, depth_map: IndexMap<NodeId,usize>, start_ids: Vec<NodeId> }`
- `PrunedTraversalResult { traversal: MultiTraversalResult, suspicious_nodes: Vec<SuspiciousNode>, drift_threshold: f32 }`
- `SuspiciousNode { id, drift_score: f32, type_code, path }`。suspicious_nodes は **drift_score 降順、同値は NodeId 昇順**（`lx-nav/src/drift_pruner.rs:45-50`）— REQ.03 の決定論性を suspicious 出力にも及ぼす整列規則。

### §2.3 出力フォーマット

**[補完] Text 出力の正確な書式（REQ.09「v3 互換」の実体）**
根拠: `lx-nav/src/reporter.rs`（DD-LX-004 §3.5。SPEC 引用の te-nav/src/reporter.rs:87-102 に対応）

- impact（visited 各行）: `{id} (type={type}, depth={depth}, path={path})`、末尾に `Summary: visited={n}`
- investigate: visited 各行（同上）→ `Suspicious nodes (drift_threshold={t}):` → 各行 `{id} (drift={score}, type={type}, path={path})` → `Summary: visited={n}, suspicious={m}`

**[補完→DD 申し送り] JSON 出力の v3 既存実装**
根拠: reporter.rs `ReportFormat::JsonLines` 分岐

- v3 の reporter には JSON Lines 実装が既にある（ただし CLI から未配線 = SPEC の言う「受理するが無視」）: visited 1 行 = `{"id","type","depth","path"}`、suspicious 1 行 = `{"suspicious":{"id","drift","type","path"}}`、末尾 1 行 = `{"summary":{"visited":n[, "suspicious":m, "drift_threshold":t]}}`。
- f32→JSON 変換は Display 経由の丸め（`f32_to_clean_f64`、reporter.rs:124-126）で `0.3` のような短い表現を保つ。drift 値の JSON 表現方針として DD に転記推奨。

**[要決定] --json の構造（DD 凍結対象）**
論点: REQ.09 は「構造化 JSON」とのみ規定。選択肢:
- (A) v3 reporter の JSON Lines をそのまま機能化（実装最小、SPEC-LGX-004 REQ.08 check の JSON Lines と整合）
- (B) 単一 JSON ドキュメント（start_ids / visited[] / edges[] / depth_map / summary を 1 オブジェクトに）
さらに (i) truncated フラグ・除外件数を含めるか（GAP-LGX-085 申し送り）、(ii) edges_traversed を含めるか（REQ.09 は「使用したエッジの情報」を結果に含むと規定するが v3 出力には無い）、(iii) start_ids を含めるか。JSON スキーマは DD 凍結対象（ハードルール 7）なので人間承認の凍結リストに載せる必要がある。

### §2.4 CLI インターフェースの詳細

**[補完] 引数・終了コードの互換契約**
根拠: `legixy.old.p1/docs/legixy_cli_compat_reference.md` §3, §4 #11/#12

- `impact <start> [--max-depth <N>]` / `investigate <start> [--max-depth <N>]`。位置引数は 1 個。グローバル: `--project-root <PATH>`（既定 `.`）、`--json`、`--models-dir`。
- 終了コード規約（§3 グローバル規約、v1.0.1）: 使用法誤り（clap 構文エラー）= exit 2、実行時失敗 = exit 1、正常 = 0。打ち切り Info 出力時も exit 0（REQ.04「終了コードは不変」）。
- 設定ファイル名は `.trace-engine.toml`（互換レイヤ）、graph ファイルパスは `config.graph.file`（`lx-cli/src/commands/impact.rs:15-18`）。

**[補完] 存在しない起点の CLI 挙動（REQ.05）**
根拠: `lx-cli/src/commands/impact.rs`、traversal.rs:42-44

- v3 実測: contains_node 不成立 → 空 visited → stdout に `Summary: visited=0` のみ、**exit 0**。
- [要決定] UC-LGX-006 代替フロー 2a は「起点が存在しない場合 ERROR を報告する」と書くが、SPEC-LGX-005.REQ.05 は「空の結果（エラーではない）」と規定し v3 実測とも一致する。SPEC が後勝ち（v0.2.0 承認 > UC）と解するのが自然だが、UC 2a の改訂（または stderr への Warning 追加の要否）は人間判断。

**[要決定] investigate の --max-depth は機能するか**
論点: v3 は investigate でも `--max-depth` を**受理するが無視**する（`lx-cli/src/commands/investigate.rs:12` で `_max_depth` 未使用、`lx-nav/src/investigate.rs:24` で常に `None` 渡し）。一方 SPEC REQ.07 は「どちらも `--max-depth N` オプションを受け付ける」とし、REQ.04 は走査一般に max_depth 制御を規定する。選択肢:
- (A) investigate でも機能化（SPEC の素直な読み。REQ.09 の --json 機能化と同型の「受理済みフラグの機能化」であり引数体系不変。打ち切り stderr Info も investigate に適用）
- (B) v3 挙動維持（無視）— SPEC REQ.04/07 との不整合が残る
v3 差分注記が SPEC に無いため、(A) を採るなら SPEC への【v3 差分】追記（人間承認）を推奨。

**[要決定] investigate の drift スコア参照の CLI 配線**
論点: v3 CLI は `investigate(&graph, ..., db: None, drift_threshold)` と **engine.db を常に開かず**、suspicious_nodes は恒常的に空（`lx-cli/src/commands/investigate.rs:21`）。一方 UC-LGX-005 基本フロー 3-4 と SPEC REQ.09（「suspicious nodes は drift 値を含む」）は drift 参照を前提とする。ライブラリ層（lx-nav DriftPruner）は db=Some 対応済みで、scores テーブルから `score_type='drift'` の MAX(value) を引く実装が存在する（drift_pruner.rs:67-111。DB 照会失敗時は stderr Warning 1 行で空 suspicious 継続 = NFR REL.02）。legixy では CLI 層で engine.db を開いて Some を渡す配線が必要と思われるが、これは v3 観測挙動の変更（suspicious が出るようになる）なので人間確認を推奨。

**[要決定] --drift-threshold フラグの追加可否**
論点: UC-LGX-005 基本フロー 1 は `[--drift-threshold <val>]` を記すが、v3 バイナリにも LGX-COMPAT-001 §4 #12 にも当該フラグは無く、SPEC REQ.07 にも無い。v3 は設定ファイル `[semantic] drift_threshold`（既定 **0.3**、`lx-core/src/config/loader.rs:236,246`）のみを使う。フラグ追加は加算的拡張として可能だが、embed `--node`/`--force` の前例（LGX-COMPAT-001 v1.1.0、ADR-LGX-002）に倣い **SPEC 改訂 + 人間承認 + ADR 記録**が必要。当面は設定ファイル経由のみとするのが安全。

### §2.5 打ち切り可観測性（REQ.04 stderr Info、v3 前例なし）

**[補完] 出力規律**: stderr へ Info、stdout は結果のみ — NFR-LGX-001.OBS.02（結果 stdout / ログ stderr）に従う。DriftPruner の既存 Warning 形式 `[nav] drift pruning skipped: {e}`（drift_pruner.rs:54）がプレフィックス前例。

**[要決定] Info の文言と「除外ノード件数」の算定法**
v3 は無言打ち切りのため前例がない【v3 差分】。論点:
- 件数の定義: (a) max_depth 超で到達可能だった未訪問ノードの総数（境界を越えて visited 制御付き探索を継続して数える必要があり、走査コストが「打ち切らない場合」と同等になる）、(b) 深度境界ノードから出る未訪問隣接ノード数（安価だが「深度 limit+1 の近似」にすぎない）。GAP-LGX-085 裁定文は「除外ノード件数」とのみ言う。
- メッセージ書式（例: `[nav] info: max-depth {N} truncated traversal; {k} reachable node(s) excluded`）と多起点（ライブラリ層）時の集計。
- 件数算定法は到達集合計算に影響しない（stdout 不変）が、性能特性に影響するため DD で確定し、--json の truncated フラグ表現（§2.3）と同時に凍結することを推奨。

### §2.6 検証（テスト）内容の補完

**[補完] T-GT-001〜005 の内容**（legixy 版 TS-LGX-001 未作成のため、旧 v3 `TS-LX-001` §6 L183-208 を転記）
- T-GT-001: 線形チェーン UC→DD→TS→TC→SRC、起点 UC → visited = [UC,DD,TS,TC,SRC]（BFS 順）。※legixy 版では REQ.01 検証方法の指示により「custom エッジを含む fixture で from→to 方向のみ到達」の確認を追加する必要がある（旧版に無い）
- T-GT-002: 同チェーン起点 SRC、逆方向 → visited = [SRC,TC,TS,DD,UC]
- T-GT-003: 起点 UC、max_depth=2 → visited = [UC,DD,TS]（3 ノードのみ）
- T-GT-004: 起点 "NONEXISTENT" → visited = []
- T-GT-005: 分岐グラフ（UC→DD, UC→TS）を 2 回実行 → 同一 visited 順
統合層のテスト前例は旧 `TS-LX-004_グラフ走査.md`（T-MT-001〜006 / T-DP-001〜005 / T-IM-001〜004 / T-IV-xxx）。
[要決定] REQ.04 の「打ち切り発生/非発生での stderr Info 有無テスト」「--max-depth なし E2E」と REQ.09 の「--json 出力スキーマテスト」は新規作成（v3 前例なし）。

---

## §3 用語・前提の補完

| 用語 | 補完内容 | 根拠 |
|------|---------|------|
| IndexMap | Rust `indexmap` crate の挿入順保持マップ。ノード格納は graph.toml 記載順。順序保持 TOML パーサが前提 | SPEC-LGX-002.REQ.08（新リポ内）、lx-graph/src/model.rs:63 |
| CTX-INV-1（決定論保証） | 「同じ入力に対して常に同じコンテキスト結果を返す」 | LEGIXY-SPEC-001 §10.1（legixy.old.p1/docs/legixy_foundational_spec.md L225） |
| CTX-INV-3（カスタムエッジ独立性） | 「カスタムエッジはチェーン上流に影響しない」— compile_context の意味的制約（所有 SPEC-LGX-003.REQ.05）。走査到達性とは別概念（REQ.01 注記どおり） | 同 §10.1、GAP-LGX-081 §2 |
| MCP-INV-1（Agent Surface 限定） | MCP は compile_context / observe / get_compile_audit の 3 ツールのみ。走査の MCP ツール化は禁止（REQ.10） | LGX-EXT-001 §6.1 |
| SUBNODE-INV-4（DAG 制約） | グラフにサイクルが存在しないこと。検証は SPEC-LGX-002.REQ.07（Kahn's algorithm、全エッジ種別対象）。REQ.06 は「これが破れていても走査が停止する」ことを要求 | LGX-EXT-001 §7.1 L615、SPEC-LGX-002.REQ.07 |
| Admin Surface / Agent Surface | CLI（人間向け）と MCP（エージェント向け）の二面分離 | SPEC-LGX-001.REQ.08（新リポ内） |
| chain / custom / parent_child | 成果物連鎖 / 人間明示の任意参照 / 親ドキュメント→サブノードの暗黙エッジ。3 種とも from/to を持つ有向エッジ | SPEC-LGX-002.REQ.04（新リポ内） |
| drift / suspicious nodes | engine.db scores テーブル（score_type='drift'）の値。閾値（既定 0.3）以上のノードが suspicious。スコア降順・ID 昇順整列 | UC-LGX-005、drift_pruner.rs、lx-core/src/config/loader.rs:246 |
| depth_map | 各ノードの起点からの BFS 距離（起点 = 0）。多起点では min(depth) | result.rs:22-26 |
| 互換対象 (a)〜(f) | (a)サブコマンド名 (b)位置引数 (c)フラグ名と値 (d)既定値 (e)終了コード (f)MCP 3 ツールの CLI マッピング | LGX-COMPAT-001 §1 |
| 前段ループ / QSET / SPP / DD 凍結 / GAP | DevProc_V4.1 の成果物体系。DD 凍結 = 境界 API 契約の凍結（ハードルール 7）。新リポには DevProc 文書も未配置（旧所在: legixy.old.p1/docs/DevProc_V4/） | legixy.old.p1/CLAUDE.md |
| v3 / 【v3 差分】 | 旧バイナリ traceability-engine v0.4.0-alpha4。SPEC 中の te-* ソース参照は旧実装リポでは lx-* に改名済み | LGX-COMPAT-001 §2 |
| TS-LGX-001 T-GT-xxx | legixy 版テスト仕様（未作成）。旧 v3 TS-LX-001 §6 の同名テストが前身 | §2.6 参照 |

---

## §4 旧実装からの参考情報

旧実装: `traceability-engine.v3.chg_to_lexigy/`（crate 名 lx-*。SPEC 引用の te-* パスに対応）

| 対象 | crate / ファイル | 内容 |
|------|------------------|------|
| 単起点 BFS 本体 | `crates/lx-graph/src/traversal.rs` | traverse_forward / traverse_reverse、max_depth 判定（L53-58）、visited 制御、edges_traversed（spanning tree） |
| グラフモデル | `crates/lx-graph/src/model.rs` | `IndexMap<NodeId,Node>`（L63）、outgoing/incoming index リスト（L129-137） |
| 多起点ラッパー | `crates/lx-nav/src/multi_traverser.rs` | 入力順マージ、min(depth)、存在しない起点の読み飛ばし |
| impact | `crates/lx-nav/src/impact.rs` | traverse_forward_multi への委譲 |
| investigate | `crates/lx-nav/src/investigate.rs` | 逆方向走査 + DriftPruner。max_depth は常に None【§2.4 要決定】 |
| drift 抽出 | `crates/lx-nav/src/drift_pruner.rs` | scores テーブル照会 SQL、降順 + ID 昇順整列、失敗時ベストエフォート |
| 結果型 | `crates/lx-nav/src/result.rs` | VisitedNode / MultiTraversalResult / PrunedTraversalResult（serde 対応済） |
| 出力整形 | `crates/lx-nav/src/reporter.rs` | Text / JsonLines 両形式、f32 丸め変換 |
| CLI 定義 | `crates/lx-cli/src/main.rs` L162-176（Impact/Investigate 引数）、L334-339（--json 未伝播箇所 = SPEC 引用の機能化対象） | clap 定義と dispatch |
| CLI コマンド | `crates/lx-cli/src/commands/impact.rs`, `investigate.rs` | .trace-engine.toml 読込 → parse_graph → lx-nav 委譲 → Text 固定出力 |
| 設定既定値 | `crates/lx-core/src/config/loader.rs` L236,246 | drift_threshold 既定 0.3 |
| 旧 DD | `docs/detailed-design/DD-LX-004_グラフ走査.md` | クレート構成（te-nav 新設 = 案 A）、公開 API、アルゴリズム、Reporter、エラー型。legixy 版 DD の出発点 |
| 旧 TS | `docs/test-specs/TS-LX-001_グラフ基盤.md` §6（T-GT-001〜005）、`docs/test-specs/TS-LX-004_グラフ走査.md` | テスト前例（§2.6 参照） |

---

## 集計

- 未解決参照: 12 件（§1。うち 2 件〔TS-LGX-001、対応 DD〕は旧リポにも legixy 版が存在せず未作成）
- [補完]: 13 件（§2.1×5、§2.2×1、§2.3×2、§2.4×2、§2.5×1、§2.6×1、ほか §3 用語 12 項目は補完扱いに含めず）
- [要決定]: 7 件（--json 構造、UC 2a と REQ.05 の矛盾、investigate の --max-depth 機能化、investigate の db 配線、--drift-threshold フラグ、打ち切り Info の文言・件数算定、新規テスト 3 種の作成）
