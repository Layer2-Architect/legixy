// BUG-007 CLI E2E 回帰テスト: サブノード細粒度の配線（R-1〜3,7）。
// 実行ファイル `legixy` を spawn し、graph.toml + 実 Markdown から
// `parse_graph` がサブノードを ParentChild エッジへ materialize することを E2E で検証する。
//
// pre-fix では CLI の load() が素の load_graph() を使い、サブノードを materialize しないため
//   - `--granularity subnode` がドキュメント全文へ fallback（subnode_id 行が出ない）
//   - トークン削減が起きない
// → RED。fix（load() を parse_graph 化）で GREEN。
//
// 親: SPEC-LGX-003.REQ.03（粒度制御）/ DD-LGX-003（サブノード自動抽出）/ DD-LGX-001（CLI 統合層）。

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_legixy");

fn run(project_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .arg("--project-root")
        .arg(project_root)
        .args(args)
        .output()
        .expect("spawn legixy")
}

/// 上流 UC（h2 サブノードあり）→ 下流 DD（chain）。subnode 粒度は上流に適用されるため
/// 起点は下流 DD ファイルにする。
const GRAPH: &str = r#"
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
"#;

const UC_DOC: &str =
    "# タイトル\n\n## セクションA\n\nアルファ本文 alpha-unique-AAA。\n\n## セクションB\n\nベータ本文 beta-unique-BBB。\n";

fn write_project(dir: &Path) {
    let gdir = dir.join("docs/traceability");
    std::fs::create_dir_all(&gdir).expect("mkdir");
    std::fs::write(gdir.join("graph.toml"), GRAPH).expect("write graph.toml");
    std::fs::write(dir.join("uc.md"), UC_DOC).expect("write uc.md");
    std::fs::write(dir.join("dd.md"), "# DD\n").expect("write dd.md");
}

#[test]
fn context_subnode_granularity_materializes_subnodes_e2e() {
    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());

    let out = run(tmp.path(), &["context", "dd.md", "--granularity", "subnode"]);
    assert!(out.status.success(), "context exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);

    // サブノードが materialize された証拠: subnode_id 行が出る（pre-fix は document fallback で出ない）。
    assert!(
        stdout.contains("subnode_id: UC-LGX-001#"),
        "サブノード materialize（subnode_id 行）が必要: {stdout}"
    );
    // h2 が 2 区画に分割されている（2 つの subnode_id エントリ）。
    let subnode_lines = stdout.matches("subnode_id: UC-LGX-001#").count();
    assert_eq!(subnode_lines, 2, "h2 が 2 サブノードに分割: {stdout}");
    // 両区画の本文（区画スライス）が出力に含まれる。
    assert!(stdout.contains("alpha-unique-AAA"), "区画A本文: {stdout}");
    assert!(stdout.contains("beta-unique-BBB"), "区画B本文: {stdout}");
}

#[test]
fn context_document_granularity_has_no_subnode_id_e2e() {
    // 対照: document 粒度では subnode_id 行は出ず、上流 UC 全文が 1 ブロックで返る。
    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());

    let out = run(tmp.path(), &["context", "dd.md"]);
    assert!(out.status.success(), "context exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("subnode_id:"),
        "document 粒度では subnode_id 行なし: {stdout}"
    );
    // 上流 UC 全文（両マーカーが同一ブロック）。
    assert!(stdout.contains("alpha-unique-AAA") && stdout.contains("beta-unique-BBB"));
}

// ── R-1/R-2/R-3: サブノード細粒度 walk のエッジ誘導解像（外部 fixture subnode_sample の in-repo ガード）──
// UC→DD→SRC（chain）+ 明示サブノード DD#s:secB（anchor=「## セクションB」）+ SRC→DD#s:secB（chain、細粒度トレース）。
// 期待:
//   R-1 context --granularity subnode src.rs < document（関連サブノードへ絞り込み）
//   R-2 investigate SRC が DD-LGX-001#s:secB を解像（document に潰れない）
//   R-3 --sections "DD-LGX-001#s:secB" の本文が当該区画（空でない）

