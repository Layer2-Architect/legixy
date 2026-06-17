Document ID: TP-LGX-009

# TP-LGX-009: MCP サーバ

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。具体化（テストデータ・期待値）は TS 層で行う。

**親**: SPEC-LGX-009
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP がカバーする SPEC の章節:

- 対象: SPEC-LGX-009 §3（REQ.01〜REQ.15）, §4（不変条件との関係）
- 関連 SPEC §:
  - LGX-COMPAT-001 §3（グローバル規約／終了コード）, §5（MCP 3 ツールと CLI マッピング）, §7（順守チェックリスト）— 凍結境界契約
  - LGX-EXT-002 §4（MCP Result Persistence）, §5.2（CACHE-INV-4）
  - LEGIXY-SPEC-001 §10（MCP-INV-1〜4, STATE-INV-1）
  - SPEC-LGX-003 REQ.07（監査ログ）, REQ.13（500,000 文字超過エラー）, REQ.15〜17（Block B フラグ）
  - SPEC-LGX-004 REQ.04（終了コード 3 値: 0 / 1 / 2）
  - SPEC-LGX-007 REQ.11（並行 observe の重複排除, MCP-INV-3）
  - NFR-LGX-001 PERF.03, COMPAT.06/10/11/12, USE.02

本 TP は **MCP サーバ層（`ts-mcp`）の責務範囲**に観点を集中する。Rust CLI 本体の挙動（サイズ判定・監査ログ記録・重複排除の実装本体・出力整列）は SPEC-LGX-003/004/007 が所有するため、本 TP では「SPEC-009 が正しく委譲しているか」のみを問い、CLI 本体の中身は対象外とする。

## 2. 観点リスト

### 2.1 FFI / 境界 API（MCP 3 ツール凍結契約・忠実転送 — 本 SPEC の核心）

- [ ] 観点 1: 公開ツールが正確に 3 つ（`compile_context` / `observe` / `get_compile_audit`）のみであることが規定されているか（MCP-INV-1）
- [ ] 観点 2: 新 MCP ツールの追加が禁止され、粒度制御等は `compile_context` のオプション引数で実現すると規定されているか
- [ ] 観点 3: MCP 入力名（snake_case）→ CLI フラグ（kebab-case）の機械変換規約が凍結されているか（`outline_only`→`--outline-only` 等）
- [ ] 観点 4: 各ツールの zod 入力スキーマ（必須/任意、型）が LGX-COMPAT-001 §5 と一致しているか
- [ ] 観点 5: CLI 出力本文をフィルタ・省略・加工せず転送する（MCP-INV-2）と規定されているか
- [ ] 観点 6: 構造化変換（JSON → MCP content）の「最小限」の境界が判別可能か（何が許容され何が意味的変更となるか）
- [ ] 観点 7: Rust CLI バイナリ解決順（`TRACEABILITY_ENGINE_BIN` → `--engine-binary` → 既定名）が維持されているか

### 2.2 境界値

- [ ] 観点 8: `target_files` の min1 制約（空配列の reject）が規定されているか
- [ ] 観点 9: `depth ≥ 1`（depth=0 / 負値 / 小数の reject）の境界が規定されているか
- [ ] 観点 10: `sections` の min length 1（空文字列の reject）が規定されているか
- [ ] 観点 11: `audit --limit` の境界（1..=50, 既定 10）が MCP `get_compile_audit` の `limit` 転送として規定されているか
- [ ] 観点 12: `maxResultSizeChars = 500000` の単位（Unicode コードポイント）と上限値の固定が規定されているか
- [ ] 観点 13: 返却本文がちょうど 500,000 / 500,001 文字のときの責務分界（判定は Rust CLI、MCP は `_meta` 宣言のみ）が規定されているか

### 2.3 エラーハンドリング（CLI exit code → MCP error マッピング）

- [ ] 観点 14: Rust CLI の非ゼロ終了コードが MCP エラー応答（`isError: true`）に変換されると規定されているか
- [ ] 観点 15: エラーメッセージが `Rust CLI failed (exit N): <stderr 本文>` 形式（v3 実在形式）で固定されているか
- [ ] 観点 16: exit 1（検証 Error）と exit 2（呼び出しミス）が Agent から判別可能な形で転送されると規定されているか（SPEC-LGX-004 REQ.04 の 3 値との整合）
- [ ] 観点 17: 数値 exit code が得られない場合（プロセス起動不能・シグナル）の `exit -1` フォールバックが規定されているか
- [ ] 観点 18: stderr 本文がプレフィクスの後に原文のまま忠実転送される（MCP-INV-2）と規定されているか
- [ ] 観点 19: 返却サイズ超過（CACHE-INV-3 のエラー、exit 非ゼロ）時に MCP がどのエラー応答を返すか、`_meta` を付与するか否かが規定されているか
- [ ] 観点 20: CLI が stdout に途中まで出力した後に異常終了した場合（部分出力）の MCP 応答が規定されているか

