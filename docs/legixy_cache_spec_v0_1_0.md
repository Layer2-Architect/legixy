# legixy 機能拡張仕様書: Prompt Caching 最適化と MCP Result Persistence

| 項目 | 内容 |
|------|------|
| Document ID | LGX-EXT-002 |
| Version | 0.1.0 |
| Status | Draft |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | LEGIXY-SPEC-001 (legixy_foundational_spec.md) |
| 関連文書 | LGX-EXT-001 (サブノード化仕様) |

---

## 目次

1. [本文書の位置づけ](#1-本文書の位置づけ)
2. [背景と目的](#2-背景と目的)
3. [Prompt Caching 最適化](#3-prompt-caching-最適化)
4. [MCP Result Persistence](#4-mcp-result-persistence)
5. [不変条件への影響と追加](#5-不変条件への影響と追加)
6. [段階的導入の方針](#6-段階的導入の方針)
7. [制約と前提](#7-制約と前提)
8. [設計判断の確定事項](#8-設計判断の確定事項)

---

## 変更履歴

| Version | Date | 主な変更内容 |
|---------|------|------------|
| 0.1.0 | 2026-04-17 | 初版 |

---

## 1. 本文書の位置づけ

本文書は legixy(有向グラフ主体のトレーサビリティエンジン)における機能拡張仕様書である。以下の二つの機能を定義する。

- Prompt Caching の活用を前提とした `compile_context` 返却内容の決定論的整列
- Claude Code v2.1.91 以降が提供する MCP Result Persistence への対応

本文書は LEGIXY-SPEC-001 (legixy_foundational_spec.md) の補完文書であり、Phase 1 開発における設計判断の基礎となる。本文書の内容は、将来的に LEGIXY-SPEC-001 の §4(エンジン機能)、§10(不変条件)に統合される可能性がある。

### 1.1 読者

本文書の想定読者は以下である。

- legixy 開発者(仕様作成者自身)
- AI エージェント(開発プロセスにおける実装担当)

### 1.2 スコープ

本文書がカバーする範囲は以下である。

- `compile_context` 返却内容の決定論的整列規則
- Anthropic Prompt Caching を前提とした出力レイアウト
- MCP サーバーにおける `_meta["anthropic/maxResultSizeChars"]` の設定
- 大規模返却結果に対するサイズ超過時の挙動
- 既存不変条件への影響評価
- Phase 1 における導入スコープ

本文書がカバーしない範囲は以下である。

- サブノード化の詳細仕様(LGX-EXT-001 で扱う)
- Claude Code 側のキャッシュ制御戦略(legixy スコープ外、前提として扱う)
- Anthropic API のキャッシュ内部実装(前提として扱う)
- セッション横断の独自キャッシュ機構(STATE-INV-1 に反するため非対象)

### 1.3 LGX-EXT-001 との関係

本文書は LGX-EXT-001(サブノード化)と独立に適用可能である。ただし、サブノード化と組み合わせた場合に最大の効果を発揮する。

両者の関係は以下。

| 拡張 | 主目的 | 効果の性質 |
|------|--------|----------|
| LGX-EXT-001(サブノード化) | 送信量そのものの削減 | 絶対量の削減 |
| LGX-EXT-002(本文書) | 再送信時の相対コスト削減 | 再利用効率の向上 |

サブノード化で「何を送るか」を絞り、本拡張で「送ったものを効率よく再利用できるようにする」という関係にある。

---

## 2. 背景と目的

### 2.1 現状の課題

legixy の Phase 1 設計(LGX-EXT-001 含む)は、`compile_context` が返すコンテキストの**量**を削減する方向で進められている。量の削減は本質的な対策だが、以下の課題が残存する。

**課題 1: 返却内容の再利用機会の未活用**

Claude Code のセッション内で `compile_context` が複数回呼び出される場合、返却内容のうち Layer Guidelines や Additional Guidelines は同一であることが多い。Anthropic の Prompt Caching 機構は、反復的なコンテキストを低コストで扱う仕組みを提供するが、返却内容の順序が呼び出しごとに変動していると、キャッシュヒット条件(プロンプト先頭からの完全一致)を満たせない。

**課題 2: 大規模返却のセッショントークン圧迫**

`compile_context --granularity document` かつ上流成果物が多い場合、返却内容が大きくなる。この大きな内容が Claude Code の会話履歴に蓄積されると、後続メッセージすべてに同じトークンが含まれる形でコンテキストを消費する。

**課題 3: 将来のキャッシュ関連機能との整合**

Phase 1 で導入される Contextual Retrieval(LGX-EXT-001 Section 5.8)はキャッシュ機構との親和性が高い処理である。キャッシュを前提とした出力仕様が Phase 1 で規定されていないと、Phase 2 以降で Contextual Retrieval を運用化する際に再設計が必要になる。

### 2.2 本拡張の目的

本拡張は以下の目的を達成する。

**目的 1: Prompt Caching ヒット条件の成立**

`compile_context` の返却内容を決定論的な順序で整列し、安定部分を先頭に配置する。これにより、Claude Code および Anthropic 側のキャッシュ機構がヒットする条件を満たす。

**目的 2: 大規模返却のハンドリング**

Claude Code v2.1.91 以降の MCP Result Persistence 機能を活用し、大きな返却内容がセッションのトークン予算を圧迫しない構造を整える。

**目的 3: Phase 2 以降への設計余地の確保**

Contextual Retrieval の運用化、investigate/impact 結果への同様の適用など、将来のキャッシュ関連機能が同じ設計思想の上に乗る基盤を確立する。

### 2.3 本拡張の非目的

以下は本拡張の目的ではない。

- Claude Code 側のキャッシュ制御挙動の直接制御(Claude Code の責務)
- Anthropic API への `cache_control` パラメータの明示設定(Claude Code の責務)
- legixy コンポーネント内での独自永続キャッシュの構築(STATE-INV-1 に反する)

---

## 3. Prompt Caching 最適化

### 3.1 Anthropic Prompt Caching の前提

Anthropic の Prompt Caching は以下の性質を持つ。

- プロンプトの先頭から**完全一致**する部分のみキャッシュとして再利用される
- キャッシュされた入力トークンは ITPM 制限にカウントされない
- キャッシュ書き込み時は通常の 1.25 倍のコスト、キャッシュ読み取り時は約 0.1 倍のコスト

Claude Code v2.1 以降は CLAUDE.md とエージェント定義を自動的にキャッシュ対象として扱う。MCP ツールの返却内容も会話履歴の一部としてキャッシュの恩恵を受けうるが、その条件は**返却内容が呼び出し間で安定していること**である。

### 3.2 compile_context 返却内容の整列原則

`compile_context` の返却内容は、以下の順序で構成する。この順序は「安定度が高いものを先、低いものを後」の方針に従う。

```
1. Layer Guidelines          ← プロジェクト全体で共通、最も安定
2. Additional Guidelines     ← 対象レイヤーで共通、やや安定
3. キャッシュブレーク点マーカ ← 安定部分と変動部分の境界
4. Upstream Artifacts        ← タスクごとに変動、決定論的順序で整列
5. Target Node Metadata      ← 対象ノードに固有、最も変動
```

この順序により、同一セッション内で `compile_context` が複数回呼び出された場合、異なるファイルに対する呼び出しでも上位セクションまで共通となり、キャッシュヒットの対象となる可能性が高まる。

### 3.3 各セクションの決定論的順序

**Layer Guidelines**

Layer Guidelines に該当するファイル群は、グラフ定義での宣言順序ではなく、以下の決定論的順序で整列する。

- ファイルパスの辞書順昇順(バイト単位比較)

**Additional Guidelines**

Additional Guidelines も同様にファイルパスの辞書順昇順で整列する。

**Upstream Artifacts**

上流成果物の列挙順序は以下で決定する。

- `--granularity document` の場合: ノード ID の辞書順昇順
- `--granularity subnode` の場合: 親ドキュメント ID の辞書順昇順でグループ化し、同一ドキュメント内ではアンカー出現順(ドキュメント内の物理的な位置順)で整列

「グラフ探索の発見順」「エッジのスコア順」は入力やスコア計算結果により順序が変動するため、キャッシュフレンドリーではない。これらの情報が必要な場合は Target Node Metadata に別途含める。

**Target Node Metadata**

対象ノードのメタデータ(ドリフトスコア、最終更新時刻、関連する Observation、探索発見順等)は、このセクションに集約する。変動が大きいため、キャッシュブレーク点より後ろに配置される。

### 3.4 キャッシュブレーク点マーカ

返却内容内に、安定部分と変動部分を区切る**論理的マーカ**を挿入する。マーカは以下の形式である。

```
<!-- cache-breakpoint: stable-end -->
```

このマーカは Additional Guidelines の末尾、Upstream Artifacts の先頭の間に配置される。

**マーカの役割**

マーカは Claude Code 側または上位エージェントがキャッシュ制御点として解釈するためのヒントである。Claude Code 側での解釈が実装されていなくても、HTML コメントとして無視されるため出力の表示や意味には影響しない。

Phase 1 ではマーカの挿入のみを規定し、Claude Code 側での解釈挙動は前提としない。将来の活用余地の確保を目的とする。

### 3.5 決定論性の保証

本拡張は LEGIXY-SPEC-001 の CTX-INV-1(決定論保証)を返却内容の**バイト列一致**まで強化する。同じ入力に対して、返却内容は順序を含めて同一のバイト列となる。

この保証は以下で検証される。

- 単体テスト: 同じグラフ定義、同じ engine.db 状態、同じ引数に対する複数回の呼び出しで、返却バイト列が一致すること
- 統合テスト: `--granularity document` および `--granularity subnode` の両方で順序保証が成立すること

---

## 4. MCP Result Persistence

### 4.1 Claude Code v2.1.91 の機能概要

Claude Code v2.1.91 以降は、MCP ツールの返却結果を**ディスクに永続化**する機能を提供する。MCP サーバーが返却メタデータに `_meta["anthropic/maxResultSizeChars"]` を含めることで、以下の挙動が有効となる。

- 返却結果のうち指定サイズ(最大 500,000 文字)までをディスクに保存
- Claude Code の会話履歴には参照とサマリのみが残り、本体は必要時にディスクから読み出される
- 同一セッション内の後続メッセージで蓄積されるトークン量が削減される

本機能は Claude Code 側で管理されるため、legixy コンポーネントは永続状態を保持しない(STATE-INV-1 に整合)。

### 4.2 MCP サーバー側の実装

legixy の MCP サーバー(TypeScript 実装)は、以下の変更を受ける。

**返却メタデータの設定**

`compile_context` の返却ペイロードの `_meta` フィールドに以下を設定する。

```typescript
{
  content: [...],  // Rust CLI から受け取った返却本文
  _meta: {
    "anthropic/maxResultSizeChars": 500000
  }
}
```

この設定により Claude Code 側で自動的に永続化が有効となる。

**適用対象ツール**

Phase 1 では以下のツールに適用する。

| ツール | 適用 | 根拠 |
|--------|------|------|
| `compile_context` | 適用 | 返却サイズが大きくなる主要ツール |
| `observe` | 非適用 | 返却は確認メッセージ程度のため小規模 |
| `get_compile_audit` | 適用 | 監査ログの返却でサイズが大きくなりうる |

**実装変更範囲**

- MCP サーバーの返却構築処理に `_meta` フィールドの付与を追加
- Rust CLI 側の変更は不要

### 4.3 サイズ超過時の挙動

**閾値の設定**

Phase 1 ではすべての対象ツールで `maxResultSizeChars: 500000` を一律に設定する。呼び出しごとの動的判定は行わない。

根拠: Claude Code 側が実際の返却サイズに応じて永続化要否を判定するため、MCP サーバー側の事前判定は不要である。

**返却サイズが 500,000 文字を超える場合**

Rust CLI が生成した返却本文が 500,000 文字を超える場合、`compile_context` は**エラーを返却する**。本文の切り捨てや要約は行わない。

エラー返却形式:

```
Error: compile_context result exceeds 500,000 characters.
Current size: <N> characters.
Suggested action:
  - Try --granularity subnode for finer-grained retrieval.
  - Narrow the target scope.
```

根拠: 切り捨てや自動要約は決定論性(CACHE-INV-1)の検証を困難にし、情報欠落時の挙動を予測しにくくする。明示的なエラーとして粒度制御をユーザーに委ねる方が、設計原則(P-02: 判断は人間に委ねる)と整合する。

### 4.4 MCP-INV-2 との整合

MCP-INV-2(忠実な転送: Rust CLI 出力のフィルタリング・省略なし)との関係を明確にする。

`_meta` フィールドの付与は Rust CLI の出力本文(`content` フィールドの内容)に対する変更ではなく、MCP プロトコルレベルでのメタデータ追加である。本文は従来通り改変なく転送される。したがって MCP-INV-2 は維持される。

### 4.5 Claude Code バージョン非依存性

`_meta["anthropic/maxResultSizeChars"]` は Claude Code v2.1.91 以降で解釈される。それ以前のバージョンでは未知のメタデータとして無視される。

したがって本機能は Claude Code のバージョンに依存しない形で実装可能である。古いバージョンでは永続化の恩恵が得られないだけで、動作そのものには影響しない。

---

## 5. 不変条件への影響と追加

### 5.1 既存不変条件への影響

LEGIXY-SPEC-001 §10 および LGX-EXT-001 Section 7.2 で定義された既存の不変条件について、本拡張の影響を評価する。

**CTX-INV-1: 決定論保証**

影響: あり(強化方向)。本拡張は返却内容の順序、およびバイト列レベルでの一致を要求する。

対応: Section 3.5 で順序保証を規定。CACHE-INV-1 として明示化。

**CTX-INV-2: グラフ整合性**

影響: なし。返却順序の変更はグラフ定義との整合性に影響しない。

**CTX-INV-3: カスタムエッジ独立性**

影響: なし。

**CTX-INV-4: DAG 制約**

影響: なし。

**FB-INV-1 ~ FB-INV-5: フィードバックループ関連**

影響: なし。Observation や Proposal の生成ロジックは変更しない。

**SCORE-INV-1, SCORE-INV-2: スコア管理**

影響: なし。スコア計算自体は変更しない。スコア情報は Target Node Metadata に集約される。

**MCP-INV-1: Agent Surface 限定**

影響: なし。新 MCP ツールの追加はない。

**MCP-INV-2: 忠実な転送**

影響: あり(解釈の明確化)。`_meta` フィールドの付与が本不変条件に抵触しないことを Section 4.4 で明確化。

**MCP-INV-3: Observation 重複排除**

影響: なし。

**MCP-INV-4: 監査ログ完全性**

影響: 小。サイズ超過時のエラーもログ対象とする。

対応: 監査ログ記録仕様にエラーケースを追加する(実装詳細)。

**STATE-INV-1, STATE-INV-2: 状態管理**

影響: なし。永続化は Claude Code 側の機能であり、legixy のコンポーネントは状態を持たない。

**SUBNODE-INV-1 ~ SUBNODE-INV-6: サブノード関連**

影響: なし。サブノード関連の不変条件はそのまま維持される。本拡張は、サブノードを含む返却にも同じ整列規則を適用する形で整合する。

### 5.2 追加すべき不変条件

本拡張に伴い、以下の不変条件を追加する。

**CACHE-INV-1: 返却内容の決定論的一致**

`compile_context` の返却内容は、同じ引数・同じグラフ定義・同じ engine.db 状態に対して、バイト単位で同一の結果を返す。順序、区切り文字、空白を含めて決定論的である。

**CACHE-INV-2: セクション配置順序の固定性**

`compile_context` の返却内容は以下の順序で構成される。

1. Layer Guidelines
2. Additional Guidelines
3. キャッシュブレーク点マーカ
4. Upstream Artifacts
5. Target Node Metadata

この順序は `--granularity` の値に依らず一貫する。

**CACHE-INV-3: 大規模返却時のエラー伝達**

返却本文が 500,000 文字を超える場合、`compile_context` は切り捨てや要約を行わず、明示的なエラーとして報告する。

**CACHE-INV-4: メタデータ付与の忠実性**

MCP サーバーが設定する `_meta` フィールドは、Rust CLI の出力本文に影響しない。本文の内容は MCP-INV-2 に従って忠実に転送される。

---

## 6. 段階的導入の方針

### 6.1 Phase 1 への組み込み

本拡張は legixy の Phase 1 に組み込む。LGX-EXT-001 Section 8.1 で既に規定された Phase 1 必須スコープに、以下を追加する。

**Phase 1 必須スコープ追加項目**

| # | 項目 | 担当レイヤー |
|---|------|------------|
| 1 | 返却セクションの配置順序実装 | Rust CLI |
| 2 | 各セクション内の決定論的整列実装 | Rust CLI |
| 3 | キャッシュブレーク点マーカの出力 | Rust CLI |
| 4 | 返却サイズ計測と 500,000 文字超過時のエラー処理 | Rust CLI |
| 5 | `_meta["anthropic/maxResultSizeChars"]` の返却への付与 | MCP サーバー |
| 6 | `_meta` 付与後も本文を改変しないことの検証 | MCP サーバー |
| 7 | CACHE-INV-1 〜 CACHE-INV-4 の実装 | Rust CLI / MCP サーバー |
| 8 | 決定論性の単体テスト(複数回呼び出しでバイト一致) | Rust CLI |
| 9 | サブノード化有無の両モードでの統合テスト | 統合 |

**Phase 2 以降に振り分ける項目**

- キャッシュヒット率の計測機構
- `maxResultSizeChars` の動的調整
- キャッシュブレーク点マーカの複数箇所配置
- investigate/impact 返却への同様の整列適用
- Contextual Retrieval キャッシュの高度化

### 6.2 LGX-EXT-001 との実装順序

本拡張と LGX-EXT-001 は独立に実装可能だが、以下の順序を推奨する。

1. LGX-EXT-001 の基本実装(サブノード化の骨格、`--granularity` 対応)
2. 本拡張の実装(返却整列、MCP メタデータ付与)
3. LGX-EXT-001 の Contextual Retrieval 実装(デフォルト無効)

この順序により、サブノード化された出力に対して本拡張の整列規則が適用された状態で Contextual Retrieval を実装することになり、将来のキャッシュ基盤が整った上での実装となる。

### 6.3 ドッグフーディングでの検証

開発環境での実運用(ドッグフーディング)で、本拡張の効果を検証する。検証項目は以下。

- 同一セッション内での `compile_context` 反復呼び出し時の体感的なキャッシュ効果
- 大規模返却時の MCP Result Persistence による後続トークン消費の削減
- 決定論性の実運用における維持確認
- サイズ超過エラー発生時のユーザー体験

---

## 7. 制約と前提

### 7.1 前提条件

本拡張の実装には以下の前提が必要である。

- legixy が有向グラフ主体の実装を完了している(Phase 1 の基礎部分)
- MCP サーバー(TypeScript 実装)のソースコードに変更を加えられる段階である
- Claude Code は v2.1.91 以降での運用を想定する(古いバージョンでの動作は Section 4.5 参照)

### 7.2 Phase 1 での制約

Phase 1 時点では以下の制約を受け入れる。

- `maxResultSizeChars` は 500,000 文字で固定(動的調整は Phase 2 以降)
- キャッシュブレーク点マーカは 1 箇所のみ(複数の粒度ブレーク点は Phase 2 以降)
- キャッシュヒット率の定量計測機構は実装しない(効果検証はドッグフーディングの定性評価)
- マーカの Claude Code 側での解釈実装は前提としない

### 7.3 既知の限界

- Claude Code および Anthropic API のキャッシュ挙動は legixy の制御範囲外であり、仕様変更により本拡張の効果が変動しうる
- キャッシュブレーク点マーカは、現時点では Claude Code 側での解釈実装が確認されていない。効果は将来の実装または上位エージェントの対応に依存する
- 返却整列の決定論性は Rust CLI と MCP サーバーの両実装で共同して保証する必要がある。どちらか一方での違反は CACHE-INV-1 の破綻を招く
- 500,000 文字超過時にエラーとする方針は、極端に大規模な上流を持つノードでの `--granularity document` 利用を制約する。この場合は `--granularity subnode` の使用が前提となる

---

## 8. 設計判断の確定事項

### 8.1 返却セクションの配置順序

Section 3.2 の通り、安定度が高いものから低いものへの固定順序を採用する。

根拠: Prompt Caching は先頭からの完全一致でヒットするため、安定部分を前に置くことが本質的な要件である。`--granularity` の値によって順序が変動すると、モード切り替え時にキャッシュが無効化されるため、一貫した順序を採用する。

### 8.2 キャッシュブレーク点マーカの形式

HTML コメント形式(`<!-- cache-breakpoint: stable-end -->`)を採用する。

根拠: Markdown として表示時に無視され、機械的検出も容易。Claude Code 側での解釈未実装の状態でも出力に支障がない。将来的な拡張時に属性追加(例: `stage="guidelines-end"`)が可能な形式である。

### 8.3 大規模返却時の挙動

切り捨て・要約は行わず、エラー報告方式を採用する。

根拠: 切り捨ては決定論性検証を複雑にし、情報欠落の挙動予測を困難にする。`--granularity subnode` への切り替え案内により、粒度制御をユーザーの明示判断に委ねる方式が LEGIXY-SPEC-001 P-02(判断は人間に委ねる)と整合する。

### 8.4 maxResultSizeChars の値

Phase 1 では 500,000(最大値)固定とする。

根拠: Claude Code 側が実返却サイズに応じて永続化要否を判定するため、MCP サーバー側の動的調整は不要。ドッグフーディングで運用パターンが見えた段階で Phase 2 以降に調整する。

### 8.5 Rust CLI と MCP サーバーの責務分担

- 返却本文の生成と順序決定: Rust CLI
- `_meta` フィールドの付与: MCP サーバー
- サイズ超過判定とエラー生成: Rust CLI

根拠: MCP-INV-2 に従い、MCP サーバーは本文を改変しない。本文に関わる判断(順序、サイズ判定)は Rust CLI の責務とし、プロトコルレベルのメタデータ付与のみを MCP サーバーの責務とする。

### 8.6 observe ツールへの適用判断

Phase 1 では `observe` への `_meta` 付与は行わない。

根拠: `observe` の返却は処理結果の確認メッセージ程度であり、永続化の恩恵が薄い。将来 `observe` の返却が肥大化する機能拡張が行われた場合に再検討する。

---

## 付録 A: 用語集

| 用語 | 定義 |
|------|------|
| Prompt Caching | Anthropic API が提供する、プロンプト先頭部分のキャッシュ機構 |
| ITPM | Input Tokens Per Minute。1 分あたりの入力トークン数制限 |
| キャッシュブレーク点 | 返却内容の中で「安定部分」と「変動部分」を区切る論理的位置 |
| キャッシュブレーク点マーカ | キャッシュブレーク点を表す HTML コメント形式のマーカ |
| MCP Result Persistence | Claude Code v2.1.91 以降の、MCP ツール返却結果をディスク永続化する機能 |
| Layer Guidelines | プロジェクトの各レイヤー(DD、UC 等)に共通するガイドライン |
| Additional Guidelines | 対象レイヤー固有の補足的ガイドライン |
| Upstream Artifacts | 対象ノードに対する上流成果物群 |
| Target Node Metadata | 対象ノード自身のメタデータ(スコア、更新時刻、Observation 等) |

---

## 付録 B: 参照文書

- LEGIXY-SPEC-001: legixy Foundational Specification(親文書)
- LGX-EXT-001: サブノード化によるコンテキスト粒度制御(関連拡張)

---

Document End

LGX-EXT-002 v0.1.0
