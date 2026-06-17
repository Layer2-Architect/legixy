# SPEC 修正案 — TP[SPEC] weak/minor GAP 解消（2026-06-10）

> **ステータス: 承認済・適用済（APPROVED & APPLIED 2026-06-10）。**
> 前回提案（GENUINE 28 件、適用済 2026-06-10）の残余である **weak/minor GAP 33 件**を解消する SPEC 改訂案。人間裁定（2026-06-10）により全 33 件は **fix（drop せず SPEC 修正で解消）** と決定され、全件適用承認（AskUserQuestion 経由）に基づき SPEC 本体へ適用済み。
> 個別裁定: **GAP-085 = stderr Info 追加【v3 差分】 / GAP-135 = 永続保持・パージなし / GAP-169 = タイムアウト導入【v3 差分・ADR-LGX-010】**。
> 適用結果: GAP 33 件 closed (2026-06-10)、**open GAP 0**、**TP 全 10 本 green**、ADR-LGX-009/010/011 起票・graph.toml 登録済み。**SPEC レベル TDD ループ完遂 → UC フェーズ解禁。**

## 0. 方針と横断事項

- 適用原則は前回と同一: 「凍結契約と正当入力の挙動は不変、黙殺されていた情報のみ可視化」「明示指定の対比は失敗を隠さない／集計系は skip + 集約 Warning」「決定論は順序のみ保証（ADR-LGX-003）」「単独開発者前提 SEC.08 のリスク受容（ADR-LGX-006 と同型）」。
- **v3 実測（2026-06-10 確認済）**: embed のモデル解決失敗は anyhow 伝播 exit 1・`[semantic] enabled` は embed 未参照（embed.rs:48-54）／ Ok finding は te-check に producer 不在＝発行されない（reporter.rs:62 は skip のみ）／ max_depth 打ち切り通知なし（multi_traverser.rs）／ MCP エラー応答に `_meta` 非付与（compile-context.ts:88-99 成功経路のみ）。
- **新規 ADR**: ADR-LGX-009（Contextual Retrieval 非決定性との両立、GAP-113）/ ADR-LGX-010（MCP 子プロセスタイムアウト導入、GAP-169 人間裁定）/ ADR-LGX-011（migrate 並行排他の SEC.08 リスク受容、GAP-150/151）。
- **NFR 申し送り**: GAP-142（migrate を PERF 予算対象外と明記）は NFR-LGX-001 側の追記も必要 → 既存の §13 行 237 drift と合わせて NFR 改訂イベントとして別途提起。

### バージョン bump 一覧
| SPEC | 現 | 適用後 | GAP |
|---|---|---|---|
| SPEC-001 | 0.7.0 | 0.7.1 | 002, 005 |
| SPEC-002 | 0.4.1 | 0.4.2 | 024 |
| SPEC-003 | 0.7.0 | 0.7.1 | 045, 047 |
| SPEC-004 | 0.7.0 | 0.8.0 | 061, 065, 072 |
| SPEC-005 | 0.3.0 | 0.4.0 | 081, 085 |
| SPEC-006 | 0.6.0 | 0.7.0 | 101, 102, 103, 105, 106, 110, 113, 118 |
| SPEC-007 | 0.4.6 | 0.5.0 | 121, 124, 135 |
| SPEC-008 | 0.6.0 | 0.7.0 | 141, 142, 144, 148, 150, 151, 158, 160 |
| SPEC-009 | 0.5.2 | 0.6.0 | 162, 168, 169, 170 |

---

## 1. SPEC-LGX-001 → 0.7.1

### GAP-002 → §4.3 追記
- §4.3 に査読確認記録を追記: 「2026-06-10 査読により §4.1 と下位 SPEC（002〜010）§4 サブセット表の整合を確認済み」。正準ソース宣言・同一コミット更新規律は §7.3（GAP-004 で新設済）が既に規定 → 参照を明示。**closes: GAP-002**（実体は §7.3 吸収済、残余の確認記録のみ）。

