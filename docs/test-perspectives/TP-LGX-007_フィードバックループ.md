Document ID: TP-LGX-007

# TP-LGX-007: フィードバックループ

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。

**親**: SPEC-LGX-007
**ステータス**: green

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は SPEC-LGX-007（フィードバックループ: observation / proposal / context_log、Admin / Agent Surface 分離）の全要求をカバーする。

- 対象: SPEC-LGX-007 §3（REQ.01〜11）、§4（不変条件 FB-INV-1〜5, MCP-INV-1〜4, STATE-INV-1）、§5（Surface 分離マトリクス）
- 関連 SPEC §: LEGIXY-SPEC-001 §10.2/§10.4、SPEC-LGX-003（FB-INV-4 主導 / context_log 記録）、SPEC-LGX-009（MCP zod スキーマ / Agent Surface）、LGX-COMPAT-001 §4・§4.1・§5、NFR-LGX-001（SEC.02, REL.07, PERF.07, MAINT.05, OBS.01）

## 2. 観点リスト

### 2.1 境界値
- [ ] 観点 B1: `observe` の `message` 空文字列・極端長・改行/Unicode（BiDi, ZWJ, normalization）の許容範囲と挙動
- [ ] 観点 B2: `observe` の `related_id`（`--related-id` num_args=0..）が 0 個 / 多数 / 重複 ID / 存在しないノード ID を含む場合の扱い
- [ ] 観点 B3: `approve <id>` / `reject <id>` の id が 0 / 負 / i64 上限 / 存在しない id のときの挙動
- [ ] 観点 B4: `proposals` が 0 件（空状態）/ 大量件数のときの表示と出力フォーマット
- [ ] 観点 B5: `reject --reason` の空文字列・極端長の許容範囲

### 2.2 エラーハンドリング
- [ ] 観点 E1: `observe` の不正 category が exit 2（使用法誤り）で reject される（REQ.01）
- [ ] 観点 E2: 存在しない proposal-id への approve / reject の終了コードとエラー伝播（意味的不正 = exit 1 か）
- [ ] 観点 E3: approve トランザクション中の部分失敗時に何が確定し何がロールバックされるか（FB-INV-2 原子性の破れ）
- [ ] 観点 E4: SQLite ロック取得失敗（busy_timeout 上限超過）時の `observe` / `approve` のエラー方針
- [ ] 観点 E5: engine.db 破損（読込不能・スキーマ不整合）時の feedback / analyze / proposals / approve / reject の挙動

### 2.3 状態遷移（proposal lifecycle: pending → approved / rejected）
- [ ] 観点 S1: pending → approved / pending → rejected 以外の遷移（approved → rejected、rejected → approved、approved → approved 等）が禁止されるか
- [ ] 観点 S2: 既に approved / rejected な proposal への再 approve / 再 reject の挙動（冪等か、エラーか）
- [ ] 観点 S3: 終端状態（approved / rejected）の不可逆性が保証されるか
- [ ] 観点 S4: observation のライフサイクル状態（pending / analyzing / 解決済み）の遷移定義と analyze が消費する状態範囲

### 2.4 並行性
- [ ] 観点 C1: 同一 observation の並行 `observe` が 1 件のみ格納される（REQ.11, MCP-INV-3）
- [ ] 観点 C2: 同一 proposal-id への approve と reject の並行実行時の決着（last-writer か、最初の勝者か、原子性で 1 つだけ成立するか）
- [ ] 観点 C3: analyze 並行実行時の proposal 重複生成抑止（FB-INV-5 semantic_key 一意性が並行下でも保たれるか）
- [ ] 観点 C4: analyze が observation を分析中（analyzing）に同一キーの `observe` が来た場合の重複排除適用範囲

