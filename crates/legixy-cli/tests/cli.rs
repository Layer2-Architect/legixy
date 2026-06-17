// Document ID: TC-CLI-001
// legixy CLI 契約適合テスト（配送軸 TC[DLV]、area=CLI。親 TS-CLI-001 / 契約 CTR-CLI-001）。
// legixy CLI 統合テスト（TC）。実行ファイル `legixy` を spawn し LGX-COMPAT-001 §3 の
// サブコマンド名・終了コード（0/1/2）・graph.toml ローダ統合を検証する。
//
// 親: LGX-COMPAT-001 §3（終了コード凍結）+ DD-LGX-001（graph load → check）+ DD-LGX-005/006（nav）。
// hermetic: tempdir に最小 graph.toml を構築（リポジトリ配置に非依存）。

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_legixy");

fn write_graph(dir: &Path, toml: &str) {
    let gdir = dir.join("docs/traceability");
    std::fs::create_dir_all(&gdir).expect("mkdir");
    std::fs::write(gdir.join("graph.toml"), toml).expect("write graph.toml");
    // FileExistence 検査（BUG-006）のため、graph に現れる `path = "..."` の実ファイルを用意する。
    // サブノード（`#` を含む node の path 等）も含め、宣言された全 path を空ファイルとして作成。
    for line in toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("path = \"") {
            if let Some(rel) = rest.strip_suffix('"') {
                let abs = dir.join(rel);
                if let Some(parent) = abs.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&abs, "").ok();
            }
        }
    }
}

fn run(project_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .arg("--project-root")
        .arg(project_root)
        .args(args)
        .output()
        .expect("spawn legixy")
}

const VALID_GRAPH: &str = r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "docs/use-cases/UC-LGX-001.md"

[[nodes]]
id = "RBA-LGX-001"
type = "RBA"
path = "docs/robustness-abstract/RBA-LGX-001.md"

[[edges]]
from = "UC-LGX-001"
to = "RBA-LGX-001"
kind = "chain"
"#;

#[test]
fn check_formal_valid_graph_exits_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["check", "--formal"]);
    assert!(out.status.success(), "valid graph → exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Summary"), "サマリ行を出力: {stdout}");
    assert!(stdout.contains("0 error"), "エラー 0: {stdout}");
}

#[test]
fn check_formal_chain_break_exits_1() {
    // 孤立した RBA（chain typecode・親 chain エッジ無し）→ ChainIntegrity Error → exit 1。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "RBA-LGX-009"
type = "RBA"
path = "docs/robustness-abstract/RBA-LGX-009.md"
"#,
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "chain 親欠落 → exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ERROR"), "ERROR finding を出力: {stdout}");
}

#[test]
fn check_full_without_db_emits_info_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["check"]);
    assert!(out.status.success(), "full・db 不在は非致命 → exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("INFO"), "意味層スキップ Info: {stdout}");
}

#[test]
fn impact_forward_traversal_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["impact", "UC-LGX-001"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("UC-LGX-001"), "起点を含む: {stdout}");
    assert!(stdout.contains("RBA-LGX-001"), "順方向到達: {stdout}");
}

#[test]
fn investigate_reverse_traversal_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["investigate", "RBA-LGX-001"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("RBA-LGX-001"), "起点を含む: {stdout}");
}

#[test]
fn missing_graph_exits_1() {
    let tmp = tempfile::tempdir().unwrap(); // graph.toml なし
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "graph 不在 → exit 1");
}

#[test]
fn unwired_subcommand_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["embed", "--all"]);
    assert_eq!(out.status.code(), Some(1), "未配線サブコマンド → exit 1");
}

#[test]
fn unknown_subcommand_exits_2() {
    let out = Command::new(BIN).arg("bogus").output().expect("spawn");
    assert_eq!(out.status.code(), Some(2), "clap 未知サブコマンド → exit 2");
}

