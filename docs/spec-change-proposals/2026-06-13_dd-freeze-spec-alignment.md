# SPEC 修正案 — DD 凍結フェーズで顕在化した SPEC 整合（2026-06-13）

> **ステータス: 承認済み・反映完了（2026-06-13）**
> DD（詳細設計・言語確定）フェーズで、v3 実装を引数互換の底本として採用した結果、
> SPEC 文言と差分・欠落が残った箇所を整合する改訂案である。
> **SPEC 変更は人間承認が必要（HR1）。** 本書の M-1〜M-4 は **2026-06-13 に人間承認** され、
> SPEC-LGX-002（0.4.2→0.4.3、M-1/M-2）・SPEC-LGX-008（0.7.0→0.7.1、M-3/M-4）へ反映済み。
> A-1（DD 修正で解決済）と B 群（凍結確認・SPEC 変更不要）は付録に記録する。

---

## 0. 背景

DD-LGX-001〜013 は LGX-COMPAT-001（引数互換）を守るため `traceability-engine.v3.chg_to_lexigy` の
実装を底本にした。その過程で、(A) v3 を採用したが SPEC 文言と乖離、(B) v3 を採用したが上流に底本なし、
(C) SPEC 文言自体に誤り・欠落、の 3 区分の未決事項が surfaced した。本書はうち **C 区分（SPEC 文言訂正）**
と **A 区分のうち SPEC 改訂が適切なもの（A-2/A-3）** を改訂案として提示する。

| 区分 | 項目 | 本書での扱い |
|---|---|---|
| A-1 | subnode 整列キー（S2-21） | **DD 修正で解決済**（SPEC 準拠＝出現順を採用、SPEC 変更不要）。付録 A |
| A-2 | heading_levels（S2-08） | **M-2**（SPEC-002 記述追加） |
| A-3 | コードフェンス内 `#`（S2-09） | **M-1**（SPEC-002 記述追加） |
| B-1〜B-4 | 実装細部の凍結確認 | **凍結確認**（SPEC 変更不要）。付録 B |
| C-1 | v0.1.0 移行元 DB 名 | **M-3**（SPEC-008 REQ.01 訂正） |
| C-2 | `[id.chains]` 変種受理 | **M-4**（SPEC-008 REQ.03/03a 記述追加） |

---

## 1. 修正一覧（M-1〜M-4）

| # | 対象 SPEC | 変更節 | 変更の性質 | バージョン bump |
|---|---|---|---|---|
| M-1 | SPEC-LGX-002 | REQ.05 | 記述追加（v3 挙動の明文化） | 0.4.2 → 0.4.3 |
| M-2 | SPEC-LGX-002 | REQ.05 | 記述追加（内部属性の明記） | 0.4.2 → 0.4.3（M-1 と同一改訂） |
| M-3 | SPEC-LGX-008 | REQ.01 | 文言訂正（移行元 DB 名） | 0.7.0 → 0.7.1 |
| M-4 | SPEC-LGX-008 | REQ.03 / REQ.03a | 記述追加（`[id.chains]` 受理） | 0.7.0 → 0.7.1（M-3 と同一改訂） |

---

## 2. M-1: SPEC-LGX-002 REQ.05 — コードフェンス内 `#` 行の非認識を明文化（A-3）

### 問題の根拠

SPEC-LGX-002.REQ.05 は自動生成サブノードを「Markdown の h2/h3 見出しから抽出」と規定するが、
**コードフェンス（` ``` ` ブロック）内の `#` 始まり行を見出しとして扱うか**を一切規定していない。
v3 実装（`crates/lx-graph/src/subnode/extractor.rs`）は**コードフェンスを認識せず**、フェンス内の
`# foo` も見出しとして抽出する。REQ.05 根拠は「既存プロジェクトの graph.toml に永続化済みの ID と
一致させるため v3 実測の生成式を凍結する」と明記しており、**フェンス認識へ変更するとサブノード集合と
ID が変わり、永続化済み ID 互換を破る**。したがって v3 挙動（非認識）を凍結し、SPEC に明記する。

