Document ID: SPEC-LGX-009

# SPEC-LGX-009: MCP サーバ

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-009 |
| Version | 0.6.2 |
| Status | Approved（人間査読済） |
| Date | 2026-04-26 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §6 |
| 対応 NFR | NFR-LGX-001.COMPAT.06, PERF.03 |
| 対応 UC | UC-LGX-002, UC-LGX-004（MCP 経由）, UC-LGX-008（observe 経由） |

---

## 1. 本文書の位置づけ

### 1.1 目的

MCP サーバ（TypeScript）の要求を定義する。MCP-INV-1〜4 の維持と、Rust CLI への忠実な引数転送を中心とする。

### 1.2 スコープ

**含む:** 提供 MCP ツールの範囲、転送動作、legixy での拡張点
**含まない:** MCP プロトコル仕様そのもの（Claude 側の仕様に従う）、ビジネスロジック（Rust CLI 側）

---

## 2. 参照文書

- LGX-EXT-001 §6 MCP-INV-1 との整合
- LEGIXY-SPEC-001 §10 MCP-INV-1〜4
- 前世代 `old.source/TypeScriptMCP/` 実装を慣例仕様として参照

---

## 3. 要求事項

### SPEC-LGX-009.REQ.01: 実装言語

**内容:** MCP サーバは TypeScript（Node.js）で実装する。legixy で言語を変更しない（v0.1.0 からの継承）。
**根拠:** LEGIXY-SPEC-001 §4
**検証方法:** リポジトリ構造の確認

### SPEC-LGX-009.REQ.02: 提供ツール 3 種のみ（MCP-INV-1）

**内容:** MCP サーバが Agent（Claude Code）に公開するツールは以下の 3 つのみとする:
- `compile_context`
- `observe`
- `get_compile_audit`

新 MCP ツールの追加は**禁止**。legixy の粒度制御も `compile_context` のオプション引数で実現する。
**根拠:** LEGIXY-SPEC-001 §10 MCP-INV-1, LGX-EXT-001 §6.2
**検証方法:** MCP `tools/list` 応答の検査

### SPEC-LGX-009.REQ.03: 忠実転送（MCP-INV-2）

**内容:** MCP サーバは Rust CLI の出力を**フィルタリング・省略・加工せず**そのまま Agent に転送する。構造化変換（JSON→MCP content）は最小限とし、意味的変更は行わない。

**ロギングとマスキング（GAP-LGX-170 対応）:**
- CLI stderr をエラー応答に含める際は**マスキングせず忠実転送**する（MCP-INV-2 優先 — MCP 層での本文改変はむしろ不変条件違反となる）
- NFR SEC.05 のクレデンシャルマスキング義務は **Rust CLI 側経路（Contextual Retrieval の API キー等）の責務**であり、MCP 転送層の責務ではないことを明確化する
- MCP サーバ自身の診断ログ（呼出トレース等）は stderr への最小限の出力に留める（有無・水準の詳細は DD）。**環境変数の値（バイナリパス・トークン等）はログに出力しない**
- **exit 0 時の非空 stderr 転送（ADR-LGX-004 可観測性保証）:** Rust CLI が exit 0 で終了しても stderr が非空の場合（例: context_log 書込失敗 Warning — SPEC-LGX-003.REQ.19）、MCP サーバは成功応答の `_meta["legixy/warnings"]` フィールドに stderr 本文を格納して Agent に転送する。これにより可用性優先の副作用（書込失敗 Warning）が MCP 経由でも Agent に到達することを SPEC として保証する。`isError: false` の応答への `_meta` 追加は MCP-INV-2（忠実転送）と整合する（REQ.13 の `_meta["anthropic/maxResultSizeChars"]` と同一拡張経路）。
  - **stderr が空の場合**: `_meta["legixy/warnings"]` フィールド自体を**省略**する（空文字列 `""` の付与は不可）。Agent は当該フィールドの不在を「Warning なし」と解釈する。
  - **適用対象ツール**: 全 3 ツール（`compile_context` / `observe` / `get_compile_audit`）共通。ツール種別によらず、exit 0 + stderr 非空であれば同規則を適用する。

