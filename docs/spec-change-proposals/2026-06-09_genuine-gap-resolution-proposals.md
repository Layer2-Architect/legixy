# SPEC 修正案 — TP[SPEC] GENUINE GAP 解消（2026-06-09）

> **ステータス: 承認済・適用済（APPROVED & APPLIED 2026-06-10）。**
> 本書は SPEC-level TDD ループで検出され敵対的精査を生存した **GENUINE GAP 28 件**（119→敵対的精査→61 生存のうち GENUINE 28）を解消するための SPEC 改訂案である。
> **人間承認（2026-06-10、AskUserQuestion 経由の裁定 + 全 28 件適用承認）に基づき、全提案を SPEC 本体に適用済み。** 対応 GAP は `closed (2026-06-10)` に更新済み。
>
> **裁定結果（§0 の判断事項）:**
> - §0.1 GAP-157（BLOCKER）→ **案A 採用**（PATH 正準・凍結契約無変更・REQ.09 自動検出へ移管。ADR-LGX-001）
> - §0.2 GAP-120 → **加算的拡張を承認**（ADR-LGX-002 + LGX-COMPAT-001 v1.1.0 §4 #4/§7 追記済み）
> - §0.6 GAP-064 → (a) **新カテゴリ GraphDag**（【v3 差分】注記、ADR-LGX-008）、(b) **DocumentId 行欠落 = Error（厳格）**（v3 実挙動と一致）
> - §0.3 ADR → **全 8 件起票済み**（ADR-LGX-001〜008）
> - §0.6 v3 実測 TODO → **3 件とも確認済み**: GAP-023 = v3 は直接上書き（refresh_subnodes.rs:357、【v3 差分】注記）/ GAP-120 = --node/--force は v3 不在（純加算）/ GAP-154 = PRAGMA user_version 実在（initializer.rs:137、差替不要）
>
> minor/WEAK 33 件（「人間判断で drop 可」注記済み）は本書の対象外（保留 — 次の人間判断事項）。

---

## 0. 先に判断が必要な事項（ブロッカー / 凍結契約 / ADR）

### 0.1 GAP-LGX-157（BLOCKER・凍結契約矛盾）— 人間裁定必須

`migrate --from/--to` が **LGX-COMPAT-001 §4 #2 では `<PATH>`**（`--from <PATH>` 必須・`--to` 既定 `--project-root`）、**SPEC-008 REQ.06 では `--from v0.1.0 --to legixy`（バージョン文字列）** と、同一フラグに異なる意味を与えている。凍結境界契約との直接矛盾（ハードルール 7）であり AI は単独確定不可。

- **案A（推奨）**: 凍結 PATH 意味を正準とし、バージョン意図は REQ.09 のバージョン自動検出へ移管。**凍結契約は無変更**（ハードルール 7 維持）。整合判断の記録として ADR 起票を推奨。
- **案B**: フラグをバージョン文字列にする＝**凍結契約の改訂（COMPAT リビジョン + ADR 必須）**。「既に凍結済み」ステータスが崩れ、パス指定手段を失う副作用あり。

→ §8（SPEC-008）に両案の適用テキストを記載。**どちらを採用するか先に決定してください。**

### 0.2 凍結契約（LGX-COMPAT-001, ハードルール 7）に触れる提案
| GAP | 内容 | 必要手続き |
|---|---|---|
| GAP-157 | migrate `--from/--to` 意味 | 案A: ADR 推奨 / 案B: COMPAT 改訂 + ADR 必須 |
| GAP-120 | embed `--node`/`--force`/`--json` が凍結 §4 #4 に未記載 | **ADR 必須 + LGX-COMPAT-001 §4 #4/§7 追記**（加算的拡張＝後方互換だが要承認） |

### 0.3 ADR 起票候補
- **embedding 決定論モデル**（GAP-104 浮動小数点ε許容 + GAP-114 content_hash 正規化 + GAP-115 model_version 複合キー）を 1 ADR に統合推奨。
- GAP-120（embed フラグ凍結契約拡張）— ADR 必須。
- GAP-157（migrate 契約整合）— ADR 推奨。
- GAP-041/139（可用性 > 監査完全性）, GAP-126（再生成不能データ保護 vs キャッシュ）, GAP-140（宣言的強制に留める判断）, GAP-185/186（スコア特殊値・model_version 照合ポリシー）— いずれも ADR 推奨。

