Document ID: SPEC-LGX-007

# SPEC-LGX-007: フィードバックループ

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-007 |
| Version | 0.5.1 |
| Status | Approved（人間査読済） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LEGIXY-SPEC-001 §2 |
| 対応 NFR | NFR-LGX-001.MAINT.05, OBS.01 |
| 対応 UC | UC-LGX-008 |

---

## 1. 本文書の位置づけ

### 1.1 目的

フィードバックループ機能（observation / proposal / context_log）の要求を定義する。v0.1.0 で既に実装されている機能を legixy の SPEC として明文化する。

### 1.2 スコープ

**含む:** observation, proposal, context_log の仕組み、Admin Surface と Agent Surface の分離
**含まない:** 承認ロジックの詳細、具体的な分析アルゴリズム

---

## 2. 参照文書

- LEGIXY-SPEC-001 §2 トレーサビリティエンジン
- CLAUDE.md 絶対ルール5（承認権限の制限）
- v0.1.0 の feedback / analyze / proposals / approve / reject 実装を慣例仕様として参照

---

## 3. 要求事項

### SPEC-LGX-007.REQ.01: observation（Agent Surface）

**内容:** Claude Code 等の Agent は `observe` MCP ツールで気づき（ガイドラインの不足、見落とし、矛盾等）を記録できる。
- 入力: category（必須・位置引数）、気づきのテキスト（message、必須・位置引数）、関連ノード ID（任意）
- 格納先: engine.db の `observations` テーブル
- **related_id の受理方針（GAP-LGX-122 対応）**: related_id は**形式検証・実在検証を行わず受理して保存**する。observation は「graph.toml に未登録の対象への気づき」を記録する用途を含むため、実在検証で reject すると記録機会を失う。正準化（重複除去・整列）は REQ.11 の重複排除キー生成時に行う
- **message の境界（GAP-LGX-121 対応）**: **空文字列・空白のみ（trim 後 0 文字）の message は受理しない** — CLI は受理済み引数の値の意味的不正として **exit 1**（位置引数として構文上は valid のため exit 2 ではない、SPEC-LGX-004.REQ.04 の分類）、MCP 層は zod の trim 後 min(1) 相当で reject。**最大長は設けない**（SQLite TEXT。過大入力の DoS 防御は NFR SEC.04 の一般則）。改行・制御文字・Unicode は**無加工で保存**する（気づきの忠実記録を優先。サニタイズ・正規化は行わない — 機密混入の検査は NFR SEC.05 のダンプ検査で担保）

**CATEGORY 列挙値（前段ループ反復 1 で凍結）:** category は以下の 3 値に凍結する（LGX-COMPAT-001 §4.1 の凍結契約と整合）:
- `compile_miss` / `review_correction` / `manual_note`

将来の category 追加は MCP zod スキーマ変更（= 凍結境界の変更）を伴うため、次バージョンの SPEC 改訂として扱う（ハードルール 7）。

**検証の層:** MCP 層（zod enum、SPEC-LGX-009）と CLI 層の双方で 3 値を検証する。CLI 層は引数パーサの値域検証（ValueEnum 相当）として実装し、不正値は使用法誤り（exit 2）で reject する。【v3 差分】v3 の CLI 経路は無検証（`category: String`）で、不正 category は保存後 analyze で「その他 → skipped」となり observation が死蔵されていた。正当な 3 値の挙動は完全不変であり、契約文書（COMPAT §4.1）が列挙を明記済みのため互換破壊とみなさない（QSET-LGX-007 Q2 回答 2026-06-07）。

**根拠:** LEGIXY-SPEC-001 §2, CLAUDE.md MCP ツール使用ルール、LGX-COMPAT-001 §4.1
**検証方法:** MCP スキーマテスト、CLI 不正 category の exit 2 テスト、正当 3 値の受理テスト

### SPEC-LGX-007.REQ.02: feedback コマンド（Admin Surface）

