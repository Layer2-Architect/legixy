Document ID: TS-LGX-002

# TS-LGX-002: コンテキスト解決（compile_context）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-002 で確定した型・関数シグネチャ（`legixy-ctx`）と ts-mcp 転送層に即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-002
**継承 TP**: TP-LGX-003（TP[SPEC] コンテキスト解決、51 観点）, TP-LGX-012（TP[UC] UC-002 フロー、22 観点）。加えて DD-LGX-002 §8 が引用する TP-LGX-009（MCP サーバ、`compile_context` 転送・`_meta` 観点）, TP-LGX-014（TP[UC] UC-004 粒度制御）を継承。

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-002 の関数 / 型:

- DD-LGX-002 §3 `legixy_ctx::ContextCompiler::compile(&self, input: &CompileInput) -> Result<ContextResult, ContextError>`
- DD-LGX-002 §3 `legixy_ctx::ContextCompiler::render(&self, result: &ContextResult) -> Result<String, ContextError>`
- DD-LGX-002 §3 `legixy_ctx::SectionFormatter::render(result: &ContextResult) -> Result<String, ContextError>`
- DD-LGX-002 §3 `legixy_ctx::SectionFormatter::enforce_size_limit(rendered: &str) -> Result<(), ContextError>`
- DD-LGX-002 §3 `legixy_ctx::UpstreamWalker::walk_chain_parent_only_with_depth(&self, start: &NodeId, depth_limit: Option<usize>) -> Result<Vec<UpstreamArtifact>, ContextError>`
- DD-LGX-002 §3 `legixy_ctx::AuditLogger::log(&self, input: &CompileInput, result: &ContextResult) -> Result<(), ContextError>`
- DD-LGX-002 §2 型: `CompileInput` / `Granularity`{Document,Subnode} / `ContextResult`(6 セクション + `unresolved_targets`) / `ResolvedTarget` / `UpstreamArtifact` / `LayerDocument` / `CustomDocument` / `TargetNodeMetadata` / `ContextError`{ResultTooLarge,Io,Db,Graph,InvalidInput,Serde} / `ContextExitStatus`
- DD-LGX-002 §3 定数: `RESULT_SIZE_LIMIT_CHARS = 500_000` / `CACHE_BREAKPOINT_MARKER = "<!-- cache-breakpoint: stable-end -->"` / `CONTEXT_LOG_BUSY_TIMEOUT_MS = 5_000`
- DD-LGX-002 §4 ts-mcp 転送層 `ts-mcp/src/tools/compile-context.ts`（SRC-LGX-002-TS）: zod スキーマ・snake_case→kebab-case 変換・`_meta` 付与

委譲（本 TS 対象外）: 粒度別フロー（subnode 展開・fallback・drift_score 付与）の検証本体（→ TS-LGX-004／DD-LGX-004 所管）、embedding drift_score の数値妥当性（→ TS-LGX-007 / `ScoreLookup`）、性能予算 PERF.03/PERF.09（→ bench / NFR-LGX-001）、並行アクセス整合性・WAL/busy_timeout の実挙動 REL.07/SEC.02（→ NFR）、`observe`/`get_compile_audit` ツールおよび子プロセス管理・タイムアウト（→ TP-LGX-009 のうち SPEC-LGX-009 / NFR 所管分）、context_log 記録項目仕様（→ SPEC-LGX-007.REQ.06）。本 TS は「`compile_context`(legixy-ctx) が SPEC-LGX-003 の規定を DD-002 の型で正しく具体化しているか」と「ts-mcp が引数を忠実転送し `_meta` を付与するか」を検証する。

## 2. ケース一覧

### ケース 1: 上流ゼロ（target が最上流 SPEC）→ 6 セクション構成維持・exit 0

- **観点出典**: TP-003 §2.1 B-07, §2.8 L-01; TP-012 §2.2 AF2; TP-014 §2.1 BF4
- **分類**: Integration
- **前提**: `target_files` = 連鎖最上流ノード 1 件（上流なし）。`granularity = Granularity::Document`、`db = None`
- **入力**: `compiler.compile(&CompileInput { target_files, granularity: Document, ..Default::default() })` → `compiler.render(&result)`
- **期待**: `Ok(result)` かつ `result.upstream.is_empty()`。`render` の出力は 6 セクション（Layer Guidelines / Additional Guidelines / cache-breakpoint marker / Upstream Artifacts / Target Node Metadata / Custom Documents）の枠を全て含む（REQ.10）。upstream が空でも Upstream Artifacts セクション枠は存在
- **境界条件**: 上流ゼロでもセクション構成は件数非依存で固定

### ケース 2: 返却本文ちょうど 500,000 文字 → Ok（境界下限）

- **観点出典**: TP-003 §2.1 B-01, §2.2 E-01; TP-009 §2.2-13
- **分類**: Unit
- **前提**: `render` 結果が `.chars().count() == 500_000`（Unicode コードポイント、REQ.13）
- **入力**: `SectionFormatter::enforce_size_limit(&rendered_500000)`
- **期待**: `Ok(())`（500,000 は「超える場合」に該当しない）
- **境界条件**: `.chars().count() <= 500_000 ⇒ Ok`。境界値（上限ちょうど）