### 0.4 バージョン bump 一覧（適用時）
| SPEC | 現 | 適用後 | GAP |
|---|---|---|---|
| SPEC-001 | 0.5.0 | 0.7.0 | 001, 003, 004 |
| SPEC-002 | 0.4.0 | 0.4.1 | 023 |
| SPEC-003 | 0.5.0 | 0.7.0(0.6.0) | 041, 043 |
| SPEC-004 | 0.6.0 | 0.7.0 | 064 |
| SPEC-006 | 0.5.0 | 0.6.0 | 104, 108, 114, 115, 120 |
| SPEC-007 | 0.4.0 | 0.4.6 | 122, 126, 127, 129, 139, 140 |
| SPEC-008 | 0.5.0 | 0.6.0 | 143, 146, 152, 153, 154, 157, 159 |
| SPEC-009 | 0.5.1 | 0.5.2 | 171 |
| SPEC-010 | 0.2.0 | 0.2.1 | 185, 186 |

### 0.5 適用順序・バッチ
- SPEC-001: GAP-004(§7新設)→GAP-001(§7参照)を同一改訂。GAP-003 は独立可。3 件を 1 査読で。
- SPEC-007: 122→126→127→129→139→140 の順（テキスト依存・履歴連番 0.4.1〜0.4.6）。
- SPEC-006: 5 件を 0.6.0 単一改訂。SPEC-010: 185/186 を 0.2.1 単一改訂。

### 0.6 GAP スコープ外の側次的所見（別途・人間判断）
- **NFR-LGX-001 内部 drift**: §13 暫定表 行237「PERF.03 < 200 ms」が §3.2 本文（Step1 Win<300ms / Step2 Linux<200ms）と未同期。NFR 側の自己整合修正（spec-change イベント）として別途提起推奨。
- **GAP-064 人間判断点 2 件**: (a) グラフ全体 DAG を新カテゴリ `GraphDag` とするか `ChainIntegrity` 配下に統合するか（v3 出力フォーマット実挙動の確認要）。(b) DocumentId 行欠落を Warning（互換猶予）とするか Error（厳格）とするか。
- **v3 実測の確認 TODO**（適用前推奨）: GAP-023（v3 の graph.toml 書込方式＝直接上書きか）/ GAP-120（v3 に `--node`/`--force` が実在するか）/ GAP-154（v3 engine.db が `PRAGMA user_version` を持つか）。

---

## 1. SPEC-LGX-001（全体要求）— GAP-001 / 003 / 004 → 0.5.0 → 0.7.0

### GAP-LGX-001 → ヘッダ表 + REQ.02（予約注記）
- **対象**: ヘッダ表「対応 UC」行、REQ.02 末尾に予約注記。本文「001〜011」は据え置き（UC-012/013 が未実在のため先行改訂すると孤児宣言になる）。
- **ヘッダ表** OLD `| 対応 UC | UC-LGX-001〜011（全体） |` → NEW `| 対応 UC | UC-LGX-001〜011（全体。UC-LGX-012/013 は予約済・未生成 — §3 REQ.02 注記参照） |`
- **REQ.02 末尾追記**: SPEC-010 §1.3 が snapshot/drift を UC-012/013 に写像予定であり、網羅宣言「001〜011」据え置きは見落としでなく**既知の予約状態**。本文「001〜013」への再改訂は UC フェーズ着手時（SPP-LGX-001 次反復）に §7 変更ポリシーに従って実施。
- **closes**: GAP-001。**備考**: GAP-004(§7) と同一改訂で適用（予約注記が §7 を参照するため 004 を先に積む）。

### GAP-LGX-003 → §4.1 凡例 + 新§4.4「検証 owner 要否ポリシー」
- 「実装」owner はあるが「検証」owner を持たない不変条件を **C（construction-time guarantee, 検出不要）/ D（needs-detection, 検出は TS/TC 層）** に分類するポリシー表を §4.4 として新設（CTX-INV-1=D, CTX-INV-3=C, FB-INV-1〜5=D, SCORE-INV-1=C, MCP-INV-1=C, MCP-INV-2/3/4=D, STATE-INV-1/2=C, SUBNODE-INV-5=C, CACHE-INV-1〜4=D）。§4.1 凡例に検証要否分類ラベルを追記。
- 含意: §4.1 で「検証」owner が空でも、C（構成的保証）か D（TS/TC 委譲）のいずれかであることを明示し、検証網羅性主張と RPC 母数の根拠を明文化。
- **closes**: GAP-003（旧 GAP-002 の同期規律 subset は GAP-004 §7.3 が吸収済み）。

