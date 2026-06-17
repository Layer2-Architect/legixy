# SUPP-LGX-000: 参照文書インベントリ

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-000 |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |
| 調査対象 | legixy/docs/specs/ SPEC-LGX-001〜010（10 ファイル） |
| 探索範囲 | legixy.old.p1/、traceability-engine.v3.chg_to_lexigy/ |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**

調査方法: 新リポジトリの SPEC 全 10 ファイルを grep して外部文書 ID を抽出し、旧プロジェクト 2 箇所で実体ファイルを探索した。対応表の「確度」列は、ファイル先頭の Document ID ヘッダを実際に読んで一致を確認したものを「確認済」、ヘッダに該当 ID が無く内容・命名から対応を推定したものを「推定」と記す。

集計: 参照文書 ID 総数 **108**（SPEC-LGX-001〜010 自身の相互参照を除く）。所在確認 **104**、所在不明 **4**（うち 3 件は意図的な未生成/削除、1 件は前身文書のみ存在 — §2 参照）。

付随的発見: 新リポジトリの `docs/specs/` と旧プロジェクト `legixy.old.p1/docs/specs/` の SPEC 10 ファイルは **内容が完全一致**（diff 差分なし）。すなわち旧プロジェクトの参照構造がそのまま新リポジトリに持ち込まれている。

---

## §1 参照文書対応表

実体パスの基点: 特記なき限り `OLD = legixy.old.p1`。

### 1.1 上位仕様・互換契約（umbrella 層）

| 文書 ID | 参照元 SPEC | 実体パス | 確度 |
|---------|------------|----------|------|
| LEGIXY-SPEC-001 | 001〜007, 009 | OLD/docs/legixy_foundational_spec.md | 確認済（ヘッダ ID 一致, v1.0.0） |
| LGX-EXT-001 | 001〜009（全 SPEC 中最多 72 回参照） | OLD/docs/legixy_subnode_spec_v0.2.1.md | 確認済（ヘッダ ID 一致, v0.2.1） |
| LGX-EXT-002 | 001, 003, 009 | OLD/docs/legixy_cache_spec_v0_1_0.md | 確認済（ヘッダ ID 一致, v0.1.0） |
| LGX-COMPAT-001 | 001〜008, 010 | OLD/docs/legixy_cli_compat_reference.md | 確認済（ヘッダ ID 一致, v1.1.0, Status: Reference） |
| NFR-LGX-001 | 001〜010（全 SPEC） | OLD/docs/nfr/NFR-LGX-001_非機能要件.md | 確認済（ヘッダ ID 一致） |
| CLAUDE.md | 001, 003, 004, 006, 007, 008 | OLD/CLAUDE.md（リポジトリルート） | 確認済（DevProc_V4.1 Author モード規律 + 互換制約を記載） |
| VAL-LGX-001 | 001, 002, 004, 006, 008 | **所在不明**（§2 参照。前身 VAL-LX-001 のみ存在） | — |

### 1.2 ユースケース（UC-LGX、OLD/docs/usecases/）

| 文書 ID | 参照元 SPEC | 実体パス | 確度 |
|---------|------------|----------|------|
| UC-LGX-001 | 001, 002, 004, 010 | OLD/docs/usecases/UC-LGX-001_グラフ読み込みと検証.md | 確認済 |
| UC-LGX-002 | 003, 009 | OLD/docs/usecases/UC-LGX-002_コンテキスト解決.md | 確認済 |
| UC-LGX-003 | 002 | OLD/docs/usecases/UC-LGX-003_サブノード自動抽出.md | 確認済 |
| UC-LGX-004 | 003, 009 | OLD/docs/usecases/UC-LGX-004_粒度制御付きコンテキスト解決.md | 確認済 |
| UC-LGX-005 | 005 | OLD/docs/usecases/UC-LGX-005_逆方向探索.md | 確認済 |
| UC-LGX-006 | 005 | OLD/docs/usecases/UC-LGX-006_順方向探索.md | 確認済 |
| UC-LGX-007 | 006 | OLD/docs/usecases/UC-LGX-007_embedding生成とドリフト検出.md | 確認済 |
| UC-LGX-008 | 007, 009 | OLD/docs/usecases/UC-LGX-008_フィードバックループ.md | 確認済 |
| UC-LGX-009 | 008 | OLD/docs/usecases/UC-LGX-009_プロジェクト初期化とマイグレーション.md | 確認済 |
| UC-LGX-010 | 001, 010 | OLD/docs/usecases/UC-LGX-010_トレーサビリティ健全性監査.md | 確認済 |
| UC-LGX-011 | 010 | OLD/docs/usecases/UC-LGX-011_閾値キャリブレーション.md | 確認済 |
| UC-LGX-012 | 001, 010 | **所在不明（意図的: 予約済・未生成）** — §2 | — |
| UC-LGX-013 | 010 | **所在不明（意図的: 予約済・未生成）** — §2 | — |

