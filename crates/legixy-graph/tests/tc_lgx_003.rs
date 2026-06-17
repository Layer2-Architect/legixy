// Document ID: TC-LGX-003
// TC-LGX-003: サブノード自動抽出（UC-LGX-003）のテストコード（TC[RED]）
//
// 親 chain: DD-LGX-003 → TS-LGX-003 → 本 TC-LGX-003 → SRC-LGX-003。
// 各テストは TS-LGX-003 の各ケースを legixy-graph::subnode の凍結 API（DD-LGX-003 §3）に束縛する。
// SRC[GREEN] 未実装（extract_subnodes_with_levels / generate_subnode_id / normalize_heading /
// is_auto_generated_fragment / validate_explicit_name / parse_graph_from_str /
// classify_subnode_kind = todo!()）のため、これらを呼ぶ全テストは panic で失敗する（RED）。
// `cargo test -p legixy-graph --no-run` は通る（型・シグネチャ整合）が `cargo test` は失敗する。
//
// ケース 1（v3 一致 ID 凍結）の期待 hex リテラルは、本 crate の generate_subnode_id（todo!()）に
// 依存せず、独立計算（Python + Rust sha2/hex の二重検算）で確定した値を束縛する（自己循環回避・
// レビュー指摘）。SHA-256(parent + "|" + heading_path.join("|"))[0..16] 小文字 hex。

use legixy_graph::subnode::{
    classify_subnode_kind, extract_subnodes, extract_subnodes_with_levels, generate_subnode_id,
    is_auto_generated_fragment, normalize_heading, parent_child_edges, parse_graph_from_str,
    validate_explicit_name, AutoSubnode, GraphParseError, SubnodeKind,
};
use legixy_graph::EdgeKind;

// 独立計算で確定した v3 一致 ID（ケース 1。Python hashlib + Rust sha2/hex で二重検算済）。
const ID_SPEC002_OVERVIEW: &str = "SPEC-LGX-002#021205cd684b883a";
const ID_DD003_API_FROZEN: &str = "DD-LGX-003#78a50fa46f44eb3e";

// ===========================================================================
// ケース 1: 既知 (parent_id, heading_path) → v3 一致 ID（Contract・最重要・凍結）
// ===========================================================================
#[test]
fn test_generate_subnode_id_matches_v3_h2() {
    // @ts: TS-LGX-003 ケース 1
    let id = generate_subnode_id("SPEC-LGX-002", &["概要".to_string()]);
    assert_eq!(id, ID_SPEC002_OVERVIEW);
}

#[test]
fn test_generate_subnode_id_matches_v3_h3() {
    // @ts: TS-LGX-003 ケース 1（h3 二段パス）
    let id = generate_subnode_id(
        "DD-LGX-003",
        &["公開 API surface".to_string(), "凍結".to_string()],
    );
    assert_eq!(id, ID_DD003_API_FROZEN);
}

#[test]
fn test_generate_subnode_id_format_is_parent_hash_hex16() {
    // @ts: TS-LGX-003 ケース 1（形式: {parent}#{16文字小文字hex}）
    let id = generate_subnode_id("SPEC-LGX-002", &["概要".to_string()]);
    let (prefix, fragment) = id.split_once('#').expect("ID は '#' を含む");
    assert_eq!(prefix, "SPEC-LGX-002");
    assert_eq!(fragment.len(), 16);
    assert!(fragment.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
}

// ===========================================================================
// ケース 2: ID 生成の決定論（property）
// ===========================================================================
proptest::proptest! {
    #[test]
    fn test_generate_subnode_id_deterministic(
        // @ts: TS-LGX-003 ケース 2
        parent in ".{0,32}",
        heading_path in proptest::collection::vec(".{0,32}", 0..=3),
    ) {
        let id1 = generate_subnode_id(&parent, &heading_path);
        let id2 = generate_subnode_id(&parent, &heading_path);
        proptest::prop_assert_eq!(&id1, &id2);
        // hex 部は常に 16 文字・全て 0-9a-f
        let fragment = id1.rsplit_once('#').map(|(_, f)| f.to_string()).unwrap_or_default();
        proptest::prop_assert_eq!(fragment.len(), 16);
        proptest::prop_assert!(fragment.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }
}

// ===========================================================================
// ケース 3: heading_path 構築 — h2 は自見出しのみ
// ===========================================================================
#[test]
fn test_extract_h2_heading_path_self_only() {
    // @ts: TS-LGX-003 ケース 3
    let subs = extract_subnodes("UC-LGX-003", "## 概要\n本文\n");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].heading_path, vec!["概要".to_string()]);
    assert_eq!(subs[0].parent_id, "UC-LGX-003");
    assert_eq!(subs[0].id, "UC-LGX-003#61802a64976fbf12");
}

