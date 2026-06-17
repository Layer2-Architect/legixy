Document ID: RPC-LGX-010

# RPC-LGX-010: トレーサビリティ健全性監査 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-010
**対象 RBA**: RBA-LGX-010
**対象 SEQA**: SEQA-LGX-010
**対象 RBD**: RBD-LGX-010
**対象 SEQD**: SEQD-LGX-010
**検査深度**: フル（報告・監査用途の計測処理。LinkCandidate 算出・空ストア代替・算出失敗例外を含む複数フロー UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 監査コマンド受付窓口 | アクターからの監査要求（`report [--json]`）を受け取る | UC-010 Step 1 |
| AR-002 | RBA | Control | 監査統括処理 | 設定解決・グラフ定義取得・ベクトルロード・スコア算出・報告生成を協調させる | UC-010 Step 1–6（全体統括） |
| AR-003 | RBA | Control | 設定解決処理 | 設定ファイル境界から `link_candidate_threshold` を解決する | UC-010 Step 2（サブ） |
| AR-004 | RBA | Boundary | 設定ファイル境界 | `.legixy.toml` から設定内容を供給する | UC-010 Step 2（サブ）/ 2a |
| AR-005 | RBA | Entity | 監査設定 | 解決済みの `link_candidate_threshold` 等の監査設定値を保持する | UC-010 Step 2（サブ）、3a–b |
| AR-006 | RBA | Control | グラフ定義取得処理 | グラフ定義境界から全エッジ（Chain / Custom / ParentChild）を取得する | UC-010 Step 2 |
| AR-007 | RBA | Boundary | グラフ定義境界 | `graph.toml` から全エッジ定義を供給する | UC-010 Step 2 |
| AR-008 | RBA | Entity | グラフエッジ集合 | 取得済みの全エッジ定義（種別・両端点）を保持する | UC-010 Step 2、3a |
| AR-009 | RBA | Control | ベクトルロード処理 | ベクトルストア境界から全件ベクトルを取得する。空状態を検知し代替フローへ分岐する | UC-010 Step 2 / 代替 2a |
| AR-010 | RBA | Boundary | ベクトルストア境界 | `engine.db` の embeddings テーブルから保存済み全件ベクトルを供給する（空状態も許容） | UC-010 Step 2 / 代替 2a |
| AR-011 | RBA | Entity | 保存済みベクトル集合 | ロード済みの全件ベクトルとその付帯情報（次元・ノード識別）を保持する | UC-010 Step 2、3a–b |
| AR-012 | RBA | Control | エッジスコア算出処理 | 既定義エッジの cosine 類似度を算出する。次元不一致・ベクトル不在・非有限スコアのエッジはスキップし集約警告を生成する | UC-010 Step 3a / SPEC-LGX-010.REQ.09錨着 |
| AR-013 | RBA | Entity | エッジスコア集合 | 算出済みのエッジ類似度スコア（算出対象エッジ・スキップエッジ・集約警告）を保持する | UC-010 Step 3a、4、5 |
| AR-014 | RBA | Control | リンク漏れ候補算出処理 | 非エッジのペアから類似度が閾値以上のリンク漏れ候補を算出する | UC-010 Step 3b |
| AR-015 | RBA | Entity | リンク漏れ候補集合 | 算出済みの候補ペアとスコアを保持する | UC-010 Step 3b、4、5 |
| AR-016 | RBA | Control | 監査報告生成処理 | エッジスコア集合・リンク漏れ候補集合・統計サマリを集約し、出力形式（text / JSON）に応じて監査報告を生成する | UC-010 Step 4–5 |
| AR-017 | RBA | Entity | 監査報告 | エッジスコア集合・リンク漏れ候補集合・統計サマリを集約した出力単位（text モードまたは JSON モード）を保持する | UC-010 Step 4–5 |
| AR-018 | RBA | Control | 空ストア通知処理 | ベクトルストアが空の場合に案内情報を生成し監査結果出力窓口へ渡す | UC-010 代替 2a |
| AR-019 | RBA | Boundary | 監査結果出力窓口 | 監査報告（stdout）と診断情報（stderr）を区別して出力する | UC-010 Step 4–5、代替 2a、例外 3a |

全 AR が UC ステップに紐づく（紐づかない AR なし）。

