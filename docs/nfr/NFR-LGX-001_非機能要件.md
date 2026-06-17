Document ID: NFR-LGX-001

# NFR-LGX-001: legixy 非機能要件

| 項目 | 内容 |
|------|------|
| Document ID | NFR-LGX-001 |
| Version | 0.5.0-provisional |
| Status | Provisional（人間査読済・暫定承認。実測で再評価予定） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | LEGIXY-SPEC-001, LGX-EXT-001 |

---

## 1. 本文書の位置づけ

### 1.1 目的

本文書は legixy が満たすべき**非機能要件（Non-Functional Requirements）**を規定する。機能要件は LEGIXY-SPEC-001 および LGX-EXT-001 に記述される。

### 1.2 対象読者

- 実装者（s2、s2-fix、s2-harden）
- Adversary（s4）: 五次元バイナリ評価の基準として参照
- プロジェクトマネージャ / 人間レビュアー

### 1.3 要件 ID 体系

本文書内の個別要件は `NFR-LGX-001.{カテゴリ}.{連番}` 形式で参照する。
例: `NFR-LGX-001.PERF.01`（パフォーマンス要件 01）

---

## 2. 参照文書

- LEGIXY-SPEC-001 `docs/legixy_foundational_spec.md`
  - 特に I-04（性能は起動時間を重視する、数十ms以内）
- LGX-EXT-001 `docs/legixy_subnode_spec_v0.2.1.md`
  - §9 制約と前提
  - §10 設計判断の確定事項
- CLAUDE.md 四次元収束判定の基準

---

## 3. パフォーマンス要件（PERF）

### 3.1 測定環境の前提

> **重要:** 本章で規定する性能目標値は **Step 1 リリース環境（Windows 11 native）** における値である。
> **Step 2 リリース環境（Ubuntu 24.04 Docker）** での目標値は、Step 2 着手時に実測して本 NFR を改訂する。
> 開発環境と Step 1 リリース環境は同一（Windows 11 native binary）であり、開発時点の実測値がそのまま Step 1 の目標値となる。

**前提環境（以下すべて満たす条件下で目標値を保証する）:**

| 項目 | 値 | 備考 |
|------|----|----|
| OS | Windows 11 Pro（x86_64） | CLAUDE.md で primary 環境 |
| CPU | Intel Core i5-12400F（6 コア 12 スレッド、最大 4.4 GHz） | 現開発機。同世代 / 相当性能以上を想定 |
| 物理メモリ | 32 GB（空きメモリ 4 GB 以上を確保） | - |
| ストレージ | NVMe SSD（シーケンシャル read ≥ 3,000 MB/s 相当） | HDD / eMMC は対象外 |
| Rust toolchain | MSRV 以上（NFR-LGX-001.COMPAT.03 参照） | stable channel |
| ビルドプロファイル | `release`（`lto = "thin"`, `strip = "symbols"`） | Cargo.toml 既定 |
| ロケール | ja-JP / en-US いずれか | - |
| アンチウイルス | プロジェクトディレクトリ除外設定済み | Windows Defender のリアルタイムスキャンは計測に影響 |

**前提が崩れた場合の取り扱い:**
- HDD / eMMC 環境、メモリ 4 GB 未満、macOS 環境での**動作保証は対象外**（COMPAT.09 参照）
- Ubuntu 24.04 LTS は Step 2 で対応予定（COMPAT.02 参照）。Step 2 着手時に Linux 環境での目標値を再測定する
- 著しく遅いマシン（例: 10 年前の CPU）での性能は本 NFR の対象外
- 測定時は前提環境で最低 5 回実行し、中央値で評価する

### 3.2 要件

測定条件は「前提環境 (§3.1) + 各行の追加条件」の組み合わせとする。OS / ストレージ等の共通条件は再掲しない。

