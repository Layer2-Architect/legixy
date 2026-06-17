Document ID: TP-LGX-008

# TP-LGX-008: マイグレーション

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。

**親**: SPEC-LGX-008
**ステータス**: green
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は SPEC-LGX-008（マイグレーション）の全要求（REQ.01〜REQ.13）を対象とする。中核は v0.1.0 engine.db → legixy スキーマ変換、matrix 形式 `.legixy.toml` / `matrix.md` → `graph.toml` 生成、`init` / `migrate` コマンド、設定ファイル探索順（旧名フォールバック）、非破壊性・冪等性・バージョン検出。

- 対象: SPEC-LGX-008 §3（REQ.01〜REQ.13）、§4（不変条件との関係: STATE-INV-1/2, FB-INV-4, SUBNODE-INV-*, CTX-INV-2）
- 関連 SPEC §: SPEC-LGX-001（STATE-INV-2 graph.toml は Git 経由、データ配置）、LGX-EXT-001 §4.4/§8.2（後方互換・Phase 1 方針）、LGX-COMPAT-001 §4（migrate/init 引数契約）, §6（設定ファイル探索順）、NFR-LGX-001.REL.01/REL.08, USE.02, COMPAT.04/05/11, PERF.07
- 委譲（本 SPEC のスコープ外で他成果物が保証する観点）: graph.toml の整合性検証本体（SPEC-LGX-002 / CTX-INV-2）、ネットワーク FS Warning 実装の試験は Step 2（REQ.12）

## 2. 観点リスト

### 2.1 境界値

- [ ] B-1: 空の v0.1.0 プロジェクト（成果物 0 件、空 engine.db / 空 matrix.md）を migrate したときの挙動（空 graph.toml を生成するか、エラーか）
- [ ] B-2: 成果物が 1 件のみの matrix → graph.toml 変換（単一ノード・エッジ 0 本）
- [ ] B-3: 巨大な v0.1.0 engine.db / 大量ノードの matrix（数万ノード）でのメモリ・所要時間境界
- [ ] B-4: `migrate --dry-run` の出力が実 migrate と同一の変更集合を表すこと（差分なし境界）
- [ ] B-5: 既存ファイルが「一部だけ」存在する init（例: `.legixy.toml` はあるが graph.toml は無い）の境界判定（§3 REQ.07 のエラー判定基準）

### 2.2 エラーハンドリング

- [ ] E-1: 各マイグレーション段階（engine.db / graph.toml / id-map / config）の失敗が型付けされ、失敗段階・バックアップ場所・リカバリ手順が提示されるか（REQ.08）
- [ ] E-2: source データ破損時（壊れた engine.db、不正 TOML な matrix `.legixy.toml`、整合しない matrix.md）の検出と挙動
- [ ] E-3: ディスクフル / 書込権限なしで graph.toml / `.bak` / id-map を書けないときの挙動と原本保護
- [ ] E-4: `migrate` の `--from` が指す入力が存在しない / 読めない場合のエラー（LGX-COMPAT-001 §4: `--from <PATH>` 必須）
- [ ] E-5: `init` で既存ファイルがある場合のエラーと `--force` 上書き時の旧 `.legixy.toml` 退避（REQ.07）
- [ ] E-6: REQ.11 のマッピング不可 ID（旧 graph.toml に対応新 ID が決まらない）時の Warning 報告と処理継続/中断方針

### 2.3 状態遷移

- [ ] S-1: 未初期化 → init 済 → migrate 済 の遷移と各状態の判定基準（何をもって「migrate 済」とするか）
- [ ] S-2: 既に legixy 形式のプロジェクトに対する `migrate` の no-op（REQ.06）と、その判定（already-migrated detection）
- [ ] S-3: 部分マイグレーション後の再実行（engine.db は変換済だが graph.toml 未生成等、中間状態）からの収束
- [ ] S-4: auto=true 自動変換が「初回コマンド実行時のみ」発火し、2 回目以降は再発火しない遷移（REQ.01）
- [ ] S-5: migrate 直後（graph.toml 生成済・Git commit 前）は「未完了」であり、commit までを完了とする運用状態の定義（STATE-INV-2 / REQ.03）

