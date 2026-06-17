Document ID: SUPP-LGX-002

# SUPP-LGX-002: SPEC-LGX-002（グラフ基盤）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-002 |
| 対象 SPEC | SPEC-LGX-002「グラフ基盤」 Version 0.4.2（2026-06-10 承認） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **位置づけ**: 本文書は SPEC 本文の変更ではなく、実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。本文書と SPEC が矛盾する場合は SPEC が優先する。
>
> **参照リポジトリ**:
> - 旧文書群: `legixy.old.p1/docs/`（以下「old.p1」）
> - 旧実装（traceability-engine.v3、crate 名は lx-* にリブランド済み）: `traceability-engine.v3.chg_to_lexigy/`（以下「v3」）

---

## §1 未解決参照

SPEC-LGX-002 が参照するが、新リポジトリ（legixy）に存在しない文書の一覧。「所在」列のファイルを新リポジトリへ持ち込む（または正準所在を決める）必要がある。

| # | 参照 ID | SPEC 内の参照箇所 | 必要な理由 | 所在（確認済み） |
|---|---------|------------------|-----------|------------------|
| 1 | LGX-EXT-001（サブノード仕様 v0.2.1） | ヘッダ表「親文書」、§2、REQ.01〜.06, .09 の根拠 | ノード/エッジのデータスキーマ正準定義（§4）、サブノード概念（§3）、SUBNODE-INV-1〜6 の原文（§7.2） | old.p1 `docs/legixy_subnode_spec_v0.2.1.md` |
| 2 | LEGIXY-SPEC-001（基盤仕様） | REQ.08, REQ.11 の根拠（§10 CTX-INV-1, CTX-INV-5） | CTX-INV-1〜4・STATE-INV 等 全不変条件の正準定義、graph.toml の Source of Truth 宣言 | old.p1 `docs/legixy_foundational_spec.md`（注: §10 に CTX-INV-5 はまだ未追記。v0.3.0 改訂は SPEC-LGX-001 側にのみ反映） |
| 3 | LGX-COMPAT-001（CLI 互換契約） | REQ.13 の根拠（§4 #9 凍結契約） | refresh-subnodes の引数・既定値・バックアップ命名、グローバルフラグ、終了コード規約の凍結内容 | old.p1 `docs/legixy_cli_compat_reference.md` |
| 4 | NFR-LGX-001（非機能要件） | ヘッダ表「対応 NFR」、REQ.10, REQ.13 の根拠 | PERF.04/05 の数値目標、SEC.03/04/06 の入力検証要件、REL.01/02/04、HARDEN.03（ファズ） | old.p1 `docs/nfr/NFR-LGX-001_非機能要件.md`（PERF.04: パース < 100 ms/1,000 ノード【暫定】、PERF.05: 抽出 < 10 ms/file） |
| 5 | UC-LGX-001 / UC-LGX-003 | ヘッダ表「対応 UC」 | グラフ読み込み・検証、サブノード自動抽出の利用シナリオ | old.p1 `docs/usecases/UC-LGX-001_グラフ読み込みと検証.md`, `UC-LGX-003_サブノード自動抽出.md` |
| 6 | VAL-LGX-001（Finding E-01, E-06, P-02, P-03） | REQ.06, .07, .08, .11 の根拠 | 各 REQ が対処する Finding の原文 | **同名文書はどこにも存在しない**（old.p1 `docs/validation/` は空）。実体は v3 `docs/validation/VAL-LX-001_外部照合記録.md`（E-01: ダングリングエッジ、E-06: Markdown 正規化不足、P-02: DAG 対象エッジ範囲、P-03: TOML パーサ順序依存 — 全 Finding の存在・内容一致を確認済み） |
| 7 | TS-LGX-001（T-GP-001〜005, T-HN-001〜003, T-VL-001/002, T-GT-005） | 各 REQ の検証方法 | テスト仕様への前方参照。legixy 用 TS-LGX-001 は**未作成**（old.p1 `docs/test-specs/` は .gitkeep のみ） | 参考実体は v3 `docs/test-specs/TS-LX-001_グラフ基盤.md`（参照される全テスト ID と、NFR REL.04 が参照する T-IG-001〜005 の存在を確認済み） |
| 8 | QSET-LGX-002（Q1〜Q4 回答）/ QSET-LGX-001 Q1 | REQ.05, .10, .12, .13 の根拠 | 凍結判断（ID 生成式・上限なし・縮退・refresh 責務）の決定経緯と詳細 | old.p1 `docs/frontend-pass/questionnaires/QSET-LGX-002_グラフ基盤.md`, `QSET-LGX-001_全体境界整合.md` |
| 9 | SPP-LGX-002 / SPP-LGX-004 | 変更履歴 0.4.0、REQ.12 | v0.4.0 差分の正文、SPEC-LGX-004.REQ.14（SubnodeIdCollision）新設の経緯 | old.p1 `docs/spec-patches/SPP-LGX-002_グラフ基盤.md`, `SPP-LGX-004_検証.md` |
| 10 | GAP-LGX-023 / GAP-LGX-024 | REQ.13（atomicity / バックアップ命名） | 0.4.1/0.4.2 改訂の背景と検証観点（中断シナリオ等） | old.p1 `docs/gap-analysis/GAP-LGX-023_refresh-subnodes-rewrite-atomicity.md`, `GAP-LGX-024_refresh-backup-naming-retention.md` |
| 11 | TE-NEXT-EXT-001 line 725 | REQ.12 の根拠 | 「check による衝突検出」を v3 仕様が約束していた根拠 | v3 `docs/traceability_engine_subnode_spec_v0.2.1.md`（line 723 付近「ハッシュ衝突の理論上の可能性は残る（16 文字で実用上ゼロ、`check --formal` で検出される）」を確認済み。LGX-EXT-001 §9.3 にも同文あり） |
| 12 | v3 ソースパス `crates/te-graph/...`, `crates/te-cli/...` | REQ.05, .12, .13 の根拠注記 | 凍結対象の実測実装 | **te-\* という crate は存在しない**。リブランドで lx-\* に改名済み。対応: `te-graph/src/subnode/id_gen.rs` → v3 `crates/lx-graph/src/subnode/id_gen.rs`、`te-graph/src/parser.rs:126-145` → v3 `crates/lx-graph/src/parser.rs`（該当ロジックは 126-148 付近）、`te-cli/src/commands/refresh_subnodes.rs` → v3 `crates/lx-cli/src/commands/refresh_subnodes.rs` |

