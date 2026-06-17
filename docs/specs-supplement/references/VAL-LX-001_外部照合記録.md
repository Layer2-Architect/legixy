Document ID: VAL-LX-001

# VAL-LX-001: 外部照合記録

| 項目 | 内容 |
|------|------|
| Document ID | VAL-LX-001 |
| Version | 1.1.0 |
| Status | **Approved - 予備レビュー + 外部 AI 照合完了、HIGH/MEDIUM/LOW 全 Finding 対応済、G0 通過** |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 照合対象 | V-DRS-SPEC-001、TE-NEXT-EXT-001、SPEC-LX-001〜009、NFR-LX-001 |

---

## 1. 本文書の位置づけ

### 1.1 目的

workflow.md フェーズ 0（外部照合）に基づき、SPEC/NFR の技術的妥当性を外部知識と照合し、重大な矛盾・漏れ・実現性上の懸念を記録する。

### 1.2 照合プロセス

本 VAL は **ハイブリッド方式** で作成する:

1. **予備レビュー**: Claude Code 内部 AI（本セッション）による SPEC/NFR の技術的検証（§5 に記述）
2. **外部 AI 照合**: 独立した AI チャット（Gemini / ChatGPT 等）による照合（§6 に記入）
3. **統合**: 両者の指摘を §7 で集約し、重大指摘への対応を記録

※ workflow.md 本来の想定は「人間が外部 AI チャットで照合」である。本文書では Claude Code 内部 AI の予備レビューを先行させ、外部照合を人間の手で補完する運用とした。

### 1.3 G0 ゲート通過条件

- 予備レビューと外部照合の両方が完了している
- 重大指摘（Severity = HIGH 以上）への対応が完了している
- 対応内容が本文書に記録されている

---

## 2. 照合対象の概要

### 2.1 対象文書一覧

| ID | タイトル | Version | 主要内容 |
|----|---------|---------|---------|
| V-DRS-SPEC-001 | V-DRS System Top Level Specification | 1.0.0 | V-DRS 全体アーキテクチャ、不変条件（CTX-INV-1〜4、MCP-INV-1〜4） |
| TE-NEXT-EXT-001 | lexigy 次世代版 機能拡張仕様書 | 0.2.1 | サブノード化、SUBNODE-INV-1〜6、graph.toml スキーマ |
| SPEC-LX-001 | lexigy 全体要求 | 0.2.0 | 8 機能カテゴリ、不変条件 × SPEC 責任マトリクス |
| SPEC-LX-002 | グラフ基盤 | 0.2.0 | graph.toml、Node/Edge、サブノード |
| SPEC-LX-003 | コンテキスト解決 | 0.2.0 | compile_context、粒度制御、並行安全性 |
| SPEC-LX-004 | 検証 | 0.2.0 | check/check --formal、severity |
| SPEC-LX-005 | グラフ走査 | 0.2.0 | 順/逆方向走査、BFS 決定性 |
| SPEC-LX-006 | embedding とドリフト検出 | 0.2.0 | ONNX、Contextual Retrieval |
| SPEC-LX-007 | フィードバックループ | 0.2.0 | observation、proposal、MCP-INV-3 実装 |
| SPEC-LX-008 | マイグレーション | 0.2.0 | v0.1.0 → v3、opt-in 自動化 |
| SPEC-LX-009 | MCP サーバ | 0.2.1 | 3 ツール限定、Node.js LTS |
| NFR-LX-001 | 非機能要件 | 0.3.0-provisional | PERF/SEC/COMPAT/OBS/REL/MAINT/USE/HARDEN |

### 2.2 技術スタックの位置づけ

- **Rust CLI**（lexigy 本体）: クロスコンパイル、Step 1 Windows native / Step 2 Ubuntu 24.04 Docker
- **TypeScript/Node.js MCP サーバ**: OS 非依存、Node.js LTS
- **SQLite WAL**（engine.db）: ローカル排他制御
- **ONNX Runtime**（CPU only）: embedding 生成

---

## 3. 重点検証領域

### 3.1 有向グラフ設計

- graph.toml を一次データとし、matrix.md を派生ビューとする方針
- `IndexMap<NodeId, Node>` で決定論的順序を保証
- `adjacency_fwd` / `adjacency_rev` 双方向隣接リスト
- Kahn's algorithm による DAG 検証

### 3.2 サブノード方式

- 自動生成（h2/h3 見出しから SHA-256 ベースで生成）
- 明示宣言（`#s:name` 形式）
- 親子 `ParentChild` エッジの自動生成
- 見出し正規化（前後空白削除、連続空白統合）

### 3.3 SHA-256 ID 設計

- `hash_input = parent_id + "|" + heading_path.join("|")`
- SHA-256 の先頭 16 文字 hex（64 bit）を ID に使用
- SUBNODE-INV-5（ID 決定性）、SUBNODE-INV-6（ID フォーマット）で保証

### 3.4 MCP-INV-1 との整合

