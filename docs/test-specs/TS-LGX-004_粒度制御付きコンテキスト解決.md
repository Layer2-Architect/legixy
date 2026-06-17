Document ID: TS-LGX-004

# TS-LGX-004: 粒度制御付きコンテキスト解決（context --granularity / --outline-only / --sections / --depth）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-004 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-004
**継承 TP**: TP-LGX-003（TP[SPEC] コンテキスト解決、観点 51 件）, TP-LGX-014（TP[UC] UC-004 粒度制御フロー、観点 22 件）。DD-LGX-004 §8（テスト分類）は TP-LGX-003 を主、TS-LGX-002（未作成・委譲先）を従として引用する。

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-004 の関数 / 型（**粒度制御固有の担当範囲**。DD-LGX-002 凍結の基盤型は委譲）:

- DD-LGX-004 §2.1 `Granularity`{Document, Subnode}（`as_str()` → `"document"` / `"subnode"`、`Default = Document`）
- DD-LGX-004 §2.1 `CompileInput` の粒度制御フィールド（`granularity` / `outline_only` / `sections: Option<Vec<String>>` / `depth_limit: Option<usize>`、`Default`）
- DD-LGX-004 §2.2 `SectionFormatter::upstream_sort_rule(granularity) -> &'static str`（`Document → "artifact_id-asc"` / `Subnode → "parent_id-asc,anchor-appearance-order"`）, `RENDER_SORT_STRATEGY`
- DD-LGX-004 §3 `build_outline(content: &str) -> String`（pub(crate)、REQ.15 ATX h1〜h3 抽出・インデント）
- DD-LGX-004 §3 `UpstreamWalker::walk_chain_parent_only_with_depth(&self, start: &NodeId, depth_limit: Option<usize>) -> Result<Vec<UpstreamArtifact>, ContextError>`（REQ.17 depth 制御）
- DD-LGX-004 §3 `SectionFormatter::render(result: &ContextResult) -> Result<String, ContextError>`（REQ.10-14 の 6 セクション整列・マーカ・サイズ上限）
- DD-LGX-004 §3 `SectionFormatter::enforce_size_limit(rendered: &str) -> Result<(), ContextError>`（REQ.13 / CACHE-INV-3 defence-in-depth）
- DD-LGX-004 §3 `ContextCompiler::compile` / `render` の **粒度制御に係る振る舞い**（subnode 展開・sections dedup/trim・depth・3a fallback・REQ.19 audit ベストエフォート）。両関数のシグネチャ自体は DD-LGX-002 で凍結（本 TS は引用のみ）
- DD-LGX-004 §3 定数 `RESULT_SIZE_LIMIT_CHARS = 500_000` / `CACHE_BREAKPOINT_MARKER = "<!-- cache-breakpoint: stable-end -->"`
- DD-LGX-004 §2.3 `ContextError::ResultTooLarge { current, limit }`（粒度制御固有 variant）

委譲（本 TS 対象外。DD-LGX-004 §10/§11 のとおり）:
- `ContextCompiler::compile` / `render` の基盤シグネチャ・`ContextResult` / `UpstreamArtifact` 構造・`ContextError` の非粒度 variant（Graph/InvalidInput/Db/Io/Serde）・Layer/Additional/Custom Documents セクションの解決そのもの・`context_log` 書込みの atomicity → **TS-LGX-002 へ委譲**（DD-LGX-002 凍結。本 TS は粒度制御が触れる範囲のみ検証）。
- 意味層 drift_score の数値妥当性・embedding 不在時のスコア欠落挙動 → **TS-LGX-007 へ委譲**（drift 検出仕様）。
- 性能予算 PERF.03（subnode 100 件 < 300ms / < 200ms）・PERF.09 → **bench / NFR-LGX-001 へ委譲**。
- 並行アクセス整合性 REL.07（busy_timeout 5000ms）・SEC.02（並行呼出し安全性）・C-02/C-03 の DB 排他 → **NFR-LGX-001 へ委譲**。
- MCP 境界での snake→kebab 変換・`_meta` 付与・zod reject（depth<1）→ **SPEC-LGX-009 / TS-LGX-009 へ委譲**（本 TS は CLI 側の受理範囲のみ検証）。
- CLI 引数構文層（clap value_parser による `--depth` 非数値/小数 reject = exit 2、granularity enum reject）→ **CLI 層 TS（DD-LGX-002 / legixy-cli）へ委譲**（本 TS は受理済み値の意味的振る舞いに集中）。

本 TS は「DD-LGX-004 が SPEC-003 の粒度制御規定（REQ.03/10-20）を DD-004 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: `Granularity` 既定値が Document・2 値のみ・`as_str` 一致