const GRAPH_SN: &str = r###"
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "uc.md"
[[nodes]]
id = "DD-LGX-001"
type = "DD"
path = "dd.md"
[[nodes]]
id = "SRC-LGX-001"
type = "SRC"
path = "src.rs"
[[nodes]]
id = "DD-LGX-001#s:secB"
type = "DD-section"
parent = "DD-LGX-001"
path = "dd.md"
anchor = "## セクションB"
[[edges]]
from = "UC-LGX-001"
to = "DD-LGX-001"
kind = "chain"
[[edges]]
from = "DD-LGX-001"
to = "SRC-LGX-001"
kind = "chain"
[[edges]]
from = "SRC-LGX-001"
to = "DD-LGX-001#s:secB"
kind = "chain"
"###;

const DD_DOC: &str = "# DD タイトル\n\n## セクションA\n\n本文A alpha-AAA。\n\n### サブA1\n\n本文A1 sub-A1。\n\n## セクションB\n\n本文B beta-BBB。\n";

fn write_sn_project(dir: &Path) {
    let gdir = dir.join("docs/traceability");
    std::fs::create_dir_all(&gdir).unwrap();
    std::fs::write(gdir.join("graph.toml"), GRAPH_SN).unwrap();
    std::fs::write(dir.join("uc.md"), "# UC\n\n## 受付\n\nUC本文。\n").unwrap();
    std::fs::write(dir.join("dd.md"), DD_DOC).unwrap();
    std::fs::write(dir.join("src.rs"), "// src\n").unwrap();
}

#[test]
fn subnode_granularity_reduces_via_edge_guided_narrowing_e2e() {
    // R-1: SRC は DD#s:secB に結線 → subnode 粒度は当該区画へ絞り込み、document より小さい。
    let tmp = tempfile::tempdir().unwrap();
    write_sn_project(tmp.path());
    let doc = run(tmp.path(), &["context", "--granularity", "document", "src.rs"]);
    let sub = run(tmp.path(), &["context", "--granularity", "subnode", "src.rs"]);
    assert!(doc.status.success() && sub.status.success());
    assert!(
        sub.stdout.len() < doc.stdout.len(),
        "subnode({}) < document({}) でトークン削減（LGX-EXT-001 目的1）",
        sub.stdout.len(),
        doc.stdout.len()
    );
    // 絞り込まれた上流に secB が含まれ、無関係区画(サブA1 等)は出ない。
    let s = String::from_utf8_lossy(&sub.stdout);
    assert!(s.contains("DD-LGX-001#s:secB"), "結線サブノードに絞る: {s}");
    assert!(!s.contains("sub-A1"), "無関係区画は出ない: {s}");
}

#[test]
fn explicit_subnode_section_body_resolved_via_anchor_e2e() {
    // R-3: 明示 #s: サブノードの本文が anchor 見出し（## セクションB）の区画スライスで返る（空でない）。
    let tmp = tempfile::tempdir().unwrap();
    write_sn_project(tmp.path());
    let out = run(
        tmp.path(),
        &[
            "context",
            "--granularity",
            "subnode",
            "--sections",
            "DD-LGX-001#s:secB",
            "src.rs",
        ],
    );
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("DD-LGX-001#s:secB"), "明示サブノードが返る: {s}");
    assert!(s.contains("beta-BBB"), "anchor 区画(セクションB)の本文が空でない: {s}");
    assert!(!s.contains("alpha-AAA"), "他区画(セクションA)は含まない: {s}");
}

#[test]
fn investigate_resolves_explicit_subnode_endpoint_e2e() {
    // R-2: investigate が明示 #s: 終点を subnode 解像度で surface（document へ潰さない）。
    let tmp = tempfile::tempdir().unwrap();
    write_sn_project(tmp.path());
    let out = run(tmp.path(), &["investigate", "SRC-LGX-001"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("DD-LGX-001#s:secB"),
        "investigate が明示サブノードを解像: {s}"
    );
}
