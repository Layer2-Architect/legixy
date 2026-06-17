Document ID: ADR-LGX-019

# ADR-LGX-019: TRIAGE「要熟慮」4 件の裁定 — SPEC-003/004/007/010 の改訂（spec-change 2026-06-13）

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定、spec-change イベント）
**対象**: TRIAGE-2026-06-13 §4 の「要熟慮」4 件（#1/#8/#13/#19）。SPEC-LGX-003.REQ.10、SPEC-LGX-004.REQ.15、SPEC-LGX-007.REQ.08、SPEC-LGX-010.REQ.03

## 1. 文脈（Context）

ADR-LGX-018 が §4 の 16 件を批准した一方、SPEC 改訂を伴いうる「要熟慮」4 件は個別議論に回された。本 ADR は 4 件を裁定し、既承認 SPEC への意図的変更（spec-change イベント、ハードルール 1・6 の例外路）として記録する。①③は **UC ループ（2026-06-13）で surface した UC↔SPEC 不整合の解消**であり、SPEC を UC に追いつかせる gap-closing。

## 2. 判断（Decision）

| # | 項目 | 裁定 | SPEC 改訂 |
|---|---|---|---|
| ① | compile_context 返却セクション 5 vs 6 | **6 セクション化**。Custom Documents を 6 番目（Target Node Metadata の後、最変動部）として正式化。UC-LGX-002 の ContextResult（custom_documents）/ v3 実装と整合。旧「5」は UC との同期漏れ | SPEC-003 REQ.10 改訂 |
| ② | CheckCategory 完全性宣言 | **注記追加**。REQ.15 の完全性・severity 固定は「**検証結果（違反 finding）**」を指し、config 由来の**設定書き方助言**（`{id}` 誤記 Warning / area=="XX" Info）は対象外と明記。DocumentId 等の severity を 3 値化しない（実装は分離出力） | SPEC-004 REQ.15 注記 |
| ③ | observation 状態モデル | **`skipped` 終端を追加**（pending/analyzing/resolved/**skipped** の 4 値）。`analyze` が構造的に Proposal 変換不能（REQ.04 に対応規則なし。例: orphan_file/semantic_similarity）と判定した observation を skipped（終端）にし、**永久再 claim を解消**。【v3差分】v3 は変換不能を pending に戻し死蔵していた | SPEC-007 REQ.08 改訂 |
| ④ | drift `--against snapshot:label:<L>` 不在時 exit | **明示 label 形式の解決失敗 = ERROR + exit 1**（snapshot_id フォールバックなし）。`snapshot delete label:<L>`（exit 1）および REQ.03 既存の非対称性原則（ベースライン不在=exit0 / 壊れた状態=exit1）と対称化。曖昧形式 `snapshot:<token>` は従来どおり exit 0 維持 | SPEC-010 REQ.03 改訂 |

## 3. 結果（Consequences）

- TRIAGE §4 の RBA 阻害 20 件が全て決着（16=ADR-018 + 4=本 ADR）。CTX-INV-5（D-09）+ VAL-LGX-001 を残し RBA 着手前提がほぼ整う。
- **v3 観測挙動変更**（③④）は【v3差分】として各 SPEC に注記。引数契約（LGX-COMPAT-001）は不変。①は UC 整合化で v3 とも一致。②は注記のみ。
- **TP[SPEC] への影響**: TP-003/004/007/010 は green を維持（本改訂は gap-closing/明確化であり新規 RED 観点を生まない）。各 TP の該当観点（REQ.10 セクション順序 / REQ.15 severity 完全性 / observation 状態 / drift exit）は改訂後も委譲先が答える。embedding は再生成（ドリフト解消）。
- **下流影響**: ①は UC-LGX-002（既に custom_documents 記載、整合）、③は UC-LGX-008（既に「proposed/skipped」記載、整合）、④は UC-LGX-013（明示 label 失敗の代替フロー追記が望ましい — DD/UC レビューで反映）。

## 4. 関連

- 統治: ADR-LGX-014（SPEC 準拠原則）、ADR-LGX-018（§4 16 件批准）
- トリアージ: TRIAGE-2026-06-13 §4
- UC ループ連動: UC-LGX-002（①）, UC-LGX-008 / GAP-LGX-256/257（③）, UC-LGX-012 GAP-LGX-291 / UC-LGX-013（④）
- 後続: CTX-INV-5 正準定義（D-09）、VAL-LGX-001 再生成
