Document ID: TP-LGX-003

# TP-LGX-003: コンテキスト解決（compile_context）

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。具体的なテストデータ・期待値は TS 層で行う。

**親**: SPEC-LGX-003
**ステータス**: green
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は `compile_context` MCP ツール（Rust CLI `legixy context` の上位層）の動作要求を対象とする。

- 対象: SPEC-LGX-003 §3 全 REQ（REQ.01〜REQ.18）, §4 不変条件対応表
- 関連 SPEC §:
  - LGX-EXT-002 §3（Prompt Caching 整列）, §4.3（サイズ超過）, §5.2（CACHE-INV-1〜3）
  - LEGIXY-SPEC-001 §10（CTX-INV-1〜4, FB-INV-3/4, MCP-INV-4, STATE-INV-1）
  - SPEC-LGX-007 §REQ.06（context_log 記録仕様）, §REQ.07（get_compile_audit）
  - SPEC-LGX-009 §REQ.04/13/15（MCP 境界での引数転送・`_meta` 付与）
  - SPEC-LGX-008（context_log.granularity カラムの migration）
  - NFR-LGX-001 PERF.03/PERF.09, REL.05, SEC.02/SEC.05, REL.03

## 2. 観点リスト

### 2.1 境界値

- [ ] 観点 B-01: 返却本文がちょうど 500,000 文字 / 500,001 文字のとき、それぞれ成功 / エラーになるか（CACHE-INV-3 の境界）
- [ ] 観点 B-02: 「文字」のカウント単位が明確か（Unicode コードポイント vs バイト vs grapheme）。サロゲートペア・結合文字・ZWJ を含む本文でのカウント挙動
- [ ] 観点 B-03: `--depth 0` / `--depth 1` / 無制限（省略）のそれぞれの返却範囲
- [ ] 観点 B-04: `--depth` に負数・小数・非数値を CLI 経由で渡したときの挙動
- [ ] 観点 B-05: `--sections` に 0 件（空文字列）/ 1 件 / 大量の ID を渡したときの挙動
- [ ] 観点 B-06: target_files が 1 件 / 多数のときの上流解決の決定性
- [ ] 観点 B-07: 上流が空（target が連鎖の最上流 = SPEC 自身）のときの返却内容

### 2.2 エラーハンドリング

- [ ] 観点 E-01: 500,000 文字超過時に切り捨て・要約をせず、明示的エラーと粒度切替提案を返すか
- [ ] 観点 E-02: エラーの種別に型がついているか（サイズ超過 / 入力不正 / DB ロック / グラフ不整合の区別）
- [ ] 観点 E-03: context_log への書き込みが失敗（DB ロック / ディスクフル / 権限）したとき、compile_context の本処理（上流返却）は成功するのか失敗するのか。MCP-INV-4（全呼出し記録）と返却成功の両立可否
- [ ] 観点 E-04: サイズ超過エラーそのものも監査ログに記録されるか（LGX-EXT-002 §5.1 MCP-INV-4「サイズ超過時のエラーもログ対象」との整合）
- [ ] 観点 E-05: 部分成功（上流の一部ファイルが読めない / 一部ノードが欠損）の扱い

### 2.3 状態遷移

- [ ] 観点 S-01: engine.db が存在する場合 / しない場合の 2 状態で返却内容が変わるか（FB-INV-4）
- [ ] 観点 S-02: pending Proposal が存在する状態でも返却が不変か（FB-INV-3）
- [ ] 観点 S-03: compile_context 実行が context_log 書き込み以外の永続状態を持たないか（STATE-INV-1, ステートレス性）

### 2.4 並行性

- [ ] 観点 C-01: 複数 compile_context 同時呼出し時に応答が混在せず各々独立に正しい結果を返すか（REQ.09）
- [ ] 観点 C-02: 並行 context_log 書き込みの排他制御（WAL + busy_timeout）で DB が破損しないか
- [ ] 観点 C-03: busy_timeout を超過してロック取得に失敗したときの挙動（リトライ / エラー / ログ欠落）

### 2.5 バージョニング・互換性

- [ ] 観点 V-01: granularity 既定値が document（v0.1.0 互換）であること、auto モードが排除されていること
- [ ] 観点 V-02: Block B 引数（outline_only / sections / depth）省略時に v0.3.0 以前と同一 argv / 同一返却になる後方互換性
- [ ] 観点 V-03: context_log への granularity カラム追加に伴う旧 DB の migration path
- [ ] 観点 V-04: 同一入力でも実行 OS（Windows / Linux）が異なるとバイト列が変わりうる要素（改行コード CRLF/LF、パス区切り）の正規化。REQ.14 のバイト決定論が OS 横断で成立するか

