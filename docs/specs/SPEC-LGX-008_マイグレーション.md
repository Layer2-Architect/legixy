Document ID: SPEC-LGX-008

# SPEC-LGX-008: マイグレーション

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-008 |
| Version | 0.7.1 |
| Status | Approved（人間査読済） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §4.4, §8.2 |
| 対応 NFR | NFR-LGX-001.COMPAT.04, COMPAT.05, REL.01 |
| 対応 UC | UC-LGX-009 |

---

## 1. 本文書の位置づけ

### 1.1 目的

v0.1.0 から legixy へのデータマイグレーションおよび新規プロジェクト初期化の要求を定義する。

### 1.2 スコープ

**含む:** engine.db のマイグレーション、.legixy.toml / matrix.md 形式の移行、設定ファイルの旧名（`.trace-engine.toml`）フォールバック探索、init コマンド
**含まない:** legixy 以降のマイグレーション（将来版）

---

## 2. 参照文書

- LGX-EXT-001 §4.4 後方互換性
- LGX-EXT-001 §8.2 Phase 1 マイグレーション方針

---

## 3. 要求事項

### SPEC-LGX-008.REQ.01: engine.db のマイグレーション方式

**内容:** v0.1.0 フォーマットのプロジェクト（feedback データを `.trace-engine/feedback.db` に保持し、matrix 形式設定を伴う）を検出した場合、legixy は feedback.db の observations / proposals / custom_edges を正準 `engine.db`（`.legixy/engine.db`、ADR-LGX-015）へ統合する形で、以下のいずれかの方式で legixy スキーマに変換する:

- **明示実行（デフォルト・推奨）:** ユーザが `legixy migrate` を実行したときのみ変換する（REQ.06）
- **opt-in 自動実行:** `.legixy.toml` に `[migration] auto = true` を設定した場合のみ、legixy の任意コマンド初回実行時に自動変換する

**デフォルト挙動（auto 未設定時）:** `check` 等の読み取り系コマンドが v0.1.0 プロジェクト（feedback.db + matrix 設定）を検出したら、**Error を返して明示的な `migrate` 実行を促す**（読み取り系コマンドが意図せず DB を書き換える副作用を避ける）。

**変換内容:**
- v0.1.0 の既存テーブル構造を検出
- legixy で追加されたカラム（例: `context_log.granularity`）を追加
- 既存データは保持

**根拠:** LGX-EXT-001 §4.3 末尾「engine.db のマイグレーションは…自動実行する」を「opt-in 自動」と解釈。ユーザの予期せぬ書き換えを防ぐ必要性（NFR-LGX-001.REL.01 非破壊性）。
**検証方法:** v0.1.0 engine.db を入力にしたマイグレーションテスト、auto 未設定時に check が Error を返すテスト

### SPEC-LGX-008.REQ.02: 非破壊性

**内容:** マイグレーションは以下を保証する:
- 失敗時は元ファイルを保持（退避命名は REQ.02a）
- 途中中断されても DB が壊れない（SQLite トランザクション使用）
- 再実行可能（冪等）
- **非 DB ファイルの atomic 書込（GAP-LGX-152 対応）**: graph.toml / migration-id-map.toml 等の平文生成物は **一時ファイル（`.tmp.{unix epoch 秒}`）へ全量書き出し → fsync → rename(2)** で確定する（直接上書き禁止。SPEC-LGX-002.REQ.13 / GAP-LGX-023 の方式と統一）
- **確定順序**: engine.db のトランザクションコミットを**先行**させ、その後に graph.toml / id-map を atomic に確定する（中断時に「DB のみ新・平文のみ新」の不整合中間状態を残さない。DB コミット後・平文確定前の中断は再実行（冪等）で回復する）
- **再開戦略（GAP-LGX-148 対応）**: **resume なし・全やり直し方式**を正準とする。中間状態の進捗記録は持たず（STATE-INV-1 と整合）、各段階（DB 変換・graph.toml 生成・id-map 生成・config 移行）の冪等性により再実行が安全に収束する。上記確定順序により中断後の中間状態は「DB のみ新」の 1 形態に限定され、再実行で解消される
- **並行アクセスと排他（GAP-LGX-150/151 対応、ADR-LGX-011）**: migrate 中の他コマンドの読取は、engine.db については **SQLite WAL の読取一貫性**に、graph.toml については **atomic rename**（読み手は常に完全な旧版か新版のみを観測）に委ねる。**専用ロックファイル等の明示排他は設けない**。二重 migrate / `auto = true` の複数コマンド同時初回起動については、SQLite の書込ロックが事実上の排他となり、敗者は busy_timeout（REL.07）超過で **Error exit 1**（メッセージで並行実行の可能性を示唆）。単独開発者前提（NFR SEC.08）下のリスク受容として ADR-LGX-011 に記録

