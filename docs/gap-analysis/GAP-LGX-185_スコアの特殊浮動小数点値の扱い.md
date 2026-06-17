Document ID: GAP-LGX-185

# GAP-LGX-185: cosine スコアの特殊浮動小数点値（NaN / ±Inf）の扱いが未定義

**親 TP**: TP-LGX-010
**観点出典**: TP-LGX-010 §2.1 境界値 観点 B9
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: GENUINE / 維持。SPEC-LGX-006 REQ.01 は mean pooling + L2 正規化を述べるがゼロベクトル（空コンテンツ由来の 0/0 NaN cosine）排除を保証せず、生成側委譲では閉じない。calibrate の clamp は「域外の実数」のみ対象で NaN は順序比較が偽のため捕捉不能、`--json` の drift が NaN を取ると JSON 不正となり REQ.06 決定性 / consumer 連携を破壊する。これは SPEC-010 の出力契約（consumer 側）の真の沈黙。**severity: minor〜major（出力契約／特殊値テーマ）** — 主トリガ（ゼロベクトル）は edge だが `--json` parse-break は実害。

## 1. 観点

`report` / `calibrate` / `drift` が扱う cosine 類似度スコアに **NaN / ±Inf** が混入した場合（ゼロベクトル embedding による 0/0、正規化漏れ等）の扱いが SPEC に定義されていない。calibrate の clamp 規定は「域外の実数」を対象とするが NaN/Inf を捕捉できない。

## 2. 現状の SPEC / UC

SPEC-LGX-010:

- REQ.05（calibrate）は「値域 [0.0,1.0] 固定の等幅 N バケット。域外スコア（負の cosine 等）は clamp して算入し、上限 1.0 は末尾バケットに含める。min/max/mean は clamp 前の生値」と述べる。**clamp は順序のある実数（負値・1 超）を想定**しており、NaN は順序比較が偽になるため `clamp(0.0, 1.0)` で 0 にも 1 にもならず、どのバケットにも入らない/集計を汚染する可能性がある。Inf も「clamp 前の生値」を mean に算入すると mean が Inf/NaN に伝播する
- REQ.04（report）の `links` の `score`・`summary` の `min_link_score / max_link_score / mean_link_score` に対する特殊値方針は皆無
- REQ.03（drift）の `drift = 1.0 − cosine` も cosine が NaN なら drift が NaN になるが、`--json` の `drift` フィールドが NaN を取りうるか（JSON は NaN を表現できない）未定義
- core-perspectives.md §境界値「負の値 / 浮動小数点の特殊値（NaN, ±Inf, -0.0）」に対応する記述が無い

SPEC-LGX-006（生成側）も embedding が正規化済み（mean pooling + L2 正規化、REQ.01）である前提を述べるが、ゼロベクトル（空コンテンツ等）由来の NaN cosine を排除する保証は明文化されていない。

## 3. 期待される情報

SPEC に追加されるべき記述（責務先の判断含む）:

- cosine 計算で NaN/±Inf が生じうるか（ゼロベクトル・正規化漏れの可能性）の前提整理。生成側で排除されるなら SPEC-LGX-006 への委譲を明記
- 本 SPEC の consumer 側で NaN/Inf スコアが現れた場合の扱い:
  - calibrate: NaN ペアを skip するか（次元不一致 skip と同様の集約 Warning 経路に乗せるか）、min/max/mean 集計から除外するか
  - report: `score` / summary 統計への NaN/Inf 伝播防止、`--json` での表現
  - drift: `--json` の `drift` が JSON 非表現値（NaN/Inf）を取らないこと（null 化等）
- 「特殊値は発生しない」と結論する場合、その不変条件（embedding が常に非ゼロ L2 正規化済）の所在を明示

## 4. 影響範囲

- 下流 TS: 既知分布 fixture テスト（REQ.05 検証方法）が特殊値ケースを含められない → 偽 green リスク
- `--json` の機械可読性（REQ.06 決定性・consumer 連携）: JSON が NaN を含むと parse 不能になりパイプ下流を壊す
- SPEC-LGX-006 との境界面（REQ.08）: 特殊値排除がエンジン責務か運用責務かの分担が不明確
- 関連: SCORE-INV 系（決定論・ハッシュ一致）と整合

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-010 v0.2.1（人間承認 2026-06-10）: REQ.09 新設 — 非有限スコア（NaN/±Inf）は calibrate/report で skip + 集約 Warning、drift で exit 1、--json は非有限値非出力（統計不能時 null）。REQ.03/04/05 に参照差込。REQ.06 決定性の前提を確立。ADR-LGX-007。

## 6. 関連 ADR

（通常は不要）