**AR-012（エッジスコア算出処理のスキップ・集約警告責務）の UCアンカー確認**: UC-010 では Step 3a にスキップ処理の記述がない。ただし RBA §6 Object Discovery にて「UC 記載欠落だが SPEC-LGX-010.REQ.09 / SPEC-LGX-006.REQ.04 に錨着」「UC レベル遡及不要を確認」と記録済み。SPEC に直接錨着する責務として扱い、UC アンカーは Step 3a（コンテキスト上の算出ステップ）に準拠。構造翻訳が情報を加えた分解(b)候補にはあたらない。

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 監査コマンド受付窓口 | 監査要求を受け付ける(出力形式種別) | アクター境界で監査要求を受理 | Actor→B1 |
| CR-002 | RBD/SEQD | 監査統括処理 | 監査を統括する(出力形式種別) / 算出失敗を伝達する(エラー情報) | フロー協調・失敗伝達 | B1→C0、各処理→C0 |
| CR-003 | RBD/SEQD | 設定解決処理 | 設定を解決する() | 設定ファイル境界から監査設定を確定 | C0→C1 |
| CR-004 | RBD/SEQD | 設定ファイル境界 | 設定を読み込む() / 設定の存在を確認する() | 設定供給 | C1→Bcfg |
| CR-005 | RBD/SEQD | 監査設定 | 設定値を確定する(設定内容) / 設定値を取り出す(設定キー) | 設定値保持・取得 | C1→Ecfg、C4→Ecfg、C5→Ecfg |
| CR-006 | RBD/SEQD | グラフ定義取得処理 | グラフ定義を取得する() | グラフ定義境界からグラフエッジ集合を確定 | C0→C2 |
| CR-007 | RBD/SEQD | グラフ定義境界 | 全エッジ定義を読み込む() / グラフ定義の存在を確認する() | エッジ定義供給 | C2→Bgraph |
| CR-008 | RBD/SEQD | グラフエッジ集合 | エッジを取り出す(エッジ識別子) / 全エッジを列挙する() | エッジ保持・列挙 | C2→Eedge、C4→Eedge |
| CR-009 | RBD/SEQD | ベクトルロード処理 | 全件ベクトルをロードする() / 空状態を検知する() | ベクトル全件ロード・空状態検知 | C0→C3 |
| CR-010 | RBD/SEQD | ベクトルストア境界 | 全件ベクトルを読み込む() / ストアが空かを確認する() | ベクトル全件供給・空確認 | C3→Bvec |
| CR-011 | RBD/SEQD | 保存済みベクトル集合 | ベクトルを取り出す(ノード識別子) / 全件を列挙する() / 件数を返す() | ベクトル保持・取得・列挙 | C3→Evec、C4→Evec、C5→Evec |
| CR-012 | RBD/SEQD | エッジスコア算出処理 | エッジスコアを算出する(...) / スキップエッジを記録する(...) / 集約警告を生成する() | cosine 類似度算出・スキップ記録・集約警告生成 | C0→C4 |
| CR-013 | RBD/SEQD | エッジスコア集合 | スコアを取り出す(エッジ識別子) / スキップ件数を返す() | スコア・スキップ・集約警告保持 | C4→Escore、C6→Escore |
| CR-014 | RBD/SEQD | リンク漏れ候補算出処理 | リンク漏れ候補を算出する(保存済みベクトル集合, 監査設定) | 非エッジペアから閾値以上の候補を算出 | C0→C5 |
| CR-015 | RBD/SEQD | リンク漏れ候補集合 | 候補を列挙する() / 件数を返す() | 候補ペア保持・列挙 | C5→Ecand、C6→Ecand |
| CR-016 | RBD/SEQD | 監査報告生成処理 | 監査報告を生成する(...) / 統計サマリを集計する(エッジスコア集合) | 監査報告生成・統計サマリ集計 | C0→C6 |
| CR-017 | RBD/SEQD | 監査報告 | テキスト形式で直列化する() / 構造化形式で直列化する() | 報告保持・text/JSON 直列化 | C6→Ereport |
| CR-018 | RBD/SEQD | 空ストア通知処理 | 案内情報を生成する() | 空ストア時の案内情報生成 | C3→C7 |
| CR-019 | RBD/SEQD | 監査結果出力窓口 | 監査報告を標準出力へ渡す(監査報告) / 案内情報を標準出力へ渡す(文字列) / 診断情報を標準エラーへ渡す(文字列) | stdout/stderr 区別出力 | C6→B2、C7→B2、C0→B2 |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・境界受付操作（出力形式種別を属性として具体化） | GREEN |
| AR-002 | CR-002 | preserved | 統括 Control。協調操作と失敗伝達操作を識別（RBA「協調」→ SEQD「監査を統括する / 算出失敗を伝達する」） | GREEN |
| AR-003 | CR-003 | preserved | 設定解決 Control が 1 操作「設定を解決する」に具体化 | GREEN |
| AR-004 | CR-004 | preserved | 設定ファイル境界が「設定を読み込む / 設定の存在を確認する」2 操作に具体化（分解は境界の典型的な操作識別） | GREEN |
| AR-005 | CR-005 | preserved | 監査設定 Entity が「設定値を確定する / 設定値を取り出す」に具体化 | GREEN |
| AR-006 | CR-006 | preserved | グラフ定義取得処理 Control が 1 操作「グラフ定義を取得する」に具体化 | GREEN |
| AR-007 | CR-007 | preserved | グラフ定義境界が「全エッジ定義を読み込む / グラフ定義の存在を確認する」2 操作に具体化 | GREEN |
| AR-008 | CR-008 | preserved | グラフエッジ集合 Entity が「エッジを取り出す / 全エッジを列挙する」に具体化 | GREEN |
| AR-009 | CR-009 | preserved | ベクトルロード処理 Control が「全件ベクトルをロードする / 空状態を検知する」に具体化 | GREEN |
| AR-010 | CR-010 | preserved | ベクトルストア境界が「全件ベクトルを読み込む / ストアが空かを確認する」に具体化 | GREEN |
| AR-011 | CR-011 | preserved | 保存済みベクトル集合 Entity が「ベクトルを取り出す / 全件を列挙する / 件数を返す」に具体化 | GREEN |
| AR-012 | CR-012 | preserved | エッジスコア算出処理 Control が「エッジスコアを算出する / スキップエッジを記録する / 集約警告を生成する」3 操作に具体化。SPEC-LGX-010.REQ.09 錨着（UC 未記載だが RBA §6 で正当化済み） | GREEN |
| AR-013 | CR-013 | preserved | エッジスコア集合 Entity が「スコアを取り出す / スキップ件数を返す」に具体化 | GREEN |
| AR-014 | CR-014 | preserved | リンク漏れ候補算出処理 Control が 1 操作「リンク漏れ候補を算出する」に具体化 | GREEN |
| AR-015 | CR-015 | preserved | リンク漏れ候補集合 Entity が「候補を列挙する / 件数を返す」に具体化 | GREEN |
| AR-016 | CR-016 | preserved | 監査報告生成処理 Control が「監査報告を生成する / 統計サマリを集計する」2 操作に具体化 | GREEN |
| AR-017 | CR-017 | preserved | 監査報告 Entity が「テキスト形式で直列化する / 構造化形式で直列化する」2 操作に具体化（text/JSON モード = UC Step 4–5 に対応） | GREEN |
| AR-018 | CR-018 | preserved | 空ストア通知処理 Control が 1 操作「案内情報を生成する」に具体化 | GREEN |
| AR-019 | CR-019 | preserved | 監査結果出力窓口 Boundary が「監査報告を標準出力へ渡す / 案内情報を標準出力へ渡す / 診断情報を標準エラーへ渡す」3 操作に具体化（stdout/stderr 区別 = NFR-LGX-001.OBS.02 に錨着） | GREEN |

