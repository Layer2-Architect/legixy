Document ID: RPC-LGX-009

# RPC-LGX-009: プロジェクト初期化とマイグレーション chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-009
**対象 RBA**: RBA-LGX-009
**対象 SEQA**: SEQA-LGX-009
**対象 RBD**: RBD-LGX-009
**対象 SEQD**: SEQD-LGX-009
**検査深度**: フル（§14.2: init/migrate 両系統で DB・ファイルシステム境界・非破壊確定を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

---

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

RBA-LGX-009 の全主語（Boundary 6 / Control 14 / Entity 6 = 26 主語）と SEQA-LGX-009 の時系列メッセージから抽出。各 AR を UC-009 ステップに紐づける。

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 初期化・移行コマンド受付窓口 | `legixy init` または `legixy migrate` 要求を受け取り、適切な統括処理へ渡す | init-1 / migrate-1 |
| AR-002 | RBA | Boundary | 設定ファイル境界 | `.legixy.toml` および旧名の探索・存在確認・供給（旧名フォールバック含む） | init-代替2a / migrate-2a |
| AR-003 | RBA | Boundary | プロジェクト生成物境界 | init が新規生成する各成果物（設定テンプレート・graph.toml・ディレクトリ・engine.db・.gitignore）の書き出し先境界 | init-2a〜2d |
| AR-004 | RBA | Boundary | 旧プロジェクト境界 | 移行元 v0.1.0 プロジェクトの DB・設定・matrix・ベクタデータの供給元境界 | migrate-2a / 2b / 3 / 5 / 6 |
| AR-005 | RBA | Boundary | 移行先プロジェクト境界 | 生成した graph.toml・migration-id-map.toml・変換後 .legixy.toml・移行後 engine.db の atomic 確定先境界（退避生成含む） | migrate-4 / 5 / 6 |
| AR-006 | RBA | Boundary | 操作結果出力窓口 | 成否・変更サマリ・エラーメッセージ・リカバリ手順をアクターへ返す | init-2/代替2a / migrate-7 |
| AR-007 | RBA | Control | 初期化統括処理 | init 要求を受け、既存生成物検査・テンプレート生成・DB 初期化を協調させ、レポートを操作結果出力窓口へ渡す | init-1〜2d |
| AR-008 | RBA | Control | 既存生成物検査処理 | `.legixy.toml` / `graph.toml` / `engine.db` の存在を確認し、`--force` 有無に応じて続行か中断かを決定する | init-代替2a |
| AR-009 | RBA | Control | 生成物テンプレート生成処理 | ICONIX 標準 8 typecode を含む `.legixy.toml` テンプレート・空の `graph.toml`・成果物ディレクトリをプロジェクト生成物境界へ書き出す | init-2a / 2b / 2c |
| AR-010 | RBA | Control | DB 初期化処理 | 初期スキーマで engine.db を新規作成し `.legixy/` 構造をプロジェクト生成物境界へ書き出す | init-2d |
| AR-011 | RBA | Control | 移行統括処理 | migrate 要求を受け、バージョン検出から移行確定・レポートまでを順に協調させる。途中失敗時は元データを保全して中断する | migrate-1〜7 |
| AR-012 | RBA | Control | バージョン検出処理 | 旧プロジェクト境界から engine.db の `user_version` と設定ファイルの `[graph]` セクション有無を参照し、プロジェクトバージョンを確定する。矛盾特徴を検出した場合は中断を要求する | migrate-2b（Object Discovery: REQ.09） |
| AR-013 | RBA | Control | 旧設定解析処理 | 旧プロジェクト境界から `.legixy.toml` / `.trace-engine.toml` を読み込み、`[id.chain]` order・`[matrix]`・`[id]` 設定を解釈して移行設定情報を確定する。破損・`[id.chain]` 欠落を検出した場合は中断を要求する | migrate-2a |
| AR-014 | RBA | Control | matrix 読み込み処理 | 旧プロジェクト境界から matrix.md / matrix.json をパースし、成果物 ID 集合を抽出する（空集合も正常） | migrate-2b |
| AR-015 | RBA | Control | graph.toml 生成処理 | 成果物 ID 集合と移行設定情報から `[[nodes]]` と `[[edges]]` を生成し、確定前に出力妥当性（パース可能性・ID 一意性）を検証する | migrate-3 |
| AR-016 | RBA | Control | ID マッピング処理 | 旧 ID と新 ID の全単射対応表を確定する。曖昧性・多対一・全体一意性違反を検出した場合は中断を要求する | migrate-3（Object Discovery: REQ.11） |
| AR-017 | RBA | Control | 設定ファイル変換処理 | 旧 `.legixy.toml` に `[graph]` セクションを追加して legixy 形式に変換する（既存 `[matrix]` セクションを保持） | migrate-4 |
| AR-018 | RBA | Control | DB 移行処理 | 旧 engine.db を新スキーマへ変換する（追加カラム付与・既存データ保持）。vectors.bin があれば embeddings テーブルへインポートする。SQLite トランザクションで非破壊性を保証する | migrate-5 / 6 |
| AR-019 | RBA | Control | 移行確定処理 | DB コミットを先行させ、その後 graph.toml・migration-id-map.toml・.legixy.toml を atomic（一時ファイル→fsync→rename）で移行先プロジェクト境界へ確定する。確定前に退避ファイルを生成する | migrate-4〜6（Object Discovery: REQ.02） |
| AR-020 | RBA | Control | 移行レポート生成処理 | 成功時は変更サマリ（生成・更新ファイル一覧・ID 書き換え件数・バックアップ場所）を確定する。失敗時は失敗段階・バックアップ場所・リカバリ手順を確定する | migrate-7 |
| AR-021 | RBA | Entity | プロジェクトバージョン情報 | 検出されたバージョン（v0.1.0 / legixy）と判定根拠を保持する | migrate-2b |
| AR-022 | RBA | Entity | 移行設定情報 | 旧設定から解釈された `[id.chain]` order・matrix 設定・ID パターンを保持し、設定値を供給する | migrate-2a / 3 / 4 |
| AR-023 | RBA | Entity | 成果物 ID 集合 | matrix から抽出されたドキュメント ID の集合（空集合含む）を保持する | migrate-2b / 3 |
| AR-024 | RBA | Entity | 有向グラフ表現 | 生成された `[[nodes]]` と `[[edges]]` の集合を保持し、ノード取り出し・妥当性確認を担う | migrate-3 |
| AR-025 | RBA | Entity | ID マッピング表 | 旧 ID → 新 ID の全単射対応表を保持し、対応引き出し・全単射確認を担う | migrate-3〜4 |
| AR-026 | RBA | Entity | 移行レポート | 変更サマリまたは失敗情報（段階・バックアップ場所・リカバリ手順）を保持し、終了状態を判定する | init-2/代替2a / migrate-7 |