### 2.5 バージョニング・互換性
- [ ] 観点 V1: observe の位置引数スタイル `<category> <message>` 維持（旧 `--category/--message` 廃止、LGX-COMPAT-001 §4.1）
- [ ] 観点 V2: CATEGORY 3 値の凍結と将来追加が次バージョン SPEC 改訂扱いであること（REQ.01, ハードルール 7）
- [ ] 観点 V3: observations / proposals テーブルが v0.1.0 スキーマを継承し旧データを読めるか（REQ.08/09、migration path）
- [ ] 観点 V4: proposals コマンドの出力フォーマット安定性（機械可読性・`--json` グローバルフラグ対応）

### 2.6 永続化（DB）
- [ ] 観点 P1: engine.db 不在時に feedback / analyze / proposals / approve / reject が安全に無効化されるか（FB-INV-4）
- [ ] 観点 P2: engine.db への書き込みが WAL + busy_timeout で排他制御される（REQ.11, NFR SEC.02 / PERF.07）
- [ ] 観点 P3: トランザクション境界の明確化（approve の単一トランザクション範囲、observe の INSERT 範囲）
- [ ] 観点 P4: engine.db がネットワーク共有上に置かれた場合の検出（NFR REL.08 への委譲整合）
- [ ] 観点 P5: 保存途中のプロセス kill / 電源断からの回復（engine.db 再生成可能性 STATE-INV-1 との整合）

### 2.7 入力検証
- [ ] 観点 I1: CATEGORY の検証が MCP 層（zod enum）と CLI 層（ValueEnum 相当）の双方で行われる（REQ.01）
- [ ] 観点 I2: `related_id` / `missing_doc` に渡された ID の形式・実在検証の有無
- [ ] 観点 I3: proposal-id の型検証（i64 パース失敗 = 使用法誤り exit 2）
- [ ] 観点 I4: `proposals --status` の値域（pending / approved / rejected）外の値の検証

### 2.8 ライフサイクル
- [ ] 観点 L1: observations テーブル空状態での analyze / feedback の挙動（生成すべき proposal がないケース）
- [ ] 観点 L2: 解決済み observation の再観測可否（同一キーで再 INSERT 可、REQ.11）
- [ ] 観点 L3: proposal の保持ポリシー（approved / rejected 済 proposal の削除・retention の有無）
- [ ] 観点 L4: observation の保持・GC ポリシー（無限蓄積の上限・パージ条件）

### 2.9 ロギング・観測性（監査ログ）
- [ ] 観点 O1: compile_context の全呼出しが context_log に記録される（REQ.06, MCP-INV-4）
- [ ] 観点 O2: get_compile_audit が context_log を加工なしで Agent に返す（REQ.07, MCP-INV-2）
- [ ] 観点 O3: approve / reject の監査証跡（approved_by / approved_at / reject_reason）の記録完全性
- [ ] 観点 O4: 監査ログの順序保証（context_log のタイムスタンプ順 / get_compile_audit の返却順序）
- [ ] 観点 O5: 監査ログ・observation message への機密情報（PII / secret / token）混入防止
- [ ] 観点 O6: context_log 記録自体が失敗した場合の compile_context 本体への影響（記録失敗で compile_context が失敗するか）

### 2.10 FFI / 境界 API（observe MCP + Admin CLI — LGX-COMPAT-001）
- [ ] 観点 F1: MCP は 3 ツール（compile_context / observe / get_compile_audit）のみ。approve / reject / analyze 等の Admin 操作が MCP に露出しない（MCP-INV-1, REQ.05, §5）
- [ ] 観点 F2: observe の MCP→CLI 引数変換（snake_case → kebab-case、位置引数化）の忠実性（MCP-INV-2, COMPAT §5）
- [ ] 観点 F3: get_compile_audit の `--limit` 既定 10 / 上限 50（1..=50）の維持（COMPAT §4・§5）
- [ ] 観点 F4: 各サブコマンドの終了コード規約（使用法誤り exit 2 / 意味的不正 exit 1）の維持（COMPAT §3）