**根拠:** LEGIXY-SPEC-001 §10 MCP-INV-2, LGX-EXT-001 §6.4、GAP-LGX-170
**検証方法:** CLI 出力と MCP 応答の差分チェック、ログ出力に環境変数値が含まれないことの検査。exit 0 かつ stderr が非空の CLI を mock した際に `_meta["legixy/warnings"]` に stderr 本文が格納されること、stderr が空の場合は当該フィールドが応答に含まれないことの検査（全 3 ツールで確認）

### SPEC-LGX-009.REQ.04: legixy 新引数の転送（granularity）

**内容:** `compile_context` の legixy 新引数 `granularity` を Agent から受領し、Rust CLI のコマンドライン引数として転送する。サーバ側の追加ロジックは引数マッピングのみ。

**Phase 2 Block B 拡張（v0.4.0-alpha3）:** `granularity = "subnode"` 指定時は、Rust CLI が legixy-ctx の Subnode モードで本来動作（サブノード単位で artifact を展開して返却）を行う。本要求の MCP 側責務は引数転送のみで変わらない。Block B 由来の追加引数（outline_only / sections / depth）は REQ.15 で別個に規定する。
**根拠:** LGX-EXT-001 §6.3, SPEC-LGX-003 REQ.10
**検証方法:** 引数ラウンドトリップテスト（`ts-mcp/tests/integration.test.ts` T-MCP-CC-003）

### SPEC-LGX-009.REQ.05: Rust CLI プロセス起動

**内容:** MCP サーバは各ツール呼出しに対し Rust CLI を子プロセスとして起動する（短命プロセス）。**MCP サーバ側**の永続キャッシュは行わない。

**補足:** Rust CLI プロセス内部でのメモリ上キャッシュ（mmap、`HashMap` 等）は許容される。CLI プロセスは呼出し毎に終了するため、MCP サーバとしてのステートレス性は保たれる。
**根拠:** LEGIXY-SPEC-001 §2, §10 STATE-INV-1（ステートレス性）
**検証方法:** プロセス動作テスト、MCP サーバ側に状態ファイルが生成されないこと

### SPEC-LGX-009.REQ.06: パフォーマンス目標

**内容:** MCP サーバのオーバーヘッド（CLI プロセス起動含む）は `compile_context` 応答全体で NFR-LGX-001.PERF.03 を満たす必要がある。現行の正準値（NFR §3.2）は配備ステップ別に:
- **Step 1（Windows ネイティブ）: < 300 ms**【E-04 で Windows プロセス起動コストを考慮し緩和】
- **Step 2（Ubuntu Docker）: < 200 ms**

数値の正準ソースは NFR-LGX-001 §3.2 および §13 のレイテンシバジェット表であり、本 REQ は参照のみを行う（GAP-LGX-171 対応 — 旧記述「< 200 ms 暫定」は NFR 改訂後の陳腐化引用だった。意図・不変条件は不変）。

**根拠:** NFR-LGX-001.PERF.03（Step1/Step2 区別、§13 バジェット）
**検証方法:** ベンチマーク（Step 別の閾値で判定）

### SPEC-LGX-009.REQ.07: エラー転送

**内容:** Rust CLI の非ゼロ終了コードは MCP エラー応答（`isError: true`）として転送する。stderr の内容も Agent が参照可能にする。

