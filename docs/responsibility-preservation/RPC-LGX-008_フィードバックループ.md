Document ID: RPC-LGX-008

# RPC-LGX-008: フィードバックループ chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-008
**対象 RBA**: RBA-LGX-008
**対象 SEQA**: SEQA-LGX-008
**対象 RBD**: RBD-LGX-008
**対象 SEQD**: SEQD-LGX-008
**検査深度**: フル（§14.2: UC-LGX-008 は承認・却下状態遷移核・FB-INV 不変条件群を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

---

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

RBA-LGX-008 / SEQA-LGX-008 から抽出。各 AR を UC-LGX-008 ステップに紐づける。

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | フィードバックコマンド受付窓口 | 人間からの自動 Observation 生成要求を受け取る | Observation 生成 1（`feedback` コマンド） |
| AR-002 | RBA | Boundary | 観測コマンド受付窓口（Agent） | Claude Code からの気づき記録要求（MCP 経由）を受け取る | Observation 生成 2（`observe` コマンド） |
| AR-003 | RBA | Boundary | 分析コマンド受付窓口 | 人間からの Proposal 生成要求を受け取る | Proposal 生成 1（`analyze` コマンド） |
| AR-004 | RBA | Boundary | Proposal 一覧表示窓口 | 人間から未承認 Proposal 一覧表示要求を受け取り、一覧をアクターへ返す | 承認・却下 1（`proposals` コマンド） |
| AR-005 | RBA | Boundary | 承認・却下コマンド受付窓口 | 人間からの承認・却下操作を受け取る | 承認・却下 2・3（`approve` / `reject`） |
| AR-006 | RBA | Boundary | 検証結果供給境界 | check 実行結果（カテゴリ所見）を供給する | Observation 生成 1（`feedback` コマンド） |
| AR-007 | RBA | Boundary | 観測記録出力窓口 | 観測記録の完了通知をアクターへ返す | Observation 生成 2（`observe` コマンド） |
| AR-008 | RBA | Control | Observation 生成統括処理 | フィードバックコマンドを受け取り、検証結果から各カテゴリの Observation を生成し重複排除して永続化する | Observation 生成 1・3（`feedback` / 重複チェック） |
| AR-009 | RBA | Control | 手動 Observation 記録処理 | Agent からの観測記録要求を受け取り、Observation として永続化する | Observation 生成 2（`observe`） |
| AR-010 | RBA | Control | Observation 重複排除処理 | (category, 正準化 related_ids) 複合キーで pending / analyzing 状態の Observation 重複を検査する | Observation 生成 3（重複チェック FB-INV-1） |
| AR-011 | RBA | Control | Proposal 生成処理 | pending の Observation を Pessimistic Claim で取り込み、カテゴリ別変換規則で Proposal を生成し、semantic_key 重複排除・失敗時差し戻しを行う | Proposal 生成 1–4（`analyze` / Pessimistic Claim / 変換 / FB-INV-5） |
| AR-012 | RBA | Control | Proposal 一覧取得処理 | 指定ステータスの Proposal を一覧として取得する | 承認・却下 1（`proposals`） |
| AR-013 | RBA | Control | Proposal 承認処理 | pending 状態の Proposal を CAS で原子的に approved へ遷移させる | 承認・却下 2（`approve` FB-INV-2） |
| AR-014 | RBA | Control | Proposal 却下処理 | pending 状態の Proposal を CAS で原子的に rejected へ遷移させる（理由必須） | 承認・却下 3（`reject`） |
| AR-015 | RBA | Entity | Observation | 気づき・自動検知の記録（category・message・関連ノード ID・status）。pending / analyzing / resolved / skipped の状態を持つ | Observation 生成 1–3 / Proposal 生成 1–2 |
| AR-016 | RBA | Entity | Proposal | 承認待ちの改善提案（kind・対象ノード・semantic_key・status）。pending / approved / rejected の状態を持つ | Proposal 生成 4 / 承認・却下 1–3 |
| AR-017 | RBA | Entity | 検証所見 | 検証結果供給境界から供給される check 結果の個別所見 | Observation 生成 1（`feedback`） / 代替 1a |