### 2.6 永続化

- [ ] 観点 P-01: engine.db 不在時に graph.toml のみで上流を返せるか（FB-INV-4）
- [ ] 観点 P-02: context_log の書き込みトランザクション境界（本処理成功と記録の atomicity）
- [ ] 観点 P-03: ディスクフル / 書込権限欠如時の context_log 挙動（→ 2.2 E-03 と連動）

### 2.7 入力検証

- [ ] 観点 I-01: target_files に graph.toml 上にノードが存在しないパスを渡したときの挙動（エラー / 空返却 / 黙殺）
- [ ] 観点 I-02: target_files にサブノード ID（`#` 付き）を渡したときの親ドキュメント上流解決（REQ.08）
- [ ] 観点 I-03: `--sections` に親ドキュメント ID（`#` なし）を渡したときの除外と Info 診断（REQ.16）
- [ ] 観点 I-04: `--sections` の不正形式（空トークン `a,,b`、空白のみトークン、重複 ID）の扱い
- [ ] 観点 I-05: `--granularity` に document / subnode 以外の値を渡したときの拒否
- [ ] 観点 I-06: `--sections` × `--granularity document` の組合せで sections が無視されること（REQ.16, REQ.18）

### 2.8 ライフサイクル

- [ ] 観点 L-01: 空状態（target_files が最上流ノードで上流ゼロ）の返却内容
- [ ] 観点 L-02: `--outline-only` で対象 artifact に h1〜h3 見出しが 1 つも無い場合の出力（空アウトライン）
- [ ] 観点 L-03: `--sections` フィルタ通過結果が空のとき、当該親ドキュメントが upstream に登場しないこと（REQ.16）
- [ ] 観点 L-04: `--depth 0` / MCP 経由 depth<1 reject という CLI/MCP の受理範囲差の正準化（REQ.17）

### 2.9 ロギング・観測性

- [ ] 観点 LOG-01: compile_context の全呼出しが context_log に記録されるか（MCP-INV-4）
- [ ] 観点 LOG-02: 記録項目（timestamp / target_files / 返却ノード / granularity）が問題特定に十分か
- [ ] 観点 LOG-03: context_log に記録される target_files パスや返却内容に機密情報（PII / 内部パス / secret）が混入する余地と、そのマスキング/秘匿方針
- [ ] 観点 LOG-04: stderr の Info 診断（REQ.16/17 の【v3 差分】）出力時に stdout・終了コードが不変であること

### 2.10 FFI / 境界 API（MCP Agent Surface, 凍結契約 LGX-COMPAT-001）

- [ ] 観点 F-01: 新 MCP ツールを追加せず compile_context のオプション引数としてのみ粒度制御を提供（MCP-INV-1, REQ.06）
- [ ] 観点 F-02: MCP 入力（snake_case）→ CLI フラグ（kebab-case）の機械変換と転送順序の固定（SPEC-LGX-009 REQ.04/15）
- [ ] 観点 F-03: `_meta["anthropic/maxResultSizeChars"]=500000` 付与が本文を改変しないこと（CACHE-INV-4, MCP-INV-2）。サイズ判定責務は Rust CLI 側
- [ ] 観点 F-04: maxResultSizeChars の単位（Unicode コードポイント）が SPEC-LGX-003 REQ.13 と一致

### 2.11 領域固有観点（決定論・キャッシュ整列）

