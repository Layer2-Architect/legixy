Document ID: ADR-LGX-016

# ADR-LGX-016: 環境変数命名規約 — `LGX_*` 正準 + 旧名フォールバック

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定、TRIAGE-2026-06-13 クラスタ C）
**対象**: SUPP-LGX-009 §2.3 / SUPP-LGX-010 D-2 / SUPP-LGX-006 2.1-c, 2.6-b

## 1. 文脈（Context）

環境変数・モデル配置・LLM キーの命名が未確定で三系統混在（TRIAGE §3 クラスタ C）:

- **MCP バイナリ解決**: `TRACEABILITY_ENGINE_BIN`（SPEC-LGX-009 REQ.08 本文）/ `LEXIGY_BIN` / `LEGIXY_BIN` が混在。
- **モデルディレクトリ解決**: `LGX_MODELS_DIR` / `TE_MODELS_DIR`（旧名）の解決順・フォールバック挙動が未確定。
- **LLM API キー**: ContextualRetrieval の具象クライアント用キー名が未定。

## 2. 判断（Decision）

人間裁定 2026-06-13。ADR-LGX-014（SPEC 準拠原則 + リブランド整合）に基づく:

- **命名規約**: `LGX_*` を正準とする。旧名（`TRACEABILITY_ENGINE_BIN` / `TE_MODELS_DIR` / `LEXIGY_*`）は**フォールバックとして併読**し、使用時は **stderr に Info で新名を案内**する（SPEC-LGX-010 既定の drift モデル解決と同パターン）。
  - MCP バイナリ解決: `LGX_BIN`（正準）＞ `TRACEABILITY_ENGINE_BIN`（旧名フォールバック）。SPEC-LGX-009 REQ.08 本文の `TRACEABILITY_ENGINE_BIN` 明記は SPEC 改訂候補として申し送り（人間承認時に新名へ）。
  - モデル解決順: `--models-dir` フラグ ＞ `LGX_MODELS_DIR` ＞ `TE_MODELS_DIR`（旧名・stderr Info 案内）＞ 設定ファイル（SPEC-LGX-010.REQ.03 既述と一致）。
- **モデル不在指定は即エラー（exit 1）**: 上位ソースで指定されたパスが不在のとき下位ソースへ**沈黙フォールバックしない**（SUPP-010 D-2）。誤ったモデルでの drift/embedding 算出を防ぐ。解決順は「未指定なら次へ」、「指定済みかつ不在ならエラー」。
- **LLM API キー**: ContextualRetrieval は `ANTHROPIC_API_KEY` を使用（SUPP-006 2.6-b。既定 LLM は最新 Claude 系を想定。具体モデル ID・エンドポイント・max_tokens は DD 凍結）。
- **モデル配布**（SUPP-006 2.1-c）: 実体は gitignore（`models/`）。再取得は `scripts/fetch-model.sh`（既存）。配置検証はファイル存在 + 出力 shape 検証（SUPP-006 2.1-e、DD）。

## 3. 結果（Consequences）

- TRIAGE クラスタ C 傘下 4 項目が一括解決。具体的な env 名文字列・解決順の最終形は DD で凍結（本 ADR の規約下）。
- 旧名フォールバック + Info 案内により v3 ユーザの既存設定が壊れない（後方互換）。
- **残存リスク**: 「指定済みパス不在 = 即エラー」と「未指定 = 次ソースへ」の境界判定を DD で厳密化（環境変数が空文字列のときの扱い等）。

## 4. 関連

- 統治: ADR-LGX-014（SPEC 準拠原則）
- 対象 SPEC: SPEC-LGX-009.REQ.08（env 名の申し送り）, SPEC-LGX-010.REQ.03（モデル解決順）
- トリアージ: TRIAGE-2026-06-13 §3 クラスタ C
