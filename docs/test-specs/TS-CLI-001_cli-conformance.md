Document ID: TS-CLI-001

# TS-CLI-001: legixy CLI 契約適合テスト仕様

> 配送軸のテスト仕様（DLV-CLI-001 の子、TC-CLI-001 の親）。CTR-CLI-001 の契約項目を**実バイナリ E2E**の
> 検証ケースへ翻訳する（DevProc_V4.1 §12 §6: P-1 実バイナリ + P-3 契約項目↔TC mapping）。

**親 DLV**: DLV-CLI-001
**対象契約**: CTR-CLI-001
**実装テスト**: TC-CLI-001（`crates/legixy-cli/tests/cli.rs`、`std::process::Command` で実バイナリ起動）

## 1. 検証方針

- **実バイナリ E2E**: `cargo build` 後、`CARGO_BIN_EXE_legixy` を spawn し、契約のサブコマンド名・位置引数・
  フラグ・既定値・**終了コード**・グローバルオプション・設定探索を検証（ユニットでなく）。
- **契約項目↔ケース mapping**: CTR-CLI-001 §3 チェックリストの各項目に ≥1 ケース。未カバーは RED。

## 2. ケース群（CTR-CLI-001 §3 と対応）

| 契約項目 | 検証ケース | 現状 |
|---|---|---|
| 19 サブコマンドサーフェス | `full_surface_declares_19_subcommands` | GREEN |
| 位置引数（context/observe 等） | context/observe/feedback 各ケース | GREEN |
| check 終了コード | `check_formal_*`, chain break→exit1 | GREEN |
| 使用法誤り exit 2 | `*_exit_2`（bogus/bad flag/required） | GREEN |
| グローバル `--json`（全コマンド）| **未実装ケース** | **RED（BUG-001）** |
| グローバル `--models-dir` | **未実装ケース** | **RED（BUG-002）** |
| 設定ファイル探索 | **未実装ケース** | **RED（BUG-003）** |
| check 形式層 5 カテゴリ | FileExistence/OrphanFile/SubnodeIdFormat | **RED（BUG-006）** |
| drift（非 NaN） | drift 正常系 | **RED（BUG-004）** |
| 排他 exit 2 | embed/refresh 排他 | **RED（BUG-008/009）** |

## 3. RED ケースの扱い

RED 項目は配送層の **TC[RED]**。`/defect-fix`（BUG-001〜010）で SRC（main.rs + 各ライブラリ）を修正し GREEN 化する。
TC-CLI-001 に RED ケースを追加 → 修正 → GREEN の順（TC[RED]→SRC[GREEN] 規律、DevProc_V4.1 §12 §6）。

## 4. 委譲

MCP サーフェスの適合は TS-MCP-001。ドメインロジックの正しさは機能軸 TS-LGX-001〜013。本 TS は
「実バイナリが LGX-COMPAT-001 §3-4 を満たすか」に集中する。
