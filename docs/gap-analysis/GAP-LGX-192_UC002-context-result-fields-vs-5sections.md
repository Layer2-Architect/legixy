Document ID: GAP-LGX-192

# GAP-LGX-192: ContextResult 4 フィールドと SPEC REQ.10 の 5 セクション配置の対応未明示

**親 TP**: TP-LGX-012
**観点**: §2.1 BF3
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-002 Step6 で返却される ContextResult の構造フィールド（targets / upstream / layer_documents / custom_documents）と SPEC-LGX-003.REQ.10 が規定する 5 セクション配置順序（Layer Guidelines / Additional Guidelines / キャッシュブレーク点マーカ / Upstream Artifacts / Target Node Metadata）の対応関係が UC フローで観察できない。`layer_documents` が Layer Guidelines + Additional Guidelines に対応するのか、`targets` が Target Node Metadata に対応するのか等の写像が不明であり、返却構造の設計意図が UC レベルで確認できない。

## 2. 現状の UC / SPEC

- **UC-LGX-002 Step6**: ContextResult として targets / upstream / layer_documents / custom_documents の 4 フィールドを列挙
- **SPEC-LGX-003.REQ.10**: 返却内容を Layer Guidelines / Additional Guidelines / キャッシュブレーク点マーカ / Upstream Artifacts / Target Node Metadata の 5 セクション（この順序固定）として規定
- **TP-LGX-003 D-03（GREEN 確定済）**: REQ.10 の 5 セクション配置順序は SPEC レベルで答え済み

## 3. 推奨対応（人間裁定）

**(A) UC-LGX-002 Step6 に ContextResult 構造と 5 セクションの対応関係を注記する**
例: 「targets は Target Node Metadata に対応、upstream は Upstream Artifacts に対応、layer_documents は Layer Guidelines + Additional Guidelines に対応」という写像を UC の説明または注記として追記する。REQ.10 の配置順序もフロー上で観察可能となる。

**(B) drop（委譲容認）**
返却構造の詳細（フィールド名 / セクション配置の写像）は DD レベルの設計事項であり、UC レベルでは ContextResult として抽象的に記述することで十分とみなす。TP-LGX-003 D-03 が GREEN である以上、UC フロー記述への明示は必須でない。

## 4. 影響範囲

- UC-LGX-002 基本フロー Step6（注記追加の場合）
- TP-LGX-012 BF3（解消後 GREEN 化）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