#[test]
fn bad_flag_exits_2() {
    let out = Command::new(BIN).args(["check", "--nope"]).output().expect("spawn");
    assert_eq!(out.status.code(), Some(2), "clap 不正フラグ → exit 2");
}

#[test]
fn full_surface_declares_19_subcommands() {
    // LGX-COMPAT-001 §3: 19 サブコマンドのサーフェスが --help に存在する（MCP-INV と凍結契約）。
    let out = Command::new(BIN).arg("--help").output().expect("spawn");
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for sub in [
        "init", "migrate", "check", "embed", "drift", "report", "calibrate", "snapshot",
        "refresh-subnodes", "context", "impact", "investigate", "feedback", "observe", "audit",
        "analyze", "proposals", "approve", "reject",
    ] {
        assert!(help.contains(sub), "サブコマンド {sub} が --help に存在");
    }
}

// ── context（MCP compile_context の下位層、DD-LGX-002） ──

const CHAIN_GRAPH: &str = r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "uc.md"

[[nodes]]
id = "SPEC-LGX-001"
type = "SPEC"
path = "spec.md"

[[edges]]
from = "SPEC-LGX-001"
to = "UC-LGX-001"
kind = "chain"
"#;

#[test]
fn context_renders_markdown_with_cache_breakpoint() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["context", "uc.md", "--command", "test"]);
    assert!(out.status.success(), "context exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 6 セクション枠 + キャッシュブレーク点マーカ（REQ.10 / REL.09）。
    assert!(stdout.contains("Upstream Artifacts"), "上流セクション: {stdout}");
    assert!(stdout.contains("cache-breakpoint: stable-end"), "マーカ: {stdout}");
}

#[test]
fn context_writes_audit_log_visible_via_audit() {
    // context → context_log 書込 → audit が反映（ctx AuditLogger 実書込、MCP get_compile_audit の裏付け）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let c = run(tmp.path(), &["context", "uc.md", "--command", "impl"]);
    assert!(c.status.success());
    let a = run(tmp.path(), &["audit"]);
    assert!(a.status.success());
    let stdout = String::from_utf8_lossy(&a.stdout);
    // payload は二重シリアライズされた JSON 文字列。target_id と command 値の存在を確認。
    assert!(stdout.contains("UC-LGX-001"), "audit に target_id: {stdout}");
    assert!(stdout.contains("command"), "payload に command キー: {stdout}");
    assert!(stdout.contains("impl"), "payload に command 値 impl: {stdout}");
}

#[test]
fn context_requires_target_files_exit_2() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["context"]); // 位置引数なし
    assert_eq!(out.status.code(), Some(2), "target_files 必須 → clap exit 2");
}

// ── feedback 群（engine.db、ADR-LGX-015 で .legixy/engine.db を tempdir に自動作成） ──

#[test]
fn observe_emits_frozen_stdout_and_dedups() {
    let tmp = tempfile::tempdir().unwrap();
    let a = run(tmp.path(), &["observe", "manual_note", "メモ", "--related-id", "DD-LGX-001"]);
    assert!(a.status.success());
    let sa = String::from_utf8_lossy(&a.stdout);
    // 凍結 stdout 形式（MCP observe が parse）。
    assert!(sa.contains("observation: id="), "frozen 形式: {sa}");
    assert!(sa.contains("skipped=false"), "新規は skipped=false: {sa}");
    // 同一 (category, related_ids) は dedup → skipped=true。
    let b = run(tmp.path(), &["observe", "manual_note", "別メモ", "--related-id", "DD-LGX-001"]);
    assert!(b.status.success());
    assert!(String::from_utf8_lossy(&b.stdout).contains("skipped=true"), "dedup");
    // .legixy/engine.db が正準パスに作られる（ADR-015）。
    assert!(tmp.path().join(".legixy/engine.db").exists(), "engine.db 正準パス");
}

