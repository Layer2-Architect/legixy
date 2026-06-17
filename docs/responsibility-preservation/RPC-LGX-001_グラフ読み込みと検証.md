Document ID: RPC-LGX-001

# RPC-LGX-001: グラフ読み込みと検証 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-001
**対象 RBA**: RBA-LGX-001
**対象 SEQA**: SEQA-LGX-001
**対象 RBD**: RBD-LGX-001
**対象 SEQD**: SEQD-LGX-001
**検査深度**: フル（§14.2: 検証=主要成功条件・終了コード契約を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 検証コマンド受付窓口 | 検証要求を受け付ける | UC-001 Step 1 |
| AR-002 | RBA | Control | 検証統括処理 | 検証フローを統括し部分失敗を継続する | UC-001 Step 1–6 |
| AR-003 | RBA | Control | 設定解決処理 | 設定を解決する | UC-001 Step 2 |
| AR-004 | RBA | Boundary | 設定ファイル境界 | 設定を供給する | UC-001 Step 2 / 2a |
| AR-005 | RBA | Entity | 検証設定 | 設定値を保持する | UC-001 Step 2 |
| AR-006 | RBA | Control | グラフ構築処理 | 有向グラフを構築する | UC-001 Step 3 |
| AR-007 | RBA | Boundary | グラフ定義境界 | グラフ定義を供給する | UC-001 Step 3 / 3a |
| AR-008 | RBA | Boundary | 成果物ファイル境界 | 成果物本文・存在・文書識別子を供給する | UC-001 Step 3, 4b, 4c |
| AR-009 | RBA | Entity | 有向グラフ | ノード・エッジを保持し走査・巡回判定する | UC-001 Step 3, 4d, 4f |
| AR-010 | RBA | Control | 形式検証処理 | 形式整合性（ID形式/存在/文書ID/チェーン/孤立/DAG）を検査する | UC-001 Step 4a–f |
| AR-011 | RBA | Control | 意味整合検証処理 | 意味整合性（再定義/数値/サブノード乖離）を検査する | UC-001 Step 4g–i, 4a |
| AR-012 | RBA | Boundary | 意味ベクトル境界 | 意味ベクトルを供給する（不在も許容） | UC-001 Step 4i |
| AR-013 | RBA | Entity | 意味ベクトル | 意味ベクトルを保持する | UC-001 Step 4i |
| AR-014 | RBA | Entity | 検証所見 | 個々の検査結果を保持する | UC-001 Step 4, 5 |
| AR-015 | RBA | Control | 検証結果集約処理 | 所見を集約し成否を確定する | UC-001 Step 5, 6 |
| AR-016 | RBA | Entity | 検証報告 | 区分別件数・成否を保持し終了状態を判定する | UC-001 Step 5, 6 |
| AR-017 | RBA | Boundary | 検証結果出力窓口 | 検証報告とログを区別して出力する | UC-001 Step 5 |

