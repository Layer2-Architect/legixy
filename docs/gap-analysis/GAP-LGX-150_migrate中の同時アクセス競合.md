Document ID: GAP-LGX-150

# GAP-LGX-150: migrate 実行中に他コマンドが同一データへアクセスした場合の競合方針が未定義

**親 TP**: TP-LGX-008
**観点出典**: TP-LGX-008 §2.4 観点 C-1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

migrate が engine.db / graph.toml を書き換えている最中に、別プロセスの legixy コマンド（check, context 等）が同じ engine.db / graph.toml を読みに来た場合の競合方針が未定義。SQLite WAL 前提（PERF.07）はあるが、graph.toml は SQLite 外のファイルであり、書き換え中の中間状態を他コマンドが読む恐れがある。

## 2. 現状の SPEC / UC

SPEC-LGX-008 §3 REQ.02 は中断耐性に触れ、NFR-LGX-001.PERF.07 は engine.db の WAL モードを規定するが、**migrate 中の engine.db / graph.toml への同時アクセスの隔離（ロック・排他・読取の一貫性）** は SPEC に未定義。STATE-INV-1（ステートレス）は通常運用の前提であり migrate 中の一時操作の並行制御は別問題。

## 3. 期待される情報

SPEC または UC に追加されるべき記述:

- migrate 中の engine.db アクセスを WAL の読取一貫性に委ねるか、明示ロックで他コマンドを待たせる/拒否するか
- graph.toml の書き換え中に他コマンドが中間状態を読まない保証（atomic 書込 = GAP-LGX-152 で被覆されるか明示）
- 同時アクセス検出時のユーザ向けメッセージ

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流 TS / TC: 並行アクセステストの期待値が書けない
- 他の GAP との依存: GAP-LGX-151（二重 migrate）, GAP-LGX-152（atomic 書込）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-008 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.02 に並行アクセス方針を確定 — engine.db は WAL 読取一貫性、graph.toml は atomic rename（常に完全な旧版か新版のみ観測）、明示排他なし（SEC.08 リスク受容）。ADR-LGX-011。

## 6. 関連 ADR

該当時に起票。