**内容:** `legixy feedback` は check の結果や embedding から未対応の observation を生成する。**人間のみが CLI で実行する**。
**根拠:** CLAUDE.md, v0.1.0 継承
**検証方法:** CLI E2E テスト

### SPEC-LGX-007.REQ.03: analyze コマンド（Admin Surface）

**内容:** `legixy analyze` は observations を集約・分析し、対応する proposal を生成する。**人間のみが CLI で実行する**。

**context_log 完全性への注記:** analyze は context_log の完全性を前提とするが、REQ.06 のベストエフォート書込方針により欠落が生じている可能性がある。analyze 自身は欠落の有無を検査・報告しない（analyze の責務は observations → proposal 変換であり、context_log の完全性検証は本 SPEC の範囲外とする）。
**根拠:** CLAUDE.md, v0.1.0 継承、ADR-LGX-004 残存リスク（欠落の見逃し）の明示
**検証方法:** CLI E2E テスト

### SPEC-LGX-007.REQ.04: proposals コマンド（Admin Surface）

**内容:** `legixy proposals` は未承認の proposal 一覧を表示する。
- フィルタ: status（pending / approved / rejected）
- 出力: proposal ID、種別、対象ノード、提案内容

**根拠:** v0.1.0 継承
**検証方法:** CLI E2E テスト

### SPEC-LGX-007.REQ.05: approve / reject（Admin Surface）

**内容:** `legixy approve <proposal-id>` と `legixy reject <proposal-id> --reason <text>` は proposal の承認・却下を行う。**人間のみが実行する** — Claude Code（Agent）は実行禁止。
- 作用対象は `status = 'pending'` の proposal のみ。終端状態（approved/rejected）への再操作は exit 1 で拒否する（状態モデルの正準定義は REQ.09）
- **`--reason` の境界（GAP-LGX-124 対応）**: **空文字列・空白のみの `--reason` は拒否**する（reject_reason は監査証跡であり、空理由を「指定あり」とみなさない）。受理済み引数の値の意味的不正として **exit 1**。最大長は設けず、無加工で保存する（REQ.01 の message と同方針）

**「人間のみ」の強制手段（GAP-LGX-140 対応）:** 本要求の強制は以下の二層で構成する:
1. **技術的境界**: approve/reject/analyze/feedback を MCP ツールとして**非露出**とする（MCP-INV-1、SPEC-LGX-009.REQ.02）。Agent の正規経路からは構造的に実行不能
2. **運用規律**: CLAUDE.md ルール 5（Agent は承認系 CLI を実行しない）による宣言的強制

Agent が Bash 経由で CLI を直接 spawn し得るリスクは、**単独開発者前提（NFR SEC.08）下のリスク受容**として明示する。実行ユーザ判別等の改ざん耐性ガードは要件としない（SEC.08 と整合。宣言的規律に留める判断は ADR に記録）。

**根拠:** CLAUDE.md 絶対ルール5、GAP-LGX-140、NFR-LGX-001.SEC.08
**検証方法:** MCP ツール一覧に含まれないこと（SPEC-LGX-009.REQ.02）

### SPEC-LGX-007.REQ.06: context_log（自動記録）

**内容:** compile_context の全呼出しは engine.db の `context_log` テーブルに記録される。
- タイムスタンプ、target_files、返却ノード、granularity（legixy 新規）

**context_log INSERT 失敗時の挙動（GAP-LGX-139 対応）:** DB は存在するが context_log へ書き込めない中間ケース（ロック競合超過・ディスク満杯等）では**可用性を優先**する — 本処理（compile_context）は成功扱い・記録はベストエフォート・stderr に Warning を残す（正準挙動は SPEC-LGX-003.REQ.19）。MCP-INV-4 の完全性は「DB が利用可能な場合に限る」。本体処理と記録の**トランザクション分離**が下流設計で正当化される。なお REQ.09 の DB **破損**時 exit 1（GAP-LGX-126、Admin 書込の整合性優先）とは方針が分岐する: 139 は Agent 読取経路の可用性優先、126 は再生成不能データの保護であり、対象と保護目的が異なる（ADR に記録）。