// ===========================================================================
// ケース 4: heading_path 構築 — h3 は直上 h2 コンテキスト + 自見出し
// ===========================================================================
#[test]
fn test_extract_h3_heading_path_includes_h2_context() {
    // @ts: TS-LGX-003 ケース 4
    let subs = extract_subnodes("UC-LGX-003", "## 基本フロー\n## 概要\n### 詳細\n");
    let h3 = subs
        .iter()
        .find(|s| s.heading_path.len() == 2)
        .expect("h3 ノードが 1 件存在する");
    assert_eq!(h3.heading_path, vec!["概要".to_string(), "詳細".to_string()]);
    // ハッシュ対象は "UC-LGX-003|概要|詳細"（独立計算値）
    assert_eq!(h3.id, "UC-LGX-003#657454e9b82c3851");
}

// ===========================================================================
// ケース 5: h1 出現で h2 コンテキストがリセットされる
// ===========================================================================
#[test]
fn test_extract_h1_resets_h2_context() {
    // @ts: TS-LGX-003 ケース 5
    let subs = extract_subnodes("DOC", "## A\n# H1\n### B\n");
    assert_eq!(subs.len(), 2, "A(h2) と B(h3) の 2 件、H1 由来なし");
    let b = subs
        .iter()
        .find(|s| s.heading_path.last().map(|x| x.as_str()) == Some("B"))
        .expect("B ノード");
    assert_eq!(b.heading_path, vec!["B".to_string()], "h2 コンテキスト None → 1 要素");
    assert!(subs.iter().all(|s| !s.heading_path.contains(&"H1".to_string())));
}

// ===========================================================================
// ケース 6: 対象外見出しレベル h4 以深は無視（クラッシュなし）
// ===========================================================================
#[test]
fn test_extract_h4_and_deeper_ignored() {
    // @ts: TS-LGX-003 ケース 6
    let subs = extract_subnodes("DOC", "#### H4\n##### H5\n###### H6\n");
    assert_eq!(subs.len(), 0);
}

// ===========================================================================
// ケース 7: ATX 見出し判定 — `#` の後に空白必須
// ===========================================================================
#[test]
fn test_extract_atx_requires_space_after_hash_no_space() {
    // @ts: TS-LGX-003 ケース 7(a)
    let subs = extract_subnodes("DOC", "##no space\n");
    assert_eq!(subs.len(), 0, "'##no space' は見出しでない");
}

#[test]
fn test_extract_atx_requires_space_after_hash_ok() {
    // @ts: TS-LGX-003 ケース 7(b)
    let subs = extract_subnodes("DOC", "## ok\n");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].heading_path, vec!["ok".to_string()]);
}

#[test]
fn test_extract_atx_bare_hashes_empty_anchor_skipped() {
    // @ts: TS-LGX-003 ケース 7(c)
    let subs = extract_subnodes("DOC", "##\n");
    assert_eq!(subs.len(), 0, "'##' のみは正規化後 anchor 空 → スキップ");
}

