Document ID: ADR-LGX-017

# ADR-LGX-017: プロセス文書パスの正準化 — `docs/DevProc_V4/` 正準、綴り誤り `docs/DevPorc` 重複 symlink を除去

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定、TRIAGE-2026-06-13 クラスタ E）
**対象**: SUPP-LGX-000 §4.3 / SUPP-LGX-001（DevPorc 綴り誤り）

## 1. 文脈（Context）

新リポジトリは DevProc_V4.1 一式へのシンボリックリンクを 2 つ持つ:

- `docs/DevProc_V4` → `DevProc_V4.1`（**正しい綴り、健在**）
- `docs/DevPorc` → 同ターゲット（**綴り誤り `DevPorc`、開発者作成の重複**）

CLAUDE.md・各プロセス文書の参照は `docs/DevProc_V4/` を使用する。綴り誤りの `docs/DevPorc` は同一ターゲットへの重複であり、参照されない。作業ツリーでは既に削除済み（`git status`: `D docs/DevPorc`）。

## 2. 判断（Decision）

人間裁定 2026-06-13:

- **正準パス**: プロセス文書への参照は `docs/DevProc_V4/`（正しい綴り）に統一する。
- **綴り誤り重複の除去**: `docs/DevPorc`（綴り誤り symlink）を除去する（作業ツリーの削除を確定）。同一ターゲットへの重複のため機能的影響はない。
- 旧 CLAUDE.md 等が `docs/DevPorc` を参照していた場合は `docs/DevProc_V4` へ修正する（現行 CLAUDE.md は既に `docs/DevProc_V4/` を参照）。

## 3. 結果（Consequences）

- パス参照の単一化。SUPP-000 §4.3 / SUPP-001 の DevPorc 綴り誤り [要決定] を解消。
- 機能的影響なし（重複 symlink の整理のみ）。

## 4. 関連

- トリアージ: TRIAGE-2026-06-13 §3 クラスタ E
- 関連: ADR-LGX-014（統治、本件は housekeeping）