### GAP-005 → §4.1 1 セル訂正（案A）
- MCP-INV-1 行の 001 列を「実装」→「**関連**」に訂正（umbrella REQ.08 は方針宣言であり具体的機構は SPEC-LGX-009）。§7.3 規律に従い同一コミットで完結（下位 SPEC 側に 001 列は存在せず影響なし）。**closes: GAP-005**。

## 2. SPEC-LGX-002 → 0.4.2

### GAP-024 → REQ.13 バックアップ命名・保持
- 同一秒衝突は**連番付与で既存を上書きしない**（SPEC-LGX-008 REQ.02a と同一規約）。保持は**累積・機械削除なし**（手動掃除前提を明示）。配置は graph.toml と同一ディレクトリ。**VCS 追跡対象外を推奨**（.gitignore ガイダンス、規範ではなく推奨）。**closes: GAP-024**。

## 3. SPEC-LGX-003 → 0.7.1

### GAP-045 → REQ.16 パース規則
- trim 後に空となるトークン（`,,`・空白のみ）は**無視**（存在しない ID と同経路・エラーにしない）。重複 ID は **dedup（set セマンティクス）**し、返却順は REQ.11 整列に従う（指定順・指定回数に依存しない → CACHE-INV-1 保全）。全トークン無効時は空 upstream で正常終了。**closes: GAP-045**。

### GAP-047 → REQ.15 見出し皆無時の出力
- h1〜h3 が 0 個の artifact は **(a) 枠（ID 見出し等）を維持し body 空**を正準とする（REQ.10「セクション構成は件数非依存で固定」と整合。artifact 省略は採らない — 上流の存在自体が情報）。空 body の正確なフォーマット（改行等）は CACHE-INV-1 を満たす固定形として DD で確定。subnode 粒度で anchor も無い縮退は同規約。**closes: GAP-047**。

## 4. SPEC-LGX-004 → 0.8.0

### GAP-061 → REQ.01 補記（空グラフ）
- ノード 0/エッジ 0 のグラフ: **finding 0 件・exit 0** + **stderr に Info 1 件**（graph 未構築の誘導、可視化原則【v3 差分】）。graph.toml 物理不在は未初期化（init 誘導、別経路）、存在するが空は正常（exit 2 ではない）。**closes: GAP-061**。

### GAP-065 → REQ.03 補記（Ok の使用条件）
- **Ok はカテゴリ finding として発行しない**（v3 実測: te-check に producer 不在）。全 pass = findings 0 件 + 全 counts 0 + exit 0 が正準表現。Ok は CheckReport 集計（ok_count）と将来拡張のための**予約 severity**。JSON 出力時も Ok finding は現れない。**closes: GAP-065**。

### GAP-072 → REQ.06 拡張（全層 check の冪等性）
- REQ.06 の冪等性・結果順序保証を**全層 check にも拡張**: 順序は安定ソートキー（severity rank 降順 → category → related_ids、詳細 DD）で決定論的。**スコア値はビット再現対象外**（ADR-LGX-003 の順序決定論と同一適用範囲 — 同一環境では実用上再現、環境差は drift_threshold 吸収）。golden 比較は順序・件数・severity で行い生スコアの厳密一致を要求しない。**closes: GAP-072**。

## 5. SPEC-LGX-005 → 0.4.0

### GAP-081 → 方向規則の一般則明記
- 全エッジ種別で「**順方向 = `from`→`to`、逆方向 = `to`→`from`**」と統一明記。REQ.08（親→サブノード）はこの一般則の具体化として位置づけ。custom の双方向辿りは採らない。CTX-INV-3（意味的制約、所有 SPEC-LGX-003.REQ.05）と走査到達性は別概念で矛盾なしを確認注記。**closes: GAP-081**。