#[test]
fn observe_empty_message_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["observe", "manual_note", "   "]);
    assert_eq!(out.status.code(), Some(1), "空 message → exit 1（SPEC-007.REQ.04）");
}

#[test]
fn audit_empty_is_json_array() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["audit", "--limit", "5"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "[]", "空監査は []（valid JSON）");
}

#[test]
fn proposals_bad_status_exits_2() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["proposals", "--status", "bogus"]);
    assert_eq!(out.status.code(), Some(2), "不正 --status は clap exit 2");
}

#[test]
fn reject_nonexistent_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["reject", "999", "--reason", "なし"]);
    assert_eq!(out.status.code(), Some(1), "不在 proposal → exit 1");
}

// ── refresh-subnodes（ADR-LGX-023、LGX-COMPAT-001 #9） ──

#[test]
fn refresh_subnodes_empty_is_clean_exit_0() {
    // engine.db に subnode 行が無い（embed 未実行）→ renames/orphans 0、exit 0。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["refresh-subnodes"]);
    assert!(out.status.success(), "dry-run exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("renames: 0"), "renames 0: {stdout}");
}

#[test]
fn refresh_subnodes_exclusive_flags_exit_2() {
    // BUG-009: 排他フラグの使用法誤りは clap が exit 2（契約 §3「使用法誤り = exit 2」）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["refresh-subnodes", "--dry-run", "--apply"]);
    assert_eq!(out.status.code(), Some(2), "--dry-run と --apply 同時 → exit 2");
}

#[test]
fn embed_all_and_node_exclusive_exit_2() {
    // BUG-008: `embed --all --node` は排他（契約 §4#4）。使用法誤り → exit 2。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["embed", "--all", "--node", "UC-LGX-001"]);
    assert_eq!(out.status.code(), Some(2), "--all と --node 同時 → exit 2");
}

#[test]
fn snapshot_create_empty_store_non_persist() {
    // BUG-010: 空ストアの snapshot create は非永続 + 明示メッセージ（UC-LGX-012）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["snapshot", "create"]);
    assert_eq!(out.status.code(), Some(0), "空ストア create は exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("非永続") || stdout.contains("embeddings がありません"),
        "非永続の明示メッセージ: {stdout}"
    );
    // list は空（永続されていない）。
    let ls = run(tmp.path(), &["snapshot", "list"]);
    let ls_out = String::from_utf8_lossy(&ls.stdout);
    assert!(ls_out.contains("no snapshots"), "永続なし → list 空: {ls_out}");
}

// ── init / migrate（legixy-mig、DD-LGX-009） ──

#[test]
fn init_creates_project_then_guards_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["init"]);
    assert!(out.status.success(), "init(空) → exit 0");
    assert!(tmp.path().join(".legixy.toml").exists(), ".legixy.toml 生成");
    assert!(
        tmp.path().join("docs/traceability/graph.toml").exists(),
        "graph.toml 生成"
    );
    // 既存 → --force なしは exit 1（GAP-LGX-143）。
    let again = run(tmp.path(), &["init"]);
    assert_eq!(again.status.code(), Some(1), "既存 init → exit 1");
    // --force → 上書き exit 0。
    let forced = run(tmp.path(), &["init", "--force"]);
    assert!(forced.status.success(), "init --force → exit 0");
}

#[test]
fn migrate_requires_from_and_errors_on_missing_source() {
    let tmp = tempfile::tempdir().unwrap();
    // --from 必須 → clap exit 2。
    let no_from = Command::new(BIN)
        .arg("--project-root")
        .arg(tmp.path())
        .arg("migrate")
        .output()
        .expect("spawn");
    assert_eq!(no_from.status.code(), Some(2), "--from 必須 → exit 2");
    // 移行元不在 → exit 1（V01NotFound）。
    let nf = run(tmp.path(), &["migrate", "--from", &tmp.path().join("nope").to_string_lossy()]);
    assert_eq!(nf.status.code(), Some(1), "移行元不在 → exit 1");
}