全 AR が UC ステップに紐づく（UC ステップに紐づかない AR なし → 構造翻訳が情報を加えていない、§9 分解(b) 候補なし）。SEQA-LGX-008 の時系列メッセージは上記 AR の実行順展開であり、新規 AR を生まない。

---

## 2. Concrete Responsibilities

RBD-LGX-008 / SEQD-LGX-008 から抽出。

| CR-ID | Source | Class | Operation(s) | Responsibility | SEQD Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | フィードバックコマンド受付窓口 | Observation 生成を受け付ける | アクター境界でフィードバック要求を受理 | 人間→Bfb |
| CR-002 | RBD/SEQD | 観測コマンド受付窓口（Agent） | 気づきの記録を受け付ける | Claude Code 境界で観測記録要求を受理 | ClaudeCode→Bobs |
| CR-003 | RBD/SEQD | 分析コマンド受付窓口 | Proposal 生成を受け付ける | アクター境界で分析要求を受理 | 人間→Ban |
| CR-004 | RBD/SEQD | Proposal 一覧表示窓口 | Proposal 一覧を要求する | アクター境界で Proposal 一覧要求を受理し返却 | 人間→Bpl, Bpl→人間 |
| CR-005 | RBD/SEQD | 承認・却下コマンド受付窓口 | 承認を受け付ける / 却下を受け付ける | アクター境界で承認・却下要求を受理 | 人間→Bap |
| CR-006 | RBD/SEQD | 検証結果供給境界 | 検証結果を取得する | 外部システム境界で check 結果を供給 | Cgen→Bcheck |
| CR-007 | RBD/SEQD | 観測記録出力窓口 | 記録完了を通知する | 出力境界で完了通知を返却 | Cman→Boutobs |
| CR-008 | RBD/SEQD | Observation 生成統括処理 | Observation 生成を統括する / 検証所見をカテゴリ別に分類する / カテゴリ不在を検知する | フィードバックフローを統括し、検証所見分類・カテゴリ不在検知・Observation 永続化を協調する | Bfb→Cgen, Cgen→Efind, Cgen→Cdedup, Cgen→Eobs |
| CR-009 | RBD/SEQD | 手動 Observation 記録処理 | 手動 Observation を記録する | observe フローを処理し、重複排除後に Observation を永続化して完了通知を出力する | Bobs→Cman |
| CR-010 | RBD/SEQD | Observation 重複排除処理 | 重複を検査する | (カテゴリ種別, 正準化済み関連ノード識別子) で Observation の重複を照合する | Cgen/Cman→Cdedup |
| CR-011 | RBD/SEQD | Proposal 生成処理 | Proposal 生成を処理する / カテゴリ別変換規則を適用する / Observation を差し戻す | analyze フローを処理し、Pessimistic Claim・カテゴリ別変換・semantic_key 重複排除・失敗時差し戻しを協調する | Ban→Cprop |
| CR-012 | RBD/SEQD | Proposal 一覧取得処理 | Proposal 一覧を取得する | 指定ステータスの Proposal を照合して一覧を返す | Bpl→Clist |
| CR-013 | RBD/SEQD | Proposal 承認処理 | Proposal を承認する | CAS で pending→approved へ原子的に遷移させる | Bap→Capr |
| CR-014 | RBD/SEQD | Proposal 却下処理 | Proposal を却下する | CAS で pending→rejected へ原子的に遷移させる（理由を記録） | Bap→Crej |
| CR-015 | RBD/SEQD | Observation | ステータスを遷移させる / 正準化済み関連ノード識別子を取り出す | Observation 自身のステータス状態遷移・正準化識別子の取り出し | Cgen→Eobs, Cdedup→Eobs 等 |
| CR-016 | RBD/SEQD | Proposal | ステータスを原子的に遷移させる / 意味キーを取り出す | Proposal 自身の原子的状態遷移・意味キー取り出し | Capr/Crej→Eprop 等 |
| CR-017 | RBD/SEQD | 検証所見 | （属性保持、操作なし） | カテゴリ・対象識別子・深刻度・メッセージの保持（非永続、実行時データ） | Cgen→Efind |