### GAP-085 → 打ち切り可観測性（人間裁定: Info 追加）
- `--max-depth` 打ち切り発生時（深度超過で除外されたノードが 1 件以上）、**stderr に Info 1 件**（打ち切り発生 + 除外件数）【v3 差分: v3 は無言打ち切り】。決定論的な到達集合・depth_map は不変。`--json` の truncated フラグは REQ.09 の DD 凍結対象へ申し送り。**closes: GAP-085**。

## 6. SPEC-LGX-006 → 0.7.0

### GAP-101 → REQ.02/09 補記（空テキスト）
- 本文が空・空白のみ（正規化後 0 文字、空 content_range 含む）のノード/サブノードは **embedding 生成を skip** し、`--json` の `skipped` に計上 + 集約 Warning 1 件（GAP-104 の skip 経路と同型）。ゼロベクトルは格納しない。skip されたノードは embedding 行なし = **未生成状態**（GAP-110 の 3 状態と整合）として drift/類似度から除外。**closes: GAP-101**。

### GAP-102 → REQ.01 補記（トークン上限超過）
- モデル最大トークン長はモデル metadata から取得。超過時は**先頭 N トークンで切り捨て**て生成し、**集約 Warning 1 件**（切り捨て件数）で可視化【v3 差分: v3 は無言】。チャンク分割平均は採らない（サブノード化が実用上の分割手段である旨を注記）。境界の厳密挙動は DD。**closes: GAP-102**。

### GAP-103 → REQ.01 補記（異常 shape）
- モデル読込時に**出力 shape 検証**（mean pooling 可能な軸構造・正の hidden 次元）を行い、非適合は **embed 起動不能 Error（exit 1）**+ モデル誤配置を示唆する診断。読込時検証（本件）と SCORE-INV-2（バージョン照合）は別経路と明記。**closes: GAP-103**。

### GAP-105 → REQ.04 補記（cosine 値域）
- 値域 **[-1.0, 1.0]** を明示（L2 正規化済みベクトルでは内積 = cosine）。負値は正常な出力（意味的反対）。浮動小数点誤差による僅かな域外値は **[-1, 1] に clamp**（GAP-104 数値安定性と整合）。SPEC-LGX-010 REQ.05 の calibrate clamp の前提を計算側で確定。**closes: GAP-105**。

### GAP-106 → REQ.01/02 補記（モデル不在）
- embed のモデル解決失敗・読込失敗は **Error + exit 1**、試行パスを stderr 通知（SPEC-LGX-010 REQ.03 と同一解決順: `--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR` > 設定。v3 実測の正準化）。**`[semantic] enabled` は check の意味層のみを制御**し、明示コマンド embed の実行可否に影響しない（v3 実挙動）。**closes: GAP-106**。

### GAP-110 → REQ.05/11 補記（3 状態）
- ノード状態を **fresh（hash 一致）/ stale（hash 不一致）/ 未生成（行なし）** の 3 値で明示。`detect_drift` は未生成を **DriftFinding(kind=missing) として含める**（戻り型表現は DD）。check の Drift 報告（SPEC-LGX-004 REQ.02）は stale と未生成を**区別したメッセージ**で Warning 報告【v3 差分: v3 は行不在を無言 skip】。**closes: GAP-110**。

### GAP-113 → REQ.06/06.1 補記 + ADR-LGX-009
- **freshness 判定は content_hash のみ**（SCORE-INV-1 不変。context_hash は寄与しない）。**context はキャッシュ**し、content_hash 不変なら**再合成しない**（LLM 非決定性を fresh 経路から排除）。再生成時（stale/--force）の context 揺れは embedding 値の揺れとして許容 — ADR-LGX-003 の「値のビット再現は対象外」に包含。合成は**逐次実行を既定**（並行化・レート制限は DD）。既定無効（REQ.06）が CTX-INV-1 前提を守る根拠であることを明示。**closes: GAP-113**（ADR-LGX-009）。

