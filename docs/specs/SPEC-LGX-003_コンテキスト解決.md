Document ID: SPEC-LGX-003

# SPEC-LGX-003: コンテキスト解決

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-003 |
| Version | 0.7.2 |
| Status | Approved（人間査読済） |
| Date | 2026-04-26 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §5.1, §5.4（Phase 2 Block B） |
| 対応 NFR | NFR-LGX-001.PERF.03, REL.05 |
| 対応 UC | UC-LGX-002, UC-LGX-004 |

---

## 1. 本文書の位置づけ

### 1.1 目的

`compile_context` MCP ツールの動作要求を legixy 向けに明文化する。v0.1.0 の動作を継承しつつ、legixy 固有の粒度制御拡張を規定する。

### 1.2 スコープ

**含む:** compile_context の入出力、粒度制御、Layer Guidelines 付加
**含まない:** 実装詳細（→ DD-LGX-00x、後続ブロックで定義）、MCP プロトコル層（→ SPEC-LGX-009）

---

## 2. 参照文書

- LGX-EXT-001 §5.1 compile_context の拡張
- LGX-EXT-001 §6 MCP-INV-1 との整合
- LEGIXY-SPEC-001 §10 CTX-INV-1〜4
- v0.1.0 の compile_context 実装（`old.source/RustCLI/`, `old.source/TypeScriptMCP/`）を慣例仕様として参照

---

## 3. 要求事項

### SPEC-LGX-003.REQ.01: 入力

**内容:** compile_context は以下の入力を受け付ける:
- `target_files`（必須）: これから作成・編集するファイルのパス配列
- `granularity`（legixy 新規、任意）: 返却する成果物の粒度（`document` / `subnode`。詳細は REQ.03）
- その他オプション: v0.1.0 を継承

**根拠:** LGX-EXT-001 §5.1
**検証方法:** MCP スキーマテスト

### SPEC-LGX-003.REQ.02: 上流成果物の返却

**内容:** target_files に対応するノードを起点として、成果物連鎖（chain エッジ）を逆方向に辿り、上流成果物の内容を返却する。具体的には以下を含む:
- Upstream Artifacts: 上流ファイルの本文またはサブノード本文
- Layer Guidelines: 該当レイヤのガイドライン
- Additional Guidelines: プロジェクト固有の補足

**根拠:** LEGIXY-SPEC-001 §2, CLAUDE.md「事前ガイダンス義務」
**検証方法:** UC-LGX-002 シナリオテスト

### SPEC-LGX-003.REQ.03: 粒度制御（legixy 新規）

**内容:** `granularity` パラメータで返却粒度を制御する:
- `document`（既定）: ドキュメント全文を返す（v0.1.0 互換）
- `subnode`: 該当サブノードの本文のみを返す

legixy ではこの 2 値のみサポートする。`auto`（関連度ベースの自動選択）等のモードは将来拡張として保留する（導入時は「関連度」の定義・閾値を含む仕様を別途策定）。
**根拠:** LGX-EXT-001 §5.1
**検証方法:** UC-LGX-004 粒度別テスト（`document` / `subnode` の 2 値）

### SPEC-LGX-003.REQ.04: CTX-INV-1 決定論的順序

**内容:** 同一の target_files と granularity に対して、常に同一順序・同一内容の結果を返す。
**根拠:** LEGIXY-SPEC-001 §10 CTX-INV-1
**検証方法:** 反復呼出しテスト

### SPEC-LGX-003.REQ.05: CTX-INV-2〜4 継承

**内容:** LEGIXY-SPEC-001 §10 で規定される CTX-INV-2（グラフ整合性）、CTX-INV-3（カスタムエッジ独立性）、CTX-INV-4（DAG 制約）を維持する。legixy で新たに違反しないこと。
**根拠:** LEGIXY-SPEC-001 §10
**検証方法:** 不変条件チェックのテスト

### SPEC-LGX-003.REQ.06: MCP-INV-1 維持

