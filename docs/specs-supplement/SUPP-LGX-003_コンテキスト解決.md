Document ID: SUPP-LGX-003

# SUPP-LGX-003: SPEC-LGX-003（コンテキスト解決）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-003 |
| 対象 SPEC | SPEC-LGX-003 v0.7.1（2026-06-10、Approved） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
>
> 出典の所在: 参照文書群は旧リポジトリ `legixy.old.p1/docs/`、旧実装（v3）は `traceability-engine.v3.chg_to_lexigy/` にある。本リポジトリ（legixy）には docs/specs/ の SPEC 10 件のみが存在する。

---

## §1 未解決参照（SPEC が参照するが本リポジトリに存在しない文書）

凡例: ✅ = 旧リポジトリ/旧実装で実体を確認済み、❌ = どこにも実体が存在しない（未作成）。

| # | 参照 ID / 名称 | SPEC 内の参照箇所 | 必要な理由 | 所在 |
|---|---------------|------------------|-----------|------|
| 1 | LEGIXY-SPEC-001（基盤仕様 v1.0.0） | ヘッダ表、§2、REQ.02/04/05、§4 | CTX-INV-1〜4・FB-INV・MCP-INV・STATE-INV の定義（§10）、P-02 設計原則（§9）、成果物連鎖モデル（§2〜3） | ✅ `legixy.old.p1/docs/legixy_foundational_spec.md` |
| 2 | LGX-EXT-001（サブノード拡張仕様 v0.2.1） | ヘッダ表、§2、REQ.01/03/06/07/08/15/16/17 | granularity 引数定義（§5.1）、subnode 返却構造（§5.2）、サブノード ID 生成（§4.5）、context_log 拡張（§4.3）、MCP-INV-1 整合（§6） | ✅ `legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md` |
| 3 | LGX-EXT-002（Prompt Caching 拡張仕様 v0.1.0） | REQ.10〜14 | セクション整列原則（§3.2〜3.5）、500,000 文字上限（§4.3）、CACHE-INV-1〜4 定義（§5.2）、設計判断確定事項（§8） | ✅ `legixy.old.p1/docs/legixy_cache_spec_v0_1_0.md` |
| 4 | NFR-LGX-001（非機能要件） | ヘッダ表、REQ.09/13/19 | PERF.03（応答時間）、PERF.09（サイズ上限）、REL.03/05/07（冪等性・BFS 決定性・busy_timeout 5000ms）、SEC.02（SQLite 排他） | ✅ `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md` |
| 5 | UC-LGX-002（コンテキスト解決） | ヘッダ表、REQ.02 検証方法 | 基本フロー（resolver→走査→layer→custom→ContextResult→context_log）と代替フロー（未対応ファイル→artifact_id null 等） | ✅ `legixy.old.p1/docs/usecases/UC-LGX-002_コンテキスト解決.md` |
| 6 | UC-LGX-004（粒度制御付き） | ヘッダ表、REQ.03/18 検証方法 | subnode 展開フロー、fallback（サブノード無し→全文）、Phase 2 Block B 代替フロー 4-A〜4-D | ✅ `legixy.old.p1/docs/usecases/UC-LGX-004_粒度制御付きコンテキスト解決.md` |
| 7 | QSET-LGX-003（質問票・回答） | REQ.13/16/17/18 の根拠 | Q1（文字単位）・Q2（親 ID/depth0 の確定）・Q3（組合せマトリクス）の決定根拠と v3 実測箇所 | ✅ `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-003_コンテキスト解決.md` |
| 8 | SPP-LGX-003（SPEC 差分パッチ） | 変更履歴 v0.5.0 | v0.5.0 改訂の差分根拠（査読証跡） | ✅ `legixy.old.p1/docs/spec-patches/SPP-LGX-003_コンテキスト解決.md` |
| 9 | GAP-LGX-041 | REQ.19 | 監査ログ書込失敗時の論点整理（選択肢と影響範囲） | ✅ `legixy.old.p1/docs/gap-analysis/GAP-LGX-041_context-log書込失敗時の本処理成否.md` |
| 10 | GAP-LGX-043 | REQ.20 | 起点不在・部分欠損の論点整理 | ✅ `legixy.old.p1/docs/gap-analysis/GAP-LGX-043_上流に存在しないパスと部分欠損の扱い.md` |
| 11 | GAP-LGX-045 | REQ.16 | sections 縮退入力の論点整理 | ✅ `legixy.old.p1/docs/gap-analysis/GAP-LGX-045_sectionsの不正形式入力の扱い.md` |
| 12 | GAP-LGX-047 | REQ.15 | outline-only 見出し皆無時の論点整理 | ✅ `legixy.old.p1/docs/gap-analysis/GAP-LGX-047_outline-only見出し皆無時の出力.md` |
| 13 | LGX-COMPAT-001（CLI/MCP 互換リファレンス） | REQ.19 根拠 | 凍結終了コード規約（exit 0/1/2）、`context` サブコマンドの引数契約、MCP→CLI マッピング。**注意: SPEC は v1.0.1 を引用するが最新は v1.1.0（2026-06-10、embed 加算拡張。context には影響なし）** | ✅ `legixy.old.p1/docs/legixy_cli_compat_reference.md` |
| 14 | ADR（REQ.19「ADR に記録する」） | REQ.19 | 可用性 > 監査完全性の設計判断 = ADR-LGX-004（accepted、2026-06-10） | ✅ `legixy.old.p1/docs/adr/ADR-LGX-004_availability-over-audit-completeness.md` |
| 15 | TS-LGX-001（T-DB-001、T-GT-005） | REQ.07 検証方法 | DB スキーマ検証・BFS 決定性のテスト仕様 | ❌ どこにも不在（旧リポジトリ `docs/test-specs/` は空。**未作成**） |
| 16 | TS-LGX-002 §15（T-CC-OUTLINE/SECTIONS/DEPTH-001） | REQ.15/16/17 検証方法 | Block B 機能のテスト仕様 | ❌ どこにも不在（**未作成**） |
| 17 | DD-LGX-00x（詳細設計） | §1.2 スコープ、REQ.15 | 空 body 固定フォーマット等「DD で確定」とされた事項の受け皿 | ❌ どこにも不在（旧リポジトリ `docs/detailed-design/` は空。**未作成**。旧実装内コメントの DD-LX-002/007 は v3 世代の別文書で legixy 正準ではない） |
| 18 | CLAUDE.md「事前ガイダンス義務」 | REQ.02 根拠 | compile_context を成果物編集前に呼ぶ運用規約 | ✅ `traceability-engine.v3.chg_to_lexigy/CLAUDE.md` L46（「成果物の作成・編集の前に compile_context を呼び出し、上流成果物を全て参照すること」） |
| 19 | v0.1.0 実装（`old.source/RustCLI/`, `old.source/TypeScriptMCP/`） | §2（慣例仕様） | 「その他オプション: v0.1.0 を継承」（REQ.01）の具体内容 | ✅ `traceability-engine.v3.chg_to_lexigy/old.source/` |
| 20 | v3 実装（`crates/te-ctx/src/...`） | REQ.13/18 根拠 | 実測正準化の根拠コード。**te-ctx は現存せず、`crates/lx-ctx/` に改名済み**（LGX-COMPAT-001 §2 の旧クレート → legixy 命名対応を参照） | ✅ `traceability-engine.v3.chg_to_lexigy/crates/lx-ctx/` |

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 [補完] — 旧文書・旧実装から補完可能なもの