全 AR が UC ステップに紐づく（UC ステップに紐づかない AR なし）。Object Discovery 由来の 3 主語（AR-012: バージョン検出処理、AR-019: 移行確定処理、AR-015: graph.toml 生成処理の妥当性検証責務）は SPEC-LGX-008 の既存要求の構造化であり、§9 分解 (b) 候補に該当しない（RBA §6 参照）。SEQA の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

---

## 2. Concrete Responsibilities

RBD-LGX-009 のクラス一覧と SEQD-LGX-009 のメッセージから抽出。

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 初期化・移行コマンド受付窓口 | 初期化を受け付ける / 移行を受け付ける(移行元パス) | アクター境界で init / migrate 要求を受理し適切な統括処理へ渡す | Actor→Bcmd |
| CR-002 | RBD/SEQD | 設定ファイル境界 | 設定の存在を確認する / 設定を読み込む | legixy 生成物の存在確認・設定内容の供給 | Ccheck→Bcfg |
| CR-003 | RBD/SEQD | プロジェクト生成物境界 | 設定テンプレートを書き出す / グラフ定義を書き出す / 成果物ディレクトリを作成する / データベースを初期化して書き出す / 管理ディレクトリの無視設定を書き出す | init が新規生成する各成果物の書き出し | Ctmpl→Bgen / Cdbinit→Bgen |
| CR-004 | RBD/SEQD | 旧プロジェクト境界 | データベースのバージョン情報を参照する / 旧設定を読み込む / マトリクスを読み込む / データベースを読み込む / ベクタデータを読み込む / ベクタデータの有無を確認する / 既存参照を読み込む | 移行元 v0.1.0 プロジェクトの各データ供給 | Cver→Bsrc / Ccfg→Bsrc / Cmat→Bsrc / Cdb→Bsrc / Cidmap→Bsrc |
| CR-005 | RBD/SEQD | 移行先プロジェクト境界 | データベースコミットを先行させる / グラフ定義を確定する / IDマッピング表を確定する / 設定ファイルを確定する / 退避ファイルを生成する | 移行成果物の atomic 確定・退避生成 | Ccommit→Bdst |
| CR-006 | RBD/SEQD | 操作結果出力窓口 | 成功サマリを出力する / エラーを出力する | 成否・サマリ・エラーをアクターへ返す | →Bout→Actor |
| CR-007 | RBD/SEQD | 初期化統括処理 | 初期化を統括する(操作要求) | init フロー全体の協調・中断判断・操作結果出力窓口への渡し | Bcmd→Cinit |
| CR-008 | RBD/SEQD | 既存生成物検査処理 | 生成物の存在を検査する / 中断か続行かを判断する(検査結果, 強制フラグ) | legixy 生成物の存在確認・続行/中断判断 | Cinit→Ccheck |
| CR-009 | RBD/SEQD | 生成物テンプレート生成処理 | テンプレートを生成する | ICONIX 標準構成のテンプレート生成・書き出し協調 | Cinit→Ctmpl |
| CR-010 | RBD/SEQD | データベース初期化処理 | データベースを初期化する | 初期スキーマの engine.db 生成・.gitignore 書き出し | Cinit→Cdbinit |
| CR-011 | RBD/SEQD | 移行統括処理 | 移行を統括する(操作要求) / 途中失敗を検知して中断する(失敗情報) | migrate フロー全体の協調・途中失敗時の非破壊中断 | Bcmd→Cmig |
| CR-012 | RBD/SEQD | バージョン検出処理 | バージョンを検出する / 矛盾を検出して中断を要求する(バージョン識別情報) | プロジェクトバージョンの自動判定・矛盾時中断要求 | Cmig→Cver |
| CR-013 | RBD/SEQD | 旧設定解析処理 | 旧設定を解析する / 破損を検出して中断を要求する(設定内容) | 旧設定の解釈・移行設定情報の確定・破損時中断要求 | Cmig→Ccfg |
| CR-014 | RBD/SEQD | マトリクス読み込み処理 | マトリクスを読み込む | matrix.md / matrix.json のパース・成果物ID集合の確定 | Cmig→Cmat |
| CR-015 | RBD/SEQD | グラフ定義生成処理 | グラフ定義を生成する(成果物ID集合, 移行設定情報) / 出力妥当性を検証する(有向グラフ表現) | ノード・エッジ生成・出力妥当性検証 | Cmig→Cgraph |
| CR-016 | RBD/SEQD | IDマッピング処理 | IDマッピングを生成する / 全単射違反を検出して中断を要求する | 旧→新 ID 全単射対応表の確定・違反時中断要求 | Cmig→Cidmap |
| CR-017 | RBD/SEQD | 設定ファイル変換処理 | 設定ファイルを変換する(移行設定情報) | 旧設定を legixy 形式へ変換（[graph] セクション追加） | Cmig→Ccfgconv |
| CR-018 | RBD/SEQD | データベース移行処理 | データベースを移行する / ベクタデータをインポートする(ベクタデータ内容) | 旧 DB の新スキーマ変換・vectors.bin インポート | Cmig→Cdb |
| CR-019 | RBD/SEQD | 移行確定処理 | 移行を確定する(有向グラフ表現, IDマッピング表, 変換後設定内容, 移行済みデータベース) | DB コミット先行 + atomic rename による非破壊確定 | Cmig→Ccommit |
| CR-020 | RBD/SEQD | 移行レポート生成処理 | 成功レポートを生成する(変更サマリ) / 失敗レポートを生成する(失敗情報) | 成否に応じた移行レポートの確定・出力窓口への渡し | Cmig→Crep |
| CR-021 | RBD/SEQD | プロジェクトバージョン情報 | （データ保持） | バージョン種別と判定根拠を保持 | Cver→Ever |
| CR-022 | RBD/SEQD | 移行設定情報 | 設定値を取り出す(設定キー) | チェーン順序・マトリクス設定・IDパターンを保持・供給 | Cgraph→Emigset / Ccfgconv→Emigset |
| CR-023 | RBD/SEQD | 成果物ID集合 | 件数を取り出す | 識別子のコレクションを保持・件数供給 | Cgraph→Eids |
| CR-024 | RBD/SEQD | 有向グラフ表現 | ノードを取り出す(識別子) / 妥当性を確認する | ノード・エッジのコレクションを保持・走査・妥当性確認 | Cidmap→Egraph / Ccommit→Egraph |
| CR-025 | RBD/SEQD | IDマッピング表 | 旧IDから新IDを引く(識別子) / 全単射を確認する | 対応エントリのコレクションを保持・引き出し・全単射確認 | Ccommit→Eidmap |
| CR-026 | RBD/SEQD | 移行レポート | 終了状態を判定する | 成否・ファイル一覧・件数・バックアップ場所・失敗段階・リカバリ手順を保持し終了状態を判定する | Crep→Ereport |