**exit code の区別（前段ループ反復 2 で訂正確定、v3 既存挙動の正準化）:** エラー応答のメッセージ先頭に exit code を `Rust CLI failed (exit N): <stderr 本文>` 形式（v3 実在形式）で含める。これにより Agent は「検証 Error（exit 1 → 成果物を修正すべき）」と「呼び出しミス（exit 2 → 自らの引数を修正すべき）」を判別し、誤った修正ループを回避できる。
- v3 実測: 全 3 MCP ツールが既に本形式で exit code を出力している（`ts-mcp/src/tools/compile-context.ts:98`、`observe.ts:96`、`get-compile-audit.ts:60`。exitCode は `engine.ts:54-78` の `RustCliError` が保持）。本 REQ はその正準化であり実装変更を要しない。反復 1 の「v3 は区別不能」という根拠記述は誤認であり本反復で訂正（QSET-LGX-011 Q1）
- プロセス起動不能・シグナル等で数値 exit code が無い場合は v3 同様 `exit -1` として同形式で報告する（`engine.ts:67` のフォールバック）
- 互換性: MCP 応答本文は凍結対象 (a)〜(f) 外だが、本形式の維持により v3 からの観測可能差分も生じない。stderr 本文の忠実転送（MCP-INV-2）はプレフィクスの後に原文のまま維持される
- 本フォーマットは本 REQ で固定し、Agent 側パースの安定性を保証する

**根拠:** NFR-LGX-001.USE.02, MCP プロトコル、QSET-LGX-009 Q2 回答（2026-06-07 訂正版）、QSET-LGX-011 Q1 回答、SPEC-LGX-004 REQ.04（終了コード 3 値）
**検証方法:** エラーシナリオテスト（exit 1 / exit 2 それぞれの `Rust CLI failed (exit N):` プレフィクス検証）

### SPEC-LGX-009.REQ.08: 設定

**内容:** MCP サーバの起動設定（Rust CLI バイナリのパス、作業ディレクトリ等）は環境変数または起動引数で渡す。設定ファイルは必要最小限。

**初期化失敗時の挙動（GAP-LGX-168 対応）:** バイナリ解決失敗（`TRACEABILITY_ENGINE_BIN` 不正・`--engine-binary` 不正・既定名不在）・作業ディレクトリ/project-root 不正は、**サーバ起動時に fail-fast せず**、各ツール呼出し時に **REQ.07 の `Rust CLI failed (exit -1):` 形式の isError 応答**として返す（v3 実質挙動の正準化 — thin forwarder は起動時にバイナリを検証しない。設定ミスの診断はツール呼出しのエラー本文で可能）。

**根拠:** デプロイ容易性、GAP-LGX-168
**検証方法:** 起動シナリオテスト（不正バイナリパス設定でサーバ起動成功 + ツール呼出しで isError を確認する失敗ケースを含む）

### SPEC-LGX-009.REQ.09: v0.1.0 MCP サーバからの更新

**内容:** legixy の MCP サーバ更新は `compile_context` 引数の拡張（引数転送ロジックの拡張）のみで完了する。大規模なリファクタは非目標。
**根拠:** LGX-EXT-001 §6.3「実装変更は引数転送ロジックの拡張のみ」
**検証方法:** 差分レビュー

### SPEC-LGX-009.REQ.10: MCP プロトコル準拠

**内容:** MCP サーバは Anthropic MCP プロトコル仕様に準拠する。サーバ側拡張プロトコルは定義しない。
**根拠:** MCP プロトコルエコシステムとの互換性維持
**検証方法:** MCP プロトコルテスト

### SPEC-LGX-009.REQ.11: Node.js バージョン（LTS 固定）

**内容:** MCP サーバは Node.js **LTS バージョン**上で動作する。
- アクティブ LTS と維持 LTS の **2 世代**をサポート対象とする
- 非 LTS（Current）バージョンは動作保証しない
- Step 1（Windows 11 native）: 開発者が Node.js LTS をインストールすることを前提
- Step 2（Ubuntu 24.04 Docker）: Docker image 内に Node.js LTS を同梱

**根拠:** NFR-LGX-001.COMPAT.10
**検証方法:** CI で LTS の最新 2 世代に対して動作確認

