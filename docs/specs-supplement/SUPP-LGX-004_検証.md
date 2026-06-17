Document ID: SUPP-LGX-004

# SUPP-LGX-004: SPEC-LGX-004（検証）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-004 |
| 対象 SPEC | SPEC-LGX-004「検証」 Version 0.8.0（Approved） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
>
> 凡例: **[補完]** = 旧文書（`legixy.old.p1/docs/`、以下 `old.p1`）または旧実装（`traceability-engine.v3.chg_to_lexigy/`、以下 `v3copy`。クレートは te-* → lx-* に改名済）から根拠付きで補完できた事項。**[要決定]** = 見つからない／人間の判断が必要な事項。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリ `legixy/docs/` には `specs/` の SPEC 10 ファイルしか存在しない。SPEC-LGX-004 が参照する以下の文書はすべて新リポジトリに未収録である。

### 1.1 所在を確認できたもの（コピー/再収録候補）

| # | 参照 ID | SPEC 内参照箇所 | 必要な理由 | 確認した所在 |
|---|---------|----------------|-----------|--------------|
| 1 | LGX-EXT-001（サブノード拡張仕様） | ヘッダ「親文書」、§2、REQ.01/07/13 根拠 | SUBNODE-INV-1〜6 の正準定義（§7.2）、Phase 2 Block F の定義 | `old.p1/docs/legixy_subnode_spec_v0.2.1.md`（INV 定義は 603〜625 行） |
| 2 | LEGIXY-SPEC-001（基盤仕様） | §2、REQ.10 根拠 | CTX-INV-1〜4 の正準定義（§10、225〜228 行） | `old.p1/docs/legixy_foundational_spec.md`。**注意: CTX-INV-5 は本文書に存在しない**（§1.2 #12 参照） |
| 3 | NFR-LGX-001（非機能要件） | ヘッダ「対応 NFR」、REQ.04/05/06/08 根拠 | PERF.02（check --formal < 500ms【暫定】/ノード1,000+エッジ2,000）、OBS.02/03/05/06、REL.02/03 の数値・定義 | `old.p1/docs/nfr/NFR-LGX-001_非機能要件.md`（80, 141〜145, 154〜155 行） |
| 4 | UC-LGX-001（グラフ読み込みと検証） | ヘッダ「対応 UC」 | check の利用シナリオ・受け入れ基準 | `old.p1/docs/usecases/UC-LGX-001_グラフ読み込みと検証.md` |
| 5 | LGX-COMPAT-001（CLI/MCP 互換リファレンス） | REQ.04/15 根拠 | 終了コードのグローバル規約（v1.0.1 追記、43 行）、check の引数契約（§4 #3: `check [--formal]` のみ）、設定ファイル探索規約 | `old.p1/docs/legixy_cli_compat_reference.md`（**現行 v1.1.0**。SPEC が言う v1.0.1 の内容を包含） |
| 6 | GAP-LGX-061 / 064 / 065 / 072（旧 073/074 は 064 に吸収） | REQ.01/03/06/15 | 各裁定の経緯・根拠（すべて closed、内容は SPEC v0.7.0/0.8.0 に反映済） | `old.p1/docs/gap-analysis/GAP-LGX-{061,064,065,072}_*.md` |
| 7 | QSET-LGX-004（Q1）/ QSET-LGX-002（Q2） | REQ.04/14 根拠 | exit 2 裁定（Q1 選択肢 A 採択）、SubnodeIdCollision Warning 裁定（Q2「C の Warning 変形」） | `old.p1/docs/frontend-pass/questionnaires/QSET-LGX-004_検証.md`、`QSET-LGX-002_グラフ基盤.md` |
| 8 | SPP-LGX-004（spec patch） | REQ.04「本 SPP 付随変更」、変更履歴 0.6.0 | v0.5.0→0.6.0 差分の正準記録 | `old.p1/docs/spec-patches/SPP-LGX-004_検証.md`（承認済 2026-06-07） |
| 9 | ADR-LGX-003（embedding 決定論モデル） | REQ.06 | スコア値ビット再現を放棄し順序決定論のみ保証する三層モデルの正準根拠 | `old.p1/docs/adr/ADR-LGX-003_embedding-determinism-model.md` |
| 10 | ADR-LGX-008（GraphDag カテゴリ分離） | REQ.01/15 の GraphDag 新設（§5 変更履歴 0.7.0 が参照する人間裁定の ADR 記録） | GraphDag/SubnodeDag 分離の選択肢比較と採択理由 | `old.p1/docs/adr/ADR-LGX-008_graphdag-category-separation.md` |
| 11 | TP-LGX-004（テスト観点） | GAP 各票の「親 TP」 | GAP の観点出典（B1/S1/S7/D2） | `old.p1/docs/test-perspectives/TP-LGX-004_検証.md` |
| 12 | ISSUE-001 / ISSUE-005 | REQ.11/12/13 根拠 | 3 層防御（機能 A/B/C）の動機・vnstudio 実測値（31 件ドリフト、テンプレノイズのベースライン） | `v3copy/issues/ISSUE-001_semantic-id-redefinition-detection.md`、`ISSUE-005_template-similarity-noise-floor.md`（§2.3 に vnstudio 実測表） |
| 13 | workflow_2026-04-20_semantic-check-and-reporting.md | REQ.02 根拠 | SemanticChecker（SEM Block）の実装経緯・bulk API 構成 | `v3copy/claudedocs/workflow_2026-04-20_semantic-check-and-reporting.md`（§2） |
| 14 | TE-NEXT-EXT-001（v3 サブノード仕様） | REQ.14【v3 差分】注記（line 725 引用） | 「ハッシュ衝突は check --formal で検出される」という v3 仕様の約束（725 行付近）の典拠 | `v3copy/docs/traceability_engine_subnode_spec_v0.2.1.md`（721 行「ハッシュ衝突の理論上の可能性は残る(16 文字で実用上ゼロ、`check --formal` で検出される)」） |
| 15 | CLAUDE.md「検証結果への対応方針」 | REQ.03 根拠 | severity 4 段階の運用方針の出典 | `v3copy/CLAUDE.md` 127〜134 行 |
| 16 | 前世代 `old.source/` | §2「慣例仕様として参照」 | v0.1.0 検証機能の慣例仕様 | `v3copy/old.source/`（ディレクトリ実在） |