**根拠:** SPEC-LGX-003.REQ.07, REQ.19, LEGIXY-SPEC-001 §2、GAP-LGX-139
**検証方法:** 呼出し後の DB 検査、context_log 書込不能 fixture での本体成功 + stderr Warning テスト

### SPEC-LGX-007.REQ.07: get_compile_audit（Agent Surface）

**内容:** `get_compile_audit` MCP ツールは context_log を Agent から参照可能にする。過去のコンテキスト合成履歴を取得できる。
**根拠:** LEGIXY-SPEC-001 §2, MCP-INV-1
**検証方法:** MCP スキーマテスト

### SPEC-LGX-007.REQ.08: observations テーブル

**内容:** observations テーブルは v0.1.0 スキーマを継承する。サブノード ID も格納可能（related_node_id フィールド）。

**observation 状態モデル（GAP-LGX-129 対応）:**
- status 集合: **pending / analyzing / resolved / skipped** の 4 値（`skipped` は spec-change 2026-06-13 / ADR-LGX-019 / TRIAGE §4 #13 で追加）
- 遷移: `observe` → pending、`analyze` 取込 → analyzing、対応する proposal の `approve` → resolved、`reject` または一時的失敗（claim release）→ pending（再分析対象に戻る）、**`analyze` が当該 observation のカテゴリを構造的に Proposal 変換不能（REQ.04 の変換規則に対応カテゴリが無い。例: orphan_file / semantic_similarity）と判定 → skipped（終端）**
- **skipped の意図**: 変換規則を持たないカテゴリの observation が毎回の `analyze` で pending↔analyzing を往復し続ける（永久再 claim）のを防ぐ終端状態。`analyze` は終端（resolved / skipped）の observation を再取込しない。【v3 差分】v3 は変換不能を pending に戻し死蔵していた（REQ.02 注記参照）
- resolved / skipped は**終端・不可逆**（skipped も監査証跡として永続保持。REQ.09 保持ポリシーと同様）
- REQ.11 の重複排除適用範囲 `status IN ('pending', 'analyzing')` は本状態モデルに接続する（resolved / skipped 後は同一キーで再観測可能だが、変換不能カテゴリは再び skipped に終端する）

**保持ポリシー（GAP-LGX-135 対応、人間裁定 2026-06-10: 永続保持）:** observation（resolved 含む）および proposal（approved/rejected の終端含む、REQ.09）は**監査証跡として永続保持**する。自動パージは行わず、パージコマンドも提供しない（提供する場合は凍結契約への追加となるため次バージョンの SPEC 改訂事項）。手動の SQL 操作による削除は運用責任域であり legixy は関知しない — 行削除後は同一 dedup キー（REQ.11）が「新規」として再受理される挙動のみ注記する。長期肥大化は単独開発者前提（NFR SEC.08）の運用規模では実用上問題とならない。再生成不能データの保護方針は ADR-LGX-005 と整合。

**根拠:** LGX-EXT-001 §4.3、GAP-LGX-129、GAP-LGX-135
**検証方法:** DB スキーマ検証、状態遷移テスト（各遷移 + resolved 終端性）

### SPEC-LGX-007.REQ.09: proposals テーブル

**内容:** proposals テーブルは v0.1.0 スキーマを継承。status（pending / approved / rejected）、approved_by、approved_at、reject_reason 等を持つ。

**engine.db 破損時の挙動（GAP-LGX-126 対応）:** engine.db が**破損**している場合（不在とは区別する）、**自動再生成せず exit 1 で明示的に失敗**する。observation / proposal は人間・Agent の判断記録という**再生成不能なユーザ生成データ**であり、STATE-INV-1 の「engine.db は再生成可能キャッシュ」扱いの**例外**として保護する（破損 DB を黙って作り直すと証跡が消失する）。破損の検出契機・診断メッセージの詳細は DD で確定する。

