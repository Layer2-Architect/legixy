Document ID: RPC-LGX-006

# RPC-LGX-006: 順方向探索 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-006
**対象 RBA**: RBA-LGX-006
**対象 SEQA**: SEQA-LGX-006
**対象 RBD**: RBD-LGX-006
**対象 SEQD**: SEQD-LGX-006
**検査深度**: フル（読み取り専用の BFS 走査・打ち切り制御・起点不在分岐を含む中程度クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 順方向探索コマンド受付窓口 | 開発者 / Linear Agent コンテナからの探索要求（`impact <node-id> [--max-depth <n>]`）を受け取る | UC-006 基本 1 |
| AR-002 | RBA | Control | 順方向探索統括処理 | 探索要求を受け、グラフ読み込み・起点確認・BFS 走査・結果整形を協調させる | UC-006 基本 1–4 |
| AR-003 | RBA | Control | グラフ読み込み処理 | グラフ定義境界から有向グラフを読み込む | UC-006 基本 2（前段） |
| AR-004 | RBA | Boundary | グラフ定義境界 | `graph.toml`（有向グラフ定義）を供給する | UC-006 事前条件 / 基本 2 |
| AR-005 | RBA | Control | 起点確認処理 | 指定された成果物 ID が有向グラフに存在するかを確認する | UC-006 代替 2a |
| AR-006 | RBA | Control | BFS 走査処理 | 有向グラフを順方向（from→to）に BFS 走査し、到達ノードと各ノードの深度を確定する。最大深度指定がある場合は深度上限で走査を打ち切り、打ち切り発生時は打ち切り情報を生成する | UC-006 基本 2・3 / 代替 1a・2a |
| AR-007 | RBA | Control | 走査結果整形処理 | 到達ノード一覧・深度情報を走査結果としてまとめ、走査結果出力窓口へ渡す | UC-006 基本 4 |
| AR-008 | RBA | Boundary | 走査結果出力窓口 | 走査結果（到達ノード一覧・深度情報）を標準出力としてアクターへ返す | UC-006 基本 4 |
| AR-009 | RBA | Entity | 有向グラフ | 読み込まれたノード・エッジの集合（順方向走査の対象データ） | UC-006 基本 2 |
| AR-010 | RBA | Entity | 走査結果 | 到達した全ノード（走査順）と各ノードの起点からの深度の集合 | UC-006 基本 4 / 代替 2a |
| AR-011 | RBA | Entity | 打ち切り情報 | 最大深度指定時に深度超過で除外されたノードが存在する場合に生成される情報（除外発生の事実と除外ノード件数） | UC-006 基本 3 / SEQA §2 |

SEQA-LGX-006 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。打ち切り情報 Entity（AR-011）は RBA-006 §6 Object Discovery で記録済み（SPEC-LGX-005.REQ.04 から導出、UC への遡及反映は人間裁定中）。SEQA を参照して構造上必要なため AR に含める。

全 11 AR が UC ステップ（または SPEC 錨着済み Object Discovery）に紐づく。

## 2. Concrete Responsibilities

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 順方向探索コマンド受付窓口 | 順方向探索を受け付ける | アクター境界で探索要求（起点識別子・最大深度指定）を受理 | Actor→B1 |
| CR-002 | RBD/SEQD | 順方向探索統括処理 | 探索を統括する | グラフ読み込み・起点確認・BFS 走査・結果整形の協調 | B1→C0, C0→各処理 |
| CR-003 | RBD/SEQD | グラフ読み込み処理 | グラフを読み込む / ノードとエッジを構築する | グラフ定義境界から有向グラフを構築する | C0→C1→Bgraph→Egraph |
| CR-004 | RBD/SEQD | グラフ定義境界 | グラフ定義を読み込む / グラフ定義の存在を確認する | グラフ定義の供給 | C1→Bgraph |
| CR-005 | RBD/SEQD | 起点確認処理 | 起点ノードの存在を確認する | 有向グラフに起点識別子を照合し存在有無を返す | C0→C2→Egraph |
| CR-006 | RBD/SEQD | BFS走査処理 | 有向グラフを順方向に走査する / 打ち切り情報を生成する / 空の走査結果を生成する | 有向グラフを順方向 BFS し走査結果と打ち切り情報を記録 | C0→C3→Egraph / Eresult / Etrunc |
| CR-007 | RBD/SEQD | 走査結果整形処理 | 走査結果を整形する | 走査結果・打ち切り情報を読み走査結果出力窓口に渡す | C0→C4→Eresult / Etrunc / B2 |
| CR-008 | RBD/SEQD | 走査結果出力窓口 | 走査結果を出力する / 打ち切り情報を出力する | 整形済み走査結果を標準出力へ返す | C4→B2→Actor |
| CR-009 | RBD/SEQD | 有向グラフ | 起点ノードを照合する / 下流エッジを辿る | 自身ノード・エッジコレクションへの照合と走査供給 | C2/C3→Egraph |
| CR-010 | RBD/SEQD | 走査結果 | 到達ノードと深度を記録する / 空の走査結果を生成する | 到達ノード・深度情報・失敗記録の保持 | C3/C1→Eresult |
| CR-011 | RBD/SEQD | 打ち切り情報 | （打ち切り発生・除外ノード件数を保持） | 打ち切り状態データの保持 | C3→Etrunc |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary、境界操作のみ（起点識別子・最大深度指定の受け取り） | GREEN |
| AR-002 | CR-002 | preserved | 同一 Control 統括。グラフ読み込み・起点確認・BFS 走査・結果整形を依頼する中心として保持 | GREEN |
| AR-003 | CR-003 | preserved | グラフ定義境界から有向グラフを構築する責務が 1:1 で対応 | GREEN |
| AR-004 | CR-004 | preserved | グラフ定義の読み込み・存在確認の操作が Boundary クラスに帰属し保持 | GREEN |
| AR-005 | CR-005 | preserved | 起点確認処理として独立 Control クラスに 1:1 で対応。BFS 走査の越権なし | GREEN |
| AR-006 | CR-006 | preserved | 有向グラフ順方向 BFS・深度打ち切り・打ち切り情報生成・空結果生成が BFS走査処理クラスに保持 | GREEN |
| AR-007 | CR-007 | preserved | 走査結果・打ち切り情報を読み出力窓口へ渡す整形責務が 1:1 で対応 | GREEN |
| AR-008 | CR-008 | preserved | 同一 Boundary、走査結果・打ち切り情報の出力操作のみ | GREEN |
| AR-009 | CR-009 | preserved | ノード・エッジコレクション保持と照合・辿る操作が Entity 自身に帰属 | GREEN |
| AR-010 | CR-010 | preserved | 到達ノード・深度情報・失敗記録の保持と記録操作が走査結果 Entity に保持 | GREEN |
| AR-011 | CR-011 | preserved | 打ち切り発生・除外ノード件数を打ち切り情報 Entity が保持。RBA §6 Object Discovery 正当化済み（SPEC-LGX-005.REQ.04 錨着） | GREEN |