---

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・init/migrate 両要求受付操作を識別 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Boundary・存在確認と読み込み操作を識別 | GREEN |
| AR-003 | CR-003 | preserved | 同一 Boundary・各書き出し操作（設定テンプレート・グラフ定義・ディレクトリ・DB・.gitignore）を識別 | GREEN |
| AR-004 | CR-004 | preserved | 同一 Boundary・DB バージョン参照・設定・マトリクス・ベクタデータ・既存参照の各読み込み操作を識別 | GREEN |
| AR-005 | CR-005 | preserved | 同一 Boundary・DB コミット先行・グラフ定義確定・IDマッピング表確定・設定ファイル確定・退避生成操作を識別 | GREEN |
| AR-006 | CR-006 | preserved | 同一 Boundary・成功サマリとエラーの出力操作を識別 | GREEN |
| AR-007 | CR-007 | preserved | 同名 Control・init フロー全体の協調操作を識別。init フローでの移行レポート Entity 直接利用（SEQD §1 では Cinit→Bout に引数として渡す形）は SEQA の抽象表現と整合（注1参照） | GREEN |
| AR-008 | CR-008 | preserved | 同名 Control・生成物の存在検査と続行/中断判断の 2 操作を識別（`--force` 指定は強制フラグ引数で表現） | GREEN |
| AR-009 | CR-009 | preserved | 同名 Control・テンプレート生成操作を識別（書き出しは Boundary への委譲） | GREEN |
| AR-010 | CR-010 | preserved | RBA の「DB 初期化処理」が RBD の「データベース初期化処理」に mapping（表記を概念語「データベース」に統一）。責務に変更なし | GREEN |
| AR-011 | CR-011 | preserved | 同名 Control・統括操作と途中失敗中断操作を識別。非破壊中断責務が「途中失敗を検知して中断する」操作に具体化 | GREEN |
| AR-012 | CR-012 | preserved | 同名 Control・バージョン検出と矛盾検出中断要求の 2 操作を識別 | GREEN |
| AR-013 | CR-013 | preserved | 同名 Control・旧設定解析と破損検出中断要求の 2 操作を識別 | GREEN |
| AR-014 | CR-014 | preserved | RBA の「matrix 読み込み処理」が RBD の「マトリクス読み込み処理」に mapping（表記を概念語「マトリクス」に統一）。責務に変更なし | GREEN |
| AR-015 | CR-015 | preserved | RBA の「graph.toml 生成処理」が RBD の「グラフ定義生成処理」に mapping（表記を概念語「グラフ定義」に統一）。出力妥当性検証責務が「出力妥当性を検証する」操作に具体化 | GREEN |
| AR-016 | CR-016 | preserved | 同名（IDマッピング処理）。ID マッピング生成と全単射違反検出中断要求の 2 操作を識別 | GREEN |
| AR-017 | CR-017 | preserved | 同名 Control・設定変換操作を識別（移行設定情報からの設定値取り出しは Entity 操作 CR-022 で対応） | GREEN |
| AR-018 | CR-018 | preserved | RBA の「DB 移行処理」が RBD の「データベース移行処理」に mapping（表記を概念語「データベース」に統一）。DB 移行とベクタデータインポートの 2 操作を識別 | GREEN |
| AR-019 | CR-019 | preserved | 同名 Control・移行確定操作を識別（DB コミット先行・atomic 確定は Ccommit→Bdst メッセージ順序で表現） | GREEN |
| AR-020 | CR-020 | preserved | 同名 Control・成功/失敗別レポート生成の 2 操作を識別。出力窓口への渡しは SEQD で `Crep->>Bout` として具体化（操作結果出力窓口へ直接委譲） | GREEN |
| AR-021 | CR-021 | preserved | 同名 Entity・バージョン種別と判定根拠の属性を識別。操作は保持のみ（プロジェクトバージョン情報を受け取って移行統括処理へ渡す役割はメッセージで表現） | GREEN |
| AR-022 | CR-022 | preserved | 同名 Entity・チェーン順序・マトリクス設定・IDパターンを保持、「設定値を取り出す」操作を識別 | GREEN |
| AR-023 | CR-023 | preserved | 同名（成果物ID集合）・識別子のコレクションを保持、「件数を取り出す」操作を識別 | GREEN |
| AR-024 | CR-024 | preserved | 同名 Entity・ノード・エッジのコレクションを保持、「ノードを取り出す」「妥当性を確認する」操作を識別 | GREEN |
| AR-025 | CR-025 | preserved | 同名（IDマッピング表）・対応エントリのコレクションを保持、「旧IDから新IDを引く」「全単射を確認する」操作を識別 | GREEN |
| AR-026 | CR-026 | preserved | 同名 Entity・成否・ファイル一覧・件数・バックアップ場所・失敗段階・リカバリ手順を保持、「終了状態を判定する」操作を識別。init フローでの Ereport レーン省略については注1を参照 | GREEN |