**S2-01 [補完] 返却テキストの具体フォーマット（セクション見出し・エントリ枠・区切り文字）**
SPEC はセクション順序（REQ.10）と整列（REQ.11）のみ規定し、レンダリング形式は未規定。v3 実測（`lx-ctx/src/section_formatter.rs`）:
- 各セクションは `# {Section Title}\n\n` で開始（例: `# Layer Guidelines`）
- エントリ間区切りは `\n---\n`、エントリが 1 件以上あるセクション末尾に `\n`、さらにセクション間に `\n`
- キャッシュブレーク点マーカは Additional Guidelines セクションの後に `<!-- cache-breakpoint: stable-end -->` + `\n\n`
- 改行コードは LF 固定（v3 コメント上の根拠: NFR-LX-001.COMPAT.08）
- Upstream エントリのヘッダ行: `artifact_id:` / `type:` / `file_path:`（`\` は `/` に正規化）/ `chain_distance:` / 任意で `subnode_id:` / `anchor:` / `drift_score:`、空行 1 つを挟んで body
- Layer/Additional エントリ: `layer:` / `node_id:` / `file_path:` / `specificity:` / `priority:` + 空行 + body
- Target Node Metadata エントリ: `artifact_id:` / `outgoing_edges:`（件数）/ `incoming_edges:`（件数）/ `subnode_count:`（body なし）
ただしこのフォーマットの正式確定は DD の責務（REQ.15 にも「固定形として DD で確定する」と明記）。v3 形式を DD のベースラインとして提案する。

**S2-02 [補完] REQ.13 エラーメッセージの正確な文字列**
v3 実測（`lx-ctx/src/error.rs:12-15`）: `compile_context result exceeds {limit} characters.\nCurrent size: {current} characters.\nSuggested action:\n  - Try --granularity subnode for finer-grained retrieval.\n  - Narrow the target scope.`（limit=500,000）。SPEC の例示と一致。定数は `lx-ctx/src/lib.rs:36` `RESULT_SIZE_LIMIT_CHARS = 500_000`、マーカは同 `:39`。サイズ計測は整形中の逐次集計（早期打切り）+ 最終 `enforce_size_limit`（`.chars().count()`、defense-in-depth）の二段構え（`section_formatter.rs:126-144`）。

**S2-03 [補完] context_log テーブルのスキーマ（REQ.07）**
SPEC は「granularity カラムを追加する」とのみ規定。v3 実測（`lx-db/src/schema.rs:155-161`）:
`context_log(id INTEGER PRIMARY KEY AUTOINCREMENT, target_id TEXT NOT NULL, granularity TEXT NULL, payload TEXT NOT NULL, created_at TEXT DEFAULT (datetime('now')))`
payload は JSON（`lx-ctx/src/audit_logger.rs:46-57`）: `command, target_files[], targets[](解決済 ID), granularity, upstream_count, layer_count, additional_count, custom_count, unresolved_count`。複数 target 時の `target_id` カラムは「最初に解決できた artifact_id」（無ければ空文字）。

**S2-04 [補完] SQLite 排他制御の具体値（REQ.09）**
v3 実測（`lx-db/src/connection.rs:24-27`）: `journal_mode=WAL`、`busy_timeout=5000`（ms）。NFR-LGX-001.REL.07 の暫定値 5000ms と一致（「人間査読時に実測して調整」の注記あり、無限リトライ禁止）。

**S2-05 [補完] CLI 引数契約と既定値（REQ.01、互換制約）**
LGX-COMPAT-001 §4 #10・§5: サブコマンドは `context <target_files...>`（位置引数・複数）、フラグは `[--command <S>] [--granularity document|subnode] [--outline-only] [--sections <ids>] [--depth <N>]`、granularity 既定 `document`。MCP `compile_context` の zod スキーマは `target_files: string[]`(min1), `command?: string`, `granularity?: "document"|"subnode"`, `outline_only?: bool`, `sections?: string`(min1), `depth?: int`(min1)。snake_case → kebab-case の機械変換で CLI へ忠実転送（`ts-mcp/src/tools/compile-context.ts:73-83`）。グローバル `--project-root` / `--json` / `--models-dir` も受理必須。

**S2-06 [補完] `--command` オプションの意味（REQ.01「その他オプション: v0.1.0 を継承」の実体）**
UC-LGX-002 基本フロー 1 で「intent」とされ、v3 では返却内容には影響せず context_log の payload `command` に記録されるのみ（`audit_logger.rs:46`）。決定論（REQ.04/14: 同一 target_files + granularity → 同一結果）と整合する。

**S2-07 [補完] granularity 不正値の扱い**
v3 実測（`lx-cli/src/commands/context.rs:28-35`）: `document`/`subnode` 以外は実行時エラー（anyhow bail → exit 1。受理済み引数の意味的不正であり LGX-COMPAT-001 v1.0.1 終了コード規約の exit 1 に該当。clap 構文エラーの exit 2 とは区別される）。

**S2-08 [補完] サブノードの定義・ID 生成・見出し抽出規則（REQ.08/16 の前提）**
LGX-EXT-001: 自動抽出対象は h2/h3 固定（§3.6）。自動生成 ID は `{親ID}#{ハッシュ}`、ハッシュは「親ID|見出し階層パス」を SHA-256 短縮 16 文字（§4.5.1、例: `DD-SP-003|認証機能|状態遷移`）。明示 ID は §4.5.2、自動/明示の区別は nodes テーブル `is_subnode`（0=ドキュメント, 1=自動, 2=明示。§4.3）。見出しテキスト正規化は §4.6。サブノード化は任意機能で、無サブノードの graph.toml は従来どおり動作（§3.5、UC-LGX-004 代替フロー 3a の fallback と整合）。

