Document ID: TP-LGX-005

# TP-LGX-005: グラフ走査の観点リスト

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。

**親**: SPEC-LGX-005
**ステータス**: green
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は SPEC-LGX-005「グラフ走査」全体（REQ.01〜REQ.10）をカバーする。

- 対象: SPEC-LGX-005 §3 要求事項 REQ.01〜REQ.10、§4 不変条件との関係
- 関連 SPEC §: SPEC-LGX-002 §3（REQ.04 エッジ種別 / REQ.07 DAG / REQ.08 決定論順序 / REQ.11 未解決エッジ）、SPEC-LGX-010 §3（`--json` 出力一貫性）、LEGIXY-SPEC-001 §10（CTX-INV-1, CTX-INV-5）、LGX-COMPAT-001 §3/§4（`impact`/`investigate` の引数・終了コード）、NFR-LGX-001 REL.05（BFS 走査決定性）

走査の「対象グラフ構造」「エッジ種別の定義」「DAG 制約」「IndexMap 格納順」「未解決エッジの除外」自体は SPEC-LGX-002 が所有する。本 TP はそれらを前提とした **走査アルゴリズムの挙動** に観点をぶつける。重複所有の観点は SPEC-LGX-002 へ委譲する。

## 2. 観点リスト

### 2.1 境界値
- [ ] 観点 1: `max_depth = 0` のとき返るノードは起点のみか（深度は起点 0、REQ.04）
- [ ] 観点 2: `max_depth` 未指定（無制限）と巨大値指定で結果が一致するか（REQ.04 既定 = 無制限）
- [ ] 観点 3: `--max-depth` に負値・非数値・usize 上限超えを与えたときの扱い（パーサ層 exit 2 か）
- [ ] 観点 4: 空グラフ（ノード 0）／単一ノード（エッジ 0）／起点が孤立ノードのときの結果
- [ ] 観点 5: `max_depth` で打ち切られたノードがある場合、打ち切りの発生は観測可能か

### 2.2 エラーハンドリング
- [ ] 観点 6: 存在しない起点 ID → 空結果（エラーではない、REQ.05）。終了コードは 0 か
- [ ] 観点 7: 起点は存在するが出ていく（入ってくる）エッジが全て未解決で除外されている場合の結果
- [ ] 観点 8: 走査中の panic / unwrap が本番経路に残らない保証（REQ.06 の safety と整合）

### 2.3 状態遷移（走査アルゴリズムの進行）
- [ ] 観点 9: visited セットの初期状態（起点は最初から visited か）と再訪抑止の意味（REQ.06）
- [ ] 観点 10: BFS のレベル進行と depth_map の対応（REQ.03/REQ.04）
- [ ] 観点 11: 同一ノードへ複数経路・複数深度で到達したとき depth_map に記録される深度の規則（最短深度優先か）

### 2.4 並行性
- [ ] 観点 12: 走査は読み取り専用のメモリ内操作か。走査中の graph.toml 外部更新の扱い（プロセス起動時ロード前提か）

### 2.5 バージョニング・互換性
- [ ] 観点 13: `max_depth` 省略時 = 無制限が v3 既定（互換対象 (d)）として維持されているか（REQ.04）
- [ ] 観点 14: `--json` の機能化が【v3 差分】として引数体系を壊さないか（REQ.09、LGX-COMPAT-001 §4）
- [ ] 観点 15: CLI サブコマンド名 `impact`（順方向）/`investigate`（逆方向）・位置引数・`--max-depth` が互換契約と一致するか（REQ.07）

### 2.6 永続化
- [ ] 観点 16: 走査自体は engine.db / graph.toml への書き込みを伴わない（読み取り専用）か

### 2.7 入力検証
- [ ] 観点 17: 起点 ID の形式検証はどこで行うか。任意文字列の起点に対する挙動（REQ.05 へ収束するか）

### 2.8 ライフサイクル
- [ ] 観点 18: 起点が単一ノード（隣接ゼロ）のとき結果は「起点 1 件」か（最後の 1 件相当）

### 2.9 ロギング・観測性
- [ ] 観点 19: 走査結果の出力に含めるべき情報の確定（ID 走査順 / type / path / depth / 使用エッジ、REQ.09）
- [ ] 観点 20: `investigate` の Text 出力が言及する suspicious nodes / drift 値の判定基準・閾値の定義（REQ.09）
- [ ] 観点 21: 起点に到達できない／未解決で除外したエッジがある場合、その事実が出力で可視化されるか（CTX-INV-5 / SPEC-LGX-002 REQ.11 との連携）

