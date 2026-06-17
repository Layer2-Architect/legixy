Document ID: DLV-CLI-001

# DLV-CLI-001: legixy CLI dispatch 設計

> 配送軸の設計層（CTR-CLI-001 の子）。境界契約の各サブコマンドを機能軸ライブラリ SRC への dispatch に
> マップする。各 UC の DD §8（CLI dispatch / 終了コード記述）の集約点。

**親 CTR**: CTR-CLI-001
**area**: CLI
**サーフェス source（SRC-CLI-001 anchor）**: `crates/legixy-cli/src/main.rs`（+ `refresh.rs`）

## 1. dispatch マッピング（契約サブコマンド → 機能 SRC）

| サブコマンド | 引数変換 | 呼出す機能 API（area=LGX の SRC） | 終了コード |
|---|---|---|---|
| check | --formal→CheckMode | `legixy_check::run/exit_code`（SRC-LGX-001） | Error>0→1 |
| context | files→PathBuf, sections カンマ分割 | `legixy_ctx::ContextCompiler`（SRC-LGX-002） | ResultTooLarge→1 |
| impact/investigate | start, --max-depth | `legixy_nav`（SRC-LGX-005/006） | 0 |
| embed/drift/report/calibrate/snapshot | NodeFilter/AgainstSpec/BucketCount | `legixy_embed`（SRC-LGX-007/010-013） | 0/1 |
| observe/feedback/analyze/proposals/approve/reject/audit | NewObservation/ProposalStatus | `legixy_feedback`（SRC-LGX-008） | exit 1/2 |
| init/migrate | force/MigrateOpts | `legixy_mig`（SRC-LGX-009） | 1/2 |
| refresh-subnodes | dry-run/apply | `refresh.rs`（ADR-LGX-023） | 1（排他は要 exit2 化、BUG-009）|

## 2. グローバル処理（契約 §3、DLV 責務）

- `--project-root`（global、既定 `.`）。engine.db パス解決は ADR-LGX-015（`.legixy/engine.db` 正準 + `.trace-engine/` 読取 fallback）。
- **要実装（RED）**: `--json` グローバル（BUG-001）/ `--models-dir` グローバル（BUG-002）/ 設定ファイル読込→Config 反映（BUG-003）。
- 引数パーサ = clap。使用法誤り = exit 2（排他は clap ArgGroup へ移し exit 2 化＝BUG-008/009）。

## 3. 既知の dispatch ギャップ（→ TC-CLI-001 が RED で束縛、/defect-fix 対象）

BUG-001（--json）/ 002（--models-dir）/ 003（config）/ 004（drift NaN+cwd）/ 005（check 意味層 None）/
006（check 5カテゴリ中2）/ 007（context subnode）/ 008/009（排他・exit）/ 010（空 snapshot）。詳細は
`legixy.test/docs/defect-root-cause-2026-06-14.md`。

## 4. 非対象

ドメインロジックは機能軸（UC→…→SRC）の責務。DLV は配送（公開・変換・dispatch・終了コード）のみ。
