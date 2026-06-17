Document ID: VAL-LGX-001

# VAL-LGX-001: 外部照合記録（横断的妥当性確認）

| 項目 | 内容 |
|------|------|
| Document ID | VAL-LGX-001 |
| Version | 1.0.0 |
| Status | Approved（前身 VAL-LX-001 の G0 通過を継承。下記 §1 参照） |
| Date | 2026-06-13（前身照合実施 2026-04-17） |
| 前身（source-of-record） | VAL-LX-001 v1.1.0（`docs/specs-supplement/references/VAL-LX-001_外部照合記録.md`） |
| 照合対象 | LEGIXY-SPEC-001、LGX-EXT-001、LGX-EXT-002、SPEC-LGX-001〜010、NFR-LGX-001 |

---

## 1. 本文書の位置づけと前身の継承

legixy は traceability-engine v3（旧称 lexigy）を OSS 公開向けにリブランドしたものであり、**新旧の SPEC 10 ファイルは内容が完全一致**している（SUPP-LGX-000 §付随的発見の diff 照合で確認済み）。したがって、前身 lexigy/v3 era に対して実施された外部照合（VAL-LX-001）の妥当性確認結果は、内容同一の legixy SPEC にそのまま継承される。

- **外部照合の実体**: 2026-04-17 に予備レビュー（Claude Code 内部 AI）+ 外部 AI 照合（ChatGPT + Gemini）を実施。HIGH 3 / MEDIUM 7 / LOW 4 / INFO 3 = 計 17 Finding を記録し、対応必要な 14 件すべてを SPEC/NFR に反映、**G0 ゲート通過**。完全な記録は前身 VAL-LX-001（references/ に保全）にある。
- **本 VAL-LGX-001 の役割**: (a) 前身の外部照合を legixy の横断的妥当性確認として正式に採用する、(b) 各 Finding を**それを解決した legixy（LGX 名義）SPEC/NFR の正準位置**にマッピングし、SPEC が引用する Finding 番号（E-01 等）のトレーサビリティを成立させる、(c) 前身が「lexigy/V-DRS/TE-NEXT-EXT」名義で記述した解決先を legixy 名義（LEGIXY-SPEC-001 / LGX-EXT-001/002 / SPEC-LGX-* / NFR-LGX-001）へ読み替える。

> 名義対応: V-DRS-SPEC-001 → **LEGIXY-SPEC-001**、TE-NEXT-EXT-001 → **LGX-EXT-001**、TE-NEXT-EXT-002 → **LGX-EXT-002**、SPEC-LX-NNN → **SPEC-LGX-NNN**、NFR-LX-001 → **NFR-LGX-001**。

## 2. Finding カタログと LGX SPEC 解決マッピング

前身 VAL-LX-001 §5（予備レビュー P-01〜P-10）/ §6（外部照合 E-01〜E-07）の全 Finding と、legixy における解決先。詳細な観察・評価本文は前身 VAL-LX-001 を参照。

### HIGH（G0 通過前に必須対応、すべて対応済）

| Finding | 概要 | legixy 解決先 | 状態 |
|---|---|---|---|
| **E-01** ダングリング・エッジ | 参照切れエッジへの振る舞い未規定 | **CTX-INV-5（LEGIXY-SPEC-001 §10.1、本セッション 2026-06-13 で §10 へ正準定義を追加 = D-09）** / SPEC-LGX-002.REQ.11（未解決エッジ許容性・部分グラフ構築） / SPEC-LGX-004（UnresolvedEdge を Warning 報告） | 🟢 |
| **E-02** クレデンシャル漏洩 | API キーが context_log に平文永続化されるリスク | NFR-LGX-001.SEC.05（ログ記録前マスキング義務） | 🟢 |
| **P-05** PERF バジェット整合性 | cold start + parse + 走査の 200ms 予算 | NFR-LGX-001 §13（再評価トリガ、Phase 4 実測） | 🟢 |

### MEDIUM（対応推奨、すべて対応済）