// ── embed 系（legixy-embed。embed は onnx feature 必須、他は store ベースで非 onnx 動作） ──

#[test]
fn report_empty_store_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["report"]);
    assert!(out.status.success(), "空 store の report は exit 0");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("links: 0"),
        "links: 0"
    );
}

#[test]
fn calibrate_empty_store_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["calibrate", "--buckets", "5"]);
    assert!(out.status.success(), "空 store の calibrate は exit 0");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("pairs: 0"),
        "pairs: 0"
    );
}

#[test]
fn snapshot_list_empty_and_delete_missing_label_exit_1() {
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let list = run(tmp.path(), &["snapshot", "list"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("no snapshots"));
    // delete の label 不在は exit 1（SPEC-LGX-010.REQ.03 6c）。
    let del = run(tmp.path(), &["snapshot", "delete", "label:missing"]);
    assert_eq!(del.status.code(), Some(1), "label 不在 delete → exit 1");
}

#[test]
fn embed_without_model_exits_1() {
    // 既定（no-onnx）テストビルド: モデル不在 or onnx 未コンパイルで embed は exit 1（graceful）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    let out = run(tmp.path(), &["embed", "--all"]);
    assert_eq!(out.status.code(), Some(1), "embed（モデル/onnx 不在）→ exit 1");
}

#[test]
fn feedback_analyze_approve_lifecycle() {
    // 壊れた graph（chain 親欠落）→ feedback（自動 observe）→ analyze（proposal）→ approve → 終端 CAS。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "RBA-LGX-009"
type = "RBA"
path = "docs/robustness-abstract/RBA-LGX-009.md"
"#,
    );
    let fb = run(tmp.path(), &["feedback"]);
    assert!(fb.status.success(), "feedback exit 0");
    assert!(
        String::from_utf8_lossy(&fb.stdout).contains("created=1"),
        "check error から 1 件 observe: {}",
        String::from_utf8_lossy(&fb.stdout)
    );

    let an = run(tmp.path(), &["analyze"]);
    assert!(an.status.success());
    assert!(
        String::from_utf8_lossy(&an.stdout).contains("1 proposal"),
        "1 proposal 生成: {}",
        String::from_utf8_lossy(&an.stdout)
    );

    let pl = run(tmp.path(), &["proposals"]);
    assert!(String::from_utf8_lossy(&pl.stdout).contains("pending"), "pending proposal");

    let ap = run(tmp.path(), &["approve", "1"]);
    assert!(ap.status.success(), "approve exit 0");

    // 終端状態への再 approve は exit 1（CAS）。
    let re = run(tmp.path(), &["approve", "1"]);
    assert_eq!(re.status.code(), Some(1), "終端再操作 → exit 1");
}

// ── 設定ファイル読込（BUG-003、LGX-COMPAT-001 §6 / SPEC-LGX-008.REQ.13） ──
// 親: CTR-CLI-001 §3 RED 項目（EXT-CONF-050/051/052・EXT-ERR-003）。

fn write_file(dir: &Path, name: &str, content: &str) {
    std::fs::write(dir.join(name), content).expect("write config");
}

/// 親 chain エッジを持たない単独 RBA。既定 order では chain typecode（親欠落 Error）だが、
/// config の `[id.chain] order` を絞れば chain 対象外になる ⇒ exit が config 由来で変わる。
const LONE_RBA_GRAPH: &str = r#"
[[nodes]]
id = "RBA-LGX-001"
type = "RBA"
path = "docs/robustness-abstract/RBA-LGX-001.md"
"#;