- [ ] 観点 D-01: 同一 target_files + granularity に対し常に同一順序・同一内容（CTX-INV-1, REQ.04）
- [ ] 観点 D-02: バイト単位決定論（順序・区切り・空白を含めて完全一致）（CACHE-INV-1, REQ.14）
- [ ] 観点 D-03: 5 セクションの配置順序が granularity に依らず固定（CACHE-INV-2, REQ.10）
- [ ] 観点 D-04: 各セクション内の整列規則（Layer/Additional はパス辞書順、Upstream は document=ノード ID 辞書順 / subnode=親 ID 辞書順 + アンカー出現順）（REQ.11）
- [ ] 観点 D-05: キャッシュブレーク点マーカが返却内に 1 箇所だけ固定位置に挿入される（REQ.12）
- [ ] 観点 D-06: グラフ探索発見順・エッジスコア順を整列の根拠に使わない（Target Node Metadata に隔離）（REQ.11）
- [ ] 観点 D-07: CTX-INV-2/3/4（グラフ整合性・カスタムエッジ独立性・DAG 制約）を compile_context が破らない（REQ.05）
- [ ] 観点 D-08: フラグ組合せの優先順位マトリクス（outline×document / sections×document / outline×sections / depth 直交）が一意に確定（REQ.18）

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 関連文書で回答 | 関連 GAP |
|---|---|---|---|
| 2.1 B-01 | GREEN | REQ.13「500,000 文字を超える場合」エラー（境界 = 超過で判定） | — |
| 2.1 B-02 | GREEN | REQ.13 カウント単位 = Unicode コードポイント（`.chars().count()`、SPEC-LGX-009 REQ.13 と同一） | — |
| 2.1 B-03 | GREEN | REQ.17（N=1 直接親 / 省略=無制限 7 階層） | — |
| 2.1 B-04 | GREEN | LGX-COMPAT-001 §1 グローバル規約「使用法誤り（引数パーサ層が検出する構文レベルの誤り）は全サブコマンドで exit 2（clap 既定の契約化）、受理済み値の意味的不正は exit 1」。`--depth` の非数値・小数は clap integer value_parser が構文エラーで reject → exit 2、負数等の意味的不正 → exit 1。凍結済み境界契約で確定 | — |
| 2.1 B-05 | GREEN | REQ.16（空文字列は zod min1 で reject／SPEC-LGX-009 REQ.15）, 1 件以上は完全一致フィルタ | — |
| 2.1 B-06 | GREEN | REQ.04, REQ.11（決定論的整列で件数非依存） | — |
| 2.1 B-07 | GREEN | REQ.02/REQ.10（上流ゼロでもセクション構成は維持。L-01 と併せ確認） | — |
| 2.2 E-01 | GREEN | REQ.13（切り捨て・要約せず明示的エラー + 提案文） | — |
| 2.2 E-02 | GREEN | サイズ超過は REQ.13 で型付け、入力不正（depth 等）は LGX-COMPAT-001 §1 の exit 2/1 規約で終了コードが確定、DB ロック超過は NFR REL.07「超過時は失敗として返す・無限リトライ禁止・上限 5000ms」で確定。グラフ不整合は SPEC-LGX-002 所管。compile_context 固有の型区別の未定義は GAP-041（書込失敗）/GAP-043（起点不在）に集約済で本行に固有の欠落なし | — |
| 2.2 E-03 | GREEN | MCP-INV-4「全呼出し記録」と本処理成功の両立、書込失敗時の挙動が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-041 |
| 2.2 E-04 | GREEN | LGX-EXT-002 §5.1「サイズ超過時のエラーもログ対象」（SPEC §4 MCP-INV-4 実装で継承） | — |
| 2.2 E-05 | GREEN | 上流ファイルの一部が読めない・ノード欠損時の部分成功/失敗が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-043 |
| 2.3 S-01 | GREEN | §4 FB-INV-4（DB 不在でも graph.toml 上流を返す）, REQ.05 | — |
| 2.3 S-02 | GREEN | §4 FB-INV-3（pending Proposal は結果に影響しない） | — |
| 2.3 S-03 | GREEN | §4 STATE-INV-1（context_log 書込以外の永続状態を持たない） | — |
| 2.4 C-01 | GREEN | REQ.09（各呼出し独立、応答混在禁止） | — |
| 2.4 C-02 | GREEN | REQ.09（WAL + busy_timeout 排他）, NFR SEC.02 | — |
| 2.4 C-03 | GREEN | busy_timeout 超過時のリトライ上限・終端状態自体は NFR REL.07「超過時は失敗として返す・無限リトライ禁止・上限 5000ms」で確定。残る未定義は「書込失敗を理由に compile_context 呼出し全体を失敗させるか（本処理は既に成功）」という呼出しレベルの成否結合のみ【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-041（書込失敗系として統合、残余の呼出し成否結合に範囲縮小） |
| 2.5 V-01 | GREEN | REQ.03（document 既定 / subnode の 2 値、auto は将来拡張に保留） | — |
| 2.5 V-02 | GREEN | SPEC-LGX-009 REQ.15「後方互換性」（省略時 v0.3.0 と同一 argv） | — |
| 2.5 V-03 | GREEN | REQ.07 + SPEC-LGX-008（context_log.granularity の migration を委譲） | — |
| 2.5 V-04 | GREEN | NFR COMPAT.08「LF/CRLF の両方を受容、出力は LF 統一」+ COMPAT.07「ファイル IO は UTF-8 固定」で改行/エンコーディング正規化を規定。REQ.14/CACHE-INV-1 の決定論スコープは明示的に「同じ入力（グラフ定義・engine.db 状態・引数）」であり OS は入力要素でない（出力は LF 統一済）。さらに COMPAT.01/02 で Step1=Windows / Step2=Linux は別リリース段で単一キャッシュプール前提なし。残るパス区切りはノード ID が論理 ID であり OS パスでない点で瑣末 | — |
| 2.6 P-01 | GREEN | §4 FB-INV-4 | — |
| 2.6 P-02 | GREEN | context_log 書込と本処理成功の atomicity / トランザクション境界が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-041 |
| 2.6 P-03 | GREEN | ディスクフル・権限欠如時の context_log 挙動が未定義（E-03 と同根）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-041 |
| 2.7 I-01 | GREEN | target_files が graph.toml 上にノードを持たないパスの場合の挙動（エラー / 空 / 黙殺）が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-043 |
| 2.7 I-02 | GREEN | REQ.08（サブノード起点 → 親ドキュメント上流解決、ParentChild エッジ連鎖扱い） | — |
| 2.7 I-03 | GREEN | REQ.16（親ドキュメント ID は除外 + stderr Info 診断、stdout/終了コード不変） | — |
| 2.7 I-04 | GREEN | `--sections` の空トークン・空白のみ・重複 ID の扱いが未定義。ただし trim 後空 = REQ.16「存在しない ID は除外」と同帰結、重複は REQ.11 整列 + CACHE-INV-1 で実質 set セマンティクス。残るは瑣末な明示エラー要否のみ【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-045 |
| 2.7 I-05 | GREEN | REQ.03（2 値のみ）+ SPEC-LGX-009（zod enum）/ CLI clap value_parser | — |
| 2.7 I-06 | GREEN | REQ.16/REQ.18（sections×document は無視） | — |
| 2.8 L-01 | GREEN | REQ.10（セクション構成は granularity・件数非依存で固定） | — |
| 2.8 L-02 | GREEN | `--outline-only` で h1〜h3 皆無時の出力が未定義。ただし REQ.15「本文は出力に含めない」+ REQ.10（セクション構成は件数非依存で固定）から「artifact 枠維持・body 空」と概ね導出可能。残るは DD レベルの空 body フォーマット固定のみ【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-047 |
| 2.8 L-03 | GREEN | REQ.16（フィルタ通過結果が空なら当該親は upstream に登場しない） | — |
| 2.8 L-04 | GREEN | REQ.17（CLI=空集合 exit 0 / MCP=zod reject の受理範囲差を正準化） | — |
| 2.9 LOG-01 | GREEN | REQ.07, §4 MCP-INV-4 | — |
| 2.9 LOG-02 | GREEN | SPEC-LGX-007 REQ.06（timestamp / target_files / 返却ノード / granularity） | — |
| 2.9 LOG-03 | GREEN | context_log の記録仕様・記録項目は SPEC-LGX-007 REQ.06 が所管（本 SPEC は REQ.07 で委譲）、get_compile_audit 経由の再露出は SPEC-LGX-007 REQ.07 所管、秘匿/マスキング方針は NFR SEC.05（対象を API キーに限定すると意図的に確定済）。保持期間・ローテーションはセキュリティ NFR 横断事項であり compile_context の動作要求（SPEC-LGX-003）の対象外。本 SPEC に固有の欠落なし | — |
| 2.9 LOG-04 | GREEN | REQ.16/REQ.17【v3 差分】（Info 診断時 stdout・終了コード不変を明記） | — |
| 2.10 F-01 | GREEN | REQ.06, §4 MCP-INV-1 | — |
| 2.10 F-02 | GREEN | SPEC-LGX-009 REQ.04/REQ.15（snake→kebab 変換・転送順序固定）, LGX-COMPAT-001 | — |
| 2.10 F-03 | GREEN | SPEC-LGX-009 REQ.13, §4 CACHE-INV-4 / MCP-INV-2（本文非改変、判定は Rust CLI） | — |
| 2.10 F-04 | GREEN | REQ.13 + SPEC-LGX-009 REQ.13（単位一致を明記） | — |
| 2.11 D-01 | GREEN | REQ.04, §4 CTX-INV-1 | — |
| 2.11 D-02 | GREEN | REQ.14, §4 CACHE-INV-1 | — |
| 2.11 D-03 | GREEN | REQ.10, §4 CACHE-INV-2 | — |
| 2.11 D-04 | GREEN | REQ.11（各セクション整列規則を明記） | — |
| 2.11 D-05 | GREEN | REQ.12（マーカ 1 箇所・固定位置） | — |
| 2.11 D-06 | GREEN | REQ.11（探索発見順・スコア順を整列に使わず Metadata へ隔離） | — |
| 2.11 D-07 | GREEN | REQ.05（CTX-INV-2/3/4 を破らない。実装本体は SPEC-LGX-002 へ委譲） | — |
| 2.11 D-08 | GREEN | REQ.18（フラグ組合せマトリクスを一意に確定） | — |