26 AR すべて preserved（1:1 または表記統一）。split / merged / shifted / lost / mutated / ambiguous なし。

**注1（AR-007 / AR-026 — init フローでの移行レポート Entity レーン）**: SEQA §2（init 基本フロー）では `Cinit->>Ereport: init 成功の結果を確定する` → `Ereport->>Bout: 移行レポートを渡す` と Entity を明示的な中継レーンとして描く。SEQD §1（init 基本フロー）では `Cinit->>Bout: 成功サマリを出力する(移行レポート)` と Entity を引数として渡す形に圧縮する。この差異は: (a) 移行レポート Entity クラスが RBD §1 に完全定義されている、(b) migrate フローおよびエラーフローでは SEQD も Ereport を明示的レーンとして使用する、(c) init フローで SEQA と SEQD が同じ圧縮パターンを採用している（一貫性あり）、(d) RBD §2 が「初期化統括処理 → 移行レポート（生成 1対1）」の関係を明記している、の 4 点から責務消失ではなく実装レベルの表現精度の違いと判断する。Entity の責務は保存されている。

---

## 4. Role Fitness Check（§5.2）

### Boundary

- **Finding**: 6 Boundary クラス（コマンド受付窓口・設定ファイル境界・プロジェクト生成物境界・旧プロジェクト境界・移行先プロジェクト境界・操作結果出力窓口）はすべて境界操作のみを保持する。最も操作数が多い旧プロジェクト境界（7 操作）も DB・設定・マトリクス・ベクタデータ・参照の読み込み/確認に限定し、変換や判断ロジックを持たない。移行先プロジェクト境界が atomic 確定・退避生成操作を持つが、これは「どう書き出すか」ではなく「書き出し先との境界契約」であり Boundary overreach に該当しない。
- **Severity**: なし / **Required action**: なし