### 2.4 入力検証（malformed MCP request）

- [ ] 観点 21: Agent からの MCP 入力が zod スキーマに違反した場合（必須 `target_files` 欠落・型不一致等、Block B 境界外の一般ケース）の MCP 応答が規定されているか
- [ ] 観点 22: zod 検証失敗は CLI を起動する前に MCP 層で reject されるため、exit code マッピング（観点 16）とは別経路であることが規定されているか
- [ ] 観点 23: `category` の列挙値（`compile_miss`/`review_correction`/`manual_note`）以外が渡された場合の検証主体（MCP zod か Rust CLI か）が規定されているか
- [ ] 観点 24: 引数転送時の injection 耐性（`sections` 文字列・`message` 等が CLI 引数として安全に渡されるか、シェル経由でなく argv 配列で渡されるか）が規定されているか

### 2.5 並行性（concurrent MCP calls）

- [ ] 観点 25: 複数の MCP ツール呼出しが同時に来た場合の MCP サーバ側の挙動（各呼出しが独立した短命 CLI プロセスを起動するか）が規定されているか
- [ ] 観点 26: 同時起動する CLI 子プロセス数の上限・backpressure・リソース枯渇時の挙動が規定されているか
- [ ] 観点 27: 並行 `observe` の重複排除が MCP 層を経由しても機能する（MCP-INV-3）ことの委譲先（SPEC-LGX-007 REQ.11）が明示されているか
- [ ] 観点 28: 並行呼出し下で `_meta` 付与・引数転送に共有可変状態が無い（ステートレス）ことが規定されているか

### 2.6 状態遷移 / ライフサイクル（short-lived / stateless）

- [ ] 観点 29: 各ツール呼出しごとに CLI を子プロセスとして起動し呼出し毎に終了する（短命プロセス）と規定されているか
- [ ] 観点 30: MCP サーバ側に永続状態・状態ファイルを持たない（STATE-INV-1）と規定されているか
- [ ] 観点 31: Rust CLI プロセス内部のメモリ上キャッシュ（mmap, HashMap）が許容され、それがステートレス性を破らないことが規定されているか
- [ ] 観点 32: MCP サーバの起動・初期化失敗時（CLI バイナリが解決できない等）の挙動が規定されているか
- [ ] 観点 33: CLI 子プロセスがハング／応答しない場合のタイムアウト・shutdown（プロセス回収）セマンティクスが規定されているか

### 2.7 バージョニング・互換性

- [ ] 観点 34: v0.1.0 MCP サーバからの更新が「引数転送ロジックの拡張のみ」で完了すると規定されているか（REQ.09）
- [ ] 観点 35: Block B 3 引数すべて optional のため、未指定呼出しが v0.3.0 以前と同一 argv を生成する（後方互換）と規定されているか
- [ ] 観点 36: Node.js LTS 2 世代サポート・非 LTS 非保証（COMPAT.10）が規定されているか
- [ ] 観点 37: Claude Code v2.1.91 未満で `_meta` が無視され動作に影響しない（バージョン非依存性, COMPAT.12）と規定されているか
- [ ] 観点 38: MCP プロトコル準拠・サーバ側拡張プロトコル非定義（REQ.10）が規定されているか
- [ ] 観点 39: OS 間可搬性（Windows/Linux 同一コード, バイナリパスとパスセパレータのみ差分吸収, COMPAT.11）が規定されているか

### 2.8 永続化（CACHE-INV-4 / `_meta` 非改変）

- [ ] 観点 40: `_meta["anthropic/maxResultSizeChars"]` の適用対象（compile_context / get_compile_audit 適用、observe 非適用）が規定されているか
- [ ] 観点 41: `_meta` 付与が本文（`content`）を改変しない（CACHE-INV-4 / MCP-INV-2 維持）と規定され、検証方法（本文バイト列が CLI 出力と一致）が示されているか
- [ ] 観点 42: サイズ判定・切り捨ては Rust CLI が行い、MCP は `_meta` 宣言のみ担う責務分界が規定されているか

### 2.9 ロギング・観測性

