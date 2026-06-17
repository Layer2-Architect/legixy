Document ID: ADR-LGX-020

# ADR-LGX-020: Rust ワークスペースの crate 分割と共有型の境界凍結（DD 基盤）

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定。HR7 境界 API 凍結、SUPP-LGX-000/001 [要決定]2-1）
**対象**: DD フェーズ全体（DD-LGX-001〜013 が参照する crate 境界と共有型の凍結）

## 1. 文脈（Context）

DD（詳細設計・言語確定）は ICONIX 二段化の言語非依存→言語固有の分岐点。RBD/SEQD のクラス・操作を実装言語（Rust CLI + TypeScript MCP）へ mapping するにあたり、**crate 分割は境界 API の凍結対象**（HR7）であり、SUPP-LGX-001 [要決定]2-1 が「正準リストを DD で凍結、凍結は人間承認」と規定していた。

DD は per-UC（DD-LGX-001〜013、SEQD と 1:1 chain）で編成する（人間裁定 2026-06-13）。一方、crate と共有型は UC を跨いで共有される（例: 有向グラフ は legixy-graph に 1 回定義され多数の UC が利用）。本 ADR は、per-UC DD 群が共通参照する **crate 境界と共有型を一括凍結**し、各 DD §4/§9 から参照される foundational decision として機能する。

## 2. 判断（Decision）

### 2.1 crate 分割（v3 実績の 10-crate を踏襲、凍結）

v3（traceability-engine.v3、te-*/lx-* 構成）の 10-crate を legixy 命名で踏襲する（LGX-COMPAT-001 §4.2）。**この crate 境界と crate 間公開 API を凍結する**（HR7。追加は許容、削除・改名・シグネチャ変更は次版 SPEC 改訂扱い）。

| crate | 責務 | 主に対応する UC / SPEC | 所有する主要共有型（概念） |
|---|---|---|---|
| **legixy-core** | 共通型・エラー階層・識別子・不変条件の土台 | 全 SPEC | 識別子、文書識別子、area/type コード、共通エラー階層、severity/区分の基底 |
| **legixy-graph** | 有向グラフ・ノード・エッジ・サブノード・DAG・グラフ構築/走査基盤 | SPEC-002, UC-001/003/005/006 | 有向グラフ、ノード、エッジ、サブノード、未解決エッジ |
| **legixy-db** | engine.db スキーマ・接続・永続層 | SPEC-007/008/010, FB/SCORE/STATE-INV | DB 接続、embeddings/observations/proposals/snapshots テーブル表現 |
| **legixy-ctx** | compile_context・コンテキスト解決・粒度制御・キャッシュ整列 | SPEC-003, UC-002/004 | コンテキスト結果、上流成果物、粒度種別、セクション整列 |
| **legixy-check** | check・検証カテゴリ・severity・finding 生成 | SPEC-004, UC-001 | 検証所見、検証報告、検証カテゴリ、終了状態 |
| **legixy-nav** | impact/investigate・グラフ走査・BFS・ドリフト走査 | SPEC-005, UC-005/006 | 走査結果、深度マップ、疑わしいノード、打ち切り情報 |
| **legixy-embed** | embedding 生成・ONNX・drift・bulk similarity・report/calibrate/snapshot 基盤 | SPEC-006/010, UC-007/010/011/012/013 | 意味ベクトル、ドリフト所見、類似度スコア集合、ヒストグラム、スナップショット |
| **legixy-feedback** | observation/proposal・analyze・approve/reject・状態モデル | SPEC-007, UC-008 | observation、proposal、状態種別（pending/analyzing/resolved/skipped・pending/approved/rejected） |
| **legixy-mig** | init/migrate・v0.1.0 変換 | SPEC-008, UC-009 | 移行レポート、ID マップ、マトリクス表現 |
| **legixy-cli** | CLI 統合バイナリ・引数パース・サブコマンドディスパッチ・終了コード | LGX-COMPAT-001 全 19 サブコマンド | CLI 引数構造、終了コード規約 |

### 2.2 TypeScript MCP（ts-mcp、凍結）

`ts-mcp`（TypeScript）は Agent Surface として **3 ツールのみ**（`compile_context` / `observe` / `get_compile_audit`、MCP-INV-1）を公開し、各ツールは引数を legixy-cli へ忠実転送する（MCP-INV-2）。MCP の TS 設計詳細は当該ツール起点の UC（UC-002 compile_context / UC-008 observe・get_compile_audit）の DD で扱う。バイナリ解決・タイムアウト・`_meta` 付与は ADR-LGX-016（env）/ SPEC-LGX-009 / ADR-LGX-004 に従う。

### 2.3 共有型の所有と凍結

- 共有型は **所有 crate（core/graph/db）で 1 回だけ定義**し、command crate（ctx/check/nav/embed/feedback/mig/cli）はそれを参照する。per-UC DD は §4 で自 UC が touch する crate を明示し、共有型は所有 crate の DD（または本 ADR）を参照して再定義しない。
- crate 間の依存は DAG（legixy-core ← 全 crate / legixy-graph ← ctx/check/nav / legixy-db ← embed/feedback/mig / legixy-cli ← 全 command crate）。循環禁止。
- 言語固有の具体型（整数幅・文字列型・`Result<T,E>`・`Vec<T>`・所有戦略・async 機構）は各 DD で確定する（本 ADR は crate 境界と概念的共有型まで）。

## 3. 結果（Consequences）

- DD-LGX-001〜013（per-UC、1:1 chain）が本 ADR の crate 境界・共有型を参照し、重複定義を避ける。
- crate 境界・crate 間公開 API は凍結（HR7）。LGX-COMPAT-001（CLI/MCP 引数）は既凍結境界として上位。
- SUPP-LGX-001 [要決定]2-1（crate 分割の正準リスト）を解消。
- **残存**: crate 内 module 分割・関数シグネチャ・型の具体は各 DD で確定（本 ADR の下で）。v3 実装（traceability-engine.v3）はシグネチャ参照の底本（引数互換のため）。

## 4. 関連

- 凍結境界: LGX-COMPAT-001 §4.2（crate 命名）, §4（19 サブコマンド・MCP 3 ツール）
- 統治: HR7（境界 API 凍結は人間承認）, ADR-LGX-014（SPEC 準拠原則）
- 連動: ADR-LGX-016（env: バイナリ解決・モデルディレクトリ）, ADR-LGX-015（DB パス）
- トリアージ: SUPP-LGX-001 [要決定]2-1
- 後続: DD-LGX-001〜013 が本 ADR を §9 で参照