### ケース 3: 返却本文 500,001 文字 → Err(ResultTooLarge)・exit 1（境界 +1）

- **観点出典**: TP-003 §2.1 B-01, §2.2 E-01; TP-012 §2.3 EF1; TP-014 §2.3 EF2; TP-009 §2.2-13
- **分類**: Unit
- **前提**: `render` 結果が `.chars().count() == 500_001`
- **入力**: `SectionFormatter::enforce_size_limit(&rendered_500001)`
- **期待**: `Err(ContextError::ResultTooLarge { current: 500_001, limit: 500_000 })`。Display は DD-002 §2.3 / REQ.13 規定の文言（`compile_context result exceeds 500,000 characters.` + `Current size:` + `Suggested action:` 2 行、切り捨て・要約なし）。`legixy-cli` が **stderr** にこの文言を出力し **exit 1**（v3 `render(&result)?` 伝播・DD-freeze 裁定 2026-06-13 B-1）
- **境界条件**: `.chars().count() > 500_000 ⇒ ResultTooLarge`。最大+1。exit 1（v3 互換・SPEC-003.REQ.13「エラーを返却する」）

### ケース 4: 文字カウント単位 = Unicode コードポイント（サロゲート・結合・ZWJ）

- **観点出典**: TP-003 §2.1 B-02; TP-009 §2.2-12
- **分類**: Unit
- **前提**: 本文にサロゲートペア（絵文字）・結合文字・ZWJ シーケンスを含む rendered 文字列
- **入力**: `enforce_size_limit` のカウントを `.chars().count()`（コードポイント数）で計測
- **期待**: バイト長や grapheme cluster 数ではなく Unicode コードポイント数で判定する（`maxResultSizeChars` SPEC-LGX-009 REQ.13 と同一単位）。同一本文に対しカウントが安定
- **境界条件**: カウント単位 = コードポイント（バイト・grapheme と区別）

### ケース 5: `--depth 1` → 直接親（chain_distance==1）のみ・N+1 階層除外

- **観点出典**: TP-003 §2.1 B-03; TP-014 §2.2 AF3; SPEC-003 §REQ.17 検証方法（T-CC-DEPTH-001）
- **分類**: Unit
- **前提**: target から 2 階層以上の上流を持つグラフ。`depth_limit = Some(1)`
- **入力**: `walker.walk_chain_parent_only_with_depth(&start, Some(1))`
- **期待**: `Ok(vec)` かつ全要素 `chain_distance == 1`。祖父以遠（chain_distance >= 2）は含まれない
- **境界条件**: N=1 = 直接親のみ（下限）

### ケース 6: `--depth 0`（CLI 経由）→ 空集合・exit 0・stderr Info

- **観点出典**: TP-003 §2.1 B-03, §2.8 L-04, §2.9 LOG-04; TP-012 §2.2 AF4; TP-014 §2.2 AF3; SPEC-003 §REQ.17【v3 差分】
- **分類**: Integration
- **前提**: 任意のグラフ。CLI 経由で `depth_limit = Some(0)`
- **入力**: `walker.walk_chain_parent_only_with_depth(&start, Some(0))` + CLI ディスパッチ
- **期待**: `Ok(Vec::new())`（空集合）。CLI は stderr に Info 診断 `Info: --depth 0 results in empty upstream (no ancestors returned).`（DD-004 §11 S2-23）を出力。**stdout・exit 0 は不変**
- **境界条件**: depth=0 = 空集合（exit 0、エラーではない）。CLI と MCP の受理範囲差（MCP は zod reject、ケース 19）

### ケース 7: `--depth` 無制限（None）→ 7 階層全返却

- **観点出典**: TP-003 §2.1 B-03; TP-014 §2.2 AF3
- **分類**: Unit
- **前提**: `SPEC → UC → RB → SEQ → DD → TS → TC → SRC` 連鎖を持つグラフ。`depth_limit = None`
- **入力**: `walker.walk_chain_parent_only_with_depth(&start, None)`
- **期待**: depth 制限なしで到達可能な全上流を返す（深さで打ち切らない）。Chain/ParentChild エッジのみ逆 BFS、Custom スキップ
- **境界条件**: None = 無制限（上限なし）

### ケース 8: `--sections` 指定 ID のみ返却（subnode 粒度）

- **観点出典**: TP-003 §2.1 B-05, §2.8 L-03; TP-014 §2.2 AF3, §2.6 R5; SPEC-003 §REQ.16 検証方法（T-CC-SECTIONS-001）
- **分類**: Integration
- **前提**: `granularity = Subnode`、`sections = Some(vec!["DD-X-001#abc", "DD-X-001#def"])`、グラフに該当 subnode 存在
- **入力**: `compile` → `render`
- **期待**: upstream に指定 ID と完全一致する subnode のみ。一致なしの親ドキュメントは upstream に登場しない（REQ.16）。整列は親 ID 辞書順 + アンカー出現順（ケース 12）
- **境界条件**: sections フィルタは完全一致・subnode 粒度時のみ有効