| ID | 項目 | 目標値 | 追加測定条件 | 検証方法 |
|----|------|--------|--------------|---------|
| NFR-LGX-001.PERF.01 | CLI コールドスタート時間 | **< 50 ms** 【暫定】 | `--help` 表示までの時間 | `hyperfine legixy --help` |
| NFR-LGX-001.PERF.02 | `check --formal` 実行時間 | **< 500 ms** 【暫定】 | ノード 1,000 + エッジ 2,000 規模の graph.toml | `hyperfine` + テスト用 graph.toml |
| NFR-LGX-001.PERF.03 | `compile_context` 応答時間 | Step 1 (Windows): **< 300 ms** 【暫定、E-04 反映】<br>Step 2 (Linux Docker): **< 200 ms** 【暫定】 | サブノード 100 件含む、最大粒度指定なし。Windows は CLI プロセス起動が重いため緩和 | MCP 経由ベンチマークテスト |
| NFR-LGX-001.PERF.04 | graph.toml パース時間 | **< 100 ms** 【暫定】 | ノード 1,000 規模 | 単体ベンチマーク |
| NFR-LGX-001.PERF.05 | サブノード自動抽出時間 | **< 10 ms/file** | Markdown ファイル 100 行程度 | 単体ベンチマーク |
| NFR-LGX-001.PERF.06 | 常駐メモリ上限（CLI 実行時） | **< 100 MB** 【暫定】 | ノード 1,000 + embedding 込み | プロセスメモリ計測 |
| NFR-LGX-001.PERF.07 | engine.db SQLite 操作 | **WAL モード必須**、fsync は `NORMAL` | 全スキーマ操作 | `PRAGMA` 検証テスト（TS-LGX-001 T-DB-002） |
| NFR-LGX-001.PERF.08 | embedding 生成スループット | **≥ 25 nodes/sec** 【L12 確定後 再評価済・ADR-LGX-022 accepted 2026-06-14】（旧暫定 ≥ 50 は L6 想定。実測 中央値 ≈ 31 nodes/sec @ i5-12400F, 2026-06-14 → PASS） | CPU only、逐次・単一スレッド（DD-007 §7）、多言語モデル `paraphrase-multilingual-MiniLM-L12-v2`（L12=12 層、L6 比 約 2 倍） | ONNX 推論ベンチマーク（`cargo bench -p legixy-embed --features onnx --bench perf08_embed_throughput`） |
| NFR-LGX-001.PERF.09 | compile_context 返却サイズ上限 | **500,000 文字**（固定、CACHE-INV-3）。超過時は切り捨てず明示的エラー返却 | `compile_context`, `get_compile_audit` の返却本文 | サイズ超過シナリオテスト |

**背景:**
- I-02（LEGIXY-SPEC-001 §9）: MCP 経由での頻繁な呼出しがあるため、起動時間を最優先で最適化する
- PERF.01-04 は MCP 呼出しのクリティカルパスに直結するため厳格な目標
- PERF.06 の上限は MCP サーバーと並列実行されることを考慮
- PERF.08 はローカル CPU 推論想定。GPU 利用は非目標

**将来の性能改善展望（本リリースでは非目標）:**
- NAPI-RS 等を用いた **Rust ネイティブアドオン化**（Node.js と同一プロセス実行）により CLI プロセス起動オーバーヘッドを排除可能
- Step 2（Docker）でさらなる最適化を行う場合は、mmap ベースの graph.toml キャッシュも検討候補
- これらは PERF バジェット（§13 再評価対象）で不足が判明した場合の改善オプション

---

## 4. セキュリティ要件（SEC）

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.SEC.01 | ファイル権限 | engine.db、graph.toml は所有者のみ読み書き可（Unix 0600、Windows 相当） | 作成時のパーミッション検証 |
| NFR-LGX-001.SEC.02 | SQLite 排他制御 | 複数プロセスからの同時書き込み時も破損しない（WAL + busy_timeout） | 同時実行ストレステスト |
| NFR-LGX-001.SEC.03 | TOML 入力検証 | 不正な graph.toml でプロセスクラッシュや任意ファイル読み込みを発生させない | ファズテスト（proptest 等） |
| NFR-LGX-001.SEC.04 | Markdown 入力検証 | 巨大ファイル（100 MB 超）や悪意ある見出し（10,000 段ネスト等）で OOM / スタックオーバーフローしない | 限界値テスト |
| NFR-LGX-001.SEC.05 | API キー取扱い | Contextual Retrieval 用 API キーは環境変数経由のみ受領、ログ・DB への記録禁止。エラー情報（スタックトレース、HTTP レスポンス等）に API キーが含まれる可能性がある箇所は**必ずマスキング処理**（例: `***REDACTED***`）を適用する | grep によるキー漏洩検査、observations テーブルのダンプ検査 |
| NFR-LGX-001.SEC.06 | パストラバーサル防止 | graph.toml 内のファイルパスは project_root 配下に限定 | 相対パス `../` を含む入力のテスト |
| NFR-LGX-001.SEC.07 | 依存ライブラリ脆弱性 | `cargo audit` で Critical/High なし | CI 必須チェック |
| NFR-LGX-001.SEC.08 | PIPE_ROLE 識別の前提 | パイプライン役割識別は**単独開発者環境**を前提とする。環境変数 `$env:PIPE_ROLE` の改ざん耐性は要件としない（悪意ある子プロセスによる役割偽装は脅威モデル外） | 脅威モデル文書化 |