### GAP-LGX-004 → 新§7「変更ポリシー」
- §6 の後に §7 を新設。§7.1 変更主体と承認（ハードルール 1、人間承認）、§7.2 下位 SPEC の partition 拡張に伴う同期トリガ（FCR ACCEPTED）と期限（UC フェーズ着手時の SPP-LGX-001 次反復）、§7.3 §4.1 マトリクスを正準とする正準ソース宣言 + **下位 SPEC §4 と同一コミット更新規律**（GAP-002 subset 吸収）。
- **closes**: GAP-004（+ GAP-002 subset を §7.3 で吸収）。新§追加のみで既存 REQ 不変＝凍結境界に抵触なし。

---

## 2. SPEC-LGX-002（グラフ基盤）— GAP-023 → 0.4.0 → 0.4.1

### GAP-LGX-023 → REQ.13（refresh-subnodes 書き換え atomicity）
- REQ.13 `**内容:**` の「バックアップ」箇条を拡張: ①バックアップ作成失敗時は本体書き換えに進まない ②**同一ディレクトリの一時ファイルへ全量書き出し→fsync→アトミック rename** で graph.toml を置換（直接上書き禁止）③書き換え失敗時は exit 1・元 graph.toml 無傷 ④順序不変条件（バックアップ→書き出し+fsync→rename）。
- `**検証方法:**` に中断シナリオ・同一ディレクトリ一時ファイル・バックアップ失敗時非書換のテストを追加。`**根拠:**` に GAP-023（REL.01 は engine.db/WAL 限定で平文 graph.toml を保護しない）を追補。
- **凍結契約不変**: CLI 引数（§4 #9）・バックアップ命名は変更せず、`--apply` の永続化セマンティクスのみ追加。
- **closes**: GAP-023。**v3 確認 TODO**: v3 が直接上書き方式だった場合は【v3 差分】注記を要する。ADR 任意。

---

## 3. SPEC-LGX-003（コンテキスト解決）— GAP-041 / 043 → 0.5.0 → 0.7.0

### GAP-LGX-041 → 新 REQ.19「監査ログ書込失敗時の終端状態」
- **正準挙動: 本処理優先（記録欠落を許容し成功扱い）。** 本処理結果生成と context_log 書込を**別トランザクション**化。DB 存在下で本処理成功・context_log 書込失敗時は **stdout 返却 + exit 0**、stderr に Warn 診断。busy_timeout(REL.07, 5000ms)超過でリトライ打ち切り。
- **MCP-INV-4 との関係**: 「全呼出記録」を**ベストエフォート**に格下げ、欠落検出は stderr Warn で確保。可用性 > 監査完全性の判断は ADR 記録。§4 MCP-INV-4 行に REQ.19 追記。
- **closes**: GAP-041。frozen exit-code 規約（記録失敗は本処理の意味的不正でないため exit 1 にしない）と整合。

### GAP-LGX-043 → 新 REQ.20「起点不在・上流部分欠損時の扱い」
- (1) 起点ノード不在パス → **無視して残りで解決・exit 0**。命名規約からの chain 位置推定はしない（推測排除）。全未登録なら空 upstream で exit 0。unresolved を Target Node Metadata + stderr Info に記録。
- (2) 上流連鎖途中の欠損 → **飛ばして残りを返す部分成功・exit 0**。欠損を**決定論的に記録**（CTX-INV-1/REQ.14 のバイト決定論を保全）。
- **closes**: GAP-043。041/043 は成功境界の両側（後者=入力解決の不完全性、前者=成功後の副作用失敗）で別概念のため別 REQ が適切。

---

## 4. SPEC-LGX-004（検証）— GAP-064 → 0.6.0 → 0.7.0

### GAP-LGX-064 → REQ.01 修正 + 新 REQ.15「形式検証カテゴリの severity 割当」（旧 073/074 吸収）
- **REQ.01** 各基幹カテゴリに severity を明示: FileExistence【Error】, DocumentId（不一致【Error】/行欠落【Warning】）, ChainIntegrity【Error】, OrphanFile【Error】, **GraphDag**（グラフ全体サイクル CTX-INV-4）【Error】, Freshness【Warning】, Subnode系【Error】。
- **新 REQ.15**: severity 割当表（割当完全性の保証 + 固定/可変方針）。Error 列のみが G1 ゲート（Error=0）と exit 1 に影響。既存 pin 済（REQ.07/10/11/12/13/14）とは重複しても矛盾なし（再掲明示）。§4 CTX-INV-4 行を REQ.15 連動に更新。
- **closes**: GAP-064（+旧 073 DocumentId severity / 旧 074 node-level DAG severity）。LGX-COMPAT-001 v1.0.1（Error>0→exit 1）と整合。CLI/MCP 引数不変。
- **人間判断点**: (a) `GraphDag` 新カテゴリ vs `ChainIntegrity` 統合（v3 出力フォーマット確認要）, (b) DocumentId 行欠落=Warning(猶予) vs Error(厳格)。