### ケース 9: `--sections` 縮退入力（空トークン無視・dedup・全無効=空）

- **観点出典**: TP-003 §2.1 B-05, §2.7 I-04; TP-014 §2.2 AF2; SPEC-003 §REQ.16（GAP-LGX-045）
- **分類**: Unit
- **前提**: `legixy-cli` 層で正規化後の `sections`。入力表記 `"a,,b"` / 前後空白 / 重複 ID / 全無効
- **入力**: (a) `"DD-X-001#a,,DD-X-001#b"`、(b) ` DD-X-001#a , DD-X-001#a `（重複）、(c) `",,"`（全空）
- **期待**: (a) trim 後空トークンは無視し `{#a,#b}` で解決、(b) dedup して `{#a}` で解決（指定順・回数は出力に影響しない＝バイト列不変、CACHE-INV-1）、(c) 空 upstream で正常終了 exit 0
- **境界条件**: 空文字列（0 件）/ 1 件 / 重複の縮退規則。set セマンティクス

### ケース 10: `--sections` に親ドキュメント ID（`#` なし）→ 除外 + stderr Info

- **観点出典**: TP-003 §2.7 I-03, §2.9 LOG-04; TP-014 §2.6 R5; SPEC-003 §REQ.16【v3 差分】
- **分類**: Integration
- **前提**: `granularity = Subnode`、`sections = Some(vec!["DD-X-001"])`（subnode ID 形式でない）
- **入力**: `compile` → CLI ディスパッチ
- **期待**: 当該 ID は subnode ID と一致せず**除外**（結果が空になりうる）。CLI は stderr に Info `Info: --sections received a document-level ID 'DD-X-001', not a subnode ID. Use '#'-containing subnode IDs instead.`（DD-004 §11 S2-23）。**stdout・exit 0 は不変**
- **境界条件**: `#` なし = 親 ID = 除外（エラーにしない）

### ケース 11: `--outline-only` h1〜h3 抽出・本文非出力・見出し皆無で枠維持空 body

- **観点出典**: TP-003 §2.8 L-02; TP-012 §2.2 AF3; TP-014 §2.2 AF3; SPEC-003 §REQ.15 検証方法（T-CC-OUTLINE-001、GAP-LGX-047）
- **分類**: Unit
- **前提**: (a) h1〜h3 を含む body、(b) 見出し皆無 / h4 以降のみ / `#abc`（スペースなし）のみの body
- **入力**: `build_outline(body)` 経由の `render`
- **期待**: (a) 各見出しを `- {title}` 行に置換、インデント `  ` × (level-1)（h1=0,h2=2,h3=4）、h4 以降・`#abc` は対象外、本文テキストは出力しない。(b) artifact 枠（ヘッダ行群 + 空行 + 空文字列、DD-004 §11 S2-25）を維持し body 空、プレースホルダなし
- **境界条件**: 見出し有 / 皆無の境界。枠維持・body 空でバイト決定論保全

### ケース 12: subnode 整列 = 親 ID 辞書順 + アンカー出現順（A-1 裁定、決定論 property）

- **観点出典**: TP-003 §2.11 D-04, §2.11 D-06; TP-014 §2.5 DF2; SPEC-003 §REQ.11; DD-002 §8 / DD-004 §11 S2-21（A-1 裁定 2026-06-13）
- **分類**: Property-based（proptest）
- **生成器**: 複数親ドキュメント × 各々複数 subnode（anchor 文字列・ドキュメント内物理位置を保持）を任意順で生成
- **不変条件**: `render(.., Subnode)` の Upstream Artifacts は **親ドキュメント ID 辞書順昇順でグループ化**し、同一ドキュメント内は **アンカー出現順（ドキュメント物理位置順）**。v3 の anchor バイト辞書順では**ない**（A-1 裁定で SPEC-003.REQ.11 準拠＝出現順を採択）。グラフ探索発見順・エッジスコア順を整列根拠にしない（Target Node Metadata に隔離）
- **反例ハンドリング**: shrink して最小の順序不一致例を記録

### ケース 13: バイト単位決定論（REQ.14、property）

- **観点出典**: TP-003 §2.11 D-01, §2.11 D-02; TP-012 §2.6 R2; TP-014 §2.6 R3; SPEC-003 §REQ.14 検証方法（document / subnode 両モード）
- **分類**: Property-based（proptest）
- **生成器**: 任意の `ContextResult`（targets/upstream/layer/additional/custom/metadata をランダム生成）+ `granularity ∈ {Document, Subnode}`
- **不変条件**: 同一 `ContextResult` に対し `render` を 10 回呼び出し、全バイト列が完全一致（順序・区切り文字・空白・LF 含む）。`SectionFormatter::render` の出力フォーマットは LF 固定（CRLF 混入なし）
- **反例ハンドリング**: shrink して最小の非決定論入力を記録

### ケース 14: セクション配置順序固定（granularity 非依存）