- [ ] 観点 43: MCP-INV-4（compile_context 全呼出しの監査ログ完全性）の実装本体が SPEC-LGX-003 REQ.07 / SPEC-LGX-007 REQ.06 に委譲され、MCP 層が CLI 出力を改変しないことで保全されると規定されているか
- [ ] 観点 44: MCP サーバ自身のロギング（エラー診断・呼出しトレース）の有無・水準と、機密情報（stderr 経由のパス・トークン等）のマスキング方針が規定されているか

### 2.10 設定

- [ ] 観点 45: 起動設定（CLI バイナリパス, 作業ディレクトリ）が環境変数または起動引数で渡され、設定ファイルは最小限と規定されているか（REQ.08）
- [ ] 観点 46: グローバル `--project-root` を MCP 層が常に付与して CLI を起動する（LGX-COMPAT-001 §3）形が維持されているか

### 2.11 可観測性（ADR-LGX-004 保証経路）

- [x] 観点 47: Rust CLI が exit 0 で終了しても stderr が非空の場合、MCP 成功応答の `_meta["legixy/warnings"]` に stderr 本文が格納されると規定されているか。また stderr が空の場合は `_meta["legixy/warnings"]` フィールド自体が省略されると規定されているか。適用対象ツール（`compile_context` のみか全 3 ツール対象か）が明示されているか

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC §X.Y で回答 | 関連 GAP |
|---|---|---|---|
| 2.1-1 ツール 3 種のみ | GREEN | REQ.02, §4（MCP-INV-1） | — |
| 2.1-2 新ツール追加禁止 | GREEN | REQ.02 | — |
| 2.1-3 snake→kebab 変換規約 | GREEN | LGX-COMPAT-001 §5 補足（委譲明示, REQ.04） | — |
| 2.1-4 zod スキーマ一致 | GREEN | REQ.15 表 + LGX-COMPAT-001 §5 | — |
| 2.1-5 忠実転送 | GREEN | REQ.03（MCP-INV-2） | — |
| 2.1-6 構造化変換の境界 | GREEN | REQ.03（MCP-INV-2: フィルタ・省略なし）+ REQ.13 検証方法「本文バイト列が Rust CLI 出力と一致」が許容/非許容の判別基準。バイト同一性が境界で、単一 text ブロックへの格納以外の再構造化は禁止と機械検証可能。変換メニューの列挙は DD レベル。（敵対的精査 2026-06-09: GAP-LGX-161 削除） | — |
| 2.1-7 バイナリ解決順 | GREEN | REQ.08/REQ.12 + LGX-COMPAT-001 §5 補足（委譲） | — |
| 2.2-8 target_files min1 | GREEN | REQ.15 表脈絡 + LGX-COMPAT-001 §5（min1） | — |
| 2.2-9 depth ≥ 1 境界 | GREEN | REQ.15 表・検証方法（depth=0/負/小数 reject） | — |
| 2.2-10 sections min length 1 | GREEN | REQ.15 表・検証方法（空 sections reject） | — |
| 2.2-11 limit 1..=50 既定 10 | GREEN | LGX-COMPAT-001 §4 #15 / §5（limit?: int 委譲） | — |
| 2.2-12 maxResultSizeChars 単位/上限 | GREEN | REQ.13（Unicode コードポイント, 500000 固定） | — |
| 2.2-13 500000/500001 責務分界 | GREEN | REQ.13（Rust CLI 判定, MCP は _meta 宣言のみ） | — |
| 2.3-14 非ゼロ exit → isError | GREEN | REQ.07 | — |
| 2.3-15 エラー形式固定 | GREEN | REQ.07（`Rust CLI failed (exit N):` v3 実在形式） | — |
| 2.3-16 exit 1/2 判別 | GREEN | REQ.07 + SPEC-LGX-004 REQ.04（3 値） | — |
| 2.3-17 exit -1 フォールバック | GREEN | REQ.07（engine.ts:67 フォールバック） | — |
| 2.3-18 stderr 忠実転送 | GREEN | REQ.07（プレフィクス後に原文維持, MCP-INV-2） | — |
| 2.3-19 サイズ超過時の _meta 付与可否 | GREEN | REQ.07/REQ.13 ともにエラー応答への `_meta` 付与有無を未規定。**minor (verification: low-value, 人間判断で drop 可)**: `_meta` は永続化ヒントでエラー本文には no-op【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-162 |
| 2.3-20 部分出力後の異常終了 | GREEN | REQ.07（非ゼロ exit → `isError:true` で stderr ベース転送）+ cache spec §4.3（サイズ超過は CLI が Error 本文生成・非ゼロ exit）。stdout/stderr 分離と exit→error マッピングは v3 実在挙動の正準化として REQ.07 で確定。（敵対的精査 2026-06-09: GAP-LGX-163 削除） | — |
| 2.4-21 zod 違反一般ケースの応答 | GREEN | LGX-COMPAT-001 §5 が 3 ツールの zod スキーマ（必須/任意・型・列挙）を凍結契約として規定。safeParse 失敗は zod スキーマ違反として一律 reject される。Block B 境界（REQ.15）はその一部。（敵対的精査 2026-06-09: GAP-LGX-164 削除） | — |
| 2.4-22 zod reject は CLI 起動前経路 | GREEN | zod 検証は MCP 層で行われ CLI を起動しない以上 exit code を持たない（REQ.07 の `Rust CLI failed (exit N):` は CLI 起動後のみ適用）のは自明。別経路は構造上自明で SPEC 明文化不要。（敵対的精査 2026-06-09: GAP-LGX-164 削除） | — |
| 2.4-23 category 列挙検証の主体 | GREEN | LGX-COMPAT-001 §5（凍結契約）が `category: "compile_miss"\|"review_correction"\|"manual_note"` を zod 列挙スキーマとして規定 → MCP 層 zod が第一防壁で CLI 到達前に reject。観点 2.1-4 と同根拠。（敵対的精査 2026-06-09: GAP-LGX-165 削除） | — |
| 2.4-24 引数 injection 耐性（argv 配列） | GREEN | LGX-COMPAT-001 §5 補足が argv **配列** `["context", ...target_files, "--command", v, ...]` を凍結契約として規定（シェル文字列でなく配列マーシャリング）。REQ.04/REQ.15 も「Rust CLI 引数として転送」。shell:false 等の spawn 実装と先頭ハイフン位置固定は DD レベル。（敵対的精査 2026-06-09: GAP-LGX-166 削除） | — |
| 2.5-25 並行呼出し → 独立プロセス | GREEN | REQ.05（呼出し毎に子プロセス起動・終了） | — |
| 2.5-26 子プロセス数上限/backpressure | GREEN | OUT_OF_SCOPE: 並行子プロセス上限/backpressure 機構は REQ.09「更新は引数転送ロジックの拡張のみ」（thin forwarder, 大規模リファクタは非目標）を超える新挙動。リソース枯渇時の挙動は NFR-LGX-001 REL 系の所有。（敵対的精査 2026-06-09: GAP-LGX-167 削除） | — |
| 2.5-27 並行 observe 重複排除委譲 | GREEN | §4（MCP-INV-3 → SPEC-LGX-007 REQ.11） | — |
| 2.5-28 ステートレスで共有可変状態なし | GREEN | REQ.05（STATE-INV-1, MCP 側永続キャッシュ無し） | — |
| 2.6-29 短命プロセス | GREEN | REQ.05 | — |
| 2.6-30 永続状態なし | GREEN | REQ.05, §4（STATE-INV-1） | — |
| 2.6-31 CLI 内メモリキャッシュ許容 | GREEN | REQ.05 補足 | — |
| 2.6-32 起動/初期化失敗時の挙動 | GREEN | バイナリ解決失敗・作業ディレクトリ不正等の MCP サーバ起動失敗の挙動が未定義。**minor (verification: low-value, 人間判断で drop 可)**: 呼出し時経路は REQ.07 `exit -1` フォールバック（プロセス起動不能）が実質回答済み。残る起動時 fail-fast 選択は DD【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-168 |
| 2.6-33 CLI ハング時のタイムアウト | GREEN | 子プロセスのタイムアウト・回収セマンティクスが未定義（PERF.03 予算はあるが上限ではない）。**minor (verification: low-value, 人間判断で drop 可)**: v3 互換凍結境界に MCP 側タイムアウト機構が無く、新設は REQ.09 を超える新挙動。回収は DD、ブロッキング上限は NFR-REL 所有【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-169 |
| 2.7-34 更新は引数転送拡張のみ | GREEN | REQ.09 | — |
| 2.7-35 Block B 後方互換 argv | GREEN | REQ.15（3 引数 optional, 同一 argv） | — |
| 2.7-36 Node.js LTS 2 世代 | GREEN | REQ.11（COMPAT.10） | — |
| 2.7-37 Claude Code バージョン非依存 | GREEN | REQ.14（COMPAT.12） | — |
| 2.7-38 MCP プロトコル準拠 | GREEN | REQ.10 | — |
| 2.7-39 OS 間可搬性 | GREEN | REQ.12（COMPAT.11） | — |
| 2.8-40 _meta 適用対象 | GREEN | REQ.13 表（observe 非適用） | — |
| 2.8-41 _meta 本文非改変 | GREEN | REQ.13（CACHE-INV-4, 検証方法明記） | — |
| 2.8-42 サイズ判定責務分界 | GREEN | REQ.13 | — |
| 2.9-43 監査ログ完全性の委譲 | GREEN | §4（MCP-INV-4 → SPEC-LGX-003 REQ.07 / SPEC-LGX-007 REQ.06） | — |
| 2.9-44 MCP 層自身のロギング/機密マスキング | GREEN | MCP サーバ自身の診断ログ水準・stderr 経由の機密マスキング方針が未定義。**minor (verification: low-value, 人間判断で drop 可)**: SEC.05 は Rust CLI 側 API キー対象で MCP 転送経路外。MCP-INV-2 が stderr 忠実転送を含意し方針は実質「マスキングせず」。明文化は一行クラリフィケーション【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-170 |
| 2.10-45 起動設定（env/引数） | GREEN | REQ.08 | — |
| 2.10-46 --project-root 常時付与 | GREEN | LGX-COMPAT-001 §3（委譲明示） | — |
| 2.11-47 exit 0 非空 stderr → _meta.warnings | GREEN | REQ.03（exit 0 + 非空 stderr → `_meta["legixy/warnings"]` 格納）+ REQ.13 適用対象表（全 3 ツール適用・stderr 空時はフィールド省略）。【2026-06-12 解消: GAP-LGX-188 closed、REQ.13 に適用対象表追加により (c) 明示完了】 | GAP-LGX-188 |