### 1.2 所在を確認できなかったもの（系列内に文書が存在しない）

| # | 参照 ID | SPEC 内参照箇所 | 状態 | 対処案 |
|---|---------|----------------|------|--------|
| 17 | **TS-LGX-001 / TS-LGX-003（テスト仕様）** | REQ.01/03/07/11/12/13 の「検証方法」欄（T-VL-001〜007、§12 T-SEM-001〜006、§13 T-ISM-001〜005、§15 T-ISD-001〜005、T-IC-001〜006） | `old.p1/docs/test-specs/` は**空**。legixy 系列の TS は未作成 | v3 の `v3copy/docs/test-specs/TS-LX-001_グラフ基盤.md`（T-VL-001 系を含む）、`TS-LX-003_検証.md`（T-SEM/T-IC/T-ISM/T-ISD を §12 以降に含む）を底本に再作成する。【要決定 D-08】 |
| 18 | **VAL-LGX-001（検証レポート）Finding E-01** | REQ.10 根拠 | `old.p1/docs/validation/` は**空**（DevProc テンプレート `VAL-template.md` のみ存在）。文書本体は系列内に見つからない | CTX-INV-5 の正準定義は現状 `old.p1/docs/spec-patches/SPP-LGX-001_全体境界整合.md` 304 行（「CTX-INV-5（未解決エッジの許容性、プロジェクト独自。VAL-LGX-001 Finding E-01）」）が唯一の一次記録。【要決定 D-09】 |
| 19 | **DD-LGX-001（詳細設計）** | SPEC 本文からは v0.1.1 で参照削除済みだが、NFR-LGX-001.OBS.06 と GAP-LGX-065 が finding シリアライズ形状の責務先として参照。REQ.02/06 の「DD で凍結」の受け皿 | `old.p1/docs/detailed-design/` は**空**。legixy 系列の DD は未作成 | v3 の DD（コード内コメントの DD-LX-001/003/007）相当を新規起草する必要がある。本 SUPP §2 の [補完] 項目が起草材料となる |

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 [補完] 旧文書・旧実装から確定できた事項