### 2.4 並行性

- [ ] C-1: migrate 実行中に他の legixy コマンド（check 等）が同じ engine.db / graph.toml にアクセスした場合の競合（SQLite ロック・WAL、PERF.07）
- [ ] C-2: 2 つの migrate を同時起動した場合の二重実行防止（idempotency / ロック）
- [ ] C-3: auto=true で複数コマンドが同時初回起動した場合の自動変換の重複起動防止（REQ.01）

### 2.5 永続化（中核）

- [ ] P-1: 途中中断（プロセス kill / Ctrl-C / 電源断）で engine.db が壊れないこと（SQLite トランザクション、REQ.02 / REL.01）
- [ ] P-2: graph.toml / id-map の書込みが atomic か（部分書き込み・途中電源断で半端な graph.toml が残らないか。temp+rename 等）
- [ ] P-3: 失敗時に元ファイル（engine.db, `.legixy.toml`）を `.bak` で保存し原本を保護（REQ.02 / REQ.13）
- [ ] P-4: 冪等性 — 同一入力に対し migrate を 2 回以上実行しても結果が同一・破壊しない（REQ.02）
- [ ] P-5: `.bak` が既に存在する場合の上書き / 連番 / 拒否の方針（再実行・連続失敗時のバックアップ衝突）
- [ ] P-6: ネットワーク FS 上の engine.db 配置検出と Warning 継続（REQ.12 / REL.08, Step 2）

### 2.6 バージョニング・互換性（中核）

- [ ] V-1: バージョン検出 — `.legixy.toml` と engine.db 双方からバージョンを読み、不整合なら Error（REQ.09）
- [ ] V-2: バージョン検出の網羅性 — そもそも v0.1.0 か legixy かを「どのフィールド/スキーマ特徴」で判定するか（version detection の根拠）
- [ ] V-3: 前方互換 — legixy が生成した graph.toml / config を旧 `traceability-engine` v0.1.0 バイナリが読めるか（読めない場合の方針明示）
- [ ] V-4: 後方互換 — v0.1.0 の `[matrix]` セクションを残す意味変更（graph→matrix 生成設定）が定義されているか（REQ.04）
- [ ] V-5: 将来拡張 — バージョン間変換ステップの列として構造化され、v0.1.0 → legixy → 将来版の段階適用が可能か（REQ.10）
- [ ] V-6: ロールバック — migrate 後に旧状態へ戻す手段（`.bak` からの復旧手順）が定義され、ロールバック自体が graph.toml/engine.db を壊さないか
- [ ] V-7: 設定ファイル探索順 — `.legixy.toml` / `.trace-engine.toml` の 4 ケース（片方のみ×2・両方・両方無し）の挙動が定義されているか（REQ.13 / LGX-COMPAT-001 §6）
- [ ] V-8: ID 互換 — 旧 graph.toml 内の手書き ID 参照（edge from/to, parent）を新 ID へ書き換え、マッピングテーブルを生成（REQ.11）
- [ ] V-9: `migrate` の `--from` / `--to` の意味 — LGX-COMPAT-001 §4 は PATH 引数、SPEC-008 REQ.06 はバージョン文字列（`--from v0.1.0 --to legixy`）。どちらが正か

### 2.7 入力検証

- [ ] I-1: matrix.md のフォーマット検証（成果物 ID 抽出規則・節構造の前提）と不正時の扱い（REQ.03）
- [ ] I-2: matrix 形式 `.legixy.toml` の `[id.chain]` 順序定義が欠落/不正な場合のエッジ生成の挙動（REQ.03）
- [ ] I-3: v0.1.0 の `custom_edges` テーブルが存在しない / スキーマ差異がある場合の継承（REQ.03）
- [ ] I-4: `migrate --format markdown|json` の出力フォーマット指定とその検証（LGX-COMPAT-001 §4）
- [ ] I-5: id-map（`.legixy/migration-id-map.toml`）の形式・新旧 ID 一意性・重複検出（REQ.11）

