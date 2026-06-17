Document ID: GAP-LGX-140

# GAP-LGX-140: 「人間のみ CLI 実行」の強制手段が MCP 非露出のみに依存している

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.11 観点 D1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

feedback / analyze / approve / reject が「人間のみが CLI で実行する」と宣言されているが、その強制手段が MCP に露出しないこと（MCP-INV-1）のみに依存している。Agent（Claude Code）が Bash ツール等で legixy CLI を直接起動すれば approve/reject を実行できてしまう経路への対処が未定義。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.02/03/05 は「人間のみが CLI で実行する」「Claude Code（Agent）は実行禁止」と記述し、検証方法は「MCP ツール一覧に含まれないこと（SPEC-LGX-009.REQ.02）」とする。これは MCP 経由の到達を塞ぐが、Agent が CLI を直接 spawn する経路（Claude Code は Bash を持つ）に対するガード（PIPE_ROLE 識別・環境変数・確認プロンプト等）が SPEC-LGX-007 に明記されていない。NFR-LGX-001 SEC.08 が「PIPE_ROLE 識別の前提」に触れるが、approve/reject への適用が SPEC-007 にリンクされていない。

## 3. 期待される情報

SPEC-LGX-007 REQ.05 に以下を追加すべき:

- 「人間のみ」が運用規律（CLAUDE.md 絶対ルール5）に依拠する宣言的保証なのか、CLI 側の技術的ガード（role 識別・対話確認等）を伴うのかの明示
- 技術的ガードを設ける場合の手段（PIPE_ROLE / 環境変数 / TTY 判定）と NFR SEC.08 との接続
- 設けない場合は「強制は運用規律であり技術的には Agent から到達可能」というリスク受容の明示

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: approve/reject のアクター権限制約が具体化できない
- 下流の DD: CLI コマンドハンドラに role ガードを入れるかの設計が決まらない
- CLAUDE.md 絶対ルール5（承認権限の制限）の技術的実効性
- 他の TP / GAP との依存関係: REQ.10（テスト不可侵 role 制御、GREEN 判定済）と role 識別機構を共有しうる

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.6（人間承認 2026-06-10）: REQ.05 に「人間のみ」の二層強制（MCP 非露出 + CLAUDE.md ルール 5）と Bash 直接 spawn の SEC.08 下リスク受容を明文化。改ざん耐性ガードは要件外。§4 MCP-INV-1 行更新。ADR-LGX-006。

## 6. 関連 ADR

承認権限の強制を宣言的規律に留めるか技術的ガードを設けるかは security/architectural 判断のため ADR 起票を推奨。