**proposal 状態モデル（GAP-LGX-127 対応、旧 GAP-128/130 統合）:**
- 遷移グラフ: **(無) → pending → {approved | rejected}** のみ。これ以外の遷移は存在しない
- approved / rejected は**終端状態であり不可逆**。終端状態の proposal への approve / reject 再操作は **exit 1 で拒否**する（承認証跡の保全。上書き・差し戻しを許さない）
- approve / reject は `status = 'pending'` の行にのみ作用する
- 並行 approve/reject の競合は `UPDATE ... WHERE status = 'pending'` の **CAS（compare-and-swap）** で解決する（更新行数 1 = 成立、0 = 他方が先行 → exit 1）。FB-INV-2（承認原子性）の実装機構
- 型システム（typestate）で表現するか実行時検証とするかは DD に委譲する

**semantic_key の正準定義（前段ループ反復 1 で確定、FB-INV-5 の実装キー）:** proposal の重複排除は kind 別の文字列キーで判定する:
- `add_chain_entry:{missing_id}`
- `add_link:{from_id}:{to_id}`（ID ペアは辞書順ソート済み）
- `update_doc:{changed_id}`

`status = 'pending'` の既存 proposal と semantic_key が一致する場合、新規 INSERT を抑止する。

**根拠:** v0.1.0 継承、QSET-LGX-007 Q1 回答（2026-06-07。v3 実測 `crates/te-feedback/src/analyzer.rs:275-312, 167-181` の正準化）、GAP-LGX-126/127
**検証方法:** DB スキーマ検証、同一 semantic_key の重複 analyze で INSERT 抑止を確認するテスト、破損 DB fixture での exit 1 テスト、終端 proposal への再操作 exit 1 テスト、並行 approve/reject の CAS テスト（成立 1・敗者 exit 1）

### SPEC-LGX-007.REQ.10: テストコード不可侵原則との両立

**内容:** proposal が「テストコード修正」の提案を生成しても、approve 後の修正は必ず人間が s1（設計者）として実施する。実装者（s2）が直接テストを書き換えることは禁止。
**根拠:** CLAUDE.md 絶対ルール1, NFR-LGX-001.MAINT.05
**検証方法:** パイプラインフックの role 別書き込み制御

### SPEC-LGX-007.REQ.11: 並行書き込み安全性（MCP-INV-3 実現）

**内容:** 複数の Claude Code セッションから `observe` が同時に呼び出された場合も、observations の整合性を保つ。同一内容の observation は重複排除される（MCP-INV-3 の実装責務）。
- engine.db への書き込みは SQLite WAL + `busy_timeout` で排他制御
- 重複排除キーの正準定義（前段ループ反復 1 で確定、v3 実測の正準化）: `(category, related_ids)` の複合一意キー。related_ids は **distinct 化（重複除去）→ 昇順ソート → JSON 文字列化**して比較する（GAP-LGX-122 対応で distinct 化ステップを追加。同一手順を REQ.09 semantic_key 生成と共有する）。適用範囲は `status IN ('pending', 'analyzing')` のみ（解決済み observation は同一キーで再観測可能。状態モデルは REQ.08）。凍結済みの比較セマンティクス（message 非包含等）は不変
- **message は重複排除キーに含まれない**（同一 category + 同一 related_ids であれば異なる message でも重複扱い）。「同一対象への同種観測の重複蓄積を防ぐ」v3 の設計を意図的選択として維持する
- 競合発生時は `BUSY` エラーを Agent に返さず、内部リトライで吸収する