**追加 RED（カテゴリ横断・NFR 整合）:**

| 観点 | 判定 | SPEC §X.Y で回答 | 関連 GAP |
|---|---|---|---|
| 2.6-性能予算の参照値整合 | GREEN | REQ.06 は PERF.03「< 200 ms 暫定」と引用するが、NFR-LGX-001 PERF.03（行 81）は Step 1 Windows < 300 ms / Step 2 Linux < 200 ms に改訂済み（古い引用）。**GENUINE（doc-drift, minor）**: 敵対的精査 2026-06-09 で NFR 行 81 + 変更履歴行 276（E-04 緩和）に対し実在 drift を確認。SPEC 本文修正は人間承認要【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-171 |

## 4. ステータスの決定

- 敵対的精査パス（2026-06-09）で 11 RED のうち 6 件を REFUTE（GAP-LGX-161/163/164/165/166/167 を削除し当該観点を GREEN 化）。残存 RED は 5 件（GAP-LGX-162/168/169/170 = minor / GAP-LGX-171 = GENUINE doc-drift）。
- RED 観点が依然 5 件残るため、本 TP のステータスは `**ステータス**: green` を維持。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。

> 2026-06-12 追記（GAP-LGX-188 解消）: 観点 47（exit 0 非空 stderr → _meta.warnings）を GREEN 化。SPEC-LGX-009.REQ.13 に `_meta["legixy/warnings"]` 適用対象表を追加し、全 3 ツール適用・空時省略を明示（GAP-188 closed）。全観点 GREEN のためステータスを green に維持。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §FFI / 境界 API 観点（Marshalling, Panic/エラー, ABI 互換性, 非同期境界）, §境界値, §エラーハンドリング, §状態遷移, §並行性, §バージョニング・互換性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（エラーメッセージから「何が起きたか/どうすれば良いか」— exit 1/2 判別観点に対応）

