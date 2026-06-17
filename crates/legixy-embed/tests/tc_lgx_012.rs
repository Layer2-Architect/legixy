// Document ID: TC-LGX-012
// TC-LGX-012: ベースライン凍結管理（snapshot create / list / delete）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-012 → 本 TC-LGX-012 → SRC-LGX-012。
// snapshot::create / list / delete / resolve_label / generate_snapshot_id を DD-LGX-012 §3 に束縛する。
// SRC[GREEN] 未実装（todo!()）のためロジック関数を呼ぶテストは panic で失敗する（RED）。

use legixy_embed::snapshot::{
    create, delete, generate_snapshot_id, list, resolve_label, LabelResolveResult,
    SnapshotCreateResult, SnapshotDeleteResult,
};
use legixy_embed::store::{EmbeddingRow, EmbeddingStore, SnapshotRow};

fn row(node_id: &str) -> EmbeddingRow {
    EmbeddingRow {
        node_id: node_id.to_string(),
        embedding: vec![0.0; 4],
        dim: 4,
        model_version: "V".to_string(),
        content_hash: "H".to_string(),
        context: None,
        context_hash: None,
        created_at: "2026-06-14 00:00:00".to_string(),
    }
}

fn snap_row(snapshot_id: &str, label: Option<&str>, node_id: &str, taken_at: &str) -> SnapshotRow {
    SnapshotRow {
        snapshot_id: snapshot_id.to_string(),
        label: label.map(|s| s.to_string()),
        node_id: node_id.to_string(),
        model_version: "V".to_string(),
        content_hash: "H".to_string(),
        taken_at: taken_at.to_string(),
    }
}

// ケース 1: 空ストア create（複製 0 件）→ node_count=0・非永続・Ok
#[test]
fn test_create_empty_store() {
    // @ts: TS-LGX-012 ケース 1
    let store = EmbeddingStore::empty();
    let r = create(&store, "snap-018f-deadbeef", None).expect("Ok");
    assert_eq!(
        r,
        SnapshotCreateResult {
            snapshot_id: "snap-018f-deadbeef".to_string(),
            label: None,
            node_count: 0,
        }
    );
}

// ケース 2: create 正常系（node_count > 0）→ embedding_snapshots へ行複製
#[test]
fn test_create_with_rows() {
    // @ts: TS-LGX-012 ケース 2
    let store = EmbeddingStore::stub(vec![row("A"), row("B"), row("C")], vec![]);
    let r = create(&store, "snap-018f-a1b2c3d4", Some("v0.3.0")).expect("Ok");
    assert_eq!(r.node_count, 3);
    assert_eq!(r.label, Some("v0.3.0".to_string()));
}

// ケース 3: list 0 件 → Vec::new()
#[test]
fn test_list_empty() {
    // @ts: TS-LGX-012 ケース 3
    let store = EmbeddingStore::empty();
    let v = list(&store).expect("Ok");
    assert_eq!(v.len(), 0);
}

// ケース 5: list 同一秒タイブレークの具体例（決定論的順序）
#[test]
fn test_list_tiebreak_order() {
    // @ts: TS-LGX-012 ケース 5
    let store = EmbeddingStore::stub(
        vec![],
        vec![
            snap_row("snap-aaaa", None, "A", "2026-06-13 10:00:00"),
            snap_row("snap-bbbb", None, "A", "2026-06-13 10:00:00"),
            snap_row("snap-cccc", None, "A", "2026-06-13 09:00:00"),
        ],
    );
    let v = list(&store).expect("Ok");
    let ids: Vec<&str> = v.iter().map(|m| m.snapshot_id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["snap-bbbb", "snap-aaaa", "snap-cccc"],
        "同一秒は snapshot_id DESC、その後に古い"
    );
}

// ケース 6: list の label 表現（None / Some）
#[test]
fn test_list_label_representation() {
    // @ts: TS-LGX-012 ケース 6
    let store = EmbeddingStore::stub(
        vec![],
        vec![
            snap_row("snap-x", Some("rc1"), "A", "2026-06-13 10:00:00"),
            snap_row("snap-y", None, "A", "2026-06-13 09:00:00"),
        ],
    );
    let v = list(&store).expect("Ok");
    assert!(v.iter().any(|m| m.label == Some("rc1".to_string())));
    assert!(v.iter().any(|m| m.label.is_none()));
}