### 2.8 ライフサイクル

- [ ] L-1: init 直後に `check --formal` が 0 ERROR で通ること、ICONIX 8 ディレクトリ + `.gitkeep` + `.legixy.toml`（8 typecode + `[id.document_id]`）が存在すること（REQ.07）
- [ ] L-2: init が生成する空 graph.toml と初期スキーマ engine.db の妥当性（FB-INV-4: 以降 DB なしでも graph 読込可）
- [ ] L-3: Phase 1 — migrate / init 直後の graph.toml にサブノードを含まずドキュメントノードのみであること（REQ.05 / SUBNODE-INV-*）
- [ ] L-4: マイグレーション処理が「初回のみの一時操作」でありステートレス性（STATE-INV-1）を壊さないこと
- [ ] L-5: init の ICONIX 既定が DevProc 二段化（RBA/RBD 等）ではなく単段 8 ディレクトリ + chain `UC→RB→SEQ→DD→TS→TC→SRC` であること（REQ.07 の二段化レイヤ区別）

### 2.9 ロギング・観測性

- [ ] O-1: `migrate` / auto 変換 / 設定フォールバック時のログ（移行 Info は「一度だけ」出力されるか, REQ.13）
- [ ] O-2: 失敗時ログから失敗段階・原因・リカバリが特定できる情報量（REQ.08 / USE.02）
- [ ] O-3: バックアップ場所・id-map 生成・書換え対象 ID がログ/出力に記録され追跡可能か
- [ ] O-4: dry-run 出力の構造（人間/機械可読、--format との関係）と「実行されないこと」の明示

### 2.10 FFI / 境界 API（init / migrate CLI、LGX-COMPAT-001）

- [ ] F-1: `init` の引数（`[--force]` のみ）が LGX-COMPAT-001 §4 行 1 と一致し、追加フラグを増やしていないか
- [ ] F-2: `migrate` の引数（`--from <PATH>` 必須, `[--to <PATH>]`, `[--dry-run]`, `[--format markdown|json]`）が LGX-COMPAT-001 §4 行 2 と一致するか
- [ ] F-3: グローバルオプション（`--project-root` / `--json` / `--models-dir`）を init / migrate が受理するか
- [ ] F-4: 終了コード — migrate / init の成功 0・失敗 非0 の規約（check の G1 終了コード規約との一貫性）
- [ ] F-5: `.legixy.toml` vs `.trace-engine.toml` の二層区別 — 本 SPEC が規定するのは legixy-the-tool の設定であり、開発側 `.trace-engine.toml`（旧バイナリが読む別レイヤ）を対象外とすることの明示（REQ.13 / REQ.04）

### 2.11 領域固有観点（マイグレーション）