---

## 5. SPEC-LGX-006（embedding とドリフト検出）— GAP-104/108/114/115/120 → 0.5.0 → 0.6.0

### GAP-LGX-104 → REQ.04 + §4（CTX-INV-1）
- **ゼロベクトル/ノルム0 ペアの cosine**: NaN/Inf を返さず **skip + 集約 Warning 1 件**（次元不一致と同経路）。standalone `drift`（SPEC-010.REQ.03）のゼロベクトルは Error 維持。
- **浮動小数点推論の値再現性**: CTX-INV-1/SCORE-INV は **走査・出力「順序」の決定性のみ保証**、ONNX 推論値のビット再現性は対象外（微小差は drift_threshold が吸収）。§4 CTX-INV-1 行に適用範囲を明記。
- **closes**: GAP-104。ADR 推奨（101 と統合可）。

### GAP-LGX-108 → REQ.08 ↔ REQ.09（内部矛盾解消）
- **判定: REQ.09（部分失敗トレランス）優先。トランザクション粒度を「ノード/サブノード単位 1 Tx」に確定**。`embed --all` 全体 1 Tx を否定（1件失敗で全ロールバックは REL.02 に反する）。1 ノード失敗時は当該 Tx のみ rollback・Error 計上、後続継続。REL.02 と REL.06 を両立。
- **closes**: GAP-108。終了コード詳細は SPEC-010 へ委譲（責務境界）。ADR 任意。

### GAP-LGX-114 → REQ.03 + REQ.12 + §4（SCORE-INV-1）
- **content_hash 計算前の正規化手順**: ①BOM 除去(COMPAT.07) ②CRLF/CR→LF(COMPAT.08) ③NFC ④末尾正規化。正規化後 UTF-8 で SHA-256。クロスプラットフォームで偽 stale/偽 fresh を防止。REQ.12 サブノード content_hash にも同一適用。
- **closes**: GAP-114。SCORE-INV-1 を環境非依存に固定。ADR 推奨（114/115 統合）。末尾正規化の厳密挙動は DD 委譲。

### GAP-LGX-115 → REQ.10 + §4（SCORE-INV-2）
- **model_version 生成方式**: (a)モデル名 + (b)ONNX ファイル内容ハッシュ + (c)前処理プロファイル(e5系 prefix 等) + (d)出力次元 の複合キー。ファイル差替を (b) で検出（偽 fresh 防止）。「変化した」判定は文字列完全一致。
- **closes**: GAP-115。SCORE-INV-2 判定を決定論化。GAP-116（旧モデル移行）と連携。hex 桁数等は DD 委譲。

### GAP-LGX-120 → REQ.02 + REQ.10（凍結契約に触れる）
- **推奨案A: フラグを凍結契約に SPEC 主導の加算的拡張として追加**（既存 `[--all]` 呼出不変＝後方互換）。REQ.02 に `--node <ID>`（複数可・--all 排他）・`--force`・`embed --json` スキーマ（`{generated, skipped, failed, errors[]}`）を確定。
- **closes**: GAP-120。**ADR 必須 + LGX-COMPAT-001 §4 #4/§7 への追記**。MCP 非公開維持（MCP-INV-1）。**v3 確認 TODO**: `--node`/`--force` が v3 実在か。

---

## 6. SPEC-LGX-007（フィードバックループ）— GAP-122/126/127/129/139/140 → 0.4.0 → 0.4.6

### GAP-LGX-122 → REQ.01 + REQ.11（related_id 正準化）
- REQ.01: related_id は**形式・実在検証せず受理して保存**（未登録対象への気づき記録用途）。REQ.11 正準化に **distinct 化ステップ**を追加（重複除去→昇順ソート→JSON 化）、semantic_key 生成と共有。凍結比較セマンティクスは不変。
- **closes**: GAP-122。