**根拠:** NFR-LGX-001.REL.01、GAP-LGX-148/150/151/152、ADR-LGX-011
**検証方法:** 中断シミュレーションテスト（rename 前中断で元 graph.toml 無傷、DB コミット後中断で再実行回復を含む）、中断後再実行の収束テスト

### SPEC-LGX-008.REQ.02a: 退避ファイルの命名規約（GAP-LGX-153 対応）

**内容:** マイグレーション・init が既存ファイルを退避する際の命名を以下に統一する:
- 退避名は `<元ファイル名>.bak.{unix epoch 秒}` とする（**固定名 `.bak` への上書き退避は禁止** — 複数回実行で原本世代が失われるため）
- 同一秒内の再実行で退避名が衝突する場合は連番サフィックスを付与し、**既存の退避ファイルを上書きしない**
- 退避ファイルは累積し、legixy が機械的に削除することはない（削除はユーザ判断）
- 本命名は refresh-subnodes の `graph.toml.refresh-bak.{epoch}`（SPEC-LGX-002.REQ.13）の慣行と整合し、`init --force` の退避（LGX-COMPAT-001 §4 #1）にも適用する

**根拠:** GAP-LGX-153、NFR-LGX-001.REL.01（非破壊性 — 退避自体の上書きも破壊である）
**検証方法:** 同一プロジェクトで migrate を 2 回実行し、退避ファイルが 2 世代とも残ることのテスト

### SPEC-LGX-008.REQ.03: matrix → graph.toml の生成

**内容:** v0.1.0 の matrix 形式 `.legixy.toml` と `matrix.md` から、以下のルールで `graph.toml` を自動生成する:
- matrix.md の各成果物 ID をノードとして抽出
- `[id.chain]`（単数形）または `[id.chains]` + `[id.areas]`（複数形・multi-area 変種、ADR-LGX-018 #15）の順序定義に基づき chain エッジを生成。両表記を受理する
- custom エッジは（v0.1.0 にあれば）`custom_edges` テーブルから継承
- サブノード情報は含めない（§8.2 準拠）

**抽出規則と不正入力（GAP-LGX-158 対応）:**
- ID 抽出は v0.1.0 の設定スキーマに従う: `[matrix]` の section 設定が対象節を、`[id]` の pattern が ID 形式を定める（具体構文の細目は DD）
- matrix.md が想定構造でない（節欠落・表構造崩れ）ために抽出が 0 件となった場合は**空入力として正常終了**（下記「空入力」と同経路、Info で件数を可視化。v0.1.0 が許容していた構造の幅を migrate 側で狭めない）
- **`[id.chain]` / `[id.chains]` のいずれも存在しない、または存在する側の `order` が欠落・不正な場合は破損（REQ.03a）として Error** — chain エッジを暗黙に 0 本として続行しない（構造情報の黙殺禁止）

**空入力（GAP-LGX-141 対応）:** 成果物 0 件の v0.1.0 入力（空 engine.db・ノードを含まない matrix・空設定）に対しては、**空の graph.toml（ノード 0・エッジ 0）を生成して正常終了（exit 0）**し、stderr に Info（移行対象 0 件）を出力する。「対象が無い」（パース成功 + 抽出 0 件）と「対象が壊れている」（REQ.03a）は明確に区別する。