---

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Per-AR Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・同一境界操作 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Boundary・同一境界操作 | GREEN |
| AR-003 | CR-003 | preserved | 同一 Boundary・同一境界操作 | GREEN |
| AR-004 | CR-004 | preserved | 同一 Boundary・境界受理と返却を保持 | GREEN |
| AR-005 | CR-005 | preserved | 同一 Boundary・承認・却下の 2 操作に分化（RBD §1 で「承認を受け付ける / 却下を受け付ける」として 1:1 識別済み） | GREEN |
| AR-006 | CR-006 | preserved | 同一 Boundary・同一境界操作 | GREEN |
| AR-007 | CR-007 | preserved | 同一 Boundary・同一境界操作 | GREEN |
| AR-008 | CR-008 | preserved | 同一 Control。統括・分類・不在検知の 3 操作として識別、UC ステップ全対応 | GREEN |
| AR-009 | CR-009 | preserved | 同一 Control・同一責務 | GREEN |
| AR-010 | CR-010 | preserved | 同一 Control・同一照合責務 | GREEN |
| AR-011 | CR-011 | **shifted（SEQD §1c / 例外 B で操作誤帰属）** | RBD では `カテゴリ別変換規則を適用する` は Proposal 生成処理（Control）の操作として定義。SEQD §1c では `Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)` と記述されており、Observation Entity（Eobs）に対してメッセージを送る形になっている。SEQD §6 Behavior Allocation 表は Cprop 帰属と正しく記載しており、シーケンス図と Behavior Allocation 表の間に矛盾がある。同様の誤帰属が例外 B にも存在する。 | RED |
| AR-012 | CR-012 | preserved | 同一 Control・同一責務。SEQD §1d で `Clist->>Eprop: 意味キーを取り出す()` が使われているが、これは Proposal の照合に意味キーを読み出す操作であり、AR の「指定ステータスで一覧取得」の範囲内 | GREEN |
| AR-013 | CR-013 | preserved | 同一 Control・CAS 承認の責務を保持 | GREEN |
| AR-014 | CR-014 | preserved | 同一 Control・CAS 却下・理由記録の責務を保持 | GREEN |
| AR-015 | CR-015 | preserved | 同一 Entity。ステータス遷移・正準化識別子取り出しを自身操作として保持 | GREEN |
| AR-016 | CR-016 | preserved | 同一 Entity。原子的状態遷移・意味キー取り出しを自身操作として保持 | GREEN |
| AR-017 | CR-017 | preserved | 同一 Entity（非永続、実行時データ）。属性を保持 | GREEN |

### Shifted 詳細（AR-011 / CR-011）

**発生箇所 1: SEQD §1c（Proposal 生成 基本フロー）**

```
Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)
Eobs-->>Cprop: Proposal 種別
```

`カテゴリ別変換規則を適用する` は RBD-LGX-008 で `Proposal 生成処理（Control）` の操作として定義されている。SEQD のシーケンス図では送信先が `Eobs`（Observation Entity）になっており、Entity が Control の変換ロジックを担わされている形になっている。Entity に変換規則ロジックが帰属されると Control leakage（逆向き: Entity が制御責務を担う Entity overreach）に相当する。

**発生箇所 2: SEQD 例外 B（終端スキップ）**

```
Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)
Eobs-->>Cprop: Proposal 種別(変換不能)
```

同一の誤帰属が例外フローでも再現している。

**SEQD §6 Behavior Allocation との矛盾**

SEQD §6 では以下のように正しく記載されている:
> `Proposal 生成を処理する / カテゴリ別変換規則を適用する / Observation を差し戻す` | Proposal 生成処理 | Control（協調）

