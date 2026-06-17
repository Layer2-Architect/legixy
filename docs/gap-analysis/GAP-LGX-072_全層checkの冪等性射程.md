Document ID: GAP-LGX-072

# GAP-LGX-072: 意味検証を含む全層 `check` の冪等性・結果順序が REQ.06 の射程に含まれるか未定義

**親 TP**: TP-LGX-004
**観点出典**: TP-LGX-004 §2.10 観点 D2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: WEAK_OR_PADDED として維持。embedding スコアの決定性そのもの（ONNX 推論・浮動小数点）は SPEC-LGX-006 + CTX-INV-1（決定論保証）+ NFR-LGX-001.REL.10（返却バイト列決定論）の責務であり SPEC-004 の射程外。check-004 固有に残るのは「全層 check の finding 出力順の安定ソートキー」のみで、REQ.06（formal の結果順序固定）からの自然な延長として導出可能。判定相当性は低い。**severity: minor（verification: low-value, 人間判断で drop 可）**。

## 1. 観点

REQ.06 は「`check --formal` は同一入力に対して常に同一の CheckReport を返す（結果順序含む）」と冪等性を保証するが、明示的に **`--formal` に限定** されている。意味検証（SemanticSimilarity / LinkCandidate / Drift / IdSemanticDrift）を含む全層 `check` の決定性・結果順序が同様に保証されるかが未定義。ONNX 推論結果や embedding 比較順が実行ごとに変動しない保証（CTX-INV-1 決定論との整合）が SPEC として未言及。

## 2. 現状の SPEC / UC

SPEC-LGX-004 §3 REQ.06 は `check --formal` のみを対象に冪等性を規定。§4 では CTX-INV-1（決定論保証）を REQ.06 に紐づけるが、これも検証動作の再現性一般を指すのみで、意味層（浮動小数点類似度・スレッド並列順）まで結果順序を固定するかは不明。NFR-LGX-001.REL.03 も `check --formal` を対象としている。

## 3. 期待される情報

SPEC に追加されるべき記述:

- 全層 `check` の冪等性・結果順序保証の有無（formal と同等か、意味層は順序のみ保証で値は近似決定的か）
- ONNX 推論・embedding 比較の決定性前提（同一モデル・同一入力で同一スコア）と、浮動小数点誤差の許容範囲
- 結果順序の安定ソートキー（finding の出力順を何で決めるか）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流の TS / TC: 全層 check の反復実行テストの期待値（厳密一致か近似一致か）が書けない
- CI: check 出力の差分検知（golden file 比較）が安定しない恐れ
- 他の TP / GAP との依存関係: GAP-LGX-063（モデル不在）・GAP-LGX-066（実行中外部更新）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-004 v0.8.0（人間裁定 fix・承認 2026-06-10）: REQ.06 を全層 check に拡張 — 安定ソートキー（severity 降順→category→related_ids、詳細 DD）による順序決定論、スコア値はビット再現対象外（ADR-LGX-003 と同一適用範囲）、golden 比較は順序・件数・severity で行う指針。

## 6. 関連 ADR

該当なし。
