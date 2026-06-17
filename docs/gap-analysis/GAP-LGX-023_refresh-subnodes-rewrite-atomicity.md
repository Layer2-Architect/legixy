Document ID: GAP-LGX-023

# GAP-LGX-023: refresh-subnodes --apply の graph.toml 書き換えの atomicity（部分書き込み耐性）が未定義

**親 TP**: TP-LGX-002
**観点出典**: TP-LGX-002 §2.6 観点 G-PS-1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**severity**: minor (theme: 永続化 / 一次データ完全性)

> **敵対的精査パス（2026-06-09）**: GENUINE と判定（refute 失敗）。一次データ graph.toml（REQ.01）の書き換え atomicity は SPEC-LGX-002 が真に沈黙しており、NFR-LGX-001.REL.01 は engine.db（WAL）限定で平文ファイルを保護しない。SEC.08 / §11 非目標は並行性の話で書き込み途中クラッシュは対象外、LGX-COMPAT-001 §4 #9 はバックアップ生成のみ規定。ただし REQ.13 が --apply 前にバックアップを必ず作成するため最悪ケースでも手動復旧可能であり、blocker/major ではなく **minor**。temp+rename を凍結明文化すれば自明にクローズ可。

## 1. 観点

`refresh-subnodes --apply` が graph.toml を新 ID へ書き換える途中で電源断・プロセス kill・ディスクフルが発生した場合、graph.toml が破損（部分書き込み）した状態で残りうる。一次データである graph.toml（REQ.01）の破損は致命的だが、書き換えの atomicity（temp ファイル + アトミック rename 等）が規定されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-002 REQ.13 は「書き換え前にバックアップ `graph.toml.refresh-bak.{unix epoch 秒}` を作成する」とし、バックアップによる事後復旧の手段は与える。しかし**書き換え自体の atomicity**（一時ファイルへ全量書き出し後にアトミック置換するのか、元ファイルへ直接上書きするのか）は未定義。直接上書き方式だと部分書き込みで graph.toml が壊れ、バックアップからの手動復旧が必要になる。

NFR-LGX-001.REL.01 は engine.db（SQLite）の電源断耐性を WAL で規定するが、graph.toml は平文ファイルでありこの保護の対象外。core-perspectives.md §永続化「部分書き込みからの回復」「保存中の電源断・プロセス kill」に直接該当する。

## 3. 期待される情報

SPEC-LGX-002 REQ.13 に追加されるべき記述:

- graph.toml 書き換えの atomicity 保証（推奨: 同一ディレクトリ上の一時ファイルへ全量書き出し → fsync → アトミック rename。これにより部分書き込みで元ファイルが壊れない）
- 書き換え失敗時の事後状態の定義（元 graph.toml が無傷で残る / バックアップで復旧可能、のいずれを保証するか）
- バックアップ作成と本体書き換えの順序・失敗時の不変条件（バックアップ未作成なら本体書き換えに進まない 等）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-001 / UC-LGX-003: 書き換え失敗時の回復手順を記述できない
- 下流 DD/TS: refresh_subnodes の永続化テスト（中断シナリオ・部分書き込み回復）の設計指針が定まらない
- 関連: GAP-LGX-021（並行アクセス）・GAP-LGX-024（バックアップ命名/保持）と同一 REQ.13 上の隣接論点

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-002 v0.4.1（人間承認 2026-06-10）: REQ.13 を atomicity 規定に拡張（バックアップ失敗時非書換・同一ディレクトリ temp+fsync+rename・失敗時 exit 1 で元ファイル無傷・順序不変条件）。v3 実測で直接上書き方式（refresh_subnodes.rs:357 std::fs::write）を確認し【v3 差分】を注記。CLI 引数・バックアップ命名は不変。

## 6. 関連 ADR

書き換え戦略（temp + rename）は実装方式の選択を含むため ADR 候補となりうるが、v3 実測（`crates/te-cli/src/commands/refresh_subnodes.rs:285-360`）の挙動を凍結する形で SPEC 明文化のみで解決する可能性もある:

- ADR-LGX-NNN: graph.toml 書き換えの atomicity 戦略（任意）
