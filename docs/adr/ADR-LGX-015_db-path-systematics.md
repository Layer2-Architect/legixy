Document ID: ADR-LGX-015

# ADR-LGX-015: DB パス/ファイル名の系統正準化 — `.legixy/engine.db` 正準 + `.trace-engine/` 読取フォールバック

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定、TRIAGE-2026-06-13 クラスタ B）
**対象**: SUPP-LGX-007 §2-4 / SUPP-LGX-010 C-4 / SUPP-LGX-006 §3 / SUPP-LGX-008 2.1 / SUPP-LGX-003 S2-22

## 1. 文脈（Context）

DB のパス・ファイル名に三系統の不一致がある（TRIAGE §3 クラスタ B）:

- **SPEC 系**（LGX-EXT-001 §4.3）: `.legixy/engine.db`
- **v3 実装 / dev ツール**: `.trace-engine/engine.db`（実バイナリ traceability-engine v0.4.0-alpha4 が使用する実体。本リポジトリの開発検証もこれを使う＝`.trace-engine.toml` と同じレイヤ、LGX-COMPAT-001 §6）
- **実在 v0.1.0 プロジェクト**: フィードバック DB の実体は `feedback.db`

legixy-the-tool（製品）の実装が、どのパスを正準とし、v3 データとどう相互運用するかを確定する必要がある。

## 2. 検討した選択肢（Options）

- **案 A**: `.trace-engine/` を維持（v3 完全互換、SPEC と乖離）
- **案 B（採用）**: `.legixy/engine.db` を正準とし、`.trace-engine/engine.db` を**読取フォールバック**として許容（不在時のみ）。v0.1.0 移行元の一次対象は `feedback.db`
- **案 C**: `.legixy/` へ完全移行し旧パスは migrate でのみ変換（フォールバックなし）

## 3. 判断（Decision）

案 B を採用する（人間裁定 2026-06-13）。

- **正準パス**: legixy-the-tool 製品の engine DB は `.legixy/engine.db`（SPEC / LGX-EXT-001 §4.3 準拠、ADR-LGX-014 の SPEC 準拠原則に整合）。
- **読取フォールバック**: `.legixy/engine.db` 不在時に `.trace-engine/engine.db` が存在すれば読取に限り使用（v3 バイナリが生成した既存データとの相互運用。書込は常に正準パスへ）。
- **v0.1.0 移行元**: migrate の一次対象ファイル名は `feedback.db`（実在 v0.1.0 の実体）。`engine.db`（user_version=0）も存在すれば対象とする（SUPP-008 2.1 案 A）。SPEC-LGX-008.REQ.01 の「engine.db を検出」文言と実態（feedback.db）の不一致は SPEC 改訂候補として申し送り（人間承認時に文言修正）。
- **レイヤ分離の明示**: dev ツール設定 `.trace-engine.toml` / `.trace-engine/engine.db` は「開発ツールの実バイナリが読む層」であり、legixy-the-tool 既定の `.legixy.toml` / `.legixy/engine.db` とは別レイヤ（CLAUDE.md / LGX-COMPAT-001 §6 の構図を踏襲）。

## 4. 結果（Consequences）

- TRIAGE クラスタ B 傘下 5 項目が一括解決（DB 不在判定、フォールバック、移行元ファイル名）。
- 具体パス文字列・フォールバック探索順の DD 凍結（SUPP-010 C-4 等）は本 ADR の方針下で DD が確定。
- v3 バイナリとの相互運用が読取フォールバックで担保される。
- **残存リスク**: 読取フォールバックで `.trace-engine/` の古い DB を誤って参照しないよう、フォールバックは「正準パス完全不在」時のみに限定する（DD で明示）。

## 5. 関連

- 統治: ADR-LGX-014（SPEC 準拠原則）
- 対象 SPEC: SPEC-LGX-008.REQ.01（移行元文言の申し送り）, LGX-EXT-001 §4.3
- 境界: LGX-COMPAT-001 §6（設定ファイル層分離）
- トリアージ: TRIAGE-2026-06-13 §3 クラスタ B