// ===========================================================================
// ケース 8: closing `#` の除去
// ===========================================================================
#[test]
fn test_extract_closing_hashes_stripped() {
    // @ts: TS-LGX-003 ケース 8
    let subs = extract_subnodes("DOC", "## 概要 ##\n");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].heading_path, vec!["概要".to_string()]);
    assert_eq!(subs[0].anchor, "概要", "anchor も closing # 除去済");
    assert_eq!(subs[0].id, "DOC#52567a07b126507c");
}

// ===========================================================================
// ケース 9: content_range のバイト半開区間 [body_start, end_byte)
// ===========================================================================
#[test]
fn test_extract_content_range_byte_halfopen() {
    // @ts: TS-LGX-003 ケース 9
    let content = "## A\nbodyA\n## B\nbodyB\n";
    let subs = extract_subnodes("DOC", content);
    assert_eq!(subs.len(), 2);
    // "## A\n" = 5 bytes -> body_start_A = 5. "## B" line_start = 11.
    let line_start_b = content.find("## B").expect("'## B' 位置");
    assert_eq!(subs[0].content_range, (5, line_start_b));
    // body_start_B = "## A\nbodyA\n## B\n".len()
    let body_start_b = content.find("bodyB").expect("'bodyB' 位置");
    assert_eq!(subs[1].content_range, (body_start_b, content.len()));
    for s in &subs {
        assert!(s.content_range.0 <= s.content_range.1);
        assert!(s.content_range.1 <= content.len());
    }
}

// ===========================================================================
// ケース 10: 空見出し・装飾のみ見出し → 正規化後空文字列でスキップ
// ===========================================================================
#[test]
fn test_extract_blank_heading_skipped() {
    // @ts: TS-LGX-003 ケース 10(a)
    let subs = extract_subnodes("DOC", "## \n");
    assert_eq!(subs.len(), 0);
}

#[test]
fn test_extract_decoration_only_h2_skipped_and_resets_context() {
    // @ts: TS-LGX-003 ケース 10(b)
    let subs = extract_subnodes("DOC", "## **__**\n### C\n");
    assert_eq!(subs.len(), 1, "装飾のみ h2 はスキップ、C(h3) のみ");
    assert_eq!(subs[0].heading_path, vec!["C".to_string()], "空 h2 → current_h2=None");
}

// ===========================================================================
// ケース 11: 正規化 5 段階の適用順序
// ===========================================================================
#[test]
fn test_normalize_decoration_and_collapse() {
    // @ts: TS-LGX-003 ケース 11(a)
    assert_eq!(normalize_heading("  **太字**  リンク  "), "太字 リンク");
}

#[test]
fn test_normalize_fullwidth_space() {
    // @ts: TS-LGX-003 ケース 11(b)
    assert_eq!(normalize_heading("A\u{3000}B"), "A B");
}

#[test]
fn test_normalize_tab_and_consecutive_whitespace() {
    // @ts: TS-LGX-003 ケース 11(c)
    assert_eq!(normalize_heading("a\tb  c"), "a b c");
}

// ===========================================================================
// ケース 12: 正規化の NFC（合成済み vs 分解の同一化）
// ===========================================================================
#[test]
fn test_normalize_nfc_decomposed_equals_composed() {
    // @ts: TS-LGX-003 ケース 12
    let decomposed = "e\u{0301}"; // e + U+0301 結合アクセント
    let composed = "\u{00e9}"; // é
    assert_eq!(normalize_heading(decomposed), normalize_heading(composed));
    assert_eq!(normalize_heading(decomposed), composed.to_string());
}