シーケンス図（§1c / 例外 B）と Behavior Allocation 表（§6）が矛盾しており、SEQD 内部で不整合が発生している。

**原因の所在**: 具体側（SEQD）の生成誤り。UC-LGX-008 の「カテゴリ別の変換」ステップは Proposal 生成処理（Control）の責務として RBA に正しく錨着されており、UC は正しい。

---

## 4. Role Fitness Check（§5.2）

### Boundary

**Finding**: 7 つの Boundary クラス（フィードバック/観測/分析/Proposal一覧/承認却下の各受付窓口・検証結果供給境界・観測記録出力窓口）はいずれも境界操作のみを保持。データ処理・状態遷移ロジックへの overreach なし。
**Severity**: なし / **Required action**: なし

### Control

**Finding**: 各 Control（Observation 生成統括処理 / 手動 Observation 記録処理 / Observation 重複排除処理 / Proposal 生成処理 / Proposal 一覧取得処理 / Proposal 承認処理 / Proposal 却下処理）は協調・調停に留まり、データ保持なし（全クラスに「属性なし、協調ロジック保持」と明記）。Service blob 化なし（Proposal 生成処理は generate/transform/rollback の 3 操作を持つが、いずれも analyze フローの協調責務の範囲内）。Control leakage なし。

**ただし**: SEQD §1c および例外 B にて `Proposal生成処理（Cprop）` の操作 `カテゴリ別変換規則を適用する` が Observation Entity（Eobs）への呼び出しとして記述されており、これは実質的に Control 操作が Entity に shifted している（§3 AR-011 参照）。Behavior Allocation 表（SEQD §6）では正しく Cprop 帰属と記載されており、シーケンス図に誤りがある。
**Severity**: Major（シーケンス図と Behavior Allocation 表の矛盾）/ **原因の所在**: 具体側 SEQD / **Required action**: REQUEST_CHANGES

### Entity

**Finding**: Observation と Proposal は自身のデータ操作のみを保持（ステータス遷移・識別子取り出し・意味キー取り出し）。Entity anemia なし（各 Entity が自身の状態管理操作を持つ）。Entity overreach なし（ただし SEQD §1c / 例外 B のシーケンス図では制御責務が誤って Eobs に帰属されているが、これは SEQD の記述誤りであり RBD の Entity 定義は正しい）。
**Severity**: なし（RBD 定義上は問題なし）/ **Required action**: なし（SEQD 修正で解消）

---

## 5. Sequential Execution Check（§5.3）