- [ ] D-1: 冪等性の再実行安全 — 既 migrate 済での再 migrate が「既に legixy なら no-op」へ収束（REQ.06 / S-2 と連動）
- [ ] D-2: 部分マイグレーション/中断からの再開戦略（resume / 全やり直し / 検出して継続のどれか, P-1/S-3 と連動）
- [ ] D-3: バックアップを取ってから上書きする順序保証（backup-before-overwrite, REQ.02/REQ.13）
- [ ] D-4: ソースデータ破損の検出と「破損を黙って引き継がない」保証（E-2 と連動、移行後 graph の妥当性）
- [ ] D-5: matrix.md / graph.toml 二重 source 問題 — migrate 後 matrix.md は graph.toml 由来の読取専用ビュー（COMPAT.05）に切り替わる遷移が定義されているか

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 関連文書 §で回答 | 関連 GAP |
|---|---|---|---|
| B-1 空プロジェクト migrate | GREEN | REQ.03 は変換ルールのみ。成果物 0 件時の出力（空 graph.toml か Error か）未定義 ※minor（verification: low-value, 人間判断で drop 可。0 件は一般則で空 graph.toml に自明収束する DD 詳細）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-141 |
| B-2 単一ノード変換 | GREEN | REQ.03（各成果物 ID をノード化、chain 順でエッジ）— 件数に依存しない一般則で被覆 | — |
| B-3 大規模入力の境界 | GREEN | サイズ・所要時間・メモリの上限/予算が SPEC・NFR(PERF) のいずれにも未記載 ※minor（verification: low-value, 人間判断で drop 可。migrate は初回一時操作で PERF.* ホットパス対象外、GAP 自身が「予算対象外と明示」を許容）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-142 |
| B-4 dry-run 等価性 | GREEN | REQ.06「変更内容を表示するが書き込まない」+ REQ.11「書き換え対象を事前確認」で意図定義 | — |
| B-5 一部のみ存在 init | GREEN | REQ.07「既存ファイルがある場合エラー」だが「どれか 1 つでも」か「全部揃う場合」か境界が曖昧【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-143 |
| E-1 段階別エラー型・提示 | GREEN | REQ.08（失敗段階・バックアップ場所・リカバリ手順を提示）+ USE.02 | — |
| E-2 source データ破損 | GREEN | 破損 engine.db / 不正 TOML / 不整合 matrix.md の検出・拒否方針が未定義 ※minor（verification: low-value, 人間判断で drop 可。DB/TOML 整合検査は入力検証の DD 詳細、REQ.09 バージョン検出 + REQ.02 原本保持で大筋被覆）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-144 |
| E-3 ディスクフル/権限 | GREEN | REQ.02（失敗時は原本を `.bak` 保持・SQLite トランザクションで途中中断耐性）+ REQ.08（失敗段階・バックアップ場所・リカバリ手順提示）+ NFR USE.02。書込不能（ENOSPC/EACCES）は「失敗」の一形態として原本保護 + エラー提示で被覆 | — |
| E-4 --from 不在/不読 | GREEN | LGX-COMPAT-001 §4 で `--from <PATH>` 必須。不在は通常のパスエラー（E-1/USE.02 で被覆） | — |
| E-5 init 既存/--force 退避 | GREEN | REQ.07（既存→エラー、`--force` で上書き）+ LGX-COMPAT-001 §4 行1（`.bak` 退避） | — |
| E-6 マッピング不可 ID | GREEN | REQ.11 は「Warning 報告し人間確認を促す」とあるが、処理継続か中断か、未マップ ID の graph.toml 上の扱いが未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-146 |
| S-1 状態遷移と判定基準 | GREEN | DUPLICATE: 「migrate 済（=legixy 形式）」判定は本質的にバージョン判定根拠の問題。GAP-LGX-154（バージョン判定の根拠、keeper）に集約。マーカ定義が定まれば「未初期化/init 済/v0.1.0/migrate 済」各状態は自然に決まる | GAP-LGX-154 |
| S-2 already-migrated no-op | GREEN | REQ.06「既に legixy の場合は no-op」 | — |
| S-3 部分migrate からの収束 | GREEN | REQ.02 は「壊れない」「再実行可能」だが、中間状態（DB変換済/graph未生成）から再実行時に何が再計算されるかの収束保証が未定義 ※minor（verification: low-value, 人間判断で drop 可。REQ.02 冪等性により全やり直しが安全で resume/再計算は非決定事項）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-148 |
| S-4 auto 初回のみ発火 | GREEN | DUPLICATE: 「初回」識別 = 「v0.1.0 形式が検出された時のみ発火、migrate 済になれば自然に発火しない」であり、判定根拠は GAP-LGX-154（keeper）に集約。auto 経路特有の追加マーカは不要（GAP-LGX-149 自身が「GAP-LGX-147 と同根」と明記） | GAP-LGX-154 |
| S-5 commit までが完了 | GREEN | REQ.03 + §4 STATE-INV-2 行（Git commit までが完了、自動 commit せず運用責任と明記） | — |
| C-1 migrate中の同時アクセス | GREEN | SQLite WAL 前提（PERF.07）はあるが、migrate 中に check 等が走った場合の競合方針が SPEC に未定義 ※minor（verification: low-value, 人間判断で drop 可。engine.db 側は PERF.07 WAL + REL.07 busy_timeout で被覆、graph.toml 側は GAP-LGX-152 atomic 書込に吸収）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-150 |
| C-2 二重 migrate 防止 | GREEN | 同時 migrate 起動時のロック/排他が未定義（冪等とは別の同時並行問題） ※minor（verification: low-value, 人間判断で drop 可。REL.07 busy_timeout が DB ロック競合を律速、REQ.02 冪等性で二重実行も破壊せず収束）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-151 |
| C-3 auto 重複起動防止 | GREEN | REQ.01 auto で複数コマンド同時初回起動時の自動変換重複防止が未定義（C-2/S-4 と連動） ※minor（verification: low-value, 人間判断で drop 可。C-2 と同一 GAP、REL.07 + 冪等性で被覆）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-151 |
| P-1 中断で DB 壊れない | GREEN | REQ.02（SQLite トランザクション、途中中断で壊れない）+ REL.01 | — |
| P-2 graph/id-map atomic 書込 | GREEN | REQ.02 の非破壊は engine.db 文脈中心。graph.toml / id-map の atomic 書込（temp+rename 等）が未明記【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-152 |
| P-3 失敗時の原本 .bak 保護 | GREEN | REQ.02（失敗時 `.bak` で原本保存）+ REQ.13（旧 config を `.bak` 退避） | — |
| P-4 冪等性 | GREEN | REQ.02（再実行可能・冪等）+ REQ.06（既 legixy は no-op） | — |
| P-5 .bak 衝突方針 | GREEN | `.bak` が既存（連続失敗・再実行）時に上書き/連番/拒否いずれかが未定義。原本上書き事故リスク【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-153 |
| P-6 ネットワークFS Warning | GREEN | REQ.12（ローカル FS 限定・ネットワーク FS は Warning 継続）+ REL.08。試験は Step 2 へ正当に委譲 | — |
| V-1 双方バージョン不整合 Error | GREEN | REQ.09（`.legixy.toml` と engine.db 双方から検出、不整合は Error） | — |
| V-2 バージョン判定の根拠 | GREEN | 「どのフィールド/スキーマ特徴で v0.1.0 か legixy か判定するか」の具体が REQ.09 に未記載【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-154 |
| V-3 前方互換（旧バイナリ読込） | GREEN | OUT_OF_SCOPE: SPEC-008 §1.2 スコープ「含まない: legixy 以降のマイグレーション」+ LGX-EXT-001 §8.2/§9.2（移行は片方向 v0.1.0→legixy、line 714）。旧バイナリでの読み戻しは移行の逆方向であり本 SPEC の対象外 | — |
| V-4 [matrix] 意味変更 | GREEN | REQ.04（`[matrix]` 残置・graph→matrix 生成設定へ意味変更）+ COMPAT.05 | — |
| V-5 将来拡張・段階適用 | GREEN | REQ.10（バージョン間変換ステップの列として構造化、段階適用可能） | — |
| V-6 ロールバック手段 | GREEN | OUT_OF_SCOPE: §4 STATE-INV-2（graph.toml は Git 経由、commit までが完了・自動 commit せず運用責任）+ REQ.02（原本 `.bak` 保持）。ロールバック = `.bak` 復元 + Git revert の運用手順に委譲。専用コマンドは将来要求 | — |
| V-7 config 探索 4 ケース | GREEN | REQ.13（4 ケースを明示: 片方のみ×2・両方優先・両方無し）+ LGX-COMPAT-001 §6 | — |
| V-8 ID 書換え+マッピング生成 | GREEN | REQ.11（id-map 自動生成、from/to/parent 書換え、dry-run 対応） | — |
| V-9 migrate --from/--to 意味矛盾 | GREEN | LGX-COMPAT-001 §4 は `--from/--to <PATH>`、SPEC-008 REQ.06 は `--from v0.1.0 --to legixy`（バージョン文字列）。契約矛盾【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-157 |
| I-1 matrix.md 形式検証 | GREEN | REQ.03「matrix.md の各成果物 ID をノード抽出」だが抽出規則・節構造前提・不正時の扱いが未定義 ※minor（verification: low-value, 人間判断で drop 可。抽出規則は `[matrix].section` + `[id].pattern` から導出可能な DD 詳細）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-158 |
| I-2 [id.chain] 欠落/不正 | GREEN | REQ.03 は chain 順でエッジ生成とあるが、順序定義が欠落/不正な v0.1.0 入力時の挙動が未定義 ※minor（verification: low-value, 人間判断で drop 可。I-1 と同一 GAP、欠落時はエッジ 0 本で続行が自明な DD 詳細）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-158 |
| I-3 custom_edges 不在/差異 | GREEN | REQ.03（custom エッジは「v0.1.0 にあれば」継承）— 不在時は継承なしで明示 | — |
| I-4 --format 指定 | GREEN | LGX-COMPAT-001 §4（`--format markdown|json` 既定 markdown） | — |
| I-5 id-map 一意性/重複検出 | GREEN | REQ.11 は生成のみ言及。新旧 ID の一意性検証・衝突（複数旧 ID→同一新 ID 等）検出が未定義【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-159 |
| L-1 init 直後 check 0 ERROR | GREEN | REQ.07（ICONIX 8 typecode+`[id.document_id]`, 8 ディレクトリ+`.gitkeep`, init 直後 check --formal 0 ERROR） | — |
| L-2 空 graph + 初期 DB 妥当 | GREEN | REQ.07 + §4 FB-INV-4（init で新規 DB、以降 DB なしでも graph 読込可） | — |
| L-3 Phase 1 ドキュメントノードのみ | GREEN | REQ.05（移行直後はサブノード含まずドキュメントノードのみ）+ SUBNODE-INV-* | — |
| L-4 ステートレス維持 | GREEN | §4 STATE-INV-1 行（初回のみの一時操作、ステートレス制約を壊さない） | — |
| L-5 init 既定は単段 ICONIX | GREEN | REQ.07（二段化は DevProc 上書きレイヤ、init 既定は単段 8 ディレクトリ + chain `UC→RB→SEQ→DD→TS→TC→SRC`） | — |
| O-1 移行 Info 一度だけ | GREEN | REQ.13（旧名のみ存在時「一度だけ Info を出力」） | — |
| O-2 失敗ログの情報量 | GREEN | REQ.08 + USE.02（該当ファイルとリカバリ方法を示唆） | — |
| O-3 backup/id-map 追跡可能 | GREEN | REQ.08 はバックアップ場所提示だが、書換え対象 ID 一覧の出力・追跡可能性が未明記（V-8 dry-run と通常 run の差） ※minor（verification: low-value, 人間判断で drop 可。REQ.11 dry-run + `--json`/`--format`（§7）+ id-map ファイル自体が監査痕跡、commit は git diff で足りる）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-160 |
| O-4 dry-run 出力構造 | GREEN | REQ.06（変更内容を表示・書き込まない）+ LGX-COMPAT-001 §4（`--format`） | — |
| F-1 init 引数一致 | GREEN | LGX-COMPAT-001 §4 行1（`[--force]` のみ）+ REQ.07 と一致 | — |
| F-2 migrate 引数一致 | GREEN | LGX-COMPAT-001 §4 行2（`--from`/`--to`/`--dry-run`/`--format`）— ※意味の矛盾は V-9 で別途 | — |
| F-3 グローバルopt 受理 | GREEN | LGX-COMPAT-001 §7（`--project-root`/`--json`/`--models-dir` 全コマンド受理） | — |
| F-4 終了コード規約 | GREEN | NFR OBS.05（`0=OK, 1=Error, 2=使用法誤り`）+ USE.04 + LGX-COMPAT-001 §3 グローバル規約（使用法誤り exit 2 / 実行時失敗 exit 1、全サブコマンド契約化）。migrate/init もこのグローバル規約に拘束される | — |
| F-5 設定二層区別の明示 | GREEN | REQ.13 / REQ.04（legixy-the-tool の設定を規定、開発側 `.trace-engine.toml` は別レイヤと明記）+ CLAUDE.md 二層区別 | — |
| D-1 再 migrate no-op 収束 | GREEN | REQ.06（既 legixy は no-op）+ REQ.02（冪等） | — |
| D-2 部分migrate 再開戦略 | GREEN | resume / 全やり直し / 検出継続のいずれを採るか未定義（S-3 と同根、再開戦略として別立て） ※minor（verification: low-value, 人間判断で drop 可。S-3 と同一 GAP、冪等性で全やり直しが安全）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-148 |
| D-3 backup-before-overwrite 順序 | GREEN | REQ.02（失敗時原本保存）+ REQ.13（`.legixy.toml` 生成時に旧を `.bak` 退避）で順序意図あり | — |
| D-4 破損を黙って引き継がない | GREEN | E-2 と同根。破損 source 検出後に「壊れた graph を生成しない」保証が未定義 ※minor（verification: low-value, 人間判断で drop 可。E-2 と同一 GAP・同根の DD 詳細）【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-144 |
| D-5 matrix.md→読取専用ビュー遷移 | GREEN | REQ.04 + COMPAT.05（matrix.md は graph.toml 由来の読取専用ビュー） | — |