### 2.11 領域固有観点（権限・Surface 境界）
- [ ] 観点 D1: feedback / analyze / approve / reject が「人間のみ CLI 実行」であることの強制手段（REQ.02/03/05）
- [ ] 観点 D2: Agent（Claude Code）が approve / reject を実行できないことの保証（MCP 非露出だけで十分か、CLI 側の追加ガードが必要か）
- [ ] 観点 D3: REQ.10 テストコード不可侵: proposal が「テスト修正」を提案しても s2 が直接書き換えない role 別書き込み制御の所在
- [ ] 観点 D4: pending proposal が compile_context 結果に影響しないこと（FB-INV-3 承認前不変性）

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 関連文書 §で回答 | 関連 GAP |
|---|---|---|---|
| 2.1 B1 message 境界 | GREEN | (該当なし — 入力は位置引数 message としか定義されず長さ/Unicode 方針なし)。重大度 minor: message は dedup キー非包含の自由テキストで主フロー成立を妨げない（verification: low-value, 人間判断で drop 可、2026-06-09 敵対的精査）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-121 |
| 2.1 B2 related_id 多数/重複/不在 | GREEN | COMPAT §4.1 で num_args=0.. のみ。重複・不在 ID の扱い未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-122 |
| 2.1 B3 approve/reject id 境界 | GREEN | LGX-COMPAT-001 §3（受理済み i64 だが存在しない値 = 意味的不正 → exit 1。範囲外は parser 層で exit 2）。0/負/上限は「構文上有効だが不在」= B3 と E2 で同一規約に収束。GAP-LGX-123 は GAP-LGX-125 と同一終了コード論点の重複として削除（2026-06-09 敵対的精査） | — |
| 2.1 B4 proposals 空/大量 | GREEN | REQ.04（一覧表示・status フィルタ）。空状態は一覧 0 件として自然に従う | — |
| 2.1 B5 reason 空/極端長 | GREEN | REQ.05 / COMPAT §4 で `--reason` 必須のみ。空文字許容/長さ方針未定義。重大度 minor: 必須性は COMPAT §4 で保証済み、品質方針の補完に留まる（verification: low-value, 人間判断で drop 可、2026-06-09 敵対的精査）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-124 |
| 2.2 E1 不正 category exit 2 | GREEN | SPEC-LGX-007 REQ.01（CLI 層 exit 2 で reject 明記） | — |
| 2.2 E2 不在 proposal-id 終了コード | GREEN | LGX-COMPAT-001 §3 グローバル規約（受理済み引数の値の意味的不正・実行時失敗 = exit 1）。不在 proposal-id は受理済み i64 の意味的不正に該当し exit 1。GAP-LGX-125 は規約の機械的適用であり削除（2026-06-09 敵対的精査） | — |
| 2.2 E3 approve 部分失敗ロールバック | GREEN | REQ.05 + FB-INV-2（単一トランザクションで完了。部分失敗はロールバック） | — |
| 2.2 E4 ロック取得失敗時方針 | GREEN | NFR-LGX-001 REL.07（busy_timeout 上限 5000ms 超過時は失敗として返す）に委譲。REQ.11 は observe 競合を内部リトライで吸収 | — |
| 2.2 E5 DB 破損時挙動 | GREEN | §4 は DB 不在のみ FB-INV-4 で扱う。破損（読込可能だがスキーマ不整合）時の各 Admin コマンド挙動が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-126 |
| 2.3 S1 不正遷移禁止 | GREEN | REQ.09 に status 列挙はあるが pending 以外からの approve/reject 禁止が明記なし【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-127 |
| 2.3 S2 再 approve/再 reject | GREEN | 終端状態への再操作が冪等 no-op かエラーか未定義。遷移グラフ定義の派生のため GAP-LGX-127 に統合（旧 GAP-LGX-128 は重複として削除、2026-06-09 敵対的精査）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-127 |
| 2.3 S3 終端状態不可逆性 | GREEN | FB-INV-3 は pending 不変性のみ。approved/rejected の不可逆性が明記なし（S1 と関連）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-127 |
| 2.3 S4 observation 状態遷移 | GREEN | REQ.11 に `status IN ('pending','analyzing')` が現れるが、状態集合と遷移（→ 解決済み）の完全定義がない【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-129 |
| 2.4 C1 並行 observe 1 件格納 | GREEN | REQ.11（並行 observe の重複排除、WAL+busy_timeout、複合一意キー正準定義） | — |
| 2.4 C2 approve/reject 並行決着 | GREEN | FB-INV-2 は単一操作の原子性のみ。同一 id への approve vs reject 競合の決着規則が未定義。遷移グラフ定義（CAS `WHERE status='pending'`）の派生のため GAP-LGX-127 に統合（旧 GAP-LGX-130 は重複として削除、2026-06-09 敵対的精査）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-127 |
| 2.4 C3 並行 analyze の重複抑止 | GREEN | FB-INV-5 + REQ.09 が「同一 semantic_key で pending な proposal は最大 1 つ」という不変条件を SPEC レベルで確定済み。DB 部分一意インデックス vs アプリ直列化の選択は機械保証の実装手段であり DD/ADR の領分。GAP-LGX-131 は SPEC 段階で回答済みのため削除（2026-06-09 敵対的精査） | — |
| 2.4 C4 analyzing 中の同一キー observe | GREEN | REQ.11（適用範囲 `status IN ('pending','analyzing')` を明記） | — |
| 2.5 V1 observe 位置引数維持 | GREEN | LGX-COMPAT-001 §4.1（位置引数スタイル維持、旧フラグ廃止） | — |
| 2.5 V2 CATEGORY 凍結 | GREEN | REQ.01（3 値凍結、追加は次版 SPEC 改訂 = ハードルール 7） | — |
| 2.5 V3 旧スキーマ継承 | GREEN | REQ.08 / REQ.09（v0.1.0 スキーマ継承を明記）。migration 詳細は SPEC-LGX-008 主導 | — |
| 2.5 V4 proposals 出力安定性 | GREEN | LGX-COMPAT-001 §7 が全コマンドでのグローバル `--json` 受理を順守事項として確定。具体的な --json 出力スキーマ（フィールド名・型・列安定性）は出力本文フォーマットであり DD / SPEC-LGX-003 系（出力規定）の領分（COMPAT §1 が「出力本文のフォーマット詳細は各 SPEC が規定」と明記）。GAP-LGX-132 は OUT_OF_SCOPE として削除（2026-06-09 敵対的精査） | — |
| 2.6 P1 DB 不在時無効化 | GREEN | §4 FB-INV-4（DB 前提のため DB 不在時は observation/proposal 機能が無効化される設計。主導は SPEC-LGX-003） | — |
| 2.6 P2 WAL+busy_timeout 排他 | GREEN | REQ.11 + NFR SEC.02 / PERF.07 | — |
| 2.6 P3 トランザクション境界 | GREEN | REQ.05/FB-INV-2（approve 単一 tx）。observe の INSERT 単位は REQ.11 で重複排除キー単位として暗黙に定義 | — |
| 2.6 P4 ネットワーク共有検出 | GREEN | NFR-LGX-001 REL.08 に委譲（起動時検出で Warning） | — |
| 2.6 P5 kill/電源断回復 | GREEN | STATE-INV-1（engine.db は再生成可能なキャッシュ）+ NFR REL.01（WAL 電源断耐性）に委譲 | — |
| 2.7 I1 category 二層検証 | GREEN | REQ.01（MCP zod enum + CLI ValueEnum 相当の二層検証を明記） | — |
| 2.7 I2 related_id 形式/実在検証 | GREEN | related_id の ID 形式検証・実在ノードか否かの検証有無が未定義（B2 と関連、観点軸が異なる）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-122 |
| 2.7 I3 proposal-id 型検証 exit 2 | GREEN | COMPAT §4（id は i64）+ §3（パーサ層の構文誤り = exit 2） | — |
| 2.7 I4 proposals --status 値域検証 | GREEN | REQ.04 が値域（pending/approved/rejected）を列挙し、LGX-COMPAT-001 §3（パーサ層の構文/値域誤り = exit 2）+ REQ.01 で確立済みの ValueEnum 検証パターンが機械的に適用される（I1/I3 と同一規約）。GAP-LGX-133 は規約の機械的適用であり削除（2026-06-09 敵対的精査） | — |
| 2.8 L1 空 observations の analyze | GREEN | REQ.03（analyze は observations を集約・分析し proposal を生成）の自然な帰結として、入力 0 件 → proposal 0 件生成 → exit 0。B4（proposals 空状態）を GREEN とした同一論法。空入力での正常終了は明示不要な既定挙動。GAP-LGX-134 は ALREADY_ANSWERED として削除（2026-06-09 敵対的精査） | — |
| 2.8 L2 解決済み observation 再観測 | GREEN | REQ.11（解決済み observation は同一キーで再観測可能を明記） | — |
| 2.8 L3 proposal retention | GREEN | 終端状態 proposal の保持・削除ポリシーが未定義（observation GC = L4 を統合）。重大度 minor: 長期運用の DB 肥大化論点で主フロー成立を妨げない（verification: low-value, 人間判断で drop 可、2026-06-09 敵対的精査）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-135 |
| 2.8 L4 observation GC | GREEN | observation の無限蓄積上限・パージ条件が未定義。proposal retention と同一の DB 肥大化/保持論点のため GAP-LGX-135 に統合（旧 GAP-LGX-136 は重複として削除）。重大度 minor（人間判断で drop 可、2026-06-09 敵対的精査）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-135 |
| 2.9 O1 全 compile_context 記録 | GREEN | REQ.06 + MCP-INV-4（全呼出し記録） | — |
| 2.9 O2 audit 加工なし転送 | GREEN | REQ.07 + MCP-INV-2（忠実な転送）。本体は SPEC-LGX-009 | — |
| 2.9 O3 approve/reject 証跡記録 | GREEN | REQ.09（approved_by / approved_at / reject_reason フィールド） | — |
| 2.9 O4 監査ログ順序保証 | GREEN | context_log 記録・get_compile_audit 返却は SPEC-LGX-003 主導（REQ.07 context_log 本体 + REQ.04 CTX-INV-1 決定論的順序）。SPEC-LGX-007 は MCP-INV-4 完全性のみ関与し、順序契約は SPEC-LGX-003 / get_compile_audit 出力（SPEC-LGX-009 転送）の領分。GAP-LGX-137 は OUT_OF_SCOPE として削除（2026-06-09 敵対的精査） | — |
| 2.9 O5 機密情報混入防止 | GREEN | NFR-LGX-001 SEC.05 が「ログ・DB への記録禁止」「必ずマスキング処理」を義務化し、検証方法に **observations テーブルのダンプ検査** を明記 — engine.db 永続化内容への適用は NFR で確定済み。GAP-LGX-138 の「適用範囲が不明」という前提は誤りで ALREADY_ANSWERED として削除（2026-06-09 敵対的精査） | — |
| 2.9 O6 記録失敗時の本体影響 | GREEN | context_log 記録失敗時に compile_context 本体が失敗するか（記録は best-effort か必須か）未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-139 |
| 2.10 F1 MCP 3 ツール限定 | GREEN | MCP-INV-1 + REQ.05（approve/reject は MCP 一覧に含まれない、SPEC-LGX-009.REQ.02）+ §5 | — |
| 2.10 F2 observe 引数変換忠実性 | GREEN | LGX-COMPAT-001 §5（snake_case→kebab-case 機械変換）+ MCP-INV-2 | — |
| 2.10 F3 audit --limit 既定/上限 | GREEN | LGX-COMPAT-001 §4（1..=50、既定 10）+ §5 | — |
| 2.10 F4 終了コード規約 | GREEN | LGX-COMPAT-001 §3（exit 2 / exit 1 の規約） | — |
| 2.11 D1 人間のみ CLI 実行の強制 | GREEN | REQ.02/03/05 は「人間のみ」と宣言するが、CLI に認証/role ガードはなく、強制は MCP 非露出のみに依存。CLI 直叩きでの Agent 実行を防ぐ手段が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-140 |
| 2.11 D2 Agent の approve/reject 不可 | GREEN | MCP-INV-1 + REQ.05（MCP 非露出により Agent からは到達不能） | — |
| 2.11 D3 テスト不可侵 role 制御 | GREEN | REQ.10 + NFR MAINT.05（パイプラインフックの role 別書き込み制御に委譲） | — |
| 2.11 D4 pending の context 不変性 | GREEN | §4 FB-INV-3（pending proposal は context 結果に影響しない） | — |