### Control

- **Finding**: 各 Control が担う責務と実行する操作の整合をチェックする（SEQD §7 Behavior Allocation に詳述）。
  - 初期化統括処理: 協調のみ。自身で設定を読まない、テンプレートを生成しない。Service blob 化なし。
  - 移行統括処理: 協調と途中失敗中断判定のみ（「途中失敗を検知して中断する」は協調の一部）。各変換処理を担う処理を適切に委譲する。Service blob 化なし。
  - バージョン検出処理・旧設定解析処理: 各自の検出/解析のみ。相互侵食なし。
  - グラフ定義生成処理: 生成と出力妥当性検証を担う。DB 変換等の越権なし。
  - 移行確定処理: atomic 確定のみ。バージョン検出等の越権なし。
  - Control leakage: なし（データ保持を Entity に委ねる。統括処理は中間データを引数で渡す）。
- **Severity**: なし / **Required action**: なし

### Entity

- **Finding**: 6 Entity（プロジェクトバージョン情報・移行設定情報・成果物ID集合・有向グラフ表現・IDマッピング表・移行レポート）はすべて自身のデータ操作のみを保持する。
  - 移行設定情報の「設定値を取り出す」、成果物ID集合の「件数を取り出す」、有向グラフ表現の「ノードを取り出す / 妥当性を確認する」、IDマッピング表の「旧IDから新IDを引く / 全単射を確認する」、移行レポートの「終了状態を判定する」はすべて自身のデータに対する操作。
  - Entity anemia: プロジェクトバージョン情報のみ明示的な操作なし（データ保持のみ）。これはドメイン上、バージョン情報はバージョン検出処理が生成・移行統括処理が読む一方向データ構造であり、自身操作を持たないことは正当。Entity anemia 扱いにしない（理由: legixy の他 RPC でも類似パターンあり、保持専用 Entity は許容）。
  - Entity overreach: なし（いずれも変換・判断ロジックを Control に委ねる）。