- **観点出典**: TP-003 §2.5 V-01（granularity 既定 document / auto 排除）, TP-014 §2.2 AF1（document/subnode の 2 値網羅）
- **分類**: Unit
- **前提**: `Granularity` 列挙型と `CompileInput::default()`
- **入力**: `Granularity::default()`、`Granularity::Document.as_str()`、`Granularity::Subnode.as_str()`、`CompileInput::default().granularity`
- **期待**: `Granularity::default() == Granularity::Document`。`Document.as_str() == "document"`、`Subnode.as_str() == "subnode"`。`CompileInput::default().granularity == Granularity::Document`。variant は 2 値のみ（`auto` は存在しない）
- **境界条件**: 既定 = Document（v0.1.0 互換）。`as_str` 値は context_log カラム値 / CLI 引数文字列と一致

### ケース 2: `CompileInput::default()` の粒度制御フィールド省略時の値（v0.3.0 後方互換）

- **観点出典**: TP-003 §2.5 V-02（Block B 引数省略時の後方互換）
- **分類**: Unit
- **前提**: `CompileInput::default()`
- **入力**: `CompileInput::default()`
- **期待**: `outline_only == false`、`sections == None`、`depth_limit == None`、`granularity == Document`、`command == None`、`target_files == vec![]`
- **境界条件**: Block B 引数省略 = v0.3.0 以前と同一返却（None/false が「フィルタなし / 無制限 / 全文」を意味）

### ケース 3: `upstream_sort_rule` の granularity 分岐（A-1 裁定: 出現順）

- **観点出典**: TP-003 §2.11 D-04（Upstream 整列規則）, TP-014 §2.5 DF2（subnode 整列規則の反映）
- **分類**: Unit
- **前提**: `SectionFormatter::upstream_sort_rule`（DD-LGX-004 §2.2、v1.1 A-1 裁定確定）
- **入力**: `upstream_sort_rule(Granularity::Document)`、`upstream_sort_rule(Granularity::Subnode)`
- **期待**: `Document → "artifact_id-asc"`。`Subnode → "parent_id-asc,anchor-appearance-order"`（**アンカー出現順**。v3 の `anchor-bytes-asc` ではない — DD-freeze 裁定 A-1）。`RENDER_SORT_STRATEGY == "index-array"`
- **境界条件**: subnode 整列キーは SPEC-LGX-003.REQ.11 準拠の出現順。バイト辞書順を返してはならない（回帰の固定）

### ケース 4: `build_outline` h1〜h3 抽出・レベル別インデント

- **観点出典**: TP-003 §2.8 L-02（outline-only 出力形式）, TP-014 §2.2 AF3（4-A outline 化）
- **分類**: Unit
- **前提**: `build_outline(content: &str) -> String`（DD-LGX-004 §3、REQ.15）
- **入力**: `"# H1\n本文a\n## H2\n本文b\n### H3\n"`
- **期待**: `"- H1\n  - H2\n    - H3\n"`（h1 インデント 0、h2 = `"  "`×1、h3 = `"  "`×2。`"- {title}"` 形式。本文行 `本文a` / `本文b` は含まれない）
- **境界条件**: インデントは `"  " × (level - 1)`。本文（見出し以外）は除外

### ケース 5: `build_outline` h4 以降・スペース無し・空タイトルの除外（境界: 下限/対象外）

- **観点出典**: TP-003 §2.8 L-02, §2.7 I-04（縮退入力）
- **分類**: Unit
- **前提**: `build_outline`
- **入力**: `"#### H4\n#abc\n#\n#   \n## valid\n"`
- **期待**: `"  - valid\n"` のみ（`#### H4` は h4+ で除外、`#abc` はスペース無しで除外、`#`（空タイトル）除外、`#   `（trim 後空タイトル）除外。残るのは h2 `valid` でインデント `"  "`×1）
- **境界条件**: h4+ 除外 / `# title` 形式（スペース必須）/ 空タイトル除外。h3 が上限・h4 が上限+1

### ケース 6: `build_outline` 見出し皆無時の空文字列（GAP-047 / S2-25）

- **観点出典**: TP-003 §2.8 L-02（見出し皆無時の出力）, TP-014 §2.5 DF1（fallback フォーマット）
- **分類**: Unit
- **前提**: `build_outline`
- **入力**: `"本文のみ。見出しなし。\n#### only h4\n#nospace\n"`
- **期待**: `""`（空文字列。h1〜h3 が 1 つも無いため。プレースホルダ文字列を挿入しない）
- **境界条件**: 見出し皆無 = 空 body の正準。artifact 枠は `SectionFormatter::render` 側で維持（ケース 13 で検証）

### ケース 7: `enforce_size_limit` 境界 499,999 / 500,000 / 500,001 文字

- **観点出典**: TP-003 §2.1 B-01（500,000 / 500,001 の成功/エラー境界）, §2.2 E-01
- **分類**: Unit
- **前提**: `SectionFormatter::enforce_size_limit(rendered: &str)`、`RESULT_SIZE_LIMIT_CHARS == 500_000`
- **入力**: (a) 499,999 文字の文字列、(b) ちょうど 500,000 文字、(c) 500,001 文字
- **期待**: (a) → `Ok(())`、(b) → `Ok(())`（「超える場合」エラーのため 500,000 ちょうどは成功）、(c) → `Err(ContextError::ResultTooLarge { current: 500_001, limit: 500_000 })`
- **境界条件**: 下限境界 0、上限 500,000（含む）、上限+1 = 500,001（エラー）。判定は `rendered.chars().count() > 500_000`

