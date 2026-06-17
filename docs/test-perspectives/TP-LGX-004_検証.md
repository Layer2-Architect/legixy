Document ID: TP-LGX-004

# TP-LGX-004: 検証（check / check --formal）観点

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。

**親**: SPEC-LGX-004
**ステータス**: green
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は SPEC-LGX-004「検証」の全要求事項（REQ.01〜REQ.14）に観点をぶつける。

- 対象: SPEC-LGX-004 §3 全 REQ、§4 不変条件マトリクス
- 関連 SPEC §: LGX-COMPAT-001 §3 グローバル終了コード規約 / §4 #3（check 終了コード凍結契約）、NFR-LGX-001（PERF.02, OBS.02/03/05/06, REL.02/03）、LGX-EXT-001 §7.1（SUBNODE-INV-1〜6）、SPEC-LGX-002.REQ.05/11/12（サブノード ID 生成・unresolved_edges・縮退）、SPEC-LGX-006（embedding 生成・bulk similarity API）、SPEC-LGX-010（embedding 運用・監査との責務境界）

検証カテゴリそのもの（FileExistence / DocumentId / ChainIntegrity / OrphanFile / Freshness / Subnode 系 / Semantic 系 / Drift / UnresolvedEdge / IdRedefined / IdSemanticMismatch / IdSemanticDrift / SubnodeIdCollision）と、その severity・終了コード・出力・冪等性・部分失敗継続を観点対象とする。embedding 生成アルゴリズム・閾値の数値妥当性そのものは SPEC-LGX-006 / NFR-LGX-001 に委譲し、本 TP では「SPEC-004 が正しく委譲しているか」のみを問う。

## 2. 観点リスト

### 2.1 境界値

- [ ] 観点 B1: 空グラフ（ノード 0 / エッジ 0）に対する check の挙動と終了コード
- [ ] 観点 B2: 単一ノード・エッジ 0 のグラフ（孤立ノードのみ）の扱い
- [ ] 観点 B3: embeddings テーブルが空（行 0）の場合の意味検証の挙動と終了コード影響
- [ ] 観点 B4: 類似度が `similarity_threshold` ちょうど（=, < ではない）のときの判定境界（SemanticSimilarity / IdSemanticDrift）
- [ ] 観点 B5: 類似度が `link_candidate_threshold` ちょうどのときの LinkCandidate 判定境界
- [ ] 観点 B6: drift（content_hash 比較）で対象ファイルが存在しない場合の severity
- [ ] 観点 B7: IdSemanticDrift の `max_pairs_per_id` 上限（既定 50）到達時・到達+1 件目の打切り挙動
- [ ] 観点 B8: NFR-LGX-001.PERF.02 が定める規模上限（ノード 1,000 + エッジ 2,000）と check の性能予算の所在

### 2.2 エラーハンドリング

- [ ] 観点 E1: 一部ファイル読込失敗時に他チェックを継続するか（部分失敗継続）
- [ ] 観点 E2: 読込失敗ファイルの severity 分類（Error か Warning か）
- [ ] 観点 E3: graph.toml 自体が破損・パース不能なときの挙動（検証の前提が崩れるケース）
- [ ] 観点 E4: ONNX モデル不在時の `check`（全層）の挙動（exit code を阻害するか、形式層だけ実行されるか）
- [ ] 観点 E5: 想定される検出失敗（embedding 不在ペア）が「致命扱いされない」ことの明示
- [ ] 観点 E6: 終了コードと finding severity の対応関係が一意に定まっているか（severity=Error のみが exit 1 を引き起こすか）

### 2.3 状態遷移（severity 分類・finding カテゴリの網羅）

- [ ] 観点 S1: 各検証カテゴリの severity が一意に定義されているか（Error / Warning / Info / Ok の割当完全性）
- [ ] 観点 S2: SUBNODE-INV-1/2/3/4/6 違反が必ず Error になることの明示
- [ ] 観点 S3: UnresolvedEdge が Warning（Error ではない）であることの明示と G1 非阻害
- [ ] 観点 S4: SubnodeIdCollision が Warning（Error ではない）であることと、検出対象が「自動生成サブノード同士の縮退のみ」である境界の明示
- [ ] 観点 S5: IdRedefined / IdSemanticMismatch / IdSemanticDrift の severity と既定 ON/OFF の明示
- [ ] 観点 S6: SUBNODE-INV-5（ID 決定性）が本 SPEC の検証対象「外」であることの明示
- [ ] 観点 S7: `Ok` severity の使用条件（問題なしを finding として表現するか、検証全体の結果としてか）