### 修正テキスト

**対象箇所:** SPEC-LGX-002 §3 REQ.05「自動生成」bullet の直後に注記を追加

**ADD（「h1 と h4 以降は抽出対象外。」の直後）:**
```
- **コードフェンス内 `#` 行の扱い（v3 実測凍結）:** Markdown コードフェンス（``` / ~~~ ブロック）
  内の `#` 始まり行も見出しとして抽出される（フェンスを認識しない）。これは v3 実装の挙動であり、
  既存 graph.toml に永続化済みのサブノード ID との一致（本 REQ 根拠）を保つため凍結する。
  フェンスを認識して除外する変更はサブノード ID を変える破壊的変更であり、次版 SPEC 改訂として扱う。
```

### バージョン bump

SPEC-LGX-002 ヘッダ表 `Version: 0.4.2` → `0.4.3`、変更履歴に M-1/M-2 を 1 行で記録（§4 参照）。

---

## 3. M-2: SPEC-LGX-002 REQ.05 — heading_levels 内部属性の v3 踏襲を明記（A-2）

### 問題の根拠

v3 実装はサブノードの見出しレベル（h2 / h3）を内部属性として保持する（DD-LGX-003 の
`extract_subnodes_with_levels` が対応）。SPEC-LGX-002 はこの属性に言及しておらず、採否が未確定だった
（DD-LGX-003 §11 [要決定残存] S2-08）。現状 legixy にこの属性を消費する REQ は存在しないため、
**公開フィールドとしては規定せず、v3 踏襲の内部属性として保持を許容する**ことを明記する
（将来の consumer REQ 出現時に公開仕様化する拡張経路を DD が確保済）。

### 修正テキスト

**対象箇所:** SPEC-LGX-002 §3 REQ.05 末尾に注記を追加

**ADD:**
```
- **見出しレベル（heading_levels）内部属性:** 自動生成サブノードの見出しレベル（h2/h3）は、
  v3 踏襲の**内部属性**として保持してよい（DD-LGX-003 `extract_subnodes_with_levels`）。
  本属性を消費・公開する要件は現時点で存在せず、公開仕様化は consumer REQ の新設を伴う
  次版 SPEC 改訂とする。ID 生成（本 REQ 生成式）には影響しない。