新リポジトリ内で解決する参照（持ち込み不要・存在確認済み）: SPEC-LGX-001 §7.1（変更ポリシー）、SPEC-LGX-003.REQ.13（500,000 文字上限）、SPEC-LGX-004.REQ.14（SubnodeIdCollision Warning）、SPEC-LGX-008.REQ.02a（退避命名規約）。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 サブノード ID 生成・見出し抽出

**S2-01 [補完] ID 生成アルゴリズムの正準実装**
REQ.05 の生成式の実測実装は v3 `crates/lx-graph/src/subnode/id_gen.rs:7-20` `generate_subnode_id()`。ハッシュ入力は `parent_id + "|" + heading_path.join("|")` を UTF-8 バイト列として SHA-256 し、**小文字 hex** の先頭 16 文字を採る。自動生成フラグメントの判定は「ちょうど 16 文字かつ全て `0-9a-f`（小文字のみ）」（同ファイル `is_auto_generated_fragment()`）。互換テストの基準実装としてこのファイルをそのまま参照可能。

**S2-02 [補完] 正規化の適用順序（REQ.06 は項目列挙のみで順序未規定）**
v3 `crates/lx-graph/src/subnode/normalizer.rs` `normalize_heading()` の順序が正準（QSET-LGX-002 Q1 回答にも明記）:
1. Markdown 装飾除去 — **長いマーカー優先**: `**`, `__`, `~~` を先に除去し、その後 `*`, `_`, `` ` `` を除去（`*` を先に除去すると `**` が誤処理されるため順序が ID に影響する）
2. Unicode NFC 正規化
3. U+3000（全角スペース）→ ASCII スペース
4. 前後トリム
5. 連続空白を 1 個の ASCII スペースに統合（タブ等**全ての whitespace 文字**が対象。スペースに限らない）

**S2-03 [補完] 見出し抽出の構文規則（REQ.05 では「h2/h3 見出しから抽出」とのみ規定）**
v3 `crates/lx-graph/src/subnode/extractor.rs` の実測挙動:
- **ATX 見出しのみ**対応（`##`/`###`）。Setext 見出し（下線式）は非対応
- `#` 列の直後は **スペース・タブ・行末のいずれか**でなければ見出しと認めない（`atx_level()`、extractor.rs:117-138。`##見出し` は不検出）
- 行頭の空白は許容（`trim_start` 後に判定）。行末の closing `#`（`## 見出し ##`）は除去
- 正規化後に空になる見出しはスキップ。空の h2 は h2 コンテキストをリセットする
- h1 出現で h2 コンテキストをリセット（REQ.05 記載どおり）。`####` 以深と h1 は無視
- 走査は行単位・1 パス・線形（REQ.10 の「線形走査」の実体）

