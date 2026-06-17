# Document ID: QSET-LGX-001

**親 SPEC**: SPEC-LGX-001
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-001（legixy 全体要求）に対してフロントエンド検査器が検出した不足・曖昧性・矛盾・境界不明を、開発者が回答可能な形に変換したもの。

本 SPEC は 8 つの下位 SPEC を束ねる傘 SPEC であるため、検出の中心は **下位 SPEC 群と凍結済み境界契約（LGX-COMPAT-001 の 19 サブコマンド）との網羅性**に置かれた。

---

## Q1: 境界整合性 / UC 生成可能性 — 未割当サブコマンドの SPEC オーナー

**質問**: 凍結済みの 19 サブコマンド（LGX-COMPAT-001 §4）のうち、以下 5 つが下位 SPEC のどの「対応 UC」「機能カテゴリ」にも明示的に割り当てられていません。これらの SPEC オーナーをどう確定しますか?

| サブコマンド | 機能 | 現状の言及 |
|---|---|---|
| `snapshot`（create/list/delete） | embedding スナップショット管理 | どの SPEC にも記述なし |
| `drift <id> --against snapshot:` | スナップショット間ドリフト比較 | SPEC-006 REQ.05 は check 内 drift 検出のみ。standalone コマンド + snapshot 比較は未規定 |
| `report` | 全リンク類似度＋候補一覧 | SPEC-006 REQ.11 が consumer として言及するのみ |
| `calibrate` | 類似度分布／推奨閾値 | 同上 |
| `refresh-subnodes` | 見出しリネーム時のサブノード ID 連鎖反映 | SPEC-002 はサブノード生成のみ。リネーム連鎖反映は未規定 |

**SPEC 上の該当箇所**: SPEC-LGX-001 §3 REQ.02（機能カテゴリ）, REQ.08（Surface 分離）

**選択肢**:

- [ ] 選択肢 A: 既存 SPEC を拡張（snapshot/drift/report/calibrate → SPEC-004 または SPEC-006、refresh-subnodes → SPEC-002）に分配する
- [x] 選択肢 B: 新規 **SPEC-LGX-010**（スナップショット・レポート・キャリブレーション系 Admin コマンド）を独立させ、REQ.02 を 9 カテゴリに拡張する
- [ ] その他: <自由記述>

**回答**:

**選択肢 B を採用**（2026-06-07 開発者決定・AI 起草）。

新規 **SPEC-LGX-010（embedding 運用・監査）** を新設し、`snapshot`（create/list/delete）・standalone `drift --against`・`report`・`calibrate` の 4 コマンドを割り当てる。REQ.02 の機能カテゴリは 9 に拡張。`refresh-subnodes` は SPEC-LGX-002（グラフ基盤）の拡張とする（QSET-LGX-002 Q3 参照）。

- 根拠: v3 実装で 4 コマンドは全て EmbeddingStore（engine.db の embeddings / embedding_snapshots テーブル）の consumer として強く凝集する（`te-cli/src/commands/{snapshot,drift,report,calibrate}.rs`）。SPEC-006 はエンジン（embedding 生成・ドリフト検出）に純化し、SPEC-006 REQ.11 の bulk similarity API が SPEC-006↔010 の境界面となる。`refresh-subnodes` は graph.toml の構造書き換えであり embedding 非依存。
- 手続: 新規 SPEC はハードルール 4 に従い `docs/traceability/graph.toml` へ登録し、SPEC-LGX-010 自体に前段ループ（QSET）を発行する。

---

## Q2: 矛盾 / 完全性 — 孤児 UC（UC-010 / UC-011）の親 SPEC

**質問**: SPEC-LGX-001.REQ.02 は機能カテゴリを 8 と宣言し「UC-LGX-001〜009 の網羅」と検証方法に記載しています。しかし実際には **UC-LGX-010（トレーサビリティ健全性監査）** と **UC-LGX-011（閾値キャリブレーション）** が存在し、どの下位 SPEC の「対応 UC」欄にも宣言されていません。この 2 UC の親 SPEC・親カテゴリをどう確定しますか?（Q1 の `report` / `calibrate` 割当と連動します）

**SPEC 上の該当箇所**: SPEC-LGX-001 §3 REQ.01, REQ.02、各下位 SPEC ヘッダ表「対応 UC」欄

**回答**:

（2026-06-07 開発者決定・AI 起草）

**UC-LGX-010（健全性監査 = report）・UC-LGX-011（閾値キャリブレーション = calibrate）とも、親 SPEC を新設の SPEC-LGX-010 とする**（Q1 の決定に従う）。SPEC-LGX-001 REQ.01/REQ.02 は「UC-LGX-001〜011 の網羅・9 機能カテゴリ」に改訂し、SPEC-LGX-010 のヘッダ表「対応 UC」欄に UC-LGX-010/011 を宣言する。

---

## Q3: 用語確認 / 責務範囲 — crate 分割の正準リスト

**質問**: REQ.03 はワークスペース crate を `legixy-core, legixy-graph, legixy-db, 将来的に legixy-cli 等` と例示します。一方 SPEC-006 / SPEC-008 本文は `legixy-core / legixy-check / legixy-ctx / legixy-nav / legixy-embed / legixy-feedback` を列挙しています。crate 分割の正準リスト（RBD/DD 段階で凍結すべき構造境界）はどちらですか? 言語固有要素は DD で初出が原則ですが、crate 名は複数 SPEC に散在しており RBD のパッケージ境界に影響します。

**SPEC 上の該当箇所**: SPEC-LGX-001 §3 REQ.03、SPEC-LGX-006 REQ.11、SPEC-LGX-008 REQ.07

**選択肢**:

- [ ] 選択肢 A: REQ.03 の `legixy-graph/legixy-db` を正準とし、他 SPEC の機能別 crate 名を例示扱いに降格
- [ ] 選択肢 B: 機能別 crate 群（core/check/ctx/nav/embed/feedback）を正準とし、REQ.03 を更新
- [x] その他（crate 分割は DD で確定し SPEC では crate 名を抽象化する 等）: 下記回答参照

**回答**:

**その他案を採用**（2026-06-07 開発者決定・AI 起草）: **SPEC 段階では crate 名を正準化しない**。

- SPEC 本文中の crate 名（REQ.03 の `legixy-graph/legixy-db`、SPEC-006/008 の機能別 crate 群）は全て「例示」に降格し、「crate 分割は DD 段階で凍結する」注記を付す（ICONIX 二段化規律: 言語固有要素は DD 初出。crate 名は Rust 固有要素）。
- RBD ではパッケージ境界を言語非依存の「機能パッケージ」として表現する。
- DD での凍結時の初期候補は LGX-COMPAT-001 §2 の 10 crate 写像（`te-core/te-graph/te-db/te-ctx/te-check/te-nav/te-embed/te-feedback/te-mig/te-cli` → `legixy-*`）とする。COMPAT §2 は凍結対象 (a)〜(f) 外の参考情報であり、DD で変更可。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 1 |
| 複数解釈 | 0 |
| 例外未定義 | 0 |
| 境界不明 | 1 |
| 矛盾 | 1 |
| 非機能不足 | 0 |
| 合計 | 3 |

## メモ

- 本票の質問は **後段コンパイル（UC/RB/SEQ/DD/TS/TC/SRC）に必要な情報の不足** に結びついている。設計判断を AI が代行する目的では発行されない。
- Q1/Q2 は新規 SPEC 追加（ハードルール 4）または既存 SPEC 改訂（ハードルール 1）を伴う可能性があり、いずれも人間の決定領域。
- 回答が確定したら SPP-LGX-001 として SPEC 差分案を発行する。