---

## 5. 互換性要件（COMPAT）

| ID | 項目 | 要件 | 備考 |
|----|------|------|------|
| NFR-LGX-001.COMPAT.01 | Step 1 OS 対応（必須） | **Windows 11 x86_64** native バイナリ（`legixy.exe`） | 第 1 リリース目標、現開発環境と同一 |
| NFR-LGX-001.COMPAT.02 | Step 2 OS 対応（必須、着手時） | **Ubuntu 24.04 LTS x86_64** Docker image（glibc 2.39） | 第 2 リリース目標、配布は Docker のみ |
| NFR-LGX-001.COMPAT.03 | Rust MSRV | **1.75.0** 【暫定】 | Cargo.toml の `rust-version` に明記 |
| NFR-LGX-001.COMPAT.04 | v0.1.0 データ移行 | v0.1.0 の engine.db とマトリクス形式の .legixy.toml から自動移行可能 | UC-LGX-009 に対応 |
| NFR-LGX-001.COMPAT.05 | v0.1.0 マトリクスビュー | matrix.md は graph.toml から自動生成される読み取り専用ビュー | LGX-EXT-001 §4.4 |
| NFR-LGX-001.COMPAT.06 | MCP プロトコル | MCP-INV-1（3ツールのみ: compile_context, observe, get_compile_audit）を維持 | LGX-EXT-001 §6.1 |
| NFR-LGX-001.COMPAT.07 | 文字エンコーディング | ファイル IO は UTF-8 固定、Windows の BOM 付き UTF-8 も受容 | - |
| NFR-LGX-001.COMPAT.08 | 改行コード | LF / CRLF の両方を受容。出力は LF 統一 | - |
| NFR-LGX-001.COMPAT.09 | macOS 対応 | **現状非対応**（Step 1/Step 2 のいずれでもサポートしない）。将来対応は未定 | 開発者の macOS 利用も想定外 |
| NFR-LGX-001.COMPAT.10 | Node.js バージョン（MCP サーバ） | **LTS 固定**。アクティブ LTS と維持 LTS の 2 世代をサポート | MCP サーバは OS 非依存（同一コードで Windows / Linux 動作）、Node.js LTS 依存のみ |
| NFR-LGX-001.COMPAT.11 | 配布形態 | Step 1: Windows native バイナリ配布（`.exe`）／Step 2: Docker image（Ubuntu 24.04 LTS base, x86_64） | Step 2 では MCP サーバ+Rust CLI+Node.js を含む一体化 image |
| NFR-LGX-001.COMPAT.12 | Claude Code バージョン依存 | MCP Result Persistence 機能（`_meta["anthropic/maxResultSizeChars"]`）は **Claude Code v2.1.91 以降**で解釈される。それ以前のバージョンでは無視され動作には影響しない（永続化恩恵のみ得られない） | LGX-EXT-002 §4.5 |

---

## 6. 可観測性要件（OBS）

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.OBS.01 | ログレベル | `RUST_LOG` 環境変数で制御、標準は `info` | `tracing` + `tracing-subscriber` 使用 |
| NFR-LGX-001.OBS.02 | 出力先 | ログは stderr、結果は stdout（パイプ可能） | 形式テスト |
| NFR-LGX-001.OBS.03 | 構造化ログ | `--log-format=json` で JSON Lines 出力 | E2E テスト |
| NFR-LGX-001.OBS.04 | エラーメッセージ | 日本語（primary）。ユーザが修正可能な内容を示唆する文言 | メッセージレビュー |
| NFR-LGX-001.OBS.05 | エラーコード | 終了コード: 0=OK, 1=Error, 2=使用法誤り | E2E テスト |
| NFR-LGX-001.OBS.06 | CheckResult の severity | Ok / Info / Warning / Error の4段階（DD-LGX-001 §2.4） | TS-LGX-001 |

