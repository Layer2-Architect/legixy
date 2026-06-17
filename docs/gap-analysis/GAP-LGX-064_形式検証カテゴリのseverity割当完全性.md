Document ID: GAP-LGX-064

# GAP-LGX-064: 形式検証カテゴリ（FileExistence / DocumentId / ChainIntegrity / Freshness）の severity が未明示

**親 TP**: TP-LGX-004
**観点出典**: TP-LGX-004 §2.3 観点 S1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: GENUINE として維持。REQ.03 は severity 4 段階を定義し、REQ.07（subnode→Error）・REQ.10/14（Warning）・REQ.11/12/13（個別指定）は severity を pin するが、**基幹形式カテゴリ FileExistence / DocumentId / ChainIntegrity / OrphanFile の既定 severity が SPEC 本文に存在しない**ことを確認した。NFR-LGX-001.OBS.06 は 4 段階の定義のみでカテゴリ別割当を持たず、慣例仕様（old.source）には割当が実在するが SPEC は §2 で参照するのみで本文に集約していない。G1 ゲート（Error=0）に直結する基幹カテゴリの severity が SPEC として未確定。**severity: 中（theme: severity-completeness / G1-gate-impact）**。
**統合（DUPLICATE 吸収）**: 旧 GAP-LGX-073（DocumentId severity）・旧 GAP-LGX-074（node-level DAG severity）は各自の本文で「GAP-064 の具体事例」と自認しており、本 GAP に統合・削除した。DocumentId 不一致=Error / 欠落=Warning、および CTX-INV-4（LEGIXY-SPEC-001 §10.1 のグラフ全体 DAG）と subnode 拡張（subnode_spec §7.1「サブノード含めたサイクル検出」）の severity も本 GAP の解決時に同じ割当表で一括 pin すること。

## 1. 観点

REQ.03 は severity 4 段階（Error/Warning/Info/Ok）を定義し、サブノード不変条件（REQ.07=Error）・UnresolvedEdge（REQ.10=Warning）・各 Id 系（REQ.11〜14）の severity は個別に明示されている。しかし REQ.01 が列挙する基幹形式カテゴリのうち **FileExistence / DocumentId / ChainIntegrity / Freshness の severity が明示されていない**。severity 割当の完全性（全カテゴリが一意の severity を持つこと）が SPEC として保証されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-004 §3 REQ.01 はカテゴリ名を列挙するのみで severity を付与していない。Freshness は REQ.09 で「Warning」と読めるが、FileExistence / DocumentId / ChainIntegrity は severity の言及がない（NFR-LGX-001.OBS.06 は段階定義のみでカテゴリ別割当を持たない）。

## 3. 期待される情報

SPEC に追加されるべき記述:

- REQ.01 の各カテゴリ（FileExistence / DocumentId / ChainIntegrity / OrphanFile / Freshness）の既定 severity の明示
- 特に G1 ゲート（Error=0）に影響する Error 級カテゴリと、Warning/Info に留まるカテゴリの分類表
- severity が設定で変更可能か固定かの方針

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-001: どの検証失敗が G1 ゲートを止めるかが利用者に不明
- 下流の TS / TC: 各カテゴリの期待 severity が書けず、終了コード期待値も連動して不定
- 他の TP / GAP との依存関係: GAP-LGX-073（DocumentId severity）・GAP-LGX-074（DAG severity）は本 GAP の具体事例

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-004 v0.7.0（人間裁定 2026-06-10）: REQ.01 各カテゴリに severity を明示し REQ.15 severity 割当表（割当完全性保証）を新設。裁定 (a) グラフ全体 DAG は新カテゴリ GraphDag【Error】（v3 は SubnodeDag 名で報告 — validation.rs:59 実測。ADR-LGX-008）、(b) DocumentId 行欠落は Error（v3 実挙動 document_id.rs:81 と一致）。旧 GAP-073/074 を吸収。

## 6. 関連 ADR

該当なし。
