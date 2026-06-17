Document ID: PAI-CLI-001
**対象成果物**: legixy 実行ファイル（`deploy/bin/legixy`、release+onnx ビルド）
**対象契約 / SPEC**: CTR-CLI-001、LGX-COMPAT-001、SPEC-LGX-001〜010、LGX-EXT-001、LGX-EXT-002、NFR-LGX-001
**実施主体**: 外部検査スイート `legixy.test`（`legixy.test`）。Author（実装）セッションと独立な主体が・黒箱で・実環境で実施（DevProc_V4.1 §13 §3 独立性要件）。
**実施環境**: release+onnx ビルド / 日本語対応 ONNX モデル `paraphrase-multilingual-MiniLM-L12-v2` / `legixy.test/fixtures/*`（hermetic）/ Linux

<!-- metadata（chain 外 independent。親への参照。CLAUDE.md ハードルール 3）
  parent_contract: CTR-CLI-001
  parent_specs: LGX-COMPAT-001, SPEC-LGX-001, SPEC-LGX-002, SPEC-LGX-003, SPEC-LGX-004,
                SPEC-LGX-005, SPEC-LGX-006, SPEC-LGX-007, SPEC-LGX-008, SPEC-LGX-010,
                LGX-EXT-001, LGX-EXT-002, NFR-LGX-001
  harness: legixy.test（独立黒箱スイート）
  area: CLI（配送軸、independent）
-->

# PAI-CLI-001: CLI 完成品契約適合検査

> 完成品適合検査（`docs/DevProc_V4/13-product-acceptance-inspection.md`）。**作者と独立に・黒箱で・実環境で**、
> 完成品（legixy 実行ファイル）が SPEC の意図と凍結境界契約を満たすかを検査する。**ケースは SPEC-REQ / CTR から
> 独立に作成**する（下流チェーン UC/DD/TS/TC・実装・in-repo TC を見ない。CTR は SPEC を外部公開する表面＝検査の経路）。
>
> **本書の位置づけ**: 既に稼働していた外部検査スイート `legixy.test` を legixy の正式な PAI 成果物として制度化したもの。
> `legixy.test` が「機能 GREEN（TC[DLV] 含む）をすり抜けた契約・SPEC 意図の不適合」を反復検出した事実が、PAI が方法論へ
> 追加された当の根拠である（13章 §「機能 GREEN は契約適合を意味しない」の実証元）。

## 0. 検査ハーネスと独立性

- **ハーネス**: `legixy.test`（別ツリー）。計画書 `legixy.test/docs/external-test-plan.md`（CLI/MCP コア）・
  `external-test-subnode-cache.md`（LGX-EXT-001/002）・`external-test-semantic-drift.md`（意味検証）。fixtures は hermetic。
- **独立性（§3）**: ケースは SPEC-REQ / CTR から独立に起こし、内部実装・in-repo TC を参照しない。判定は**外部観測のみ**
  （終了コード・stdout/stderr・出力スキーマ・ファイル副作用）。自動生成サブノード ID 等のハッシュ依存値は黒箱で実測取得し、
  内部式を前提にしない。
- **作者独立**: 実装（Author）セッションが書いた in-repo 回帰（TC[DLV]）とは**別分布**。両輪で運用し、in-repo へ畳まない
  （独立性＝発見力を失うため。13章 §6 / 12章補正）。

## 1. 検査計画（SPEC-REQ / CTR 表面 → PAI ケース 全数 mapping）

判定は外部観測のみ。`legixy.test` の EXT-* カテゴリを SPEC-REQ / CTR 契約項目へ割り当てる（P-3 全数被覆）。