**S2-09 [補完] subnode 粒度の返却フィールド（REQ.03 の返却構造詳細）**
LGX-EXT-001 §5.2: 各エントリ = サブノード ID、親ドキュメント ID、アンカー（見出しテキストまたは anchor フィールド値）、本文（該当セクションのみ）、ドリフトスコア（エッジごと）。UC-LGX-004 4-D: 親ドキュメント自体は返さず子サブノードを個別 artifact 展開、body は `content_range` で切り出した部分テキストのみ（`Document ID:` 行・ヘッダ表・変更履歴等のテンプレ部分を含まない）。v3 実装は `lx-ctx/src/compiler.rs:206-260`（サブノード無しドキュメントは全文 fallback、`lx-ctx/src/subnode/` に抽出器）。

**S2-10 [補完] 上流走査のアルゴリズム（REQ.02/08/17 の実体）**
v3 実測（`lx-ctx/src/upstream_walker.rs`）: Chain / ParentChild エッジのみを逆方向（incoming）に BFS。Custom エッジはスキップ（CTX-INV-3）。visited セットで循環遮断（CTX-INV-4 耐性）。決定論は IndexMap 挿入順の隣接リストに依存（NFR REL.05、SPEC-LGX-002.REQ.08 の順序保持 TOML パーサが前提）。depth_limit は `Some(N)` で chain_distance ≤ N、`None` で無制限。起点がグラフ未登録なら空 Vec を返す（REQ.20-1 と整合）。複数起点の重複は seen セットで除去（`compiler.rs:180-188`）。