- **観点出典**: TP-003 §2.11 D-03; TP-012 §2.1 BF3; SPEC-003 §REQ.10（CACHE-INV-2）
- **分類**: Unit
- **前提**: 同一グラフ・同一入力で `granularity = Document` と `Subnode`
- **入力**: 両モードで `compile` → `render`
- **期待**: 6 セクションの配置順序が両モードで同一（1.Layer Guidelines → 2.Additional Guidelines → 3.cache-breakpoint marker → 4.Upstream Artifacts → 5.Target Node Metadata → 6.Custom Documents）。順序は granularity に依らない
- **境界条件**: セクション順序は内容変動から独立して固定

### ケース 15: キャッシュブレーク点マーカが 1 箇所・固定位置

- **観点出典**: TP-003 §2.11 D-05; SPEC-003 §REQ.12 検証方法
- **分類**: Unit
- **前提**: 任意の `ContextResult`
- **入力**: `render(&result)`
- **期待**: 出力に `CACHE_BREAKPOINT_MARKER`（`<!-- cache-breakpoint: stable-end -->`）が **ちょうど 1 回** 出現し、Additional Guidelines 末尾と Upstream Artifacts 先頭の間に位置する
- **境界条件**: マーカ件数=1・固定位置（Custom Documents 等の存否に依らない）

### ケース 16: 起点ノード未登録 → 無視して残りで解決・exit 0・Info 診断・metadata 記録

- **観点出典**: TP-003 §2.7 I-01, §2.2 E-05; TP-012 §2.2 AF1; TP-014 §2.5 DF1; SPEC-003 §REQ.20-1（GAP-LGX-043）
- **分類**: Integration
- **前提**: `target_files` に graph.toml 未登録パス 1 件 + 登録パス 1 件
- **入力**: `compile(&input)` → `render`
- **期待**: `Ok(result)`。未登録起点は無視し登録済み起点で解決し **exit 0**。未解決の `ResolvedTarget` は `artifact_id: None`。`result.unresolved_targets` / `TargetNodeMetadata.unresolved_targets` に当該パスを記録（PathBuf 辞書順昇順、DD-004 §11 S2-24）。stderr Info `Info: the following target paths were not found in the graph and were skipped: ...`（DD-004 §11 S2-23）
- **境界条件**: 部分未登録 = 残りで解決。全未登録は空 upstream exit 0（ケース 17）

### ケース 17: 全起点未登録 → 空 upstream・exit 0

- **観点出典**: TP-003 §2.7 I-01; TP-012 §2.2 AF1; SPEC-003 §REQ.20-1
- **分類**: Integration
- **前提**: `target_files` 全件が graph.toml 未登録
- **入力**: `compile(&input)`
- **期待**: `Ok(result)` かつ `result.upstream.is_empty()`、全 `ResolvedTarget.artifact_id == None`、`exit 0`。命名規約からの chain 位置推定は行わない（推測排除）
- **境界条件**: 全件未登録（最大欠損）でも Err に昇格せず exit 0

### ケース 18: 上流連鎖途中の欠損（ファイル不在）→ 部分成功・空 body・決定論記録・exit 0

- **観点出典**: TP-003 §2.2 E-05; TP-012 §2.3 EF4; TP-014 §2.3 EF1; SPEC-003 §REQ.20-2
- **分類**: Integration
- **前提**: 上流連鎖途中のエッジ先ノードのファイル実体が不在、他は正常
- **入力**: `compile(&input)` → `render`
- **期待**: `Err` に昇格せず `Ok(result)`。欠損ノードは `UpstreamArtifact { body: String::new(), .. }` として継続し他の上流も返す（exit 0）。欠損は出力内に決定論的に記録（記録位置・形式が固定でバイト決定論を保全、REQ.14）
- **境界条件**: ファイル不在は finding でなく空 body 継続（ContextError へ昇格させない）

### ケース 19: MCP 経由 `depth: 0` / 空配列 / 空 sections → zod reject（CLI 起動前）

- **観点出典**: TP-009 §2.2-8, §2.2-9, §2.2-10, §2.4-21, §2.4-22; TP-003 §2.1 B-04, §2.7 I-05; TP-012 §2.6 R4; SPEC-003 §REQ.17, SPEC-LGX-009 §REQ.15
- **分類**: Contract（TS / ts-mcp）
- **前提**: ts-mcp `compile-context.ts` の zod スキーマ（`target_files: array(string).min(1)` / `depth: number().int().min(1)` / `sections: string().min(1)` / `granularity: enum(["document","subnode"])`）
- **入力**: (a) `{ depth: 0 }`、(b) `{ target_files: [] }`、(c) `{ sections: "" }`、(d) `{ granularity: "auto" }`
- **期待**: いずれも zod safeParse 失敗で MCP 層が reject（**Rust CLI を起動しない別経路**、exit code マッピング対象外）。CLI 経由 `--depth 0` の空集合 exit 0（ケース 6）と受理範囲が異なることが正準
- **境界条件**: MCP=zod reject（depth≥1・min1）/ CLI=空集合 exit 0 の受理範囲差

### ケース 20: MCP snake_case → kebab-case 変換・argv 配列忠実転送