**性能予算（GAP-LGX-142 対応):** migrate は**初回限りの一時的操作であり NFR の PERF 予算対象外**とする（NFR 側への明記は NFR 改訂イベントとして別途提起）。入力サイズ上限は設けず、全ロード方式を許容する（悪意入力による OOM 防止は NFR SEC.04 が別途適用される）。

**根拠:** LGX-EXT-001 §4.4, §8.2、GAP-LGX-141/142/158
**検証方法:** 既存 v0.1.0 プロジェクトでの変換テスト、空入力 fixture（空 graph.toml + exit 0 + Info）テスト、[id.chain] 欠落 fixture（Error）テスト

### SPEC-LGX-008.REQ.03a: ソースデータ破損の検出（GAP-LGX-144 対応）

**内容:** 移行元データの破損を検出した場合、migrate は **Error（exit 1）で中断し、原本を温存し、部分移行を行わない**（REQ.02 非破壊性と整合）。「破損を黙って引き継がない」ことを保証する:
- **検出対象と方法**: engine.db = open/クエリ失敗・必須テーブルの欠落、matrix 形式 `.legixy.toml` / `.trace-engine.toml` = TOML パース失敗・必須構造（`[id.chain]` または `[id.chains]` の order 等）の欠落・不正
- **区別**: 「対象が無い」（パース成功 + 抽出 0 件 = REQ.03 の空入力、正常終了）と「対象が壊れている」（本 REQ、Error）を診断メッセージで明確に区別する
- **出力の妥当性保証**: 生成した graph.toml は**確定（atomic rename）前に**パース可能性と ID 一意性（REQ.11 の全単射検証と共通機構）を検証し、壊れた入力から壊れた出力を生成しない（CTX-INV-2 保全）

**根拠:** GAP-LGX-144（旧 §2.11 観点 D-4 統合）、NFR-LGX-001.REL.01
**検証方法:** 破損 fixture（壊れた engine.db / 不正 TOML / order 欠落）での Error + 原本無傷テスト

### SPEC-LGX-008.REQ.04: .legixy.toml の移行

**内容:** `.legixy.toml` に `[graph]` セクションを追加する。`[matrix]` セクションは後方互換のため残す（graph.toml から matrix.md を生成する設定としての意味に変更）。
**根拠:** LGX-EXT-001 §4.4
**検証方法:** 設定ファイル比較テスト

### SPEC-LGX-008.REQ.05: Phase 1 はドキュメントノードのみ

**内容:** マイグレーション時点ではサブノード情報は生成しない（§8.2）。ユーザが明示的に `embed --all` 等を実行するか、`check` がサブノード抽出をトリガしない限り、graph.toml にはドキュメントノードのみが記録される。
**根拠:** LGX-EXT-001 §8.2
**検証方法:** マイグレーション直後の graph.toml 検査

### SPEC-LGX-008.REQ.06: migrate コマンド

**内容:** `legixy migrate` は明示的なマイグレーション実行コマンドとして提供する。引数は凍結契約（LGX-COMPAT-001 §4 #2）どおり**パス意味**で確定する（GAP-LGX-157 人間裁定 2026-06-10・案A）:
- `--from <PATH>`（**必須**）: 移行元 v0.1.0 プロジェクトのルートパス
- `[--to <PATH>]`: 出力先プロジェクトのルートパス（既定: `--project-root` と同じ）
- `[--dry-run]`: 変更内容を表示するが書き込まない
- `[--format markdown|json]`: レポート出力形式（既定 markdown）
- **バージョンは引数で指定しない** — 移行元・移行先のバージョンは REQ.09 のバージョン自動検出で決定する
- 既に legixy の場合は no-op