### SPEC-LGX-009.REQ.12: OS 間可搬性

**内容:** MCP サーバのコードは Windows / Linux で同一。OS 差分は以下の設定のみで吸収する:
- Rust CLI バイナリのパス（Windows: `legixy.exe` / Linux: `legixy`）
- ファイルパスセパレータ（Node.js `path` モジュールで吸収）

Rust CLI バイナリのパスは環境変数または起動引数で指定する（NFR-LGX-001.COMPAT.11 の配布形態に対応）。
**根拠:** Node.js の OS 非依存性、NFR-LGX-001.COMPAT.10, COMPAT.11
**検証方法:** Step 1（Windows）と Step 2（Ubuntu Docker）の両方で同一 MCP コードが動作すること

### SPEC-LGX-009.REQ.13: MCP Result Persistence 用メタデータの付与（CACHE-INV-4）

**内容:** MCP サーバは、Claude Code v2.1.91+ が提供する MCP Result Persistence 機能を活用するため、返却ペイロードの `_meta` フィールドに以下を設定する:

```typescript
{
  content: [...],  // Rust CLI の返却本文（改変なし）
  _meta: {
    "anthropic/maxResultSizeChars": 500000,
    "legixy/warnings": "<stderr 本文>"  // exit 0 かつ stderr 非空の場合のみ存在（REQ.03）
  }
}
```

**`_meta["anthropic/maxResultSizeChars"]` 適用対象:**
| ツール | 適用 | 根拠 |
|--------|------|------|
| `compile_context` | **適用** | 返却サイズが大きくなる主要ツール |
| `get_compile_audit` | **適用** | 監査ログの返却でサイズが大きくなりうる |
| `observe` | **非適用** | 返却が確認メッセージ程度で小規模、永続化の恩恵が薄い |

**`_meta["legixy/warnings"]` 適用対象（GAP-LGX-188 対応、REQ.03 exit 0 stderr 転送）:**
| ツール | 適用 | 根拠 |
|--------|------|------|
| `compile_context` | **適用** | context_log 書込失敗 Warning 等が発生しうる（SPEC-LGX-003.REQ.19） |
| `get_compile_audit` | **適用** | REQ.03 の一般規定（exit 0 非空 stderr）に従う |
| `observe` | **適用** | REQ.03 の一般規定（exit 0 非空 stderr）に従う |

`_meta["anthropic/maxResultSizeChars"]` とは適用範囲が異なる（`observe` は `maxResultSizeChars` 非適用だが `legixy/warnings` は適用対象）。`legixy/warnings` は exit 0 + stderr 非空の場合のみ存在するフィールドであり、stderr が空の場合はフィールド自体を省略する（空文字列を格納しない）。

**単位と責務所在（前段ループ反復 1 で確定）:**
- `maxResultSizeChars` の単位は **Unicode コードポイント数**であり、SPEC-LGX-003 REQ.13 の「500,000 文字」と同一概念・同一単位である（QSET-LGX-009 Q1 / QSET-LGX-003 Q1 回答 2026-06-07。v3 実測 = Rust `.chars().count()`）
- 500,000 超過の**サイズ判定とエラー生成は Rust CLI 側**（SPEC-LGX-003 REQ.13）が行う。MCP サーバは本文サイズの判定・切り捨てを行わず、`_meta` の宣言のみを担う（v3 実測の正準化: `ts-mcp/src/tools/compile-context.ts` はメタデータ付与のみ）

**MCP-INV-2 との整合:** `_meta` フィールドの付与は Rust CLI の出力本文（`content` フィールド）に対する変更ではなく、MCP プロトコルレベルでのメタデータ追加である。本文は従来通り改変なく転送される。したがって MCP-INV-2（忠実な転送）は維持される。