- **観点出典**: TP-009 §2.1-3, §2.1-5, §2.4-24; TP-003 §2.10 F-02; TP-012 §2.6 R4; SPEC-LGX-009 §REQ.04/15
- **分類**: Contract（TS / ts-mcp）
- **前提**: ts-mcp が MCP 入力を CLI フラグへ変換
- **入力**: `{ target_files: ["a.md"], outline_only: true, sections: "X#a", depth: 2, granularity: "subnode", command: "fix" }`
- **期待**: `outline_only`→`--outline-only`、`depth`→`--depth`、`granularity`→`--granularity` 等の snake→kebab 変換。argv は **配列**マーシャリング（シェル文字列経由でなく `["context", "a.md", "--command", "fix", ...]`、injection 耐性）。CLI 出力本文をフィルタ・省略・加工せず転送（MCP-INV-2）
- **境界条件**: 引数変換の固定規約・argv 配列転送

### ケース 21: MCP `_meta["anthropic/maxResultSizeChars"] = 500000` 付与・本文非改変

- **観点出典**: TP-009 §2.8-40, §2.8-41, §2.8-42; TP-003 §2.10 F-03, §2.10 F-04; SPEC-LGX-009 §REQ.13（CACHE-INV-4）
- **分類**: Contract（TS / ts-mcp）
- **前提**: `compile_context` の MCP 応答
- **入力**: 任意の正常 `compile_context` 呼出し
- **期待**: 応答 `_meta["anthropic/maxResultSizeChars"]` == `500000`（Unicode コードポイント単位、REQ.13 と一致）。`_meta` 付与は本文（`content`）を改変しない（本文バイト列が CLI 出力と一致）。サイズ判定責務は Rust CLI 側（MCP は `_meta` 宣言のみ）
- **境界条件**: `_meta` は永続化ヒントで本文 no-op（CACHE-INV-4）

### ケース 22: MCP exit 0 + 非空 stderr → `_meta["legixy/warnings"]` 転送・空時省略

- **観点出典**: TP-009 §2.11-47; TP-003 §2.9 LOG-04; TP-012 §2.3 EF2; SPEC-003 §REQ.19, SPEC-LGX-009 §REQ.03（ADR-LGX-004）
- **分類**: Contract（TS / ts-mcp）
- **前提**: Rust CLI が exit 0 で完了し stderr に Warning 行（例: 監査ログ書込失敗）を出力 / stderr が空
- **入力**: (a) exit 0 + 非空 stderr、(b) exit 0 + 空 stderr
- **期待**: (a) MCP 成功応答の `_meta["legixy/warnings"]` に stderr の Warning 行を集積、(b) `_meta["legixy/warnings"]` フィールド自体を省略。適用対象は全 3 ツール（legixy 新規。v3 未実装）
- **境界条件**: stderr 非空 → warnings 付与 / 空 → フィールド省略

### ケース 23: 監査ログ書込失敗 → 本処理 Ok 維持・exit 0・stderr Warning

- **観点出典**: TP-003 §2.2 E-03, §2.6 P-02, §2.6 P-03; TP-012 §2.3 EF2; SPEC-003 §REQ.19（GAP-LGX-041）
- **分類**: Integration
- **前提**: engine.db 存在・本処理成功・context_log 書込のみ失敗（読取専用 DB fixture 等）
- **入力**: `compiler.compile(&input)`（内部で `AuditLogger::log` が失敗）
- **期待**: `compile` は `Ok(result)` を維持（書込失敗は `ContextError` に昇格させない）。`AuditLogger::log` は `Err` を stderr に `[legixy-ctx] Warning: audit log write failed (best-effort): {e}`（DD-004 §11 S2-23）で出力し常に `Ok(())` を返す。CLI は結果を stdout に返却し **exit 0**。記録欠落は許容（MCP-INV-4 ベストエフォート）
- **境界条件**: 副作用（記録）失敗は本処理成否と分離・別 Tx・exit 0

### ケース 24: engine.db 不在（db=None）→ graph.toml のみで上流返却・記録スキップ

- **観点出典**: TP-003 §2.3 S-01, §2.6 P-01; TP-012 §2.5 DF3; SPEC-003 §4 FB-INV-4
- **分類**: Integration
- **前提**: `ContextCompiler::new(.., db: None, ..)`
- **入力**: `compile(&input)` → `render`
- **期待**: graph.toml のみで上流走査を返す（`LayerResolver` は空 Vec、`AuditLogger::log` は no-op で `Ok(())`）。DB ファイルを新規作成しない（DD-002 §3 S2-22）。返却内容は DB 不在で degrade するが exit 0
- **境界条件**: DB 不在でも graph.toml 上流を返す（FB-INV-4）

### ケース 25: ステートレス read-only 不変（compile はグラフ/DB を変更しない、property）

- **観点出典**: TP-003 §2.3 S-03, §2.4 C-01; TP-012 §2.4 AT2; SPEC-003 §4 STATE-INV-1; DD-002 §3 不変条件（read-only）
- **分類**: Property/Integration
- **前提**: 任意の入力（成功・部分成功いずれも）。`graph: &TraceGraph` / `db: Option<&Connection>` を借用
- **入力**: `compile(&input)` 実行前後の graph / engine.db のハッシュ（context_log 書込以外の永続状態）
- **期待**: 実行前後で graph が不変。`context_log` 書込み以外の永続状態を持たない（STATE-INV-1）。pending Proposal が存在しても結果不変（FB-INV-3）。エラー時も中間状態破壊なし
- **境界条件**: 借用による read-only 保証（context_log 書込のみ唯一の副作用）

