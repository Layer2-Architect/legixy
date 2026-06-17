Document ID: ADR-LGX-001

# ADR-LGX-001: migrate `--from`/`--to` を PATH 意味で正準化しバージョンは自動検出へ委ねる

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（人間裁定、AskUserQuestion 経由）
**対象**: SPEC-LGX-008 §3 REQ.06 / REQ.09、LGX-COMPAT-001 §4 #2

## 1. 文脈（Context）

- 背景: TP[SPEC] 敵対的精査を生存した GAP-LGX-157（BLOCKER）。`migrate --from/--to` に対し、凍結境界契約 LGX-COMPAT-001 §4 #2 は `<PATH>`（`--from` 必須・`--to` 既定 `--project-root`）、SPEC-LGX-008 REQ.06（旧版）は `--from v0.1.0 --to legixy`（バージョン文字列）と、同一フラグへ異なる意味を与えていた。
- 制約: LGX-COMPAT-001 は凍結済み境界契約（ハードルール 7）。v3 実バイナリも `from: PathBuf` で実装済み（`traceability-engine.v3/crates/te-cli/src/main.rs:66-73`）。
- 関連: SPEC-LGX-008 REQ.06/REQ.09、GAP-LGX-154（バージョン自動検出）、GAP-LGX-157。

## 2. 検討した選択肢（Options）

### 選択肢 A: 凍結 PATH 意味を正準とする（採用）

- 概要: REQ.06 を `--from <PATH>` 必須 / `[--to <PATH>]` 既定 `--project-root` に確定。バージョン意図は REQ.09 のバージョン自動検出（`PRAGMA user_version` 一次根拠）へ移管。凍結契約は無変更。
- 利点: 凍結契約・v3 実装事実の双方と整合（整合性）。既存ユーザの呼出が壊れない（堅牢性）。SPEC テキスト修正のみで実現（実現性）。
- 欠点: バージョンを明示指定する手段がない（自動検出の信頼に依存）。
- 影響範囲: SPEC-LGX-008 のみ。

### 選択肢 B: フラグをバージョン文字列に改訂

- 概要: `--from <VERSION>` / `--to <VERSION>` とし、凍結契約を改訂（COMPAT リビジョン + ADR 必須）。
- 利点: バージョン明示の意図が引数に現れる。
- 欠点: 凍結契約の改訂（「既に凍結済み」ステータスの崩壊）。v3 実装と非互換。パス指定手段を失い別フラグ `--source <PATH>` の新設が必要。
- 影響範囲: LGX-COMPAT-001・SPEC-LGX-008・将来の DD/TS/TC、既存ユーザの呼出互換。

## 3. 判断（Decision）

選択肢 A を採用する（人間裁定 2026-06-10）。

理由:

- 凍結契約（ハードルール 7）と v3 実装事実の両方に整合し、契約改訂という重い手続きと互換破壊を回避できる。
- バージョン判定は GAP-LGX-154 で確定した自動検出（user_version 一次根拠）が v3 実在機構であり、引数指定より堅牢。

## 4. 結果（Consequences）

### 期待される効果
- BLOCKER GAP-LGX-157 の解消、migrate の CLI シグネチャ確定により下流 DD/TS/TC が着手可能になる。
- 凍結契約の安定性維持（凍結後無変更の前例を守る）。

### 受け入れる代償
- バージョン明示指定の手段を持たない（自動検出が誤る場合は矛盾特徴 Error〔REQ.09〕で停止する設計で補完）。

### 残存リスク
- 自動検出の誤判定 → REQ.09 の「矛盾特徴は Error」で停止し、無言の誤マイグレーションは起きない。

## 5. 関連

- closes: GAP-LGX-157
- 関連 GAP: GAP-LGX-154（旧 147/149 統合）
- 凍結契約: LGX-COMPAT-001 §4 #2（無変更）