### GAP-118 → REQ.09/12 補記（content_range 検証）
- producer（LGX-EXT-001 §4.5.1）を信頼しつつ embed 側で**防御的検証**: 逆転（range.0 > range.1）・ファイル長超過・UTF-8 文字境界違反は**当該サブノードのみ Error 計上 + 継続**（REQ.09 トレランス）。**panic 禁止**（SEC.03/04、安全な切り出しは DD）。空 range（range.0 == range.1）は GAP-101 の空テキスト経路（skip）。**closes: GAP-118**。

## 7. SPEC-LGX-007 → 0.5.0

### GAP-121 → REQ.01 補記（message 境界）
- **空文字列・空白のみの message は受理しない**: CLI は値の意味的不正として **exit 1**（位置引数として構文上は valid のため exit 2 ではない、REQ.04 分類）、MCP は zod `min(1)` 相当（trim 後判定）。**最大長は設けない**（SQLite TEXT）。改行・Unicode は**無加工で保存**（忠実記録。サニタイズしない — SEC.05 の機密検査はダンプ検査で担保）。**closes: GAP-121**。

### GAP-124 → REQ.05 補記（--reason 境界）
- **空文字列・空白のみの `--reason` は拒否**（監査証跡の品質: 空理由を「指定あり」とみなさない）。受理済み引数の値の意味的不正として **exit 1**。最大長なし・無加工保存。**closes: GAP-124**。

### GAP-135 → REQ.08/09 補記（人間裁定: 永続保持）
- proposal（終端含む）・observation（resolved 含む）は**監査証跡として永続保持**。自動パージなし・パージコマンドも提供しない（提供は将来 SPEC 改訂事項）。手動 SQL 操作は運用責任域で legixy は関知しない。行削除後は同一 dedup キーが「新規」扱いになる挙動を注記。長期肥大化は SEC.08 単独開発者規模では実用上問題とならない旨を根拠に記録。**closes: GAP-135**。

## 8. SPEC-LGX-008 → 0.7.0

### GAP-141 → REQ.03 補記（空入力）
- 成果物 0 件の v0.1.0 入力 → **空 graph.toml（ノード 0/エッジ 0）を生成し正常終了 exit 0** + stderr Info（移行対象 0 件）。「対象なし」（パース成功 + 0 件）と「破損」（GAP-144）の区別を明記。**closes: GAP-141**。

### GAP-142 → REQ.03 補記（性能予算）
- migrate は**初回一時操作のため PERF 予算対象外**と明示（NFR 側の対応追記は NFR 改訂イベントへ申し送り）。メモリは全ロード方式を許容（SEC.04 の悪意入力 OOM 防止は別途有効）。サイズ上限なし。**closes: GAP-142**。

### GAP-144 → 新 REQ.03a（破損検出）
- 破損検出: engine.db = open/クエリ失敗・必須テーブル欠落、matrix/.legixy.toml = TOML パース失敗・必須構造欠落。検出時 **Error exit 1・原本温存・部分移行しない**（REQ.02 整合）。生成した graph.toml は**確定前に**パース可能性 + ID 一意性（REQ.11 全単射検証と共通）を検証し、壊れた出力を生成しない。**closes: GAP-144**。

### GAP-148 → REQ.02 補記（再開戦略）
- **resume なし・全やり直し方式**: 中間状態の記録を持たず（STATE-INV-1）、各段階の冪等性（REQ.02）で再実行が安全に収束。DB コミット先行 + atomic 確定（GAP-152 適用済）により中間状態は「DB のみ新」の 1 形態に限定され、再実行で解消。**closes: GAP-148**。

### GAP-150/151 → REQ.02 補記 + ADR-LGX-011（並行アクセス・排他）
- migrate 中の並行アクセス: engine.db は **WAL の読取一貫性**に委ね、graph.toml は **atomic rename**（REQ.02）により読み手は常に完全な旧版か新版のみを観測。**専用ロックファイル等の明示排他は設けない**。二重 migrate / auto 重複起動は SQLite 書込ロック（busy_timeout 超過で片方 Error exit 1）が事実上の排他となる。単独開発者前提（SEC.08）下のリスク受容として ADR-LGX-011 に記録。**closes: GAP-150, GAP-151**。