> **裁定記録（GAP-LGX-157、BLOCKER 解消）:** 旧版の本 REQ にあった `--from v0.1.0 --to legixy`（バージョン文字列）は、凍結契約 LGX-COMPAT-001 §4 #2（`--from <PATH>` 必須・`--to` 既定 `--project-root`）および v3 実装事実（`crates/te-cli/src/main.rs:66-73` で `from: PathBuf`）と矛盾していた。人間裁定により**凍結 PATH 意味を正準**とし、バージョン意図は REQ.09 自動検出へ移管（凍結契約は無変更、ハードルール 7 維持）。整合判断は ADR に記録。

**根拠:** ユーザの明示的制御、LGX-COMPAT-001 §4 #2（凍結）、GAP-LGX-157
**検証方法:** CLI E2E テスト（--from 省略時 exit 2・--to 既定値・--dry-run 非書込を含む）

### SPEC-LGX-008.REQ.07: init コマンド（新規プロジェクト）

**内容:** `legixy init` は新規プロジェクトを legixy 形式で初期化する。
- `.legixy.toml` のテンプレートを生成（**既定は DD-LGX-007 §3.1.1 に従い、ICONIX 標準 8 typecode（SPEC/UC/RB/SEQ/DD/TS/TC/SRC）の `[id.types.*]` セクションと `[id.document_id]` セクションを含む完全 template** を出力する。ICONIX はあくまで既定選択であり、エンジン本体はプロセス非依存）
- `docs/traceability/graph.toml` の空ファイルを生成
- `.legixy/engine.db` を初期スキーマで作成
- **ICONIX 成果物用 8 ディレクトリを既定で作成**（`docs/specs/`, `docs/usecases/`, `docs/robustness/`, `docs/sequence/`, `docs/detailed-design/`, `docs/test-specs/`, `tests/`, `src/`）。各ディレクトリに `.gitkeep` を配置し Git 追跡可能にする
- 既存ファイルがある場合はエラー（`--force` で上書き）

**「既存ファイル」の判定対象と --force の破壊範囲（GAP-LGX-143 対応）:**
- 「既存」判定の対象は **legixy 管理生成物のみ**とする: `.legixy.toml` / `.trace-engine.toml`、`docs/traceability/graph.toml`、`.legixy/engine.db`
- ICONIX 8 ディレクトリやユーザドキュメントの存在は判定対象**外**（既存リポジトリへの後付け init を妨げない）
- `--force` の上書き対象も **legixy 生成物のみ**。上書き前に REQ.02a の命名で退避する。`.gitkeep` は不足分のみ補完し、**既存ユーザファイルには一切触れない**
- `init [--force]` のシグネチャは不変（LGX-COMPAT-001 §4 #1 維持）

**プロセス非依存性（2026-04-20 明記）:** エンジン本体（legixy-core/legixy-check/legixy-ctx/legixy-nav/legixy-embed/legixy-feedback）は typecode 体系を設定ファイル駆動で処理し、ICONIX 固有の列挙型やハードコードは持たない。非 ICONIX プロセス（Waterfall / Agile / BDD / RDD 等）を採用するプロジェクトは、init 後に `.legixy.toml` の `[id.chain]` と `[id.types.*]` を書き換えるだけで利用できる。init の ICONIX 既定は「旧版からの移行ユーザー + ICONIX 採用プロジェクトに対する再学習コストを最小化」するための選択であり、エンジン本体はプロセス非依存である。

**DevProc 二段化との関係（前段ループ反復 1 で確定）:** ICONIX 二段化（RBA/RBD・SEQA/SEQD の抽象/具体分離、`robustness-abstract`/`robustness-detail` 等のディレクトリ）は、本リポジトリが DevProc_V4.1 運用として init 後に `.trace-engine.toml`（開発側設定）を**上書きして採用している別レイヤ**であり、legixy-the-tool の init 既定生成物ではない。init は単段 ICONIX 8 ディレクトリと chain `UC→RB→SEQ→DD→TS→TC→SRC` を既定として維持する（TS-LGX-007〔init 直後 check〕の期待値と既存利用者体験の安定。v3 実測 `crates/te-mig/src/initializer.rs:56-68` の正準化）。二段化テンプレートの同梱は将来要求とし本版のスコープ外とする。