**エラー応答での非付与（GAP-LGX-162 対応）:** `isError: true` のエラー応答には **`_meta` を付与しない**（v3 実測: compile-context.ts は成功経路でのみ付与〔:88-91〕、catch 経路では非付与）。`_meta` は大きな成功本文の永続化ヒントであり、短いエラー本文には意味を持たない。成功/失敗で応答構造が分岐することを正準とする（サイズ超過エラー〔CACHE-INV-3〕の本文形式は REQ.07 / cache spec §4.3 のとおり）。

**根拠:** LGX-EXT-002 §4.1〜4.4, §5.2 CACHE-INV-4, §8.5〜8.6、NFR-LGX-001.COMPAT.12
**検証方法:** MCP 返却ペイロードに `_meta["anthropic/maxResultSizeChars"]` が含まれることの検査、本文バイト列が Rust CLI 出力と一致することの検査

### SPEC-LGX-009.REQ.14: Claude Code バージョン非依存性

**内容:** `_meta["anthropic/maxResultSizeChars"]` は Claude Code v2.1.91 以降で解釈される。それ以前のバージョンでは未知のメタデータとして無視される。

したがって本機能は Claude Code のバージョンに依存しない形で実装可能である。古いバージョンでは永続化の恩恵が得られないだけで、MCP ツールの動作そのものには影響しない。
**根拠:** LGX-EXT-002 §4.5、NFR-LGX-001.COMPAT.12
**検証方法:** Claude Code v2.1.91 未満のバージョンで動作確認（手動テスト）

### SPEC-LGX-009.REQ.15: Phase 2 Block B 引数の転送（outline_only / sections / depth）

**内容:** v0.4.0-alpha3 で Rust CLI `legixy context` に追加された Block B フラグ群（SPEC-LGX-003 REQ.15〜17）を、`compile_context` MCP ツールから Agent が指定可能とする。MCP サーバは zod schema 上で以下 3 引数を optional として受領し、Rust CLI の対応フラグへ忠実に転送する。

| MCP 引数 | 型 / 制約 | Rust CLI 引数 | 由来 SPEC-LGX-003 REQ |
|----------|-----------|---------------|----------------------|
| `outline_only` | `boolean?`（optional） | `--outline-only`（値なしフラグ。`true` の時のみ付与） | REQ.15 |
| `sections` | `string?`（optional, min length 1, コンマ区切り） | `--sections <value>`（文字列を分割せず Rust に渡す） | REQ.16 |
| `depth` | `integer?`（optional, ≥ 1） | `--depth <N>`（`String(n)` で渡す） | REQ.17 |

**転送順序:** `context <files...> [--command] [--granularity] [--outline-only] [--sections] [--depth]`。Rust CLI 側 clap は引数順序に依存しないが、MCP 側のテスト（`ts-mcp/tests/tools/compile-context.test.ts`）はこの順序を pinning する。

**MCP-INV-1 維持:** 本要求は新 MCP ツールを追加せず、既存 `compile_context` の引数オプション拡張のみで実現する（LGX-EXT-001 §6.2、REQ.02）。

**MCP-INV-2 維持:** Block B フラグの転送は引数マッピングであり、Rust CLI 出力本文（`content[].text`）の改変ではない（REQ.03）。

**後方互換性:** 3 引数すべて optional のため、引数を指定しない呼び出しは v0.3.0 以前と同一の argv（`context <files...> [--command] [--granularity]`）を生成する。
**根拠:** LGX-EXT-001 §6.3（引数転送ロジックの拡張で完結）、SPEC-LGX-003 REQ.15〜17（Rust 側受け口）、ロードマップ workflow_2026-04-28_v0.4.0-ga-roadmap.md §2.2 MCP-SYNC
**検証方法:** Zod safeParse による境界条件テスト（depth=0 / 負値 / 小数 / 空 sections の reject）、Rust CLI argv の forwarding テスト（mock RustEngine の argv 配列検証）。`ts-mcp/tests/tools/compile-context.test.ts` 11 テスト。