**S2-11 [補完] Layer Guidelines / Additional Guidelines のデータソースと解決規則（REQ.02 の前提）**
SPEC は「該当レイヤのガイドライン」「プロジェクト固有の補足」とのみ規定。v3 実測（`lx-ctx/src/layer_resolver.rs`）: engine.db の `layer_rules` / `layer_documents` テーブル（`lx-db/src/schema.rs:141-152`）から解決。ルール payload は `path_glob` / `specificity`（既定 1）/ `priority`（既定 1）、First-Match-Wins（specificity DESC, priority ASC）。Additional Guidelines は `rule_name` が `additional:` プレフィクスのルール（Phase 1 運用規約）。db 不在時は両セクション空（FB-INV-4 と整合: 上流は graph.toml のみで返る）。

**S2-12 [補完] 性能・設定の前提値**
- NFR-LGX-001.PERF.03: compile_context 応答 Step 1 (Windows) < 300ms、Step 2 (Linux Docker) < 200ms（サブノード 100 件、MCP 経由ベンチ）
- 設定ファイル: `.legixy.toml` 既定 + `.trace-engine.toml` 旧名フォールバック（SPEC-LGX-008.REQ.13、LGX-COMPAT-001 §6）。graph.toml のパスは設定 `graph.file` から取得（v3: `commands/context.rs:23-25`。なお v3 は `.trace-engine.toml` 固定読みなので、フォールバック実装は legixy 新規）
- chain の 7 階層（REQ.17 の「SPEC → UC → ... → SRC」）は設定 `[id.chain] order = ["UC","RB","SEQ","DD","TS","TC","SRC"]` + independent（SPEC/NFR/VAL）に対応