**S2-04 [補完] content_range の正確な定義（REQ.03 は「本文のバイト範囲」とのみ規定）**
v3 extractor.rs 実測: `(body_start, end_byte)` の**バイトオフセット半開区間**。`body_start` = 見出し行の次の行頭、`end_byte` = 次に現れる**同レベル以浅**の抽出対象見出しの行頭（行頭空白を除いた `#` 開始位置ではなく trim 前のオフセット `line_start = offset + trimmed_start`）、なければ EOF。見出し行自体は含まない。

**S2-05 [補完] anchor フィールドの内容（REQ.03 は「見出しテキスト」とのみ規定）**
自動生成サブノードの `anchor` には**正規化前の生見出しテキスト**（`#` マーカーと closing `#` を除去、前後トリム済み、Markdown 装飾は残る）が入る（extractor.rs `raw_line`）。明示サブノードでは graph.toml の `anchor` フィールド値（LGX-EXT-001 §4.1 の例では `"## 状態遷移"` のように `#` 込み）。両者で形式が揃っていない点に注意。

**S2-06 [補完] 自動生成サブノードの type と格納順**
- type は `{親type}-section`（v3 parser.rs:133、LGX-EXT-001 §4.1 の明示サブノードと同形式）
- 格納順: graph.toml 記載の明示ノードを記載順に格納した**後**、ドキュメントノードの記載順 × 各ドキュメント内の見出し出現順で自動生成ノードを追記する（v3 parser.rs:104-146。REQ.08 の「IndexMap 挿入順」の具体化）
- 親ドキュメントのファイル不存在・読込失敗時は**そのドキュメントの抽出をスキップして継続**（部分失敗トレランス、NFR REL.02。parser.rs:106-112）

**S2-07 [補完] 明示 ID の文字制約の正準定義**
REQ.05 の `#s:{任意名}` の「任意名」の制約は LGX-EXT-001 §4.5.2: 英数字・ハイフン・アンダースコアのみ、英数字で始まり英数字で終わる、1〜63 文字。検証実装は v3 id_gen.rs `validate_explicit_name()`。

**S2-08 [要決定] graph.toml の `heading_levels` フィールドの採否**
SPEC-LGX-002 REQ.05 は「h1 と h4 以降は抽出対象外」（h2/h3 固定）とし、LGX-EXT-001 §3.6 もレベル指定フィールドを「Phase 1 では未使用の予約」とする。しかし **v3 実装はノード別 `heading_levels` 指定を既に実装済み**（Phase 2 Block D、parser.rs:121-125 で `unwrap_or_else(|| vec![2,3])`、refresh-subnodes も同フィールドを尊重）。
- 案 A: v3 同様に実装する（実行時互換重視。`heading_levels` 入りの既存 graph.toml が v3 と同じ ID 集合を生む）
- 案 B: SPEC 文言どおり h2/h3 固定とし、フィールドは無視する（SPEC 忠実。ただし v3 で当該フィールドを使っていたプロジェクトと ID 集合が変わる互換破壊リスク）
- 論点: 「実行時引数互換」（CLI 引数）と「データ互換」（graph.toml の解釈）のどちらまで凍結対象とするか。SPEC 改訂（人間承認）が必要になる可能性が高い。