### 2.10 FFI / 境界 API（CLI 引数・終了コード）
- [ ] 観点 22: グローバル `--project-root` / `--json` / `--models-dir` を `impact`/`investigate` が受理するか（LGX-COMPAT-001 §3）
- [ ] 観点 23: 起点未存在・空結果時の終了コードは 0 か（OBS.05 / USE.04 と整合、REQ.05）
- [ ] 観点 24: MCP ツールとして走査が公開されないこと（MCP-INV-1、REQ.10）

### 2.11 領域固有観点（グラフ走査・決定論）
- [ ] 観点 25: 隣接ノードの処理順は IndexMap 挿入順に従うか（REQ.03）。同一起点・同一グラフで visited 順・depth_map が常に同一か（CTX-INV-1 / REL.05）
- [ ] 観点 26: 1 ノードから chain / custom / parent_child の複数種別エッジが出ている場合、隣接処理順の決定規則は何か。とくに**システム生成の parent_child エッジ**は graph.toml 挿入順を持たないため、種別間の整列規則が必要
- [ ] 観点 27: `custom` エッジに対する「順方向 / 逆方向」のセマンティクス定義（chain は連鎖方向で自明だが custom は from→to が方向か）（REQ.01/02）
- [ ] 観点 28: parent_child エッジの方向（親→サブノードが順方向、REQ.08）が全種別共通の from→to 規則と矛盾しないか
- [ ] 観点 29: DAG が破れた入力（サイクルあり）でも有限停止するか。visited による停止保証（REQ.06）
- [ ] 観点 30: 自己参照エッジ（node→自身）が存在するとき、走査は起点を再出力せず停止するか（REQ.06 の防御対象に self-loop が含まれるか）
- [ ] 観点 31: 走査対象エッジ種別をフィルタする手段は提供されないか（REQ.01/02 は 3 種別を常に対象と読める）— フィルタ非提供が意図的か

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 委譲先 §で回答 | 関連 GAP |
|---|---|---|---|
| 2.1 観点 1 | GREEN | SPEC-LGX-005 REQ.04（起点 0、max_depth 超を返さない → 0 で起点のみ） | — |
| 2.1 観点 2 | GREEN | SPEC-LGX-005 REQ.04（省略時 = 無制限 = 到達可能全ノード） | — |
| 2.1 観点 3 | GREEN | LGX-COMPAT-001 §3 グローバル規約（構文レベル誤り = exit 2）。`--max-depth` の値型/範囲は引数パーサ層が所有 | — |
| 2.1 観点 4 | GREEN | SPEC-LGX-005 REQ.05（起点不在 = 空）+ REQ.06（visited で停止）。孤立起点は起点のみ返す（REQ.04 深度 0） | — |
| 2.1 観点 5 | GREEN | (該当なし) — 敵対的精査: minor (verification: low-value, 人間判断で drop 可)。--max-depth 明示時の打ち切りは利用者認識済み・到達集合不変、truncated フラグは DD 委譲済み【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-085 |
| 2.2 観点 6 | GREEN | SPEC-LGX-005 REQ.05（空結果・非エラー）。exit 0 は OBS.05/USE.04 と整合（観点 23 参照） | — |
| 2.2 観点 7 | GREEN | 敵対的精査 OUT_OF_SCOPE: 未解決エッジの検出・除外・記録は SPEC-LGX-002 REQ.11（unresolved_edges）、その報告は SPEC-LGX-004 の check（Warning）が所有。走査は構造上 SPEC-LGX-002 が構築した解決済み部分グラフ上で動作し、可視化の責務を SPEC-005 が二重に負わない。GAP-LGX-088 削除 | — |
| 2.2 観点 8 | GREEN | SPEC-LGX-005 REQ.06（無限ループ禁止）+ NFR-LGX-001 SEC.03/REQ.10 入力耐性（panic 禁止は SPEC-LGX-002 が所有） | — |
| 2.3 観点 9 | GREEN | 敵対的精査 ALREADY_ANSWERED: SPEC-LGX-005 REQ.06（visited セットで循環防止）。visited が再訪を抑止する以上、起点は visited 登録され 1 回のみ出力されるのが標準 BFS の定義的帰結。self-loop はサイクルの最小ケースで REQ.06 の防御対象に内包。GAP-LGX-084 削除 | — |
| 2.3 観点 10 | GREEN | SPEC-LGX-005 REQ.03/REQ.04（BFS レベル = 深度、depth_map を返す） | — |
| 2.3 観点 11 | GREEN | 敵対的精査 ALREADY_ANSWERED: SPEC-LGX-005 REQ.03（BFS + 同一 depth_map）+ REQ.04（深度 = BFS レベル）。BFS は定義上、最初の到達 = 最短深度を記録し再訪では更新しない。多経路/サイクルでも REQ.06 の visited 抑止と組合せ一意に定まる標準 BFS セマンティクス。GAP-LGX-083 削除 | — |
| 2.4 観点 12 | GREEN | SPEC-LGX-005 §1.2（実装詳細は DD）。走査は CLI プロセス起動時ロード後のメモリ内読み取りで、並行更新は構造上発生しない（REL.07 は DB 用で走査に無関係） | — |
| 2.5 観点 13 | GREEN | SPEC-LGX-005 REQ.04（省略時 = 無制限を互換対象 (d) として維持と明記） | — |
| 2.5 観点 14 | GREEN | SPEC-LGX-005 REQ.09（`--json` 機能化【v3 差分】、引数体系 (a)〜(f) 不変と明記） | — |
| 2.5 観点 15 | GREEN | SPEC-LGX-005 REQ.07 + LGX-COMPAT-001 §4 #11/#12（`impact`/`investigate` `<start>` `[--max-depth N]`） | — |
| 2.6 観点 16 | GREEN | SPEC-LGX-005 §1.2（走査は読み取り系）。書き込みを伴う要求は本 SPEC に存在しない | — |
| 2.7 観点 17 | GREEN | SPEC-LGX-005 REQ.05（不在起点 = 空）。任意文字列起点も「不在」に収束。形式検証は走査の責務外 | — |
| 2.8 観点 18 | GREEN | SPEC-LGX-005 REQ.04（深度 0 = 起点）+ REQ.09（走査順に起点を含む） | — |
| 2.9 観点 19 | GREEN | SPEC-LGX-005 REQ.09（ID 走査順 / type / path / 使用エッジ / 深度を含む。フィールド名・スキーマは DD 凍結対象と正しく委譲） | — |
| 2.9 観点 20 | GREEN | 敵対的精査 OUT_OF_SCOPE: drift 値の算出は SPEC-LGX-006（detect_drift / SCORE-INV-1）、判定の方針は LEGIXY-SPEC-001 §6（逆方向探索 = drift で枝刈り・怪しいノード優先）、しきい値 `drift_threshold` は `.legixy.toml`（LGX-COMPAT-001 §6）、消費コマンドの出力仕様は SPEC-LGX-006 §164 が SPEC-LGX-010 へ委譲、`--json` フィールド名は REQ.09 が DD 凍結に委譲済み。SPEC-005 は走査の到達集合列挙に責務を限定。GAP-LGX-086 削除 | — |
| 2.9 観点 21 | GREEN | 敵対的精査 OUT_OF_SCOPE（観点 7 と同根）: 未解決エッジ記録 = SPEC-LGX-002 REQ.11、報告 = SPEC-LGX-004 check。CTX-INV-5 連携は構築層 + check の役割分担で閉じており SPEC-005 走査出力の責務外。GAP-LGX-088 削除 | — |
| 2.10 観点 22 | GREEN | LGX-COMPAT-001 §3（グローバル 3 種を全コマンドで受理）+ §7 順守事項 | — |
| 2.10 観点 23 | GREEN | NFR-LGX-001 OBS.05（0=OK）/ USE.04 + SPEC-LGX-005 REQ.05（空結果は非エラー → exit 0） | — |
| 2.10 観点 24 | GREEN | SPEC-LGX-005 REQ.10（Admin Surface 限定、MCP-INV-1） | — |
| 2.11 観点 25 | GREEN | SPEC-LGX-005 REQ.03 + SPEC-LGX-002 REQ.08（IndexMap 挿入順）+ NFR REL.05（決定性） | — |
| 2.11 観点 26 | GREEN | 敵対的精査 OUT_OF_SCOPE: エッジ格納順・IndexMap 挿入順・system 生成 parent_child エッジの整列は SPEC-LGX-002 REQ.08（決定論的順序 / IndexMap）の所有領域。TP §1 が「IndexMap 格納順は SPEC-LGX-002 が所有」と明示的に委譲済み。走査は SPEC-002 が固定した順序を消費するのみ。GAP-LGX-082 削除 | — |
| 2.11 観点 27 | GREEN | 敵対的精査: minor (verification: low-value, 人間判断で drop 可)。SPEC-LGX-002 REQ.04 が custom も有向エッジ（from/to）と定義し、有向エッジの「順方向に辿る」= from→to は一意。REQ.08（親→サブノード = 順方向）が既に一般則を具体化済みで custom も同型。CTX-INV-3 は意味的制約で SPEC-LGX-003 REQ.05 所有。本文への一文明記のみで挙動不変【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-081 |
| 2.11 観点 28 | GREEN | SPEC-LGX-005 REQ.08（親→サブノードが順方向）。from→to を順方向とする規則で一貫（観点 27 がこれを一般化する） | — |
| 2.11 観点 29 | GREEN | SPEC-LGX-005 REQ.06（DAG 破れでも無限ループ禁止、visited で停止） | — |
| 2.11 観点 30 | GREEN | 敵対的精査 ALREADY_ANSWERED（観点 9 と同根）: SPEC-LGX-005 REQ.06（visited で循環防止）。self-loop は最小サイクルで visited 抑止に内包され、起点を再出力せず有限停止するのは標準 BFS の定義的帰結。GAP-LGX-084 削除 | — |
| 2.11 観点 31 | GREEN | SPEC-LGX-005 REQ.01/02（3 種別を常に対象）+ §1.2（走査の実装詳細は DD）。エッジ種別フィルタ非提供は CLI 互換契約（LGX-COMPAT-001 §4 #11/#12 に `--kind` 等のフラグなし）と整合し、意図的と判断 | — |