### 2.2 [要決定] — 人間の判断・DD 確定が必要なもの

**S2-20 [要決定] Custom Documents セクションの位置づけ（REQ.10 と v3 実装の矛盾）**
REQ.10 は返却を 5 セクション構成と規定するが、v3 は **6 番目のセクション「Custom Documents」**（カスタムエッジ由来文書: from_id/to_id/file_path/reason + body、from_id→to_id 辞書順）を Target Node Metadata の後に出力する（`section_formatter.rs:108-122`）。UC-LGX-002 フロー 5〜6 も custom_documents を返却に含めると明記。選択肢: (a) v3 同様 6 番目として出力（事実上の互換維持。ただし REQ.10/CACHE-INV-2 の改訂 = 人間承認が必要）、(b) Target Node Metadata セクション内に統合（REQ.10 の文言は維持されるが v3 と出力が変わる）、(c) 返却から除外（UC-LGX-002 と矛盾）。**推奨は (a) + SPEC 改訂**だが承認が必要。

**S2-21 [要決定] subnode 粒度の整列キー: SPEC「アンカー出現順」vs v3「anchor 辞書順」**
REQ.11 は subnode 時「親ドキュメント ID 辞書順 + **アンカー出現順（物理位置順）**」と規定。しかし v3 実装は anchor **文字列のバイト辞書順**で比較しており（`section_formatter.rs:75-90`。なお同ファイルの `upstream_sort_rule` は "content_range-asc" を自称しコード実体と不一致）、見出しの物理順と辞書順が異なるドキュメントでは SPEC と v3 で出力バイト列が異なる。SPEC を正とするなら content_range（開始オフセット）等の位置情報で整列する実装が必要で、v3 出力との互換は破れる。選択肢: (a) SPEC どおり出現順で実装（v3 とのバイト列差異を受容）、(b) v3 実測を正準化して SPEC を改訂（人間承認）。バイト決定論（REQ.14）はどちらでも満たせるが、どちらを正とするかは人間判断。

**S2-22 [要決定] engine.db の open 経路と CLI 経由の監査ログ（REQ.07 実現の前提）**
v3 の CLI `context` コマンドは ContextCompiler に **db=None を渡しており**（`lx-cli/src/commands/context.rs:42`）、CLI 直接実行では context_log 記録も Layer Guidelines 解決も行われない（AuditLogger は db=None で no-op）。REQ.07「全呼出しは context_log に記録」を満たすには engine.db を開く経路の新設が必要。論点: ①読取系コマンドが engine.db を**新規作成**してよいか（STATE-INV-1: engine.db は再生成可能キャッシュ、FB-INV-4: DB 不在でも動作必須 → 「存在する場合のみ開いて記録、不在なら記録なしで成功」が一貫するが、その場合 DB 不在環境では監査ゼロとなる）。②MCP-INV-4 ベストエフォート（REQ.19）の「DB が利用可能な場合」の定義（ファイル存在? open 成功? テーブル存在?）。DD + 人間確認が必要。

