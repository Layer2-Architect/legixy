# Calibration Protocol — measuring the semantic layer's response

**English** | [日本語](#日本語)

> A reproducible procedure for a third party to measure how legixy's **semantic layer**
> (SemanticSimilarity / Drift) responds to *meaning-preserving* vs *meaning-destroying* edits —
> and to report the result as a GitHub issue.
>
> The semantic layer's response characteristics are still **n = 1** (see
> [README → Limitations](README.md#limitations--non-goals)). This protocol is the scaffold for
> turning that into n > 1. If you run it, **please open an issue with your numbers** (template in §3).

The instrument **reports deviation, not anomaly** — it is not a judge. The point of calibrating is
to learn, on *your* corpus, how far meaning has to move before the instrument notices, and to set
thresholds accordingly.

---

## 0. Prerequisites (the semantic layer is opt-in, OFF by default)

The semantic layer does not run in a default build. You need all three:

1. **An onnx-enabled binary.** The `onnx` Cargo feature is **off by default**.
   - Prebuilt: the GitHub Release binaries are onnx-enabled (see [README → Install](README.md#install)).
   - From source: `cargo build --release -p legixy-cli --features onnx`
2. **The local embedding model.** `install.sh` / `install.ps1` fetch it, or run `bash scripts/fetch-model.sh`.
   Default model: **`paraphrase-multilingual-MiniLM-L12-v2`** (multilingual incl. Japanese, 384-dim, mean pooling).
3. **`[semantic] enabled = true` in config.** Fresh projects (the bootstrap template) ship it **disabled**.
   The thresholds below are the defaults (this repository's own `.trace-engine.toml` uses them):

   ```toml
   [semantic]
   enabled = true
   model = "paraphrase-multilingual-MiniLM-L12-v2"
   similarity_threshold     = 0.4   # linked pair below this  -> SemanticSimilarity Warning
   link_candidate_threshold = 0.7   # unlinked pair above this -> LinkCandidate Info
   drift_threshold          = 0.3   # drift above this        -> Drift Warning
   ```

   Point the binary at the model with `--models-dir <dir>` or `LGX_MODELS_DIR`, or place it at
   `<project-root>/models/paraphrase-multilingual-MiniLM-L12-v2/`.

---

## 1. Building meaning-preserving / meaning-destroying edit pairs

Take one artifact and produce edited variants. A **preserving** edit keeps the requirements and
moves only the wording; a **destroying** edit moves the meaning while keeping the form/length
plausible (the kind of change that passes a diff review). Use at least these three types:

| Type | Kind | What you change | Expectation |
|---|---|---|---|
| **Paraphrase** (control) | preserving | vocabulary/phrasing only; every condition intact | instrument barely moves (high similarity, low drift) |
| **Condition-deletion** | destroying | remove a requirement/condition/branch | meaning moves the most |
| **Numeric-alteration** | destroying | change a number / threshold / unit | **known blind spot** — often barely registers |

### Worked example (real numbers, reproducible)

Parent `SPEC-LGX-001` and child `UC-LGX-001` describe the same login flow (email+password, error on
failure, lock after 5 failures for 15 min), linked by a `chain` edge. After `embed --all` and a
`baseline` snapshot, we edited only `UC-LGX-001`:

| Edit on UC-LGX-001 | drift vs baseline | SPEC↔UC similarity | `check` |
|---|---|---|---|
| — (baseline) | — | 0.897 | clean |
| **A. Paraphrase** (5×/15min kept) | 0.031 | 0.865 | clean |
| **B. Condition-deletion** (drop the lockout rule) | **0.076** | **0.828** | clean |
| **C. Numeric-alteration** (5→50 failures, 15→1 min) | 0.009 | 0.892 | clean |

Two honest findings worth reporting:

- **Condition-deletion moved the instrument most** (highest drift, lowest similarity), paraphrase
  moved it modestly, and **numeric-alteration moved it *least* — the model is nearly blind to
  `5 → 50` / `15 → 1`.** Report this kind of blind spot; it is a property of the instrument, not a bug.
- **None of these subtle one-sentence edits crossed the default thresholds** (similarity stayed
  > 0.4, drift < 0.3), so `check` stayed clean. That is exactly *why* you calibrate (§2): run
  `legixy calibrate` on your real corpus, read the distribution, and choose thresholds that make
  *your* meaningful deviations cross the line.

---

## 2. Recording the instrument's response

All commands below need the onnx build + model (§0). Global flags (`--project-root`, `--models-dir`,
`--json`) go **before** the subcommand. Add `--json` for machine-readable output.

```bash
# one-time: generate embeddings for every node, then freeze a baseline to drift against
legixy --project-root <repo> --models-dir <model-dir> embed --all
legixy --project-root <repo> --models-dir <model-dir> snapshot create --label baseline

# after each edit: re-embed just that node, then read the instrument
legixy --project-root <repo> --models-dir <model-dir> embed --node <ID> --force
legixy --project-root <repo> --models-dir <model-dir> --json drift  <ID> --against snapshot:baseline   # -> {"drift": 1 - cos(now, baseline)}
legixy --project-root <repo> --models-dir <model-dir> --json report                                    # -> {"links":[{"from","to","score","kind"}], ...}
legixy --project-root <repo> --models-dir <model-dir> check                                            # -> "Summary: N error, N warning, N info"

# threshold calibration: histogram of linked-pair similarities (+ a recommended threshold)
legixy --project-root <repo> --models-dir <model-dir> calibrate --recommend --buckets 10
```

What the instrument emits, and the field to record:

- **SemanticSimilarity** — a linked pair whose cosine similarity is **below** `similarity_threshold`
  (0.4) → `check` **Warning**. Read the pair score from `report` (`.links[].score`).
- **LinkCandidate** — an *unlinked* pair **above** `link_candidate_threshold` (0.7) → `check`
  **Info** (a suggested missing link). Read candidates from `report` (`.candidates`).
- **Drift** — for an edited node, `drift = 1 − cos(current, baseline)`; **above** `drift_threshold`
  (0.3) → Warning. Read it from `drift … --against snapshot:<label>` (`.drift`).

### Recording fields (one row per edit)

| Field | Example |
|---|---|
| model | `paraphrase-multilingual-MiniLM-L12-v2` |
| thresholds (sim / link / drift) | `0.4 / 0.7 / 0.3` |
| target ID | `UC-LGX-001` |
| edit type | `condition-deletion` |
| observed score(s) | `drift=0.076`, `SPEC↔UC similarity=0.828` |
| verdict | `Warning` / `Info` / `none` |

---

## 3. Issue report template

Open an issue on this repository and paste the block below (fill in `<…>`). Keep one **Measurements**
row per edit. This is the record the articles ask you to share.

```markdown
### legixy semantic-layer calibration report

**Environment**
- legixy version: <e.g. 0.4.0-alpha4>   (build: `--features onnx`)
- OS / arch: <e.g. Ubuntu 24.04 x86_64>
- model: paraphrase-multilingual-MiniLM-L12-v2
- thresholds: similarity=0.4 / link_candidate=0.7 / drift=0.3   <or your calibrated values>
- corpus: <what you measured on — a few sentences>

**Measurements** (one row per edit)
| target ID | edit type | observed score(s) | verdict (Warning/Info/none) |
|---|---|---|---|
| <ID> | paraphrase (preserve)      | similarity=<x>, drift=<y> | <none> |
| <ID> | condition-deletion (break) | similarity=<x>, drift=<y> | <Warning/none> |
| <ID> | numeric-alteration (break) | similarity=<x>, drift=<y> | <Warning/none> |

**calibrate output** (optional)
- recommended threshold: <from `legixy calibrate --recommend`>
- notes: <blind spots, surprises — e.g. "numeric changes barely registered">
```

---

### This document's status

`CALIBRATION.md` is reader-facing protocol documentation (a peer of `README.md` and the manuals), so
it is **not** registered as a node in `docs/traceability/graph.toml` — consistent with this repo's
convention that meta-docs are not chain artifacts. A reader who follows it produces a *record* that
is analogous to a **VAL** artifact (`docs/validation/`, e.g. `VAL-LGX-001`); the protocol is the
how-to, the VAL is the result.

---
---

# 日本語

[English](#calibration-protocol--measuring-the-semantic-layers-response) | **日本語**

> legixy の**意味層**（SemanticSimilarity / Drift）が *意味保存編集* と *意味破壊編集* に
> どう応答するかを、第三者が**再現可能な手順**で測り、結果を GitHub issue として報告するための手順書。
>
> 意味層の応答特性はまだ **n = 1**（[README → 限界](README.md#限界非目標)）。本プロトコルはそれを
> n > 1 にするための足場です。実施したら **数値を添えて issue を立ててください**（テンプレは §3）。

計器は**逸脱を報告するのであって異常を断じない** ― 審判ではありません。校正の目的は、*あなたの*
コーパス上で「意味がどれだけ動けば計器が気づくか」を知り、それに合わせて閾値を決めることです。

---

## 0. 前提（意味層は既定オフのオプトイン機能）

意味層は既定ビルドでは動きません。次の3つすべてが必要です。

1. **onnx 有効ビルド。** Cargo の `onnx` フィーチャは**既定オフ**。
   - ビルド済み: GitHub Release のバイナリは onnx 有効（[README → Install](README.md#日本語)）。
   - ソースから: `cargo build --release -p legixy-cli --features onnx`
2. **ローカル埋め込みモデル。** `install.sh` / `install.ps1` が取得、または `bash scripts/fetch-model.sh`。
   既定モデル **`paraphrase-multilingual-MiniLM-L12-v2`**（日本語含む多言語・384次元・mean pooling）。
3. **設定の `[semantic] enabled = true`。** 新規プロジェクト（bootstrap テンプレート）は**無効**で配布。
   下記の閾値が既定値（本リポジトリの `.trace-engine.toml` もこの値）:

   ```toml
   [semantic]
   enabled = true
   model = "paraphrase-multilingual-MiniLM-L12-v2"
   similarity_threshold     = 0.4   # リンク間がこれ未満  -> SemanticSimilarity Warning
   link_candidate_threshold = 0.7   # 非リンクがこれ超過  -> LinkCandidate Info
   drift_threshold          = 0.3   # drift がこれ超過    -> Drift Warning
   ```

   モデルは `--models-dir <dir>` か `LGX_MODELS_DIR` で指す、または
   `<project-root>/models/paraphrase-multilingual-MiniLM-L12-v2/` に配置。

---

## 1. 意味保存編集 / 意味破壊編集のペアの作り方

1つの成果物から編集後バリアントを作ります。**保存**編集は要件を保ったまま語彙だけ動かし、**破壊**
編集は形式・分量はもっともらしく保ったまま意味だけを動かします（diff レビューを通過する類の変更）。
最低限この3類型を使います。

| 類型 | 種別 | 変える対象 | 期待 |
|---|---|---|---|
| **言い換え型**（対照） | 保存 | 語彙・言い回しのみ。条件はすべて温存 | 計器はほぼ動かない（高類似度・低 drift） |
| **条件削除型** | 破壊 | 要件・条件・分岐を削除 | 意味が最も動く |
| **数値改変型** | 破壊 | 数値・閾値・単位を改変 | **既知の盲点** ― ほとんど反応しないことが多い |

### 実例（実測値・再現可能）

親 `SPEC-LGX-001` と子 `UC-LGX-001` は同一のログインフロー（メール+パスワード、失敗時エラー、5回
失敗で15分ロック）を記述し、`chain` エッジでリンク。`embed --all` と `baseline` snapshot の後、
`UC-LGX-001` だけを編集:

| UC-LGX-001 への編集 | drift(対 baseline) | SPEC↔UC 類似度 | `check` |
|---|---|---|---|
| —（ベースライン） | — | 0.897 | clean |
| **A. 言い換え型**（5回/15分は温存） | 0.031 | 0.865 | clean |
| **B. 条件削除型**（ロック規則を削除） | **0.076** | **0.828** | clean |
| **C. 数値改変型**（5→50回・15→1分） | 0.009 | 0.892 | clean |

報告に値する正直な知見が2つ:

- **条件削除型が計器を最も動かし**（最大 drift・最小類似度）、言い換え型は中程度、**数値改変型は
  *最も動かさない* ― モデルは `5→50` / `15→1` をほぼ捉えない。** こうした盲点こそ報告してください。
  これは計器の性質であってバグではありません。
- **これらの微細な一文編集はどれも既定閾値を越えませんでした**（類似度 > 0.4・drift < 0.3 のまま）
  ので `check` は clean。これが校正（§2）の必要な理由です ― 実コーパスで `legixy calibrate` を回し、
  分布を見て、*あなたにとって*有意な逸脱が線を越える閾値を選びます。

---

## 2. 計器応答の記録様式

以下は onnx ビルド + モデル（§0）が必要。グローバルフラグ（`--project-root` / `--models-dir` /
`--json`）はサブコマンドの**前**。機械可読出力には `--json`。

```bash
# 一度だけ: 全ノードを embed → 比較基点の baseline を凍結
legixy --project-root <repo> --models-dir <model-dir> embed --all
legixy --project-root <repo> --models-dir <model-dir> snapshot create --label baseline

# 各編集の後: そのノードだけ再 embed → 計器を読む
legixy --project-root <repo> --models-dir <model-dir> embed --node <ID> --force
legixy --project-root <repo> --models-dir <model-dir> --json drift  <ID> --against snapshot:baseline   # -> {"drift": 1 - cos(現在, baseline)}
legixy --project-root <repo> --models-dir <model-dir> --json report                                    # -> {"links":[{"from","to","score","kind"}], ...}
legixy --project-root <repo> --models-dir <model-dir> check                                            # -> "Summary: N error, N warning, N info"

# 閾値校正: リンク間類似度のヒストグラム（+ 推奨閾値）
legixy --project-root <repo> --models-dir <model-dir> calibrate --recommend --buckets 10
```

計器が出すものと、記録する欄:

- **SemanticSimilarity** ― リンク間のコサイン類似度が `similarity_threshold`(0.4) **未満** →
  `check` **Warning**。ペアのスコアは `report` の `.links[].score`。
- **LinkCandidate** ― *非リンク*のペアが `link_candidate_threshold`(0.7) **超過** → `check`
  **Info**（リンク欠落の示唆）。`report` の `.candidates`。
- **Drift** ― 編集ノードについて `drift = 1 − cos(現在, baseline)`。`drift_threshold`(0.3) **超過**
  → Warning。`drift … --against snapshot:<label>` の `.drift`。

### 記録項目（編集1件 = 1行）

| 項目 | 例 |
|---|---|
| モデル名 | `paraphrase-multilingual-MiniLM-L12-v2` |
| 閾値（sim / link / drift） | `0.4 / 0.7 / 0.3` |
| 対象 ID | `UC-LGX-001` |
| 編集タイプ | `条件削除型` |
| 観測スコア | `drift=0.076`、`SPEC↔UC 類似度=0.828` |
| 判定 | `Warning` / `Info` / `なし` |

---

## 3. issue 報告テンプレート

本リポジトリに issue を立て、下のブロックを貼って `<…>` を埋めてください（**Measurements** は
編集1件 = 1行）。これが記事の促す「共有する記録」です。

上の英語版 §3 のテンプレートをそのまま使えます（言語非依存）。日本語で記す場合の対訳:

```markdown
### legixy 意味層 校正レポート

**環境**
- legixy バージョン: <例 0.4.0-alpha4>   (build: `--features onnx`)
- OS / arch: <例 Ubuntu 24.04 x86_64>
- モデル: paraphrase-multilingual-MiniLM-L12-v2
- 閾値: similarity=0.4 / link_candidate=0.7 / drift=0.3   <または校正後の値>
- コーパス: <何を測ったか ― 数文で>

**Measurements**（編集1件 = 1行）
| 対象 ID | 編集タイプ | 観測スコア | 判定(Warning/Info/なし) |
|---|---|---|---|
| <ID> | 言い換え型（保存） | 類似度=<x>, drift=<y> | <なし> |
| <ID> | 条件削除型（破壊） | 類似度=<x>, drift=<y> | <Warning/なし> |
| <ID> | 数値改変型（破壊） | 類似度=<x>, drift=<y> | <Warning/なし> |

**calibrate 出力**（任意）
- 推奨閾値: <`legixy calibrate --recommend` の値>
- 所見: <盲点・意外な点 ― 例「数値改変はほぼ無反応」>
```

---

### 本ドキュメントの位置づけ

`CALIBRATION.md` は読者向けの手順書（`README.md` やマニュアルの同位）であり、リポジトリの規約
（meta-doc はチェーン成果物としない）に従い `docs/traceability/graph.toml` には**登録しません**。
本書に沿って読者が作る*記録*は **VAL** 成果物（`docs/validation/`、例 `VAL-LGX-001`）に相当します
（本書は手順、VAL は結果）。