| 領域（SPEC-REQ / CTR） | PAI ケース群（legixy.test） | 黒箱操作（例） | SPEC 意図の観測 |
|---|---|---|---|
| CTR-CLI-001 §3 終了コード規約 / LGX-COMPAT-001 §3 | EXT-CHK-*, EXT-GLOB-*, EXT-*（exit 0/1/2） | `legixy check --formal` 他 19 サブコマンド | exit code が契約どおり（誤用=2 / 実行時失敗=1 / 正常=0） |
| LGX-COMPAT-001 §3 グローバルフラグ | EXT-GLOB-002/003 | `--json` / `--models-dir` を各コマンドで受理 | exit 2 で拒否しない・JSON 出力 |
| SPEC-LGX-008.REQ.13 設定ファイル | EXT-CONF-050/051/052, EXT-ERR-003 | `.legixy.toml`→`.trace-engine.toml` 探索 | 優先順位・旧名 fallback・不正 TOML=exit1 |
| SPEC-LGX-004.REQ.01 形式層 5 カテゴリ | EXT-CHK-003/004/008, EXT-SUB-EXPL-003 | `check --formal`（不正 ID/path/cycle/slug） | ChainIntegrity / FileExistence / GraphDag / SubnodeIdFormat |
| SPEC-LGX-004.REQ.02 意味層 | EXT-SEM-*（semantic-drift） | `check`（Full）/ embeddings 在 | SemanticSimilarity / LinkCandidate / Drift |
| SPEC-LGX-007 フィードバックループ | EXT-FB-*, EXT-APR-* | feedback→analyze→approve 全周 | chain_integrity / link_candidate / drift → Proposal → 承認 |
| SPEC-LGX-005/006 グラフ走査 | EXT-NAV-*, EXT-SUB-EXPL-002 | impact / investigate | 順/逆方向到達・サブノード解像 |
| SPEC-LGX-003 / LGX-EXT-001 サブノード粒度 | EXT-SUB-AUTO/EXPL/GRAN-* | `context --granularity subnode --sections` | 自動/明示サブノード・トークン削減・区画本文 |
| SPEC-LGX-006 / LGX-EXT-002 embed / CR / cache | EXT-CACHE-CR/DET/META-* | `embed --all`（CR）/ `compile_context` 決定論 | context 列生成・バイト決定論・`_meta` 500000 |
| SPEC-LGX-010 snapshot / drift | EXT-SNAP-*, EXT-SUB-DRIFT-001 | snapshot / drift（部分ドリフト） | サブノード単位 drift・label 解決 |
| NFR-LGX-001 | EXT（PERF/REL/SEC 抜粋） | 実 ONNX throughput・busy_timeout・マスキング | 実測閾値（ADR-LGX-022）・SQLITE_BUSY・API キー非出力 |

> 完全なケース定義・期待値は `legixy.test/docs/*.md` を参照（黒箱ハーネス側が source-of-record）。本 PAI は
> 検査計画・結果・振り分け・プロセス所見を記録する。

## 2. 結果（検査サイクル履歴）

| サイクル | 日付 | 検出 | 振り分け | 状態 |
|---|---|---|---|---|
| 初回 | 2026-06-14 | **契約違反 54 件**（PASS135 / 要改善54 / 未実施49）。全チェーン GREEN + TC[DLV] pass にも関わらず | 全件 /defect-fix（SPEC/契約欠陥 0）。根本原因 BUG-001〜010 | 全クローズ |
| 再検証 | 2026-06-16 午前 | **R-1〜R-7**（配送/統合 E2E 配線の残ギャップ） | /defect-fix | R-4/5/6/7 クローズ |
| 再検証 | 2026-06-16 第2回 | **R-1/R-2/R-3**（サブノード細粒度 walk 結線）+ 軽微 R-7（slug 制約） | /defect-fix | 全クローズ |

**現状判定（2026-06-16）**: 要改善 **0**。R-1〜R-7 + BUG-001〜010 全クローズを実バイナリ（`deploy/bin/legixy`）で確認。
代表的観測:
- R-1: `context --granularity subnode src/order.rs` = **1414 字 < document 2146 字**（LGX-EXT-001 目的1 トークン削減）
- R-2: `investigate SRC-SN-001` が `DD-SN-001#s:state-machine` を解像（document へ潰れない）
- R-3: `--sections "DD-SN-001#s:state-machine"` が「状態遷移」区画本文を返す（空でない）
- R-7: `s:bad id!` → SubnodeIdFormat Error / exit 1、`s:state-machine` → exit 0