**根拠:** LEGIXY-SPEC-001 §10 MCP-INV-3, NFR-LGX-001.SEC.02
**検証方法:** 並行 observe ストレステスト（同一 observation を複数並行で送信 → 1 件のみ格納）

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| FB-INV-1（Observation 冪等性） | 実装 | REQ.01（observation 記録）, REQ.11（同一内容は重複排除） |
| FB-INV-2（Proposal 承認原子性） | 実装 | REQ.05（approve/reject は単一トランザクションで完了） |
| FB-INV-3（承認前不変性） | 実装 | REQ.04（pending status の proposal 管理）, REQ.09（状態モデル: pending→{approved\|rejected} のみ・終端不可逆・CAS による並行競合解決、GAP-LGX-127） |
| FB-INV-4（DB 不在時安全性） | 関連 | SPEC-LGX-003 主導。本 SPEC は DB 前提のため DB 不在時は observation/proposal 機能が無効化される設計。DB **破損**時は不在と区別し exit 1 で明示失敗（REQ.09、GAP-LGX-126 — observation/proposal は再生成不能データとして保護） |
| FB-INV-5（Proposal 重複排除） | 実装 | REQ.09（semantic_key 相当のキーで pending 重複排除） |
| MCP-INV-1（Agent Surface 限定） | 関連 | REQ.01〜05 が Admin / Agent Surface 分離に従う。強制は MCP 非露出（技術的境界）+ CLAUDE.md ルール 5（運用規律）の二層（REQ.05、GAP-LGX-140 — Bash 直接 spawn は SEC.08 下のリスク受容） |
| MCP-INV-2（忠実な転送） | 関連 | `observe`, `get_compile_audit` の出力を MCP で加工しない。実装本体は SPEC-LGX-009 |
| MCP-INV-3（Observation 重複排除） | 実装 | REQ.11（並行 observe の重複排除） |
| MCP-INV-4（監査ログ完全性） | 実装 | REQ.06（context_log 自動記録 — 完全性は DB 利用可能時に限る、書込失敗時は可用性優先のベストエフォート、GAP-LGX-139）, REQ.07（get_compile_audit による参照） |
| STATE-INV-1（ステートレス性） | 関連 | observation/proposal は engine.db に永続化されるが、engine.db は再生成可能なキャッシュであり、MCP サーバ・Rust CLI プロセス自体はステートレス |

**本 SPEC が関与しない不変条件:** CTX-INV-1〜5、SUBNODE-INV-1〜6、SCORE-INV-1/2、STATE-INV-2、CACHE-INV-1〜4

---

## 5. Surface 分離マトリクス

| 機能 | Admin Surface (CLI) | Agent Surface (MCP) |
|------|---------------------|---------------------|
| observation 記録 | - | `observe` |
| feedback 生成 | `feedback` | - |
| analyze 実行 | `analyze` | - |
| proposals 表示 | `proposals` | - |
| approve | `approve` | - |
| reject | `reject` | - |
| context_log 記録 | 自動 | 自動 |
| context_log 参照 | - | `get_compile_audit` |

---

