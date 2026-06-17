Document ID: GAP-LGX-171

# GAP-LGX-171: REQ.06 の PERF.03 参照値が NFR 改訂後の値と不整合

**親 TP**: TP-LGX-009
**観点出典**: TP-LGX-009 §3 追加 RED（カテゴリ横断・NFR 整合 / ライフサイクル性能予算）
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: GENUINE（doc-drift, minor）。NFR-LGX-001 PERF.03（行 81）は Step 1 (Windows) < 300 ms【E-04 反映】/ Step 2 (Linux Docker) < 200 ms に改訂済み（変更履歴 0.4.0-provisional 行 276 が E-04 緩和を確認）。SPEC-LGX-009 REQ.06 は単一値「< 200 ms 暫定」のまま固定されており、Step 1/2 区別と Windows 緩和値が未反映。SPEC 本文の参照値修正は人間承認が必要（ハードルール 1）。なお NFR §13 暫定表（行 237）も「< 200 ms」のままで NFR 内部にも残留 drift があり、同期時に併せて指摘する。

## 1. 観点

REQ.06 が引用する PERF.03 の値「< 200 ms 暫定」が、NFR-LGX-001 側の改訂後の値（Step 1 Windows < 300 ms / Step 2 Linux < 200 ms）と一致していない。MCP オーバーヘッド（CLI プロセス起動含む）の達成目標が SPEC と NFR で食い違う。

## 2. 現状の SPEC

SPEC-LGX-009 §3 REQ.06 は **「MCP サーバのオーバーヘッドは `compile_context` 応答全体で NFR-LGX-001.PERF.03（< 200 ms 暫定）を満たす」** と記すが、NFR-LGX-001 PERF.03 は 0.4.0-provisional（E-04 反映）で **Step 1（Windows）< 300 ms / Step 2（Linux Docker）< 200 ms** に改訂済み（Windows は CLI プロセス起動が重いため緩和）。REQ.06 は単一値「< 200 ms」を引用したままで、Step 1/2 の区別と緩和値が反映されていない。NFR §13 は Step 1 で「プロセス起動 50 ms + graph パース 100 ms + 実処理 ≦ PERF.03（< 300 ms）」のバジェットリスクを明記しており、MCP プロセス起動オーバーヘッドを負う本 SPEC で特に重要。

## 3. 期待される情報

SPEC に追加されるべき記述（人間承認が必要な SPEC 変更案として提示）:

- REQ.06 の PERF.03 引用を NFR の現行値に合わせる: Step 1（Windows）< 300 ms / Step 2（Linux Docker）< 200 ms
- MCP プロセス起動オーバーヘッド（v3 で 50 ms 想定）が Step 1 バジェットに収まるかの整合確認
- 「暫定」ラベルの扱い（NFR 側と同期）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-002 / UC-LGX-004（MCP 経由 compile_context）: 性能の成功定義が SPEC/NFR で二重定義
- 下流の TS（PERF ベンチマークテスト）: どの閾値で pass 判定するかが不定（200 か 300 か, OS 別か）
- NFR-LGX-001 PERF.03 との一貫性（SPEC が古い値を固定し drift）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-009 v0.5.2（人間承認 2026-06-10）: REQ.06 の陳腐化引用「PERF.03 < 200 ms 暫定」を現行 NFR 値（Step1 Windows < 300 ms【E-04】/ Step2 Ubuntu Docker < 200 ms）に同期し、正準ソースを NFR §3.2/§13 と明示。数値同期のみ・意図不変。NFR §13 行 237 の内部 drift は NFR 側改訂として別途提起。

## 6. 関連 ADR

なし（NFR との数値同期。architectural 判断ではない）