### GAP-158 → REQ.03 補記（抽出規則）
- ID 抽出は v0.1.0 設定スキーマに従う: `[matrix]` の section 設定 + `[id]` の pattern。節欠落・表構造崩れで**抽出 0 件になった場合は空入力（GAP-141）として正常終了 + Info**（構造完全性は要求しない — v0.1.0 の許容範囲を狭めない）。**`[id.chain] order` の欠落/不正は破損（REQ.03a）として Error**（chain エッジを暗黙 0 本にしない — 黙殺禁止）。具体構文の細目は DD。**closes: GAP-158**。

### GAP-160 → REQ.08 補記（成功時サマリ）
- 通常 migrate 成功時、**stdout に変更サマリ**（生成/更新ファイル一覧・書換 ID 件数・バックアップ場所）を出力（STATE-INV-2 の「確認して Git commit」運用を支える）。`--format json` 時は構造化（スキーマ DD）。診断は stderr（OBS.02）。**closes: GAP-160**。

## 9. SPEC-LGX-009 → 0.6.0

### GAP-162 → REQ.13 補記（エラー時 _meta）
- **エラー応答（isError: true）には `_meta` を付与しない**（v3 実測: 成功経路のみ付与、compile-context.ts:88-99）。成功/失敗で応答構造が分岐することを明記（永続化ヒントは成功本文にのみ意味を持つ）。**closes: GAP-162**。

### GAP-168 → REQ.08 補記（起動失敗）
- バイナリ解決失敗・project-root 不正は**起動時 fail-fast せず**、各ツール呼出し時に **REQ.07 の `Rust CLI failed (exit -1):` 形式で isError 返却**（v3 実質挙動の正準化。thin forwarder は起動時にバイナリを検証しない）。**closes: GAP-168**。

### GAP-169 → 新 REQ.16（子プロセスタイムアウト、人間裁定: 導入【v3 差分・ADR 必須】）
- MCP→CLI 子プロセスに**タイムアウトを導入**: 既定 **30 秒**、環境変数 `LGX_MCP_TIMEOUT_SEC` で変更可（`0` = 無効化 = v3 互換動作）。超過時は SIGTERM → 猶予 5 秒 → SIGKILL、プロセスは必ず回収（ゾンビ防止）。応答は `isError: true`・本文 `Rust CLI failed (timeout after {N}s):`（REQ.07 と同系の新形式）。**部分出力は転送しない**（全か無か — MCP-INV-2 忠実転送の保全）。正当入力は 30 秒内に完了するため正当入力空間の挙動は不変（PERF.03 予算 < 300ms に対し 100 倍マージン）。ADR-LGX-010 起票。**closes: GAP-169**。

### GAP-170 → REQ.03 補記（ロギング・マスキング）
- CLI stderr は**マスキングせず忠実転送**を確定（MCP-INV-2 優先 — MCP 層での改変はむしろ INV 違反）。SEC.05 のクレデンシャルマスキングは **Rust CLI 側経路（Contextual Retrieval API キー）の責務**と明確化。MCP サーバ自身の診断ログは stderr 最小限（詳細 DD）とし、**環境変数の値（バイナリパス等）はログに出力しない**。**closes: GAP-170**。

---

## 10. 適用後の作業
1. 各 SPEC 改訂 + 変更履歴記入。
2. GAP 33 件 → `closed (2026-06-10)`、§5 解決経緯記入。
3. TP 全 10 本の該当観点 GREEN 化 → **全 TP green**（open GAP 0）。
4. ADR-LGX-009/010/011 起票 + graph.toml 登録。
5. `bash scripts/trace-check.sh` で TDD ゲート green を確認 → **UC フェーズ解禁**。