11 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-006 §4 mapping が新規クラスなしを明示。

## 4. Role Fitness Check（§5.2）

### Boundary
- Finding: 各 Boundary クラス（受付窓口・グラフ定義境界・走査結果出力窓口）は境界操作のみ保持。`走査結果出力窓口` の `打ち切り情報を出力する` は Boundary 内の自己メッセージ（SEQD §1: `B2->>B2`）だが、出力先はアクターへの境界操作として一貫している。Boundary overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Control
- Finding: 各 Control（統括/グラフ読み込み/起点確認/BFS 走査/走査結果整形）は調停・処理に留まりデータ保持なし。統括処理は各処理への依頼のみ（Service blob 化なし）。グラフ読み込み処理はグラフ定義境界→有向グラフの変換のみ（走査の越権なし）。起点確認処理は有向グラフへの照合依頼のみ（BFS 走査の越権なし）。BFS 走査処理は走査・記録のみ（整形・出力の越権なし）。Control leakage なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity
- Finding: 有向グラフは自身のノード・エッジコレクションへの照合・辿る操作のみ（走査結果概念の混入なし）。走査結果は到達ノード・深度情報・失敗記録への記録操作のみ（整形ロジックなし）。打ち切り情報は属性保持のみ（操作なし）。Entity anemia / overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow（--max-depth 指定あり）
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 基本 1（impact 実行） | Actor→受付窓口→統括処理 | 順方向探索を受け付ける→探索を統括する | Yes | |
| 基本 2（BFS 走査前: グラフ読込） | 統括→グラフ読み込み→グラフ定義境界→有向グラフ | グラフを読み込む→グラフ定義を読み込む→ノードとエッジを構築する | Yes | |
| 基本 2（起点確認） | 統括→起点確認→有向グラフ | 起点ノードの存在を確認する→起点ノードを照合する | Yes | |
| 基本 2–3（BFS 走査・打ち切り） | 統括→BFS走査→有向グラフ→走査結果・打ち切り情報 | 有向グラフを順方向に走査する→下流エッジを辿る→到達ノードと深度を記録する→打ち切り情報を生成する | Yes | |
| 基本 4（結果整形・出力） | 統括→整形→走査結果・打ち切り情報→出力窓口 | 走査結果を整形する→到達ノードと深度を取り出す→打ち切り情報を取り出す→走査結果を出力する | Yes | |

### Alternative Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 1a（--max-depth 未指定: 全深度走査） | BFS走査処理が深度上限なしで全深度走査、打ち切り情報なし | 有向グラフを順方向に走査する(深度上限なし)→到達ノードと深度を記録する（打ち切り情報生成なし） | Yes | |
| 代替 2a（起点不在: 空結果 exit 0） | 起点確認→不在→BFS走査→空走査結果→整形→出力 | 起点ノードを照合する(存在なし)→空の走査結果を生成する→走査結果を整形する→走査結果を出力する（成功終了） | Yes | SPEC-LGX-005.REQ.05 に錨着 |

### Exception Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外: graph.toml 読込失敗 | グラフ読み込み→グラフ定義境界→失敗→走査結果に失敗記録→整形→エラー出力 | グラフ定義を読み込む(失敗)→空の走査結果を生成する→到達ノードと深度を記録する(失敗記録)→走査結果を整形する(失敗記録)→走査結果を出力する(エラー内容) + 終了状態(失敗) | Yes | UC 事前条件（graph.toml 存在）未充足 |

全 UC フローが SEQA / SEQD 上で責務の不整合なく実行可能。

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。打ち切り情報 Entity は RBA §6 Object Discovery で正当化済み）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None
- **Ambiguous Mappings**: None

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 11 |
| Preserved | 11 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 11 |
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

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC フロー実行可能）。抽象責務集合（UC 錨着）が具体責務集合へ 1:1 で保存されている。打ち切り情報 Entity（AR-011/CR-011）は RBA-006 §6 Object Discovery にて SPEC-LGX-005.REQ.04 を根拠とした正当な導出として記録済みであり invented としてカウントしない。

<!-- VERDICT:APPROVE -->