**H-01. CheckResult / CheckReport のデータ構造**（REQ.03/08 が前提とするが SPEC に定義なし）
根拠: `v3copy/crates/lx-core/src/types.rs` 48〜62 行。
```rust
pub struct CheckResult { severity: Severity, category: CheckCategory, message: String, related_ids: Vec<String> }
pub struct CheckReport { results: Vec<CheckResult>, error_count: usize, warning_count: usize, info_count: usize, ok_count: usize }
```
Severity は `Ok/Info/Warning/Error` の unit enum（serde 既定のため JSON では `"Ok"` 等の PascalCase 文字列、types.rs 5〜11 行）。`CheckReport::new` が severity rank（Error=3, Warning=2, Info=1, Ok=0）の**降順 stable sort** と件数集計を行う（types.rs 64〜67 行）。REQ.03 の「Ok は予約 severity」（ok_count 集計のみ・finding 非発行）はこの構造で実現される。

**H-02. JSON Lines 出力スキーマ**（REQ.08「JSON Lines 出力対応」の具体形）
根拠: `v3copy/crates/lx-check/src/reporter.rs`。
- 1 行 1 CheckResult: `{"severity":"Warning","category":"Freshness","message":"...","related_ids":["..."]}`
- 末尾サマリ行: `{"summary":{"error_count":N,"warning_count":N,"info_count":N,"ok_count":N}}`
- text 出力: `[ERROR]   Category: message [related_ids: ...]` 形式 + `Summary: N OK, N INFO, N WARNING, N ERROR` 行。Ok finding は text でも skip（reporter.rs の `if r.severity == Severity::Ok { continue; }`、SPEC が引用する v3 reporter.rs:62 に対応）。

**H-03. 閾値・設定の既定値一覧**（REQ.02/09/11/12/13 が名前のみ参照する設定キーの既定値）
根拠: `v3copy/crates/lx-core/src/config/loader.rs` 233〜264 行、`config/model.rs` 167〜285 行。
| 設定 | キー | 既定値 |
|---|---|---|
| `[semantic]` | enabled / model / similarity_threshold / drift_threshold / link_candidate_threshold / include_subnodes | false / `all-MiniLM-L6-v2` / **0.4** / **0.3** / **0.7** / **true** |
| `[freshness]` | enabled / method | false（設定不在時。init テンプレートは true）/ `"mtime"` |
| `[id_changelog]` | enabled / source / citation_pattern / max_citations_per_id | false / `spec_header` / `\|\s*{ID}\s*\|` / **50** |
| `[id_semantic_mismatch]` | enabled / severity / unit_normalization / keywords | false / `info` / true / `[]`（空） |
| `[id_semantic_drift]` | enabled / similarity_threshold / citation_pattern / max_pairs_per_id | false / **0.75** / `\|\s*{ID}\s*\|` / **50** |

REQ.13 の「similarity_threshold 既定 0.75」は `[id_semantic_drift]` 専用値であり、`[semantic].similarity_threshold`（既定 0.4、REQ.02 の SemanticSimilarity 用）とは**別キー**である点に注意。

**H-04. 設定ファイル名の解決**（REQ.09「`.legixy.toml` で設定」の互換規約）
根拠: `old.p1/docs/legixy_cli_compat_reference.md` 163 行（チェックリスト）「設定ファイル探索: `.legixy.toml` 既定 + `.trace-engine.toml` 旧名フォールバック（SPEC-LGX-008.REQ.13）」。v3 実装は `.trace-engine.toml` 固定（`v3copy/crates/lx-cli/src/commands/check.rs:13`）。legixy では SPEC-LGX-008.REQ.13（新リポジトリに存在）に従い両対応とする。

