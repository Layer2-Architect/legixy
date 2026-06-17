Document ID: GAP-LGX-110

# GAP-LGX-110: 一度も embed されていないノードの drift 検出（未生成 vs 古い）の区別が未定義

**親 TP**: TP-LGX-006
**観点出典**: TP-LGX-006 §2.3 観点 S-01
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重要度**: minor (verification: low-value, 人間判断で drop 可) — 敵対的精査パス 2026-06-09: SCORE-INV-1（「ハッシュ一致するスコアのみ fresh」）により行不在は自明に non-fresh。detect_drift の戻り型（未生成の表現）は DD 事項。状態 3 分類の明文化は望ましいが核心ではない。WEAK_OR_PADDED として維持。

## 1. 観点

REQ.05 は「content_hash が前回の embedding 生成時と変化した場合」を drift（古い）とするが、**そもそも embedding 行が存在しないノード**（新規追加され一度も embed されていない）の扱いが未定義。「未生成」と「古い（stale）」は区別されるべき状態であり、check（SPEC-LGX-004）が両者をどう報告するかの前提として SPEC-006 の状態定義が必要。bulk API の `detect_drift`（REQ.11）が未生成ノードをどう扱うか（DriftFinding に含めるか）も不明。

## 2. 現状の SPEC / UC

SPEC-LGX-006 §3 REQ.05 は content_hash「変化」のみを drift 条件とする。REQ.11 `detect_drift` は「各ノードのファイル SHA-256 と store の保存済 content_hash を比較」とするが、**store に当該ノードの行が無い場合**（未 embed）の戻り（DriftFinding に含めるか、別カテゴリか、無視か）が記述されていない。§4 不変条件 SCORE-INV-1 は「ハッシュ一致するスコアのみ fresh」とするが、行不在の状態が「fresh でない」のどのサブ状態かは未定義。

## 3. 期待される情報

- ノードの 3 状態の明示: fresh（hash 一致）/ stale（hash 不一致）/ 未生成（embedding 行なし）
- `detect_drift` が未生成ノードを DriftFinding に含めるか、別途報告するか、無視するか
- check の Drift Warning（SPEC-LGX-004 REQ.02 連携）が未生成と stale を区別して報告するか

## 4. 影響範囲

- UC-LGX-007: 新規ノード追加後の drift / freshness の事後条件
- SPEC-LGX-004（check の Drift 報告）: 未生成ノードを Warning にするかの前提が SPEC-006 で未確定
- 下流 DD / TS: detect_drift の戻り型（未生成を表現する enum/Option）とテスト

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-006 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.05 に 3 状態（fresh/stale/未生成）を定義。detect_drift は未生成を DriftFinding(kind=missing) として包含（表現は DD）、check の Drift 報告は stale と未生成を区別したメッセージで Warning【v3 差分: v3 は行不在を無言 skip】。

## 6. 関連 ADR

該当なし（状態定義の明文化）。