**内容:** 粒度制御のために新 MCP ツールを追加しない。`compile_context` のオプション引数としてのみ提供する。
**根拠:** LGX-EXT-001 §6.1
**検証方法:** SPEC-LGX-009.REQ.02 の充足

### SPEC-LGX-003.REQ.07: 監査ログ

**内容:** compile_context の全呼出しは engine.db の `context_log` テーブルに記録される。legixy では `granularity` カラムを追加する。
**根拠:** LGX-EXT-001 §4.3（context_log 拡張）
**検証方法:** DB スキーマ検証（TS-LGX-001 T-DB-001）

### SPEC-LGX-003.REQ.08: サブノード親への解決

**内容:** サブノードが target_files に含まれる場合、compile_context はまずサブノードを起点とし、必要に応じて親ドキュメントの上流を辿る。ParentChild エッジも連鎖の一部として扱う。
**根拠:** LGX-EXT-001 §3.4, §5.1
**検証方法:** サブノード起点テスト

### SPEC-LGX-003.REQ.09: 並行呼出し安全性

**内容:** 複数の `compile_context` が同時に呼び出された場合も、各呼出しは独立に正しい結果を返す。engine.db への書き込み（`context_log`）は SQLite WAL + `busy_timeout` により排他制御する。共有状態の破損や応答の混在を発生させてはならない。
**根拠:** NFR-LGX-001.SEC.02, REL.03
**検証方法:** 並行実行ストレステスト（複数 Claude Code セッション同時起動を模擬）

### SPEC-LGX-003.REQ.10: 返却セクションの配置順序（CACHE-INV-2）

**内容:** `compile_context` の返却内容は以下の 6 セクションを**この順序**で構成する。順序は `--granularity` の値に依らず一貫する。

1. Layer Guidelines（プロジェクト全体で共通、最も安定）
2. Additional Guidelines（対象レイヤーで共通、やや安定）
3. キャッシュブレーク点マーカ（安定部分と変動部分の境界、REQ.12 で規定）
4. Upstream Artifacts（タスクごとに変動、決定論的順序で整列）
5. Target Node Metadata（対象ノード固有、最も変動）
6. Custom Documents（カスタムエッジ由来文書: from_id / to_id / file_path / reason + body。from_id→to_id 辞書順で整列。Target Node Metadata の後、キャッシュブレーク点より後の最変動部）

> **6 セクション化の経緯（spec-change 2026-06-13、ADR-LGX-019、TRIAGE §4 #1）**: 旧 REQ.10 は「5 セクション」と規定していたが、対応 UC-LGX-002 の ContextResult は `custom_documents` フィールドを返却に含めると明記しており、v3 実装（`section_formatter.rs`）も Custom Documents を 6 番目に出力していた。旧「5」は UC-LGX-002 との同期漏れであり、本改訂で 6 セクションに正準化する（UC・v3 と整合）。

安定度が高いものを先、低いものを後に配置することで、Anthropic Prompt Caching のヒット条件（プロンプト先頭からの完全一致）を満たしやすくする。
**根拠:** LGX-EXT-002 §3.2, §5.2 CACHE-INV-2, §8.1
**検証方法:** 返却内容のセクション順序検査テスト

### SPEC-LGX-003.REQ.11: 各セクション内の決定論的整列

**内容:** 各セクション内のアイテムは以下の決定論的順序で整列する。
- **Layer Guidelines**: ファイルパスの**辞書順昇順**（バイト単位比較）
- **Additional Guidelines**: ファイルパスの**辞書順昇順**（バイト単位比較）
- **Upstream Artifacts**:
  - `--granularity document` の場合: **ノード ID の辞書順昇順**
  - `--granularity subnode` の場合: 親ドキュメント ID の辞書順昇順でグループ化し、同一ドキュメント内では**アンカー出現順**（ドキュメント内の物理的な位置順）
- **Target Node Metadata**: 変動が大きいためキャッシュブレーク点より後ろに集約