### GAP-LGX-126 → REQ.09 + §4（FB-INV-4）
- engine.db **破損**（不在と区別）時: **自動再生成せず exit 1 で明示失敗**。observation/proposal は再生成不能なユーザ生成データとして保護（STATE-INV-1 の再生成可能キャッシュ扱いの例外）。
- **closes**: GAP-126。ADR 推奨。

### GAP-LGX-127 → REQ.09 + REQ.05（proposal 状態モデル、旧 128/130 統合）
- **遷移グラフ**: (無)→pending→{approved|rejected} のみ。approved/rejected は**終端・不可逆**。approve/reject は `status='pending'` のみに作用。終端への再操作は **exit 1 で拒否**（証跡保全）。並行 approve/reject は `UPDATE...WHERE status='pending'` の **CAS**（行数 1=成立）。§4 FB-INV-3 行更新。
- **closes**: GAP-127（旧 128/130 吸収）。typestate vs 実行時は DD 委譲。

### GAP-LGX-129 → REQ.08 + REQ.11（observation 状態モデル）
- status: pending/analyzing/resolved。遷移: observe→pending, analyze→analyzing, approve→resolved, reject/未生成→pending。resolved は終端不可逆。dedup 適用範囲(pending/analyzing)を状態モデルへ接続。
- **closes**: GAP-129。**備考**: 127+129 を将来 §3.1 状態モデルに統合する再編を ADR 付きで推奨（本提案は GAP 追跡性優先で各 REQ 追記に留める）。

### GAP-LGX-139 → REQ.06 + §4（MCP-INV-4）
- context_log INSERT 失敗時（DB あるが書けない中間ケース）: **可用性優先** — 本体は成功・記録は best-effort・警告は残す。MCP-INV-4 完全性は「DB 利用可能時に限る」と明示。本体と記録を**分離 tx**化が下流で正当化される。
- **closes**: GAP-139。ADR 推奨。126（Admin 書込整合性優先で exit 1）と方針分岐（139=Agent 読取可用性優先）。

### GAP-LGX-140 → REQ.05 + §4（MCP-INV-1）
- 「人間のみ」は二層: **MCP 非露出（技術的境界）+ CLAUDE.md ルール5（運用規律）**。Agent の Bash 直接 spawn は**単独開発者前提（NFR SEC.08）下のリスク受容**として明示。改ざん耐性ガードは要件としない（SEC.08 と整合）。
- **closes**: GAP-140。ADR 推奨（宣言的規律に留める判断）。

---

## 7. SPEC-LGX-008（マイグレーション）— GAP-143/146/152/153/154/157/159 → 0.5.0 → 0.6.0

### GAP-LGX-157 → REQ.06（BLOCKER・§0.1 参照）
**案A（推奨・凍結契約無変更）**: REQ.06 を `--from <PATH>`（必須）/`[--to <PATH>]`（既定 --project-root）/`[--dry-run]`/`[--format markdown|json]` に確定。バージョンは引数で取らず **REQ.09 自動検出**で決定。既 legixy なら no-op。根拠に LGX-COMPAT-001 §4 #2（凍結）。
**案B（凍結契約改訂・COMPAT リビジョン+ADR 必須）**: `--from <VERSION>`/`--to <VERSION>`。パス指定手段喪失の副作用、別フラグ `--source <PATH>` 検討要。
- **closes**: GAP-157。**人間裁定 + ADR（案A 推奨/案B 必須）。**

### GAP-LGX-154 → REQ.09（バージョン検出、旧 147/149 統合）
- engine.db: `PRAGMA user_version` を一次根拠（0 なら legixy 追加カラム有無で二次判定）。`.legixy.toml`/`.trace-engine.toml`: `[graph]` セクション有無。マーカ欠落は v0.1.0 とみなす。矛盾特徴は Error。権威ソースを `PRAGMA user_version` に固定。
- **closes**: GAP-154（旧 147/149 の判定基盤）。**v3 確認 TODO**: user_version 実在か（無ければ version テーブル方式に差替可）。

### GAP-LGX-146 → REQ.11 + §4（CTX-INV-2）
- マッピング不可 ID: **既定 abort**（graph.toml/id-map 不変、非破壊性優先）。継続(opt-in)時は旧 ID 残置せず**当該エッジを除外**（dangling 防止）。継続フラグ名は GAP-157 確定後に DD で定義。
- **closes**: GAP-146。

### GAP-LGX-159 → REQ.11 + §4（SUBNODE-INV-3）
- id-map は**旧→新 ID 全単射を保証**。旧 ID 側重複（曖昧性）→ Error、新 ID 側衝突（多対一）→ Error、graph 全体での新 ID 一意性(SUBNODE-INV-3)違反→ Error。--dry-run でも検証。
- **closes**: GAP-159。