**H-05. DocumentId 検査のアルゴリズム詳細**（REQ.01 の DocumentId カテゴリ）
根拠: `v3copy/crates/lx-check/src/document_id.rs`、`config_loader.rs`。
- 走査範囲はファイル**先頭 32 行**（`MAX_HEADER_LINES = 32`、document_id.rs:9）。行欠落 Error のメッセージにも「先頭 32 行内」と明示される。
- パターンは `[id.document_id].pattern`（既定 `"Document ID:"`）の **literal prefix 一致**。`{id}` プレースホルダは解釈されず、含まれていた場合は config 由来 Warning（category=DocumentId）が発行される（config_loader.rs §9.1 処理）。
- このほか formal check は config 由来 finding を先頭に出力する: `[id].area == "XX"`（init テンプレ既定値のまま）の場合 Info（formal_checker.rs `config_warnings()`）。**REQ.15 の割当表に現れない config 由来 Warning/Info が v3 に存在する**ため、legixy では DocumentId カテゴリの一部として扱うか要整理（→ D-06）。

**H-06. FileExistence / OrphanFile の走査前提**（REQ.01）
根拠: `v3copy/crates/lx-check/src/config_loader.rs`。`[id.types.<CODE>]` 拡張セクション（`dir` / `ext` / `file_pattern = "prefix"|"contains"`）が OrphanFile の走査対象ディレクトリと ID 抽出方式を定める。`prefix` = ファイル名が `{ID}_<desc>.<ext>` 形式、`contains` = 内容先頭に `Document ID: {ID}`。この拡張スキーマは SPEC-LGX-004 本文に現れないが OrphanFile 実装に必須。

**H-07. Freshness 検出のアルゴリズム**（REQ.09）
根拠: `v3copy/crates/lx-check/src/freshness.rs`。
- 対象は **chain エッジのみ**（Custom/ParentChild は対象外）。
- `from`（上流）の mtime > `to`（下流）の mtime なら Warning 1 件。
- mtime 取得失敗（ファイル不在等）は**無言 skip**（FileExistence 側で検出される前提）。
- `method != "mtime"` の場合は**全体 no-op**（v3 は mtime のみ実装。git は未実装 → D-02）。

**H-08. 検査の実行順序**（REQ.06 の順序決定論の前提となる v3 実測順）
根拠: `v3copy/crates/lx-check/src/formal_checker.rs` 89〜140 行 + `lx-graph/src/validation.rs:8-16`。
formal: (0) config 由来警告 → (1) FileExistence → (2) DocumentId → (3) ChainIntegrity → (4) OrphanFile → (5) Freshness → (6) IdChangelog → (7) IdSemanticDrift（enabled 時）→ (8) IdSemanticMismatch → (9) validate_graph（DAG → UnresolvedEdge → サブノード不変条件）。run_all はこれに SemanticChecker を加える。最終的に `CheckReport::new` が severity 降順 stable sort するため、**同一 severity 内の順序は上記実行順**で決まる。legixy の安定ソートキー（severity → category → related_ids）はこれの拡張（詳細 DD、→ D-05）。

**H-09. SemanticChecker の縮退挙動**（REQ.02「embeddings テーブルが空の場合は Info 1 件」の具体形）
根拠: `v3copy/crates/lx-check/src/semantic_checker.rs` 25〜63 行。
- engine.db 不在/開けない → Info 1 件、**category = SemanticSimilarity**、message「engine.db が開けません。`legixy init` + `embed --all` を実行してください」（v3 文言は `lexigy`）。
- embeddings 空 → Info 1 件、同カテゴリ、「embeddings が未生成です。`legixy embed --all` を実行してください」。
- 誘導 Info の category が専用カテゴリでなく SemanticSimilarity を流用する点は SPEC に未記載の実装詳細。

**H-10. citation_pattern の `{ID}` 置換セマンティクス**（REQ.11/13）
根拠: `v3copy/crates/lx-check/src/id_changelog.rs` 262〜264 行。ID を `regex::escape` した上で pattern 文字列中の `{ID}` を文字列置換し、全体を正規表現としてコンパイルする。ID 中の `.` 等は自動エスケープされる。

**H-11. IdRedefined の引用走査打切り**（REQ.11 に記載漏れのパラメータ）
根拠: `config/model.rs` IdChangelogConfig。`max_citations_per_id`（既定 50）で 1 ID あたりの引用列挙を打ち切る（TS-LX-003 T-IC-006 が 60 行 fixture でこの上限を検証）。REQ.13 の `max_pairs_per_id` と対になる REQ.11 側の打切りパラメータだが SPEC 本文に現れない。