「グラフ探索の発見順」や「エッジのスコア順」は入力やスコア計算結果により順序が変動するためキャッシュフレンドリーではない。これらが必要な場合は Target Node Metadata に別途含める。
**根拠:** LGX-EXT-002 §3.3
**検証方法:** 同一入力に対する複数回呼出しで順序が一致することの単体テスト

### SPEC-LGX-003.REQ.12: キャッシュブレーク点マーカ

**内容:** Additional Guidelines 末尾と Upstream Artifacts 先頭の間に、以下の HTML コメント形式のマーカを挿入する:

```
<!-- cache-breakpoint: stable-end -->
```

このマーカは Claude Code 側または上位エージェントがキャッシュ制御点として解釈するためのヒントである。HTML コメントのため表示や意味には影響しない。Phase 1 ではマーカ挿入のみを規定し、Claude Code 側での解釈挙動は前提としない（将来の活用余地の確保が目的）。
**根拠:** LGX-EXT-002 §3.4, §8.2
**検証方法:** 返却内容にマーカ文字列が 1 箇所含まれることの単体テスト

### SPEC-LGX-003.REQ.13: 大規模返却時のエラー（CACHE-INV-3）

**内容:** Rust CLI が生成した返却本文が **500,000 文字を超える場合**、`compile_context` は**エラーを返却する**。本文の切り捨てや自動要約は行わない。

エラー返却形式:
```
Error: compile_context result exceeds 500,000 characters.
Current size: <N> characters.
Suggested action:
  - Try --granularity subnode for finer-grained retrieval.
  - Narrow the target scope.
```

切り捨てや自動要約は決定論性（CACHE-INV-1）の検証を困難にし、情報欠落時の挙動予測を不可能にするため、明示的なエラーとして粒度制御をユーザに委ねる（P-02「判断は人間に委ねる」との整合）。

**カウント単位（前段ループ反復 1 で確定）:** 「文字」は **Unicode コードポイント数**（Rust `.chars().count()` 相当）で計測する。SPEC-LGX-009 REQ.13 の `maxResultSizeChars` と同一概念・同一単位である。なお REQ.14 の「バイト単位で同一」は出力の決定論（同一入力 → 同一バイト列）の要求であり、本カウント単位とは独立で矛盾しない。
**根拠:** LGX-EXT-002 §4.3, §5.2 CACHE-INV-3, §8.3、NFR-LGX-001.PERF.09、QSET-LGX-003 Q1 回答（2026-06-07。v3 実測〔`crates/te-ctx/src/section_formatter.rs:129-144` の `.chars().count()`〕の正準化）
**検証方法:** 超過シナリオで適切なエラーと提案が返されることの単体テスト

### SPEC-LGX-003.REQ.14: バイト単位決定論（CACHE-INV-1）

**内容:** 同じ入力（グラフ定義、engine.db 状態、引数）に対して、`compile_context` の返却内容は**順序、区切り文字、空白を含めてバイト単位で同一**となる。本 REQ は CTX-INV-1（決定論保証）を**バイト列レベル**まで強化する。

バイト単位決定論の実現には以下の要素が共同で寄与する:
- REQ.10（セクション配置順序の固定）
- REQ.11（各セクション内の決定論的整列）
- REQ.12（マーカの固定位置）
- SPEC-LGX-002.REQ.08（順序保持 TOML パーサ）

**根拠:** LGX-EXT-002 §3.5, §5.2 CACHE-INV-1
**検証方法:** 同じグラフ定義・同じ engine.db 状態・同じ引数に対する複数回呼出しで返却バイト列が完全一致することの単体テスト + 統合テスト（`--granularity document` / `--granularity subnode` の両モード）

### SPEC-LGX-003.REQ.15: outline_only 出力（v0.4.0、Phase 2 Block B）

**内容:** `compile_context --outline-only` 指定時、各 upstream artifact の本文を **ATX 見出し（h1〜h3）の階層リスト**に置換する。本文（見出し以外のテキスト）は出力に含めない。