### ケース 26: 冪等性 — 同一入力 → 同一 ContextResult・同一 render

- **観点出典**: TP-003 §2.11 D-01; TP-012 §2.1 BF4; TP-014 §2.1 BF4; SPEC-003 §REQ.04/REQ.14（CTX-INV-1）
- **分類**: Property-based（proptest）
- **生成器**: 任意の有効入力 `CompileInput`（target_files・granularity・各フラグ）
- **不変条件**: 同一入力に対し `compile` を複数回呼び出し、`ContextResult`（フィールド順・件数・整列）が常に一致。`render` 出力もバイト一致（監査ログ副作用を除き冪等、DD-002 §3）
- **反例ハンドリング**: shrink して最小の非冪等入力を記録

### ケース 27: granularity 不正値（受理済み意味的不正）→ InvalidInput → exit 1

- **観点出典**: TP-003 §2.7 I-05; TP-014 §2.2 AF1; DD-002 §2.3 / §6（ContextError::InvalidInput → exit 1）
- **分類**: Unit
- **前提**: CLI clap value_parser を通過した後の経路で granularity が document/subnode 以外（意味的不正、構文層 reject されないケース）
- **入力**: `compile` 内の granularity 検証
- **期待**: `Err(ContextError::InvalidInput(_))`。`legixy-cli` が **exit 1**（受理済み引数の意味的不正、LGX-COMPAT-001 §3）。引数構文誤り（clap 層）の exit 2 とは区別
- **境界条件**: 意味的不正 = exit 1 / 構文誤り = exit 2 / size 超過 = exit 1

### ケース 28: 終了コード契約 0/1/2（LGX-COMPAT-001 凍結）