#[test]
fn config_legixy_toml_custom_chain_order_respected() {
    // EXT-CONF-050: `.legixy.toml` が実際に解析される。
    // order=["UC"] なら RBA は chain 対象外 → ChainIntegrity Error 無し → exit 0。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), LONE_RBA_GRAPH);
    write_file(tmp.path(), ".legixy.toml", "[id.chain]\norder = [\"UC\"]\n");
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "config の order=[UC] で RBA は chain 外 → exit 0: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn config_legixy_toml_takes_priority_over_trace_engine() {
    // EXT-CONF-052: 両ファイル存在時は `.legixy.toml` 優先。
    // .legixy.toml=order[UC]（RBA 対象外→exit0）、.trace-engine.toml=order[UC,RBA]（RBA 親欠落→exit1）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), LONE_RBA_GRAPH);
    write_file(tmp.path(), ".legixy.toml", "[id.chain]\norder = [\"UC\"]\n");
    write_file(
        tmp.path(),
        ".trace-engine.toml",
        "[id.chain]\norder = [\"UC\", \"RBA\"]\n",
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        ".legixy.toml 優先 → exit 0: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn config_legacy_trace_engine_fallback_read() {
    // EXT-CONF-051: `.legixy.toml` 不在時は旧名 `.trace-engine.toml` を読む（+ 移行 Info を stderr）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), LONE_RBA_GRAPH);
    write_file(
        tmp.path(),
        ".trace-engine.toml",
        "[id.chain]\norder = [\"UC\"]\n",
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "旧名 fallback の order=[UC] で RBA は chain 外 → exit 0: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn config_malformed_toml_exits_1() {
    // EXT-ERR-003: 不正 TOML の config は exit 1（実行時失敗）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    write_file(tmp.path(), ".legixy.toml", "this is = = not valid toml [[[\n");
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "不正 TOML → exit 1");
}

// ── グローバル --json / --models-dir（BUG-001/002、LGX-COMPAT-001 §3） ──

#[test]
fn global_json_accepted_by_check() {
    // EXT-GLOB-002 / EXT-CHK-002: check が `--json` を受理（exit 2 で拒否しない）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["check", "--formal", "--json"]);
    assert_ne!(out.status.code(), Some(2), "--json を受理（拒否 exit 2 でない）");
    assert_eq!(out.status.code(), Some(0), "クリーン graph → exit 0");
}

#[test]
fn global_json_before_subcommand_accepted() {
    // EXT-GLOB-002: サブコマンド前のグローバル `--json` も受理。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = Command::new(BIN)
        .arg("--project-root")
        .arg(tmp.path())
        .args(["--json", "check", "--formal"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(0), "グローバル位置の --json 受理");
}

#[test]
fn check_json_emits_json_findings() {
    // EXT-CHK-002 / SD-CHK-004: check --json は finding を JSON で出す。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), LONE_RBA_GRAPH); // ChainIntegrity Error 1 件
    let out = run(tmp.path(), &["check", "--formal", "--json"]);
    assert_eq!(out.status.code(), Some(1), "Error あり → exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ChainIntegrity"), "JSON に category: {stdout}");
    assert!(stdout.trim_start().starts_with('{'), "JSON オブジェクト行: {stdout}");
}

#[test]
fn check_json_zero_findings_emits_no_output() {
    // 0 件の check --json は無出力（JSON Lines = 0 レコード）。余分な空行を出さない。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["check", "--formal", "--json"]);
    assert_eq!(out.status.code(), Some(0), "クリーン graph → exit 0");
    assert!(
        out.stdout.is_empty(),
        "0 件は無出力（空行も出さない）: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn impact_json_emits_jsonlines() {
    // EXT-NAV-005: impact が `--json` を受理し JSON で出力。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["impact", "UC-LGX-001", "--json"]);
    assert_eq!(out.status.code(), Some(0), "impact --json exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains('{'), "JSON 出力: {stdout}");
    assert!(stdout.contains("UC-LGX-001"), "起点を含む: {stdout}");
}

// ── check 形式層カテゴリ（BUG-006、SPEC-LGX-004.REQ.01） ──

#[test]
fn check_file_existence_missing_file_exits_1() {
    // EXT-CHK-004: ノード path が実在しない → FileExistence Error → exit 1。
    // graph.toml を手書きして当該ファイルを敢えて作らない（write_graph の自動生成を回避）。
    let tmp = tempfile::tempdir().unwrap();
    let gdir = tmp.path().join("docs/traceability");
    std::fs::create_dir_all(&gdir).unwrap();
    std::fs::write(
        gdir.join("graph.toml"),
        r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "docs/use-cases/UC-LGX-001.md"
"#,
    )
    .unwrap();
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "path 不在 → exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FileExistence"), "FileExistence Error: {stdout}");
}

#[test]
fn check_malformed_document_id_exits_1() {
    // EXT-CHK-003: ID 形式不正（`{type}-{area}-{seq}` に非適合）→ ChainIntegrity Error → exit 1。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "BADID"
type = "UC"
path = "docs/use-cases/BADID.md"
"#,
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "不正 ID → exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ID 形式が不正"), "ID 形式 Error: {stdout}");
}