---

## 7. 信頼性要件（REL）

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.REL.01 | engine.db 破損耐性 | WAL モード + 適切な PRAGMA 設定で電源断耐性 | 破損シミュレーションテスト |
| NFR-LGX-001.REL.02 | 部分失敗時の挙動 | `check` 中に1ファイルの読み込みに失敗しても、他のチェックは継続する（Error として記録） | TS-LGX-001 拡張 |
| NFR-LGX-001.REL.03 | 冪等性 | `check --formal` を同一入力で複数回実行しても同一結果 | 反復実行テスト |
| NFR-LGX-001.REL.04 | サブノード ID 決定性（SUBNODE-INV-5） | 同一 `parent_id + heading_path` から常に同一 ID | TS-LGX-001 T-IG-001 |
| NFR-LGX-001.REL.05 | BFS 走査決定性（CTX-INV-1） | 同一グラフ・同一起点から常に同一 visited 順 | TS-LGX-001 T-GT-005 |
| NFR-LGX-001.REL.06 | トランザクション境界 | embedding 生成・格納は単一トランザクション内で完結 | 中断テスト |
| NFR-LGX-001.REL.07 | SQLite busy_timeout 上限 | 並行呼出し時のロック待機は**上限時間を設定**し、超過時は失敗として返す。暫定値 **5000 ms**（人間査読時に実測して調整）。無限リトライは禁止 | 並行書き込みストレステスト、上限超過時の Error 報告確認 |
| NFR-LGX-001.REL.08 | engine.db 配置条件 | engine.db は**ローカルファイルシステム上**に配置する。ネットワーク共有（SMB/NFS/CIFS 等)上の配置を禁止（SQLite WAL が共有メモリに依存するため）。起動時検出で Warning 出力 | Step 2 Docker 環境での配置確認テスト |
| NFR-LGX-001.REL.09 | キャッシュブレーク点マーカ出力保証 | `compile_context` 返却には `<!-- cache-breakpoint: stable-end -->` マーカが 1 箇所含まれる（CACHE-INV-1/2 連携）。マーカは Additional Guidelines 末尾と Upstream Artifacts 先頭の間に配置 | マーカ存在の単体テスト |
| NFR-LGX-001.REL.10 | 返却バイト列の決定論性 | 同一入力（graph 定義、engine.db 状態、引数）に対して `compile_context` の返却バイト列が完全一致（CACHE-INV-1 実装保証）| 同一入力に対する複数回呼出しのバイナリ比較テスト |

---

## 8. 保守性要件（MAINT）

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.MAINT.01 | テストカバレッジ | 行カバレッジ **≥ 80%** 【暫定】 | `cargo llvm-cov` |
| NFR-LGX-001.MAINT.02 | Clippy 警告 | `cargo clippy -- -D warnings` で 0 警告 | CI 必須 |
| NFR-LGX-001.MAINT.03 | ドキュメントコメント | 公開 API には rustdoc コメント必須 | `cargo doc` ビルド検証 |
| NFR-LGX-001.MAINT.04 | 依存ライブラリ数 | 直接依存 **≤ 30 crates** 【暫定】 | `cargo tree -e=normal --depth 1` |
| NFR-LGX-001.MAINT.05 | テストコード不可侵 | 実装修正時にテストコード・TS を変更しない（CLAUDE.md 絶対ルール1） | パイプラインフックで強制 |
| NFR-LGX-001.MAINT.06 | トレーサビリティ | 全 SRC ファイルに Document ID を含める | `check --formal` |

---

