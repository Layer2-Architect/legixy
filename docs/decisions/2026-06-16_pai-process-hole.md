# プロセス所見: PAI が露呈した DevProc の穴（presence ≠ SPEC outcome）

| 項目 | 内容 |
|---|---|
| 日付 | 2026-06-16 |
| 起点 | PAI-CLI-001 / PAI-MCP-001（外部黒箱検査 `legixy.test`）が検出した R-1〜R-7 |
| 種別 | DevProc プロセス改善所見（13章 §「PAI の本質＝プロセス自体の穴の発見」） |
| ステータス | 記録済み（DevProc 方法論への反映は **提案**。方法論の一方的改変はしない） |
| 関連 | `legixy.test/docs/defect-root-cause-2026-06-14.md`（初回 54 件の RC-1〜RC-5）、PAI-CLI-001 §6、PAI-MCP-001 §6 |

## 1. 所見の核（穴の同定）

R-1〜R-7（および初回 54 件の一部）に共通する構造的欠落:

> **チェーン（UC→…→SRC→TC[DLV]）は「機能の *存在*（presence）」を GREEN にしたが、
> SPEC が要求する「*意図・成果*（intent / outcome）」を検証していなかった。**

具体例:
- **R-1（最も鋭い）**: チェーンは「サブノードが materialize する」「`--granularity subnode` を受理する」という *presence* を
  GREEN にした。だが LGX-EXT-001 **目的1=トークン削減** という *outcome* を測っていなかった。実バイナリでは逆に増大
  （5202 > 2146）していたが、in-repo TC は「`subnode_id` 行が出るか」を見て「トークンが減るか」を測っていなかった。
- **R-4/R-5**: 意味層チェッカ単体（`tc_bug005_semantic`）と analyzer 変換規則は各々 GREEN（presence）だったが、
  「check 結果 *や embedding* から Observation を生成し全周接続する」という SPEC-LGX-007.REQ.02 の *outcome* が未配線。
- **R-3**: サブノード ID 形式検査は presence を GREEN にしたが、明示 `#s:` の **本文が返る** という outcome が空だった。
- **R-7（slug）**: `#s:<slug>` を「非空」で受理（presence）したが、文字制約という *intent*（LGX-EXT-001 §4.5.2）を強制せず。

これは初回 54 件の RC-1/RC-2（凍結契約の適合テストが成果物として無い・CLI 統合層がチェーン孤児）の **より深い層**である:
契約適合テストを in-repo 化（Phase B の `[6/6]` ゲート）した *後* でも、独立黒箱（PAI）が更に 7 件を検出した。
**＝ in-repo 契約ゲートは必要だが不十分。独立 PAI チャネルが不可欠**（13章 §6 / 12章補正の実証）。

## 2. なぜすり抜けたか（root-cause）

1. **TC/TC[DLV] が presence-oriented**: テストケースが「出力に X が現れる」「exit が N」という*存在・形式*を検証し、
   SPEC が掲げる*定量的成果*（削減量・本文非空・全周到達・解像度一致）を assert していなかった。
2. **Author と同一分布**: in-repo テストは実装者が書くため、実装者が見落とした観点（= outcome の未測定）も同時に見落とす。
   独立な主体・独立な観点（SPEC から起こす PAI）でしか露見しない。
3. **配線（統合層）の被覆不足**: 単体は GREEN でも E2E 配線が欠落（R-4/R-5/R-3）。単体 presence の合算 ≠ 製品の outcome。

## 3. DevProc 改善 **提案**（方法論の一方的改変はしない）

13章の趣旨（PAI 失敗→root-cause→DevProc 改善）に従い、以下を**提案**として記録する。採否は方法論オーナーの裁定事項:

- **P-A（TC/PAI の outcome 化）**: TC[DLV] および PAI ケースは、SPEC の*意図・成果*を観測する assertion を最低 1 つ含む
  ことを推奨（presence のみのケースを禁ずる）。例: 「subnode 粒度の返却サイズ < document」「明示サブノード本文が非空」。
  本 legixy では in-repo 回帰に既に内在化済み（`subnode < document` のバイト比較等）。
- **P-B（PAI ゲートの制度化）**: リリース前に独立 PAI ゲート（PAI-CLI-001 §3）を必須化。Author の in-repo GREEN を
  ゲート通過の根拠としない（品質偏向防止＝Author の自己申告への過信を構造的に排除）。
- **P-C（両輪の明文化）**: 独立黒箱（PAI / 外部スイート）を in-repo へ畳まない。決定論ケースは TC[DLV] へ昇格しても、
  独立チャネルは存続させる（独立性＝発見力の保存）。
- **P-D（テンプレート注記）**: `templates/TC-template`・`PAI-template` に「presence だけでなく SPEC outcome を assert」の
  注記追加を提案。

## 4. 本プロジェクトでの内在化状況（既了）

| 提案 | legixy での状態 |
|---|---|
| P-A outcome 化 | 一部内在化（`tc_bug007_cli_e2e.rs` の `subnode < document` 比較・本文非空 assert・investigate 解像 assert） |
| P-B PAI ゲート | PAI-CLI-001 §3 に手順を制度化（独立黒箱・要改善0で通過） |
| P-C 両輪 | PAI-CLI-001 §5 / §3-5 に明文化（trace-check に畳まない） |
| P-D テンプレート | DevProc 方法論側のため**提案に留める**（本決定記録で起票） |

## 5. 自己批判（記録）

Author（実装）セッションは Phase B 完了時に「適合水準到達」と述べたが、独立 PAI が R-1〜R-7 を更に検出した。
これは「確率論的変換器（AI）を自身の出力で検証してはならない」「Author は自身の出力に過信せず Reviewer 層と PAI に
検証を委ねる」（`00-philosophy.md` §1 品質偏向防止）の実例であり、PAI が構造的に対処しようとしている当の弱点である。