## 4. ステータスの決定

RED 観点が 8 件残存するため、本 TP のステータスは `**ステータス**: green`。うち 2 件（I-04 / L-02）は minor（低価値・人間判断で drop 可）。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。

RED 観点（GAP 起票対象、敵対的精査パス 2026-06-09 後）:
- 2.2 E-03 / 2.4 C-03 / 2.6 P-02 / 2.6 P-03 → GAP-LGX-041（context_log 書込失敗時の本処理成否と atomicity。**GENUINE**。C-03 は残余の呼出し成否結合に範囲縮小）
- 2.2 E-05 / 2.7 I-01 → GAP-LGX-043（上流に存在しないパス / 部分欠損の扱い。**GENUINE**）
- 2.7 I-04 → GAP-LGX-045（`--sections` の不正形式入力の扱い。**minor**, REQ.16+REQ.11 から概ね導出可、人間判断で drop 可）
- 2.8 L-02 → GAP-LGX-047（`--outline-only` で見出し皆無時の出力。**minor**, REQ.15+REQ.10 から概ね導出可、人間判断で drop 可）

### 敵対的精査パス（2026-06-09）で削除した GAP（過剰生成）:
- **GAP-LGX-042 削除**（B-04/E-02/S-01/P-01/LOG-01 に固有の RED なし）: FB-INV-4 と MCP-INV-4 の「論理的衝突」は虚構。SPEC-LGX-007 §4 が「FB-INV-4… 本 SPEC は DB 前提のため DB 不在時は observation/proposal 機能が無効化される設計」と明記済。MCP-INV-4「全呼出し記録」は DB 依存のフィードバック下位系（SPEC-LGX-007 REQ.06 所管）内で定義された不変条件であり、DB 不在という前提不成立時には記録の前提（DB 存在）が満たされないだけ。FB-INV-4 は「DB が無くてもグラフ上流を返す」を保証するのみで、記録の発生を主張していない。両者は素な状態領域で動作し矛盾しない。所管は SPEC-LGX-007。
- **GAP-LGX-044 削除**（V-04 を GREEN 化）: NFR COMPAT.08「出力は LF 統一」+ COMPAT.07「UTF-8 固定」で改行/エンコ正規化が既定。REQ.14/CACHE-INV-1 の決定論スコープは明示的に「グラフ定義・engine.db 状態・引数」で OS 非入力。
- **GAP-LGX-046 削除**（B-04 / E-02 を GREEN 化）: LGX-COMPAT-001 §1 グローバル規約（構文誤り → exit 2、意味的不正 → exit 1、clap 既定の契約化）で CLI `--depth` 不正値の終了コードが凍結済。
- **GAP-LGX-048 削除**（LOG-03 を GREEN 化）: context_log 所管は SPEC-LGX-007 REQ.06/07、マスキング方針は NFR SEC.05（API キー限定で意図的確定）。compile_context の動作要求（SPEC-LGX-003）の対象外。

## 5. 観点ナレッジベース参照

- `docs/perspectives/core-perspectives.md` §境界値, §エラーハンドリング, §状態遷移, §並行性, §バージョニング・互換性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性, §FFI / 境界 API
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（エラーメッセージの可読性: REQ.13 提案文）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 51 件（GREEN 43 / RED 8）、GAP-LGX-041〜048 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 4 件 / 維持 4 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