**S2-09 [要決定] コードフェンス内の `#` 行の扱い**
v3 extractor はコードフェンス（``` 〜 ```）を認識せず、フェンス内の `## foo` 行も見出しとして抽出する。SPEC は無言。
- 案 A: v3 挙動を維持（既存 graph.toml に永続化された ID との一致が REQ.05 凍結の目的であり、フェンス処理の追加は抽出集合＝ID 集合を変えるため互換破壊になりうる）
- 案 B: CommonMark 準拠でフェンス内を除外（直感的だが互換テスト「v3 生成 ID と一致」に反する入力が存在する）
- REQ.05 の凍結趣旨からは案 A が整合的だが、SPEC 上の明文がないため人間確認を推奨。

### 2.2 エッジ・DAG・決定論

**S2-10 [補完] エッジ方向のセマンティクス**
SPEC はエッジ 3 種を定義するが from/to の向きの意味は未規定。実測・原文:
- `chain`: **下流成果物 → 上流成果物**（LGX-EXT-001 §4.2 の例: `from = "SRC-SP-003"`, `to = "DD-SP-003#..."`、§3.4「このコードは DD の特定セクションに対応する」）
- `parent_child`: **親ドキュメント → サブノード**（REQ.04 記載どおり。v3 parser.rs:149-158 で `from: 親, to: 子` を全サブノードに生成）
- `custom` の方向セマンティクスは old.p1 `docs/gap-analysis/GAP-LGX-081_custom-エッジの方向セマンティクス.md` が論点化している（SPEC-LGX-005 側の管轄）。グラフ基盤としては「from/to を区別して保持し、向きを反転しない」ことだけ守ればよい。

**S2-11 [補完] Kahn's algorithm の決定論と検出ロジック（REQ.07）**
v3 `crates/lx-graph/src/validation.rs:19-49` `check_dag()`: 全エッジ種別で in-degree を計算し、**IndexMap 挿入順で in-degree 0 のノードをシード**（決定論、CTX-INV-1 対応のコメントあり）、BFS で処理。処理済みノード数 < 全ノード数ならサイクルあり。サイクル検出時の報告（残ノードの列挙等）の出力形式は SPEC-LGX-004（check）の管轄。

**S2-12 [補完] unresolved_edges のデータ構造（REQ.11）**
v3 `crates/lx-graph/src/model.rs:54-59`: `UnresolvedEdge { from: NodeId, to: NodeId, kind: EdgeKind, missing: MissingEndpoint }`。どちらの端点が欠けたか（from / to / 両方）を記録する。Warning 化（報告形式・件数表示）は SPEC-LGX-004 の管轄。

**S2-13 [補完] 未知の edge kind はパースエラー（クラッシュではない）**
REQ.04 の 3 種以外の `kind` 文字列を持つエッジは、v3 parser.rs:163-169 で `GraphParseError::ValidationError`（「unknown edge kind」）として**エラー返却**される（REQ.11 の「未解決エッジとして許容」の対象は ID 不在のみで、kind 不正は対象外）。REQ.10 の「クラッシュ禁止」とは矛盾しない（通常エラー経路）。

**S2-14 [補完] graph.toml のパスは設定駆動（REQ.01 の精密化）**
既定パスは `docs/traceability/graph.toml` だが、v3 では設定ファイル（`.trace-engine.toml`）の `[graph] file` で上書き可能（v3 `crates/lx-core/src/config/model.rs:22-31`、既定値が `docs/traceability/graph.toml`）。legixy では設定ファイル名のフォールバック探索（`.legixy.toml` / 旧名 `.trace-engine.toml`）が SPEC-LGX-008 の管轄。