### 判定サマリ（2026-06-09 敵対的精査パス後）
- 総観点数: 31
- GREEN: 29
- RED: 2（GAP-LGX-081, 085 — いずれも minor。verification: low-value, 人間判断で drop 可）

> **敵対的精査パス（2026-06-09）**: 当初 RED 9 行 / GAP 7 件のうち 5 件を削除（観点 7/21 → GAP-088、観点 9/30 → GAP-084、観点 11 → GAP-083、観点 20 → GAP-086、観点 26 → GAP-082）。内訳:
> - **OUT_OF_SCOPE**: GAP-082（エッジ格納/IndexMap 順 → SPEC-LGX-002 REQ.08）、GAP-086（drift 算出 → SPEC-LGX-006、判定方針 → LEGIXY-SPEC-001 §6、しきい値 → `.legixy.toml`、出力仕様 → SPEC-LGX-010、スキーマ → DD）、GAP-088（未解決エッジ記録 → SPEC-LGX-002 REQ.11、報告 → SPEC-LGX-004 check）
> - **ALREADY_ANSWERED**: GAP-083（BFS 最短深度・再訪不更新 = REQ.03/04 の定義的帰結）、GAP-084（visited による self-loop / 起点 1 回出力 = REQ.06 の定義的帰結）
> - **維持（minor）**: GAP-081（custom from→to 方向の本文明記 — REQ.04 の有向エッジ定義 + REQ.08 で実質確定済み、文章整備のみ）、GAP-085（max_depth 打ち切り通知 — 利用者認識済み・到達集合不変、DD 委譲済み）。いずれも挙動契約を変えない low-value で人間判断で drop 可。

## 4. ステータスの決定

2026-06-09 の敵対的精査パス後も RED 観点が 2 件（観点 5 / 観点 27、いずれも minor）残るため、本 TP のステータスは `**ステータス**: green` を維持する。なお 2 件とも「verification: low-value, 人間判断で drop 可」であり、人間が drop を承認すれば全 GREEN（green）へ遷移しうる。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値, §エラーハンドリング, §状態遷移, §並行性, §バージョニング・互換性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性, §FFI / 境界 API 観点
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（observability の観点 20/21 着想）
- 領域固有（グラフ走査・決定論）: BFS 決定性 / 種別混在エッジ整列 / DAG 破れ時の停止 / self-loop（本 TP で新規追加。AT で確認後に core-perspectives.md へ昇格候補）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成、観点 31 件追加、RED 7 件（GAP-LGX-081〜088、087 欠番）を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 5 件 / 維持 2 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
