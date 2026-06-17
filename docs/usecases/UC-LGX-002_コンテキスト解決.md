Document ID: UC-LGX-002

# UC-LGX-002: コンテキスト解決（compile_context）

## 概要

Claude Code（MCP 経由）または開発者が、編集対象ファイルに対して参照すべき上流成果物を取得する。

## アクター

- Claude Code（MCP サーバー経由）
- 開発者（CLI 直接実行）

## 事前条件

- `.legixy.toml` と `graph.toml` が存在する
- 対象ファイルが graph.toml 内のノードに対応している

## 基本フロー

1. アクターが `legixy context <files> [--command <intent>]` を実行する
2. システムがファイルパスから対応する成果物 ID を逆引きする（resolver）
3. システムが成果物 ID から有向グラフを逆方向に走査し、上流成果物を収集する
4. システムがレイヤールールに基づくガイドライン文書を解決する
5. システムがカスタムエッジに基づく追加文書を解決する
6. システムが結果を ContextResult として返却する:
   - targets: 解決されたファイルと成果物 ID
   - upstream: 上流成果物一覧（chain_distance 順）
   - layer_documents: レイヤーガイドライン
   - custom_documents: カスタムエッジ文書
7. システムが context_log に監査ログを記録する

## 代替フロー

- 2a. ファイルがどのノードにも対応しない場合、targets の artifact_id を null として返す
- 3a. 上流成果物が存在しない場合、upstream を空配列として返す
- 4-A. （Phase 2 Block B、SPEC-LGX-003.REQ.15）`--outline-only` が指定された場合、各 upstream artifact の本文を h1〜h3 見出しの階層リストに置換して返却する。トークン消費削減用途
- 4-B. （Phase 2 Block B、SPEC-LGX-003.REQ.16）`--sections <ids>` が指定された場合、`--granularity subnode` 経路で展開される子サブノードのうち指定 ID と一致するもののみを upstream に含める
- 4-C. （Phase 2 Block B、SPEC-LGX-003.REQ.17）`--depth N` が指定された場合、上流走査を N 階層に制限する（N=1 で直接の親のみ）

## 事後条件

- 同じ入力に対して常に同じ結果が返される（CTX-INV-1: 決定論保証）
- context_log に記録が追加される（MCP-INV-4: 監査ログ完全性）

## 関連要求

- SPEC-LGX-003.REQ.01〜09（既存）
- SPEC-LGX-003.REQ.15（outline_only 出力、Phase 2 Block B）
- SPEC-LGX-003.REQ.16（sections フィルタ、Phase 2 Block B）
- SPEC-LGX-003.REQ.17（depth_limit、Phase 2 Block B）

## 関連不変条件

- CTX-INV-1: 決定論保証
- CTX-INV-2: グラフ整合性
- CTX-INV-3: カスタムエッジ独立性
- MCP-INV-2: 忠実な転送
- MCP-INV-4: 監査ログ完全性