### ケース 8: 文字カウント単位 = Unicode コードポイント（サロゲート・結合・ZWJ）

- **観点出典**: TP-003 §2.1 B-02（カウント単位）, §2.10 F-04（単位一致）
- **分類**: Unit
- **前提**: `enforce_size_limit`。カウントは `.chars().count()`（Unicode コードポイント数、REQ.13）
- **入力**: マルチバイト・サロゲートペア（絵文字 😀 等）・結合文字・ZWJ を含み **コードポイント数 == 500,001** だがバイト数 / grapheme 数は異なる文字列
- **期待**: `Err(ResultTooLarge { current: 500_001, .. })`（コードポイント数で判定。バイト数や grapheme 数では判定しない）
- **境界条件**: 「文字」= Unicode コードポイント（SPEC-LGX-009 REQ.13 と同単位。grapheme・バイトと区別）

### ケース 9: `walk_chain_parent_only_with_depth` depth=Some(1) で直接親のみ

- **観点出典**: TP-003 §2.1 B-03（depth 返却範囲）, TP-014 §2.2 AF3（4-C depth_limit）
- **分類**: Integration
- **前提**: `SPEC → UC → RBA → … → SRC` の chain を持つグラフ。start = ある下流ノード
- **入力**: `walk_chain_parent_only_with_depth(&start, Some(1))`
- **期待**: `Ok(vec)` で `chain_distance == 1` の直接親のみを含む。`chain_distance >= 2` のノードは除外
- **境界条件**: N=1 = 直接親（chain_distance == 1）まで。Chain/ParentChild エッジのみ逆方向 BFS

### ケース 10: depth=Some(2) で祖父まで / depth=None で無制限

- **観点出典**: TP-003 §2.1 B-03, §2.5 V-02（省略時無制限 = v0.2.0 互換）
- **分類**: Integration
- **前提**: ケース 9 と同グラフ
- **入力**: (a) `walk_chain_parent_only_with_depth(&start, Some(2))`、(b) `..(&start, None)`
- **期待**: (a) → `chain_distance <= 2` のノードを含む（祖父まで）。(b) → 全祖先（無制限、7 階層全て）を返す
- **境界条件**: N=2 = chain_distance ≤ 2、None = 無制限。N と返却深さの単調対応

### ケース 11: depth=Some(0) で空集合・正常終了（exit 0、REQ.17 v3 差分）

- **観点出典**: TP-003 §2.1 B-03, §2.8 L-04（depth 0 受理範囲）, §2.9 LOG-04（Info 時 stdout/exit 不変）
- **分類**: Integration
- **前提**: 任意のグラフ。start に祖先が存在する
- **入力**: (a) `walk_chain_parent_only_with_depth(&start, Some(0))`、(b) CLI 経由 `legixy context <file> --depth 0`
- **期待**: (a) → `Ok(vec![])`（空 Vec）。(b) → exit 0、stdout は空 upstream のセクション構成を維持、stderr に Info 診断 `Info: --depth 0 results in empty upstream (no ancestors returned).`（DD-LGX-004 §11 S2-23）。stdout・終了コードは Info 出力により変化しない
- **境界条件**: N=0 = 空集合 + exit 0（CLI 受理）。MCP zod reject（depth≥1）は SPEC-LGX-009 / TS-LGX-009 へ委譲（受理範囲差は正準）

### ケース 12: subnode 粒度の 6 セクション順・整列・キャッシュマーカ（バイト決定論）

- **観点出典**: TP-003 §2.11 D-03（6 セクション配置順序）, D-04（整列）, D-05（マーカ 1 箇所）, TP-014 §2.1 BF3/BF4（返却構造）
- **分類**: Integration
- **前提**: subnode を持つ上流を含むグラフ。`granularity = Subnode`。`compile` → `render`（= `SectionFormatter::render`）の 2 段
- **入力**: `compile(&CompileInput{ granularity: Subnode, target_files: vec![target], ..default() })` の結果を `render`
- **期待**: 返却本文が 6 セクションをこの順で構成: (1) Layer Guidelines（パス辞書順）→ (2) Additional Guidelines（パス辞書順）→ (3) `<!-- cache-breakpoint: stable-end -->`（== `CACHE_BREAKPOINT_MARKER`、本文中 1 箇所のみ・固定位置）→ (4) Upstream Artifacts（親 ID 辞書順 + 同一ドキュメント内アンカー出現順）→ (5) Target Node Metadata → (6) Custom Documents（from_id→to_id 辞書順）。各 UpstreamArtifact は `artifact_id:` / `type:` / `file_path:` / `chain_distance:` / `subnode_id:` / `anchor:` / `drift_score:` ヘッダ + 空行 + body の形式（DD-LGX-004 §11 S2-01）
- **境界条件**: セクション順は granularity 非依存で固定。subnode 粒度では subnode_id が Some、整列は出現順（A-1）

