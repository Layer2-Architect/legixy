Document ID: UC-LGX-004

# UC-LGX-004: 粒度制御付きコンテキスト解決

## 概要

compile_context に `--granularity subnode` を指定し、上流成果物をサブノード単位で取得する。トークン消費の削減が目的。

## アクター

- Claude Code（MCP 経由）
- 開発者（CLI 直接実行）

## 事前条件

- UC-LGX-002 の事前条件を満たす
- 上流成果物にサブノードが存在する（自動抽出 or 明示定義）

## 基本フロー

1. アクターが `legixy context <files> --granularity subnode` を実行する
2. UC-LGX-002 の基本フロー 2〜5 と同様に上流成果物を解決する
3. 上流成果物がサブノードを持つ場合:
   a. 対象ファイルからサブノードへのエッジを辿り、関連サブノードを特定する
   b. 各サブノードの本文（該当セクションのみ）を抽出する
   c. ドリフトスコア（エッジごと）を付与する
4. 結果の UpstreamArtifact に subnode_id, anchor, content, drift_score を含めて返却する

## 代替フロー

- 1a. `--granularity document`（デフォルト）の場合、UC-LGX-002 と同一の動作をする
- 3a. サブノードが存在しない上流成果物は、ドキュメント全体として返却する（fallback、Phase 2 Block B 後も維持）
- 4-A. （Phase 2 Block B、SPEC-LGX-003.REQ.15）`--outline-only` が指定された場合、各サブノード artifact の body は当該サブノードの anchor 文字列のみ（本文省略）となる
- 4-B. （Phase 2 Block B、SPEC-LGX-003.REQ.16）`--sections <ids>` が指定された場合、コンマ区切りで指定したサブノード ID と一致するサブノードのみが upstream に含まれる。指定 ID が graph に存在しない場合は単に除外する（エラーにしない）
- 4-C. （Phase 2 Block B、SPEC-LGX-003.REQ.17）`--depth N` が指定された場合、上流走査を N 階層に制限する。サブノード展開は depth_limit 通過後の親ドキュメントに対して実行される
- 4-D. （Phase 2 Block B、観察事項 1 解消）`--granularity subnode` 指定時、親ドキュメント全文ではなく **子サブノード（h2/h3 自動抽出）を個別 UpstreamArtifact として展開**する。各サブノードは `content_range` で切り出した部分テキストのみを body に含み、`Document ID:` 行・ヘッダ表・変更履歴等のテンプレ部分を含まない

## 事後条件

- UC-LGX-002 の事後条件と同じ（決定論保証、監査ログ）
- サブノードモードでは、ドキュメント全体ではなく関連セクションのみが返却される
- Phase 2 Block B 以降、`--granularity subnode` は親ドキュメントを返さず、子サブノードを個別 artifact に展開する（サブノードが存在しない場合のみ親全文へ fallback）

## 関連要求

- SPEC-LGX-003.REQ.03（粒度制御、`document` / `subnode`）
- SPEC-LGX-003.REQ.08（サブノード親への解決）
- SPEC-LGX-003.REQ.15（outline_only 出力、Phase 2 Block B）
- SPEC-LGX-003.REQ.16（sections フィルタ、Phase 2 Block B）
- SPEC-LGX-003.REQ.17（depth_limit、Phase 2 Block B）

## 関連不変条件

- CTX-INV-1: 決定論保証
- MCP-INV-1: Agent Surface 限定（新ツール追加なし）
- MCP-INV-2: 忠実な転送