// ===========================================================================
// ケース 13: 正規化の冪等性（property）
// ===========================================================================
proptest::proptest! {
    #[test]
    fn test_normalize_idempotent(text in ".{0,64}") {
        // @ts: TS-LGX-003 ケース 13
        let once = normalize_heading(&text);
        let twice = normalize_heading(&once);
        proptest::prop_assert_eq!(&once, &twice);
        // 出力に U+3000・前後空白・装飾マーカーを含まない
        // （U+3000 / バックティックは char リテラルを直接マクロ内に書くと書式文字列誤認するため変数束縛）
        let fullwidth_space = '\u{3000}';
        let backtick = '`';
        let has_fullwidth = once.contains(fullwidth_space);
        let has_backtick = once.contains(backtick);
        proptest::prop_assert!(!has_fullwidth);
        proptest::prop_assert_eq!(once.trim(), once.as_str());
        proptest::prop_assert!(!once.contains('*'));
        proptest::prop_assert!(!once.contains('_'));
        proptest::prop_assert!(!has_backtick);
    }
}

// ===========================================================================
// ケース 14: is_auto_generated_fragment — ちょうど 16 文字小文字 hex
// ===========================================================================
#[test]
fn test_is_auto_generated_fragment_boundaries() {
    // @ts: TS-LGX-003 ケース 14
    assert!(is_auto_generated_fragment("0123456789abcdef")); // (a) 16 小文字 hex
    assert!(!is_auto_generated_fragment("0123456789ABCDEF")); // (b) 大文字
    assert!(!is_auto_generated_fragment("0123456789abcde")); // (c) 15 文字
    assert!(!is_auto_generated_fragment("0123456789abcdef0")); // (d) 17 文字
    assert!(!is_auto_generated_fragment("s:overview")); // (e) 明示
    assert!(!is_auto_generated_fragment("0123456789abcdeg")); // (f) 'g' は hex 外
}

// ===========================================================================
// ケース 15: validate_explicit_name — 文字制約・長さ・両端英数
// ===========================================================================
#[test]
fn test_validate_explicit_name_cases() {
    // @ts: TS-LGX-003 ケース 15
    assert!(validate_explicit_name("overview").is_ok()); // (a)
    assert!(validate_explicit_name("").is_err()); // (b) 空
    let len64: String = std::iter::repeat('a').take(64).collect();
    assert!(validate_explicit_name(&len64).is_err()); // (c) 64 > 63
    assert!(validate_explicit_name("-abc").is_err()); // (d) 先頭ハイフン
    assert!(validate_explicit_name("abc-").is_err()); // (e) 末尾ハイフン
    assert!(validate_explicit_name("ab.c").is_err()); // (f) 不正文字 '.'
    assert!(validate_explicit_name("a_b-c1").is_ok()); // (g) 中間 _/- は可
}

// ===========================================================================
// ケース 16: validate_explicit_name 長さ境界（63/64、property 補強）
// ===========================================================================
proptest::proptest! {
    #[test]
    fn test_validate_explicit_name_length_boundary(len in 1usize..=70) {
        // @ts: TS-LGX-003 ケース 16
        // 英数のみ（両端英数・文字制約は常に満たす）
        let name: String = std::iter::repeat('a').take(len).collect();
        let result = validate_explicit_name(&name);
        if len <= 63 {
            proptest::prop_assert!(result.is_ok(), "len {} は Ok", len);
        } else {
            proptest::prop_assert!(result.is_err(), "len {} は Err", len);
        }
    }
}

// ===========================================================================
// ケース 17: パース統合 — Document + 自動生成サブノード + ParentChild エッジ
// ===========================================================================
#[test]
fn test_parse_integration_document_subnodes_and_parentchild() {
    // @ts: TS-LGX-003 ケース 17
    // 注: parse_graph_from_str は todo!() のため本テストは panic で RED。
    // 期待値は parse_graph_from_str が返す TraceGraph（既存型）に対して束縛する。
    // .md 実体は project_root 配下に存在する必要があるため一時ディレクトリを用いる。
    let dir = std::env::temp_dir().join("tc_lgx_003_case17");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("doc.md"), "## First\nbody\n## Second\nbody\n");
    let toml_str = r#"