**出力形式:**
- 各見出しを `- {title}` 行として出力（先頭 `-` + 半角スペース + 見出しテキスト）
- 見出しレベルに応じてインデント `  `（半角スペース 2 個）× `(level - 1)` を付与（h1 はインデント 0、h2 は 2、h3 は 4）
- `####` 以降（h4 以降）は対象外
- 見出し記号と本文の間にスペースが無い行（`#abc` 等）は対象外

`--granularity subnode` と組合せた場合、各サブノード artifact の body は当該サブノードの **anchor 文字列のみ**となる（本文省略）。

**見出し皆無時の出力（GAP-LGX-047 対応）:** 対象 artifact に h1〜h3 見出しが 1 つも存在しない場合（本文に見出しなし・h4 以降のみ・スペース無し `#abc` のみ等）:
- **artifact の枠（ノード ID 等のヘッダ構造）は維持し、body を空とする**ことを正準とする（artifact ごとの省略は採らない — 上流に存在すること自体が情報であり、REQ.10「セクション構成は件数非依存で固定」と整合）
- プレースホルダ文字列は挿入しない。空 body の正確なフォーマット（改行・空白）は CACHE-INV-1（バイト決定論）を満たす固定形として DD で確定する
- subnode 粒度で anchor も存在しない縮退ケースも同規約（枠維持・body 空）

**根拠:** LGX-EXT-001 §5.4 Phase 2 Block B、`compile_context` のトークン消費削減（vnstudio 88,303 bytes → outline 数 KB の削減実績）、GAP-LGX-047
**検証方法:** TS-LGX-002 §15 T-CC-OUTLINE-001（h1〜h3 見出しが含まれ、本文が含まれないことを確認。見出し皆無 artifact で枠維持 + body 空 + バイト決定論のテストを含む）

### SPEC-LGX-003.REQ.16: sections フィルタ（v0.4.0、Phase 2 Block B）

**内容:** `compile_context --sections <ids>` でコンマ区切りのサブノード ID を指定すると、`--granularity subnode` 経路で展開される子サブノードのうち、**指定 ID と一致するサブノードのみ**を upstream に含める。

**入力形式:**
- コンマ区切り文字列（例: `"DD-X-001#abc,DD-X-001#def"`）
- 各 ID は graph.toml 上の subnode ノード ID と完全一致を要求する（前後空白は許容）
- 指定 ID が graph.toml 上のサブノードに存在しない場合、当該 ID は単に除外される（エラーにしない）
- 親ドキュメントノード ID（サブノード ID 形式 `#` を含まないもの）を指定した場合、当該 ID は subnode ID と一致しないため**単に除外される**（エラーにせず正常終了。結果が空になりうる。v3 実測の正式化）。【v3 差分】この場合 stderr に Info 診断（親ドキュメント ID は --sections では無効である旨とサブノード ID の使用案内）を出力する（stdout・終了コードは不変、QSET-LGX-003 Q2 回答 2026-06-07）
- **不正形式・縮退入力（GAP-LGX-045 対応）**: trim 後に空となるトークン（連続コンマ `,,`・先頭/末尾コンマ・空白のみ）は**無視する**（存在しない ID と同経路・エラーにしない）。**重複 ID は dedup（set セマンティクス）**し、返却は REQ.11 の決定論的整列（親 ID 辞書順 + アンカー出現順）に従う — 指定順・指定回数は出力に影響しない（入力の表記揺れで出力バイト列が変わらないこと = CACHE-INV-1 保全）。全トークンが無効/空の場合は空 upstream で正常終了（exit 0）

**動作:**
- `--granularity subnode` と組合せた時のみ有効（`document` 粒度時は無視）
- `sections` フィルタを通過したサブノードのみが upstream artifact として返却される
- フィルタ通過した結果が空の場合、当該親ドキュメントは upstream に登場しない