#[test]
fn check_malformed_subnode_id_exits_1() {
    // EXT-SUB-EXPL-003: 不正な `#` 接尾辞のサブノード ID → SubnodeIdFormat Error → exit 1。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "UC-LGX-001#not_valid"
type = "UC"
path = "docs/use-cases/UC-LGX-001.md"
"#,
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "不正サブノード ID → exit 1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("SubnodeIdFormat"), "SubnodeIdFormat Error: {stdout}");
}

#[test]
fn check_explicit_subnode_invalid_slug_exits_1() {
    // EXT-SUB-EXPL-003（slug 制約）: 明示 `#s:<slug>` は LGX-EXT-001 §4.5.2 の文字制約
    // （英数/ハイフン/アンダースコア・両端英数・1〜63 文字）を強制する。`s:bad id!`（空白・記号混入）
    // → SubnodeIdFormat Error → exit 1。pre-fix は非空のみ検査で通過していた。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "docs/use-cases/UC-LGX-001.md"
[[nodes]]
id = "UC-LGX-001#s:bad id!"
type = "UC-section"
parent = "UC-LGX-001"
path = "docs/use-cases/UC-LGX-001.md"
"#,
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "不正 slug → exit 1");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("SubnodeIdFormat"),
        "SubnodeIdFormat Error: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn check_explicit_subnode_valid_slug_ok() {
    // 対照: 妥当な slug（英数・ハイフン、両端英数）は通過（exit 0）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "docs/use-cases/UC-LGX-001.md"
[[nodes]]
id = "UC-LGX-001#s:state-machine"
type = "UC-section"
parent = "UC-LGX-001"
path = "docs/use-cases/UC-LGX-001.md"
"#,
    );
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "妥当 slug → exit 0: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn global_models_dir_accepted() {
    // EXT-GLOB-003: グローバル `--models-dir` を受理（exit 2 で拒否しない）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = Command::new(BIN)
        .arg("--project-root")
        .arg(tmp.path())
        .arg("--models-dir")
        .arg("/tmp/nonexistent-models")
        .args(["impact", "UC-LGX-001"])
        .output()
        .expect("spawn");
    assert_ne!(out.status.code(), Some(2), "--models-dir を受理（拒否 exit 2 でない）");
}

// ─────────────────────────────────────────────────────────────────────────────
// legixy.test 外部スイートからの移植（ONNX 不要・決定論ケースのみ）。
// 契約適合の回帰ガードを in-repo・CI で常時強制する（P-1）。広い意味/ONNX シナリオは
// 外部 legixy.test（独立検証チャネル）に温存（00-philosophy §2.4）。各 fn 先頭に EXT-ID を明記。
// ─────────────────────────────────────────────────────────────────────────────