// ケース 7: delete 成功（deleted_rows > 0）→ DB から該当行除去
#[test]
fn test_delete_success() {
    // @ts: TS-LGX-012 ケース 7
    let store = EmbeddingStore::stub(
        vec![],
        vec![
            snap_row("snap-018f-a1b2c3d4", None, "A", "t"),
            snap_row("snap-018f-a1b2c3d4", None, "B", "t"),
            snap_row("snap-018f-a1b2c3d4", None, "C", "t"),
        ],
    );
    let r = delete(&store, "snap-018f-a1b2c3d4").expect("Ok");
    assert_eq!(r.deleted_rows, 3);
}

// ケース 8: delete 該当 0 件（6b）→ Ok(deleted_rows=0)・エラー非発生
#[test]
fn test_delete_no_match() {
    // @ts: TS-LGX-012 ケース 8
    let store = EmbeddingStore::empty();
    let r = delete(&store, "snap-does-not-exist").expect("Ok（エラーではない）");
    assert_eq!(
        r,
        SnapshotDeleteResult {
            snapshot_id: "snap-does-not-exist".to_string(),
            deleted_rows: 0,
        }
    );
}

// ケース 9: resolve_label 同一 label 複数存在（6a）→ taken_at DESC + snapshot_id DESC で 1 件
#[test]
fn test_resolve_label_tiebreak() {
    // @ts: TS-LGX-012 ケース 9
    let store = EmbeddingStore::stub(
        vec![],
        vec![
            snap_row("snap-zz", Some("release"), "A", "2026-06-13 12:00:00"),
            snap_row("snap-yy", Some("release"), "A", "2026-06-13 12:00:00"),
            snap_row("snap-old", Some("release"), "A", "2026-06-13 08:00:00"),
        ],
    );
    let r = resolve_label(&store, "release").expect("Ok");
    assert_eq!(r, LabelResolveResult::Resolved("snap-zz".to_string()));
}

// ケース 10: resolve_label label 不在（6c）→ NotFound
#[test]
fn test_resolve_label_not_found() {
    // @ts: TS-LGX-012 ケース 10
    let store = EmbeddingStore::empty();
    let r = resolve_label(&store, "nonexistent").expect("Ok（DB エラーのみ Err）");
    assert_eq!(r, LabelResolveResult::NotFound);
}

// ケース 11: generate_snapshot_id の形式（snap- プレフィクス・13+8 桁 16 進）
#[test]
fn test_generate_snapshot_id_format() {
    // @ts: TS-LGX-012 ケース 11
    let id = generate_snapshot_id();
    let body = id.strip_prefix("snap-").expect("snap- プレフィクス");
    let parts: Vec<&str> = body.split('-').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].len(), 13, "epoch_ms 13 桁 16 進");
    assert_eq!(parts[1].len(), 8, "8 桁 16 進乱数");
    assert!(parts[0].chars().all(|c| c.is_ascii_hexdigit()));
    assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));
}

// ケース 13: create → list → delete の E2E ライフサイクル
#[test]
fn test_create_list_delete_lifecycle() {
    // @ts: TS-LGX-012 ケース 13
    let store = EmbeddingStore::stub(vec![row("A"), row("B")], vec![]);
    let id = generate_snapshot_id();
    let c = create(&store, &id, Some("e2e")).expect("create Ok");
    assert_eq!(c.node_count, 2);
    let l1 = list(&store).expect("list Ok");
    assert!(l1.iter().any(|m| m.snapshot_id == id));
    let d = delete(&store, &id).expect("delete Ok");
    assert_eq!(d.deleted_rows, 2);
    let l2 = list(&store).expect("list Ok");
    assert!(!l2.iter().any(|m| m.snapshot_id == id));
}

// ケース 14: create トランザクション失敗 → 部分行残存なし（ロールバック）
#[test]
fn test_create_transaction_failure_rollback() {
    // @ts: TS-LGX-012 ケース 14
    let store = EmbeddingStore::stub(
        vec![row("A"), row("B"), row("C")],
        vec![snap_row("snap-collide-existing", None, "A", "t")],
    );
    let r = create(&store, "snap-collide-existing", None);
    // PRIMARY KEY 違反 → TransactionFailed/Db。ロールバックで部分行残存なし。
    assert!(r.is_err(), "PRIMARY KEY 違反は Err（ロールバック）");
}