### ケース 13: outline-only × subnode = anchor のみ / 見出し皆無 artifact の枠維持・空 body

- **観点出典**: TP-003 §2.8 L-02, TP-014 §2.2 AF3（4-A）, §2.5 DF1
- **分類**: Integration
- **前提**: `granularity = Subnode`、`outline_only = true`。一部 subnode は h1〜h3 を持ち、一部は見出し皆無
- **入力**: `compile(&CompileInput{ granularity: Subnode, outline_only: true, target_files: vec![target], ..default() })` → `render`
- **期待**: subnode 粒度 × outline_only では各 subnode artifact の body は **anchor 文字列のみ**（REQ.15 末尾）。見出し皆無 subnode（anchor も無い縮退ケース含む）は **artifact ヘッダ枠を維持し body 空**（ヘッダ行群 + 空行 + 末尾改行のみ、S2-25）。プレースホルダ文字列なし
- **境界条件**: outline × subnode = anchor のみ。見出し皆無 = 枠維持 + 空 body（REQ.10 件数非依存と整合）

### ケース 14: sections フィルタ — 存在 ID のみ通過 / 不在 ID 除外（混在）

- **観点出典**: TP-003 §2.1 B-05（sections 1 件以上）, §2.8 L-03（フィルタ通過空時の不在）, TP-014 §2.2 AF3（4-B）
- **分類**: Integration
- **前提**: `granularity = Subnode`。グラフに subnode `DD-X-001#abc` が存在、`DD-X-001#zzz` は不在
- **入力**: `compile(&CompileInput{ granularity: Subnode, sections: Some(vec!["DD-X-001#abc".into(), "DD-X-001#zzz".into()]), target_files: vec![target], ..default() })`
- **期待**: upstream に `DD-X-001#abc` の subnode artifact のみ含む。不在 `#zzz` はエラーにせず単に除外。ある親ドキュメントの通過 subnode が 0 件なら当該親は upstream に登場しない（REQ.16）
- **境界条件**: 完全一致フィルタ。不在 ID = 除外（エラー昇格しない）

### ケース 15: sections — 全 ID 不在 / 空トークン dedup・trim（境界: 0 件結果）

- **観点出典**: TP-003 §2.7 I-04（空トークン `a,,b`・空白・重複）, §2.1 B-05（空文字列）, §2.11 D-02（バイト決定論保全）
- **分類**: Integration
- **前提**: `granularity = Subnode`
- **入力**: (a) `sections: Some(vec!["#nope1".into(), "#nope2".into()])`（全不在）、(b) compile に渡る前に trim 後空トークン（`""`, `"  "`, 連続コンマ由来）除去・重複 dedup 済みの `sections`（DD-LGX-004 §3 注記: 渡す前に trim/dedup 完了）
- **期待**: (a) → 空 upstream で正常終了（exit 0）。(b) → 重複指定・指定順は出力に影響せず、返却は REQ.11 整列（親 ID 辞書順 + アンカー出現順）に従いバイト決定論（CACHE-INV-1）を保全
- **境界条件**: 全無効/空 = 空 upstream exit 0。set セマンティクス（指定順・回数非依存）

### ケース 16: sections に親ドキュメント ID（`#` なし）→ 除外 + Info 診断

- **観点出典**: TP-003 §2.7 I-03（親 ID 除外 + Info）, §2.9 LOG-04（Info 時 stdout/exit 不変）, TP-014 §2.2 AF3
- **分類**: Integration
- **前提**: `granularity = Subnode`。CLI 経由実行
- **入力**: `legixy context <file> --granularity subnode --sections "DD-X-001"`（`#` を含まない親ドキュメント ID）
- **期待**: 当該 ID は subnode ID と一致しないため除外（エラーにしない）。exit 0。stderr に Info 診断 `Info: --sections received a document-level ID 'DD-X-001', not a subnode ID. Use '#'-containing subnode IDs instead.`（DD-LGX-004 §11 S2-23）。stdout・終了コードは不変
- **境界条件**: 親 ID（`#` なし）= 除外 + Info。subnode ID（`#` 付き）と区別

### ケース 17: sections × document 粒度 = sections 無視（REQ.18）

- **観点出典**: TP-003 §2.7 I-06（sections×document 無視）, §2.11 D-08（フラグ組合せ）, TP-014 §2.6 R5
- **分類**: Integration / フラグ組合せ
- **前提**: `granularity = Document`、`sections = Some(vec![...])`
- **入力**: `compile(&CompileInput{ granularity: Document, sections: Some(vec!["DD-X-001#abc".into()]), target_files: vec![target], ..default() })`
- **期待**: `sections` は無視され、document 粒度の全文 upstream を返す（フィルタ非適用）。返却は `granularity = Document, sections = None` と同一バイト列
- **境界条件**: sections は subnode 粒度時のみ有効。document 時は無視（REQ.16/18）

### ケース 18: フラグ組合せ優先順位マトリクス（REQ.18: sections→outline の順）

