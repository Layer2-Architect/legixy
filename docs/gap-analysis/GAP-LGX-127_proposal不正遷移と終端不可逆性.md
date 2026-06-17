Document ID: GAP-LGX-127

# GAP-LGX-127: proposal の状態遷移グラフ・終端不可逆性・再操作・並行決着の一括未定義

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.3 観点 S1 / S2 / S3 / §2.4 観点 C2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**統合（2026-06-09 敵対的精査パス）**: 旧 GAP-LGX-128（終端 proposal への再操作の冪等/エラー）および旧 GAP-LGX-130（同一 id への並行 approve/reject の決着）は本 GAP に統合した。いずれも「proposal の遷移グラフ定義」という同一論点の派生（逐次再操作・並行競合）であり、解決には単一の遷移規則定義で足りるため。

## 1. 観点

proposal の状態遷移は `pending → approved` と `pending → rejected` のみが正当だが、それ以外の遷移（approved → rejected、rejected → approved、approved → approved 等）が禁止されること、および終端状態（approved / rejected）が不可逆であることが定義されていない。あわせて以下の派生論点も未定義:
- (旧 128) 既に approved/rejected な proposal への再 approve/再 reject が冪等 no-op かエラー（exit 1）か
- (旧 130) 同一 proposal-id への並行 approve/reject の決着（最初に commit した 1 操作のみ成立 = `UPDATE ... WHERE status='pending'` の行数判定か、後勝ちか）

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.09 は status 列挙値（pending / approved / rejected）と各フィールドを定義するが、許容される遷移グラフを定義していない。§4 の FB-INV-3 は **pending** の不変性（context 結果に影響しない）のみで、終端状態の不可逆性には触れない。approve/reject が「pending な proposal のみ」を対象とするという前提条件も明記されていない。

## 3. 期待される情報

SPEC-LGX-007 REQ.05 / REQ.09 に以下を追加すべき:

- approve/reject は `status = 'pending'` の proposal にのみ作用する（前提条件）
- 正当な遷移は `pending → approved` / `pending → rejected` のみで、それ以外は禁止
- approved / rejected は終端状態であり不可逆
- (旧 128) 終端状態への再操作の応答方針（エラー exit 1 で拒否し既存証跡を変更しない／冪等 no-op で approved_by/approved_at を上書きしない、のいずれか）
- (旧 130) 同一 proposal への並行 approve/reject は最初に commit した 1 操作のみ成立し、後続は終端状態として上記再操作規則に従う（`UPDATE ... WHERE status='pending'` の行数で CAS 判定）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: proposal lifecycle の状態遷移図（逐次・並行・再操作を含む）が確定しない
- 下流の RBA/SEQA/DD: proposal アグリゲートの不変条件（型レベルでの遷移禁止、CAS 更新）が設計できない
- 下流の TS / TC: 不正遷移・再操作・並行 approve/reject ストレステストの期待結果が決まらない

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.3（人間承認 2026-06-10）: REQ.09 に proposal 状態モデル（pending→{approved|rejected} のみ・終端不可逆・再操作 exit 1・CAS 並行解決）を確定、REQ.05 に pending 限定作用を明記。旧 GAP-128/130 を吸収。typestate vs 実行時は DD 委譲。

## 6. 関連 ADR

状態遷移を型レベルで禁止する（typestate）か実行時チェックにするかは設計判断のため、DD 段階で ADR 検討。