### SPEC-LGX-009.REQ.16: CLI 子プロセスのタイムアウトと回収（GAP-LGX-169 対応、人間裁定 2026-06-10【v3 差分】）

**内容:** MCP サーバは Rust CLI 子プロセスに**タイムアウトを設ける**:
- 既定値 **30 秒**。環境変数 **`LGX_MCP_TIMEOUT_SEC`** で変更可能。**`0` 指定でタイムアウト無効**（= v3 互換動作）
- 超過時は **SIGTERM → 猶予 5 秒 → SIGKILL** で打ち切り、子プロセスは必ず回収する（ゾンビ防止）
- タイムアウト時の応答は `isError: true`・本文 `Rust CLI failed (timeout after {N}s):`（REQ.07 の形式と同系の新ケース）
- 打ち切り時、子プロセスの**部分出力（stdout/stderr）は転送しない**（全か無か — 部分本文の転送は MCP-INV-2 の忠実性を損なう）
- 【v3 差分】v3 MCP 層にタイムアウト機構は存在しない（無限待ち）。正当入力は PERF.03 予算（< 300 ms）に対し 100 倍のマージンを持つ 30 秒内に完了するため、**正当入力空間の挙動は不変**であり、変化するのは病的なハング時の挙動のみ（凍結契約の引数体系にも不変）

**根拠:** GAP-LGX-169（人間裁定: タイムアウト導入）、ADR-LGX-010、NFR-LGX-001.PERF.03
**検証方法:** ハングする mock CLI でのタイムアウト + SIGTERM/SIGKILL + 回収テスト、`LGX_MCP_TIMEOUT_SEC=0` で無限待ちとなる（タイムアウトしない）テスト、部分出力が転送されないことの検査

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| MCP-INV-1（Agent Surface 限定） | 実装 | REQ.02（3 ツールのみ公開） |
| MCP-INV-2（忠実な転送） | 実装 | REQ.03（stderr 忠実転送。exit 0 時の非空 stderr → `_meta["legixy/warnings"]` 変換も本文改変にあたらない）、REQ.13（`_meta` 付与は本文改変にあたらない） |
| MCP-INV-3（Observation 重複排除） | 関連 | SPEC-LGX-007.REQ.11 で実装。本 SPEC は転送層として整合性を損なわない |
| MCP-INV-4（監査ログ完全性） | 関連 | REQ.03（CLI 出力を改変しないため監査情報が保全される）。実装本体は SPEC-LGX-003.REQ.07 と SPEC-LGX-007.REQ.06 |
| STATE-INV-1（ステートレス性） | 実装 | REQ.05（MCP サーバは短命プロセス起動、永続状態を持たない） |
| CACHE-INV-4（メタデータ付与忠実性） | 実装 | REQ.13（`_meta` 付与は本文改変にあたらない） |