- **観点出典**: TP-003 §2.11 D-08（組合せマトリクス一意確定）, TP-014 §2.2 AF3（適用順序）
- **分類**: フラグ組合せ（Integration）
- **前提**: 各組合せのグラフ。DD-LGX-004 §8 フラグ組合せ表
- **入力**: (a) `outline_only=true × document`、(b) `sections=Some × document`、(c) `outline_only=true × sections=Some × subnode`、(d) `depth_limit=Some(1)` を各組合せに直交適用
- **期待**: (a) → 各 artifact 本文を見出し階層リストに置換（全文ではない）。(b) → sections 無視（ケース 17）。(c) → **sections フィルタが先、outline 化が後**（sections で絞った subnode の body を anchor のみに置換）。(d) → depth は各組合せに直交し独立に適用（REQ.18）
- **境界条件**: 組合せ時の適用順序が一意に確定（sections→outline、depth 直交）

### ケース 19: subnode 粒度 + サブノード不在上流 → document fallback（代替 3a）

- **観点出典**: TP-014 §2.2 AF2（fallback 発火条件）, §2.4 AT2（subnode 皆無時の責任境界）, §2.5 DF1
- **分類**: Integration
- **前提**: `granularity = Subnode`。上流ノード A はサブノードエッジを持つ、上流ノード B はサブノードが 0 件（`subnodes_of(&B).is_empty()`）
- **入力**: `compile(&CompileInput{ granularity: Subnode, target_files: vec![target], ..default() })`
- **期待**: ノード A は subnode 単位で個別展開（`subnode_id = Some`）。ノード B は **ノード単位で document 粒度同様に全文返却**（`subnode_id = None`、3a fallback）。fallback はノード単位で判定・適用され、アクターへのエラー通知は不要
- **境界条件**: fallback 発火 = ノード単位で `subnodes_of(parent).is_empty()`。subnode 有/無の混在グラフで両挙動が共存

### ケース 20: ResultTooLarge エラーメッセージ固定（REQ.13、提案文あり）

- **観点出典**: TP-003 §2.2 E-01（切り捨てず明示エラー + 提案）, §2.2 E-02（型付け）
- **分類**: Unit / Contract
- **前提**: `ContextError::ResultTooLarge` の Display 実装（DD-LGX-004 §2.3 / §11 S2-02、v3 `error.rs:12-15` 整合）
- **入力**: `ResultTooLarge { current: 500_001, limit: 500_000 }` の Display 出力
- **期待**: SPEC-LGX-003.REQ.13 の固定文字列に一致: `Error: compile_context result exceeds 500,000 characters.` + `Current size: 500001 characters.` + 提案文（`Try --granularity subnode for finer-grained retrieval.` / `Narrow the target scope.`）。切り捨て・自動要約をしない（明示エラー）。終了コードは ContextError → exit 1
- **境界条件**: メッセージ固定（提案文必須）。サイズ超過 = 型付きエラー（exit 1）

### ケース 21: 監査ログ書込失敗 → 本処理成功 exit 0 + stderr Warning（REQ.19、v3 差分）

- **観点出典**: TP-003 §2.2 E-03（書込失敗時の本処理成否）, §2.9 LOG-01, TP-014 §2.5 DF3（監査記録ステップ）
- **分類**: Integration
- **前提**: `compile()` の本処理（上流解決）は成功するが、`audit.log(input, &result)` が `Err` を返す状況（DB ロック / 権限 / ディスクフル）
- **入力**: 監査書込が失敗する条件下で `compile(&input)`
- **期待**: `compile` は `Ok(ContextResult)` を返す（本処理優先・audit ベストエフォート、`if let Err(e) = audit.log(..) { eprintln!(...) }` で Ok 維持。v3 は `?` 伝搬 = 差分）。CLI 経由では exit 0、stderr に `[legixy-ctx] Warning: audit log write failed (best-effort): {e}`（DD-LGX-004 §11 S2-23）。stdout の本文は不変
- **境界条件**: audit 失敗は本処理成否に影響しない（exit 0、REQ.19）。Db エラーでも compile() 内部での非 audit 経路は伝搬（exit 1）

### ケース 22: 起点ノード不在・上流部分欠損 → 部分成功 exit 0（REQ.20、決定論的記録）

- **観点出典**: TP-003 §2.7 I-01（起点不在）, §2.2 E-05（部分欠損）, TP-014 §2.3 EF1（Step3 失敗パス）
- **分類**: Integration
- **前提**: (a) target_files に graph.toml 上にノードを持たないパスを含む、(b) 上流連鎖途中のノードのファイルが不在
- **入力**: (a) `compile(&CompileInput{ target_files: vec![known, unknown], ..default() })`、(b) 連鎖途中が欠損するグラフで compile
- **期待**: (a) → 不在 target は無視して残りで解決、`Ok(ContextResult)` exit 0。未解決は Target Node Metadata 末尾の `unresolved_targets:`（パス辞書順）に決定論的に記録（S2-24）。CLI 経由は stderr Info `Info: the following target paths were not found in the graph and were skipped: {paths}`。(b) → 欠損ノードを飛ばして残りの上流を返す部分成功 exit 0。欠損ノードは空 body で継続（`enrich_upstream` の `let Some(node) = .. else { return Ok(art) }`）
- **境界条件**: 起点不在 / 部分欠損 = 部分成功 exit 0。欠損記録は決定論的（CACHE-INV-1 保全）