## 6. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草、v0.1.0 既存機能を legixy SPEC として明文化） |
| 2026-04-17 | 0.1.1-draft | F-01 修正: 不変条件テーブルの MCP-INV 名称を LEGIXY-SPEC-001 §10 と一致。F-05 追加: REQ.11（並行書き込み安全性、MCP-INV-3 の実装責務）を追加 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 と §5 を入れ替え（§4 不変条件、§5 Surface 分離）で他 SPEC と構造統一。§4 表に「役割」列を追加、対象外不変条件を明記 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-d 対応: §4 表に FB-INV-1〜5（Observation 冪等性、Proposal 承認原子性、承認前不変性、DB 不在時安全性、Proposal 重複排除）と STATE-INV-1 を追加し、既存 REQ のカバー関係を明記 |
| 2026-06-07 | 0.4.0 | 前段ループ反復 1（QSET-LGX-007 回答 → SPP-LGX-007 承認）対応: REQ.11 の重複排除キーを正準定義（(category, related_ids 昇順 JSON) 複合キー・pending/analyzing 限定・message 非包含。「content_hash 等」の乖離記述を訂正）。REQ.09 に proposal semantic_key の kind 別正準定義を追加。REQ.01 に CATEGORY 3 値凍結と CLI 層検証（不正値 exit 2【v3 差分】）を明記 |
| 2026-06-10 | 0.4.1 | TP[SPEC] GAP 解消（人間承認 2026-06-10）: GAP-LGX-122 対応で REQ.01 に related_id の無検証受理方針、REQ.11 正準化に distinct 化ステップを追加（semantic_key 生成と共有、凍結比較セマンティクス不変） |
| 2026-06-10 | 0.4.2 | GAP-LGX-126 対応: REQ.09 に engine.db 破損時（不在と区別）の「自動再生成せず exit 1」を確定。observation/proposal を再生成不能データとして STATE-INV-1 の例外として保護。§4 FB-INV-4 行を更新（ADR 記録） |
| 2026-06-10 | 0.4.3 | GAP-LGX-127 対応（旧 128/130 統合）: REQ.09 に proposal 状態モデル（pending→{approved\|rejected} のみ・終端不可逆・再操作 exit 1・CAS 並行解決）を確定。REQ.05 に pending 限定作用を明記。§4 FB-INV-3 行を更新 |
| 2026-06-10 | 0.4.4 | GAP-LGX-129 対応: REQ.08 に observation 状態モデル（pending/analyzing/resolved、resolved 終端不可逆）を確定し、REQ.11 dedup 適用範囲を状態モデルへ接続。127+129 の §3.1 状態モデル節への将来統合は ADR 付き再編として推奨（本改訂は GAP 追跡性優先で各 REQ 追記） |
| 2026-06-10 | 0.4.5 | GAP-LGX-139 対応: REQ.06 に context_log INSERT 失敗時の可用性優先（本体成功・記録ベストエフォート・stderr Warning、正準は SPEC-LGX-003.REQ.19）を確定。126 との方針分岐（Admin 整合性優先 vs Agent 可用性優先）を明記。§4 MCP-INV-4 行を更新（ADR 記録） |
| 2026-06-10 | 0.4.6 | GAP-LGX-140 対応: REQ.05 に「人間のみ」の二層強制（MCP 非露出 + CLAUDE.md ルール 5）と Bash 直接 spawn の SEC.08 下リスク受容を明文化。改ざん耐性ガードは要件外。§4 MCP-INV-1 行を更新（ADR 記録） |
| 2026-06-12 | 0.5.1 | ADR-LGX-004 可観測性強化（spec-change 2026-06-12）: REQ.03 に context_log ベストエフォート書込による欠落可能性の注記を追加。analyze 自身は欠落検出を行わず、完全性検証は本 SPEC 範囲外と明示（--audit-health 新設は実現性問題により不採用） |
| 2026-06-13 | 0.6.0 | spec-change（ADR-LGX-019、TRIAGE §4 #13）: REQ.08 observation 状態モデルに `skipped` 終端を追加（pending/analyzing/resolved/skipped の 4 値）。analyze が構造的に Proposal 変換不能と判定したカテゴリ（orphan_file / semantic_similarity 等）を skipped 終端化し永久再 claim を解消。【v3差分】v3 は変換不能を pending に戻し死蔵 |
| 2026-06-10 | 0.5.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10、3 件単一改訂）: GAP-LGX-121 — REQ.01 に message 境界（空/空白のみは exit 1・MCP は zod min(1)・最大長なし・無加工保存）。GAP-LGX-124 — REQ.05 に --reason 境界（空/空白のみ拒否 exit 1・無加工保存）。GAP-LGX-135 — REQ.08 に保持ポリシー（人間裁定: 永続保持・自動/コマンドパージなし・手動削除は運用責任域・削除後の dedup キー再利用挙動を注記） |
