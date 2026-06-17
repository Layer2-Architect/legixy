Document ID: GAP-LGX-187

# GAP-LGX-187: `legixy check --audit-health` コマンドの SPEC-LGX-004 への新 REQ 追加

**親 SPEC**: SPEC-LGX-004（検証）、SPEC-LGX-007（フィードバックループ）
**観点出典**: SPEC-LGX-007.REQ.03 注記（spec-change 2026-06-12）
**ステータス**: closed (2026-06-12, dropped — 実現性の問題により不採用)
**起票日**: 2026-06-12
**起票理由**: spec-change 2026-06-12（ADR-LGX-004 可観測性強化）の決定 2 = B により、`legixy analyze` 実行前の context_log 健全性確認コマンドを別 GAP として分離
**クローズ理由**: 試験 INSERT + ROLLBACK によるロック競合等の不安定性（実現性の問題）により `check --audit-health` の実装は不要と判断。ADR-LGX-013 rejected。SPEC-LGX-004 への REQ.16 追加なし。

## 1. 観点

`legixy analyze` は context_log の完全性を前提とするが、ADR-LGX-004 のベストエフォート書込方針により欠落が生じている可能性がある（SPEC-LGX-007.REQ.03 注記）。analyze 自身は欠落検出を行わない。

人間が analyze を実行する前に「context_log のデータは信頼できるか」を確認できるコマンドが存在しない。

## 2. 現状の SPEC

- SPEC-LGX-004 の `check` / `check --formal` は トレーサビリティグラフの構造・意味検証のみを対象とする（REQ.01/REQ.02）。engine.db の稼働健全性は対象外。
- SPEC-LGX-007.REQ.03 の注記に「完全性確認コマンドの新設は GAP-LGX-187 として管理する」と参照が記載された（spec-change 2026-06-12 適用後）。

## 3. 推奨対応

SPEC-LGX-004 に新 REQ として `legixy check --audit-health` を追加する。

想定する検査内容（設計時に確定）:
- **書込可能性テスト**: context_log テーブルへの試験 INSERT + ROLLBACK（現在も書き込める状態かを確認）
- **ディスク残量チェック**: DB ファイルのある領域の空き容量を閾値と比較
- **スキーマ整合性**: context_log テーブルのスキーマが現行バージョンと一致するか
- **最終記録の新鮮さ**: 最終 context_log エントリのタイムスタンプが異常に古くないか（オプション）

出力形式・終了コード・NFR REL との関係は設計時（DD 前の SPEC REQ 追加段階）に確定する。

## 4. 影響範囲

- SPEC-LGX-004: 新 REQ 追加（バージョン bump）
- LGX-COMPAT-001: `check --audit-health` は v3 に存在しない新サブコマンド（加算的拡張として ADR 記録要）
- TP-LGX-004: 新 REQ に対応する観点追加