### ケース 23: 同一入力 → 同一バイト列（property、CACHE-INV-1 / CTX-INV-1）

- **観点出典**: TP-003 §2.11 D-01（CTX-INV-1）, D-02（バイト決定論）, §2.3 S-03（ステートレス）, TP-014 §2.6 R3
- **分類**: Property-based（proptest）
- **生成器**: `granularity ∈ {Document, Subnode}`、`outline_only ∈ {true, false}`、`sections ∈ {None, Some(任意の subnode ID 集合)}`、`depth_limit ∈ {None, Some(0..=7)}`、`target_files`（既知ノードの部分集合）をランダム生成
- **不変条件**: 同一 `CompileInput` に対し `compile` → `render`（= `SectionFormatter::render`）は常に同一バイト列を返す。複数回呼出し・呼出し順に依らずバイト一致（順序・区切り・空白を含む）。`compile` は read-only（context_log 以外の永続状態を変更しない）
- **反例ハンドリング**: shrink して最小の入力（最少 target / 最短 sections）で記録。OS 横断の改行は LF 統一前提（NFR COMPAT.08、V-04 は委譲）

### ケース 24: read-only 不変（粒度制御呼出しがグラフ/DB を変更しない）

- **観点出典**: TP-003 §2.3 S-01/S-03（ステートレス・FB-INV-4）, TP-014 §2.4 AT1
- **分類**: Property / Integration
- **前提**: 任意の `CompileInput`（document / subnode、各フラグ）。engine.db 存在ケースと不在ケース
- **入力**: `compile` 実行前後の graph.toml / engine.db のハッシュ
- **期待**: 実行前後で graph.toml が不変。engine.db は context_log 書込以外不変（DB 不在時は graph.toml のみで上流を返す、FB-INV-4）。`ContextCompiler<'a>` は借用のみ（read-only、DD-LGX-004 §5）。granularity・各フラグは権限要件・状態を変えない
- **境界条件**: 借用による read-only。副作用は context_log 書込のみ（その atomicity は TS-LGX-002 へ委譲）

## 3. 観点カバレッジ表