注: UC-LGX-001〜011 全ファイルについてヘッダの Document ID とファイル名の一致をスクリプトで照合済（不一致 0 件）。

### 1.3 前段ループ成果物（QSET / SPP、各 11 ファイル全 SPEC 対応）

QSET（質問票）: OLD/docs/frontend-pass/questionnaires/、SPP（仕様パッチ）: OLD/docs/spec-patches/。番号 001〜010 は SPEC-LGX-001〜010 に対応、011 は MCP サーバ反復 2。全 22 ファイルでヘッダ ID とファイル名の一致をスクリプト照合済（不一致 0 件、確度: 全件確認済）。

| 文書 ID | 参照元 SPEC | 実体パス（OLD/docs/ 配下） |
|---------|------------|---------------------------|
| QSET-LGX-001 | 001, 002, 010 | frontend-pass/questionnaires/QSET-LGX-001_全体境界整合.md |
| QSET-LGX-002 | 002, 004 | frontend-pass/questionnaires/QSET-LGX-002_グラフ基盤.md |
| QSET-LGX-003 | 003, 009 | frontend-pass/questionnaires/QSET-LGX-003_コンテキスト解決.md |
| QSET-LGX-004 | 004, 010 | frontend-pass/questionnaires/QSET-LGX-004_検証.md |
| QSET-LGX-005 | 005 | frontend-pass/questionnaires/QSET-LGX-005_グラフ走査.md |
| QSET-LGX-006 | 006, 010 | frontend-pass/questionnaires/QSET-LGX-006_embeddingとドリフト検出.md |
| QSET-LGX-007 | 007 | frontend-pass/questionnaires/QSET-LGX-007_フィードバックループ.md |
| QSET-LGX-008 | 008 | frontend-pass/questionnaires/QSET-LGX-008_マイグレーション.md |
| QSET-LGX-009 | 009 | frontend-pass/questionnaires/QSET-LGX-009_MCPサーバ.md |
| QSET-LGX-010 | 010 | frontend-pass/questionnaires/QSET-LGX-010_embedding運用と監査.md |
| QSET-LGX-011 | 009 | frontend-pass/questionnaires/QSET-LGX-011_MCPサーバ-反復2.md |
| SPP-LGX-001 | 001, 010 | spec-patches/SPP-LGX-001_全体境界整合.md |
| SPP-LGX-002 | 002 | spec-patches/SPP-LGX-002_グラフ基盤.md |
| SPP-LGX-003 | 003 | spec-patches/SPP-LGX-003_コンテキスト解決.md |
| SPP-LGX-004 | 002, 004 | spec-patches/SPP-LGX-004_検証.md |
| SPP-LGX-005 | 005 | spec-patches/SPP-LGX-005_グラフ走査.md |
| SPP-LGX-006 | 006 | spec-patches/SPP-LGX-006_embeddingとドリフト検出.md |
| SPP-LGX-007 | 007 | spec-patches/SPP-LGX-007_フィードバックループ.md |
| SPP-LGX-008 | 008 | spec-patches/SPP-LGX-008_マイグレーション.md |
| SPP-LGX-009 | 009 | spec-patches/SPP-LGX-009_MCPサーバ.md |
| SPP-LGX-010 | 010 | spec-patches/SPP-LGX-010_embedding運用と監査.md |
| SPP-LGX-011 | 009 | spec-patches/SPP-LGX-011_MCPサーバ-反復2.md |

### 1.4 ADR（OLD/docs/adr/）

SPEC から参照されるのは下表 5 件。旧プロジェクトには ADR-LGX-001〜011 の全 11 件が存在する（未参照の 001/002/004/006/007/008 も実装判断の背景として有用、§3 参照）。