**H-12. IdSemanticDrift の「定義側/引用側」判定**（REQ.13）
根拠: `v3copy/crates/lx-check/src/id_semantic_drift.rs` 54〜131 行。
- 対象 embedding は `is_subnode = 1` の行のみ。
- ID 集合は **type_code == "SPEC"** のノード本体から抽出（ID Changelog 表 + `|` 始まりの定義表行）。
- 定義側/引用側の判定はサブノードの `parent_id` が指す親ノードの **type_code が "SPEC" か否か**で行う。
- Warning メッセージには ID・類似度・閾値・定義側/引用側それぞれの path/subnode/anchor を含める。

**H-13. 終了コードの実装位置と graph.toml 物理不在時の v3 挙動**（REQ.01 空グラフ規定の対側）
根拠: `v3copy/crates/lx-check/src/reporter.rs` exit_code()（0/1）、`formal_checker.rs` load_graph()（graph.toml 不在 → `CheckCliError::GraphNotFound` で Err 返却）、`lx-cli/src/commands/check.rs`（Err は実行エラー、v3 では exit 1 系）。設定ファイル不在も `ConfigNotFound` で同様。legixy の「init 誘導の別経路」はこの Err 経路に文言を付与する形が自然（exit code は D-04）。

**H-14. G1 ゲートの定義**（REQ.10〜15 が繰り返し参照）
根拠: `old.p1/docs/legixy_cli_compat_reference.md` §4 #3「`check` 終了コード: Error 件数>0 で 1、それ以外 0（**G1 ゲート**）」、`old.p1/docs/DevProc_V4/08-gates.md`（フェーズ進行ゲートの 3 層構造）、`06-trace-engine.md` §5（SPEC レベル TDD ゲートと統合スクリプト）。実装観点では **G1 = 「check が Error 0 件で exit 0 を返すこと」をフェーズ進行の機械検証条件とするゲート**。Warning/Info は G1 を阻害しない。

### 2.2 [要決定] 人間の判断が必要な事項

**D-01. `--log-format=json` と互換契約の矛盾（最重要）**
REQ.08 と NFR-LGX-001.OBS.03 は `--log-format=json` を規定するが、LGX-COMPAT-001 v1.1.0 のグローバルオプションは `--json`（flag）のみで `--log-format` は存在せず、v3 実装も `--json` で JSON Lines を選択する（`lx-cli/src/commands/check.rs:19-23`）。互換チェックリストの「加算的拡張の規律」（引数追加は SPEC 改訂 + 人間承認 + ADR 記録が必須）に照らし、選択肢は: (a) `--log-format` を加算的拡張として正式追加（ADR 起票）、(b) REQ.08 を「`--json` で JSON Lines」と読み替える SPEC 訂正、(c) 両者を別名として併存。**実装前に裁定必須**（CLI 互換制約に直結）。

**D-02. freshness method "git" の扱い**
REQ.09 は「mtime **または git 履歴**に基づき」と規定するが、v3 実装は mtime のみで `method != "mtime"` は無言 no-op（freshness.rs:19-22）。git 方式のアルゴリズム（コミット時刻? 最終変更コミットの比較?）は系列内のどの文書にも存在しない。選択肢: (a) Phase 1 は mtime のみ実装し git は将来拡張と明記、(b) git 方式を DD で設計。なお `method = "git"` 指定時に無言 no-op とするか Warning を出すかも未規定。

**D-03. IdSemanticMismatch の数値抽出仕様の SPEC/v3 乖離**
REQ.12 の正規表現は `\b(\d+(?:\.\d+)?)\s*(ms|秒|分|時間|byte|MB|GB)\b` + 比較演算子（以内/以上/未満）抽出を規定するが、v3 実装は `(\d+(?:\.\d+)?)\s*(ms|秒|分|時間|s|min|hour)`（`lx-check/src/id_semantic_mismatch.rs:32`）で、(a) 単位集合が異なる（SPEC: byte/MB/GB あり・s/min/hour なし、v3: 逆）、(b) `\b` 境界なし、(c) 比較演算子抽出は v3 実装に見当たらない、(d) v3 の正規化は ms 換算のみで byte 系正規化は存在しない。SPEC を正とするなら新規実装が必要であり、byte/MB/GB の正規化規則（1MB = 10^6 か 2^20 か）も未規定。