| Finding | 概要 | legixy 解決先 | 状態 |
|---|---|---|---|
| **P-02** DAG 検証対象エッジ | Chain/Custom/ParentChild のどれを DAG 検証対象とするか | SPEC-LGX-002.REQ.07（全エッジ種別対象） | 🟢 |
| **P-04** Contextual Retrieval 障害時動作 | タイムアウト・リトライ・フォールバック未定義 | SPEC-LGX-006.REQ.06.1（タイムアウト 30s・指数バックオフ 3 回・無効継続） | 🟢 |
| **P-06** Win/Linux ONNX 性能差 | Step 2 Linux 再測定要 | NFR-LGX-001 §3.1（Step 2 再測定明記） | 🟢 |
| **P-07** SQLite busy_timeout 未規定 | 無限リトライと Agent タイムアウトの衝突 | NFR-LGX-001.REL.07（上限 5000ms、無限リトライ禁止） | 🟢 |
| **E-03** SQLite WAL × Docker 配置条件 | ネットワーク FS 上の WAL 破損リスク | NFR-LGX-001.REL.08（ローカル FS 配置、ネットワーク共有禁止） / SPEC-LGX-008.REQ.12 | 🟢 |
| **E-04** Windows プロセス起動コスト | CreateProcess オーバーヘッドで 200ms 予算逼迫 | NFR-LGX-001.PERF.03（Windows で 300ms 緩和 / NAPI-RS 展望） | 🟢 |
| **E-05** v0.1.0→v3 ID 互換性断絶 | レガシー ID とハッシュ ID の非互換 | SPEC-LGX-008.REQ.11（ID マッピング自動生成 + 書き換え + --dry-run） | 🟢 |

### LOW（対応望ましい、すべて対応済）

| Finding | 概要 | legixy 解決先 | 状態 |
|---|---|---|---|
| **P-03** TOML 順序保持 | IndexMap 決定性はパーサ依存 | SPEC-LGX-002.REQ.08（順序保持パーサ必須） | 🟢 |
| **P-10** PIPE_ROLE 改ざん耐性 | 環境変数による役割偽装 | NFR-LGX-001.SEC.08（単独開発者環境前提、役割偽装は脅威モデル外） | 🟢 |
| **E-06** Markdown 装飾文字正規化 | ID 生成入力の正規化不足 | SPEC-LGX-002.REQ.06（Markdown 装飾除去・全角空白・NFC） | 🟢 |
| **E-07** ONNX「±20%」根拠不足 | 数値仮説の一次情報欠如 | 前身 VAL §5 P-06 から「±20%」削除済（Step 2 再測定必須のみ残置） | 🟢 |

### INFO（対応不要 / 将来検討）

| Finding | 扱い |
|---|---|
| **P-01** SHA-256 16 文字衝突確率 | 実用規模で無視可能、SubnodeIdUniqueness（SPEC-LGX-004）で事後検出 |
| **P-08** 時系列整合性 INV 未定義 | 緊急性なし。将来 TIME-INV 導入候補として残置 |
| **P-09** テストコード不可侵原則 | フック（pipeline gate）で実装済 |

## 3. G0 ゲート判定（継承）

| 項目 | 状態 |
|------|------|
| 予備レビュー完了 | 🟢 完了（前身、2026-04-17） |
| 外部 AI 照合完了 | 🟢 完了（ChatGPT + Gemini、2026-04-17） |
| HIGH/MEDIUM/LOW Finding への対応完了 | 🟢 全 14 件 対応済（§2、SPEC-LGX-* / NFR-LGX-001 に反映） |
| SPEC 内容の新旧一致 | 🟢 SUPP-LGX-000 で diff 照合済（前身照合の妥当性が legixy にそのまま成立） |
| **G0 ゲート** | 🟢 **通過（継承）** |

## 4. legixy 固有の補完（2026-06-13）

- **CTX-INV-5 の正準定義追加（D-09）**: E-01 の対処として新設された CTX-INV-5（未解決エッジの許容性）は、SPEC-LGX-002.REQ.11 が実装し SPEC-LGX-004 が検出していたが、正準定義元の LEGIXY-SPEC-001 §10 に欠落していた（REQ.11 の「根拠: §10 CTX-INV-5」がダングリング参照）。本セッションで §10.1 に正準定義を追加（LEGIXY-SPEC-001 v1.1.0）、E-01 のトレーサビリティを完成させた。
- 前身 VAL-LX-001 §10.5 の未解決検討事項（CTX-INV-5 の上位仕様昇格、VAL-LX-002 の要否、CTX ブロック DD/TS 新設）は、legixy では DevProc_V4.1 のフェーズ進行（RBA 以降）で扱う。

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-06-13 | 1.0.0 | 初版。前身 VAL-LX-001 v1.1.0（lexigy/v3 era、G0 通過済）の外部照合を legixy の横断的妥当性確認として採用。全 Finding（P-01〜P-10 / E-01〜E-07）を LGX 名義 SPEC/NFR の解決先へマッピング。CTX-INV-5 正準定義追加（D-09）を §4 に記録。前身は references/ に source-of-record として保全 |