### TP-LGX-003（TP[SPEC]）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| §2.1 B-01 サイズ境界 500,000/500,001 | 境界値 | ケース 7 |
| §2.1 B-02 カウント単位 | 境界値 | ケース 8 |
| §2.1 B-03 depth 返却範囲 | 境界値 | ケース 9, 10, 11 |
| §2.1 B-04 depth 不正値（非数値/小数/負） | 境界値 | CLI 層 TS（clap value_parser、exit 2/1）へ委譲 |
| §2.1 B-05 sections 0/1/大量 | 境界値 | ケース 14, 15 |
| §2.1 B-06 target_files 件数の決定性 | 境界値 | ケース 23（property） |
| §2.1 B-07 上流空（最上流起点） | 境界値 | ケース 11（空 upstream 構成維持）, 22 |
| §2.2 E-01 切り捨てず明示エラー+提案 | エラー | ケース 20 |
| §2.2 E-02 エラー型付け | エラー | ケース 20（ResultTooLarge）, 21（Db 伝搬境界） |
| §2.2 E-03 書込失敗時の本処理成否 | エラー | ケース 21 |
| §2.2 E-04 サイズ超過の監査記録 | エラー | TS-LGX-002 / TS-LGX-009 へ委譲（audit 記録仕様は SPEC-LGX-007） |
| §2.2 E-05 部分成功（読込失敗/欠損） | エラー | ケース 22 |
| §2.3 S-01 DB 有/無で返却不変 | 状態 | ケース 24（FB-INV-4） |
| §2.3 S-02 pending Proposal 不変 | 状態 | TS-LGX-002 へ委譲（FB-INV-3、非粒度） |
| §2.3 S-03 ステートレス性 | 状態 | ケース 23, 24 |
| §2.4 C-01 並行呼出し独立性 | 並行 | NFR-LGX-001 / SEC.02 へ委譲 |
| §2.4 C-02 並行書込み排他（WAL） | 並行 | NFR-LGX-001 / REL.07 へ委譲 |
| §2.4 C-03 busy_timeout 超過挙動 | 並行 | NFR-LGX-001 / REL.07 へ委譲 |
| §2.5 V-01 granularity 既定/2値/auto排除 | 互換 | ケース 1 |
| §2.5 V-02 Block B 引数省略後方互換 | 互換 | ケース 2, 10 |
| §2.5 V-03 context_log granularity migration | 互換 | SPEC-LGX-008 / TS-LGX-002 へ委譲 |
| §2.5 V-04 OS 横断改行正規化 | 互換 | NFR COMPAT.07/08 へ委譲（ケース 23 は LF 統一前提） |
| §2.6 P-01 DB 不在で graph.toml 上流 | 永続化 | ケース 24（FB-INV-4） |
| §2.6 P-02 context_log atomicity | 永続化 | TS-LGX-002 へ委譲（書込 Tx 境界、非粒度） |
| §2.6 P-03 ディスクフル/権限時挙動 | 永続化 | ケース 21（書込失敗 = ベストエフォート） |
| §2.7 I-01 起点ノード不在 | 入力 | ケース 22 |
| §2.7 I-02 サブノード起点→親上流解決 | 入力 | TS-LGX-002 へ委譲（REQ.08 上流解決セマンティクス） |
| §2.7 I-03 親 ID sections 除外+Info | 入力 | ケース 16 |
| §2.7 I-04 sections 不正形式（空/重複） | 入力 | ケース 5, 15 |
| §2.7 I-05 granularity 不正値拒否 | 入力 | CLI 層 TS（clap enum）/ TS-LGX-009（zod）へ委譲（ケース 1 で 2 値性は確認） |
| §2.7 I-06 sections×document 無視 | 入力 | ケース 17 |
| §2.8 L-01 上流ゼロ時の構成 | ライフ | ケース 11, 22 |
| §2.8 L-02 outline 見出し皆無時出力 | ライフ | ケース 6, 13 |
| §2.8 L-03 sections 通過空で親不在 | ライフ | ケース 14 |
| §2.8 L-04 depth 0 / MCP reject 受理範囲差 | ライフ | ケース 11（CLI 側）/ MCP 側は TS-LGX-009 へ委譲 |
| §2.9 LOG-01 全呼出し記録 | 観測 | ケース 21（記録試行・失敗時 Warning） |
| §2.9 LOG-02 記録項目十分性 | 観測 | TS-LGX-002 / SPEC-LGX-007 へ委譲（context_log 記録仕様） |
| §2.9 LOG-03 機密情報マスキング | 観測 | NFR SEC.05 / SPEC-LGX-007 へ委譲 |
| §2.9 LOG-04 Info 時 stdout/exit 不変 | 観測 | ケース 11, 16, 21 |
| §2.10 F-01 新ツール非追加 | 境界 API | TS-LGX-009 へ委譲（MCP-INV-1、ケース 1 でオプション引数性は含意） |
| §2.10 F-02 snake→kebab 変換・転送順序 | 境界 API | SPEC-LGX-009 / TS-LGX-009 へ委譲 |
| §2.10 F-03 `_meta` maxResultSizeChars 非改変 | 境界 API | TS-LGX-009 へ委譲（サイズ判定責務は Rust CLI = ケース 7/8） |
| §2.10 F-04 maxResultSizeChars 単位一致 | 境界 API | ケース 8（コードポイント単位） |
| §2.11 D-01 同一入力同一順序（CTX-INV-1） | 決定論 | ケース 23 |
| §2.11 D-02 バイト単位決定論 | 決定論 | ケース 12, 15, 23 |
| §2.11 D-03 6 セクション配置順固定 | 決定論 | ケース 12 |
| §2.11 D-04 各セクション整列規則 | 決定論 | ケース 3, 12 |
| §2.11 D-05 キャッシュマーカ 1 箇所固定 | 決定論 | ケース 12 |
| §2.11 D-06 探索順/スコア順を整列に使わない | 決定論 | ケース 3, 12（Metadata 隔離） |
| §2.11 D-07 CTX-INV-2/3/4 を破らない | 決定論 | ケース 24（read-only）/ 本体は SPEC-LGX-002・TS-LGX-002 へ委譲 |
| §2.11 D-08 フラグ組合せマトリクス | 決定論 | ケース 17, 18 |