**S2-15 [補完] REQ.12「縮退」の正確な意味**
v3 parser.rs では自動生成ノードを `nodes.insert(id, node)` で逐次挿入するため、同一 ID の重複は**後勝ち**（最後に出現した見出しの anchor / content_range が残る）。QSET-LGX-002 Q2 回答も「自動抽出は後勝ち」と明記。実装はこの後勝ち縮退を再現すること（衝突の検出・報告は生成段階では行わず SPEC-LGX-004.REQ.14 が担う）。

**S2-16 [補完] ノード ID 本体（`{type}-{area}-{seq}`）の検証は設定駆動**
REQ.03 の ID 形式の具体構文（area 集合・seq 桁数）は SPEC では未規定。v3 では設定 `[id]`（`pattern`, `area`, `seq_digits`, `areas`, `chains`）から正規表現を構築して検証する（v3 `crates/lx-core/src/config/model.rs:41-79` `IdConfig::build_id_regex()`。例: `UC-SP-\d{3}`）。形式検証の報告は SPEC-LGX-004 の管轄。

### 2.3 refresh-subnodes（REQ.13）

**S2-17 [補完] リネーム検出（新旧 ID 対応付け）のアルゴリズム**
SPEC は「突合し、対応を提示する」とのみ規定。v3 `crates/lx-cli/src/commands/refresh_subnodes.rs:120-240` `detect_changes()` の実測:
1. 親ドキュメント（.md のドキュメントノード）ごとに、graph.toml 由来の自動生成サブノード集合と、現ファイルから再抽出した集合の差分を取る（removed = graph にあるが抽出されない、added = 抽出されるが graph にない）
2. removed × added の全ペアについて **anchor 文字列の Levenshtein 距離**を計算し、最小距離ペアから貪欲にリネームとして対応付ける（**距離の閾値なし**）
3. 対応付け残りは orphan（Removed / Added）として報告のみ。**orphan は --apply でも graph.toml に反映されない**（リネームのみ書き換え対象）
4. 明示 ID（`#s:`）は対象外（AutoGenerated のみ）
5. リネーム 0 件なら --apply でも no-op（バックアップも作らない）

**S2-18 [要決定] Levenshtein 閾値なしによる誤対応**
v3 は閾値を持たないため、無関係な「見出し 1 個削除 + 別の見出し 1 個追加」も最小距離ペアとして rename 扱いされ、--apply でエッジが誤った新 ID に付け替えられうる。dry-run 既定が安全弁。
- 案 A: v3 実測を維持（互換・簡潔。dry-run での人間確認を運用前提とする）
- 案 B: 距離閾値や正規化距離比を導入し、超過分は rename でなく orphan として報告（安全性向上だが v3 と提示結果が変わる）
- 出力（提示結果）は凍結契約の対象か否かが論点。LGX-COMPAT-001 §4 #9 は引数とバックアップのみ規定しており、検出結果の改善は許容と読める余地がある。

**S2-19 [補完] 出力形式・グローバルフラグ・終了コード**
- グローバルフラグ `--project-root <PATH>`（既定 `.`）、`--json`、`--models-dir <PATH>` が全サブコマンド共通で存在する（LGX-COMPAT-001 §3）。refresh-subnodes も `--json` で `RefreshReport` の JSON（`renames[] {old_id, new_id, parent_id, old_anchor, new_anchor, upstream_edge_count, custom_edge_count}`, `orphans[] {id, parent_id, anchor, kind: removed|added}`, `parents_scanned`）を出力する（v3 refresh_subnodes.rs:26-60, 360-368）
- テキスト出力のヘッダは dry-run 時 `=== Subnode ID changes detected (dry-run) ===`、apply 時 `=== Subnode ID changes applied ===`（同 365-372）
- `--dry-run` と `--apply` の同時指定はエラー（同 62-64）。終了コード規約: 引数構文エラー exit 2、実行時失敗 exit 1（LGX-COMPAT-001 §3 グローバル規約）
- graph.toml の書き換えは文字列置換ではなく **TOML 構造（toml::Value）で読込 → `[[nodes]]` の id/parent と `[[edges]]` の from/to を置換 → 再シリアライズ**（v3 同 308-356）。v3 は `toml::to_string_pretty` で再出力するため**コメント・書式は保存されない**。コメント保持が必要なら toml_edit 採用を検討（ノード記載順は保持必須 — REQ.08）

