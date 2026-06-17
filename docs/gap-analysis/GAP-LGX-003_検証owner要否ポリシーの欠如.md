Document ID: GAP-LGX-003

# GAP-LGX-003: 「検証」owner を持たない不変条件の検証要否ポリシーが未規定

**親 TP**: TP-LGX-001
**観点出典**: TP-LGX-001 §2.2 観点 7（不変条件 × SPEC 責任マトリクスの完全性 — 実装のみで検証 owner を持たない不変条件の扱い）
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**severity**: minor
**敵対的精査メモ（2026-06-09）**: GENUINE と確認。SPEC-LGX-004 §4 は自 SPEC が検証する不変条件と関与しない不変条件（CTX-INV-3, MCP-INV-1〜4, SUBNODE-INV-5）を列挙するが、これは下位 SPEC の自己宣言であって、検証 owner を持たない不変条件（CTX-INV-1 決定論性 / CACHE-INV-1 バイト決定論 / MCP-INV-2 忠実転送 等）を「型・構成的保証で検証不要」と「実行時検出が必要」に分類する umbrella レベルのポリシーは NFR-LGX-001・foundational spec §10 を含めどこにも存在しない（grep 確認済）。SUBNODE-INV-5 のみ §4.2 が個別根拠を持つ。検証網羅性の主張と RPC 母数に影響するため refute 不能。観点 8（GAP-002）の整合検証とは別概念（こちらは要否ポリシー、あちらは二重マトリクス整合）で DUPLICATE ではない。severity は minor。

## 1. 観点

§4.1 マトリクス上で「実装」owner はあるが「検証」owner をどの SPEC も持たない不変条件について、検証が不要であることの根拠（型による構成的保証／検証対象外）が示されているか。また、どの不変条件が検証 owner を要し、どれは要さないかのポリシーが umbrella で規定されているか。

## 2. 現状の SPEC / UC

SPEC-LGX-001 §4.1 を全行精査すると、「検証」owner を 1 件も持たない不変条件が多数存在する。確認した範囲では:

- CTX-INV-1（決定論保証）, CTX-INV-3（カスタムエッジ独立性）
- FB-INV-1〜5（フィードバックループ全 5 件）
- SCORE-INV-1（ハッシュ一致保証）
- MCP-INV-1（Agent Surface 限定）, MCP-INV-2（忠実な転送）, MCP-INV-3（Observation 重複排除）, MCP-INV-4（監査ログ完全性）
- STATE-INV-1（ステートレス性）, STATE-INV-2（graph.toml は Git 経由）
- SUBNODE-INV-5（ID 決定性）
- CACHE-INV-1〜4（全 4 件）

これらはいずれも実装 owner を持つため「孤児不変条件」ではない（観点 6 は GREEN）が、**違反を実行時に検出する手段（検証 owner）が割り当てられていない**。§4.2 は SUBNODE-INV-5 について「生成ロジック側で保証されるため検証対象外」と個別に根拠を述べているが、上記の他多数（特に CTX-INV-1 決定論性、CACHE-INV-1 バイト単位決定論、MCP-INV-2 忠実転送）については検証不要の根拠も、検証要否を判断する一般ポリシーも記載が無い。凡例（§4.1）も「検証＝違反検出手段の規定」と定義するのみで、どの不変条件がそれを要するかの規範を持たない。

## 3. 期待される情報

SPEC に追加されるべき記述:

- 不変条件を「型・構成的に保証され実行時検証を要しないもの（construction-time guarantee）」と「実行時/CI で違反検出が必要なもの（needs detection）」に分類するポリシー、または各不変条件への分類ラベル付与。
- 検証 owner を持たない不変条件それぞれについて、SUBNODE-INV-5 と同様の「なぜ検証不要か」の一行根拠（例: CTX-INV-1 は決定論性テスト＝property test として TS 層で扱うため SPEC レベル検証 owner を置かない、等）。
- 特に決定論系（CTX-INV-1, CACHE-INV-1）と忠実転送（MCP-INV-2）は外部観察可能な事後条件を持つため、検証 owner 不在が意図的か漏れかを明示する。

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 検証フェーズ（SPEC-LGX-004）: どの不変条件に対する違反検出を check / semantic_check が実装すべきかの境界が曖昧になり、検証の網羅性を主張できない。
- TS / TC: 検証 owner 不在の不変条件をテストする責任が宙に浮き、property test / E2E でカバーすべきか実装側の単体テストで足りるかの判断が成果物追跡から導けない。
- 下流の RPC（責務保存率検査）: 不変条件 → 検証成果物の対応が欠落すると保存率の母数が不定になる。

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-001 v0.7.0（人間承認 2026-06-10）: §4.4「検証 owner 要否ポリシー」を新設し、検証 owner を持たない全不変条件を C（construction-time guarantee）/ D（needs-detection、TS/TC 層委譲）に分類。§4.1 凡例に分類参照を追記。

## 6. 関連 ADR

- 「construction-time guarantee と needs-detection の二分」を不変条件管理ポリシーとして採用する場合、ADR-LGX-NNN を起票。