- MCP ツールは `compile_context`、`observe`、`get_compile_audit` の 3 つに限定
- 粒度制御は `compile_context` の引数拡張で実現
- 新 MCP ツールの追加は禁止

### 3.5 不変条件体系

CTX-INV-1〜4、MCP-INV-1〜4、SUBNODE-INV-1〜6 の計 14 件。SPEC-LX-001 §4.1 に 14 × 9 責任マトリクスを掲載。

---

## 4. 不変条件の漏れ・矛盾の確認

### 4.1 網羅性の確認

| 観点 | カバーする不変条件 | 評価 |
|------|------------------|------|
| 決定性 | CTX-INV-1 | OK |
| グラフ整合性 | CTX-INV-2、CTX-INV-4、SUBNODE-INV-4 | OK |
| エッジ独立性 | CTX-INV-3 | OK |
| MCP 面 | MCP-INV-1〜4 | OK |
| サブノード整合性 | SUBNODE-INV-1〜3、5、6 | OK |
| データ耐性（DB 破損） | （NFR.REL.01 でカバー） | OK（NFR 側） |
| 時系列整合性 | **なし** | **要検討**（→ Finding P-08） |

### 4.2 相互矛盾の有無

SPEC-LX-001 §4 のマトリクスを俯瞰した結果、明らかな矛盾は発見されなかった。

---

## 5. 予備レビュー結果（Claude Code 内部 AI による）

本節は本セッションで実施した予備レビューの結果である。独立 AI チャットによる照合と統合する必要がある。

### Finding P-01【INFO】SHA-256 先頭 16 文字 hex の衝突確率

**対象:** TE-NEXT-EXT-001 §4.5.1、SPEC-LX-002.REQ.05（SUBNODE-INV-5）

**観察:** SHA-256 の先頭 16 文字 hex = 64 bit 空間。同一 parent_id 下のサブノード N 件に対して衝突確率 ≈ N² / 2^65:
- N=1,000 → 約 1.5×10^-14
- N=10,000 → 約 1.5×10^-12
- N=100,000 → 約 1.5×10^-10

**評価:** 実用規模（1 親あたり数百〜数千サブノード）では衝突確率は無視可能。加えて SUBNODE-INV-3（ID 一意性）を check で事後検証するため二重の保護。

**推奨対応:** 現状の設計で問題なし。衝突時のエラーハンドリングが check で行われることを SPEC-LX-004 の検証カテゴリに明示するとより安心（既に SubnodeIdUniqueness として規定されているため対応済）。

### Finding P-02【MEDIUM】DAG 検証対象エッジの範囲が不明確

**対象:** SPEC-LX-002.REQ.07、CTX-INV-3、CTX-INV-4、SUBNODE-INV-4

**観察:** SPEC-LX-002.REQ.07 は Kahn's algorithm で DAG 制約を検証すると規定するが、対象となるエッジ種別（Chain / Custom / ParentChild のうちどれを含めるか）が明記されていない。CTX-INV-3 は「カスタムエッジは chain 上流に影響しない」と規定するため、カスタムエッジは DAG 検証対象から除外する解釈もありうる。

**評価:** 設計上、循環は全エッジ種別で発生しうる（例: カスタムエッジ A→B、Chain B→A）。仕様が曖昧なため実装ぶれが発生する可能性。

**推奨対応:** SPEC-LX-002.REQ.07 に「Chain / Custom / ParentChild の全エッジ種別を DAG 検証対象とする」と一文追加。あるいは「Chain + ParentChild のみを DAG 検証対象とし、Custom は警告扱い」と方針を明示。

### Finding P-03【LOW】IndexMap 順の決定性は TOML パーサ依存

**対象:** SPEC-LX-002.REQ.08、CTX-INV-1

**観察:** IndexMap による決定論的順序を保証するには、graph.toml パース時に TOML ファイル内のノード定義順が維持される必要がある。Rust の `toml` crate は `indexmap` フィーチャにより順序保持対応するが、この要件が SPEC レベルで明記されていない。

**評価:** 実装時に `serde(transparent)` と `IndexMap` の併用、または `toml::Table` のフィールドアクセス順に注意が必要。実装漏れがあると CTX-INV-1 違反になる。

**推奨対応:** SPEC-LX-002.REQ.08 または対応 DD に「TOML パーサはノード定義順を保持するもの（toml crate の preserve_order フィーチャ等）を用いる」と明記。

### Finding P-04【MEDIUM】Contextual Retrieval の外部 API 依存リスク

**対象:** SPEC-LX-006.REQ.06、NFR-LX-001.SEC.05

**観察:** SPEC-LX-006 REQ.06 で Contextual Retrieval は LLM API 呼出しを前提とする。API キー管理（SEC.05）、レート制限、プロバイダ障害時のタイムアウト・リトライ戦略が未定義。

**評価:** デフォルト無効のため通常動作に影響しないが、有効化時にネットワーク不通・API エラーで embed が永続的に失敗する可能性。