19 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-010 §4 mapping が新規クラスなし（RBA 主語と 1:1）を明示確認。

## 4. Role Fitness Check（§5.2）

### Boundary
- Finding: 5 Boundary クラス（監査コマンド受付窓口・設定ファイル境界・グラフ定義境界・ベクトルストア境界・監査結果出力窓口）はすべて境界操作（受け付け・読み込み・確認・出力）のみ保持。Boundary overreach なし。Boundary 同士の直接通信なし（SEQD §7 Noun-Verb ルール確認済み）。
- Severity: なし / 原因の所在: — / Required action: なし

### Control
- Finding: 8 Control（監査統括処理・設定解決処理・グラフ定義取得処理・ベクトルロード処理・エッジスコア算出処理・リンク漏れ候補算出処理・監査報告生成処理・空ストア通知処理）はすべて調停・処理に留まり、データ保持なし。監査統括処理は協調のみで万能化していない（Service blob 化なし）。Control leakage なし。エッジスコア算出処理の「スキップエッジを記録する / 集約警告を生成する」はエッジスコア集合（Entity）が保持するスキップ・警告データを生成するための操作であり、自身にデータを保持していない（blob 化なし）。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity
- Finding: 6 Entity（監査設定・グラフエッジ集合・保存済みベクトル集合・エッジスコア集合・リンク漏れ候補集合・監査報告）は自身のデータ操作のみ。監査報告の「テキスト形式で直列化する / 構造化形式で直列化する」は自身データに対する操作で Entity anemia / overreach なし。エッジスコア集合に「スコアエントリ」「スキップエッジ」「集約警告」の 3 属性群があるが、いずれも算出処理の産物として同一 Entity に集約することが妥当（エッジスコア計測の凝集単位）。Entity anemia なし。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Step 1（`legixy report [--json]` 実行） | Actor→監査コマンド受付窓口→監査統括処理 | 監査要求を受け付ける→監査を統括する | Yes | |
| Step 2（graph.toml パース）サブ: 設定解決 | 監査統括処理→設定解決処理→設定ファイル境界→監査設定 | 設定を解決する→設定の存在を確認する→設定を読み込む→設定値を確定する | Yes | |
| Step 2（グラフ定義取得） | 監査統括処理→グラフ定義取得処理→グラフ定義境界→グラフエッジ集合 | グラフ定義を取得する→グラフ定義の存在を確認する→全エッジ定義を読み込む | Yes | |
| Step 2（ベクトルロード） | 監査統括処理→ベクトルロード処理→ベクトルストア境界→保存済みベクトル集合 | 全件ベクトルをロードする→ストアが空かを確認する→全件ベクトルを読み込む | Yes | |
| Step 3a（全エッジ cosine 類似度算出） | エッジスコア算出処理→グラフエッジ集合/保存済みベクトル集合/監査設定→エッジスコア集合 | エッジスコアを算出する→全エッジを列挙する→ベクトルを取り出す→設定値を取り出す→スコアを取り出す/スキップ件数を返す | Yes | |
| Step 3b（リンク漏れ候補算出） | リンク漏れ候補算出処理→保存済みベクトル集合/監査設定→リンク漏れ候補集合 | リンク漏れ候補を算出する→全件を列挙する→設定値を取り出す | Yes | |
| Step 4（text モード出力） | 監査報告生成処理→エッジスコア集合/リンク漏れ候補集合→監査報告→監査結果出力窓口 | 監査報告を生成する→スコアを取り出す→候補を列挙する→統計サマリを集計する→テキスト形式で直列化する→監査報告を標準出力へ渡す | Yes | |
| Step 5（JSON モード出力） | （Step 4 の形式分岐） | 構造化形式で直列化する（SEQD 代替 5a） | Yes | SEQD §2 代替 5a に独立フローとして展開 |
| Step 6（exit 0） | 監査統括処理が正常完了を確定 | 監査を統括する 戻り = 監査報告 | Yes | |

### Alternative Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 2a（embeddings 空 → INFO + exit 0） | ベクトルロード処理→空ストア通知処理→監査結果出力窓口 | ストアが空かを確認する→空状態を検知する→案内情報を生成する→案内情報を標準出力へ渡す | Yes | 算出処理は起動しない |

### Exception Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外 3a（算出失敗 → exit 1） | エッジスコア算出/リンク漏れ候補算出が失敗を監査統括処理へ伝達 | エッジスコアを算出する(失敗)→算出失敗を伝達する→診断情報を標準エラーへ渡す | Yes | 終了コード 1 |

全 UC フローが SEQA / SEQD 上で責務の不整合なく実行可能。

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None
- **Ambiguous Mappings**: None

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 19 |
| Preserved | 19 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 19 |
| Invention rate（監視用） | 0% |

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

## 9. Required Changes

- なし（保存失敗なし）

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC フロー実行可能）。抽象責務集合（UC 錨着）が具体責務集合へ 1:1 で保存されている。新規クラスの発見なし（RBD §4 確認）。エッジスコア算出処理のスキップ・集約警告責務は SPEC-LGX-010.REQ.09 に錨着済みで正当化されており、発明(invented)にはあたらない。

<!-- VERDICT:APPROVE -->