### Basic Flow

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Observation 生成 1（feedback）| 人間→フィードバックコマンド受付窓口→Observation 生成統括処理 | Observation 生成を受け付ける→Observation 生成を統括する | Yes | |
| Observation 生成 1（検証結果取得）| 統括処理→検証結果供給境界→検証所見カテゴリ分類 | 検証結果を取得する→検証所見をカテゴリ別に分類する | Yes | |
| Observation 生成 3（重複チェック FB-INV-1）| 統括処理→重複排除処理→Observation 照合→Observation 永続化 | 重複を検査する→（照合）→Observation 永続化 | **Partial** | SEQD §1a で Cdedup→Eobs の操作が `ステータスを遷移させる(照合ステータス種別)` と記述。重複チェックは読み取り操作であり、状態遷移操作（書き込み）を使って照合するのは意味的に不自然。例外 A では `正準化済み関連ノード識別子を取り出す()` が使われており、同一 Cdedup→Eobs 経路で 2 種の操作が使い分けられている。ただし Cdedup の責務（重複を検査する）自体は保存されているため、この差異は SEQD 内の実装選択の一貫性問題として記録するが、AR レベルの保存失敗ではない。 |
| Observation 生成 2（observe）| Claude Code→観測コマンド受付窓口→手動 Observation 記録処理→重複排除→Observation→観測記録出力窓口 | 気づきの記録を受け付ける→手動 Observation を記録する→重複を検査する→記録完了を通知する | Yes | |
| Proposal 生成（analyze）| 人間→分析コマンド受付窓口→Proposal 生成処理→Observation（Pessimistic Claim）→Proposal（semantic_key チェック）→Proposal 永続化→Observation proposed | Proposal 生成を受け付ける→Proposal 生成を処理する→ステータスを遷移させる(分析中)→意味キーを取り出す→（カテゴリ変換：問題あり、下記参照）→Proposal 永続化→ステータスを遷移させる(提案済み) | **Partial** | SEQD §1c で `Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)` が Entity への誤帰属（§3 AR-011 参照）。UC ステップ「カテゴリ別の変換」自体は SEQD に存在するが、操作の送信先が誤っている。フロー全体の実行可能性は論理的には成立するが、Behavior Allocation が正しくない。 |
| 承認・却下 1（proposals）| 人間→Proposal 一覧表示窓口→Proposal 一覧取得処理→Proposal→Proposal 一覧返却 | Proposal 一覧を要求する→Proposal 一覧を取得する→意味キーを取り出す→Proposal のコレクション | Yes | |
| 承認・却下 2（approve）| 人間→承認・却下コマンド受付窓口→Proposal 承認処理→Proposal（CAS）| 承認を受け付ける→Proposal を承認する→ステータスを原子的に遷移させる(保留→承認済み) | Yes | |
| 承認・却下 3（reject）| 人間→承認・却下コマンド受付窓口→Proposal 却下処理→Proposal（CAS 理由付き）| 却下を受け付ける→Proposal を却下する→ステータスを原子的に遷移させる(保留→却下済み) | Yes | |

### Alternative Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 1a（カテゴリ不在 → Observation 生成なし）| 統括処理→検証結果取得→検証所見のカテゴリ不在検知→スキップ | 検証結果を取得する→検証所見をカテゴリ別に分類する→カテゴリ不在を検知する→生成件数ゼロ返却 | Yes | SEQD §2 代替 1a は SEQA §3 代替 1a と一致 |
| 代替 2a（analyze 処理中失敗 → Observation を保留に差し戻し）| Proposal 生成処理→Observation 差し戻し（pending） | Proposal 生成を処理する→ステータスを遷移させる(分析中)→（失敗発生）→ステータスを遷移させる(保留) | Yes | claim release が SEQD §2 代替 2a に明記 |

### Exception Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外 A（重複検出 FB-INV-1 → Observation 生成なし）| 重複排除処理→Observation 照合→重複あり→生成抑制 | 重複を検査する→正準化済み関連ノード識別子を取り出す→重複あり→生成抑制 | Yes | |
| 例外 B（変換不能カテゴリ → skipped 終端）| Proposal 生成処理→Observation（変換不能）→skipped 遷移 | ステータスを遷移させる(分析中)→カテゴリ別変換規則を適用する→（変換不能）→ステータスを遷移させる(スキップ終端) | **Partial** | `Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)` が 基本フロー §1c と同じ Entity 誤帰属を持つ |
| 例外 C（semantic_key 重複排除 FB-INV-5 → Proposal 生成しない）| Proposal 生成処理→Proposal（同一 semantic_key）→Proposal 生成抑制→Observation proposed へ | 意味キーを取り出す→同一意味キー存在→Proposal 生成なし→ステータスを遷移させる(提案済み) | Yes | |

---

## 6. Mismatches

### Lost Responsibilities
None

### Invented Responsibilities
None（具体側に抽象側根拠のない責務なし）

### Shifted Responsibilities

**SH-001**: `カテゴリ別変換規則を適用する` 操作の SEQD での誤帰属