**D-04. graph.toml 物理不在時（init 誘導経路）の終了コード**
REQ.01 は「物理的に存在しない場合は未初期化（init 誘導）として別経路で扱う」とするのみで exit code 未規定。v3 は GraphNotFound エラーで exit 1。選択肢: (a) exit 1 維持（実行エラー扱い、v3 互換）、(b) exit 0 + 誘導のみ。LGX-COMPAT-001 の終了コード契約（実行時失敗 = exit 1）との整合からは (a) が自然だが明文化が必要。

**D-05. 安定ソートキーの詳細（DD 凍結事項）**
REQ.06 は「severity rank 降順 → category → related_ids。詳細キーは DD で凍結」とする。未確定: category の順序定義（enum 宣言順か名前辞書順か）、related_ids の比較方法（先頭要素か結合文字列か）、同キー時の message の扱い、v3 の「severity のみ sort + 実行順保存」（H-08）との互換。DD 起草時に凍結要。

**D-06. v3 の config 由来 Warning/Info（H-05 後段）の REQ.15 割当表上の位置づけ**
v3 は `[id.document_id].pattern` の `{id}` 誤記 Warning と `[id].area == "XX"` Info を category=DocumentId で発行する。REQ.15 は「本表に現れないカテゴリの追加は本 SPEC の改訂を要する」と割当完全性を宣言しており、DocumentId = Error 固定の表とこれら Warning/Info finding の関係（DocumentId カテゴリの severity が実際には 3 値になる）は整理が必要。選択肢: (a) config 検査を check から分離、(b) 専用カテゴリ新設（SPEC 改訂）、(c) REQ.15 の「固定」は違反 finding の severity を指し config 助言は対象外と注記。

**D-07. SubnodeIdCollision の実装データフロー**
REQ.14 は検出要件のみ規定。v3 parser は衝突情報を保持せず無言縮退する（`lx-graph/src/parser.rs` — SPEC 引用の te-graph/src/parser.rs:126-145 に対応）ため、検出には (a) parser が縮退発生を graph 構造体に記録し validate_graph が finding 化する、(b) check 側で graph.toml + Markdown を再走査する、の 2 案がある。SPEC-LGX-002.REQ.12（縮退挙動の規定、新リポジトリに存在）との実装分担を DD で確定要。

**D-08. TS-LGX-001 / TS-LGX-003 の再作成**
SPEC の全「検証方法」欄が参照するテストケース ID（T-VL/T-SEM/T-IC/T-ISM/T-ISD）の正準文書が legixy 系列に存在しない（§1.2 #17）。v3 の TS-LX-001/TS-LX-003 を底本にできるが、legixy 新設分（GraphDag、空グラフ fixture、SubnodeIdCollision、REQ.15 突合テスト、全層冪等性）はテストケース新規起草が必要。番号体系（TS-LGX-001 が「グラフ基盤」か「検証」か — v3 では TS-LX-001=グラフ基盤に T-VL 系が所属）の確認も含め人間承認対象。

**D-09. VAL-LGX-001 の欠落と CTX-INV-5 の正準定義**
REQ.10 の根拠文書 VAL-LGX-001（Finding E-01）が系列内に存在せず、CTX-INV-5「未解決エッジの許容性」の定義文は SPP-LGX-001:304 と SPEC-LGX-002.REQ.11（新リポジトリに存在）から再構成するしかない。実装は SPEC-LGX-002.REQ.11 + 本 SPEC REQ.10 で可能だが、トレーサビリティ完全性のため VAL-LGX-001 の復元または「LEGIXY-SPEC-001 §10 への CTX-INV-5 追記」の人間判断が望ましい。

**D-10. クレート命名の凍結**
REQ.02 は「crate 名は例示であり DD で凍結（SPEC-LGX-001.REQ.03）」とする。候補が 3 系統ある: SPEC 例示 `legixy-check` / COMPAT §2 の対応表 `te-* → legixy-core/...` / 参照可能な旧実装コピーの実名 `lx-check` 等。DD 起草時に凍結要（CLI バイナリ名・`--version` 出力文字列の決定も同時に必要）。

