Document ID: GAP-LGX-154

# GAP-LGX-154: v0.1.0 か legixy かを判定する具体的根拠（スキーマ特徴）が未定義

**親 TP**: TP-LGX-008
**観点出典**: TP-LGX-008 §2.6 観点 V-2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

REQ.09 は `.legixy.toml` と engine.db の双方からバージョンを検出し不整合なら Error とするが、**そもそも各ファイルから「バージョン」をどのフィールド/スキーマ特徴で読むか**（engine.db の `PRAGMA user_version` か専用 version テーブルか、`.legixy.toml` の `[graph]` 有無か明示 version キーか）が定義されていない。これが定まらないと検出ロジック自体が書けない。

## 2. 現状の SPEC / UC

SPEC-LGX-008 §3 REQ.09 で **「双方からバージョンを検出し、不整合があれば Error」** に触れているが、**検出に用いる具体的フィールド/特徴** は未定義。REQ.01 の「v0.1.0 フォーマットの engine.db を検出」も検出方法の具体が無い。

## 3. 期待される情報

SPEC または UC に追加されるべき記述:

- engine.db のバージョン判定根拠（`PRAGMA user_version` / version テーブル / 特定カラムの有無 例: `context_log.granularity`）
- `.legixy.toml` / `.trace-engine.toml` のバージョン判定根拠（`[graph]` 有無 / `[matrix]` のみ / 明示 version キー）
- バージョン情報が欠落している（古すぎて version マーカが無い）場合の扱い

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流 TS / TC: バージョン検出・不整合検出テストの期待値が書けない
- 他の GAP との依存: GAP-LGX-147（migrate 済判定）, GAP-LGX-149（auto 初回識別）と同根の判定基盤

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-008 v0.6.0（人間承認 2026-06-10）: REQ.09 を PRAGMA user_version 一次根拠（0 なら追加カラム二次判定）+ [graph] セクション判定 + マーカ欠落 v0.1.0 + 矛盾 Error に確定。v3 実測で user_version=3 の実使用を確認（initializer.rs:137, autodetect.rs:99）し差替不要。旧 GAP-147/149 統合。

## 6. 関連 ADR

該当時に起票。