領域固有観点として、本 SPEC が「境界 API（MCP 3 ツール凍結契約 + 忠実転送）」を核心とすることから、FFI/境界 API 観点（argv マーシャリング, 子プロセス境界, バイナリ解決の ABI 相当性, 子プロセスのライフサイクル/タイムアウト）を中心に据えた。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 46 件 + NFR 整合 1 件 = 計 47 件。GREEN 36 件 / RED 11 件（GAP-LGX-161〜171）。前段ループ 2 反復で REQ.07（exit code 形式）・REQ.13（_meta 責務）が硬化済みのため当該領域は GREEN 多数。残存 RED は MCP サーバ層固有の運用観点（並行プロセス管理, タイムアウト, malformed 入力一般ケース, 自身のロギング, エラー経路の _meta 扱い, PERF 参照値整合）に集中。 |
| 2026-06-09 | 敵対的精査パス: 削除 6 件 / 維持 5 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
| 2026-06-12 | spec-change 2026-06-12（ADR-LGX-004 可観測性強化）: 観点 47（2.11-47）を追加して RED 化（GAP-LGX-188 起票）。SPEC-LGX-009.REQ.13 適用対象表追加（GAP-188 closed）により観点 47 を GREEN 化。全観点 GREEN、本 TP は green を維持 |