**根拠:** UC-LGX-009、`/sc:analyze` Finding 1（2026-04-20、ICONIX+SDD+TDD 新規プロジェクトで init 直後に check --formal が機能しなかった問題）の解消、QSET-LGX-008 Q1 回答（2026-06-07、選択肢 A）
**検証方法:** 空ディレクトリでの init テスト（TS-LGX-007 §Init.*）。init 直後に `check --formal` が 0 ERROR、ICONIX 8 ディレクトリ + `.gitkeep` が存在、`.legixy.toml` が ICONIX 8 typecode + `[id.document_id]` を含むことを確認

### SPEC-LGX-008.REQ.08: マイグレーション失敗時のメッセージ

**内容:** マイグレーション失敗時はユーザに以下を提示する:
- 失敗した段階（engine.db, graph.toml, 等）
- 元ファイルのバックアップ場所
- リカバリ手順

**成功時の変更サマリ（GAP-LGX-160 対応）:** 通常実行（dry-run でない）の成功時、**stdout に変更サマリ**を出力する: 生成/更新したファイルの一覧・書き換えた ID の件数（および一覧または id-map への参照）・バックアップの場所。これは STATE-INV-2 の「ユーザが内容を確認して Git commit するまでが完了」という運用を支える事後条件である。`--format json` 時は同内容を構造化出力する（スキーマは DD）。診断・進捗は stderr（NFR OBS.02）。

**根拠:** NFR-LGX-001.USE.02、GAP-LGX-160
**検証方法:** エラーシナリオテスト、成功時サマリの出力内容検証テスト

### SPEC-LGX-008.REQ.09: バージョン検出（GAP-LGX-154 対応、旧 147/149 統合）

**内容:** プロジェクトのバージョン（v0.1.0 / legixy）は以下の判定基準で自動検出する。migrate の移行元/移行先バージョン決定（REQ.06）にも本検出を用いる:
- **engine.db**: `PRAGMA user_version` を**一次根拠（権威ソース）**とする（v3 実測: `crates/te-mig/src/initializer.rs:137` で user_version=3 を設定、`crates/te-cli/src/autodetect.rs:99` で判定に実使用）。user_version が 0 の場合は legixy 追加カラムの有無で二次判定する
- **`.legixy.toml` / `.trace-engine.toml`**: `[graph]` セクションの有無で判定する
- **マーカ欠落**: いずれのバージョンマーカも持たないプロジェクトは v0.1.0 とみなす（最も保守的な解釈）
- **矛盾特徴**: 検出結果が矛盾する場合（例: engine.db は legixy だが .legixy.toml は v0.1.0 形式）は Error を返す

**根拠:** 整合性保証、GAP-LGX-154（旧 GAP-147/149 の判定基盤を統合）
**検証方法:** 不整合シナリオテスト、user_version 0/3・[graph] 有無・マーカ欠落の各組合せの判定テスト

### SPEC-LGX-008.REQ.10: 将来のマイグレーション拡張性

**内容:** マイグレーション実装は「バージョン間変換ステップの列」として構造化する。将来の新バージョン追加時に v0.1.0 → legixy → 新バージョン の段階的適用が可能であること。
**根拠:** 保守性
**検証方法:** 実装レビュー

### SPEC-LGX-008.REQ.11: v0.1.0 → legixy の ID 互換性維持