### TP-LGX-014（TP[UC]）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| §2.1 BF1 事前条件検証タイミング | UC フロー | ケース 19（subnode 有無の動的判定 = 3a fallback） |
| §2.1 BF2 委譲後の接続点 | UC フロー | TS-LGX-002 へ委譲（UC-002 上流解決の接続点） |
| §2.1 BF3 返却構造と ContextResult 整合 | UC フロー | ケース 12（UpstreamArtifact 拡張フィールド） |
| §2.1 BF4 subnode 成功事後条件 | UC フロー | ケース 12, 13 |
| §2.2 AF1 granularity 全 case/不正値 | UC フロー | ケース 1（2 値性）/ 不正値は CLI 層 TS へ委譲 |
| §2.2 AF2 3a fallback 発火条件 | UC フロー | ケース 19 |
| §2.2 AF3 フラグ適用順序（4-A〜4-D） | UC フロー | ケース 18 |
| §2.2 AF4 4-D と基本フロー整合 | UC フロー | ケース 12, 19（subnode 個別展開） |
| §2.2 AF5 1a 事後条件収束 | UC フロー | ケース 17（document = UC-002 同等）/ 詳細は TS-LGX-002 |
| §2.3 EF1 Step3 失敗パス | UC フロー | ケース 22（部分欠損・content_range 系） |
| §2.3 EF2 大規模返却エラー（subnode 展開後） | UC フロー | ケース 20 |
| §2.3 EF3 drift_score 付与失敗 | UC フロー | TS-LGX-007 へ委譲（embedding 不在・スコア欠落） |
| §2.4 AT1 アクター権限の粒度間一貫性 | アクター | ケース 24（権限/状態不変） |
| §2.4 AT2 subnode 皆無時の責任境界 | アクター | ケース 19（自動 fallback、通知不要） |
| §2.5 DF1 fallback 時 UpstreamArtifact 形式 | データ | ケース 13, 19（subnode_id=None） |
| §2.5 DF2 整列規則の反映 | データ | ケース 3, 12 |
| §2.5 DF3 監査ログ記録ステップ | データ | ケース 21 |
| §2.6 R1 Phase 依存挙動の可視化 | 領域 | ケース 19（Phase 2 Block B subnode 展開を正準として検証） |
| §2.6 R2 基本フロー/事後条件記述整合 | 領域 | ケース 12, 19 |
| §2.6 R3 決定論保証の具体化 | 領域 | ケース 23 |
| §2.6 R4 MCP-INV-1 確認可能性 | 領域 | TS-LGX-009 へ委譲（ケース 1 でオプション引数性は含意） |
| §2.6 R5 sections×document 無視 | 領域 | ケース 17 |

> 継承 TP 観点（TP-003 全 51 + TP-014 全 22 = 73 観点）はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（漏れ 0、人間ゲート判断対象）。基盤型・上流解決セマンティクス・atomicity は TS-LGX-002（DD-LGX-002 凍結基盤）へ、意味層スコアは TS-LGX-007 へ、性能・並行は NFR-LGX-001 へ、MCP 境界・引数構文は SPEC-LGX-009 / TS-LGX-009 / CLI 層 TS へ責務分離し、本 TS は粒度制御固有（Granularity / build_outline / depth / sections / SectionFormatter 整列・サイズ上限 / 3a fallback / REQ.19-20）に集中する。

## 4. テスト技法選択

- **同値分割**: `granularity ∈ {Document, Subnode}`、`sections ∈ {None, 存在のみ, 不在混在, 全不在, 空/重複}`、`depth_limit ∈ {None, 0, 1, 2}`、見出し ∈ {h1-h3, h4+, スペース無し, 空タイトル, 皆無}。
- **境界値分析**: サイズ上限（499,999 / 500,000 / 500,001、ケース 7）、カウント単位（コードポイント vs バイト/grapheme、ケース 8）、depth（0 / 1 / 2 / None、ケース 9-11）、見出しレベル（h3 上限 / h4 上限+1、ケース 5）。
- **Property-based**: 同一入力 → 同一バイト列（CACHE-INV-1 / CTX-INV-1、ケース 23）、read-only 不変（ケース 24）。
- **状態遷移 / フラグ組合せ**: REQ.18 組合せマトリクス（outline×document / sections×document / outline×sections-subnode / depth 直交、ケース 17, 18）、subnode 有/無混在の 3a fallback（ケース 19）。
- **Contract**: ResultTooLarge メッセージ固定（ケース 20）、exit コード規約（ContextError → exit 1、audit 失敗 → exit 0）。

## 5. テスト基盤

- 言語: Rust（CLI 本体、crate `legixy-ctx`）
- フレームワーク: cargo test（Unit / Integration）
- Property-based: proptest（ケース 23, 24。`granularity` / `sections` / `depth_limit` / `target_files` の生成器）
- モック: graph.toml フィクスチャ（subnode 有/無混在）、一時 SQLite engine.db（audit 失敗注入は権限/ロックで再現）。`build_outline` / `enforce_size_limit` / `upstream_sort_rule` は純関数のため依存なし

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1, 2 | TC-LGX-004（Granularity / CompileInput 既定） | legixy-ctx/src/compiler.rs（unit） |
| ケース 3 | TC-LGX-004（upstream_sort_rule） | legixy-ctx/src/section_formatter.rs（unit） |
| ケース 4, 5, 6 | TC-LGX-004（build_outline） | legixy-ctx/src/compiler.rs（unit） |
| ケース 7, 8, 20 | TC-LGX-004（enforce_size_limit / ResultTooLarge） | legixy-ctx/src/section_formatter.rs, error.rs（unit） |
| ケース 9, 10, 11 | TC-LGX-004（walk_chain_parent_only_with_depth） | legixy-ctx/tests/depth.rs（integration） |
| ケース 12, 13, 14, 15, 16, 17, 18, 19 | TC-LGX-004（compile+render 粒度制御） | legixy-ctx/tests/granularity.rs（integration） |
| ケース 21, 22 | TC-LGX-004（audit best-effort / REQ.20） | legixy-ctx/tests/resilience.rs（integration） |
| ケース 23, 24 | TC-LGX-004（決定論 / read-only） | legixy-ctx/tests/determinism.rs（property） |