- **Severity**: なし / **Required action**: なし

---

## 5. Sequential Execution Check（§5.3）

### Basic Flow — init 系統

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| init-1（`legixy init` 実行） | Actor→コマンド受付窓口→初期化統括処理 | 初期化を受け付ける→初期化を統括する | Yes | |
| init-2a（.legixy.toml 生成） | 初期化統括処理→生成物テンプレート生成処理→プロジェクト生成物境界 | テンプレートを生成する→設定テンプレートを書き出す | Yes | |
| init-2b（graph.toml 生成） | 生成物テンプレート生成処理→プロジェクト生成物境界 | グラフ定義を書き出す | Yes | |
| init-2c（各ディレクトリ生成） | 生成物テンプレート生成処理→プロジェクト生成物境界 | 成果物ディレクトリを作成する | Yes | |
| init-2d（.legixy/ 生成） | DB初期化処理→プロジェクト生成物境界 | データベースを初期化して書き出す→管理ディレクトリの無視設定を書き出す | Yes | |
| init 終了（成功出力） | 移行レポート→操作結果出力窓口→Actor | 成功サマリを出力する→終了コード 0 | Yes | 注1参照（Entity 圧縮） |

### Basic Flow — migrate 系統

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| migrate-1（`legixy migrate --from` 実行） | Actor→コマンド受付窓口→移行統括処理 | 移行を受け付ける(移行元パス)→移行を統括する | Yes | |
| migrate-2a（.legixy.toml 解析） | 移行統括処理→旧設定解析処理→旧プロジェクト境界→移行設定情報 | 旧設定を解析する→旧設定を読み込む→移行設定情報確定 | Yes | |
| migrate-2b（matrix パース） | 移行統括処理→matrix読み込み処理→旧プロジェクト境界→成果物ID集合 | マトリクスを読み込む→マトリクスを読み込む→成果物ID集合確定 | Yes | |
| migrate-3（nodes/edges 生成） | graph.toml生成処理→成果物ID集合/移行設定情報→有向グラフ表現 | グラフ定義を生成する→件数を取り出す/設定値を取り出す→出力妥当性を検証する | Yes | |
| migrate-4（.legixy.toml 変換） | 設定ファイル変換処理→移行設定情報 | 設定ファイルを変換する→設定値を取り出す | Yes | |
| migrate-5（DB 移行） | DB移行処理→旧プロジェクト境界 | データベースを移行する→データベースを読み込む→ベクタデータの有無を確認する | Yes | |
| migrate-6（vectors.bin） | DB移行処理→旧プロジェクト境界（vectors.bin）→移行先プロジェクト境界 | ベクタデータを読み込む→ベクタデータをインポートする | Yes | |
| migrate-7（移行レポート出力） | 移行レポート生成処理→移行レポート→操作結果出力窓口→Actor | 成功レポートを生成する→成功サマリを出力する→終了コード 0 | Yes | |