**根拠:** LGX-EXT-001 §5.4 Phase 2 Block B、機能 C（IdSemanticDrift）の特定サブノード対比検証用途、Block F 連動
**検証方法:** TS-LGX-002 §15 T-CC-SECTIONS-001（指定 ID のみ返却されることを確認）

### SPEC-LGX-003.REQ.17: depth_limit（v0.4.0、Phase 2 Block B）

**内容:** `compile_context --depth N` で上流走査を **N 階層に制限**する。

**入力形式:**
- 正の整数 N（1 以上）
- N=1 で target の直接の親（chain_distance == 1）のみ
- N=2 で祖父まで（chain_distance ≤ 2）
- 省略時は無制限（v0.2.0 互換、`SPEC → UC → RB → SEQ → DD → TS → TC → SRC` の 7 階層全てを返却）

**動作:**
- `UpstreamWalker::walk_chain_parent_only_with_depth(start, depth_limit)` の BFS depth 制御で実現
- N=0 を CLI 経由で指定した場合、walker は**空集合を返し正常終了する**（exit 0。v3 実測の正式化）。MCP 経由では zod 制約（depth ≥ 1、SPEC-LGX-009 REQ.15）により reject されるため、CLI と MCP で受理範囲が異なることを正準とする。【v3 差分】空集合となる場合は stderr に Info 診断を出力する（stdout・終了コードは不変、QSET-LGX-003 Q2 回答 2026-06-07）

**根拠:** LGX-EXT-001 §5.4 Phase 2 Block B、深い chain での絞り込み（vnstudio dogfeeding 観察事項）
**検証方法:** TS-LGX-002 §15 T-CC-DEPTH-001（`--depth 1` で直接上流のみ、N+1 階層は除外されることを確認）

### SPEC-LGX-003.REQ.18: フラグ組合せの優先順位（前段ループ反復 1 新設）

**内容:** `--granularity` / `--outline-only` / `--sections` / `--depth` の組合せ時の挙動を以下のマトリクスで確定する:

| 組合せ | 確定挙動 |
|---|---|
| `--outline-only` × `--granularity document` | 各 artifact の本文を**見出し階層リストに置換**して返す（全文ではない。REQ.15 の出力形式に従う） |
| `--sections` × `--granularity document` | `--sections` は**無視**（REQ.16 のとおり） |
| `--outline-only` × `--sections`（subnode 粒度） | **sections フィルタが先、outline 化が後**。sections で絞り込んだ後、残った subnode の body を anchor のみに置換（REQ.15 末尾の規定と整合） |
| `--depth` × 上記すべて | 直交（walker の探索深度のみを制御し、粒度・outline・sections と干渉しない） |

**根拠:** QSET-LGX-003 Q3 回答（2026-06-07）、v3 実測（`crates/te-ctx/src/compiler.rs:201-253` の適用順）
**検証方法:** 組合せマトリクスの各行に対応する E2E テスト（UC-LGX-004 粒度別テストの拡充）

### SPEC-LGX-003.REQ.19: 監査ログ書込失敗時の終端状態（GAP-LGX-041 対応）

**内容:** compile_context の本処理が成功し、context_log（REQ.07）への記録のみが失敗した場合の正準挙動を**本処理優先（記録欠落を許容し成功扱い）**と定める:
- 本処理の結果生成と context_log 書込は**別トランザクション**とする
- engine.db が存在する状態で本処理成功・context_log 書込失敗となった場合、**結果を stdout に返却し exit 0** とする。記録失敗は stderr に Warning 診断として出力する
- 書込失敗の判定は busy_timeout（NFR REL.07、5000ms）超過によるリトライ打ち切りを含む
- **MCP-INV-4 との関係**: 「全呼出記録」は**ベストエフォート**であり、完全性保証は「DB が利用可能な場合に限る」。欠落の検出可能性は以下の二経路で確保する:
  1. **プロセス内（同期）**: stderr に Warning 診断を出力する（CLI 直接実行時の確認用）
  2. **MCP 経由（非同期・Agent 到達保証）**: SPEC-LGX-009.REQ.03 の `_meta["legixy/warnings"]` 転送により、MCP 経由呼出し時でも Agent が各呼出しで書込失敗を検知できる

  上記 2 経路はいずれも**呼出し単位の揮発的通知**である。書込失敗が連続する場合、Agent はセッション内の累積 Warning から恒常的障害を検知できる。永続的な失敗履歴の記録は本 SPEC の範囲外とし、ディスク満杯等の根本原因の解消を運用対応とする。可用性 > 監査完全性の設計判断は ADR に記録する

