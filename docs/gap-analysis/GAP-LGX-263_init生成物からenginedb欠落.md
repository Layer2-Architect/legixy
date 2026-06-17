Document ID: GAP-LGX-263

# GAP-LGX-263: UC init フロー生成物一覧から engine.db が欠落

**親 TP**: TP-LGX-019
**観点**: §2.1 BF5
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-009 init Step 2 の「以下を生成する」5 項目が SPEC-LGX-008.REQ.07 の必須生成物と一致しているか。具体的には `engine.db`（初期スキーマ DB）が UC の列挙から欠落していないか。

## 2. 現状の UC / SPEC

UC-LGX-009 の init 基本フロー Step 2 は以下の 5 項目を列挙する:

1. `.legixy.toml`（legixy テンプレート、`[graph]` セクション含む）
2. `docs/traceability/graph.toml`（サンプルノード/エッジ付き）
3. 各成果物タイプのディレクトリ
4. `.legixy/` ディレクトリ（.gitignore 付き）
5. `docs/traceability/matrix.md`（空テンプレート）

SPEC-LGX-008.REQ.07 は以下を必須生成物として規定する:
- `.legixy.toml` のテンプレートを生成（ICONIX 8 typecode + `[id.document_id]`）
- `docs/traceability/graph.toml` の空ファイルを生成
- **`.legixy/engine.db` を初期スキーマで作成**
- ICONIX 成果物用 8 ディレクトリ + `.gitkeep` を作成

UC の列挙項目 4「`.legixy/` ディレクトリ（.gitignore 付き）」には engine.db の生成が含意されているかもしれないが、UC フロー記述として **engine.db の作成が観察可能でない**。

§4 の FB-INV-4「DB 不在時も上流は正常返却」と init での engine.db 作成の意図（TP-LGX-008 L-2 で確立）が UC 事後条件から見えない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
init Step 2 の列挙に以下を追加する:
- `.legixy/engine.db`（初期スキーマで作成）

あわせて項目 4 を「`.legixy/` ディレクトリ（.gitignore 付き）」のままとし、その配下に engine.db が含まれることを注記する。

**(B) drop（委譲容認）案:**
「`.legixy/` ディレクトリ（.gitignore 付き）」が engine.db を内包することを自明とし、SPEC-LGX-008.REQ.07 + TP-LGX-008 L-2 へ委譲確定とする。UC の列挙は代表的生成物にとどめる。

## 4. 影響範囲

- 下流 RBA/SEQA: init のロバストネス図で engine.db 作成が entity として現れるか
- 下流 TS-LGX-007（init テスト）: `check --formal` が init 直後に通ることの前提として engine.db 存在が必要
- UC 事後条件「有効な legixy プロジェクト構造が作成される」の完全性

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