**本 SPEC が関与しない不変条件:** CTX-INV-1〜5、SUBNODE-INV-1〜6、FB-INV-1〜5、SCORE-INV-1/2、STATE-INV-2、CACHE-INV-1/2/3

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | F-01 修正: REQ.05/REQ.10 と §4 不変条件テーブルの MCP-INV-3/4 の名称誤記を LEGIXY-SPEC-001 §10 と一致。F-09 修正: REQ.05 を「MCP サーバ側の永続キャッシュ禁止、Rust CLI 内部のメモリキャッシュは許容」と明確化 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列を追加、対象外不変条件（CTX-INV-*, SUBNODE-INV-*）を明記 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.2.1 | リリース戦略明確化に伴い、REQ.11（Node.js LTS 固定）、REQ.12（OS 間可搬性）を追加 |
| 2026-04-17 | 0.3.0 | S1-d 対応: LGX-EXT-002 統合で REQ.13（`_meta["anthropic/maxResultSizeChars"]` 付与、CACHE-INV-4）、REQ.14（Claude Code バージョン非依存性）を追加。§4 表に CACHE-INV-4 と STATE-INV-1 を追加、MCP-INV-2 に REQ.13 の整合を明記 |
| 2026-04-26 | 0.4.0 | MCP-SYNC（v0.4.0-alpha4）: Phase 2 Block B 連動として REQ.04 に granularity の本来動作（subnode 展開）を追記、REQ.15（outline_only / sections / depth の転送、SPEC-LGX-003 REQ.15〜17 由来）を新設。引数マッピング表と Zod 境界制約（depth ≥ 1, sections min length 1）を明示。MCP-INV-1/2 不変条件は維持（新ツール追加なし、本文改変なし） |
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-009 回答 → SPP-LGX-009 承認）対応: REQ.13 に maxResultSizeChars の単位（Unicode コードポイント、SPEC-LGX-003 REQ.13 と同一）とサイズ判定の責務所在（Rust CLI 側判定、MCP は _meta 宣言のみ）を確定。REQ.07 にエラー応答への `[exit N]` プレフィクス埋め込み【v3 差分】を新設（Agent の検証 Error / 呼び出しミス判別） |
| 2026-06-07 | 0.5.1 | 前段ループ反復 2（QSET-LGX-011 回答 → SPP-LGX-011 承認）対応: REQ.07 の根拠記述の事実誤認（「v3 は exit code を区別不能」→ 実際は全 3 ツールが `Rust CLI failed (exit N):` 形式で出力済み）を訂正し、`[exit N]` 新フォーマット【v3 差分】を撤回。v3 実在形式の正準化に差し替え（実装変更不要化） |
| 2026-06-10 | 0.5.2 | TP[SPEC] GAP 解消（人間承認 2026-06-10）: GAP-LGX-171 対応で REQ.06 の陳腐化引用「PERF.03 < 200 ms 暫定」を現行 NFR 値（Step1 Windows < 300 ms【E-04】/ Step2 Ubuntu Docker < 200 ms）に同期し、数値の正準ソースを NFR §3.2/§13 と明示（数値同期のみ・意図不変）。NFR §13 暫定表行 237 の内部 drift は NFR 側改訂として別途提起 |
| 2026-06-12 | 0.6.1 | ADR-LGX-004 可観測性強化（spec-change 2026-06-12）: REQ.03 に exit 0 時の非空 stderr を `_meta["legixy/warnings"]` に格納転送する規定を追加（GAP-LGX-188 起票）。検証方法に対応テストを追記。§4 MCP-INV-2 行に exit 0 stderr 変換の整合性宣言を追記 |
| 2026-06-12 | 0.6.2 | GAP-LGX-188 解消: REQ.13 に `_meta["legixy/warnings"]` をスキーマに追記し、適用対象表を追加（全 3 ツール適用・observe は maxResultSizeChars 非適用だが warnings 適用）。空時フィールド省略を明示 |
| 2026-06-10 | 0.6.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10、4 件単一改訂）: GAP-LGX-162 — REQ.13 にエラー応答（isError）への _meta 非付与を確定（v3 実測の正準化、成功/失敗で応答構造分岐）。GAP-LGX-168 — REQ.08 に初期化失敗の挙動（起動 fail-fast せず呼出時 `Rust CLI failed (exit -1):` isError、v3 実質挙動の正準化）。GAP-LGX-169 — **REQ.16 新設（人間裁定: 子プロセスタイムアウト導入【v3 差分】）**: 既定 30 秒・`LGX_MCP_TIMEOUT_SEC` 変更可・0 で無効・SIGTERM→5 秒→SIGKILL・必ず回収・部分出力非転送・ADR-LGX-010。GAP-LGX-170 — REQ.03 に stderr 忠実転送（マスキングなし = MCP-INV-2 優先）・SEC.05 は Rust CLI 側責務・診断ログ最小限・環境変数値の非ログ出力 |