### 2.4 並行性

- [ ] 観点 C1: check 実行中に対象ファイル / graph.toml / engine.db が外部更新された場合の整合性保証の有無
- [ ] 観点 C2: 複数 check の同時実行（読み取り専用前提の明示があるか）

### 2.5 バージョニング・互換性（FFI/境界 API: 終了コード契約）

- [ ] 観点 V1: check / check --formal の終了コード 0/1/2 が LGX-COMPAT-001 の凍結契約と一致するか
- [ ] 観点 V2: exit 2（使用法誤り）の定義が「引数パーサ層が検出する構文レベルの誤り」に限定され、値の意味的不正は exit 1 に分類される境界の明示
- [ ] 観点 V3: 新規 finding カテゴリ追加（UnresolvedEdge / IdRedefined / IdSemanticMismatch / IdSemanticDrift / SubnodeIdCollision）が既存の終了コード規約・G1 ゲートを破らないことの明示
- [ ] 観点 V4: v3（旧 traceability-engine）からの移行で既存プロジェクトの check が G1 で fail しない後方互換性の担保（SubnodeIdCollision を Error にしない理由）
- [ ] 観点 V5: 新規カテゴリのデフォルト OFF（IdRedefined / IdSemanticMismatch / IdSemanticDrift）による後方互換性の担保
- [ ] 観点 V6: `.legixy.toml` / `.trace-engine.toml` どちらの設定ファイルから閾値・オプトインを読むか（設定ソースの所在）

### 2.6 永続化

- [ ] 観点 P1: drift 検査の比較元（embeddings テーブルの保存済 content_hash）が存在しない / 古い場合の挙動
- [ ] 観点 P2: engine.db 不在時の check（全層）の挙動
- [ ] 観点 P3: embeddings テーブルとファイル実体の不整合（drift）検出が全成果物を網羅するかの明示

### 2.7 入力検証

- [ ] 観点 I1: SubnodeIdFormat（SUBNODE-INV-6）の検証段階（構文）の定義
- [ ] 観点 I2: ID Changelog の入力ソース（spec_header / toml_config / both）と表形式の構文要件
- [ ] 観点 I3: IdSemanticMismatch の数値・単位抽出パターンと単位正規化の定義
- [ ] 観点 I4: citation_pattern の既定値と、ID 引用走査の対象範囲（chain 下流に限定されるか）

### 2.8 ライフサイクル

- [ ] 観点 L1: 初回実行（embed 未実行 = embeddings 空）の誘導挙動
- [ ] 観点 L2: include_subnodes が無効な場合の IdSemanticDrift の挙動（前提条件の明示）

### 2.9 ロギング・観測性

- [ ] 観点 O1: CheckReport の出力先（stdout）とログの出力先（stderr）の分離
- [ ] 観点 O2: `--log-format=json`（JSON Lines）出力対応の明示
- [ ] 観点 O3: finding の message が「何が起きたか・どこか・どの ID か」を特定できる情報量を持つか（related_ids / ファイルパス + 行番号）
- [ ] 観点 O4: 機密情報（content の生テキスト）の出力範囲の明示（check 内 Drift は判定のみで生スコア一覧を出さない責務境界）

### 2.10 冪等性・決定性

- [ ] 観点 D1: 同一入力に対し check --formal が常に同一 CheckReport（結果順序含む）を返すことの明示
- [ ] 観点 D2: 全層 check（意味検証含む）の冪等性が REQ.06 の射程に含まれるか

### 2.11 領域固有観点（トレーサビリティエンジン / 検証）