- **観点出典**: TP-003 §2.5 V-01/V-02; TP-009 §2.3-16; TP-012 §2.2 AF4; SPEC-003 §REQ.13/19/20, SPEC-LGX-004 §REQ.04, LGX-COMPAT-001 §3
- **分類**: Contract
- **前提**: (a) 正常・部分成功、(b) graph/config 読込失敗・InvalidInput・ResultTooLarge、(c) clap 構文誤り
- **入力**: それぞれ CLI ディスパッチ
- **期待**: (a)→**0**、(b)→**1**（`ContextError::Io`/`Graph`/`Db`/`InvalidInput`・`ResultTooLarge`〔v3 互換・stderr、DD-freeze 裁定 B-1〕）、(c)→**2**（clap 既定）。Block B 3 引数省略時は v0.3.0 以前と同一 argv・同一返却（後方互換、REQ.15）
- **境界条件**: exit 2 は構文層限定。監査ログ失敗は exit 0、ResultTooLarge と意味的不正は exit 1

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-003 §2.1 B-01 サイズ境界 | 境界値 | ケース 2, 3 |
| TP-003 §2.1 B-02 文字カウント単位 | 境界値 | ケース 4 |
| TP-003 §2.1 B-03 depth 範囲 | 境界値 | ケース 5, 6, 7 |
| TP-003 §2.1 B-04 depth 不正値 | 境界値 | ケース 19（MCP reject）, 27（意味的不正） |
| TP-003 §2.1 B-05 sections 件数 | 境界値 | ケース 8, 9 |
| TP-003 §2.1 B-06 target_files 件数決定性 | 境界値 | ケース 13, 26（決定論） |
| TP-003 §2.1 B-07 上流空 | 境界値 | ケース 1 |
| TP-003 §2.2 E-01 サイズ超過明示エラー | エラー | ケース 3 |
| TP-003 §2.2 E-02 エラー型区別 | エラー | ケース 3, 27, 28 |
| TP-003 §2.2 E-03 監査ログ書込失敗の本処理成否 | エラー | ケース 23 |
| TP-003 §2.2 E-04 サイズ超過もログ対象 | エラー | ケース 23（ベストエフォート記録）|
| TP-003 §2.2 E-05 部分成功（欠損）| エラー | ケース 16, 18 |
| TP-003 §2.3 S-01 DB 有無で内容変化 | 状態 | ケース 24 |
| TP-003 §2.3 S-02 pending Proposal 不変 | 状態 | ケース 25（FB-INV-3）|
| TP-003 §2.3 S-03 ステートレス性 | 状態 | ケース 25 |
| TP-003 §2.4 C-01/C-02/C-03 並行性 | 並行 | ケース 25（read-only）+ NFR REL.07/SEC.02 委譲 |
| TP-003 §2.5 V-01 granularity 既定 document | 互換 | ケース 14, 28 |
| TP-003 §2.5 V-02 Block B 省略時後方互換 | 互換 | ケース 28 |
| TP-003 §2.5 V-03 granularity カラム migration | 互換 | SPEC-LGX-008 へ委譲 |
| TP-003 §2.5 V-04 OS 横断バイト決定論 | 互換 | ケース 13（LF 固定）|
| TP-003 §2.6 P-01 DB 不在で graph.toml のみ | 永続化 | ケース 24 |
| TP-003 §2.6 P-02/P-03 書込 atomicity・ディスクフル | 永続化 | ケース 23 |
| TP-003 §2.7 I-01 未登録パス | 入力 | ケース 16, 17 |
| TP-003 §2.7 I-02 subnode 起点→親解決 | 入力 | TS-LGX-004 / DD-004 へ委譲 |
| TP-003 §2.7 I-03 親 ID への Info 診断 | 入力 | ケース 10 |
| TP-003 §2.7 I-04 sections 不正形式 | 入力 | ケース 9 |
| TP-003 §2.7 I-05 granularity 拒否 | 入力 | ケース 19, 27 |
| TP-003 §2.7 I-06 sections×document 無視 | 入力 | ケース 8（subnode のみ有効）|
| TP-003 §2.8 L-01 空状態返却 | ライフ | ケース 1 |
| TP-003 §2.8 L-02 outline 見出し皆無 | ライフ | ケース 11 |
| TP-003 §2.8 L-03 フィルタ結果空で親非登場 | ライフ | ケース 8 |
| TP-003 §2.8 L-04 depth 0 CLI/MCP 差 | ライフ | ケース 6, 19 |
| TP-003 §2.9 LOG-01/LOG-02 記録 | 観測 | ケース 23（記録経路）/ SPEC-LGX-007 委譲 |
| TP-003 §2.9 LOG-03 機密マスキング | 観測 | NFR SEC.05 / SPEC-LGX-007 委譲 |
| TP-003 §2.9 LOG-04 Info 診断時 stdout/exit 不変 | 観測 | ケース 6, 10, 16 |
| TP-003 §2.10 F-01 新ツール追加なし | FFI | ケース 20（オプション引数）|
| TP-003 §2.10 F-02 snake→kebab 転送 | FFI | ケース 20 |
| TP-003 §2.10 F-03 _meta 本文非改変 | FFI | ケース 21 |
| TP-003 §2.10 F-04 maxResultSizeChars 単位一致 | FFI | ケース 4, 21 |
| TP-003 §2.11 D-01 同一入力同一結果 | 決定論 | ケース 26 |
| TP-003 §2.11 D-02 バイト単位決定論 | 決定論 | ケース 13 |
| TP-003 §2.11 D-03 6 セクション順序固定 | 決定論 | ケース 14 |
| TP-003 §2.11 D-04 セクション内整列規則 | 決定論 | ケース 12（subnode）, 8 |
| TP-003 §2.11 D-05 マーカ 1 箇所固定 | 決定論 | ケース 15 |
| TP-003 §2.11 D-06 探索順/スコア順を整列に使わない | 決定論 | ケース 12 |
| TP-003 §2.11 D-07 CTX-INV-2/3/4 を破らない | 決定論 | ケース 25（read-only）/ SPEC-LGX-002 委譲 |
| TP-003 §2.11 D-08 フラグ組合せマトリクス | 決定論 | ケース 8, 11（sections 先・outline 後）, 28 |
| TP-012 §2.1 BF1〜BF4 基本フロー連鎖 | UC フロー | ケース 1, 14, 26 |
| TP-012 §2.2 AF1 代替 2a と REQ.20 整合 | UC フロー | ケース 16, 17 |
| TP-012 §2.2 AF2 上流空時構造維持 | UC フロー | ケース 1 |
| TP-012 §2.2 AF3 フラグ組合せ | UC フロー | ケース 8, 11 |
| TP-012 §2.2 AF4 depth 0 差分 | UC フロー | ケース 6, 19 |
| TP-012 §2.3 EF1 大規模返却エラー | UC フロー | ケース 3 |
| TP-012 §2.3 EF2 監査ログ書込失敗終端 | UC フロー | ケース 22, 23 |
| TP-012 §2.3 EF4 上流部分欠損 | UC フロー | ケース 18 |
| TP-012 §2.4 AT2 責任境界 | UC フロー | ケース 25 |
| TP-012 §2.5 DF3 DB 不在動作 | UC フロー | ケース 24 |
| TP-012 §2.6 R2 決定論保証 | UC フロー | ケース 13, 26 |
| TP-012 §2.6 R4 MCP-INV-2 忠実転送 | UC フロー | ケース 20, 21 |
| TP-014 §2.1 BF4 subnode 成功事後条件 | UC フロー | ケース 8, 26 |
| TP-014 §2.2 AF1 不正 granularity | UC フロー | ケース 19, 27 |
| TP-014 §2.2 AF2 fallback 発火 | UC フロー | TS-LGX-004 / DD-004 へ委譲（subnode 不在 fallback 本体）|
| TP-014 §2.2 AF3 フラグ適用順序 | UC フロー | ケース 8, 11（sections→outline）|
| TP-014 §2.3 EF1 Step3 失敗パス | UC フロー | ケース 18 |
| TP-014 §2.3 EF2 大規模返却（subnode）| UC フロー | ケース 3 |
| TP-014 §2.3 EF3 drift_score 付与失敗 | UC フロー | TS-LGX-007（ScoreLookup）へ委譲 |
| TP-014 §2.5 DF1 fallback 時フォーマット | UC フロー | TS-LGX-004 へ委譲（subnode fallback フォーマット）|
| TP-014 §2.5 DF2 subnode 整列規則 | UC フロー | ケース 12 |
| TP-014 §2.5 DF3 監査ログ記録ステップ | UC フロー | ケース 23 |
| TP-014 §2.6 R3 決定論具体化 | UC フロー | ケース 13 |
| TP-014 §2.6 R5 sections×document 無視 | UC フロー | ケース 8, 10 |
| TP-009 §2.1-3 snake→kebab 変換 | MCP | ケース 20 |
| TP-009 §2.1-5 忠実転送（フィルタなし）| MCP | ケース 20, 21 |
| TP-009 §2.2-8 target_files min1 | MCP | ケース 19 |
| TP-009 §2.2-9 depth≥1 境界 | MCP | ケース 19 |
| TP-009 §2.2-10 sections min length 1 | MCP | ケース 19 |
| TP-009 §2.2-12 maxResultSizeChars 単位/上限 | MCP | ケース 4, 21 |
| TP-009 §2.2-13 500000/500001 責務分界 | MCP | ケース 2, 3, 21 |
| TP-009 §2.3-16 exit 1/2 判別転送 | MCP | ケース 28 |
| TP-009 §2.4-21/22 zod 違反・CLI 起動前経路 | MCP | ケース 19 |
| TP-009 §2.4-24 argv 配列 injection 耐性 | MCP | ケース 20 |
| TP-009 §2.8-40/41/42 _meta 適用・本文非改変・責務 | MCP | ケース 21 |
| TP-009 §2.11-47 exit 0 非空 stderr → _meta.warnings | MCP | ケース 22 |
| TP-009 §2.5/2.6 子プロセス・タイムアウト・並行 | MCP | NFR / SPEC-LGX-009 所管へ委譲 |