## 9. 使用性要件（USE）

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.USE.01 | CLI ヘルプ | `legixy <subcommand> --help` で全オプション表示 | 目視 |
| NFR-LGX-001.USE.02 | エラー出力の実用性 | Error 発生時、該当ファイルとリカバリ方法を示唆 | メッセージレビュー |
| NFR-LGX-001.USE.03 | プログレス表示 | 長時間処理（embed --all 等）で進捗表示 | 目視 |
| NFR-LGX-001.USE.04 | 終了コード一貫性 | Error 1件以上 → 終了コード 1 | E2E テスト |

---

## 10. 形式的ハードニング要件（HARDEN）

※ CLAUDE.md 絶対ルール 8（四次元収束）に対応。

| ID | 項目 | 要件 | 検証方法 |
|----|------|------|---------|
| NFR-LGX-001.HARDEN.01 | プロパティベーステスト | サブノード ID 生成、グラフ走査、DAG 検証の主要関数に `proptest` 適用 | `cargo test --features proptest` |
| NFR-LGX-001.HARDEN.02 | ミューテーションテスト | s2-harden フェーズで `cargo mutants` を実行、生存ミュータント率 **< 10%** 【暫定】 | `cargo mutants` |
| NFR-LGX-001.HARDEN.03 | ファズテスト | graph.toml パーサと Markdown 見出し抽出に `cargo fuzz` 適用、1 時間無クラッシュ | `cargo fuzz run` |

---

## 11. 非目標（Out of Scope）

以下は本バージョンでは**対象外**とする:

- 国際化: エラーメッセージの英語版は将来対応
- リモート DB 対応: engine.db は必ずローカルファイル
- マルチユーザ同時編集: 単一開発者の作業想定
- GPU 推論: ONNX は CPU 実行のみ
- GUI: CLI のみ
- ネットワーク通信: Contextual Retrieval API 以外、外部通信なし

---

## 12. 四次元収束判定との対応

CLAUDE.md で定義される四次元収束判定の各次元と NFR の対応:

| 次元 | 主な NFR |
|------|---------|
| 機能収束 | SEC.03-06（入力検証）、REL.02-06 |
| 非機能収束 | PERF.*、SEC.*、COMPAT.*、OBS.*、REL.01 |
| 品質収束 | MAINT.*、HARDEN.* |
| トレーサビリティ収束 | MAINT.05, MAINT.06 |

---

## 13. 暫定合意事項と再評価トリガ

以下の **【暫定】** マーカー付き項目は、人間査読により暫定承認済み。
**再評価条件:** Phase 4（実装・テスト）で実測値が目標と著しく乖離した場合（達成不能 / 過度に余裕あり）、本 NFR に差し戻して改訂する。

| # | 項目 | 暫定値 | 主な再評価判断基準 |
|---|------|--------|------------------|
| 1 | PERF.01 コールドスタート | < 50 ms | 実測で安定達成不能 or 大幅余裕があれば調整 |
| 2 | PERF.02 `check --formal` | < 500 ms | 同上 |
| 3 | PERF.03 `compile_context` | < 200 ms | 同上 |
| 4 | PERF.04 graph.toml パース | < 100 ms | 同上 |
| 5 | PERF.06 メモリ上限 | < 100 MB | 同上 |
| 6 | PERF.08 embedding スループット | ~~≥ 50 nodes/sec~~ → **≥ 25 nodes/sec（確定）** | ✅ 2026-06-14 L12 確定後に再評価実施・**人間 ratification 済**（実測 ≈ 31 nodes/sec @ i5-12400F = PASS、ADR-LGX-022 accepted）。以後の更なる緩和は新たな実測 + ADR を要する |
| 7 | COMPAT.03 Rust MSRV | 1.75.0 | 依存クレートが要求する MSRV 次第で確定 |
| 8 | MAINT.01 カバレッジ | ≥ 80% | 実運用で過不足が判明したら調整 |
| 9 | MAINT.04 依存数 | ≤ 30 crates | 実装完了時点で確定値に |
| 10 | HARDEN.02 生存ミュータント率 | < 10% | s2-harden で実行コストと品質のバランスを評価 |
| 11 | 派生文書の扱い | 保留 | legixy リリース近辺で本番/ユーザ環境向け NFR の要否を再判断 |

**再評価ワークフロー:**
- Phase 4 の s3（テスト実行）または s2-harden（形式的ハードニング）が実測値を記録
- 目標値との乖離が明確な場合、**ループ3（人間判断）** として本 NFR に差し戻す
- 改訂時は新 Version（0.2.x 以降）を発行し、変更理由を §14 に記載