**集計（2026-06-09 敵対的精査パス後）**: 全 51 観点 / GREEN 37 / RED 14。GAP は 20 件 → **15 件**（削除 5 件: GAP-LGX-145/147/149/155/156、維持 15 件: GAP-LGX-141/142/143/144/146/148/150/151/152/153/154/157/158/159/160）。RED 14 観点は 15 GAP に対応（一部観点は同一 GAP に集約 — E-2/D-4=144、S-3/D-2=148、C-2/C-3=151、I-1/I-2=158）。なお維持 GAP のうち多数は ※minor（人間判断で drop 可）注記付きで、純粋に GENUINE と判定したのは GAP-LGX-143/146/152/153/154/157/159 の 7 件。

**初版（敵対的精査前）**: 全 51 観点 / GREEN 31 / RED 20（GAP-LGX-141〜160 の連番 20 件を発行）

## 4. ステータスの決定

敵対的精査パス（2026-06-09）後も RED 観点が 14 件（維持 GAP 15 件）存在するため、本 TP のステータスは `**ステータス**: green` のまま。維持中の全 GAP が closed になり次第、該当観点を GREEN へ更新し、TP を `green` に再評価する。なお ※minor 注記付き RED は人間判断で drop 可能であり、GENUINE と判定した 7 件（GAP-LGX-143/146/152/153/154/157/159）の解消が実質的な前提となる。GAP-LGX-157 は LGX-COMPAT-001 §4（凍結境界契約）と REQ.06 の `--from`/`--to` 意味矛盾であり、SPEC 修正（人間承認）を要する blocker。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値, §エラーハンドリング, §状態遷移, §並行性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性, §バージョニング・互換性, §FFI/境界 API, §領域固有観点（バージョン管理系・大規模データパイプライン: バックフィル冪等性）
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX, §永続化と同期（クラッシュリカバリ）, §待機/進捗の UX（dry-run/進捗）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成、観点 51 件（11 カテゴリ）、RED 20 / GREEN 31。GAP-LGX-141〜160 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 5 件 / 維持 15 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