### Alternative Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| init 代替 2a（.legixy.toml 既存） | 既存生成物検査処理→設定ファイル境界→存在確認→初期化統括処理→操作結果出力窓口 | 設定の存在を確認する→中断か続行かを判断する→エラーを出力する | Yes | |
| migrate 代替 2b（旧プロジェクト不在） | バージョン検出処理→旧プロジェクト境界（不在）→移行統括処理→移行レポート生成処理→操作結果出力窓口 | データベースのバージョン情報を参照する→矛盾を検出→途中失敗を検知→失敗レポートを生成する→エラーを出力する | Yes | |

### Exception Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外: migrate 途中失敗（非破壊中断） | graph.toml生成処理→有向グラフ表現（妥当性検証失敗）→移行統括処理中断→移行レポート生成処理 | グラフ定義を生成する→出力妥当性を検証する（失敗）→途中失敗を検知して中断→失敗レポートを生成する→エラーを出力する | Yes | 移行確定処理は実行されず元データ保全 |
| 例外: migrate 旧設定破損 | 旧設定解析処理→旧プロジェクト境界（破損）→移行統括処理中断 | 旧設定を解析する→破損を検出して中断を要求する→途中失敗を検知して中断する→失敗レポートを生成する | Yes | |

全 UC フロー（init 基本・init 代替 2a / migrate 基本・migrate 代替 2b・例外 E1/E2）が SEQA / SEQD 上で責務の不整合なく実行可能。

---

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。RBD §4 が「新規クラスの発見なし（RBA 主語と 1:1 対応）」を明記。4 件の表記統一は DB→データベース / matrix→マトリクス / graph.toml→グラフ定義 / 「DB 初期化処理」→「データベース初期化処理」であり、名称変更のみで責務に変更なし）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None（4 件の表記統一は意味変質ではなく同一責務の概念語への統一）
- **Ambiguous Mappings**: None（init フローでの移行レポート Entity レーン省略は注1で根拠を明示し解消）

---

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 26 |
| Preserved | 26 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 26 |
| Invention rate（監視用） | 0% |

---

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（解消済：注1で根拠を明示）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし（§4 参照: Boundary overreach なし / Service blob なし / Control leakage なし / Entity anemia は保持専用として正当 / Entity overreach なし）
- [x] UC 基本/代替/例外フローが具体側で実行可能（§5 参照: init 2 フロー・migrate 5 フロー全て実行可能）

---

## 9. Required Changes

なし（保存失敗なし）

---

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC 基本/代替/例外フロー全て実行可能）。

抽象責務集合（UC-009 錨着）の 26 AR が具体責務集合 26 CR へ 1:1 で保存されている。init 系統と migrate 系統の 2 フロー構造という複雑性にもかかわらず、RBD-LGX-009 §4 が明示する「新規クラスの発見なし」の通り、4 件の表記統一（概念語への統一）以外に具体側の逸脱は検出されなかった。init フローでの移行レポート Entity レーン省略も責務消失ではなく一貫した表現精度の差異と確認した。

<!-- VERDICT:APPROVE -->
