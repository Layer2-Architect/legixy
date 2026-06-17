# specs-supplement: SPEC 実装補完情報

| 項目 | 内容 |
|------|------|
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |
| 対象 | docs/specs/ の SPEC-LGX-001〜010（いずれも Approved 版） |

本ディレクトリは、docs/specs/ の仕様を実装するにあたって**不足している情報**を SPEC 本文に手を入れずに補完するための参考資料集である。SPEC 本文の変更には人間承認が必要（SPEC-LGX-001 §7.1）であり、本ディレクトリの記載はいかなる意味でも SPEC を上書きしない。SPEC と矛盾する記載を発見した場合は SPEC が優先する。

各文書の記法:
- **[補完]** — 不足情報を旧文書（legixy.old.p1/docs）・旧実装（traceability-engine.v3.chg_to_lexigy）から特定し、根拠パス付きで補完したもの。
- **[要決定]** — 情報がどこにも存在しない、または SPEC と旧実装が矛盾しており、人間の裁定（場合により SPEC 改訂＝人間承認）が必要なもの。

## 文書一覧

| 文書 | 対象 | [補完] | [要決定] |
|------|------|:---:|:---:|
| [SUPP-LGX-000_参照文書インベントリ.md](SUPP-LGX-000_参照文書インベントリ.md) | 全 SPEC の外部参照 108 件の所在対応表・持ち込み優先順位 | - | - |
| [SUPP-LGX-001_legixy-全体要求.md](SUPP-LGX-001_legixy-全体要求.md) | SPEC-LGX-001 | 2 | 4 |
| [SUPP-LGX-002_グラフ基盤.md](SUPP-LGX-002_グラフ基盤.md) | SPEC-LGX-002 | 18 | 7 |
| [SUPP-LGX-003_コンテキスト解決.md](SUPP-LGX-003_コンテキスト解決.md) | SPEC-LGX-003 | 12 | 9 |
| [SUPP-LGX-004_検証.md](SUPP-LGX-004_検証.md) | SPEC-LGX-004 | 14 | 11 |
| [SUPP-LGX-005_グラフ走査.md](SUPP-LGX-005_グラフ走査.md) | SPEC-LGX-005 | 13 | 7 |
| [SUPP-LGX-006_embeddingとドリフト検出.md](SUPP-LGX-006_embeddingとドリフト検出.md) | SPEC-LGX-006 | 17 | 13 |
| [SUPP-LGX-007_フィードバックループ.md](SUPP-LGX-007_フィードバックループ.md) | SPEC-LGX-007 | 11 | 9 |
| [SUPP-LGX-008_マイグレーション.md](SUPP-LGX-008_マイグレーション.md) | SPEC-LGX-008 | 13 | 10 |
| [SUPP-LGX-009_MCPサーバ.md](SUPP-LGX-009_MCPサーバ.md) | SPEC-LGX-009 | 16 | 6 |
| [SUPP-LGX-010_embedding運用と監査.md](SUPP-LGX-010_embedding運用と監査.md) | SPEC-LGX-010 | 26 | 10 |

## 横断的な重要発見（実装着手前に裁定が必要なもの）

1. **参照文書がリポジトリに存在しない。** 新リポジトリには SPEC 10 ファイルしかなく、正準定義（不変条件・UC・NFR・CLI 互換契約・CLAUDE.md 規範）はすべて legixy.old.p1 側にある。必須 7 群の持ち込みが実装の前提（SUPP-LGX-000 §3）。
2. **VAL-LGX-001 / TS-LGX-NNN / DD-LGX-NNN は LGX 名義の実体がどこにも無い。** 前身（VAL-LX-001 / TS-LX-NNN / DD-LX-NNN）のみ存在。リブランド時に参照 ID だけ書き換えられたと推定され、各 SPEC の「検証方法」欄の根拠が宙に浮いている。
3. **DB のパス・ファイル名の系統不一致。** SPEC 系は `.legixy/engine.db`、v3 実装と互換契約は `.trace-engine/`、実在 v0.1.0 プロジェクトの DB 実体は `feedback.db`。CLI 互換制約（traceability-engine.v3 と実行時引数互換）に直結する（SUPP-LGX-007 / 008 / 010）。
4. **SPEC と v3 実装の挙動乖離が多数。** 字義どおり実装すると v3 観測挙動が変わるもの（context のセクション構成・subnode 整列順、investigate の drift 配線、check の --log-format、migrate の実在データ拒絶など）が各 SUPP に【要決定】として列挙されている。乖離の裁定（v3 準拠か SPEC 準拠か）は加算的拡張規律により人間承認＋ADR の対象。
5. **環境変数・配置の未確定。** MCP のバイナリ解決変数が三系統（TRACEABILITY_ENGINE_BIN / LEXIGY_BIN / legixy）、embedding モデル実体の入手・配布手順が未規定。
6. **新リポジトリの `docs/DevPorc` は綴り誤りのシンボリックリンク**（正: DevProc）。旧 CLAUDE.md の参照パスとも不一致。