**根拠:** GAP-LGX-041。凍結終了コード規約（LGX-COMPAT-001 v1.0.1: exit 1 は受理済み引数の意味的不正・実行時失敗）に対し、記録失敗は本処理の意味的不正ではないため exit 1 としない
**検証方法:** context_log を書込不能にした fixture（読取専用 DB 等）で、結果返却 + exit 0 + stderr Warning を確認するテスト

### SPEC-LGX-003.REQ.20: 起点ノード不在・上流部分欠損時の扱い（GAP-LGX-043 対応）

**内容:** compile_context の入力解決が不完全な場合の挙動を以下に確定する:
1. **起点ノード不在パス**: 指定された起点のうち graph.toml に未登録のものは**無視し、残りの起点で解決して exit 0**。命名規約からの chain 位置推定は行わない（推測排除）。全起点が未登録の場合は空の upstream で exit 0。未解決の起点は Target Node Metadata セクションおよび stderr Info 診断に記録する
2. **上流連鎖途中の欠損**（エッジ先ノードのファイル不在等）: 欠損ノードを**飛ばして残りの上流を返す部分成功・exit 0**。欠損は出力内に**決定論的に記録**する（CTX-INV-1 / REQ.14 のバイト単位決定論を保全するため、欠損の記録位置・形式も決定論的とする）

**根拠:** GAP-LGX-043。REQ.19（成功後の副作用失敗）と本 REQ（入力解決の不完全性）は成功境界の両側で別概念のため別 REQ とする
**検証方法:** ①未登録起点を含む呼出、②全起点未登録、③上流途中欠損、の各 fixture で exit 0・決定論的出力・診断記録を確認するテスト

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-1（決定論保証） | 実装 | REQ.04（同一入力→同一結果）, REQ.14（バイト単位に強化） |
| CTX-INV-2（グラフ整合性） | 関連 | REQ.05（legixy で違反しないと宣言、実装本体は SPEC-LGX-002） |
| CTX-INV-3（カスタムエッジ独立性） | 実装 | REQ.05（compile_context の動作として上流に影響させない） |
| CTX-INV-4（DAG 制約） | 関連 | REQ.05（前提として要求）+ 実装本体は SPEC-LGX-002.REQ.07 |
| FB-INV-3（承認前不変性） | 実装 | pending の Proposal が compile_context 結果に影響しない動作を実装 |
| FB-INV-4（DB 不在時安全性） | 実装 | engine.db が存在しない場合でも graph.toml のみで上流情報を返せる動作を実装 |
| MCP-INV-1（Agent Surface 限定） | 実装 | REQ.06（新 MCP ツール追加禁止） |
| MCP-INV-4（監査ログ完全性） | 実装 | REQ.07（compile_context 呼出しを context_log に記録）, REQ.19（書込失敗時はベストエフォート — 完全性は DB 利用可能時に限る） |
| STATE-INV-1（ステートレス性） | 関連 | compile_context 実行時に context_log 書き込み以外の永続状態を持たない |
| CACHE-INV-1（バイト単位決定論） | 実装 | REQ.14 |
| CACHE-INV-2（セクション配置順序） | 実装 | REQ.10 |
| CACHE-INV-3（大規模返却エラー） | 実装 | REQ.13 |

