Document ID: ADR-LGX-011

# ADR-LGX-011: migrate の並行排他は明示機構を設けず SEC.08 リスク受容とする

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（weak GAP fix 一括承認 2026-06-10）
**対象**: SPEC-LGX-008 §3 REQ.02、NFR-LGX-001.SEC.08 / PERF.07 / REL.07

## 1. 文脈（Context）

- 背景: GAP-LGX-150（migrate 中の同時アクセス競合）・GAP-LGX-151（二重 migrate / auto 重複起動の排他）。migrate は engine.db と graph.toml をまたいで書き換える一時操作であり、並行する読取・二重実行への方針が未確定だった。
- 制約: 単独開発者前提（NFR SEC.08）。graph.toml は SQLite 外の平文ファイル。

## 2. 検討した選択肢（Options）

### 選択肢 A: 既存機構への委譲 + 明示排他なし（採用）

- 読取の一貫性: engine.db = SQLite WAL の読取一貫性、graph.toml = atomic rename（GAP-LGX-152 適用済 — 読み手は常に完全な旧版か新版のみ観測）。
- 二重実行: SQLite 書込ロックが事実上の排他となり、敗者は busy_timeout（REL.07）超過で Error exit 1。
- ロックファイル・PID ロック等の専用機構は導入しない。

### 選択肢 B: 専用ロックファイル導入

- 利点: 二重実行を確実に検出・即時拒否。欠点: stale lock の回収（クラッシュ後の残留ロック）という新しい故障モードを持ち込む。単独開発者環境で二重 migrate が起きる頻度に対し複雑性が見合わない。

## 3. 判断（Decision）

選択肢 A を採用する。

理由:

- 必要な安全性（中間状態を読ませない・二重書込で壊れない）は WAL + atomic rename + SQLite ロックの既存機構で既に達成されており、追加機構は故障モードを増やすだけ。
- ADR-LGX-006（人間のみ CLI の宣言的強制）と同じ SEC.08 リスク受容の枠組みであり、脅威モデルに対する一貫した判断。

## 4. 結果（Consequences）

### 期待される効果
- 実装の単純性維持。クラッシュ後に手動でロック掃除する運用が不要。

### 受け入れる代償
- 二重実行の検出が即時でなく busy_timeout（5 秒）後になる。エラーメッセージで並行実行の可能性を示唆して補う。

### 残存リスク
- マルチユーザ・CI 並列環境への展開時は本 ADR を supersede し専用排他を再検討（トリガ: SEC.08 改訂 — ADR-LGX-006 と同一トリガ）。

## 5. 関連

- closes: GAP-LGX-150, GAP-LGX-151
- 同型: ADR-LGX-006（SEC.08 リスク受容）。依存: GAP-LGX-152（atomic rename）