### GAP-LGX-152 → REQ.02（非 DB ファイル atomic 書込）
- graph.toml / id-map を **temp(.tmp.{epoch})→fsync→rename(2)** で atomic 保護。**engine.db コミットを先行**させその後 graph.toml/id-map を atomic 確定（不整合中間状態を残さない）。
- **closes**: GAP-152。SPEC-002 REQ.13/GAP-023 と整合。

### GAP-LGX-153 → 新 REQ.02a（`.bak` 退避名衝突解決）
- 退避名を `<元名>.bak.{unix epoch 秒}`（固定 `.bak` 上書き禁止）。同一秒衝突は連番付与で既存を上書きしない。バックアップは累積、機械的削除しない。
- **closes**: GAP-153。refresh-subnodes `.refresh-bak.{epoch}` 慣行と統一。init --force 退避(§4 #1)も本命名に統一。

### GAP-LGX-143 → REQ.07（init 既存ファイル判定/--force 破壊性）
- 「既存」判定対象を **legixy 管理生成物**（`.legixy.toml`/`.trace-engine.toml`, graph.toml, engine.db）に限定。ICONIX 8 dir やユーザドキュメント存在は判定対象外。`--force` は legixy 生成物のみ上書き（事前に REQ.02a 退避）、`.gitkeep` は不足分のみ補完、既存ユーザファイル不変。
- **closes**: GAP-143。`init [--force]` シグネチャ不変（§4 #1 維持）。

---

## 8. SPEC-LGX-009（MCP サーバ）— GAP-171 → 0.5.1 → 0.5.2

### GAP-LGX-171 → REQ.06（doc-drift 同期）
- REQ.06 の陳腐化引用「PERF.03 < 200 ms 暫定」を現行 NFR 値に同期: **Step1 Windows < 300 ms【E-04】/ Step2 Ubuntu Docker < 200 ms**。Step1/2 区別・Windows 緩和・NFR §13 バジェット参照を明記。
- **closes**: GAP-171。数値同期のみ（意図・不変条件不変）。**別件（NFR 側）**: §13 暫定表 行237 の内部 drift を NFR 改訂として別途提起（§0.6）。

---

## 9. SPEC-LGX-010（embedding 運用と監査）— GAP-185/186 → 0.2.0 → 0.2.1

### GAP-LGX-185 → 新 REQ.09（NaN/±Inf の扱い）+ REQ.03/04/05 参照差込
- 非有限スコア（NaN/±Inf）は**対比に算入しない**: calibrate/report は **skip + 集約 Warning**（生成側非ゼロ L2 正規化保証は SPEC-006 所在、本 REQ は consumer フォールバック）。**drift は exit 1**（明示対比は壊れた状態を隠さない）。**`--json` は非有限値を一切出さない**（統計不能時は `null`）。REQ.06 決定性の前提。
- **closes**: GAP-185。ADR 推奨。

### GAP-LGX-186 → REQ.03 + §4（SCORE-INV-2 過大宣言訂正）
- drift で **ベースライン保存 model_version と現行 model_version が異なる場合（次元一致）→ exit 1**（SCORE-INV-2 違反、同一次元・別バージョン遷移を捕捉）。**§4 の「検証=次元不一致 Error のみ」過大宣言を「model_version 照合が一次検出、次元不一致は補完」へ訂正**。§1.3 UC-013 代替フローに model_version 不一致を追記。
- **closes**: GAP-186。**ownership: model_version 照合は drift コマンドの出力契約＝ SPEC-010 REQ.03 所在**（SPEC-006 は生成・bulk API に責務限定）。ADR 推奨。

---

## 10. 適用後の作業
1. 各 SPEC を承認版テキストで改訂（上記 OLD→NEW / 新 REQ / 新§）。
2. 対応 GAP の `**ステータス**: open` → `closed (2026-06-09)`、§5 解決経緯を記入。
3. 関連 TP の該当観点を GREEN に、全観点 GREEN なら TP を `**ステータス**: green` に。
4. 承認された ADR を `docs/adr/` に起票（typecode 登録 + graph.toml ノード追加）。
5. GAP-120/157 で凍結契約を改訂する場合は LGX-COMPAT-001 を改訂。
6. `bash scripts/trace-check.sh` で全観点 GREEN・open GAP 0 を確認 → UC フェーズへ。
