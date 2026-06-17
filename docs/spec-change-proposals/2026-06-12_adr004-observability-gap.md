# SPEC 修正案 — ADR-LGX-004 可観測性ギャップの解消（2026-06-12）

> **ステータス: 提案中（未承認）**
> 本書は ADR-LGX-004「監査ログ書込失敗時は可用性を優先する」の残存リスク「欠落の見逃し」に対し、
> 検出路を強化するための SPEC 改訂案である。
> **SPEC 変更は人間承認が必要。** 本書の提案はすべて承認待ち。

---

## 0. 問題の所在

ADR-LGX-004 の残存リスク欄は以下を認めている：

> 欠落の見逃し → stderr Warning による検出可能性を SPEC 要件として固定。
> **恒常的な書込失敗は運用で検知する。**

この「運用で検知する」が、現行 SPEC では **具体的な到達経路が保証されていない**。

### 問題 A（主因）：exit 0 時の stderr が MCP 経由で Agent に届く保証がない

compile_context が context_log 書込失敗でも exit 0 を返す場合、
SPEC-LGX-009 REQ.03 は「CLI stderr をエラー応答に含める際は忠実転送する」と規定しているが、
これは **非ゼロ終了コード（isError: true）限定の文脈**で書かれている。

exit 0 の成功応答に対して stderr（Warning）を転送する規定が存在しないため、
MCP サーバが stderr を破棄した場合に SPEC 違反が生じない。
これにより「stderrで検知できる」というADR-004の前提が実装依存の状態にある。

### 問題 B（従因）：context_log 欠落が analyze の完全性に影響するが SPEC に明記がない

SPEC-LGX-007 REQ.03（`legixy analyze`）は observations と context_log から proposal を生成するが、
context_log に書込失敗による欠落がある場合の分析品質への影響が記述されていない。
analyze を実行した人間は「この分析は完全なデータに基づいているか」を判断できない。

---

## 1. 修正一覧

| # | 対象 SPEC | 変更節 | 変更の性質 | バージョン bump |
|---|-----------|--------|------------|----------------|
| M-1 | SPEC-LGX-009 | REQ.03 | 記述追加 | 0.6.0 → 0.6.1 |
| M-2 | SPEC-LGX-003 | REQ.19 | 記述変更（強化） | 0.7.0 → 0.7.1 |
| M-3 | SPEC-LGX-007 | REQ.03 | 記述追加 | 0.5.0 → 0.5.1 |

---

## 2. 修正 M-1: SPEC-LGX-009 REQ.03 — exit 0 時の非空 stderr 転送を明示

### 問題の根拠

SPEC-LGX-009 REQ.03「ロギングとマスキング」節の現行テキスト：

> CLI stderr をエラー応答に含める際は**マスキングせず忠実転送**する（MCP-INV-2 優先）

「エラー応答に含める際は」という限定句により、exit 0 時の stderr 転送が規定の対象外となっている。
REQ.07 も「非ゼロ終了コードは MCP エラー応答として転送する」に限定されており、exit 0 stderr の
転送は現行 SPEC のどこにも規定されていない。

### 修正テキスト

**対象箇所:** SPEC-LGX-009 §3 REQ.03「ロギングとマスキング」節

**OLD（第1〜2 bullet）:**
```
- CLI stderr をエラー応答に含める際は**マスキングせず忠実転送**する（MCP-INV-2 優先 — MCP 層での本文改変はむしろ不変条件違反となる）
- NFR SEC.05 のクレデンシャルマスキング義務は **Rust CLI 側経路（Contextual Retrieval の API キー等）の責務**であり、MCP 転送層の責務ではないことを明確化する
```