| 文書 ID | 参照元 SPEC | 実体パス | 確度 |
|---------|------------|----------|------|
| ADR-LGX-003 | 004, 006 | OLD/docs/adr/ADR-LGX-003_embedding-determinism-model.md | 確認済（ヘッダ ID 一致） |
| ADR-LGX-005 | 007 | OLD/docs/adr/ADR-LGX-005_protect-irreproducible-data-on-corruption.md | 確認済 |
| ADR-LGX-009 | 006 | OLD/docs/adr/ADR-LGX-009_contextual-retrieval-determinism.md | 確認済 |
| ADR-LGX-010 | 009 | OLD/docs/adr/ADR-LGX-010_mcp-child-process-timeout.md | 確認済 |
| ADR-LGX-011 | 008 | OLD/docs/adr/ADR-LGX-011_migrate-concurrency-risk-acceptance.md | 確認済 |

### 1.5 GAP 分析（OLD/docs/gap-analysis/）

SPEC から参照される GAP は 61 ID。うち 60 件の実体を確認（全 GAP ファイルでヘッダ ID とファイル名の一致をスクリプト照合済、不一致 0 件）。GAP-LGX-116 のみ不在（意図的削除、§2）。

| 文書 ID | 参照元 SPEC | 実体ファイル名（OLD/docs/gap-analysis/ 配下） |
|---------|------------|--------------------------------------------|
| GAP-LGX-001 | 001 | GAP-LGX-001_uc網羅宣言と下位spec拡張の不一致.md |
| GAP-LGX-002 | 001 | GAP-LGX-002_不変条件マトリクスの整合検証手段欠如.md |
| GAP-LGX-003 | 001 | GAP-LGX-003_検証owner要否ポリシーの欠如.md |
| GAP-LGX-004 | 001 | GAP-LGX-004_umbrella変更ポリシー統治手順の欠如.md |
| GAP-LGX-005 | 001 | GAP-LGX-005_mcp-inv-1実装owner語義の不整合.md |
| GAP-LGX-023 | 002, 008 | GAP-LGX-023_refresh-subnodes-rewrite-atomicity.md |
| GAP-LGX-024 | 002 | GAP-LGX-024_refresh-backup-naming-retention.md |
| GAP-LGX-041 | 003 | GAP-LGX-041_context-log書込失敗時の本処理成否.md |
| GAP-LGX-043 | 003 | GAP-LGX-043_上流に存在しないパスと部分欠損の扱い.md |
| GAP-LGX-045 | 003 | GAP-LGX-045_sectionsの不正形式入力の扱い.md |
| GAP-LGX-047 | 003 | GAP-LGX-047_outline-only見出し皆無時の出力.md |
| GAP-LGX-061 | 004 | GAP-LGX-061_空グラフ時のcheck挙動.md |
| GAP-LGX-064 | 004 | GAP-LGX-064_形式検証カテゴリのseverity割当完全性.md |
| GAP-LGX-065 | 004 | GAP-LGX-065_Ok_severityの使用条件.md |
| GAP-LGX-072 | 004 | GAP-LGX-072_全層checkの冪等性射程.md |
| GAP-LGX-081 | 005 | GAP-LGX-081_custom-エッジの方向セマンティクス.md |
| GAP-LGX-085 | 005 | GAP-LGX-085_max_depth打ち切りの可観測性.md |
| GAP-LGX-101 | 006 | GAP-LGX-101_空テキストノードのembedding生成.md |
| GAP-LGX-102 | 006 | GAP-LGX-102_巨大テキストのトークン上限超過時の扱い.md |
| GAP-LGX-103 | 006 | GAP-LGX-103_モデル出力shapeの異常時の扱い.md |
| GAP-LGX-104 | 006 | GAP-LGX-104_ゼロベクトルcosineと浮動小数点推論の数値安定性.md |
| GAP-LGX-105 | 006 | GAP-LGX-105_cosine類似度の値域定義.md |
| GAP-LGX-106 | 006 | GAP-LGX-106_embed時のモデル不在読込失敗の挙動.md |
| GAP-LGX-108 | 006 | GAP-LGX-108_embed部分失敗とトランザクション境界の整合.md |
| GAP-LGX-110 | 006 | GAP-LGX-110_未生成ノードのドリフト扱い.md |
| GAP-LGX-113 | 006 | GAP-LGX-113_ContextualRetrievalの並行性と非決定性.md |
| GAP-LGX-114 | 006 | GAP-LGX-114_contentハッシュの正規化方針.md |
| GAP-LGX-115 | 006 | GAP-LGX-115_modelバージョン識別子の生成と変化判定.md |
| GAP-LGX-116 | 006 | **所在不明（意図的削除: DUPLICATE/OUT_OF_SCOPE）** — §2 |
| GAP-LGX-118 | 006 | GAP-LGX-118_サブノードcontentRange不正値の検証.md |
| GAP-LGX-120 | 006 | GAP-LGX-120_embedのjson出力スキーマと個別ノード指定引数契約.md |
| GAP-LGX-121 | 007 | GAP-LGX-121_observe-message境界.md |
| GAP-LGX-122 | 007 | GAP-LGX-122_related-id重複と実在検証.md |
| GAP-LGX-124 | 007 | GAP-LGX-124_reject-reason境界.md |
| GAP-LGX-126 | 007 | GAP-LGX-126_DB破損時の挙動.md |
| GAP-LGX-127 | 007 | GAP-LGX-127_proposal不正遷移と終端不可逆性.md |
| GAP-LGX-129 | 007 | GAP-LGX-129_observation状態集合と遷移定義.md |
| GAP-LGX-135 | 007 | GAP-LGX-135_proposal保持ポリシー.md |
| GAP-LGX-139 | 007 | GAP-LGX-139_context-log記録失敗時の本体影響.md |
| GAP-LGX-140 | 007 | GAP-LGX-140_人間のみCLI実行の強制手段.md |
| GAP-LGX-141 | 008 | GAP-LGX-141_空プロジェクトのmigrate挙動.md |
| GAP-LGX-142 | 008 | GAP-LGX-142_大規模入力のサイズ所要時間境界.md |
| GAP-LGX-143 | 008 | GAP-LGX-143_init既存ファイル判定境界.md |
| GAP-LGX-144 | 008 | GAP-LGX-144_ソースデータ破損の検出方針.md |
| GAP-LGX-146 | 008 | GAP-LGX-146_マッピング不可ID時の処理方針.md |
| GAP-LGX-148 | 008 | GAP-LGX-148_部分migrate中断からの再開戦略.md |
| GAP-LGX-150 | 008 | GAP-LGX-150_migrate中の同時アクセス競合.md |
| GAP-LGX-152 | 008 | GAP-LGX-152_graphtoml書込のアトミック性.md |
| GAP-LGX-153 | 008 | GAP-LGX-153_bakファイル衝突時の方針.md |
| GAP-LGX-154 | 008 | GAP-LGX-154_バージョン判定の根拠.md |
| GAP-LGX-157 | 008 | GAP-LGX-157_migrateフロム引数の意味契約矛盾.md |
| GAP-LGX-158 | 008 | GAP-LGX-158_matrix抽出規則と不正入力.md |
| GAP-LGX-159 | 008 | GAP-LGX-159_idマップの一意性と重複検出.md |
| GAP-LGX-160 | 008 | GAP-LGX-160_書換え対象IDの追跡可能性.md |
| GAP-LGX-162 | 009 | GAP-LGX-162_エラー応答へのmeta付与可否.md |
| GAP-LGX-168 | 009 | GAP-LGX-168_サーバ起動初期化失敗時の挙動.md |
| GAP-LGX-169 | 009 | GAP-LGX-169_CLI子プロセスのタイムアウト.md |
| GAP-LGX-170 | 009 | GAP-LGX-170_MCP層自身のロギングと機密マスキング.md |
| GAP-LGX-171 | 009 | GAP-LGX-171_PERF03参照値の整合.md |
| GAP-LGX-185 | 010 | GAP-LGX-185_スコアの特殊浮動小数点値の扱い.md |
| GAP-LGX-186 | 010 | GAP-LGX-186_同一次元別モデルバージョンのベースライン妥当性.md |