**D-11. 空グラフ時 stderr Info の文言と JSON モードでの表現**
REQ.01 は「stderr に Info 1 件（graph 未構築・成果物未登録の誘導)」を規定するが、(a) 文言、(b) この Info が CheckReport の findings/counts に**含まれない**こと（含めると「finding 0 件」と矛盾）の明示、(c) `--json` 時に stderr へ出す形式（plain text か JSON Lines か）が未規定。v3 に対応実装はない（v3 は無言）。DD で確定し、(b) は SPEC の意図（finding 0 件・counts 0）から「stderr ログでありレポート外」と解するのが整合的。

---

## §3 用語・前提の補完

| 用語 | 定義・補足 | 根拠 |
|------|-----------|------|
| CTX-INV-1〜4 | 1: 決定論保証（同一入力→同一結果）/ 2: グラフ整合性 / 3: カスタムエッジ独立性 / 4: DAG 制約（サイクル不存在） | `old.p1/docs/legixy_foundational_spec.md` 225〜228 行 |
| CTX-INV-5 | 未解決エッジの許容性（プロジェクト独自追加。基盤仕様 §10 には未収録） | `old.p1/docs/spec-patches/SPP-LGX-001_全体境界整合.md` 304 行、SPEC-LGX-002.REQ.11 |
| SUBNODE-INV-1〜6 | 1: 親子整合性（親ドキュメントノードの実在）/ 2: パス整合性（subnode.path == 親.path）/ 3: ID 一意性 / 4: サブノード含む全グラフの DAG（CTX-INV-4 の拡張）/ 5: 自動生成 ID の決定論性（親 ID + 見出し階層パスのハッシュ。上位見出しリネームによる連鎖変化は正しい挙動）/ 6: 明示 ID（`s:` 接頭辞）と自動生成 ID（16 進ハッシュ）の形式的区別 | `old.p1/docs/legixy_subnode_spec_v0.2.1.md` §7.2（603〜625 行） |
| G1 ゲート | check が Error 0 件（exit 0）であることを条件とするフェーズ進行の機械検証ゲート | H-14 参照 |
| 前段ループ / QSET / SPP | DevProc_V4.1 の Raw SPEC → Accepted SPEC ゲート工程。QSET = AI 起票の質問票、SPP = 回答反映の SPEC 差分案（人間承認で SPEC に反映） | `old.p1/docs/DevProc_V4/08-gates.md` §3、`03a-frontend-pass.md` |
| 3 層防御（機能 A/B/C） | A = IdRedefined（Changelog 宣言ベース、REQ.11）/ B = IdSemanticMismatch（regex 数値照合、REQ.12）/ C = IdSemanticDrift（サブノード embedding 類似度、REQ.13） | `v3copy/issues/ISSUE-001_...md` §2 |
| Phase 2 Block A / F | A = サブノード embedding 登録（SPEC-LGX-006.REQ.09）、F = IdSemanticDrift（本 SPEC REQ.13）。LGX-EXT-001 の段階導入計画のブロック名 | `old.p1/docs/legixy_subnode_spec_v0.2.1.md` §8、`v3copy/claudedocs/workflow_2026-04-28_te-next-ext-001-phase2.md` |
| v0.1.0 検証機能 / 前世代 old.source | リブランド前系譜（traceability-engine v0.1.0 / さらに前世代 lexigy）の check 実装。「慣例仕様」として参照される | `v3copy/old.source/`、`v3copy/docs/manual.md` |
| vnstudio | ISSUE-001/005 の起票元ドッグフーディングプロジェクト（112 ノード規模の実測ベースライン提供元） | `v3copy/issues/ISSUE-005_...md` §2.3 |
| engine.db / embeddings テーブル | SQLite キャッシュ DB。embeddings テーブルは node_id / content_hash / vector / is_subnode / parent_id / anchor 等を保持（Git 管理外・再生成可能） | `v3copy/crates/lx-db/`、SPEC-LGX-006.REQ.12（新リポジトリに存在） |
| content_hash | BOM 除去 → 改行 LF 正規化 → NFC → 末尾正規化後 UTF-8 の SHA-256（環境非依存化） | `old.p1/docs/adr/ADR-LGX-003_...md` §2 選択肢 A-2 |
| heading_path 正規化 | サブノード ID 生成の入力（Markdown 装飾除去・全角空白・NFC 等）。SPEC-LGX-002.REQ.05/06 が正準（新リポジトリに存在） | SPEC-LGX-002 v0.4.x §3 |