const CYCLE_GRAPH: &str = r#"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "uc.md"
[[nodes]]
id = "DD-LGX-001"
type = "DD"
path = "dd.md"
[[edges]]
from = "UC-LGX-001"
to = "DD-LGX-001"
kind = "chain"
[[edges]]
from = "DD-LGX-001"
to = "UC-LGX-001"
kind = "custom"
"#;

#[test]
fn check_cycle_graphdag_error_exit_1() {
    // EXT-CHK-008: 有向サイクル → GraphDag Error → exit 1。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CYCLE_GRAPH);
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "サイクル → exit 1");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("GraphDag"),
        "GraphDag Error: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn version_flag_exits_0() {
    // EXT-GLOB-004: -V / --version → exit 0、バージョン表示。
    let out = Command::new(BIN).arg("--version").output().expect("spawn");
    assert!(out.status.success(), "--version exit 0");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("legixy"),
        "バージョン文字列"
    );
}

#[test]
fn context_json_accepted_and_valid() {
    // EXT-CTX-002: context が `--json` を受理し JSON を出力。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), CHAIN_GRAPH);
    std::fs::write(tmp.path().join("uc.md"), "# UC\n").unwrap();
    let out = run(tmp.path(), &["context", "uc.md", "--json"]);
    assert_eq!(out.status.code(), Some(0), "context --json exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.trim_start().starts_with('{'), "JSON: {stdout}");
    assert!(stdout.contains("markdown"), "markdown キー: {stdout}");
}

#[test]
fn investigate_json_accepted() {
    // EXT-NAV-005: investigate が `--json` を受理。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["investigate", "RBA-LGX-001", "--json"]);
    assert_eq!(out.status.code(), Some(0), "investigate --json exit 0");
}

#[test]
fn report_json_accepted() {
    // EXT-RPT-002: report が `--json` を受理（空ストアでも exit 0・JSON）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["report", "--json"]);
    assert_eq!(out.status.code(), Some(0), "report --json exit 0");
}

#[test]
fn observe_json_accepted() {
    // EXT-FB-015: observe が `--json` を受理し {id, skipped} を出す。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["observe", "manual_note", "気づき", "--json"]);
    assert_eq!(out.status.code(), Some(0), "observe --json exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"id\""), "id キー: {stdout}");
    assert!(stdout.contains("skipped"), "skipped キー: {stdout}");
}

#[test]
fn feedback_and_proposals_json_accepted() {
    // EXT-FB-003 / EXT-FB-032 / EXT-APR-003: feedback/analyze/proposals が `--json` を受理。
    // 壊れた graph で feedback が observation を自動生成 → analyze → proposals を JSON で。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), LONE_RBA_GRAPH); // ChainIntegrity Error → observation 化
    let fb = run(tmp.path(), &["feedback", "--json"]);
    assert_eq!(fb.status.code(), Some(0), "feedback --json exit 0");
    assert!(
        String::from_utf8_lossy(&fb.stdout).contains("observations_created"),
        "feedback JSON: {}",
        String::from_utf8_lossy(&fb.stdout)
    );
    let an = run(tmp.path(), &["analyze", "--json"]);
    assert_eq!(an.status.code(), Some(0), "analyze --json exit 0");
    assert!(
        String::from_utf8_lossy(&an.stdout).trim_start().starts_with('['),
        "analyze JSON 配列: {}",
        String::from_utf8_lossy(&an.stdout)
    );
    let pl = run(tmp.path(), &["proposals", "--json"]);
    assert_eq!(pl.status.code(), Some(0), "proposals --json exit 0");
    assert!(
        String::from_utf8_lossy(&pl.stdout).trim_start().starts_with('['),
        "proposals JSON 配列: {}",
        String::from_utf8_lossy(&pl.stdout)
    );
}