**内容:** v0.1.0 の既存 graph.toml（存在する場合）に手動記述された ID 参照を、legixy 移行時に保全する:
- **マッピングテーブルの自動生成**: 旧 ID と新 ID（legixy で自動生成された SHA-256 ベース ID）の対応表を `.legixy/migration-id-map.toml` として生成する
- **参照の自動書き換え**: 既存 graph.toml 内のエッジ `from`/`to` や `parent` フィールドに登場する旧 ID を、マッピングに従い新 ID に書き換える
- **Dry-run 対応**: `legixy migrate --dry-run` で書き換え対象を事前確認できる
- **マッピング不可 ID の処理（GAP-LGX-146 対応、旧「非互換警告」を精密化）**: マッピング不可の ID を検出した場合の既定挙動は **abort**（graph.toml / id-map とも不変のまま exit 1 — 非破壊性優先、CTX-INV-2 保全）。継続は opt-in とし、継続時は旧 ID を**残置せず当該エッジを除外**する（dangling 参照の防止）。継続フラグの名称は DD で定義する（GAP-LGX-157 裁定済みの引数体系と整合させる）
- **id-map の全単射保証（GAP-LGX-159 対応）**: migration-id-map は旧 ID → 新 ID の**全単射**を保証する。①旧 ID 側の重複（同一旧 ID に複数新 ID = 曖昧性）→ Error、②新 ID 側の衝突（複数旧 ID が同一新 ID = 多対一）→ Error、③書き換え後 graph 全体での新 ID 一意性（SUBNODE-INV-3）違反 → Error。これらの検証は `--dry-run` でも実施する

本プロジェクト自身は v0.1.0 で graph.toml を使用していないため影響は小さいが、将来の legixy ユーザが v0.1.0 から移行するシナリオで重要。
**根拠:** VAL-LGX-001 Finding E-05、REQ.02（非破壊性）
**検証方法:** 旧 ID を含む v0.1.0 graph.toml を入力にしたマイグレーション + マッピングテーブル生成確認テスト

### SPEC-LGX-008.REQ.12: engine.db の配置条件（Step 2 Docker）

**内容:** Step 2 の Docker 配布（NFR-LGX-001.COMPAT.11）における engine.db の配置は以下を満たす:
- **Docker ローカルボリューム** または **コンテナ内ローカルファイルシステム** 上に配置する
- **ネットワーク共有（SMB / NFS / CIFS / ネットワーク越し bind mount 等）上の配置を禁止** する
- bind mount を使用する場合、ホストがローカルファイルシステム（ext4、NTFS 等）であることを前提とする
- SQLite WAL モード（NFR-LGX-001.PERF.07）は共有メモリを使用するため、ネットワーク FS では動作保証できない

起動時に engine.db の配置先がネットワーク FS と判定された場合、**Warning を出力し動作を継続**する（Error にしない理由: 判定が誤検知の可能性あり、運用者責任とする）。
**根拠:** SQLite 公式（sqlite.org/wal.html）、VAL-LGX-001 Finding E-03、NFR-LGX-001.REL.08
**検証方法:** ネットワーク共有上に engine.db を配置した際の Warning 出力テスト（Step 2 で実施）

### SPEC-LGX-008.REQ.13: 設定ファイルの探索順序（旧名フォールバック）

**規定対象の明確化（前段ループ反復 1 で確定）:** 本 REQ（および REQ.04）が規定するのは**成果物 legixy（legixy-the-tool）が生成・読む設定ファイル**である。本リポジトリ自身の開発運用が旧 `traceability-engine` バイナリで読む `.trace-engine.toml` は開発プロセス側の**別レイヤ**であり、本 REQ の対象外（CLAUDE.md「プロジェクト固有の補足」の二層区別と整合。QSET-LGX-008 Q2 回答 2026-06-07）。

**内容:** legixy は設定ファイルを以下の優先順位で探索する。最初に見つかった 1 ファイルのみを使用し、複数同時存在時は上位を採用する。

1. `.legixy.toml`（**既定・正式名**）
2. `.trace-engine.toml`（**旧名フォールバック**、`traceability-engine` 互換）

挙動の詳細:
- `.legixy.toml` が存在すればそれを読み、`.trace-engine.toml` は無視する。
- `.legixy.toml` が無く `.trace-engine.toml` のみ存在する場合は後者を読み、**一度だけ Info を出力**して `.legixy.toml` への移行を案内する（処理は継続、Error にしない）。`legixy migrate` 実行時は `.legixy.toml` を生成し、旧ファイルは `.bak` 退避する（REQ.04 と整合）。
- 両方存在する場合は `.legixy.toml` を優先し、`.trace-engine.toml` の不一致があれば Warning を出力する。
- どちらも無い場合は未初期化として扱う（`init` 誘導）。
- 設定ファイルの**内容スキーマは両ファイルで同一**（§6 / LGX-COMPAT-001 §6）。ファイル名のみが異なる。
- `init` は常に `.legixy.toml` を生成する（旧名では生成しない）。