**NEW（第1〜3 bullet）:**
```
- CLI stderr をエラー応答に含める際は**マスキングせず忠実転送**する（MCP-INV-2 優先 — MCP 層での本文改変はむしろ不変条件違反となる）
- NFR SEC.05 のクレデンシャルマスキング義務は **Rust CLI 側経路（Contextual Retrieval の API キー等）の責務**であり、MCP 転送層の責務ではないことを明確化する
- **exit 0 時の非空 stderr 転送（ADR-LGX-004 可観測性保証）:** Rust CLI が exit 0 で終了しても stderr が非空の場合（例: context_log 書込失敗 Warning — SPEC-LGX-003.REQ.19）、MCP サーバは成功応答の `_meta["legixy/warnings"]` フィールドに stderr 本文を格納して Agent に転送する。これにより可用性優先の副作用（書込失敗 Warning）が MCP 経由でも Agent に到達することを SPEC として保証する。`isError: false` の応答への `_meta` 追加は MCP-INV-2（忠実転送）と整合する（REQ.13 の `_meta["anthropic/maxResultSizeChars"]` と同一拡張経路）。
```

### 変更後の検証方法の追記

**REQ.03 検証方法（末尾に追加）:**

> exit 0 かつ stderr が非空の CLI を mock した際に、MCP 成功応答の `_meta["legixy/warnings"]` に stderr 本文が格納されることの検査

### バージョン bump

SPEC-LGX-009 ヘッダ表 `Version: 0.6.0` → `0.6.1`

---

## 3. 修正 M-2: SPEC-LGX-003 REQ.19 — 欠落検出路を二経路に強化

### 問題の根拠

現行 REQ.19 の当該箇所：

> 欠落の検出可能性は **stderr Warning で確保する**。可用性 > 監査完全性の設計判断は ADR に記録する

「stderr Warning で確保する」は M-1 が実装される前提では MCP 経由の到達が未保証であり、
「確保する」という断言が過剰な保証になっている。
M-1 を追加した上で、検出路を二経路として明示する形に書き換える。

### 修正テキスト

**対象箇所:** SPEC-LGX-003 §3 REQ.19 末尾段落

**OLD:**
```
**MCP-INV-4 との関係**: 「全呼出記録」は**ベストエフォート**であり、完全性保証は「DB が利用可能な場合に限る」。欠落の検出可能性は stderr Warning で確保する。可用性 > 監査完全性の設計判断は ADR に記録する
```

**NEW:**
```
**MCP-INV-4 との関係**: 「全呼出記録」は**ベストエフォート**であり、完全性保証は「DB が利用可能な場合に限る」。欠落の検出可能性は以下の二経路で確保する:
1. **プロセス内（同期）**: stderr に Warning 診断を出力する（CLI 直接実行時の確認用）
2. **MCP 経由（非同期・Agent 到達保証）**: SPEC-LGX-009.REQ.03 の `_meta["legixy/warnings"]` 転送により、MCP 経由呼出し時でも Agent が各呼出しで書込失敗を検知できる

上記 2 経路はいずれも**呼出し単位の揮発的通知**である。書込失敗が連続する場合、Agent はセッション内の累積 Warning から恒常的障害を検知できる。永続的な失敗履歴の記録は本 SPEC の範囲外とし、ディスク満杯等の根本原因の解消を運用対応とする。可用性 > 監査完全性の設計判断は ADR に記録する
```

### バージョン bump

SPEC-LGX-003 ヘッダ表 `Version: 0.7.0` → `0.7.1`、変更履歴に以下を追加：

> | 2026-06-12 | 0.7.1 | ADR-LGX-004 可観測性強化: REQ.19 の欠落検出路を stderr + SPEC-LGX-009.REQ.03 _meta.warnings の二経路に変更。MCP 経由での Agent 到達保証を明示 |

---

## 4. 修正 M-3: SPEC-LGX-007 REQ.03 — analyze の context_log 完全性への注記

### 問題の根拠

SPEC-LGX-007 REQ.03 は現行 1 行の記述であり、context_log に欠落がある場合に
`legixy analyze` が生成する proposal の信頼度について何も述べていない。

analyze を実行する人間は、以下を知る手段がない：
- 分析対象の context_log に欠落があるか
- 欠落があるとすれば分析の網羅性にどの程度影響するか

「人間のみが実行する Admin Surface」であることから、この情報は人間が判断する上で必要である。

### 修正テキスト

**対象箇所:** SPEC-LGX-007 §3 REQ.03