**推奨対応:** SPEC-LX-006 または NFR-LX-001.REL に以下を追加:
- Contextual Retrieval 呼出しのタイムアウト値（暫定: 30 秒）
- リトライ戦略（指数バックオフ、最大 3 回）
- 障害時の代替動作（Contextual Retrieval なしの通常 embedding にフォールバック）

### Finding P-05【HIGH】PERF.01 cold start と PERF.03 compile_context の予算整合性

**対象:** NFR-LX-001.PERF.01、PERF.03、PERF.04、SPEC-LX-009.REQ.05

**観察:** NFR.PERF 目標値の内訳:
- PERF.01 CLI cold start < 50 ms （`--help` 表示まで）
- PERF.04 graph.toml パース < 100 ms （ノード 1,000 規模）
- PERF.03 compile_context 応答 < 200 ms

compile_context 呼出しのパスは「プロセス起動 + graph パース + 走査 + 解決 + 出力」となり、PERF.01 と PERF.04 を足すと 150 ms 消費、残り 50 ms で走査・解決・出力を完了する必要がある。SPEC-LX-009 REQ.05 で CLI プロセスが短命（呼出し毎起動）と規定されているため、毎回 graph パースが発生する。

**評価:** 目標達成には以下のいずれかが必要:
- 目標値を緩和（例: PERF.03 < 300 ms）
- graph パース最適化（mmap、部分パース）
- PERF.01 の 50 ms は `--help` 専用であり、実際の compile_context 呼出しはより長くなることを認める

NFR §3.1 の測定環境（i5-12400F）では実現可能性あるが、暫定値として Phase 4 で再評価する方針が §13 にあるため致命的ではない。

**推奨対応:** NFR §13 の再評価トリガに本 Finding を追記。Phase 4 実測時に PERF.01 / PERF.03 / PERF.04 の合計バジェットを検証し、必要なら目標値を見直す。

### Finding P-06【MEDIUM】Windows / Linux 間の ONNX 性能差

**対象:** NFR-LX-001.PERF.08、COMPAT.01（Step 1 Windows）、COMPAT.02（Step 2 Ubuntu Docker）

**観察:** PERF.08（embedding 50 nodes/sec、CPU only）は Windows 11 開発環境で測定される。Step 2 の Ubuntu Docker 環境では ONNX Runtime の Linux 実装、コンテナオーバーヘッドが性能に影響する可能性。

**評価:** 懸念（Linux 再測定必要）は妥当。具体的な差分幅は実測するまで不明（外部照合 Finding E-07 でも数値仮説は根拠不足と指摘された）。

**推奨対応:** NFR §3.1 に「Step 2 着手時に Linux 環境で再測定」と既に記載済（対応済）。追加対応不要。

### Finding P-07【MEDIUM】SQLite busy_timeout 値が未規定

**対象:** SPEC-LX-003.REQ.09、SPEC-LX-007.REQ.11、NFR-LX-001.SEC.02

**観察:** 並行呼出し安全性の REQ で「SQLite WAL + busy_timeout による排他制御」と規定するが、`busy_timeout` の具体値が未定義。無限リトライは Agent のタイムアウトと衝突する可能性。

**評価:** 典型的な推奨値は 5000 ms（5 秒）。v3 の想定ワークロード（開発者 1 名、並行 Claude Code セッション最大 4〜5）では 5000 ms が妥当。

**推奨対応:** NFR-LX-001.REL に新項目追加を推奨:
- `NFR-LX-001.REL.07 SQLite busy_timeout`: **5000 ms**（暫定）

### Finding P-08【INFO】時系列整合性の不変条件が未定義

**対象:** 全 SPEC、特に SPEC-LX-007（context_log の記録順）

**観察:** 不変条件体系は 14 件あるが、時系列整合性（context_log.created_at の単調増加、embeddings.created_at の古さで drift 判定等）は INV として定義されていない。

**評価:** 現状の設計では各テーブルの `created_at` で時系列を扱うが、NTP ずれやクロックスキップ時の挙動が未規定。実用上は問題ないことが多いが、厳密性のためには INV 化も検討余地あり。

**推奨対応:** 緊急性なし。将来 SUBNODE/CTX 相当の新カテゴリ TIME-INV の導入を検討候補として残す。

### Finding P-09【INFO】テストコード不可侵原則の実装完了

**対象:** CLAUDE.md 絶対ルール1、SPEC-LX-007.REQ.10、本セッションで実装した gate-check.ps1

