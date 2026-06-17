Document ID: ADR-LGX-012

# ADR-LGX-012: 新リポジトリ再開ブートストラップ（legixy.old.p1 からの検証済み状態の移設）

**ステータス**: proposed（AI 起案・人間査読待ち）
**起票日**: 2026-06-12
**承認日**: —
**承認者**: —
**対象**: プロジェクト全体（SPEC-LGX-001〜010、DevProc_V4.1 パイプライン状態）

## 1. 文脈（Context）

- 新リポジトリ `legixy` は当初 `docs/specs/`（SPEC-LGX-001〜010、Approved 版）のみを保持して開始された。SPEC 群は前身リポジトリ `legixy.old.p1` の同名ファイルと**内容完全一致**（SUPP-LGX-000 §1 で diff 照合済）。
- 前身リポジトリでは DevProc_V4.1 のパイプラインが「前段ループ完了（FCR×11 全 ACCEPTED）→ TP[SPEC]×10 全 green → GAP×61 全 closed → UC-LGX-001〜011 作成済」まで進行し、ICONIX 層（RBA 以降）は未着手だった。
- ハードルール 9 の機械ゲート（`scripts/trace-check.sh` [4/5]）は SPEC ごとに ACCEPTED な FCR の実体を要求する。SPEC 本文へ「前段スキップ」注記を書く代替手段はハードルール 1（SPEC 変更は人間承認）により AI 単独では使えない。
- 実装に不足する情報は `docs/specs-supplement/`（SUPP-LGX-000〜010、非正準・人間査読待ち）として整理済み。

## 2. 検討した選択肢（Options）

### 選択肢 A: 前身リポジトリの検証済み成果物を移設して再開（採用）

SUPP-LGX-000 §3 の持ち込み優先順位に従い、SPEC 層・UC 層の成果物と検証エビデンスを無改変で移設する。前段ループ・SPEC レベル TDD ループを再実行しない（同一内容の SPEC に対する完了済みループの再実行は工数のみ増やし、人間承認履歴を再現できない）。

### 選択肢 B: 前段ループを新リポジトリで再実行

成果物がリポジトリ内で自己完結する利点はあるが、FCR の ACCEPTED は人間承認を含むため AI 単独で再現できず、完了済みの人間判断を形式的に再要求することになる。棄却。

### 選択肢 C: ADR スキップ宣言（03a §11）で前段ゲートを通す

SPEC 本文への「**前段スキップ**: ADR-...」追記が必要であり、ハードルール 1 に抵触（人間承認が先に必要）。エビデンス実体が存在するのにスキップ扱いにするのは記録としても不正確。棄却。

## 3. 判断（Decision）

選択肢 A を採用。2026-06-12 に以下を実施した:

1. **git 初期化 + DevProc 標準ディレクトリツリー作成**（`bootstrap/init-tree.sh` 相当を手動実行）。
2. **プロセスインフラ移設**: `.trace-engine.toml`、`CLAUDE.md`（Author モード）、`scripts/*.sh`、`.claude/`（settings / commands / agents）、`.git/hooks/pre-commit`、ONNX モデル（`models/`、ハードリンク複製）、engine 状態 DB（`.trace-engine/engine.db`）。
3. **成果物移設**（無改変、SUPP-LGX-000 §3 準拠）: 基礎文書 4（LEGIXY-SPEC-001 / LGX-EXT-001 / LGX-EXT-002 / LGX-COMPAT-001）、NFR×1、UC×11、ADR×11、GAP×61、SPP×11、QSET×11、FCR×11、TP×10、spec-change-proposals×2、`docs/traceability/graph.toml` + `matrix.md`、観点ベース×2。
4. **VAL-LX-001（前身名義の外部照合記録）**は typecode ディレクトリ外の `docs/specs-supplement/references/` に配置。VAL-LGX-001 としての改名・正準化は人間承認待ち（SUPP-LGX-000 §2）。
5. **`docs/DevProc_V4` シンボリックリンクを追加**（→ `DevProc_V4.1`）。CLAUDE.md の参照パスを充足する。既存の `docs/DevPorc`（綴り誤りだが開発者が作成）は温存。
6. **`bash scripts/trace-check.sh` 全 5 ゲート PASS を確認**（ERROR 0 / FCR ACCEPTED=11 / TP green=10 / GAP closed=61 / レイヤ汚染 0）。

## 4. 帰結（Consequences）

- パイプライン現在位置: **UC 層まで GREEN**。次フェーズは UC レベルの仕様レベル TDD ループ（TP[UC] ⇄ GAP[UC]）、その GREEN 後に ICONIX 抽象層（RBA/SEQA、ハードルール 11 により AI 自律実行域）。
- `docs/specs-supplement/` は**非正準の参考資料**であり traceability グラフには登録しない（ハードルール 4: 新 typecode は `.trace-engine.toml` 更新が先。SUPP の typecode 化は不要と判断 — グラフ検証の対象にする必然性がなく、SPEC との優先関係は各 SUPP 冒頭の宣言で足りる）。
- 下流着手前に SUPP-LGX-001〜010 の [要決定] 86 件（特に DB パス系統・SPEC/v3 実装乖離・環境変数名）の人間裁定が必要。これらは SPEC 改訂（人間承認 + 必要に応じ ADR）に波及し得る。
- 前身リポジトリの `.devproc/`（VERDICT ログ等のランタイム状態）と testbed-logs は移設しない（新リポジトリで新規に蓄積する）。
- engine 状態 DB（`.trace-engine/engine.db`）は embedding キャッシュ・ドリフトベースラインの継続性のため移設した。文書内容は同一のため整合する。再生成可能であり、不整合が出た場合は削除して `traceability-engine embed` で再構築してよい。