**OLD:**
```
### SPEC-LGX-007.REQ.03: analyze コマンド（Admin Surface）

**内容:** `legixy analyze` は observations を集約・分析し、対応する proposal を生成する。**人間のみが CLI で実行する**。
**根拠:** CLAUDE.md, v0.1.0 継承
**検証方法:** CLI E2E テスト
```

**NEW:**
```
### SPEC-LGX-007.REQ.03: analyze コマンド（Admin Surface）

**内容:** `legixy analyze` は observations を集約・分析し、対応する proposal を生成する。**人間のみが CLI で実行する**。

**context_log 完全性の注記:** analyze は context_log の完全性を前提とするが、REQ.06 の
ベストエフォート書込方針により context_log に欠落が生じている可能性がある。
analyze 実行時に `legixy check --audit-health`（別途 REQ 化を推奨、本改訂の範囲外）等で
context_log の健全性を確認することを運用ガイドラインとして推奨する。
analyze 自身は context_log 欠落の有無を検査・報告しない（analyze の責務は observations → proposal 変換であり、
context_log の完全性検証は別コマンドの責務とする）。

**根拠:** CLAUDE.md, v0.1.0 継承、ADR-LGX-004 残存リスク（欠落の見逃し）の明示
**検証方法:** CLI E2E テスト
```

### バージョン bump

SPEC-LGX-007 ヘッダ表 `Version: 0.5.0` → `0.5.1`、変更履歴に以下を追加：

> | 2026-06-12 | 0.5.1 | ADR-LGX-004 可観測性強化: REQ.03 に context_log ベストエフォート書込によるデータ完全性の注記を追加。analyze 自身は欠落検出を行わず、健全性確認は別コマンドの責務と明示 |

---

## 5. 修正の相互依存関係

```
M-1 (SPEC-LGX-009 REQ.03 exit 0 stderr 転送)
  ↓ 転送保証を前提として
M-2 (SPEC-LGX-003 REQ.19 二経路記述) が「MCP 経由保証」を参照
  ↓ ADR-LGX-004 の残存リスク「運用で検知する」が具体化
M-3 (SPEC-LGX-007 REQ.03 analyze 注記) は M-1/M-2 と独立して適用可能
```

**適用順序:** M-1 → M-2 を同一改訂で適用（M-2 が M-1 を参照するため）。
M-3 は独立して適用可能だが同一改訂で適用することを推奨。

---

## 6. 修正の範囲外とした事項（意図的除外）

以下は今回の修正範囲に含めなかった。理由を明示する。

### 6.1 context_log 書込失敗の永続記録（observations テーブルへの書込）

observations テーブルは `category = correction / suggestion / issue` の三値（SPEC-LGX-007 REQ.01）で
定義されており、システム生成の健全性情報を混在させると analyze の分析対象が汚染される。
新規スキーマ（`health_counters` テーブル等）の追加は DD 段階の判断として保留。

### 6.2 `legixy check --audit-health` コマンドの新設

M-3 の注記で参照したが、新コマンドの仕様化は本改訂の範囲を超えるため別 GAP として管理することを推奨。

### 6.3 ADR-LGX-004 の改訂

ADR は決定時の文脈を記録するものであり、SPEC を修正することで残存リスクへの対応を記録する。
ADR 本体の「残存リスク」欄に「M-1 により軽減済み」の注記を追記することは任意。

---

## 7. 判断が必要な事項

### 7.1 `_meta["legixy/warnings"]` のキー名

`_meta["anthropic/maxResultSizeChars"]` は Anthropic 社の名前空間を使用している。
`warnings` フィールドに使用するキー名として以下の選択肢がある：

- **案A（推奨）**: `"legixy/warnings"` — プロジェクト固有名前空間、明確
- **案B**: `"warnings"` — シンプルだが他の MCP サーバとの衝突可能性

→ **どちらを採用するか判断してください。**

### 7.2 M-3 の「運用ガイドライン」の記述形式

SPEC に「運用ガイドラインとして推奨する」と書くことの適切性：
legixy の SPEC は要求（REQ）として実装義務を規定するものであり、
「推奨」という表現が要求記述のスタイルと合っているかの確認が必要。

→ 「`--audit-health` の新設 GAP を提起することを推奨する」という注記スタイルへの変更も選択肢。
