# SUPP-LGX-001: legixy 全体要求 実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-001 |
| 対象 SPEC | SPEC-LGX-001 v0.7.1（2026-06-10, Approved） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> 本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。

---

## 1. 未解決参照

SPEC-LGX-001 が直接参照するが、新リポジトリ（legixy）に存在しない文書。所在の全数調査は SUPP-LGX-000_参照文書インベントリ.md を参照。

| 参照 ID | 参照箇所 | 用途 | 所在 |
|---------|---------|------|------|
| LEGIXY-SPEC-001（`docs/legixy_foundational_spec.md`） | §2, REQ.01/03/06/08 | 全体概要・エンジン機能・不変条件 17 件の正準定義 | legixy.old.p1/docs/legixy_foundational_spec.md |
| LGX-EXT-001（`docs/legixy_subnode_spec_v0.2.1.md`） | §2, REQ.04/05/06/07/08 | サブノード仕様・SUBNODE-INV-1〜6 の定義 | legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md |
| LGX-EXT-002 | §4.1/4.2（CACHE-INV-1〜4） | Prompt Caching + MCP Result Persistence 仕様 | legixy.old.p1/docs/legixy_cache_spec_v0_1_0.md |
| LGX-COMPAT-001 | REQ.03（crate 写像の初期候補 §2） | CLI 互換契約・10 crate 写像 | legixy.old.p1/docs/legixy_cli_compat_reference.md |
| NFR-LGX-001 | ヘッダ表, §1.2 | 非機能要件の数値目標 | legixy.old.p1/docs/nfr/ |
| UC-LGX-001〜011 | ヘッダ表, REQ.02 | 機能カテゴリの網羅検証の母数 | legixy.old.p1/docs/usecases/ |
| CLAUDE.md「絶対ルール5」「ハードルール1」 | REQ.08, §7.1 | Surface 分離・SPEC 変更承認の根拠規範 | legixy.old.p1/CLAUDE.md（新リポジトリに CLAUDE.md 自体が無い） |
| QSET-LGX-001 / SPP-LGX-001 / VAL-LGX-001 | REQ.03 根拠, §6, GAP 注記 | 設計判断の経緯 | legixy.old.p1/docs/（VAL-LGX-001 は LGX 名義の実体が無く、前身 VAL-LX-001 のみ — SUPP-LGX-000 §2 参照） |

## 2. 実装に必要だが未規定の事項

### 2-1. [要決定] crate 分割の正準リスト（REQ.03）
SPEC は crate 名を「例示」とし正準リストを DD 段階で凍結するとする。DD は未作成。
- 初期候補: LGX-COMPAT-001 §2 の 10 crate 写像。
- 旧実装の実績: traceability-engine.v3.chg_to_lexigy/crates/ の 10 crate（lx-core, lx-graph, lx-db, lx-cli, lx-ctx, lx-check, lx-embed, lx-feedback, lx-nav, lx-mig）。
- 推奨: 旧実装の lx-* 10 crate 構成をそのまま初期凍結案として DD に採用（互換制約・移植容易性の両面で最有力）。凍結は人間承認で。

### 2-2. [補完] CLI 互換制約の具体化（REQ.08 Admin Surface）
記憶事項・LGX-COMPAT-001 より: legixy は traceability-engine.v3 バイナリと**実行時引数互換**を維持しなければならない。REQ.08 のコマンド列挙（check, feedback, …）の正確な引数仕様は LGX-COMPAT-001（最新 v1.1.0、legixy.old.p1/docs/legixy_cli_compat_reference.md）が正準。SPEC 本文には引数レベルの記述が無いため、実装時は同文書の持ち込みが必須。

### 2-3. [要決定] 「Agent Surface = MCP 3 ツール」の検証手段（REQ.08）
検証方法は「MCP サーバが 3 ツールのみ公開していること」だが、検証の自動化手段（tools/list 応答のスナップショットテスト等)は TS/TC 層未作成のため未定義。SUPP-LGX-009 §2 の TS-LGX 系不在と同根。

### 2-4. [要決定] §4.1 マトリクス整合の自動検証
§4.3 は「自動検証ツールは将来課題」と明記。実装フェーズで specs ディレクトリの整合 lint を作るかは人間判断（DevProc 側ツールとの役割分担も含む）。

### 2-5. [補完] UC-LGX-012/013 予約の扱い
REQ.02 予約注記のとおり、UC-LGX-012/013 は**未生成が正**。実装時に snapshot/drift 系フロー（SPEC-LGX-010）の UC 参照が必要になっても、新規 UC を AI 判断で生成してはならない（SPP-LGX-001 次反復で人間が実施）。

## 3. 用語・前提の補完

- **v0.1.0**: 前身 traceability-engine の matrix.md 主体の版。移行元データ形式の実体は SUPP-LGX-008 §2 参照（SPEC の「engine.db」と実在 DB 名 feedback.db の不一致あり）。
- **Surface**: Admin（CLI）/ Agent（MCP）の操作面の分離概念。定義の正準は LEGIXY-SPEC-001 §2。
- **前段ループ / FCR / SPP / QSET / GAP / TP**: DevProc_V4.1 の工程用語。新リポジトリでは docs/DevPorc → DevProc_V4.1 のシンボリックリンクで参照可能だが、**ディレクトリ名が `DevPorc`（綴り誤り）**で旧 CLAUDE.md の参照パス `docs/DevProc_V4/` と不一致（SUPP-LGX-000 §4 参照、要修正）。

## 4. 旧実装からの参考情報

- 旧実装一式: traceability-engine.v3.chg_to_lexigy/（Rust workspace + ts-mcp + deploy/manual.md）。
- 旧プロジェクト文書一式: legixy.old.p1/docs/（specs はリブランド後と内容完全一致、加えて DD・TS・UC・ADR・NFR・GAP 等の全工程文書あり）。
- 持ち込み優先順位は SUPP-LGX-000 §3 を参照（必須 7 群: LGX-COMPAT-001 / LEGIXY-SPEC-001 / LGX-EXT-001 / LGX-EXT-002 / NFR-LGX-001 / CLAUDE.md / UC-LGX-001〜011）。