#[test]
fn audit_limit_range_enforced() {
    // EXT-FB-021: --limit は 1..=50。51 は使用法誤り → exit 2、境界 1/50 は受理。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    assert_eq!(
        run(tmp.path(), &["audit", "--limit", "51"]).status.code(),
        Some(2),
        "51 は範囲外 → exit 2"
    );
    assert_eq!(
        run(tmp.path(), &["audit", "--limit", "0"]).status.code(),
        Some(2),
        "0 は範囲外 → exit 2"
    );
    assert_ne!(
        run(tmp.path(), &["audit", "--limit", "50"]).status.code(),
        Some(2),
        "50 は受理"
    );
}

#[test]
fn reject_missing_reason_exits_2() {
    // EXT-APR-021: reject の必須 `--reason` 欠落は使用法誤り → exit 2。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["reject", "1"]);
    assert_eq!(out.status.code(), Some(2), "--reason 欠落 → exit 2");
}

#[test]
fn approve_non_integer_id_exits_2() {
    // EXT-APR-031: 非整数 ID は使用法誤り → exit 2。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["approve", "not-a-number"]);
    assert_eq!(out.status.code(), Some(2), "非整数 ID → exit 2");
}

#[test]
fn snapshot_requires_subcommand_exits_2() {
    // EXT-SNAP-010: snapshot のサブコマンド省略は使用法誤り → exit 2。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let out = run(tmp.path(), &["snapshot"]);
    assert_eq!(out.status.code(), Some(2), "サブコマンド省略 → exit 2");
}

#[test]
fn corrupt_engine_db_exits_1() {
    // EXT-INV-013: engine.db 破損 → exit 1（自動再生成しない、STATE-INV-1）。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), VALID_GRAPH);
    let db_dir = tmp.path().join(".legixy");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::write(db_dir.join("engine.db"), b"not a sqlite database").unwrap();
    // 破損 db を読む read 系コマンド（proposals）→ exit 1。
    let out = run(tmp.path(), &["proposals"]);
    assert_eq!(out.status.code(), Some(1), "破損 engine.db → exit 1");
}

const MULTI_AREA_CONFIG: &str = r#"
[id]
pattern = "{type}-{area}-{seq}"
areas = ["LGX", "CLI"]
[[id.chains]]
area = "LGX"
order = ["UC", "RBA", "SEQA", "RBD", "SEQD", "DD", "TS", "TC", "SRC"]
[[id.chains]]
area = "CLI"
order = ["CTR", "DLV", "TS", "TC", "SRC"]
"#;

const MULTI_AREA_GRAPH: &str = r#"
[[nodes]]
id = "CTR-CLI-001"
type = "CTR"
path = "ctr.md"
[[nodes]]
id = "DLV-CLI-001"
type = "DLV"
path = "dlv.md"
[[edges]]
from = "CTR-CLI-001"
to = "DLV-CLI-001"
kind = "chain"
"#;

#[test]
fn multi_area_config_resolves_per_area_chain() {
    // EXT-CONF-060/061（外部は fixture 無しで未実施）: multi-area config を実読込し
    // area 別 chain order を解決。CTR(根)→DLV は連結済 → 0 error / exit 0。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(tmp.path(), MULTI_AREA_GRAPH);
    write_file(tmp.path(), ".legixy.toml", MULTI_AREA_CONFIG);
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "multi-area: CTR 根 + DLV 連結 → exit 0: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn multi_area_config_detects_break_in_delivery_chain() {
    // multi-area で配送チェーン断裂（DLV に CTR 親エッジ無し）→ ChainIntegrity Error → exit 1。
    let tmp = tempfile::tempdir().unwrap();
    write_graph(
        tmp.path(),
        r#"
[[nodes]]
id = "DLV-CLI-001"
type = "DLV"
path = "dlv.md"
"#,
    );
    write_file(tmp.path(), ".legixy.toml", MULTI_AREA_CONFIG);
    let out = run(tmp.path(), &["check", "--formal"]);
    assert_eq!(out.status.code(), Some(1), "配送チェーン断裂 → exit 1");
}