**S2-23 [要決定] stderr 診断（Info/Warning）の正確な文言と形式**
REQ.16（親 ID 指定時 Info）、REQ.17（depth 0 空集合時 Info）、REQ.19（記録失敗 Warning）、REQ.20（未解決起点 Info）はいずれも【v3 差分】= v3 に実装が存在せず、文言・書式（プレフィクス `Warning:`/`Info:` の有無、ID の列挙形式、ロケール）が未規定。QSET-LGX-003 Q2 回答に例文（「`--sections` に親ドキュメント ID が指定されました。サブノード ID（`#` 付き）を指定してください」）があるのみ。stdout 決定論に影響しないため自由度はあるが、テスト期待値の確定に文言固定が必要 → DD で確定 + 人間査読。

**S2-24 [要決定] REQ.20「欠損の決定論的記録」の具体フォーマット**
未解決起点を「Target Node Metadata セクションおよび stderr Info に記録」、上流途中欠損を「出力内に決定論的に記録」とするが、記録の位置・キー名・整列規則は未規定。v3 は `ContextResult.unresolved_targets`（`lx-ctx/src/result.rs:20`）を保持するものの **render には含めていない**（payload の `unresolved_count` のみ）ため、v3 から形式を補完できない。Target Node Metadata 内に `unresolved_targets:`（パス辞書順）等の固定形を DD で定義する必要。CACHE-INV-1 への影響があるため形式は SPEC/DD レベルで固定し人間査読を通すべき。

**S2-25 [要決定] REQ.15 見出し皆無時の「空 body の正確なフォーマット」**
SPEC 自身が「固定形として DD で確定する」と明記（GAP-LGX-047 由来）。候補: artifact ヘッダ行群 + 空行 + 空文字列（= v3 の `render_upstream_entry` で body が空のケースと同形）。DD-LGX-00x が未作成のため確定先が存在しない（§1 #17 参照）。

**S2-26 [要決定] Target Node Metadata の内容範囲**
LGX-EXT-002 §3.3 は「ドリフトスコア、最終更新時刻、関連する Observation、探索発見順等」を Target Node Metadata に集約するとするが、v3 実装は `outgoing_edges/incoming_edges 件数 + subnode_count` のみ（`result.rs:62-66`、`section_formatter.rs:265-272`）。REQ.20 の未解決起点記録も加わる。何を必須フィールドとするかは未規定であり、増やすほど変動度が上がる（キャッシュ後方の配置で吸収可能だが決定論的整形は必要）。DD で確定 + 人間判断。

**S2-27 [要決定] TS-LGX-001 / TS-LGX-002 / DD-LGX-00x の不在**
REQ.07/15/16/17 の検証方法が参照するテスト仕様と、複数 REQ が確定先とする DD が**どこにも存在しない**（§1 #15〜17）。実装に先立ち DevProc_V4.1 のフローに従って新規作成が必要。作成順序（DD 先行か TS 先行か）はプロセス判断。

**S2-28 [要決定] PERF.03 計測条件の解釈**
NFR-LGX-001.PERF.03 は「サブノード 100 件含む、最大粒度指定なし」を条件とするが、Block B フラグ（outline-only/sections/depth）併用時の目標値は未規定。GAP-LGX-171（PERF03 参照値の整合）が旧リポジトリに存在するため参照のこと（`legixy.old.p1/docs/gap-analysis/GAP-LGX-171_PERF03参照値の整合.md`）。

---

## §3 用語・前提の補完