参考: 旧プロジェクトには SPEC 未参照の GAP-LGX-151（二重migrate及びauto重複起動の排他）も存在する。

---

## §2 所在不明の文書（4 件）

| 文書 ID | 状況 | 根拠 |
|---------|------|------|
| **VAL-LGX-001** | LGX 名義の実体ファイルは両旧プロジェクトのどこにも存在しない。`legixy.old.p1/docs/validation/` は .gitkeep のみの空ディレクトリ。**前身文書 VAL-LX-001** が `traceability-engine.v3.chg_to_lexigy/docs/validation/VAL-LX-001_外部照合記録.md`（v1.1.0, Approved）として存在し、SPEC が引用する Finding 番号（E-01 ダングリング・エッジ、E-05/E-06、P-02〜P-04 等）と内容が一致する。**推定**: リブランド時に参照 ID のみ LX→LGX に書き換えられ、文書自体は再作成されなかった | VAL-LX-001 §6 に Finding E-01〜E-07、E-01 の対処（CTX-INV-5 新設）が SPEC-LGX-001 v0.3.0 変更履歴と一致 |
| **UC-LGX-012** | **意図的な未生成（予約済）**。SPEC-LGX-001 ヘッダに「UC-LGX-012/013 は予約済・未生成」と明記。snapshot（ベースライン凍結ライフサイクル）の UC として SPEC-LGX-010 受理後の UC フェーズで生成予定 | SPEC-LGX-001 §3 REQ.02 予約注記、SPEC-LGX-010 §1.3 |
| **UC-LGX-013** | 同上。drift standalone 対比の UC として生成予定 | 同上 |
| **GAP-LGX-116** | **意図的に削除済**。TP-LGX-006（旧プロジェクト test-perspectives）に「DUPLICATE（GAP-LGX-115 派生）+ OUT_OF_SCOPE として GAP-LGX-116 削除」と記録。SPEC-LGX-006 §の「GAP-LGX-116（別管理）」参照は旧 embedding（v0.1.0）移行の論点を指し、実質は GAP-LGX-115 と UC-LGX-009 / NFR COMPAT.04 が所有 | OLD/docs/test-perspectives/TP-LGX-006_embeddingとドリフト検出.md V-04 行 |