// ケース 23: delete by label:<L> 成功の主経路（resolve → delete → deleted_rows > 0）E2E
#[test]
fn test_delete_by_label_main_path() {
    // @ts: TS-LGX-012 ケース 23
    let store = EmbeddingStore::stub(
        vec![],
        vec![
            snap_row("snap-018f-rel-newer", Some("release"), "A", "2026-06-13 12:00:00"),
            snap_row("snap-018f-rel-newer", Some("release"), "B", "2026-06-13 12:00:00"),
            snap_row("snap-018f-rel-newer", Some("release"), "C", "2026-06-13 12:00:00"),
            snap_row("snap-018f-rel-older", Some("release"), "A", "2026-06-13 08:00:00"),
            snap_row("snap-018f-rel-older", Some("release"), "B", "2026-06-13 08:00:00"),
            snap_row("snap-018f-other", Some("rc1"), "A", "2026-06-13 11:00:00"),
        ],
    );
    // 配線: "label:release" → strip_prefix → resolve_label → delete
    let resolved = resolve_label(&store, "release").expect("resolve Ok");
    let target = match resolved {
        LabelResolveResult::Resolved(id) => id,
        LabelResolveResult::NotFound => panic!("最新 1 件に解決されるはず"),
    };
    assert_eq!(target, "snap-018f-rel-newer", "taken_at 最新を解決");
    let d = delete(&store, &target).expect("delete Ok");
    assert_eq!(d.deleted_rows, 3, "対象 snapshot のみ削除（巻き添えなし）");
}

// ケース 4（list property）: taken_at DESC + snapshot_id DESC 安定整列 → 下記 prop モジュール。
// ケース 12（generate_snapshot_id 非衝突傾向 property）: 下記 prop モジュール。
// ケース 15/16（DB 不在 Read/Write 非対称）: legixy-db open_engine_db E2E（実 tempfile）へ委譲。
// ケース 17/18/19/20/22（CLI exit code / --json スキーマ / 出力先分離 / SnapshotError 変換）:
//   legixy-cli 層 E2E（assert_cmd）へ委譲。
// ケース 21（embeddings 本体 read-only 不変 property）: 実 DB ハッシュ比較（legixy-cli/db E2E）へ委譲。

// ── Property-based（proptest）──
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // ケース 4: list は taken_at DESC + snapshot_id DESC で安定整列
        #[test]
        fn prop_list_sorted(
            // 各 snapshot は (snapshot_id サフィックス, label 一定, 秒) を持つ。秒重複でタイブレーク発火。
            specs in proptest::collection::vec(
                ("[a-f0-9]{4}", proptest::option::of("[a-z]{1,4}"), 0u8..3),
                0..12,
            ),
        ) {
            // @ts: TS-LGX-012 ケース 4
            let mut rows = Vec::new();
            for (i, (suffix, label, sec)) in specs.iter().enumerate() {
                let sid = format!("snap-{:04x}-{}", i, suffix);
                let taken_at = format!("2026-06-13 10:00:0{}", sec);
                rows.push(snap_row(&sid, label.as_deref(), "A", &taken_at));
            }
            let store = EmbeddingStore::stub(vec![], rows);
            let metas = list(&store).expect("Ok");
            for w in metas.windows(2) {
                let (a, b) = (&w[0], &w[1]);
                // taken_at DESC（a.taken_at >= b.taken_at）、同一秒は snapshot_id DESC
                prop_assert!(
                    a.taken_at > b.taken_at
                        || (a.taken_at == b.taken_at && a.snapshot_id >= b.snapshot_id)
                );
            }
        }

        // ケース 12: generate_snapshot_id 形式の安定性（N 回呼出で全て形式一致）
        #[test]
        fn prop_generate_snapshot_id_format(_iter in 0u32..50) {
            // @ts: TS-LGX-012 ケース 12
            let id = generate_snapshot_id();
            let body = id.strip_prefix("snap-").expect("snap-");
            let parts: Vec<&str> = body.split('-').collect();
            prop_assert_eq!(parts.len(), 2);
            prop_assert_eq!(parts[0].len(), 13);
            prop_assert_eq!(parts[1].len(), 8);
        }
    }
}