**観察:** テストコード不可侵原則（s2 が tests/* に書き込み不可）はフック（`.claude/hooks/gate-check.ps1`）で強制される設計となり、動作確認済。

**評価:** プロセス強制が実装レベルで確立済。

**推奨対応:** 対応済。追加不要。

### Finding P-10【LOW】PIPE_ROLE 環境変数の耐性

**対象:** 本セッションで実装した pipe-start.ps1 / gate-check.ps1

**観察:** 役割識別は環境変数 `$env:PIPE_ROLE` に依存。子プロセスから上書き可能であり、悪意ある子プロセスが role を偽装できる。

**評価:** 本システムは単独開発者の環境内で動くため、悪意ある子プロセスは想定外。実害はないが、将来マルチユーザ運用時の脅威モデルに追加すべき。

**推奨対応:** NFR-LX-001.SEC に「パイプライン役割識別は単独開発者環境を前提とする（環境変数の改ざんは想定外）」との明示を追加検討。

### 予備レビューまとめ

| Severity | 件数 | Finding ID |
|----------|------|-----------|
| HIGH | 1 | P-05 |
| MEDIUM | 4 | P-02, P-04, P-06, P-07 |
| LOW | 2 | P-03, P-10 |
| INFO | 3 | P-01, P-08, P-09 |

---

## 6. 外部 AI 照合結果

### 6.1 実施状況

**実施完了**（2026-04-17）

- **ChatGPT** による外部照合: 2 件の新規 Finding 提示（SQLite WAL の Docker 配置条件、ONNX 性能差の記述根拠）+ 予備 Finding 4 件（P-02/P-04/P-05/P-07）の妥当性確認
- **Gemini** による外部照合: 5 件の新規 Finding 提示（ダングリング参照、クレデンシャル漏洩、Windows プロセス起動コスト、v0.1.0→v3 ID 互換性、Markdown 正規化）

合計 **7 件の外部 Finding** を §6.3 に記録、総括を §6.4 に記述。

### 6.2 照合手順（実施済）

使用したプロンプト例:

> 添付の要求仕様について、以下の観点で技術的妥当性を検証してください:
> 1. 有向グラフ設計の表現力と DAG 制約の実装可能性
> 2. SHA-256 ハッシュ 16 文字を ID に用いることの衝突リスク
> 3. 不変条件（14 件）の網羅性・相互整合性
> 4. パフォーマンス目標値（cold start 50 ms、compile_context 200 ms、embedding 50 nodes/sec）の実現性
> 5. セキュリティ要件（入力検証、API キー管理、排他制御）の十分性
> 6. v0.1.0 からのマイグレーション戦略の非破壊性
>
> 発見した懸念を Finding として列挙し、Severity（HIGH/MEDIUM/LOW/INFO）を付けてください。

### 6.3 外部 AI Finding

予備レビュー（§5）の Finding P-XX への確認に加え、新規 Finding を E-01〜E-07 として記録する。

---

### Finding E-01【HIGH】ダングリング・エッジ（参照切れ）への対処未規定（Gemini）

**対象:** SPEC-LX-001 §4（不変条件体系）、SPEC-LX-002（グラフ基盤）

**観察:** サブノードは Markdown 見出し（h2/h3）から自動生成される。運用中に見出しが削除・変更されると、他ノードからの `custom` エッジが存在しないノード ID を指すダングリング状態が発生する。現状の 14 件の不変条件には、この「参照整合性喪失」に対するエンジンの振る舞い（例外とするか、警告として部分グラフ構築を許容するか）が規定されていない。

**評価:** 構造的完全性を重んじるシステムで、参照切れの未定義はパニックや無限ループの要因となる。ドキュメント駆動で動的にグラフが変化する本システムでは、ダングリングは必然的に発生する状態であり、これを制御する不変条件の欠落はアーキテクチャ上の盲点。

**推奨対応:** 新しい不変条件（例: `GRAPH-INV-1 未解決エッジ許容性`）を追加し、以下のいずれかを明確化する:
- DAG 検証時には未解決エッジを無視し、`check` コマンドで警告（Warning）を出す
- 未解決エッジが存在するグラフは不整合とし Error 扱い

### Finding E-02【HIGH】context_log へのクレデンシャル漏洩リスク（Gemini）

**対象:** SPEC-LX-006.REQ.06（Contextual Retrieval）、SPEC-LX-007（フィードバックループ）、NFR-LX-001.SEC.05

**観察:** Contextual Retrieval では LLM API を呼び出すが、通信エラーやプロンプト情報が `observation` として `context_log` テーブルに記録される可能性がある。環境変数で渡された API キーがエラーダンプやペイロードに含まれ、SQLite DB 上に平文で永続化されるリスクが明示的に排除されていない。

**評価:** ローカル環境前提とはいえ、認証情報が DB に永続化されることはセキュリティ要件の重大な違反。

**推奨対応:** NFR-LX-001.SEC に「ログ記録前のクレデンシャル（API キー等）マスキング処理の義務付け」を追加。具体的には SEC.05 を拡張して「API キー文字列を含む可能性のあるエラー情報は DB/ログに記録する前に `***` 等でマスクする」と規定。

### Finding E-03【MEDIUM】SQLite WAL の Docker 配置条件が未定義（ChatGPT）

**対象:** NFR-LX-001.COMPAT.02（Step 2 Ubuntu Docker）、SPEC-LX-008（マイグレーション）、engine.db 配置

**観察:** SQLite 公式ドキュメントは WAL モードが**同一マシン上のプロセス**を前提とし、共有メモリを使うためネットワークファイルシステムには不向きと明示している（参照: sqlite.org/wal.html）。しかし Step 2 Docker 移行時に engine.db をどこに配置するか（コンテナ内ローカル / bind mount / volume）の要件が未規定。特に Windows ホスト上の共有パスや SMB/NFS 越しストレージを bind mount する運用は避ける必要がある。

**評価:** 配置条件を誤ると、ロック・共有メモリ前提が崩れて DB 破損リスクが発生する。Step 2 の実装前に明確化すべき。

**推奨対応:** NFR-LX-001.REL または SPEC-LX-008 に以下を追加:
- engine.db は **Docker ローカルボリューム**または**コンテナ内ローカルファイルシステム**上に配置する
- ネットワーク共有（SMB / NFS / CIFS 等）上の配置を**禁止**する
- bind mount 利用時はホストがローカルファイルシステムであることを前提とする

### Finding E-04【MEDIUM】Windows 環境での CLI プロセス起動オーバーヘッド（Gemini）

**対象:** NFR-LX-001.PERF.03（compile_context < 200 ms）、SPEC-LX-009.REQ.05（短命プロセス）

**観察:** Windows 上で Node.js（MCP サーバ）から Rust バイナリ（CLI）を `child_process.spawn` で都度起動する場合、OS 側のプロセス生成オーバーヘッドが 10〜30 ms 発生するケースが多い（Windows の CreateProcess は Linux の fork + exec より重い）。予備 Finding P-05 で指摘された予算問題（PERF.01 50 ms + PERF.04 100 ms = 150 ms 消費）と合わせて、200 ms バジェット達成がさらに厳しくなる。

**評価:** Rust 内の最適化（TOML パーサ等）だけでは OS レベルの起動コストを吸収しきれない可能性。

**推奨対応（いずれか）:**
- NFR.PERF.03 の目標値を Windows 環境に限り **300 ms に緩和**
- 将来的に CLI ではなく **NAPI-RS 等による Node.js 内ネイティブアドオン化**（Node.js と同一プロセス実行）を NFR の展望として記載
- Phase 4 実測時に再評価（予備 Finding P-05 の再評価トリガに統合）

### Finding E-05【MEDIUM】v0.1.0 → v3 ID 互換性の断絶（Gemini）

**対象:** SPEC-LX-008（マイグレーション）、TE-NEXT-EXT-001 §4.5.1

**観察:** v3 で導入される SHA-256 ベースの自動生成サブノード ID は、v0.1.0 時代の既存 graph.toml（仮に存在するとして）に手動記述されていた ID と非互換。v0.1.0 は元々サブノード未対応のため、ドキュメント ID（UC-LX-001 等）については互換性は保たれるが、**graph.toml 自体を v0.1.0 で編集していた場合の参照切れ**が発生しうる。

**評価:** SPEC-LX-008 REQ.02 の「非破壊性」を謳うマイグレーション戦略において、既存の ID 参照が壊れることはユーザの信頼を損なう。本プロジェクトでは v0.1.0 の graph.toml は未使用のため影響は小さいが、将来の v3 ユーザが v0.1.0 から移行するシナリオでは問題化しうる。

**推奨対応:** SPEC-LX-008 に以下を追加:
- v0.1.0 のレガシー ID と v3 のハッシュ ID の**マッピングテーブル自動生成**
- または graph.toml 内の ID 参照の**自動書き換えスクリプト**の提供
- マイグレーション結果として非互換となる ID を `migrate --dry-run` で提示

### Finding E-06【LOW】ID 生成入力の Markdown 正規化規則が不十分（Gemini）

**対象:** TE-NEXT-EXT-001 §4.5.1（SHA-256 ID 生成）、§4.6（見出し正規化）

**観察:** ハッシュ生成に使われる `heading_path` は正規化ルール（前後空白削除、連続空白統合）が規定されているが、Markdown 装飾文字（`**太字**`、`_italic_`、`#`等）の扱いが不明確。Markdown フォーマッタ（Prettier 等）で `##  見出し` → `## 見出し` や `**強調**見出し` の追加等で正規化前文字列が変化し、ID が壊れる可能性。

**評価:** ハッシュ衝突自体（P-01）は数学的に無視可能だが、入力文字列の揺れによる **ID の不安定性**が運用で発生しうる。

**推奨対応:** TE-NEXT-EXT-001 §4.6 または SPEC-LX-002 に「ハッシュ生成前の文字列正規化に以下を含む」を追記:
- Markdown 装飾文字（`*`、`_`、`` ` ``）のストリップ
- 全角半角スペース統一（半角へ）
- Unicode NFC 正規化

### Finding E-07【LOW】ONNX 性能差の記述根拠が弱い（ChatGPT）

**対象:** 予備 Finding P-06（Windows/Linux 間の ONNX 性能差）

**観察:** ONNX Runtime 自体がクロスプラットフォームなのは公式に明記されているが、予備レビューで記述した「±20% 程度に収まることが多い」は経験則であり、外部一次情報では裏付けられない。

**評価:** 懸念そのもの（Linux 再測定必要）は妥当だが、数値仮説（±20%）は仕様根拠として弱い。

**推奨対応:** P-06 の記述から「±20%」という数値を削除し、「Step 2 着手時に Linux 環境で再測定必須」のみを残す。

---

### 6.4 外部 AI 照合の総括

**ChatGPT** と **Gemini** の 2 つの独立した外部 AI による照合を実施した。両者は異なる観点から次の示唆を提供した:

- **ChatGPT**: 予備 Finding（P-02/P-04/P-05/P-07）の妥当性を外部知識（Rust toml crate、Anthropic Contextual Retrieval、SQLite 公式、NIST FIPS 180-4）で裏付け。新規に **運用条件**（SQLite WAL × Docker）と**根拠の置き方**（ONNX 性能差）の 2 件を指摘
- **Gemini**: **システム境界**（I/O、プロセス間通信、ファイル変更時の状態不整合）での例外処理の弱さを指摘。ダングリング参照、クレデンシャル漏洩、プロセス起動コスト、ID 互換性、Markdown 正規化の 5 件

**コア・ロジックの方向性は妥当**だが、実装の境界（Boundary）に対する防壁を仕様レベルで補強する必要がある。特に以下の 2 点は G0 通過前に対応方針を明確化すべき:

- **E-01（ダングリング参照）**: 動的グラフで必然的に発生する状態を不変条件として定義する
- **E-02（クレデンシャル漏洩）**: ログ記録前のマスキング処理を NFR に義務付ける

また **P-05 + E-04（PERF バジェット）** は Windows 環境での実現性を Phase 4 実測で厳密に検証する必要がある。

---

## 7. 重大指摘の統合と対応

予備レビュー（§5）と外部照合（§6）の指摘を統合する。

### 7.1 Severity 別集約（統合後）

| Severity | 予備レビュー | 外部 AI | 統合後 |
|----------|:-----------:|:------:|:------:|
| HIGH | 1（P-05） | 2（E-01, E-02） | **3** |
| MEDIUM | 4（P-02, P-04, P-06, P-07） | 3（E-03, E-04, E-05） | **7** |
| LOW | 2（P-03, P-10） | 2（E-06, E-07） | **4** |
| INFO | 3（P-01, P-08, P-09） | 0 | **3** |
| **合計** | **10** | **7** | **17** |

### 7.2 対応が必要な Finding とステータス

#### HIGH（G0 通過前に必須対応）

| Finding | 対応内容 | 対応先 | 状態 |
|---------|---------|-------|-----|
| **P-05** PERF バジェット整合性 | NFR §13 に PERF バジェット整合性の補足を追記、Phase 4 実測で再評価 | NFR-LX-001 §13 | 🟢 **対応済** |
| **E-01** ダングリング・エッジ | CTX-INV-5「未解決エッジの許容性」を新設、SPEC-LX-001 §4 マトリクス 14→15 行、SPEC-LX-002.REQ.11 実装、SPEC-LX-004.REQ.10 検証 | SPEC-LX-001/002/004 | 🟢 **対応済** |
| **E-02** クレデンシャル漏洩 | NFR.SEC.05 にマスキング義務を追記（「API キーが含まれる可能性のあるエラー情報は必ずマスキング」） | NFR-LX-001.SEC.05 | 🟢 **対応済** |

#### MEDIUM（対応推奨）

| Finding | 対応内容 | 対応先 | 状態 |
|---------|---------|-------|-----|
| **P-02** DAG 検証対象エッジ | SPEC-LX-002.REQ.07 に「Chain / Custom / ParentChild の全種別対象」と明記 | SPEC-LX-002.REQ.07 | 🟢 **対応済** |
| **P-04** Contextual Retrieval 障害時動作 | SPEC-LX-006.REQ.06.1 を新設（タイムアウト 30 秒、指数バックオフ 3 回リトライ、無効扱いでの継続） | SPEC-LX-006.REQ.06.1 | 🟢 **対応済** |
| **P-06** Win/Linux 性能差 | §3.1 Step 2 再測定は既存、E-07 で「±20%」削除 | NFR §3.1 / 本 VAL §5 P-06 | 🟢 **対応済** |
| **P-07** SQLite busy_timeout | NFR.REL.07 として「上限時間 5000 ms、無限リトライ禁止」を明記 | NFR-LX-001.REL.07 | 🟢 **対応済** |
| **E-03** SQLite WAL × Docker 配置条件 | NFR.REL.08 engine.db 配置条件、SPEC-LX-008.REQ.12 追加 | NFR-LX-001.REL.08 / SPEC-LX-008.REQ.12 | 🟢 **対応済** |
| **E-04** Windows プロセス起動コスト | PERF.03 を Step 1 Windows で 300 ms に緩和、PERF 末尾に NAPI-RS 展望 | NFR-LX-001.PERF.03 | 🟢 **対応済** |
| **E-05** v0.1.0 → v3 ID 互換性 | SPEC-LX-008.REQ.11 として ID マッピング自動生成 + 自動書き換え + --dry-run 対応 | SPEC-LX-008.REQ.11 | 🟢 **対応済** |

#### LOW（対応望ましい）

| Finding | 対応内容 | 対応先 | 状態 |
|---------|---------|-------|-----|
| **P-03** TOML 順序保持 | SPEC-LX-002.REQ.08 に「順序保持パーサ必須」と明記 | SPEC-LX-002.REQ.08 | 🟢 **対応済** |
| **P-10** PIPE_ROLE 改ざん耐性 | NFR.SEC.08 として「単独開発者環境前提、役割偽装は脅威モデル外」を明記 | NFR-LX-001.SEC.08 | 🟢 **対応済** |
| **E-06** Markdown 装飾文字正規化 | SPEC-LX-002.REQ.06 に Markdown 装飾文字除去・全角空白・NFC を追加 | SPEC-LX-002.REQ.06 | 🟢 **対応済** |
| **E-07** ONNX 「±20%」削除 | 本 VAL §5 P-06 から数値仮説を削除 | 本 VAL §5 P-06 | 🟢 **対応済** |

### 7.3 対応不要と判断された Finding

| Finding | 理由 |
|---------|------|
| P-01 | 実用規模で衝突確率無視可能、既に SubnodeIdUniqueness で検出される |
| P-08 | 緊急性なし、将来の検討事項として残す（TIME-INV 導入候補） |
| P-09 | 対応済（gate-check.ps1 で実装） |

---

## 8. 結論と G0 ゲート判定

### 8.1 G0 ゲート判定

| 項目 | 状態 |
|------|------|
| 予備レビュー完了 | 🟢 完了 |
| 外部 AI 照合完了 | 🟢 完了（ChatGPT + Gemini、2026-04-17） |
| HIGH Finding への対応完了 | 🟢 **対応済**（P-05, E-01, E-02） |
| MEDIUM Finding への対応完了 | 🟢 **対応済**（P-02, P-04, P-06, P-07, E-03, E-04, E-05） |
| LOW Finding への対応完了 | 🟢 **対応済**（P-03, P-10, E-06, E-07） |
| **G0 ゲート** | 🟢 **通過** |

### 8.2 次のアクション（S1-c）

1. ~~外部 AI 照合実施~~ → **完了**
2. ~~外部 Finding の §6.3 への転記~~ → **完了**
3. ~~§7 の重大指摘統合~~ → **完了**
4. **対応必要な Finding に基づく SPEC/NFR 改訂**（HIGH 3 件 + MEDIUM 7 件、合計 10 件）
5. 本文書の Status を `Approved` に格上げ
6. G0 ゲート通過を宣言

### 8.3 改訂対象と影響範囲

SPEC-TE / NFR-TE の改訂対象（S1-c で実施）:

| ファイル | 対象 Finding | 改訂方針 |
|---------|-------------|---------|
| **SPEC-LX-001** §4 | E-01 | 新規不変条件 `GRAPH-INV-1 未解決エッジ許容性` を追加、14 × 9 マトリクスを 15 × 9 に拡張 |
| **SPEC-LX-002** REQ.07, REQ.08 | P-02, P-03 | DAG 対象エッジ種別、TOML 順序保持パーサの明記 |
| **SPEC-LX-002** or TE-NEXT-EXT-001 §4.6 | E-06 | Markdown 装飾文字正規化の追記 |
| **SPEC-LX-006** REQ.06 | P-04 | Contextual Retrieval のタイムアウト・リトライ・フォールバック |
| **SPEC-LX-008** | E-05 | v0.1.0 → v3 ID 互換性維持（マッピング or 書き換え） |
| **NFR-LX-001** PERF.03 | P-05, E-04 | Windows 環境でのバジェット見直し / NAPI-RS 展望 |
| **NFR-LX-001** SEC.05 | E-02 | クレデンシャルマスキング義務 |
| **NFR-LX-001** SEC | P-10 | PIPE_ROLE 改ざん耐性の前提明記（軽微） |
| **NFR-LX-001** REL | P-07, E-03 | `REL.07 busy_timeout`（上限時間要件）、`REL.08 SQLite WAL 配置条件`（ネットワーク FS 禁止） |
| **NFR-LX-001** §3.1 | E-07 | 既に Step 2 再測定明記済、予備 P-06 の「±20%」削除のみ |
| 本 VAL §5 P-06 | E-07 | 「±20%」記述を削除

---

## 10. Addendum: TE-NEXT-EXT-002 統合（2026-04-17）

本 VAL-LX-001 は v1.0.0 Approved 取得後、**TE-NEXT-EXT-002「Prompt Caching 最適化と MCP Result Persistence」** の追加を受けて、S1-d として以下の対応を実施した。

### 10.1 TE-NEXT-EXT-002 の概要

| 項目 | 内容 |
|------|------|
| ID | TE-NEXT-EXT-002 |
| Version | 0.1.0 |
| 主目的 | (A) `compile_context` 返却の決定論的整列（Prompt Cache ヒット向上）<br>(B) Claude Code v2.1.91+ の MCP Result Persistence 対応（大規模返却の永続化） |
| 新規不変条件 | CACHE-INV-1（バイト単位決定論）、CACHE-INV-2（セクション配置順序）、CACHE-INV-3（大規模返却エラー）、CACHE-INV-4（メタデータ付与忠実性） |

### 10.2 SPEC/NFR への反映（S1-d で実施済）

| ファイル | Version | 主な変更 |
|---------|---------|---------|
| SPEC-LX-001 | 0.3.0 → **0.4.0** | §4.1 マトリクスを 15 → 28 行に拡張（FB/SCORE/STATE/CACHE-INV を追加）、§4.2 カテゴリ別責任を拡充 |
| SPEC-LX-003 | 0.2.0 → **0.3.0** | REQ.10〜14（セクション配置、決定論整列、マーカ、サイズ超過エラー、バイト単位決定論）を新設 |
| SPEC-LX-006 | 0.3.0 → **0.3.1** | §4 に SCORE-INV-1/2 追加（既存 REQ でカバー確認） |
| SPEC-LX-007 | 0.2.0 → **0.3.0** | §4 に FB-INV-1〜5 と STATE-INV-1 追加 |
| SPEC-LX-008 | 0.3.0 → **0.3.1** | §4 に STATE-INV-1/2、FB-INV-4 追加 |
| SPEC-LX-009 | 0.2.1 → **0.3.0** | REQ.13（`_meta` 付与、CACHE-INV-4）、REQ.14（Claude Code バージョン非依存性）を新設 |
| NFR-LX-001 | 0.4.0-provisional → **0.5.0-provisional** | COMPAT.12（Claude Code v2.1.91+ 依存）、PERF.09（500K 文字上限）、REL.09/10（マーカ出力、バイト決定論）を新設 |
| matrix.md | - | 外部参照文書セクションに TE-NEXT-EXT-002 追加 |
| graph.toml | - | ヘッダーコメントに TE-NEXT-EXT-002 の影響を記載 |

### 10.3 外部 AI 再照合の扱い（B 案採用）

本拡張は V-DRS 設計者の判断で追加された性能最適化機能である。
- 既存 G0 通過（VAL-LX-001 v1.0.0 Approved）は維持する
- 外部 AI 再照合は実施せず、V-DRS 設計者の責任下で本拡張を受容する（B 案運用）
- 将来、本拡張の実装段階（Phase 2 以降）で懸念が顕在化した場合、別途 VAL-LX-002 または本 VAL の addendum 更新で対応する

### 10.4 既存漏れ INV の対応（副次作業）

S1-d 実施時に、V-DRS-SPEC-001 §13.2/13.3/13.5 の以下の不変条件が SPEC-LX-001 §4 マトリクスに**未登録**だったことが判明。
- FB-INV-1〜5（フィードバックループの不変条件、§13.2）
- SCORE-INV-1/2（スコア管理の不変条件、§13.3）
- STATE-INV-1/2（状態管理の不変条件、§13.5）

原因: S1-a 予備レビュー時に V-DRS-SPEC-001 §13 を CTX-INV と MCP-INV のみグレップし、FB/SCORE/STATE のカテゴリを見落としていた。

対応: S1-d.1 で 9 件の既存漏れ INV と 4 件の新規 CACHE-INV を合わせて 13 件をマトリクスに追加。既存 REQ が各 INV を実装していることを §4 更新で明記。

### 10.5 未解決として残った検討事項

| # | 項目 | 扱い |
|---|------|------|
| 1 | CTX-INV-5（プロジェクト独自）の V-DRS-SPEC-001 §13 への昇格提案 | 別ワークフロー、V-DRS 設計者判断 |
| 2 | VAL-LX-002（TE-NEXT-EXT-002 向け外部照合）の実施要否 | Phase 2 実装時に再判断 |
| 3 | CTX ブロック向け DD-LX-0xx / TS-LX-0xx の新設 | CTX ブロック着手時 |

---

## 9. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（Claude Code 内部 AI による予備レビュー実施、外部 AI 照合は未実施） |
| 2026-04-17 | 0.2.0-draft | 外部 AI 照合（ChatGPT + Gemini）を実施、7 件の新規 Finding（E-01〜E-07）を §6.3 に記録、§6.4 に総括、§7 を再集約（統合後 HIGH 3/MEDIUM 7/LOW 4/INFO 3）、§8 に S1-c の改訂対象マトリクスを追加 |
| 2026-04-17 | 1.0.0 | S1-c 完了。全 14 件の対応必要 Finding（HIGH 3/MEDIUM 7/LOW 4）への対応を SPEC-LX-001/002/004/006/008 と NFR-LX-001 に適用。§7 の対応状態を全て 🟢 対応済に更新、§8 G0 ゲート判定を 🟢 通過に格上げ、Status を Approved に。§5 P-06 から「±20%」を削除（E-07 反映） |
| 2026-04-17 | 1.1.0 | S1-d 完了。§10 Addendum を追加（TE-NEXT-EXT-002 統合と既存漏れ INV 対応の経緯）。SPEC/NFR への反映結果を記録。外部 AI 再照合は実施せず B 案運用（設計者責任下で受容）。G0 通過状態は維持 |