実装上の扱い: 4 件とも「参照切れによる仕様欠落」ではない。ただし VAL-LGX-001 は SPEC の不変条件根拠として頻繁に引用されるため、前身 VAL-LX-001 を参考資料として持ち込むことを推奨する（§3）。

---

## §3 持ち込み優先順位

### 必須（実装着手前に新リポジトリへ配置すべきもの）

1. **LGX-COMPAT-001** — `OLD/docs/legixy_cli_compat_reference.md`。CLI/MCP 互換契約（プロジェクト最重要制約）。これ無しでは引数体系・終了コードを決められない。
2. **LEGIXY-SPEC-001** — `OLD/docs/legixy_foundational_spec.md`。全 SPEC の親文書。不変条件（CTX-INV 等）の正準定義元。
3. **LGX-EXT-001** — `OLD/docs/legixy_subnode_spec_v0.2.1.md`。最多参照（72 回）。サブノード化のデータ構造・SUBNODE-INV の定義元。
4. **LGX-EXT-002** — `OLD/docs/legixy_cache_spec_v0_1_0.md`。Prompt Caching / MCP Result Persistence 仕様（SPEC-LGX-003/009 の前提）。
5. **NFR-LGX-001** — `OLD/docs/nfr/NFR-LGX-001_非機能要件.md`。全 10 SPEC が参照。性能・信頼性・互換（COMPAT.04 等）の要件元。
6. **CLAUDE.md** — `OLD/CLAUDE.md`。開発プロセス規律と互換制約の常時参照文書。新リポジトリのルートに配置（後述の DevProc パス問題に注意、§4）。
7. **UC-LGX-001〜011（11 ファイル）** — `OLD/docs/usecases/`。SPEC の受け入れ基準・走査フローの定義元。DevProc チェーンの次フェーズ入力。

### 推奨（設計判断・GAP 解決の根拠として強く推奨）

8. **ADR-LGX-001〜011（全 11 ファイル）** — `OLD/docs/adr/`。SPEC 参照は 5 件だが、未参照 ADR（006 human-only CLI、007 非有限値ポリシー、008 GraphDAG 分離等）も実装判断を直接拘束する。一括持ち込みが安全。
9. **GAP-LGX 参照 60 ファイル** — `OLD/docs/gap-analysis/`。SPEC の REQ がどの欠陥を塞ぐためのものかの背景。境界条件・エラー処理の実装時に必読。GAP-LGX-151 含め全件一括が管理上単純。
10. **VAL-LX-001（VAL-LGX-001 の前身）** — `traceability-engine.v3.chg_to_lexigy/docs/validation/VAL-LX-001_外部照合記録.md`。SPEC が引用する Finding E/P 系の唯一の実体。持ち込み時は「前身文書」である旨の注記を付すか、人間承認の上で VAL-LGX-001 として改名・改訂する。
11. **SPP-LGX-001〜011** — `OLD/docs/spec-patches/`。SPEC 変更履歴の根拠。特に SPP-LGX-001 は UC-LGX-012/013 生成時の次反復が予告されており必要になる。

