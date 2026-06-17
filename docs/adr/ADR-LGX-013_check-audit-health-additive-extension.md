Document ID: ADR-LGX-013

# ADR-LGX-013: check --audit-health を LGX-COMPAT-001 §4 #3 への加算的拡張として追加する

**ステータス**: rejected
**起票日**: 2026-06-12
**却下日**: 2026-06-12
**却下理由**: 実現性の問題により不採用。試験 INSERT + ROLLBACK によるロック競合等の不安定性が解消できないと判断。GAP-LGX-187 は "closed (dropped)" として処理する。
**対象**: ~~SPEC-LGX-004 §3 REQ.16、LGX-COMPAT-001 §4 #3~~（未適用）

## 1. 文脈（Context）

- 背景: SPEC-LGX-007.REQ.03 の注記（spec-change 2026-06-12）において、`legixy analyze` は context_log の完全性を前提とするが検証手段がないとして、完全性確認コマンドの新設を GAP-LGX-187 として分離した。
- ADR-LGX-004 の残存リスク「恒常的な書込失敗は運用で検知する」の具体的手段として、`check --audit-health` が必要と判断された。
- LGX-COMPAT-001 §4 #3（check の凍結フラグ一覧: `[--formal]`）に `--audit-health` が存在しないため、加算的拡張の手続きが必要。
- 関連: GAP-LGX-187、ADR-LGX-002（embed --node/--force 加算的拡張の先行例）

## 2. 検討した選択肢（Options）

### 選択肢 A: check の新フラグとして `--audit-health` を追加（採用）

- 概要: 既存 `check` サブコマンドに `--formal` と排他の新フラグ `--audit-health` を加える。凍結契約への加算的拡張として LGX-COMPAT-001 §4 #3 に追記。
- 利点: 既存 `check`・`check --formal` の挙動は完全に不変（後方互換）。ユーザの直感的なコマンド体系（check 系はすべて `check [--flag]`）と一致。
- 欠点: `check` ファミリが増加し help の見通しが悪くなりうる（minor）。

### 選択肢 B: 独立サブコマンド `legixy audit-health` として追加

- 概要: 新サブコマンドとして追加。
- 欠点: 19 サブコマンドを 20 に増やす（COMPAT-001 §4 サブコマンド数増加）。check 系の命名一貫性が崩れる。加算的拡張の難易度が選択肢 A より高い（サブコマンド名の凍結契約追加）。
- 不採用。

## 3. 判断（Decision）

選択肢 A を採用する。

理由:

- ADR-LGX-002（embed --node/--force）と同一の加算的拡張パターンに従う。既存呼出の互換を維持しつつ新機能を追加できる。
- `--audit-health` は `--formal`（構造検証）と同じ "check の特定モード" という意味で一貫性がある。
- `--formal` と排他（同時指定は exit 2）とすることで、G1 ゲート（`check --formal`）との意図しない混同を防止する。

## 4. 結果（Consequences）

### 期待される効果
- GAP-LGX-187 の解消。`legixy analyze` 実行前に context_log の健全性を確認する手段が提供される。
- ADR-LGX-004 の「恒常的な書込失敗は運用で検知する」が具体的なコマンドとして実装可能になる。

### 受け入れる代償
- `check` の help に `--audit-health` が追加される（既存ユーザへの視覚的変化）。

### 残存リスク
- `check --audit-health` の試験 INSERT + ROLLBACK は、ロック競合等で結果が不安定になりうる（稼働中の高負荷環境）。DD 段階でタイムアウトと再試行を設計に含めることを推奨。

## 5. 関連

- closes: GAP-LGX-187
- 先行例: ADR-LGX-002（embed --node/--force 加算的拡張）
- 対象 SPEC: SPEC-LGX-004.REQ.16
- 凍結契約更新: LGX-COMPAT-001 v1.2.0（§4 #3、§7 適用済みリスト更新）