- **AR**: AR-011（Proposal 生成処理 / Control / `カテゴリ別変換規則を適用する` は Control 操作）
- **CR**: CR-011（Proposal 生成処理）/ CR-015（Observation）
- **詳細**: SEQD §1c および例外 B において、`Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)` と記述されており、カテゴリ変換規則の適用操作が Observation Entity（Eobs）に送信されている。RBD-LGX-008 §1 では `カテゴリ別変換規則を適用する` は `Proposal 生成処理（Control）` の操作として定義されており、SEQD §6 Behavior Allocation 表でも Cprop 帰属と正しく記載されている。シーケンス図と Behavior Allocation 表の間で Behavior Allocation が矛盾している。
- **Severity**: Major（シーケンス図上の Control 操作が Entity に shifted、SEQD 内部矛盾）
- **原因の所在**: 具体側（SEQD）生成誤り。UC は正しい。
- **影響箇所**: SEQD §1c（基本フロー Proposal 生成）、SEQD 例外 B（終端スキップ）

### Mutated Responsibilities
None

### Ambiguous Mappings
None

---

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---:|---:|
| Total abstract responsibilities | 17 |
| Preserved | 16 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 1 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 94.1%（16/17） |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 17 |
| Invention rate（監視用） | 0% |

---

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0（SH-001 は SEQD-LGX-008 修正で解消、review-fix loop 2026-06-13）
- [x] ambiguous = 0
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし（RBD 定義上）
- [x] UC 基本/代替/例外フローが具体側で実行可能（論理的フローは成立するが SEQD シーケンス図の Behavior Allocation が不正確）

絶対条件ゲート: **GREEN**（SH-001 は SEQD-LGX-008 修正で解消済、review-fix loop 2026-06-13。下記 §10 参照）

---

## 9. Required Changes

### RC-001（Major）— SEQD §1c シーケンス図の操作帰属修正

**対象**: SEQD-LGX-008 §1c（基本フロー 1c. Proposal 生成）

**現在の記述（誤り）**:
```
Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)
Eobs-->>Cprop: Proposal 種別
```

**修正後**（Cprop への自己呼び出し、または操作名の送信先を Cprop に変更）:
```
Cprop->>Cprop: カテゴリ別変換規則を適用する(Observation)
Cprop-->>Cprop: Proposal 種別
```

あるいは、変換結果を Observation から取り出す形であれば Observation の操作として別途 RBD に定義する必要があるが、UC の意図（Proposal 生成処理が変換規則を適用する）からは Cprop への自己呼び出しが正しい。

### RC-002（Major）— SEQD 例外 B シーケンス図の操作帰属修正

**対象**: SEQD-LGX-008 例外 B（終端スキップ）

**現在の記述（誤り）**:
```
Cprop->>Eobs: カテゴリ別変換規則を適用する(Observation)
Eobs-->>Cprop: Proposal 種別(変換不能)
```

**修正後**（RC-001 と同様）:
```
Cprop->>Cprop: カテゴリ別変換規則を適用する(Observation)
Cprop-->>Cprop: Proposal 種別(変換不能)
```

**修正の根拠**: RBD-LGX-008 §1（Control クラス定義）および SEQD-LGX-008 §6（Behavior Allocation 表）がともに `カテゴリ別変換規則を適用する` を `Proposal 生成処理` の操作として正しく記載している。SEQD シーケンス図のみが誤っており、具体側 AI 生成の局所的誤りである。UC-LGX-008・RBA-LGX-008 の修正は不要。

---

## 10. Verdict（§9 規律）

初回検査で shifted 1 件（SH-001）を検出した。原因は SEQD シーケンス図の生成誤りで UC-LGX-008・RBA-LGX-008 は正しい（§9.1 分解 (a) 具体側逸脱）。

**review-fix loop（2026-06-13）で解消**: §9.2 規律に従い SEQD-LGX-008 §1c・例外 B のシーケンス図で `カテゴリ別変換規則を適用する` の送信先を Observation Entity から Proposal 生成処理（自己呼び出し）へ修正。Behavior Allocation 表と一致。再検査で全 17 AR preserved、shifted=0、絶対条件ゲート GREEN。UC への遡及なし。

<!-- VERDICT:APPROVE -->