```

### バージョン bump

M-1 と同一改訂（0.4.2 → 0.4.3）。

---

## 4. M-3: SPEC-LGX-008 REQ.01 — v0.1.0 移行元 DB 名の訂正（C-1）

### 問題の根拠

SPEC-LGX-008.REQ.01 は「**v0.1.0 フォーマットの `engine.db` を検出した場合**」と記すが、v0.1.0（前身
traceability-engine）の feedback データ（observations / proposals / custom_edges）は
**`.trace-engine/feedback.db`** に格納されていた（v3 `crates/lx-mig/src/db.rs`:
「v0.1.0 feedback.db → v3 engine.db データ移行。feedback.db の observations / proposals / custom_edges を
engine.db に単一トランザクションで copy」）。`engine.db` は**移行先**の名称であり、REQ.01 は移行元と
移行先を混同している。`.legixy/engine.db` が正準移行先である点は ADR-LGX-015 で確定済。

### 修正テキスト

**対象箇所:** SPEC-LGX-008 §3 REQ.01 冒頭文

**OLD:**
```
**内容:** v0.1.0 フォーマットの `engine.db` を検出した場合、legixy は以下のいずれかの方式で legixy スキーマに変換する:
```

**NEW:**
```
**内容:** v0.1.0 フォーマットのプロジェクト（feedback データを `.trace-engine/feedback.db` に保持し、
matrix 形式設定を伴う）を検出した場合、legixy は feedback.db の observations / proposals / custom_edges を
正準 `engine.db`（`.legixy/engine.db`、ADR-LGX-015）へ統合する形で、以下のいずれかの方式で legixy スキーマに変換する:
```

**併せて「デフォルト挙動」段落の訂正:**

**OLD:**
```
**デフォルト挙動（auto 未設定時）:** `check` 等の読み取り系コマンドが v0.1.0 engine.db を検出したら、**Error を返して明示的な `migrate` 実行を促す**（読み取り系コマンドが意図せず DB を書き換える副作用を避ける）。
```

**NEW:**
```
**デフォルト挙動（auto 未設定時）:** `check` 等の読み取り系コマンドが v0.1.0 プロジェクト（feedback.db + matrix 設定）を検出したら、**Error を返して明示的な `migrate` 実行を促す**（読み取り系コマンドが意図せず DB を書き換える副作用を避ける）。
```

### バージョン bump

SPEC-LGX-008 ヘッダ表 `Version: 0.7.0` → `0.7.1`。

---

## 5. M-4: SPEC-LGX-008 REQ.03 — `[id.chains]` multi-area 変種の受理を明記（C-2）

### 問題の根拠

SPEC-LGX-008.REQ.03 / REQ.03a は chain 定義として `[id.chain]`（単数形）のみを参照するが、
v0.1.0 には `[id.chains]`（複数形）+ `[id.areas]` による **multi-area 変種**が存在し、migrate は
これを受理する必要がある（DD-LGX-009 §2.4、ADR-LGX-018 #15 で受理を承認済）。SPEC が単数形のみを
規定すると、複数形変種の v0.1.0 入力が REQ.03a の「`[id.chain]` order 欠落」で誤って破損扱いになる。

### 修正テキスト

**対象箇所 1:** SPEC-LGX-008 §3 REQ.03「内容」bullet

**OLD:**
```
- `[id.chain]` の順序定義に基づき chain エッジを生成
```

**NEW:**
```
- `[id.chain]`（単数形）または `[id.chains]` + `[id.areas]`（複数形・multi-area 変種、ADR-LGX-018 #15）の順序定義に基づき chain エッジを生成。両表記を受理する
```

**対象箇所 2:** REQ.03「不正入力」bullet

**OLD:**
```
- **`[id.chain]` の `order` が欠落・不正な場合は破損（REQ.03a）として Error** — chain エッジを暗黙に 0 本として続行しない（構造情報の黙殺禁止）
```

**NEW:**
```
- **`[id.chain]` / `[id.chains]` のいずれも存在しない、または存在する側の `order` が欠落・不正な場合は破損（REQ.03a）として Error** — chain エッジを暗黙に 0 本として続行しない（構造情報の黙殺禁止）
```

**対象箇所 3:** REQ.03a「検出対象と方法」

**OLD:**
```
必須構造（`[id.chain]` order 等）の欠落・不正
```

**NEW:**
```
必須構造（`[id.chain]` または `[id.chains]` の order 等）の欠落・不正
```

### バージョン bump

M-3 と同一改訂（0.7.0 → 0.7.1）。

---

## 6. 変更履歴行（承認後に各 SPEC へ追記する案）

- **SPEC-LGX-002**: `| 2026-06-13 | 0.4.3 | DD 整合: REQ.05 にコードフェンス内 # の非認識（v3 実測凍結・永続化 ID 安定）と heading_levels 内部属性の v3 踏襲を明文化（M-1/M-2、DD-LGX-003） |`
- **SPEC-LGX-008**: `| 2026-06-13 | 0.7.1 | DD 整合: REQ.01 の移行元 DB 名を feedback.db（.trace-engine/feedback.db）に訂正・engine.db は移行先と明示（M-3、ADR-LGX-015）。REQ.03/03a に [id.chains]+[id.areas] multi-area 変種の受理を明記（M-4、ADR-LGX-018 #15） |`

---

## 7. 判断が必要な事項

1. **M-2 の粒度**: heading_levels を「内部属性として保持してよい」程度に留めるか、いっそ SPEC から
   完全に言及せず DD のみの扱いにするか。本案は「v3 踏襲を明記し公開仕様化は将来」とした（最小コミット）。
2. **M-1 の将来方針**: コードフェンス認識（フェンス内 `#` 除外）を将来の破壊的改訂として予約するか、
   恒久的に非認識とするか。本案は「次版 SPEC 改訂として扱う」と予約のみ。