[[nodes]]
id = "DOC-LGX-001"
type = "DOC"
path = "doc.md"
"#;
    let graph = parse_graph_from_str(toml_str, &dir).expect("Ok");
    // Document 1 + AutoGenerated 2 = 3 ノード
    assert_eq!(graph.node_count(), 3);
    // AutoGenerated サブノードは親 DOC-LGX-001 への ParentChild エッジを 1 本ずつ持つ
    let pc = parent_child_edges(&graph);
    assert_eq!(pc.len(), 2);
    assert!(pc.iter().all(|e| e.from == "DOC-LGX-001" && e.kind == EdgeKind::ParentChild));
    // 各サブノードの parent_id・path が親と一致（SUBNODE-INV-2）
    for e in &pc {
        let sub = graph.node(&e.to).expect("サブノード存在");
        assert_eq!(sub.parent_id.as_deref(), Some("DOC-LGX-001"));
        assert_eq!(sub.path, "doc.md");
        assert_eq!(classify_subnode_kind(&sub.id), SubnodeKind::AutoGenerated);
    }
}

#[test]
fn test_parse_integration_zero_subnode_backward_compat() {
    // @ts: TS-LGX-003 ケース 17（境界: 見出しなし .md → ParentChild 0）
    let dir = std::env::temp_dir().join("tc_lgx_003_case17b");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("empty.md"), "本文のみ、見出しなし\n");
    let toml_str = r#"
[[nodes]]
id = "DOC-LGX-002"
type = "DOC"
path = "empty.md"
"#;
    let graph = parse_graph_from_str(toml_str, &dir).expect("Ok");
    assert_eq!(graph.node_count(), 1);
    assert_eq!(parent_child_edges(&graph).len(), 0);
}

// ===========================================================================
// ケース 18: 縮退 — 同一 heading_path の複数見出しは同一 ID に後勝ち統合
// ===========================================================================
#[test]
fn test_parse_integration_degenerate_last_wins() {
    // @ts: TS-LGX-003 ケース 18
    let dir = std::env::temp_dir().join("tc_lgx_003_case18");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("dup.md"), "## 概要\nA\n## 概要\nB\n");
    let toml_str = r#"
[[nodes]]
id = "DOC-LGX-003"
type = "DOC"
path = "dup.md"
"#;
    let graph = parse_graph_from_str(toml_str, &dir).expect("Ok（Warning/Err なし）");
    // 同一 heading_path "概要" は 1 ノードに縮退 → Document 1 + 縮退サブノード 1 = 2
    assert_eq!(graph.node_count(), 2);
    assert_eq!(parent_child_edges(&graph).len(), 1);
}

// ===========================================================================
// ケース 19: 明示優先 — 明示サブノードと衝突する自動生成はスキップ
// ===========================================================================
#[test]
fn test_parse_integration_explicit_takes_precedence() {
    // @ts: TS-LGX-003 ケース 19
    let dir = std::env::temp_dir().join("tc_lgx_003_case19");
    let _ = std::fs::create_dir_all(&dir);
    // "## 概要" は DOC-LGX-004 親で id = DOC-LGX-004#<hex> を生む。
    // 同 ID の明示ノードを graph.toml に置き、自動生成側スキップを検証。
    let auto_id = generate_subnode_id("DOC-LGX-004", &["概要".to_string()]);
    let _ = std::fs::write(dir.join("ex.md"), "## 概要\nbody\n");
    let toml_str = format!(
        r#"
[[nodes]]
id = "DOC-LGX-004"
type = "DOC"
path = "ex.md"

[[nodes]]
id = "{auto_id}"
type = "DOC-section"
path = "ex.md"
parent = "DOC-LGX-004"
"#
    );
    let graph = parse_graph_from_str(&toml_str, &dir).expect("Ok");
    // 当該 ID のノードは明示のまま（自動生成で上書きされない）。Document 1 + Explicit 1 = 2。
    assert_eq!(graph.node_count(), 2);
    // 明示優先は「グラフ上の当該ノードが graph.toml 由来の明示宣言（type_code=DOC-section）として
    // 残り、自動生成で上書きされない」ことで検証する。classify_subnode_kind は ID 文字列のみを見る
    // 純関数で、16桁hex の auto_id は宣言経緯に依らず AutoGenerated を返す（ケース17・DD-LGX-003 §2.2
    // 凍結契約と整合）。よって「明示優先」は宣言型で確認する（旧 assert は純関数に宣言経緯を求める
    // 矛盾で、ケース17 と両立不能だったため SRC[GREEN] 時に訂正）。
    assert_eq!(
        graph.node(&auto_id).map(|n| n.type_code.as_str()),
        Some("DOC-section"),
        "明示宣言ノード（DOC-section）が衝突する自動生成で上書きされず残る"
    );
    assert_eq!(classify_subnode_kind(&auto_id), SubnodeKind::AutoGenerated);
}