| 用語 | 定義（出典） |
|------|-------------|
| CTX-INV-1〜4 | 決定論保証 / グラフ整合性 / カスタムエッジ独立性 / DAG 制約（LEGIXY-SPEC-001 §10.1） |
| FB-INV-3 / FB-INV-4 | 承認前不変性（pending Proposal は context 結果に影響しない）/ DB 不在時安全性（DB がなくてもグラフ上流は正常に返される）（同 §10.2） |
| MCP-INV-1 / 2 / 4 | Agent Surface 限定（MCP は compile_context, observe, get_compile_audit の 3 ツールのみ）/ 忠実な転送（Rust CLI 出力のフィルタリング・省略なし）/ 監査ログ完全性（同 §10.4。INV-4 は REQ.19 でベストエフォートに緩和） |
| STATE-INV-1 | ステートレス性: 永続状態は graph.toml（Git 管理）と engine.db（再生成可能キャッシュ）のみ（同 §10.5） |
| CACHE-INV-1〜4 | バイト単位決定論 / セクション配置順序固定 / 大規模返却エラー / _meta 付与の忠実性（LGX-EXT-002 §5.2） |
| chain エッジ / ParentChild / Custom | エッジ種別。上流走査は Chain + ParentChild のみ遡り Custom は遡らない（CTX-INV-3、LGX-EXT-001 §3.4、v3 `upstream_walker.rs`）。Custom 由来文書は Custom Documents として別掲（§2 S2-20） |
| サブノード / anchor / content_range | ドキュメントを見出し単位等で細分化したノード（LGX-EXT-001 §3.1〜3.6）。anchor = 見出しテキスト等のドキュメント内アンカー、content_range = 該当セクションの本文範囲。ID 形式は `{親ID}#{ハッシュ16桁}`（自動）または明示 ID（§4.5） |
| Layer Guidelines / Additional Guidelines | engine.db の layer_rules/layer_documents による path_glob ベースのガイドライン文書解決。additional は `additional:` プレフィクスのルール（v3 `layer_resolver.rs`。§2 S2-11） |
| P-02「判断は人間に委ねる」 | LEGIXY-SPEC-001 §9 の設計原則。REQ.13 がエラー方式（切捨て・要約の禁止）を採る根拠 |
| Phase 2 Block B | LGX-EXT-001 の段階的導入計画（§8）における機能ブロック。outline_only / sections / depth の 3 拡張（同 §5.1「Phase 1 で実装しない引数」が v0.4.0-alpha3 で実装され REQ.15〜17 として formal 化された）。**注意: SPEC の「LGX-EXT-001 §5.4（Phase 2 Block B）」という参照は v0.2.1 文書の §5.4（semantic_check のサブノード対応）と節番号が一致しない。Block B の実体は §5.1 の Phase 2 引数群 + §8 の導入方針と読むのが妥当（軽微な参照ズレ、人間確認推奨）** |
| 前段ループ / QSET / SPP / TP / GAP / FCR | DevProc_V4.1（`legixy.old.p1/docs/DevProc_V4/` 配下に vendored、特に 03a-frontend-pass.md）の frontend-pass 用語。QSET = 質問票、SPP = SPEC 差分パッチ、TP = テスト観点、GAP = ギャップ票、FCR = frontend check result（`legixy.old.p1/docs/frontend-pass/check-results/FCR-LGX-003_コンテキスト解決.md` も存在） |
| vnstudio dogfeeding | 旧実装のドッグフーディング先プロジェクトでの観測（REQ.15 の 88,303 bytes 削減実績、REQ.17 の deep chain 観察の出典）。一次データは legixy リポジトリに不在 |
| engine.db | `.legixy/engine.db`（LGX-EXT-001 §4.3。v3 実体の配置は NFR REL.08 参照）。WAL モード、busy_timeout 5000ms |
| 「文字」 | Unicode コードポイント数（Rust `.chars().count()`、QSET-LGX-003 Q1 で確定。UTF-8 バイト数・UTF-16 単位ではない） |
| 凍結終了コード規約 | exit 0 = 成功（空結果・部分成功含む）、exit 1 = 受理済み引数の意味的不正・実行時失敗、exit 2 = 引数パーサ層の構文誤り（LGX-COMPAT-001 §3 グローバル規約 v1.0.1） |

---

## §4 旧実装からの参考情報

### 4.1 クレート対応

SPEC 中の `crates/te-ctx/...` 参照は、旧実装リポジトリでは `lx-` プレフィクスに改名済み（LGX-COMPAT-001 §2 は legixy で `legixy-core/...` 命名を指示）。本 SPEC 関連の対応:

