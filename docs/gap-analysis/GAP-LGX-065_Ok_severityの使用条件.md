Document ID: GAP-LGX-065

# GAP-LGX-065: `Ok` severity の使用条件（個別 Ok finding か全体結果か）が未定義

**親 TP**: TP-LGX-004
**観点出典**: TP-LGX-004 §2.3 観点 S7
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: WEAK_OR_PADDED として維持。REQ.03 が `Ok: 問題なし` を定義済。Ok finding を個別カテゴリ単位で発行するか全体ステータスとするかは慣例仕様（old.source の CheckReport は ChainIntegrity 等にカテゴリ単位 Ok finding を発行）で示唆され、最終的な finding シリアライズ形状は DD-LGX-001 §2.4（NFR-LGX-001.OBS.06 が参照）の責務。SPEC レベルの決定相当性は低い。**severity: minor（verification: low-value, 人間判断で drop 可）**。

## 1. 観点

REQ.03 は severity の 1 つに `Ok`（問題なし）を含めるが、`Ok` を **いつ finding として発行するか** が未定義。個別カテゴリごとに「Ok」finding を返すのか、問題なしの場合は finding 0 件で CheckReport 全体が Ok と判定されるのか、両方なのかが曖昧。これは空グラフ・全 pass 時の出力形状（GAP-LGX-061）にも波及する。

## 2. 現状の SPEC / UC

SPEC-LGX-004 §3 REQ.03 は `Ok: 問題なし` と定義するのみ。REQ.08 の CheckReport 出力でも Ok finding の有無に触れていない。他 severity（Error/Warning/Info）は具体的な発行条件が各 REQ にあるが、Ok だけ発行トリガが宙に浮いている。

## 3. 期待される情報

SPEC に追加されるべき記述:

- `Ok` が (a) 全検証 pass 時の CheckReport 全体ステータス、(b) カテゴリ単位の正常 finding、のどちらの意味で使われるか
- finding が 0 件の場合の CheckReport の表現（空配列 + 全体 Ok か、Ok finding 1 件か）
- `--log-format=json`（REQ.08）出力時の Ok の表現

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流の TS / TC: 全 pass 時の出力期待値（finding 配列の中身）が書けない
- MCP / 外部ツール連携: CheckReport を機械解析する側のスキーマ前提が固定できない
- 他の TP / GAP との依存関係: GAP-LGX-061（空グラフ）・GAP-LGX-071（finding スキーマ）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-004 v0.8.0（人間裁定 fix・承認 2026-06-10）: REQ.03 に Ok の使用条件を確定 — カテゴリ finding として発行しない（v3 実測: producer 不在・reporter.rs:62 で skip）。全 pass = findings 0 件 + counts 0 + exit 0 が正準。Ok は集計・将来拡張のための予約 severity。

## 6. 関連 ADR

該当なし。