全 AR が UC ステップに紐づく（UC ステップに紐づかない AR なし → 構造翻訳が情報を加えていない、§9 分解(b) 候補なし）。SEQA-001 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 検証コマンド受付窓口 | 検証を受け付ける | アクター境界で検証要求を受理 | Actor→B1 |
| CR-002 | RBD/SEQD | 検証統括処理 | 検証を統括する / 部分失敗を継続判定する | フロー協調・部分失敗継続 | B1→C0, C0→各処理 |
| CR-003 | RBD/SEQD | 設定解決処理 | 設定を解決する | 設定ファイル境界から検証設定を確定 | C0→C1→Bcfg |
| CR-004 | RBD/SEQD | 設定ファイル境界 | 設定を読み込む / 存在を確認する | 設定供給 | C1→Bcfg |
| CR-005 | RBD | 検証設定 | 設定値を取り出す | 設定値保持 | — |
| CR-006 | RBD/SEQD | グラフ構築処理 | 有向グラフを構築する / 未解決エッジを記録する | グラフ構築 | C0→C2 |
| CR-007 | RBD/SEQD | グラフ定義境界 | グラフ定義を読み込む / 存在を確認する | グラフ定義供給 | C2→Bgraph |
| CR-008 | RBD/SEQD | 成果物ファイル境界 | 成果物本文を読み込む / 存在を確認する / 文書識別子行を取り出す | 成果物供給 | C2→Bfile |
| CR-009 | RBD/SEQD | 有向グラフ | ノードを取り出す / 隣接を辿る / 巡回を判定する | グラフ保持・走査 | C3→Egraph |
| CR-010 | RBD/SEQD | 形式検証処理 | 形式整合性を検査する | 形式検証 | C0→C3 |
| CR-011 | RBD/SEQD | 意味整合検証処理 | 意味整合性を検査する | 意味検証 | C0→C4 |
| CR-012 | RBD/SEQD | 意味ベクトル境界 | 意味ベクトルを取り出す / 有無を確認する | ベクトル供給 | C4→Bvec |
| CR-013 | RBD | 意味ベクトル | （ベクトル保持） | ベクトル保持 | — |
| CR-014 | RBD/SEQD | 検証所見 | 検証所見を生成する | 所見保持 | C3/C4→Efind |
| CR-015 | RBD/SEQD | 検証結果集約処理 | 所見を集約する | 所見集約・成否確定 | C0→C5 |
| CR-016 | RBD/SEQD | 検証報告 | 検証報告を構成する / 終了状態を判定する | 報告保持・終了状態判定 | C5→Ereport |
| CR-017 | RBD/SEQD | 検証結果出力窓口 | 検証報告を出力する / ログを出力する | 報告出力 | C5→B2 |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・境界操作 | GREEN |
| AR-002 | CR-002 | preserved | 統括 Control。部分失敗継続を操作化 | GREEN |
| AR-003 | CR-003 | preserved | — | GREEN |
| AR-004 | CR-004 | preserved | — | GREEN |
| AR-005 | CR-005 | preserved | — | GREEN |
| AR-006 | CR-006 | preserved | — | GREEN |
| AR-007 | CR-007 | preserved | — | GREEN |
| AR-008 | CR-008 | preserved | — | GREEN |
| AR-009 | CR-009 | preserved | 走査・巡回判定を Entity 自身の操作に保持 | GREEN |
| AR-010 | CR-010 | preserved | — | GREEN |
| AR-011 | CR-011 | preserved | — | GREEN |
| AR-012 | CR-012 | preserved | — | GREEN |
| AR-013 | CR-013 | preserved | — | GREEN |
| AR-014 | CR-014 | preserved | — | GREEN |
| AR-015 | CR-015 | preserved | — | GREEN |
| AR-016 | CR-016 | preserved | 終了状態判定を報告 Entity に保持 | GREEN |
| AR-017 | CR-017 | preserved | — | GREEN |

17 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-001 §4 mapping が新規クラスなしを確認済み。

## 4. Role Fitness Check（§5.2）

### Boundary
- Finding: 各 Boundary クラス（受付窓口・各供給境界・出力窓口）は境界操作のみ保持。Boundary overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Control
- Finding: 各 Control（統括/設定解決/グラフ構築/形式検証/意味整合検証/集約）は調停・処理に留まり、データ保持なし。Service blob 化なし（統括処理は協調のみで万能化していない）。Control leakage なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity
- Finding: 各 Entity（検証設定/有向グラフ/検証所見/検証報告/意味ベクトル）は自身のデータ操作のみ。有向グラフの走査・巡回判定、検証報告の終了状態判定は自身データに対する操作で Entity anemia / overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Step 1 | Actor→受付窓口→統括 | 検証を受け付ける→検証を統括する | Yes | |
| Step 2 | 統括→設定解決→設定ファイル境界→検証設定 | 設定を解決する→設定を読み込む→設定値を確定する | Yes | |
| Step 3 | 統括→グラフ構築→定義/成果物境界→有向グラフ | 有向グラフを構築する→定義を読み込む→構築する | Yes | |
| Step 4a–f | 形式検証→有向グラフ/設定→検証所見 | 形式整合性を検査する→ノード取得/巡回判定→所見生成 | Yes | |
| Step 4g–i | 意味整合検証→意味ベクトル→検証所見 | 意味整合性を検査する→ベクトル取得→所見生成 | Yes | |
| Step 5 | 集約→検証報告→出力窓口 | 所見を集約する→報告構成→報告出力 | Yes | |
| Step 6 | 集約が成否確定 | 終了状態を判定する | Yes | |

### Alternative Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 4a 無印check | 統括→意味整合検証 | 意味整合性を検査する | Yes | UC-007 へ委譲境界 |
| 2a/3a 不在 | 設定/グラフ解決→不在→致命所見 | 存在を確認する→致命所見生成→集約 | Yes | exit 1 へ収束 |

### Exception Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 部分失敗継続 | グラフ構築→読込失敗→所見→継続 | 成果物本文を読み込む(失敗)→所見生成→部分失敗を継続判定する | Yes | 致命に昇格せず継続 |

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
| Total abstract responsibilities | 17 |
| Preserved | 17 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 17 |
| Invention rate（監視用） | 0% |

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（解消済）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

## 9. Required Changes

- なし（保存失敗なし）

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC フロー実行可能）。抽象責務集合（UC 錨着）が具体責務集合へ 1:1 で保存されている。

<!-- VERDICT:APPROVE -->
