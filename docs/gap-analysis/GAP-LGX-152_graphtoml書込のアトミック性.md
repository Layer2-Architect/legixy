Document ID: GAP-LGX-152

# GAP-LGX-152: graph.toml / id-map の書込みアトミック性が未定義

**親 TP**: TP-LGX-008
**観点出典**: TP-LGX-008 §2.5 観点 P-2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

REQ.02 の非破壊性・中断耐性は SQLite トランザクション（engine.db）を主眼に置くが、`graph.toml` と `.legixy/migration-id-map.toml` は SQLite 外の通常ファイルである。これらの書込み中に電源断/kill が起きた際に **半端な（途中まで書かれた）graph.toml が残らない atomic 書込（temp file + rename 等）** が保証されるかが未定義。

## 2. 現状の SPEC / UC

SPEC-LGX-008 §3 REQ.02 で **「途中中断されても DB が壊れない（SQLite トランザクション使用）」** に触れているが、これは engine.db 文脈であり、**graph.toml / id-map といった非 DB ファイルの atomic 書込** は未定義。STATE-INV-2（graph.toml は Git 経由）は変更の追跡を述べるが書込み原子性とは別。

## 3. 期待される情報

SPEC または UC に追加されるべき記述:

- graph.toml / id-map の atomic 書込方式（temp file へ書いてから rename、fsync 含む）の明示
- 書込み途中中断時に旧 graph.toml（あれば）が温存され、半端ファイルが残らない保証
- engine.db のトランザクションと graph.toml 書込みの完了順序（DB コミット後に graph.toml 確定 等）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流 TS / TC: 電源断/中断シミュレーションでの graph.toml 健全性テストの期待値が書けない
- CTX-INV-2（グラフ整合性）: 半端 graph.toml は不変条件違反
- 他の GAP との依存: GAP-LGX-145（書込不能）, GAP-LGX-148（中断再開）, GAP-LGX-150（同時アクセス）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-008 v0.6.0（人間承認 2026-06-10）: REQ.02 に非 DB ファイルの temp(.tmp.{epoch})+fsync+rename atomic 書込と engine.db コミット先行順序を確定。SPEC-LGX-002.REQ.13/GAP-023 と方式統一。

## 6. 関連 ADR

該当時に起票。