// ===========================================================================
// ケース 20: 部分失敗継続 — 一部 .md 不在/読込失敗をスキップして残りを処理
// ===========================================================================
#[test]
fn test_parse_integration_partial_failure_continues() {
    // @ts: TS-LGX-003 ケース 20
    let dir = std::env::temp_dir().join("tc_lgx_003_case20");
    let _ = std::fs::create_dir_all(&dir);
    // present.md は存在し h2 を持つ。missing.md は存在しない。
    let _ = std::fs::write(dir.join("present.md"), "## Present\nbody\n");
    let _ = std::fs::remove_file(dir.join("missing.md"));
    let toml_str = r#"
[[nodes]]
id = "DOC-LGX-005"
type = "DOC"
path = "missing.md"

[[nodes]]
id = "DOC-LGX-006"
type = "DOC"
path = "present.md"
"#;
    let graph = parse_graph_from_str(toml_str, &dir).expect("Ok（Err に昇格しない）");
    // 存在側のサブノード 1 が生成される（不在側はサブノード 0）。Document 2 + サブノード 1 = 3。
    assert_eq!(graph.node_count(), 3);
    let pc = parent_child_edges(&graph);
    assert_eq!(pc.len(), 1);
    assert!(pc.iter().all(|e| e.from == "DOC-LGX-006"));
}

// ===========================================================================
// ケース 21: 不正 edge kind / パス逸脱 → GraphParseError::ValidationError（exit 1 経路）
// ===========================================================================
#[test]
fn test_parse_integration_unknown_edge_kind_validation_error() {
    // @ts: TS-LGX-003 ケース 21(a)
    let dir = std::env::temp_dir().join("tc_lgx_003_case21a");
    let _ = std::fs::create_dir_all(&dir);
    let toml_str = r#"
[[nodes]]
id = "A-LGX-001"
type = "A"
path = "a.md"

[[nodes]]
id = "B-LGX-001"
type = "B"
path = "b.md"

[[edges]]
from = "A-LGX-001"
to = "B-LGX-001"
kind = "bogus_kind"
"#;
    let err = parse_graph_from_str(toml_str, &dir).expect_err("ValidationError を返す");
    assert!(matches!(err, GraphParseError::ValidationError(_)));
}

#[test]
fn test_parse_integration_path_traversal_validation_error() {
    // @ts: TS-LGX-003 ケース 21(b)
    let dir = std::env::temp_dir().join("tc_lgx_003_case21b");
    let _ = std::fs::create_dir_all(&dir);
    let toml_str = r#"
[[nodes]]
id = "A-LGX-002"
type = "A"
path = "../escape.md"
"#;
    let err = parse_graph_from_str(toml_str, &dir).expect_err("パス逸脱で ValidationError");
    assert!(matches!(err, GraphParseError::ValidationError(_)));
}