> 継承 4 TP（TP-003/012/014/009）の観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。粒度別フロー本体（subnode 展開・fallback フォーマット）は TS-LGX-004 / DD-LGX-004、drift_score 数値妥当性は TS-LGX-007、context_log 記録項目仕様は SPEC-LGX-007、性能・並行・WAL/子プロセス管理は NFR / SPEC-LGX-009 へ委譲。本 TS は `legixy-ctx` の compile/render/walk/log の finding 生成・整列（A-1=アンカー出現順）・6 セクション順・キャッシュブレーク点・バイト決定論・ResultTooLarge(exit1/stderr、v3 互換)・exit 規約と、ts-mcp の忠実転送・`_meta` 付与に集中する。

## 4. テスト技法選択

- 同値分割: `Result<ContextResult, ContextError>` の `Ok` 系（正常・部分成功・空上流）と `Err` 系（ResultTooLarge / InvalidInput / Io / Graph）を別ケースに分離。granularity（Document/Subnode）・depth（0/1/None）・sections（空/1件/重複）の入力空間を分割。
- 境界値分析: サイズ上限 500,000 / 500,001（ケース 2/3）、depth 0/1/無制限（ケース 5/6/7）、sections 0件/1件/重複（ケース 9）、文字カウント単位（ケース 4）。
- Property-based: バイト単位決定論（ケース 13）、subnode 整列の決定論＝アンカー出現順（ケース 12）、冪等性（ケース 26）、read-only 不変（ケース 25）を不変条件として property 化。
- 状態遷移: DB 有無（ケース 24）、監査ログ書込成否（ケース 23）、起点解決の全/部分/未登録（ケース 16/17/18）。

## 5. テスト基盤

- 言語: Rust（`legixy-ctx`） + TypeScript（`ts-mcp` 転送層）
- フレームワーク: cargo test（Rust） / Vitest（TS MCP）
- Property-based: proptest（Rust ケース 12/13/25/26） / fast-check（TS、ケース 19/20/21/22 の入力生成）
- モック: engine.db は一時 SQLite fixture（読取専用 DB で書込失敗を再現、ケース 23）。ts-mcp は Rust CLI 子プロセスを spawn スタブ化し argv・stderr・exit code を観測（ケース 19〜22）。グラフは graph.toml fixture を借用（read-only）。

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1〜18, 23〜28 | TC-LGX-002（Rust） | `legixy-ctx/tests/*.rs` |
| ケース 11 | TC-LGX-002（T-CC-OUTLINE-001 相当） | `legixy-ctx/tests/outline.rs` |
| ケース 8 | TC-LGX-002（T-CC-SECTIONS-001 相当） | `legixy-ctx/tests/sections.rs` |
| ケース 5 | TC-LGX-002（T-CC-DEPTH-001 相当） | `legixy-ctx/tests/depth.rs` |
| ケース 12, 13, 25, 26 | TC-LGX-002（proptest） | `legixy-ctx/tests/determinism.rs` |
| ケース 19〜22 | TC-LGX-002-TS | `ts-mcp/test/compile-context.test.ts` |