**S2-20 [補完] 連番サフィックスの具体形式（REQ.13 1a「SPEC-LGX-008.REQ.02a と同一規約」）**
SPEC-LGX-008.REQ.02a（新リポジトリに存在）: 退避名は `<元ファイル名>.bak.{unix epoch 秒}`、同一秒の衝突時は連番サフィックスを付与し既存を上書きしない、累積保持・機械削除なし。refresh-subnodes では `graph.toml.refresh-bak.{epoch}` ベースに同じ衝突回避を適用する。連番の正確な書式（例: `.1` 後置か `-1` か）は REQ.02a でも未規定のため、DD で一意に定めること（**[要決定]（軽微）**: 推奨は `graph.toml.refresh-bak.{epoch}.{n}`、n=1 から）。なお v3 実装にはこの衝突回避自体が存在しない（【v3 差分】、GAP-LGX-024）。

**S2-21 [補完] atomicity 実装の v3 差分（REQ.13 2〜4）**
v3 は `std::fs::write` による直接上書き（refresh_subnodes.rs:356 付近）であり、SPEC が要求する「一時ファイル + fsync + アトミック rename」は**新規実装**となる（GAP-LGX-023 の経緯どおり）。バックアップ作成は v3 でも `std::fs::copy` で書き換え前に実施し、失敗時は書き換えに進まない（`?` で早期 return、同 292-303）— この順序不変条件は v3 と同じ。一時ファイル命名は SPEC-LGX-008.REQ.02 が `.tmp.{unix epoch 秒}` 方式を「SPEC-LGX-002.REQ.13 と統一」と宣言しているため、これに合わせるのが整合的。

**S2-22 [要決定] --apply 時の engine.db / 旧 embedding キャッシュの扱い**
v3 のファイルヘッダコメント（refresh_subnodes.rs:10）は「--apply で graph.toml + custom_edges + engine.db を更新」と謳うが、**実装は graph.toml の書き換えのみ**で engine.db には触れない（旧 ID の embeddings/scores 行は孤児として残置され、次回 embed/check 時に再生成される想定）。
- 案 A: v3 実測（graph.toml のみ）を踏襲し、engine.db はキャッシュ（STATE-INV-1）として自然失効に任せる
- 案 B: --apply 時に旧 ID のキャッシュ行を削除/付け替えする（DB 肥大防止だが v3 差分になる）
- SPEC-LGX-006/010（embedding 管轄）との整合確認が必要。

### 2.4 その他

**S2-23 [要決定] matrix.md の生成主体と編集検知（REQ.02）**
REQ.02 は matrix.md を「graph.toml から自動生成される読み取り専用ビュー」とし「編集検知は CI で実施」とするが、**生成コマンドが存在しない**: LGX-COMPAT-001 §4 の凍結 19 サブコマンドに matrix 生成はなく、v3 実装にも graph.toml → matrix.md の生成機能はない（v3 `crates/lx-mig/src/matrix.rs` は逆方向、v0.1.0 matrix.md → graph.toml 変換のみ）。LEGIXY-SPEC-001 にも「マトリクスは派生物として生成される」（line 119）とあるのみ。
- 案 A: Phase 1 では生成機能を実装しない（matrix.md 不存在を許容し、REQ.02 は「存在する場合は手動編集禁止」の規律として扱う）
- 案 B: 既存サブコマンド（check / report 等）のフラグとして生成機能を追加（凍結契約上は「加算的拡張」として人間承認 + ADR が必要 — embed --node の前例あり、LGX-COMPAT-001 §4 #4）
- 案 C: legixy 外の CI スクリプトの責務とする
- いずれにせよ生成フォーマットと「冒頭の生成元情報」の文面、CI 検知方法が未定義。SPEC-LGX-004 / 運用文書との分担を人間が決める必要がある。