| SPEC 中の参照 | 実体（行番号はほぼ一致） |
|---|---|
| `crates/te-ctx/src/section_formatter.rs:129-144` | `traceability-engine.v3.chg_to_lexigy/crates/lx-ctx/src/section_formatter.rs`（`enforce_size_limit` は :126-144） |
| `crates/te-ctx/src/compiler.rs:201-253` | 同 `crates/lx-ctx/src/compiler.rs`（granularity 分岐 + outline/sections 適用は :180-260） |
| `te-ctx/src/upstream_walker.rs:49-55` | 同 `crates/lx-ctx/src/upstream_walker.rs`（depth 制御は :50-56） |

### 4.2 関連ファイル一覧（読解の起点）

- `crates/lx-ctx/src/compiler.rs` — compile 本体。granularity 分岐、subnode 展開、sections フィルタ、outline 適用順（REQ.18 マトリクスの根拠）、`build_outline`（:376-397、REQ.15 の抽出規則どおり: `#`〜`###`、スペース必須、末尾 `#` 除去、インデント 2×(level-1)）
- `crates/lx-ctx/src/section_formatter.rs` — REQ.10〜14 主担当。セクション順序・整列・マーカ・サイズ上限
- `crates/lx-ctx/src/upstream_walker.rs` — REQ.02/08/17。Chain+ParentChild 逆方向 BFS、depth_limit
- `crates/lx-ctx/src/audit_logger.rs` — REQ.07/19。ベストエフォート書込（db=None で no-op、Err は stderr のみで Ok 維持 = REQ.19 の先行実装）
- `crates/lx-ctx/src/layer_resolver.rs` — Layer/Additional Guidelines 解決（First-Match-Wins）
- `crates/lx-ctx/src/file_resolver.rs` / `content_cache.rs` / `custom_edge_resolver.rs` / `result.rs` / `error.rs` / `subnode/` — 入力パス→ノード解決、本文キャッシュ、Custom 文書、返却構造体、エラー型、サブノード抽出
- `crates/lx-db/src/schema.rs`（context_log :155、layer_rules :141、layer_documents :148）、`connection.rs`（WAL/busy_timeout :24-27）
- `crates/lx-cli/src/main.rs`（Context サブコマンド定義 :135）、`commands/context.rs`（CLI 経路。**db=None 問題 → §2 S2-22**）
- `ts-mcp/src/tools/compile-context.ts` — MCP 側 zod スキーマ、kebab-case 変換、`_meta["anthropic/maxResultSizeChars"]=500000` 付与
- `old.source/RustCLI/`, `old.source/TypeScriptMCP/` — v0.1.0 慣例仕様（REQ.01「その他オプション継承」の出典）
- `deploy/manual.md`（および `legixy.old.p1/docs/DevProc_V4/manual/traceability-engine.v3/manual.md`）— 利用者視点のコマンド説明

### 4.3 v3 を参考にする際の注意（SPEC との既知の乖離）

1. **subnode 整列**: v3 は anchor 辞書順、SPEC は出現順（§2 S2-21）。「v3 実測の正準化」は REQ.13/16/17/18 についてのものであり、整列キーには及ばない。
2. **CLI 経路の db=None**: v3 の CLI `context` は context_log 記録・Layer Guidelines 解決が機能していない（§2 S2-22）。REQ.07 の実装参考にはならない。
3. **Custom Documents**: v3 の 6 セクション目は REQ.10 の 5 セクション規定に現れない（§2 S2-20）。
4. **stderr Info 診断**: REQ.16/17/19/20 の【v3 差分】部分は v3 に存在しない新規実装。
5. **設定ファイル名**: v3 は `.trace-engine.toml` 固定。legixy は `.legixy.toml` 優先 + 旧名フォールバック（SPEC-LGX-008.REQ.13）。

---

（以上。本文書は人間査読を経るまで実装判断の唯一根拠としないこと。）