**根拠:** LGX-COMPAT-001 §6（実行時引数互換とは別に、既存 `traceability-engine` プロジェクトを再 init なしで読めるようにするための後方互換）。
**検証方法:** (a) `.legixy.toml` のみ → それを使用、(b) `.trace-engine.toml` のみ → 読込み + 移行 Info、(c) 両方 → `.legixy.toml` 採用 + 不一致時 Warning、(d) 両方無し → init 誘導、の 4 ケーステスト。

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-2（グラフ整合性） | 関連 | REQ.03（matrix → graph 変換で整合性を保つ、実装本体は SPEC-LGX-002）, REQ.11（マッピング不可 ID は既定 abort・継続時もエッジ除外で dangling 参照を残さない、GAP-LGX-146） |
| FB-INV-4（DB 不在時安全性） | 関連 | REQ.07（init コマンドで新規 DB 作成、以降は DB なしでも graph 読み込み可） |
| STATE-INV-1（ステートレス性） | 関連 | 本マイグレーション処理は初回のみの一時的操作であり、以降は legixy がステートレスである制約を壊さない |
| STATE-INV-2（graph.toml は Git 経由） | 実装 | REQ.03（graph.toml の自動生成後、**ユーザが Git commit するまでが完了**という運用ガイダンスを文書化）, REQ.11（ID 書き換え後も同様）。本 SPEC では Git commit を自動実行せず、運用側責任として明記 |
| SUBNODE-INV-1（親存在） | 関連 | REQ.05（移行直後はドキュメントノードのみ） |
| SUBNODE-INV-2（パス整合性） | 関連 | REQ.05 |
| SUBNODE-INV-3（ID 一意性） | 関連 | REQ.05, REQ.11（id-map 全単射保証 — 新 ID 側衝突と graph 全体一意性違反を Error、GAP-LGX-159） |
| SUBNODE-INV-4（DAG） | 関連 | REQ.05 |
| SUBNODE-INV-6（ID フォーマット） | 関連 | REQ.05 |