**S2-24 [要決定] SEC.06（パストラバーサル防止）の検証位置と挙動**
REQ.10 は「project_root 外への参照パス」で**クラッシュしない**ことのみ要求し、検出時の扱い（Error / Warning / 無視）と検証実施箇所（パース時 or check 時）が本 SPEC では未規定。NFR-LGX-001.SEC.06 は「graph.toml 内のファイルパスは project_root 配下に限定」と要求する。v3 の実装位置・挙動（`../` の正規化、シンボリックリンクの扱い）の実測確認を DD 作成時に行い、検証カテゴリと severity は SPEC-LGX-004 の体系（severity 割当）に従って確定すること。

**S2-25 [補完] PERF 目標の前提（参考）**
NFR-LGX-001.PERF.04: graph.toml パース < 100 ms（ノード 1,000 規模、【暫定】注記つき）、PERF.05: サブノード自動抽出 < 10 ms/file（100 行程度の Markdown）。いずれも単体ベンチマークで検証（old.p1 `docs/nfr/NFR-LGX-001_非機能要件.md` line 82-83）。REQ.08 の IndexMap・線形走査はこの予算内に収まる設計選択である。

---

## §3 用語・前提の補完

| 用語 | 定義・補足 | 出典 |
|------|-----------|------|
| サブノード / ドキュメントノード / 自動生成・明示サブノード | ドキュメント内の一区画を指すノード。必ず 1 つのドキュメントノードに属する | LGX-EXT-001 §3.1, §3.2, 付録 A |
| heading_path / ハッシュ対象文字列 | 上位見出しから対象見出しまでの正規化済みテキスト列。ハッシュ入力は `parent_id + "\|" + join("\|")` | LGX-EXT-001 §4.5.1 + QSET-LGX-002 Q1（parent_id を含む点が精密化） |
| 連鎖変化 | 上位見出しリネームで配下サブノード ID が連鎖的に変化する現象。**仕様として受容される正しい挙動**（SUBNODE-INV-5）。対応ツールが refresh-subnodes | LGX-EXT-001 §4.5.1, §9.3、SPEC-LGX-002.REQ.13 |
| CTX-INV-1〜4, FB-INV, SCORE-INV, MCP-INV-1〜4, STATE-INV-1/2 | 不変条件の正準定義一覧 | LEGIXY-SPEC-001 §10（old.p1 `docs/legixy_foundational_spec.md`） |
| CTX-INV-5（未解決エッジの許容性） | SPEC-LGX-001 v0.3.0 で追加。**LEGIXY-SPEC-001 原文 §10 には未反映**のため、正準は新リポジトリの SPEC-LGX-001 §4 を参照すること | SPEC-LGX-001 変更履歴 0.3.0、VAL-LX-001 Finding E-01 |
| SUBNODE-INV-1〜6 | 親存在 / パス整合 / ID 一意性 / DAG / ID 決定性 / 形式的区別。原文は LGX-EXT-001 §7.2 | LGX-EXT-001 §7.2 |
| Admin Surface / Agent Surface | MCP-INV-1 により MCP（Agent Surface）は compile_context, observe, get_compile_audit の 3 ツールのみ。refresh-subnodes 等それ以外の CLI は Admin Surface（人間運用専用、MCP 非公開） | LEGIXY-SPEC-001 §10.4、LGX-COMPAT-001 §5 |
| 成果物タイプコード（REQ.03 の SPEC, UC, RB, SEQ, …） | 正準集合はプロジェクト設定（`.trace-engine.toml` / `.legixy.toml` の `[id]`）が定める。開発プロセス側は RB/SEQ を RBA/SEQA/RBD/SEQD に二段化しているが、エンジンは設定された任意のタイプコードを扱う（ハードコードしない） | old.p1 CLAUDE.md、DevProc_V4 02-typecodes.md、v3 lx-core IdConfig |
| 前段ループ / QSET / SPP / FCR | DevProc_V4.1 の Raw SPEC → Accepted SPEC 工程。REQ.05 等の「QSET-LGX-002 Q1 回答」はこの工程の決定記録 | old.p1 `docs/DevProc_V4/03a-frontend-pass.md` |
| G1 ゲート | check の Error 件数 > 0 で exit 1 となる CI ゲート。REQ.12 が縮退を Warning 留めにする理由（既存プロジェクトの G1 fail 回避） | LGX-COMPAT-001 §4 #3、QSET-LGX-002 Q2 |
| IndexMap | 挿入順を保持する連想配列（Rust indexmap crate）。TOML パーサは `toml` crate の `preserve_order` 相当の順序保持実装が必須 | REQ.08、VAL-LX-001 Finding P-03 |
| 「前段ループ反復 1 新設」 | SPEC v0.4.0（2026-06-07）で QSET-LGX-002 回答 → SPP-LGX-002 承認により追加された REQ であることを示す注記 | SPEC-LGX-002 変更履歴 0.4.0 |