- [ ] 観点 R1: ChainIntegrity と OrphanFile の検出対象差分の明示（chain エッジ vs graph.toml 未登録ファイル）
- [ ] 観点 R2: DocumentId 不一致（ファイル先頭 `Document ID:` と graph.toml ID）の severity
- [ ] 観点 R3: DAG 制約違反（CTX-INV-4 / SUBNODE-INV-4 サイクル）の検出と severity
- [ ] 観点 R4: SemanticSimilarity が対象とするエッジ種別（chain / カスタム / 親子）の明示
- [ ] 観点 R5: check（判定）と SPEC-LGX-010 report（計測）の出力責務の非重複の明示
- [ ] 観点 R6: check 内 Drift（content_hash 比較 Warning）と standalone drift（embedding 対比定量値）が別機能であることの明示
- [ ] 観点 R7: SubnodeIdUniqueness（SUBNODE-INV-3 / Error）と SubnodeIdCollision（自動生成縮退 / Warning）の役割分担の明示

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC §X.Y / 委譲先で回答 | 関連 GAP |
|---|---|---|---|
| 2.1 B1 空グラフ | GREEN | （exit/finding は REQ.04+REQ.06 から導出可〔空グラフ=0 Error=exit 0〕。残るは graph 未構築誘導 Info の有無のみ）【WEAK: minor, 人間判断で drop 可】【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-061 |
| 2.1 B2 単一ノード/孤立 | GREEN | REQ.01（OrphanFile は graph.toml 未登録ファイル対象。孤立ノードは検証対象外と読める） | — |
| 2.1 B3 embeddings 空 | GREEN | REQ.02（embeddings 空時は Info 1 件のみ・exit code 非影響） | — |
| 2.1 B4 similarity 閾値境界 | GREEN | REQ.02 / REQ.13（`< similarity_threshold` と明示。`<` は厳密不等号） | — |
| 2.1 B5 link_candidate 閾値境界 | GREEN | REQ.02（`≥ link_candidate_threshold` と明示。等号含む） | — |
| 2.1 B6 drift 対象不在 | GREEN | REQ.02（ファイル不在も Warning と明示） | — |
| 2.1 B7 max_pairs_per_id 打切り | GREEN | REQ.13（既定 50 で打切り、Info 通知と明示） | — |
| 2.1 B8 性能予算規模 | GREEN | NFR-LGX-001.PERF.02 へ委譲（ノード1,000+エッジ2,000で<500ms。Date欄に対応 NFR 明記） | — |
| 2.2 E1 部分失敗継続 | GREEN | REQ.05（一部読込失敗でも他チェック継続） | — |
| 2.2 E2 読込失敗 severity | GREEN | REQ.05（Error として CheckReport に記録） | — |
| 2.2 E3 graph.toml 破損 | GREEN | REQ.04（受理済み引数の値の意味的不正・実行時失敗は exit 1。graph.toml パース失敗は構文層〔clap〕誤りではなく実行時失敗 → exit 1）+ LGX-COMPAT-001 §3 グローバル規約。慣例仕様 old.source は `load_config_or_exit` / `load_matrix_or_exit` で `process::exit(1)` を実装。【敵対的精査 2026-06-09: ALREADY_ANSWERED, GAP-LGX-062 削除】 | — |
| 2.2 E4 モデル不在時 check | GREEN | REQ.02（embeddings 空 = 非致命 Info 1 件・exit 非影響）。モデル不在は embed --all 実行不能 → embeddings 空と同一帰結であり REQ.02 の degraded・非致命パターンに収束。【敵対的精査 2026-06-09: ALREADY_ANSWERED, GAP-LGX-063 削除（069 と同主題）】 | — |
| 2.2 E5 検出失敗非致命 | GREEN | REQ.02（embeddings 空=非致命）/ REQ.13（embedding 不在ペアはスキップ、致命扱いしない） | — |
| 2.2 E6 severity↔exit 対応 | GREEN | REQ.04（exit は Error 件数で決定）+ REQ.03（severity 4 段階）。Error のみが exit 1 を引く | — |
| 2.3 S1 severity 割当完全性 | GREEN | （REQ.03 は 4 段階を定義し subnode 系/UnresolvedEdge/Id 系の severity は pin されるが、基幹形式カテゴリ FileExistence / DocumentId / ChainIntegrity / OrphanFile の既定 severity が SPEC 本文に未集約。NFR.OBS.06 も割当を持たない）【GENUINE: 旧 R2/R3 GAP を吸収】【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-064 |
| 2.3 S2 SUBNODE-INV 必ず Error | GREEN | REQ.07（INV-1/2/3/4/6 違反は必ず Error） | — |
| 2.3 S3 UnresolvedEdge Warning | GREEN | REQ.10（Warning、G1 通過と明示） | — |
| 2.3 S4 SubnodeIdCollision 境界 | GREEN | REQ.14（Warning、自動生成縮退のみ対象、明示ノード衝突は対象外と明示） | — |
| 2.3 S5 Id系 severity/既定 | GREEN | REQ.11（Warning, 既定 false）/ REQ.12（既定 Info, 昇格可, 既定 false）/ REQ.13（Warning, 既定 false） | — |
| 2.3 S6 INV-5 対象外 | GREEN | REQ.07（INV-5 は生成側保証、TS でのみ検証）/ §4 末尾 | — |
| 2.3 S7 Ok 使用条件 | GREEN | （REQ.03 で `Ok` を定義済。発行形状〔個別 Ok finding か全体ステータスか〕は DD-LGX-001 §2.4 の finding シリアライズ責務寄り）【WEAK: minor, 人間判断で drop 可】【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-065 |
| 2.4 C1 実行中外部更新 | GREEN | 並行アクセス/ロックは NFR-LGX-001.REL.07（SQLite busy_timeout 上限 5000ms・無限リトライ禁止）の射程。REQ.06 冪等性は同一入力前提で十分。【敵対的精査 2026-06-09: OUT_OF_SCOPE（NFR.REL.07）, GAP-LGX-066 削除】 | — |
| 2.4 C2 同時実行 | GREEN | 同上。複数 check 同時実行時のロック方針は NFR-LGX-001.REL.07 + REL.08（engine.db 配置条件）が規定。【敵対的精査 2026-06-09: OUT_OF_SCOPE（NFR.REL.07）+ C1 と DUPLICATE, GAP-LGX-067 削除】 | — |
| 2.5 V1 終了コード契約一致 | GREEN | REQ.04 + LGX-COMPAT-001 §4 #3 / §3 グローバル規約（0/1/2 凍結） | — |
| 2.5 V2 exit2 境界 | GREEN | REQ.04（構文誤り=exit2、値の意味的不正=exit1 と明示） | — |
| 2.5 V3 新カテゴリ G1 非阻害 | GREEN | REQ.10/11/12/13/14 各「G1 ゲート: 阻害しない」+ §4 末尾 | — |
| 2.5 V4 v3 移行後方互換 | GREEN | REQ.14（v3 で通っていたプロジェクトの G1 fail 回避が Warning 採用理由と明示。【v3 差分】） | — |
| 2.5 V5 デフォルト OFF 互換 | GREEN | REQ.11/12/13（既定 false / オプトイン）+ §4 末尾 | — |
| 2.5 V6 設定ソース所在 | GREEN | 設定ファイル探索順（`.legixy.toml` → `.trace-engine.toml`）・同一スキーマは SPEC-LGX-008.REQ.13 / LGX-COMPAT-001 §6 が規定（GAP 本文も自認）。SPEC-004 が `.legixy.toml` 名で参照するのは正しい委譲。【敵対的精査 2026-06-09: OUT_OF_SCOPE（SPEC-LGX-008.REQ.13）, GAP-LGX-068 削除】 | — |
| 2.6 P1 drift 比較元不在/古い | GREEN | REQ.02（保存済 hash と比較、不一致=Warning、ファイル不在も Warning。比較元なし=embeddings 空は Info 誘導） | — |
| 2.6 P2 engine.db 不在 | GREEN | REQ.02（embeddings 空 = 非致命 Info・形式層継続）。DB ファイル不在は embeddings 空の上位ケースで同一帰結。LEGIXY-SPEC-001 §10.2 FB-INV-4「DB 不在時安全性: DB がなくてもグラフ上流は正常に返される」が基盤保証。【敵対的精査 2026-06-09: ALREADY_ANSWERED（REQ.02 + FB-INV-4）, GAP-LGX-069 削除（063 と同じ degraded パターン）】 | — |
| 2.6 P3 drift 網羅範囲 | GREEN | REQ.02（「全成果物に対し」content_hash 比較と明示） | — |
| 2.7 I1 SubnodeIdFormat 段階 | GREEN | REQ.01/REQ.07（SubnodeIdFormat=SUBNODE-INV-6）。詳細式は SPEC-LGX-002 へ委譲 | — |
| 2.7 I2 Changelog 入力ソース | GREEN | REQ.11（spec_header/toml_config/both、表列 Date\|ID\|Change\|Note を明示） | — |
| 2.7 I3 数値・単位抽出 | GREEN | REQ.12（正規表現・単位正規化 ms↔秒↔分 を明示） | — |
| 2.7 I4 citation_pattern/走査範囲 | GREEN | REQ.11/12/13（既定 `\|\s*{ID}\s*\|`、chain 下流走査を明示） | — |
| 2.8 L1 初回実行誘導 | GREEN | REQ.02（embeddings 空時に「embed --all 未実行」誘導 Info） | — |
| 2.8 L2 include_subnodes 無効時 | GREEN | REQ.13（「embedding 不在ペアはスキップ、致命扱いしない」）。include_subnodes=false → サブノード embedding 不在 → 全ペアがこの skip 規則に該当。挙動は REQ.13 で導出可能。【敵対的精査 2026-06-09: ALREADY_ANSWERED（REQ.13 skip 規則）, GAP-LGX-070 削除】 | — |
| 2.9 O1 出力先分離 | GREEN | REQ.08（CheckReport=stdout、ログ=stderr） | — |
| 2.9 O2 json log 形式 | GREEN | REQ.08（`--log-format=json` JSON Lines） | — |
| 2.9 O3 message 情報量 | GREEN | CheckResult finding のスキーマ（フィールド集合・JSON シリアライズ）は DD-LGX-001 §2.4 の責務（NFR-LGX-001.OBS.06 が明示参照）。SPEC レベルでは REQ.10/11 が message/related_ids/location を categorically 規定済で観点充足。【敵対的精査 2026-06-09: OUT_OF_SCOPE（DD-LGX-001 §2.4）, GAP-LGX-071 削除】 | — |
| 2.9 O4 生スコア非出力 | GREEN | REQ.02（check は判定のみ、生スコア一覧を出力しない。report との責務境界明示） | — |
| 2.10 D1 formal 冪等性 | GREEN | REQ.06（同一入力で同一 CheckReport、結果順序含む） | — |
| 2.10 D2 全層 check 冪等性 | GREEN | （embedding スコア決定性は SPEC-LGX-006 + CTX-INV-1 + NFR.REL.10 が所有。残るは全層 check の finding 出力順安定キーのみで REQ.06 から自然延長）【WEAK: minor, 人間判断で drop 可】【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-072 |
| 2.11 R1 ChainIntegrity/Orphan 差分 | GREEN | REQ.01（ChainIntegrity=chain エッジ整合、OrphanFile=graph.toml 未登録ファイル） | — |
| 2.11 R2 DocumentId 不一致 severity | GREEN | S1（GAP-LGX-064）に統合。DocumentId 不一致=Error / 欠落=Warning は慣例仕様（SPEC §2 参照の old.source formal.rs）で示唆され、severity 集約は keeper GAP-LGX-064 で一括 pin。【敵対的精査 2026-06-09: DUPLICATE→GAP-LGX-064, GAP-LGX-073 削除】 | → GAP-LGX-064 |
| 2.11 R3 DAG 違反 severity | GREEN | S1（GAP-LGX-064）に統合。node-level DAG は CTX-INV-4（LEGIXY-SPEC-001 §10.1 グラフ全体）、subnode 拡張は subnode_spec §7.1「`check --formal` でサブノード含めたサイクル検出」が約束。基幹 severity 集約は keeper GAP-LGX-064。【敵対的精査 2026-06-09: DUPLICATE→GAP-LGX-064, GAP-LGX-074 削除】 | → GAP-LGX-064 |
| 2.11 R4 SemanticSimilarity 対象エッジ | GREEN | REQ.02（chain/カスタム/親子エッジを対象と明示） | — |
| 2.11 R5 check/report 責務非重複 | GREEN | REQ.02（check=判定 / report=計測、SPEC-LGX-010 へ委譲明示） | — |
| 2.11 R6 Drift 同名別機能 | GREEN | REQ.02（check 内 Drift と standalone drift は同名別機能と明示） | — |
| 2.11 R7 SubnodeUniqueness/Collision 分担 | GREEN | REQ.14 + §4（INV-3=Error の一意性 / 自動生成縮退=Warning の collision） | — |

集計: **全 51 観点 / GREEN 51 / RED 0**

## 4. ステータスの決定

RED 観点が 4 件残存するため、本 TP のステータスは `**ステータス**: green`。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。
内訳: GENUINE 1 件（S1 / GAP-LGX-064 基幹形式カテゴリ severity 割当）、WEAK（minor, 人間判断で drop 可）3 件（B1 / GAP-LGX-061, S7 / GAP-LGX-065, D2 / GAP-LGX-072）。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値 / §エラーハンドリング / §状態遷移 / §並行性 / §バージョニング・互換性 / §永続化 / §入力検証 / §ライフサイクル / §ロギング・観測性 / §FFI・境界 API 観点（ABI 互換性 = 終了コード契約に読み替え）
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（finding message の可読性 O3 に適用）
- LGX-COMPAT-001 §3 / §4 #3 / §7（終了コード凍結契約）
- NFR-LGX-001（PERF.02, OBS.02/03/05/06, REL.02/03）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 検証コマンドには本質的に N/A のため、エラー UX（message の可読性）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 49 件（GREEN 35 / RED 14）。GAP-LGX-061〜074 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 10 件 / 維持 4 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