**本 SPEC が関与しない不変条件:** CTX-INV-1/3/4/5, MCP-INV-1〜4、SUBNODE-INV-5、FB-INV-1/2/3/5、SCORE-INV-1/2、CACHE-INV-1〜4

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | F-01 修正: 不変条件テーブルの CTX-INV-3 誤記を CTX-INV-2（グラフ整合性）に修正。F-07 修正: REQ.01 を「明示実行デフォルト、auto は .legixy.toml で opt-in」に変更、読み取り系コマンドの副作用防止 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列を追加、SUBNODE-INV-* の集約記述を個別行（1,2,3,4,6）に展開、対象外不変条件（CTX-INV-1/3/4, MCP-INV-*, SUBNODE-INV-5）を明記 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-c 対応: REQ.11 v0.1.0 → legixy ID 互換性維持（マッピングテーブル自動生成、--dry-run 対応、Finding E-05）、REQ.12 engine.db 配置条件（ローカル FS 限定、ネットワーク FS 禁止、Finding E-03）を新設 |
| 2026-04-17 | 0.3.1 | S1-d 対応: §4 表に FB-INV-4（DB 不在時安全性）、STATE-INV-1（ステートレス性）、STATE-INV-2（graph.toml は Git 経由）を追加。STATE-INV-2 は migrate 後の Git commit を運用ガイダンスとして明記 |
| 2026-04-20 | 0.4.0-draft | INIT Block（workflow_2026-04-20_init-template-spec.md）で REQ.07 を改訂: ICONIX 標準 8 typecode + `[id.document_id]` を含む完全 template を明記、ICONIX 成果物用 8 ディレクトリ（`docs/{specs,usecases,robustness,sequence,detailed-design,test-specs}` + `tests/` + `src/`）を init 時に作成する要件を追加。`/sc:analyze` Finding 1（2026-04-20、init 直後の check --formal が機能不全）への対応 |
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-008 回答 → SPP-LGX-008 承認）対応: REQ.07 に「二段化は DevProc 運用の上書きレイヤであり init 既定は単段 ICONIX を維持」を明記（Q1、選択肢 A）。REQ.13/REQ.04 の規定対象を「legixy-the-tool の設定」と明確化（Q2）。ヘッダ Version の不整合（0.3.1 表記 vs 履歴 0.4.0-draft）も本版で吸収 |
| 2026-06-10 | 0.6.0 | TP[SPEC] GAP 解消（人間承認 2026-06-10、7 件単一改訂）: **GAP-LGX-157（BLOCKER）人間裁定・案A** — REQ.06 を凍結契約どおり `--from <PATH>` 必須 / `--to <PATH>` 既定 --project-root / --dry-run / --format に確定、バージョンは REQ.09 自動検出へ移管（凍結契約無変更、ADR 記録）。GAP-LGX-154（旧 147/149 統合）— REQ.09 を PRAGMA user_version 一次根拠 + [graph] セクション判定 + マーカ欠落 v0.1.0 + 矛盾 Error に確定（v3 実在確認済）。GAP-LGX-146 — REQ.11 マッピング不可 ID は既定 abort・継続 opt-in 時エッジ除外。GAP-LGX-159 — REQ.11 id-map 全単射保証（曖昧性/多対一/全体一意性違反 Error、--dry-run でも検証）。GAP-LGX-152 — REQ.02 非 DB ファイルの temp+fsync+rename と DB コミット先行順序。GAP-LGX-153 — REQ.02a 退避命名 `<名>.bak.{epoch}`（固定 .bak 禁止・累積保持）を新設。GAP-LGX-143 — REQ.07「既存」判定を legixy 生成物に限定、--force の破壊範囲限定。§4 CTX-INV-2 / SUBNODE-INV-3 行を更新 |
| 2026-06-10 | 0.7.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10、8 件単一改訂）: GAP-LGX-141 — REQ.03 空入力は空 graph.toml + exit 0 + Info。GAP-LGX-142 — REQ.03 migrate は PERF 予算対象外（NFR 申し送り）・全ロード許容。GAP-LGX-158 — REQ.03 抽出規則（[matrix]/[id] 設定準拠・構造崩れ 0 件は空入力扱い・[id.chain] 欠落は破損 Error）。GAP-LGX-144 — REQ.03a 新設（破損検出・Error 中断・原本温存・出力妥当性検証）。GAP-LGX-148 — REQ.02 に resume なし全やり直し方式（冪等収束）。GAP-LGX-150/151 — REQ.02 に並行アクセス方針（WAL 読取一貫性 + atomic rename・明示排他なし・二重実行は SQLite ロックで敗者 exit 1、SEC.08 リスク受容 = ADR-LGX-011）。GAP-LGX-160 — REQ.08 に成功時変更サマリ（stdout、--format json 構造化、STATE-INV-2 運用支援） |
| 2026-06-13 | 0.7.1 | DD 整合（人間承認 2026-06-13、spec-change 提案 2026-06-13_dd-freeze-spec-alignment M-3/M-4）: REQ.01 の移行元 DB 名を `engine.db` → `feedback.db`（`.trace-engine/feedback.db`）に訂正し、移行先＝正準 `engine.db`（`.legixy/engine.db`、ADR-LGX-015）への observations/proposals/custom_edges 統合を明示（v3 lx-mig/db.rs 整合）。REQ.03/03a に `[id.chains]`+`[id.areas]` multi-area 変種の受理を明記（ADR-LGX-018 #15、DD-LGX-009）。CLI 引数・終了コードは不変 |