集計（2026-06-09 敵対的精査パス後）: 全 49 観点中 GREEN 35 / RED 14。RED 14 観点は **9 個の GAP** に集約:
- GENUINE（6 GAP）: GAP-LGX-122（related_id 正準化、B2/I2）、GAP-LGX-126（DB 破損挙動、E5）、GAP-LGX-127（proposal 遷移グラフ・終端不可逆・再操作・並行決着、S1/S2/S3/C2 — 旧 128/130 を吸収）、GAP-LGX-129（observation 状態集合・遷移、S4）、GAP-LGX-139（context_log 記録失敗時の MCP-INV-4 vs FB-INV-4、O6）、GAP-LGX-140（人間のみ CLI 実行の強制手段、D1）
- minor（人間判断で drop 可）（3 GAP）: GAP-LGX-121（message 境界、B1）、GAP-LGX-124（reason 境界、B5）、GAP-LGX-135（proposal/observation retention、L3/L4 — 旧 136 を吸収）

正味 GENUINE 6 + minor 3 = **9 GAP ファイル**（121,122,124,126,127,129,135,139,140）。削除 11 件（123,125,128,130,131,132,133,134,136,137,138）。

> 敵対的精査の所見: 前段パスは 20 上限ぴったりに張り付き、自認どおり過剰生成していた。終了コード規約（COMPAT §3）で機械的に答えの出る境界系（123/125/133）、遷移グラフ論点の人為分割（128/130 → 127）、retention の二分割（136 → 135）、NFR/他 SPEC が所有する論点（131/132/134/137/138）を削減。残存は SPEC 段階で本当に未解決の意思決定（遷移グラフ・状態集合・破損挙動・監査完全性 vs 可用性・正準化・承認権限の強制手段）に限定。

