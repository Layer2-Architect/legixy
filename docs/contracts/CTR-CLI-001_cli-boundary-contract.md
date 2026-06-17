Document ID: CTR-CLI-001

# CTR-CLI-001: legixy CLI 境界契約（配送サーフェス = `legixy` バイナリ）

> 配送軸（DevProc_V4.1 §12）のチェーン根。`legixy` 実行ファイルが外部公開する引数規約・終了コード・
> グローバル仕様・設定探索を**凍結**する（ハードルール 7）。本契約の正準根拠は `docs/legixy_cli_compat_reference.md`
> （LGX-COMPAT-001 v1.1.0）§3・§4。本ノードはそれを配送チェーンの根として graph 編入したもの。

**サーフェス種別**: CLI バイナリ（`legixy`）
**area**: CLI
**凍結状態**: frozen（HR7、LGX-COMPAT-001 は既に凍結境界）
**正準根拠**: LGX-COMPAT-001 §3（グローバル）, §4（19 サブコマンド）
**dispatch 設計**: DLV-CLI-001
**適合テスト**: TC-CLI-001（`crates/legixy-cli/tests/cli.rs`、実バイナリ E2E）

## 1. 公開サーフェス（19 サブコマンド、LGX-COMPAT-001 §4）

`init` / `migrate` / `check` / `embed` / `drift` / `report` / `calibrate` / `snapshot` / `refresh-subnodes` /
`context` / `impact` / `investigate` / `feedback` / `observe` / `audit` / `analyze` / `proposals` / `approve` / `reject`。

位置引数・フラグ・既定値は LGX-COMPAT-001 §4 の表に凍結。

## 2. グローバル規約（LGX-COMPAT-001 §3）

- グローバルオプション（**全コマンド共通**）: `--project-root <PATH>`（既定 `.`）, `--json`（flag）, `--models-dir <PATH>`。
- 終了コード規約: 使用法誤り（パーサ層）= **exit 2** / 受理済み値の意味的不正・実行時失敗 = **exit 1** / 検証結果（check Error>0）= 各規定。
- 設定探索: `.legixy.toml` 既定 + `.trace-engine.toml` 旧名フォールバック（SPEC-LGX-008.REQ.13）。

## 3. 適合チェックリスト（→ TC-CLI-001 に全数 mapping、P-3）

> 各項目に対応する TC[DLV] ケースが ≥1 件必要。**Phase B（2026-06-14）で BUG-001〜010 を `/defect-fix` で GREEN 化**し、
> 全項目が TC-CLI-001（実バイナリ E2E、55 ケース）でカバー済み。trace-check `[6/6]` 契約適合ゲートで CI 強制。
> 広い意味/ONNX シナリオは外部 `legixy.test`（独立検証チャネル、00-philosophy §2.4）に温存。

| # | 契約項目（LGX-COMPAT-001 §7） | TC-CLI ケース | 状態 |
|---|---|---|---|
| 1 | 19 サブコマンド名・別名を維持 | `full_surface_declares_19_subcommands` | ✅ GREEN |
| 2 | 位置引数の順序・個数（observe/drift/context） | observe/context/drift 各ケース | ✅ GREEN |
| 3 | フラグ名・既定値（config 由来） | `config_*`（chain order/閾値を config から解決） | ✅ GREEN（BUG-003 修正済） |
| 4 | グローバル `--project-root`/`--json`/`--models-dir` | `global_json_*`/`*_json_*`/`global_models_dir_accepted` | ✅ GREEN（BUG-001/002 修正済） |
| 5 | check 終了コード（Error>0→1） | `check_formal_*`/`check_cycle_*` | ✅ GREEN |
| 6 | snapshot/refresh-subnodes 排他・既定 | `refresh_subnodes_exclusive_flags_exit_2`/`embed_all_and_node_exclusive_exit_2` | ✅ GREEN（BUG-009 修正済、exit 2）|
| 7 | （MCP）→ CTR-MCP-001 へ | — | （別サーフェス） |
| 8 | （MCP）→ CTR-MCP-001 へ | — | （別サーフェス） |
| 9 | （MCP）→ CTR-MCP-001 へ | — | （別サーフェス） |
| 10 | 設定探索 `.legixy.toml`+`.trace-engine.toml` | `config_legixy_toml_*`/`config_legacy_*`/`config_malformed_*` | ✅ GREEN（BUG-003 修正済）|
| 11 | 使用法誤り exit 2 / 実行時失敗 exit 1 | `*_exits_2`/`*_exits_1` 各ケース | ✅ GREEN（BUG-008/009 修正済）|

加えて check 形式層 5 カテゴリ（SPEC-LGX-004.REQ.01: FileExistence/ChainIntegrity/OrphanFile/GraphDag/SubnodeIdFormat）
**全実装**（`check_file_existence_*`/`check_malformed_*`/`check_cycle_*`、BUG-006 修正済）、drift 実 ONNX 推論（BUG-004）、
check 意味層 SemanticChecker 配線（BUG-005）、context subnode 粒度スライス（BUG-007）も GREEN。残ギャップ無し（2026-06-14）。

## 4. 凍結変更の扱い

凍結後の契約変更は次バージョンの契約改訂（HR7・人間承認）。配送軸の下流（DLV→TS→TC→SRC）へインクリメンタル再構築する（`10-modification-events.md`）。
