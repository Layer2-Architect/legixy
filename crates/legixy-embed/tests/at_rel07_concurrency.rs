// AT-LGX-001 連動 / NFR-LGX-001.REL.07 — SQLite 並行制御（busy_timeout 上限・無限リトライ禁止）。
//
// 検証対象: REL.07「並行呼出し時のロック待機は上限時間を設定し、超過時は失敗として返す。
//   暫定値 5000ms。無限リトライは禁止」。PERF.07（WAL）も配線として併せて確認。
//
// 設計: 同一 on-disk engine.db に 2 コネクションを開く。
//   ① locker（生 rusqlite 接続）が BEGIN EXCLUSIVE で write ロックを占有し続ける。
//   ② EmbeddingStore（短い busy_timeout=300ms）が upsert を試みる → 上限分だけ待機後に
//      SQLITE_BUSY を Err として返す（無限待機しない）ことを実時間で確認する。
//
// onnx 不要（純 SQLite 並行性）。

use std::time::{Duration, Instant};

use legixy_embed::{EmbedResult, Embedder, EmbeddingStore};
use legixy_graph::Node;
use rusqlite::Connection;

fn sample_node() -> Node {
    Node {
        id: "UC-LGX-001".to_string(),
        type_code: "UC".to_string(),
        path: "uc.md".to_string(),
        parent_id: None,
        anchor: None,
    }
}

fn sample_result() -> EmbedResult {
    // スタブ Embedder（onnx 不要）で決定的な EmbedResult を得る。
    Embedder::stub("rel07:test:plain:8", 8)
        .embed_node("rel07 concurrency probe", None, "UC-LGX-001")
        .expect("stub embed")
}

#[test]
fn rel07_busy_timeout_bounds_wait_and_returns_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("engine.db");

    // 競合側 store を先に開く（schema 初期化 + WAL、ロック非保持で安全に）。busy_timeout=300ms。
    let store_b = EmbeddingStore::open_on_disk(&db_path, 300).expect("open store_b");

    // locker が write ロックを占有（BEGIN EXCLUSIVE）。
    let locker = Connection::open(&db_path).expect("open locker");
    locker
        .busy_timeout(Duration::from_millis(0))
        .expect("locker busy_timeout 0");
    locker
        .execute_batch("BEGIN EXCLUSIVE;")
        .expect("locker acquires exclusive write lock");

    // store_b の書込みはロック競合 → busy_timeout(300ms) 内で SQLITE_BUSY を Err 返却。
    let node = sample_node();
    let result = sample_result();
    let started = Instant::now();
    let r = store_b.upsert_with_subnode_meta(&node, &result);
    let elapsed = started.elapsed();

    // ① 失敗として返る（無限リトライをしない）。
    assert!(
        r.is_err(),
        "ロック競合は Err（無限リトライ禁止、REL.07）。実際: {r:?}"
    );
    let msg = format!("{:?}", r.unwrap_err()).to_lowercase();
    assert!(
        msg.contains("lock") || msg.contains("busy"),
        "ロック由来のエラーである旨が報告される。実際のメッセージ: {msg}"
    );

    // ② 上限時間分は待機した（busy_timeout が効いている）。
    assert!(
        elapsed >= Duration::from_millis(250),
        "busy_timeout 分は待機する（>=~300ms）。実測 {elapsed:?}"
    );
    // ③ かつ無限には待たない（上限で打ち切り）。busy_timeout の十分上のマージン。
    assert!(
        elapsed < Duration::from_secs(3),
        "上限で打ち切り、無限待機しない。実測 {elapsed:?}"
    );

    eprintln!("[OBSERVE] REL.07 busy_timeout=300ms 競合書込み: elapsed={elapsed:?}, err 報告 OK");

    // 後始末: ロック解放（解放後は同 store で書込み成功することも確認）。
    locker.execute_batch("COMMIT;").expect("locker commit");
    store_b
        .upsert_with_subnode_meta(&node, &result)
        .expect("ロック解放後は書込み成功");
}