## 4. ステータスの決定

RED 観点が残るため、本 TP のステータスは `red`。敵対的精査パス後の発行分 9 GAP（GAP-LGX-121, 122, 124, 126, 127, 129, 135, 139, 140）が close されるまで UC（UC-LGX-008）着手不可。うち 3 件（121/124/135）は minor 重大度（人間判断で drop すれば残 6 件で UC 着手可）。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値 / §エラーハンドリング / §状態遷移 / §並行性 / §バージョニング・互換性 / §永続化 / §入力検証 / §ライフサイクル / §ロギング・観測性 / §FFI/境界 API
- `docs/perspectives/core-perspectives.md` §領域固有観点（決済/取引系の「二重課金防止 = idempotency key」を observation 冪等性・proposal 重複排除に転用）
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（バルク操作の一部失敗表示 → analyze/proposals の部分結果表示）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 49 件（10 カテゴリ + 領域固有 1 カテゴリ）。GREEN 29 / RED 20、GAP-LGX-121〜140 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 11 件 / 維持 9 件（うち minor 3 件）。GREEN 35 / RED 14 に更新。削除＝123,125,128,130,131,132,133,134,136,137,138（ALREADY_ANSWERED/OUT_OF_SCOPE/DUPLICATE）。128/130→127 統合、136→135 統合 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