---

## §4 旧実装からの参考情報

旧実装リポジトリ: `traceability-engine.v3.chg_to_lexigy/`
**注意:** SPEC が引用する `crates/te-check/...`・`crates/te-graph/...` は v3 オリジナルのクレート名であり、参照可能なコピーでは `lx-*` に改名済（`Document ID` コメントも DD-LX-* 系）。行番号は概ね一致する（例: SPEC 引用 reporter.rs:50-57 = exit_code、reporter.rs:62 = Ok skip、document_id.rs:81 = 行欠落 Error、validation.rs:52-65 = サイクル検出、parser.rs:126-145 = 無言縮退）。

| 対象 REQ | 該当 crate | 関連ファイル（v3copy 配下） | 内容 |
|---|---|---|---|
| REQ.01/05/15 | lx-check | `crates/lx-check/src/formal_checker.rs` | 検査オーケストレーション・実行順（H-08）・graph/config 不在エラー |
| REQ.01 | lx-check | `crates/lx-check/src/{file_existence,document_id,chain_integrity,orphan_file}.rs` | 形式検査 4 カテゴリの実装 |
| REQ.01/07/10 | lx-graph | `crates/lx-graph/src/validation.rs` | DAG（v3 では SubnodeDag 名 — legixy は GraphDag に分離）、UnresolvedEdge、サブノード不変条件検査 |
| REQ.14 | lx-graph | `crates/lx-graph/src/parser.rs`、`src/subnode/` | 同一 heading_path の無言縮退箇所（検出機構は新規実装） |
| REQ.02 | lx-check / lx-embed | `crates/lx-check/src/semantic_checker.rs`、`crates/lx-embed/src/similarity.rs` | SemanticSimilarity/LinkCandidate/Drift + bulk similarity API（compute_edge_scores / compute_link_candidates / detect_drift） |
| REQ.03/04/08 | lx-core / lx-check | `crates/lx-core/src/types.rs`、`crates/lx-check/src/reporter.rs` | CheckResult/CheckReport/Severity/CheckCategory（17 カテゴリ enum）、text/JSON Lines レンダラ、exit_code |
| REQ.09 | lx-check | `crates/lx-check/src/freshness.rs` | mtime 比較（chain のみ） |
| REQ.11 | lx-check | `crates/lx-check/src/id_changelog.rs` | Changelog 表/TOML パース、citation 走査、`{ID}` 置換（H-10） |
| REQ.12 | lx-check | `crates/lx-check/src/id_semantic_mismatch.rs` | 数値抽出・ms 正規化（SPEC との乖離は D-03） |
| REQ.13 | lx-check | `crates/lx-check/src/id_semantic_drift.rs` | サブノードペア類似度（定義側判定は H-12） |
| 設定全般 | lx-core / lx-check | `crates/lx-core/src/config/{model,loader}.rs`、`crates/lx-check/src/config_loader.rs` | 全設定スキーマ・既定値（H-03）・`[id.types]` 拡張（H-06） |
| CLI 結線 | lx-cli | `crates/lx-cli/src/commands/check.rs` | `check [--formal]` + グローバル `--json`、exit 処理 |
| テスト底本 | — | `docs/test-specs/TS-LX-001_グラフ基盤.md`、`TS-LX-003_検証.md` | T-VL / T-SEM / T-IC / T-ISM / T-ISD の全ケース定義（D-08 の底本） |
| 運用記録 | — | `CLAUDE.md`（127〜134 行）、`claudedocs/workflow_2026-04-20_semantic-check-and-reporting.md`、`issues/ISSUE-001`・`ISSUE-005`、`docs/traceability_engine_subnode_spec_v0.2.1.md`、`deploy/manual.md`、`old.source/` | severity 運用方針、SEM Block 経緯、3 層防御の動機、TE-NEXT-EXT-001、前世代慣例仕様 |

---

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-06-12 | 初版（AI 生成、人間査読待ち）。SPEC-LGX-004 v0.8.0 を対象に未解決参照 19 件・[補完] 14 件・[要決定] 11 件を整理 |
