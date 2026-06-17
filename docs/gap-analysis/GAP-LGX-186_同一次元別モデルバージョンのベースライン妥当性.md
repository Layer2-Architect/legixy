Document ID: GAP-LGX-186

# GAP-LGX-186: snapshot ベースラインと現行 embedding の model_version 不一致（同一次元）の扱いが未定義

**親 TP**: TP-LGX-010
**観点出典**: TP-LGX-010 §2.6 バージョニング・互換性 観点 V6
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: GENUINE / 維持（major）。検証: SPEC-010 §4 不変条件表は SCORE-INV-2（モデルバージョン一致, foundational §line 245「現在のモデルバージョンと一致するスコアのみ有効」）の検証手段を REQ.03 の**次元不一致 Error のみ**と宣言する。次元不一致 Error は dims が異なるときしか発火しない。REQ.02 は snapshot が model_version を保存すると述べるが、REQ.03 に baseline 保存 model_version と現行 model_version を照合する要求が一切無い。よって 384→別 384 次元モデルへの切替（SPEC-006 REQ.10 が再生成を要求するが旧 snapshot は凍結保持）では次元一致のため Error が発火せず、SCORE-INV-2 違反の意味不能ベクトル間で偽 drift 値が静かに返る。§4 が「SCORE-INV-2 検証 = 次元不一致 Error」と主張する点が証明可能に不完全。**severity: major（モデルバージョン遷移／SCORE-INV-2 検出漏れテーマ）**。

## 1. 観点

`drift --against snapshot:<L>` で、ベースライン snapshot が**旧 model_version で凍結**され、現行 embedding が**別 model_version だが同一次元**の場合の意味的妥当性が未定義。次元不一致は Error（REQ.03）で捕捉されるが、**次元が一致したまま model_version だけ異なる**ケースは検出されず、意味のない drift 値を返しうる。

## 2. 現状の SPEC / UC

SPEC-LGX-010:

- REQ.03 は次元数**不一致**を exit 1 とし、§4 で「SCORE-INV-2（モデルバージョン一致）の検証 = REQ.03（drift の次元不一致 Error が違反状態の検出手段）」と位置づける
- しかし SCORE-INV-2 は「現在のモデルバージョンと一致するスコアのみ有効」であり、**次元が同じでも model_version が異なれば SCORE-INV-2 違反**になる。次元一致・model_version 不一致のケースは次元不一致 Error では捕捉できない
- REQ.02 は snapshot が `content_hash / model_version を含む行を複製しベースラインの同一性情報を保持」する（§4 SCORE-INV-1）と述べる。つまり baseline には model_version が**保存されている**が、drift 対比時にこの保存 model_version と現行 model_version を**照合する要求が無い**
- 例: モデルを `paraphrase-multilingual-MiniLM-L12-v2`(384) から別の 384 次元モデルへ更新（SPEC-LGX-006 REQ.10 は再生成を要求するが、旧 snapshot は凍結されたまま）。両者 384 次元のため REQ.03 の次元不一致 Error は発火せず、意味的に比較不能なベクトル間で drift 値が算出される

## 3. 期待される情報

SPEC に追加されるべき記述:

- `drift --against snapshot:` で baseline の保存 model_version と現行 embedding の model_version が**異なる（次元は一致）**場合の扱い:
  - Error（exit 1、SCORE-INV-2 違反として明示対比は失敗を隠さない）とするか
  - Warning + 算出継続（model_version 差を stderr に明示した上で drift 値を返す）とするか
  - その判断根拠（SCORE-INV-2 の「有効性」をどこまで運用層で強制するか）
- baseline 不在系（exit 0）・次元不一致（exit 1）の既存分類との一貫性
- SCORE-INV-2 の「検証手段」が次元不一致のみでは不十分である点の §4 への反映

## 4. 影響範囲

- UC-LGX-013（drift）: model_version 遷移期の代替フローが定義できない（§1.3 は「モデル解決失敗」「次元不一致」を挙げるが model_version 不一致は挙げていない）
- 下流 DD / TS: model_version 照合ロジック・モデル切替テストの観点が確定しない
- SCORE-INV-2 の実効性: 次元不一致のみを検出手段とすると同一次元のモデル切替で偽の drift が静かに通る
- 関連: SPEC-LGX-006 REQ.10（モデル更新時再生成）と整合

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-010 v0.2.1（人間承認 2026-06-10）: REQ.03 に model_version 不一致（次元一致）→ exit 1 を追加（一次検出、SPEC-LGX-006.REQ.10 完全一致照合）。§4 SCORE-INV-2 行の過大宣言を「model_version 照合一次・次元不一致補完」に訂正、§1.3 UC-013 代替フローに追記。ownership は drift 出力契約＝SPEC-LGX-010 所在。ADR-LGX-007。

## 6. 関連 ADR

model_version 不一致を Error とするか継続とするかは運用ポリシー判断のため、確定時に ADR を検討:

- ADR-LGX-NNN: drift ベースライン対比における model_version 照合ポリシー