**PERF バジェット整合性（VAL-LGX-001 Finding P-05、E-04）:**
本 NFR §3.2 の PERF.01/PERF.03/PERF.04 の合計バジェットは Step 1（Windows）では以下のリスクを持つ:
- プロセス起動 50 ms + graph パース 100 ms + 実処理 ≦ PERF.03 (< 300 ms)
- CLI 子プロセス起動オーバーヘッドは Windows で 10-30 ms 存在（Gemini 指摘 E-04）
- Step 1 で暫定値 300 ms に緩和済（PERF.03）
- 実測で更なる乖離があれば、Rust 内最適化（mmap）または NAPI-RS 化（同一プロセス実行）を検討する

**Phase 2 以降の改善候補（LGX-EXT-002 §6.1 Phase 2 項目）:**
- `maxResultSizeChars` の動的調整（現在は 500,000 固定）
- キャッシュヒット率の計測機構導入
- キャッシュブレーク点マーカーの複数箇所配置
- investigate/impact 返却への同様の整列適用

---

## 14. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | §3.1 測定環境の前提を追加。性能要件が現開発環境に依拠することを明記。§13 に派生文書の検討事項を追加（人間査読指摘により） |
| 2026-04-17 | 0.1.2-draft | §3.1 の CPU（i5-12400F）・物理メモリ（32 GB）を人間査読で確定。§13 から対応項目を削除 |
| 2026-04-17 | 0.2.0-provisional | 人間査読により残 8 項目を暫定承認。【要決定】マーカーを【暫定】に変更。§13 を「暫定合意事項と再評価トリガ」として再構成し、Phase 4 実測時の再評価ワークフローを明記 |
| 2026-04-17 | 0.3.0-provisional | リリース戦略を Step 1（Windows 11 native）/ Step 2（Ubuntu 24.04 Docker）に明記。§3.1 を Step 1/2 両対応の表現に更新。§5 COMPAT.01/02 をリライト、COMPAT.09（macOS 非対応）、COMPAT.10（Node.js LTS 固定）、COMPAT.11（配布形態）を新設 |
| 2026-04-17 | 0.4.0-provisional | S1-c 対応: SEC.05 にクレデンシャルマスキング義務追加（E-02）、SEC.08 PIPE_ROLE 識別の前提（P-10）を新設、PERF.03 を Step 1 Windows で 300 ms に緩和（P-05/E-04）、PERF 末尾に NAPI-RS 展望を追加、REL.07 busy_timeout 上限（P-07）・REL.08 engine.db 配置条件（E-03）を新設、§13 に PERF バジェット整合性の補足追記 |
| 2026-04-17 | 0.5.0-provisional | S1-d 対応: LGX-EXT-002 統合で COMPAT.12（Claude Code v2.1.91+ 依存、CACHE-INV-4 関連）、PERF.09（500,000 文字上限、CACHE-INV-3）、REL.09（キャッシュブレーク点マーカ出力保証）、REL.10（返却バイト列決定論、CACHE-INV-1）を新設。§13 に Phase 2 以降の改善候補を追記 |
| 2026-06-14 | 0.6.0-provisional | Phase 4 実測フェーズの NFR 検証着手。**PERF.08 実測 = 中央値 ≈ 31 nodes/sec @ i5-12400F**（criterion、`--features onnx`、L12 確定後）→ §13 #6 再評価トリガ発火。暫定閾値を ≥50（L6 想定）→ **≥25 nodes/sec** へ再評価（ADR-LGX-022 proposed）。**REL.07 実測**: EmbeddingStore に `busy_timeout` PRAGMA を配線し、上限超過時 `SQLITE_BUSY` を Error として返す（無限リトライ無し）並行ストレステストを追加（`crates/legixy-embed/tests/at_rel07_concurrency.rs`）。AT-LGX-001（E2E/ONNX/PERF/REL 受け入れ基準）を新設。ONNX 同一環境決定性を AT で実証（同一入力→バイト一致、ADR-LGX-003 の順序/入力/モデル決定性の範囲内・環境間ビット再現は対象外を再確認） |