---

## §4 旧実装からの参考情報

旧実装ルート: `traceability-engine.v3.chg_to_lexigy/`（Rust workspace。SPEC 注記の `te-*` crate は `lx-*` に改名済み）

| 領域 | crate / ファイル | 関連 REQ |
|------|------------------|----------|
| graph.toml パース・自動抽出統合・ParentChild 生成・未解決エッジ分離 | `crates/lx-graph/src/parser.rs`（縮退 = 後勝ち insert は 126-148 付近） | REQ.01, .03, .04, .11, .12 |
| ノード/エッジ/UnresolvedEdge モデル | `crates/lx-graph/src/model.rs` | REQ.03, .04, .11 |
| サブノード ID 生成・明示名検証・自動判定 | `crates/lx-graph/src/subnode/id_gen.rs` | REQ.05 |
| 見出し正規化（5 段階） | `crates/lx-graph/src/subnode/normalizer.rs` | REQ.06 |
| 見出し抽出（ATX、h2/h3、content_range） | `crates/lx-graph/src/subnode/extractor.rs` | REQ.05, .10 |
| DAG 検証（Kahn、IndexMap 順シード） | `crates/lx-graph/src/validation.rs`（`check_dag()` :19-49） | REQ.07, .08 |
| refresh-subnodes（検出・Levenshtein 対応付け・apply・バックアップ・レポート） | `crates/lx-cli/src/commands/refresh_subnodes.rs` | REQ.13 |
| CLI グローバルフラグ・サブコマンド定義 | `crates/lx-cli/src/main.rs` | REQ.13 |
| 設定（[graph] file 既定値、[id] pattern） | `crates/lx-core/src/config/model.rs` | REQ.01, .03 |
| check 統合（第 1 層） | `crates/lx-check/src/` | REQ.07（報告は SPEC-LGX-004 管轄） |
| テスト仕様（T-GP/T-HN/T-VL/T-GT/T-IG 系の原文） | `docs/test-specs/TS-LX-001_グラフ基盤.md` | 全 REQ の検証方法 |
| 外部照合記録（E-01/E-06/P-02/P-03 原文） | `docs/validation/VAL-LX-001_外部照合記録.md` | REQ.06, .07, .08, .11 |
| v3 サブノード仕様（TE-NEXT-EXT-001） | `docs/traceability_engine_subnode_spec_v0.2.1.md` | REQ.12 根拠 |
| 運用マニュアル | `deploy/manual.md` | 運用参考 |
| MCP サーバ（引数転送） | `ts-mcp/` | 本 SPEC 範囲外（SPEC-LGX-007/009） |

**互換テストの作り方（REQ.05 検証方法「v3 生成 ID と一致」）**: v3 の `crates/lx-graph/src/subnode/` 配下のユニットテスト（id_gen / extractor / normalizer の #[cfg(test)]）と TS-LX-001 §T-IG/T-HN/T-SE 系のケースをそのまま fixture 化し、既知の (parent_id, heading_path) → ID の期待値ペアを legixy 側テストに移植するのが最短。

---

（以上。本文書は人間査読を経るまで設計判断の根拠として単独使用しないこと）