---

## 付録 A: A-1（DD 修正で解決済・SPEC 変更不要）

**項目:** subnode 整列キー（SUPP S2-21）。SPEC-LGX-003.REQ.11 は subnode 粒度で「アンカー出現順
（ドキュメント物理位置順）」を規定。DD-LGX-002/004 は当初 v3 の anchor バイト辞書順を採用していた。

**裁定（2026-06-13）:** **SPEC 準拠（出現順）を正準とする**。理由:
1. 出現順（ドキュメント読み順）の方が消費側（AI）の文脈読解に資する（v3 の見出しテキスト辞書順は
   セクションをアルファベット順に散らす）。
2. 出力の整列順は **LGX-COMPAT-001 の凍結対象外**（同契約は CLI 引数・終了コード・MCP 3 ツールのみ凍結）。
3. サブノード ID は heading_path ハッシュ（SPEC-LGX-002.REQ.05）で生成され**整列順に不変**。
4. CACHE-INV-1（決定論）は出現順でも充足。

**反映:** DD-LGX-004 §11/§12（v1.1）で `upstream_sort_rule` の Subnode 分岐を
`anchor-bytes-asc` → `anchor-appearance-order` に訂正。DD-LGX-002 §8 テスト記述も訂正。**SPEC 変更なし**。

---

## 付録 B: B 群（凍結確認・SPEC 変更不要）

DD が v3 を底本に確定したが上流文書に明記がない実装細部。いずれも凍結を確認し SPEC 変更を要しない。

| # | 項目 | 確定挙動 | 確認結果 |
|---|---|---|---|
| B-1 | ResultTooLarge の終了動作（DD-002） | **exit 1 / stderr（人間裁定 2026-06-13 で確定）** | **当初の凍結確認は誤りだった**: DD-002 v1.0 は実際には exit 0/stdout を凍結しており、v3（`render(&result)?` 伝播で exit 1）・SPEC-LGX-003.REQ.13「エラーを返却する」・DD-LGX-004（`ContextError→exit 1`）・LGX-COMPAT-001 §3（終了コード凍結）と矛盾していた。**TS フェーズ**（TS-002/004 が具体的 exit を主張）で顕在化。人間裁定により **exit 1/stderr** へ統一。DD-002 §11 v1.2・TS-002 ケース 3/28 を訂正。DD-004/TS-004 は元から exit 1 で整合済 |
| B-2 | content_hash 末尾正規化（DD-007 §3） | 4 段正規化（BOM 除去→CRLF/CR→LF→NFC→末尾改行 1 個）に正規化 | **v3 は前処理 pass-through（`preprocessor.rs`: `text.to_string()`、正規化なし）**。DD-007 の正規化は v3 継承ではなく legixy 新規改善（GAP-LGX-114、環境非依存 content_hash = SCORE-INV-1）。legixy は自 DB を再生成するため v3 との hash 一致は不要・互換問題なし。「末尾改行 1 個」は冪等で POSIX テキスト慣行に整合。**legixy 設計決定として凍結確認**（v3 一致は対象外と判明、B-2 の「0 vs 1」前提は誤りと訂正） |
| B-3 | model_version 書式（DD-007 §3, GAP-115） | `{model_name}:{onnx_sha256_8hex}:{profile}:{dim}` | SPEC は書式を規定せず GAP-LGX-115 で確定済。実装規約として凍結（SPEC 改訂不要）。**凍結確認** |
| B-4 | custom_edges 継承（DD-009 §2.6） | B 案: v3 同様に転記＋ check 側で孤児検出 | v3 `lx-mig/src/db.rs` が custom_edges を copy する挙動と整合。検出は legixy-check の責務（二段構え）。**凍結確認** |

> B-2 補足: 当初「v3 が末尾改行 0 個か 1 個か」という確認だったが、v3 は正規化を一切行わない
> （pass-through）ことが判明。よって legixy の 4 段正規化は v3 と意図的に異なる改善であり、
> content_hash の値は v3 と一致しない（legixy が自 engine.db を生成するため問題にならない）。