// ===========================================================================
// ケース 22: コードフェンス内 `#` 行も見出しとして抽出（v3 非認識、凍結 — M-1）
// ===========================================================================
#[test]
fn test_extract_code_fence_hash_still_extracted() {
    // @ts: TS-LGX-003 ケース 22
    let subs = extract_subnodes("DOC", "```\n## InCodeFence\n```\n");
    assert_eq!(subs.len(), 1, "フェンスを理由に除外しない（v3 非認識・凍結）");
    assert_eq!(subs[0].heading_path, vec!["InCodeFence".to_string()]);
    assert_eq!(subs[0].id, "DOC#eaa12937ce3ecf99");
}

// ===========================================================================
// ケース 23: extract_subnodes は extract_subnodes_with_levels(.., &[2,3]) の糖衣
// ===========================================================================
#[test]
fn test_extract_subnodes_is_sugar_for_levels_2_3() {
    // @ts: TS-LGX-003 ケース 23
    let content = "## H2\n### H3\n#### H4\n";
    let sugar = extract_subnodes("P", content);
    let explicit: Vec<AutoSubnode> = extract_subnodes_with_levels("P", content, &[2, 3]);
    assert_eq!(sugar, explicit);
}

// ===========================================================================
// ケース 24: heading_levels 内部属性 — 指定レベルのみ抽出（M-2、ID 無影響）
// ===========================================================================
#[test]
fn test_extract_with_levels_only_h2() {
    // @ts: TS-LGX-003 ケース 24(a)
    let subs = extract_subnodes_with_levels("DOC", "## A\n### B\n", &[2]);
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].heading_path, vec!["A".to_string()]);
}

#[test]
fn test_extract_with_levels_only_h3() {
    // @ts: TS-LGX-003 ケース 24(b)
    let subs = extract_subnodes_with_levels("DOC", "## A\n### B\n", &[3]);
    assert_eq!(subs.len(), 1);
    // levels=[3] 単独指定時 current_h2 は更新されないため B.heading_path == ["B"]
    assert_eq!(subs[0].heading_path, vec!["B".to_string()]);
}

#[test]
fn test_extract_with_levels_both() {
    // @ts: TS-LGX-003 ケース 24(c)
    let subs = extract_subnodes_with_levels("DOC", "## A\n### B\n", &[2, 3]);
    assert_eq!(subs.len(), 2);
}

// ===========================================================================
// ケース 25: extract_subnodes_with_levels の levels 空 → 空 Vec（クラッシュなし）
// ===========================================================================
#[test]
fn test_extract_with_empty_levels_returns_empty() {
    // @ts: TS-LGX-003 ケース 25
    let subs = extract_subnodes_with_levels("DOC", "## A\n### B\n", &[]);
    assert_eq!(subs.len(), 0);
}

// ===========================================================================
// ケース 26: 任意入力でも panic しない + 構造不変条件（property、堅牢性の決定論部）
// ===========================================================================
proptest::proptest! {
    #[test]
    fn test_extract_no_panic_and_invariants(content in ".{0,256}") {
        // @ts: TS-LGX-003 ケース 26
        let subs = extract_subnodes("P", &content);
        for s in &subs {
            // content_range.0 <= content_range.1 <= content.len()
            proptest::prop_assert!(s.content_range.0 <= s.content_range.1);
            proptest::prop_assert!(s.content_range.1 <= content.len());
            // heading_path に空要素を含まない
            proptest::prop_assert!(s.heading_path.iter().all(|h| !h.is_empty()));
            // id は {P}#{16hex} 形式
            let (prefix, frag) = s.id.split_once('#').expect("id は '#' を含む");
            proptest::prop_assert_eq!(prefix, "P");
            proptest::prop_assert_eq!(frag.len(), 16);
            proptest::prop_assert!(frag.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
        }
    }
}