### 参考（必要時に参照すれば足りるもの）

12. **QSET-LGX-001〜011** — `OLD/docs/frontend-pass/questionnaires/`。前段ループは完了済（SPEC は Accepted 相当）のため履歴的価値が主。
13. **FCR-LGX-001〜011** — `OLD/docs/frontend-pass/check-results/`。同上（受理判定の記録）。
14. **TP-LGX-001〜010** — `OLD/docs/test-perspectives/`。次フェーズ（TP[UC] / TS）の参考。GAP-LGX-116 削除の記録元でもある。
15. **traceability/graph.toml・matrix.md** — `OLD/docs/traceability/`。トレーサビリティグラフの現状スナップショット。
16. **spec-change-proposals 2 ファイル** — `OLD/docs/spec-change-proposals/`。GAP 解決提案の経緯。
17. **旧実装一式** — `traceability-engine.v3.chg_to_lexigy/`（§4.2 参照）。互換検証のリファレンス実装・実測対象として参照。

---

## §4 補足

### 4.1 LGX-COMPAT-001（CLI 互換契約）の概要

- **由来**: `traceability-engine` v0.4.0-alpha4 の実バイナリ・MCP 層・設定ファイルの実測（Status: Reference, v1.1.0, 2026-06-10）。
- **互換対象**: (a) サブコマンド名 (b) 位置引数 (c) フラグ名と値 (d) 既定値 (e) 終了コード (f) MCP 3 ツールの CLI マッピング。出力本文フォーマットは SPEC-LGX-003/004/006 側が規定。
- **サブコマンドは 19 個**（init / migrate / check / embed / drift / report / calibrate / snapshot ほか）。グローバルオプションは `--project-root`（既定 `.`）、`--json`、`--models-dir`。
- **終了コード規約**: 引数パーサ層の使用法誤り = exit 2（clap 既定の契約化）、意味的不正・実行時失敗 = exit 1、`check` は Error 件数 > 0 で exit 1（G1 ゲート）。
- **加算的拡張の前例**: `embed --node <ID>` / `--force` は SPEC 主導で追加された加算的拡張（v1.1.0、人間承認 2026-06-10）。既存呼出の挙動は不変であること（後方互換）が拡張の条件。
- MCP 層は常に `<bin> --project-root <root> <subcommand...>` 形式で CLI を子プロセス起動する。

### 4.2 旧実装（traceability-engine.v3.chg_to_lexigy）の構成概要

- **Rust ワークスペース**: `crates/lx-core, lx-graph, lx-db, lx-ctx, lx-check, lx-nav, lx-embed, lx-feedback, lx-mig, lx-cli`（LGX-COMPAT-001 によれば旧称 te-* → legixy では legixy-* に改名予定。CLI 統合バイナリは cli クレート）。
- **MCP サーバ**: `ts-mcp/`（TypeScript、3 ツールを CLI 子プロセスにマッピング）。
- **モデル**: `models/all-MiniLM-L6-v2`（ONNX、embedding 384 次元）。
- **文書体系は LX 名義**: `docs/specs/SPEC-LX-001〜009`、`NFR-LX-001`、`UC-LX-001〜011`、`VAL-LX-001`、detailed-design `DD-LX-*`、robustness `RB-LX-*`、sequence `SEQ-LX-*`、test-specs `TS-LX-*`。LGX 文書群（legixy.old.p1）の前身であり、ID 番号の対応はおおむね 1:1 だが**正準は LGX 側**。実装参照時は LX/LGX の取り違えに注意。
- `old.source/` 配下にさらに旧世代（RustCLI / TypeScriptMCP）が残置されている。

### 4.3 その他の留意点（人間査読時の確認推奨）

- **新リポジトリの DevProc ディレクトリ名**: 新リポジトリは `docs/DevPorc/`（綴り: DevPorc）に DevProc_V4 一式を持つが、CLAUDE.md（旧）およびメモリ上の記録は `docs/DevProc_V4/` を参照する。持ち込み時にディレクトリ名の正規化（DevPorc → DevProc_V4）またはパス参照の修正が必要。
- 新旧の SPEC 10 ファイルは内容完全一致のため、旧プロジェクト側のトレーサビリティ成果物（graph.toml 等）はそのまま流用可能な見込み（要検証）。
