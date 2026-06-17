Document ID: UC-LGX-008

# UC-LGX-008: フィードバックループ（observation → proposal → approve/reject）

## 概要

検証結果や手動報告から Observation を生成し、Proposal（改善提案）に変換する。人間が承認・却下を判断する。

## アクター

- システム（自動 Observation 生成: `feedback` コマンド）
- Claude Code（手動 Observation 記録: `observe` コマンド、MCP 経由）
- 人間（Proposal の承認・却下: `approve` / `reject` コマンド）

## 事前条件

- engine.db が存在する

## 基本フロー

### Observation 生成

1. `feedback` コマンド: check 結果から自動で Observation を生成する
   - ChainIntegrity → chain_integrity カテゴリ
   - LinkCandidate → link_candidate カテゴリ
   - Drift → drift カテゴリ
   - OrphanFile → orphan_file カテゴリ
2. `observe` コマンド: 手動で Observation を記録する（MCP 経由）
3. 重複チェック: (category, related_ids) が一致する pending の Observation は生成しない（FB-INV-1）

### Proposal 生成

1. `analyze` コマンド: pending の Observation から Proposal を生成する
2. Pessimistic Claim パターン: pending → analyzing → proposed/skipped
3. カテゴリ別の変換:
   - chain_integrity → add_chain_entry
   - link_candidate → add_link
   - drift → update_doc
4. semantic_key で Proposal の重複を排除する（FB-INV-5）

### 承認・却下（人間のみ）

1. `proposals` コマンド: pending の Proposal 一覧を表示する
2. `approve <id>`: Proposal を承認する（原子的トランザクション: FB-INV-2）
3. `reject <id> --reason <reason>`: Proposal を却下する（理由必須）

## 代替フロー

- 1a. check 結果に該当カテゴリがない場合、Observation は生成されない
- 2a. analyze で処理中に失敗した場合、Observation を pending に戻す（claim release）

## 事後条件

- pending の Proposal は context 結果に影響しない（FB-INV-3）
- engine.db がなくてもグラフ上流は正常に返される（FB-INV-4）

## 関連不変条件

- FB-INV-1〜5（全て）