## 3. サマリ & リリースゲート手順（DevProc_V4.1 §13 / 08-gates §12）

**PASS / 要改善 / 未実施 = 全カテゴリ PASS、要改善 0（2026-06-16）。要改善ゼロでリリースゲート通過可。**

### リリース前 PAI ゲート（制度化、本 §が手順の正準）
1. **凍結**: リリース対象の完成品をビルド（`LEGIXY_ONNX=1 bash deploy/build-deploy.sh` → `deploy/bin/legixy`）。
2. **独立実行**: Author と独立な主体が `legixy.test` を凍結バイナリに対して黒箱実行（実環境・実モデル）。
   判定は外部観測のみ。**Author の in-repo GREEN を根拠にゲート通過としない**（自己申告の過信防止＝品質偏向防止）。
3. **判定**: 全 PAI ケース PASS（要改善 0）でゲート通過。**要改善 > 0 はリリース不可**。
4. **失敗時フロー（二段）**:
   - 症状の修復: SPEC/契約欠陥なら `/spec-change`、実装ギャップなら `/defect-fix`（legixy の R/BUG は全て後者だった）。
   - **構造の修復（PAI の本質）**: 「なぜ全チェーン GREEN をすり抜けたか」を root-cause し、DevProc プロセスの穴として
     記録・改善（§6 / `docs/decisions/2026-06-16_pai-process-hole.md`）。
5. **独立性の保持**: 本ゲートは `scripts/trace-check.sh`（Author が回す機械ゲート）には**畳まない**。畳むと作者と同一分布化し
   発見力を失う（12章補正 / 13章 §6）。in-repo の TC[DLV] / `[6/6]` 契約ゲートは必要だが不十分、本 PAI が独立チャネルとして補完。

## 4. 未カバー契約項目（検査不備）

- 現時点で全数 mapping 上の漏れなし。`legixy.test` の「未実施 49」は ONNX 重量級シナリオ・対人 UX 等の運用判断で保留した
  もので、契約項目の漏れではない（AT-LGX-001 の対人 UX 観察と相補。AT(UX) と PAI(契約/機能) は別チャネル・相互代替不可）。

## 5. 申し送り（TC[DLV] へ昇格した回帰・独立性を壊さない範囲）

決定論・ONNX 不要なケースは in-repo の TC[DLV] / unit 回帰へ昇格済み（独立性は `legixy.test` 側に温存）:
- `crates/legixy-cli/tests/cli.rs`（契約適合 E2E、外部 EXT-* の決定論ケース移植 + slug 制約）
- `crates/legixy-cli/tests/tc_bug007_cli_e2e.rs`（サブノード細粒度 R-1/R-2/R-3）
- `crates/legixy-feedback/tests/tc_feedback_semantic_loop.rs`（意味層→feedback R-4/R-5）
- `crates/legixy-embed/tests/tc_cache_cr_002.rs`（contextual retrieval R-6 + 二重 embed）
- 昇格しても **PAI（独立黒箱）は廃止しない**。両輪運用（13章 §6）。

## 6. プロセスの穴（DevProc へのフィードバック）

R-1〜R-7 が露呈した穴: **チェーンは「機能の存在」を GREEN にしたが、SPEC の「意図・成果」を未検証だった**
（例: R-1 はサブノードが materialize する＝presence を検証したが、LGX-EXT-001 目的1「トークン削減」という outcome を
測っていなかった）。詳細な root-cause と DevProc 改善提案は `docs/decisions/2026-06-16_pai-process-hole.md`。

## 改訂履歴

| 日付 | 版 | 変更 |
|---|---|---|
| 2026-06-16 | 1.0.0 | 初版。稼働中の外部スイート `legixy.test` を CLI 完成品 PAI として正式化。54件 + R-1〜R-7 の履歴・現状全 PASS・リリースゲート手順を記録。 |
