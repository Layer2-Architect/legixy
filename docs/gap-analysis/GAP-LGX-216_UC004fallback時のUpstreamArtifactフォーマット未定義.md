Document ID: GAP-LGX-216

# GAP-LGX-216: UC-LGX-004 fallback 時の UpstreamArtifact フォーマット（subnode_id 等の有無）が未定義

**親 TP**: TP-LGX-014
**観点**: §2.5 DF1（fallback 時の UpstreamArtifact フォーマット）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-004 の基本フロー Step 4 は「結果の UpstreamArtifact に subnode_id / anchor / content / drift_score を含めて返却する」と拡張フィールドを明示するが、代替フロー 3a（サブノード不在 fallback でドキュメント全体を返却）時の同フォーマットが未定義である。具体的には:

- fallback 時の UpstreamArtifact において subnode_id フィールドは null か absent か
- anchor フィールドは null か absent か
- drift_score フィールドはドキュメント全体のスコアか null か

基本フロー（subnode 展開）と代替フロー 3a（ドキュメント全体 fallback）では UpstreamArtifact の構造が異なる可能性があり、その差分が UC フロー記述に観察可能でない。

## 2. 現状の UC / SPEC

- **UC-LGX-004 基本フロー Step 4**: 「UpstreamArtifact に subnode_id / anchor / content / drift_score を含めて返却する」。
- **UC-LGX-004 代替フロー 3a**: 「サブノードが存在しない上流成果物は、ドキュメント全体として返却する（fallback）」。フォーマット詳細なし。
- **SPEC-LGX-003**: subnode_id / anchor の有無は DD レベルの設計事項として委譲されているが、UC フロー内で両ケースのデータ構造差が示されていない。
- **問題点**: fallback 時の UpstreamArtifact が subnode モードと同一の構造（一部フィールドが null/absent）なのか、UC-LGX-002 と同一の構造（拡張フィールドなし）なのかが不明確。これは RBA / DD レベルでのデータモデル設計に影響する。

## 3. 推奨対応（人間裁定）

**(A) UC への追記案**

代替フロー 3a に「fallback 時の UpstreamArtifact は content にドキュメント全文を含み、subnode_id / anchor は absent（または null）、drift_score はドキュメントレベルのスコアを付与する（embedding 不在時は absent）」を追記する。これにより基本フローと fallback の構造差が観察可能になる。

**(B) drop（委譲容認）案**

UpstreamArtifact の詳細フォーマット（フィールドの null/absent 区別含む）は DD レベルのデータモデル設計として SPEC 委譲し、UC フローの修正は不要と裁定する。UC フローは「フォーマット仕様」ではなく「ユースケースのフロー記述」の役割に限定する。

## 4. 影響範囲

- UC-LGX-004 §代替フロー 3a（追記案の場合）
- 下流への影響: RBA の UpstreamArtifact ドメインオブジェクト設計（subnode あり/なしの polymorphism 方針）に直接影響。GENUINE 確定の場合は RBA 着手前に解消が望ましい

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