**本 SPEC が関与しない不変条件:** MCP-INV-2, MCP-INV-3、SUBNODE-INV-1〜6、SCORE-INV-1/2、FB-INV-1/2/5、STATE-INV-2、CACHE-INV-4

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | F-01 修正: CTX-INV-2/3/4 の名称を LEGIXY-SPEC-001 §10 と一致させた。F-05 追加: REQ.09（並行呼出し安全性）を追加。F-11 修正: REQ.03 の granularity=auto を legixy から削除、document/subnode の 2 値のみサポート |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列（実装/関連）を追加、MCP-INV-4 を追加、対象外不変条件（MCP-INV-2/3、SUBNODE-INV-*）を明記 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-d 対応: LGX-EXT-002 統合で REQ.10（セクション配置順序、CACHE-INV-2）、REQ.11（決定論的整列）、REQ.12（キャッシュブレーク点マーカ）、REQ.13（大規模返却エラー、CACHE-INV-3）、REQ.14（バイト単位決定論、CACHE-INV-1）の 5 件を追加。§4 表に CACHE-INV-1/2/3、FB-INV-3/4、STATE-INV-1 を追加 |
| 2026-04-26 | 0.4.0 | LGX-EXT-001 Phase 2 Block B 対応（v0.4.0-alpha3 実装の formal 化）: REQ.15（outline_only 出力、h1〜h3 見出し階層リスト化）、REQ.16（sections フィルタ、コンマ区切りサブノード ID 指定）、REQ.17（depth_limit、上流走査 N 階層制限）の 3 件を追加。`--granularity subnode` 時の親→子サブノード展開挙動（vnstudio 観察事項 1 解消）を REQ.03 / REQ.08 経由で追加保証。親文書欄に LGX-EXT-001 §5.4 を追記 |
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-003 回答 → SPP-LGX-003 承認）対応: REQ.13 に「文字」= Unicode コードポイント数を確定（SPEC-LGX-009 REQ.13 と同一単位、REQ.14 のバイト決定論とは独立）。REQ.16 の親ドキュメント ID 指定・REQ.17 の depth 0 を「空集合で正常終了 + stderr Info 診断【v3 差分】」に確定。REQ.18 フラグ組合せマトリクスを新設 |
| 2026-06-10 | 0.7.0 | TP[SPEC] GAP 解消（人間承認 2026-06-10）: GAP-LGX-041 対応で REQ.19（監査ログ書込失敗時の終端状態 — 本処理優先・別 Tx・exit 0 + stderr Warning、MCP-INV-4 をベストエフォートに確定、ADR 記録）を新設、§4 MCP-INV-4 行に REQ.19 を追記。GAP-LGX-043 対応で REQ.20（起点不在は無視して残りで解決 exit 0・上流部分欠損は部分成功 exit 0・欠損の決定論的記録）を新設 |
| 2026-06-12 | 0.7.2 | ADR-LGX-004 可観測性強化（spec-change 2026-06-12）: REQ.19 の欠落検出路を「stderr のみ」から「プロセス内 stderr + MCP 経由 `_meta["legixy/warnings"]`」の二経路に変更。MCP 経由での Agent 到達保証を明示（SPEC-LGX-009.REQ.03 と連動） |
| 2026-06-13 | 0.8.0 | spec-change（ADR-LGX-019、TRIAGE §4 #1）: REQ.10 を 5→6 セクションに改訂し Custom Documents を 6 番目（Target Node Metadata の後、最変動部）として正式化。UC-LGX-002 ContextResult（custom_documents）/ v3 実装と整合。旧「5」は UC 同期漏れ |
| 2026-06-10 | 0.7.1 | weak GAP 解消（人間裁定 fix・承認 2026-06-10）: GAP-LGX-045 対応で REQ.16 に縮退入力規則（trim 後空トークンは無視・重複 ID は dedup + REQ.11 整列・全無効は空 upstream exit 0、CACHE-INV-1 保全）を追記。GAP-LGX-047 対応で REQ.15 に見出し皆無時の出力規約（枠維持・body 空・プレースホルダなし・固定フォーマットは DD、REQ.10 整合）を追記 |
