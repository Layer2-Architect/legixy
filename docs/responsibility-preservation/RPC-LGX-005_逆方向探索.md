Document ID: RPC-LGX-005

# RPC-LGX-005: 逆方向探索 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-005
**対象 RBA**: RBA-LGX-005
**対象 SEQA**: SEQA-LGX-005
**対象 RBD**: RBD-LGX-005
**対象 SEQD**: SEQD-LGX-005
**検査深度**: フル（§14.2: 逆方向 BFS 走査・ドリフト評価・DB 不在時安全性を担う中クリティカリティ UC。ただし全フロー網羅のためフル適用）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

RBA-LGX-005 と SEQA-LGX-005 から抽出。各 AR を UC-LGX-005 ステップに紐づける。

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | 逆方向探索コマンド受付窓口 | 逆方向探索要求（起点ノード ID・閾値指定）を受け取る | UC-005 基本 1 |
| AR-002 | RBA | Control | 逆方向探索統括処理 | 探索フロー（閾値解決・グラフ構築・逆方向走査・ドリフト評価・結果整形）を協調させる。DB 不在時はドリフト評価を省略して走査結果のみ返す（FB-INV-4） | UC-005 基本 1–5 / 代替 3a |
| AR-003 | RBA | Control | 閾値解決処理 | `--drift-threshold` 引数の有無を判断し、指定値または設定ファイルのデフォルト値を有効ドリフト閾値として確定する | UC-005 代替 1a |
| AR-004 | RBA | Boundary | 設定ファイル境界 | `--drift-threshold` 未指定時にデフォルトドリフト閾値を供給する | UC-005 代替 1a |
| AR-005 | RBA | Entity | 有効ドリフト閾値 | 閾値解決処理が確定した有効ドリフト閾値の値を保持する | UC-005 代替 1a / 基本 4 |
| AR-006 | RBA | Control | グラフ構築処理 | グラフ定義境界から有向グラフを構築する | UC-005 基本 2 |
| AR-007 | RBA | Boundary | グラフ定義境界 | `graph.toml` からグラフ定義を供給する | UC-005 基本 2（事前条件） |
| AR-008 | RBA | Entity | 有向グラフ | 構築されたノード・エッジの集合。逆方向走査の走査対象 | UC-005 基本 2 |
| AR-009 | RBA | Control | 逆方向走査処理 | 起点ノードから有向グラフを逆方向（上流方向）に幅優先で走査し、訪問ノードと各ノードの深さを記録する | UC-005 基本 2 |
| AR-010 | RBA | Entity | 走査状態 | 逆方向走査中の訪問済みノード・深さ・疑わしいノードを蓄積する | UC-005 基本 2, 4 |
| AR-011 | RBA | Control | ドリフト評価処理 | ドリフトスコア格納境界から各エッジのドリフトスコアを参照し、有効ドリフト閾値以上のエッジを「疑わしい」としてマークして疑わしいノードを確定する。DB 不在時はこの処理を省略する | UC-005 基本 3, 4 / 代替 3a |
| AR-012 | RBA | Boundary | ドリフトスコア格納境界 | `engine.db` から各エッジのドリフトスコアを供給する（不在・未生成時は参照されない） | UC-005 基本 3 / 代替 3a |
| AR-013 | RBA | Entity | ドリフトスコア | ドリフト評価処理が参照する各エッジの意味的類似度スコア | UC-005 基本 3 |
| AR-014 | RBA | Control | 探索結果整形処理 | 走査状態から訪問ノード（走査順）・疑わしいノード（スコア降順）・深さマップを探索結果としてまとめ、探索結果出力窓口に渡す | UC-005 基本 5 |
| AR-015 | RBA | Entity | 探索結果 | 走査結果をまとめた出力データ（訪問ノード・疑わしいノード・深さマップの三者） | UC-005 基本 5 |
| AR-016 | RBA | Boundary | 探索結果出力窓口 | 探索結果（visited / suspicious_nodes / depth_map）をアクターへ返す | UC-005 基本 5 |

全 16 AR が UC ステップに紐づく。SEQA-LGX-005 の時系列メッセージ（基本/代替/例外フロー）はこれら AR の実行順展開であり、UC に対応のない新規 AR を生まない。RBA §6 の Object Discovery（「閾値解決処理」「走査状態」Entity の明示、DB 不在時安全性の構造化）はいずれも既存 UC-005 / SPEC-001 の範囲内の構造化であり、新たな UC ステップを必要とする概念追加ではない。

## 2. Concrete Responsibilities

RBD-LGX-005 と SEQD-LGX-005 から抽出（クラス + 操作）。

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | 逆方向探索コマンド受付窓口 | 逆方向探索を受け付ける | アクター境界で探索要求を受理し統括処理へ渡す | Actor→B1 |
| CR-002 | RBD/SEQD | 逆方向探索統括処理 | 探索を統括する / ドリフトスコア格納先の不在を検知する | 各処理を協調利用。格納先不在時はドリフト評価処理を省略し整形へ進む（FB-INV-4） | B1→C0, C0→各処理 |
| CR-003 | RBD/SEQD | 閾値解決処理 | 閾値を解決する | 引数指定時は要求値、未指定時は設定ファイル境界のデフォルト値を有効ドリフト閾値として確定 | C0→C1 |
| CR-004 | RBD/SEQD | 設定ファイル境界 | 設定を読み込む / デフォルトドリフト閾値を取り出す | 設定ファイルからデフォルトドリフト閾値を供給 | C1→Bcfg |
| CR-005 | RBD/SEQD | 有効ドリフト閾値 | 閾値値を確定する / 閾値値を取り出す | 閾値値を保持し取り出す | C1→Ethr, C4→Ethr |
| CR-006 | RBD/SEQD | グラフ構築処理 | 有向グラフを構築する | グラフ定義境界を読み有向グラフを構築 | C0→C2 |
| CR-007 | RBD/SEQD | グラフ定義境界 | グラフ定義を読み込む / グラフ定義の存在を確認する | グラフ定義を供給 | C2→Bgraph |
| CR-008 | RBD/SEQD | 有向グラフ | ノードとエッジを構築する / ノードの存在を確認する / 上流隣接ノードを辿る | ノード・エッジを保持し走査・存在確認に応じる | C2→Egraph, C3→Egraph |
| CR-009 | RBD/SEQD | 逆方向走査処理 | 起点ノードから逆方向に走査する / 起点ノードの存在を確認する | 有向グラフを上流方向に辿り走査状態を生成・更新 | C0→C3 |
| CR-010 | RBD/SEQD | 走査状態 | 訪問済みとして記録する / 疑わしいとしてマークする / 走査失敗を記録する | 訪問済みノード・深さ・疑わしいノード・走査失敗フラグを保持・操作 | C3→Escan, C4→Escan |
| CR-011 | RBD/SEQD | ドリフト評価処理 | ドリフトを評価する / 閾値以上のノードを疑わしいとしてマークする | ドリフトスコア格納境界を参照し有効閾値と照合、疑わしいノードを走査状態にマーク | C0→C4 |
| CR-012 | RBD/SEQD | ドリフトスコア格納境界 | 格納先の存在を確認する / ドリフトスコアを参照する | ドリフトスコアを供給（不在時は不在を返す） | C4→Bdb |
| CR-013 | RBD/SEQD | ドリフトスコア | スコア値を保持する | エッジのスコア値を保持 | C4→Edrift |
| CR-014 | RBD/SEQD | 探索結果整形処理 | 探索結果を整形する / 訪問ノードを走査順に取り出す / 疑わしいノードをスコア降順に取り出す / 深さマップを取り出す | 走査状態から三者コレクションを取り出し探索結果をまとめ出力窓口へ渡す | C0→C5 |
| CR-015 | RBD/SEQD | 探索結果 | 探索結果を組み立てる / 終了状態を判定する | 三者コレクションを所有し終了状態を判定 | C5→Eresult |
| CR-016 | RBD/SEQD | 探索結果出力窓口 | 探索結果を出力する / エラー結果を出力する | 探索結果またはエラー結果をアクターへ出力 | C5→B2 |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary。境界操作（受け付ける）のみ保持 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Control。統括・DB 不在検知責務が操作（探索を統括する / ドリフトスコア格納先の不在を検知する）に正確に操作化 | GREEN |
| AR-003 | CR-003 | preserved | 同一 Control。引数有無判断・設定ファイル参照・閾値確定責務が操作（閾値を解決する）に保存 | GREEN |
| AR-004 | CR-004 | preserved | 同一 Boundary。デフォルト閾値供給責務が（設定を読み込む / デフォルトドリフト閾値を取り出す）に保存 | GREEN |
| AR-005 | CR-005 | preserved | 同一 Entity。閾値値保持責務が（閾値値を確定する / 閾値値を取り出す）に保存。閾値確定元属性が追加されているが概念型による属性識別であり RBD §4 の範囲内 | GREEN |
| AR-006 | CR-006 | preserved | 同一 Control。グラフ構築責務が（有向グラフを構築する）に保存 | GREEN |
| AR-007 | CR-007 | preserved | 同一 Boundary。グラフ定義供給責務に加え、グラフ定義の存在確認操作が追加されている。これは RBD §4 の「操作識別」の一環（具体クラス設計上の自然な split）であり、AR の責務範囲（グラフ定義を供給する）から外れない justified split | GREEN |
| AR-008 | CR-008 | preserved | 同一 Entity。ノード・エッジ保持・走査・存在確認責務が（ノードとエッジを構築する / ノードの存在を確認する / 上流隣接ノードを辿る）に保存 | GREEN |
| AR-009 | CR-009 | preserved | 同一 Control。逆方向 BFS 走査・深さ記録責務が（起点ノードから逆方向に走査する / 起点ノードの存在を確認する）に保存。起点ノード存在確認は例外フロー実現に必要な操作識別であり AR-009 の走査責務の範囲内 | GREEN |
| AR-010 | CR-010 | preserved | 同一 Entity。走査状態の蓄積・操作責務が（訪問済みとして記録する / 疑わしいとしてマークする / 走査失敗を記録する）に保存。走査失敗フラグ属性は例外フロー実現のための属性識別であり AR-010 の責務範囲内 | GREEN |
| AR-011 | CR-011 | preserved | 同一 Control。ドリフト評価・疑わしいノード確定・DB 不在時省略責務が（ドリフトを評価する / 閾値以上のノードを疑わしいとしてマークする）に保存。DB 不在時省略の判断は CR-002（統括）が格納先不在を検知して省略するフローで担保 | GREEN |
| AR-012 | CR-012 | preserved | 同一 Boundary。ドリフトスコア供給責務が（格納先の存在を確認する / ドリフトスコアを参照する）に保存。格納先存在確認は DB 不在時安全性（FB-INV-4）の実現に不可欠 | GREEN |
| AR-013 | CR-013 | preserved | 同一 Entity。スコア保持責務が（スコア値を保持する）に保存 | GREEN |
| AR-014 | CR-014 | preserved | 同一 Control。整形責務が（探索結果を整形する / 訪問ノードを走査順に取り出す / 疑わしいノードをスコア降順に取り出す / 深さマップを取り出す）に保存。操作の細分化は UC 基本 5 の三者コレクション形式（visited / suspicious_nodes / depth_map）に対応する正当な操作識別 | GREEN |
| AR-015 | CR-015 | preserved | 同一 Entity。探索結果保持責務が（探索結果を組み立てる / 終了状態を判定する）に保存。終了状態判定操作は UC 事後条件（読み取り専用操作）の成否判定に相当し、AR-015 の責務範囲内 | GREEN |
| AR-016 | CR-016 | preserved | 同一 Boundary。出力責務が（探索結果を出力する / エラー結果を出力する）に保存。エラー結果出力は例外フロー実現のための境界操作であり AR-016 の範囲内 | GREEN |

16 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-005 §4 が新規クラス発見なしを明示確認済み。

## 4. Role Fitness Check（§5.2）

### Boundary

- Finding: 逆方向探索コマンド受付窓口・設定ファイル境界・グラフ定義境界・ドリフトスコア格納境界・探索結果出力窓口の各クラスは境界操作のみ保持（供給・受理・出力）。Control の協調ロジックや Entity のデータ操作を担っていない。Boundary overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Control

- Finding: 閾値解決処理・グラフ構築処理・逆方向走査処理・ドリフト評価処理・探索結果整形処理は各々の協調・処理に留まり、データを自身に保持しない（属性なし）。逆方向探索統括処理は協調のみ担い万能化していない（データ保持・境界直接アクセス・走査ロジック保持なし）。Service blob 化なし。Control leakage なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity

- Finding: 有効ドリフト閾値・有向グラフ・走査状態・ドリフトスコア・探索結果は自身のデータ操作のみ担う。有向グラフが「上流隣接ノードを辿る」操作を持つのは自身データに対する走査であり Entity anemia / overreach なし。走査状態が「訪問済みとして記録する」「疑わしいとしてマークする」「走査失敗を記録する」を持つのも自身データ操作のみ。探索結果が「終了状態を判定する」を持つのも自身の成否フラグへのアクセスであり overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow（`investigate <node-id> --drift-threshold <val>`）

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 基本 1（investigate 実行） | Actor→受付窓口→統括 / 逆方向探索を要求する | 逆方向探索を受け付ける→探索を統括する | Yes | |
| 代替 1a 分岐（閾値指定あり） | 統括→閾値解決 / 要求値を有効ドリフト閾値として確定 | 閾値を解決する→閾値値を確定する | Yes | |
| 基本 2（逆方向 BFS 走査） | 統括→グラフ構築→グラフ定義境界→有向グラフ / 統括→逆方向走査→有向グラフ→走査状態 | 有向グラフを構築する→グラフ定義を読み込む→ノードとエッジを構築する / 起点ノードから逆方向に走査する→上流隣接ノードを辿る→訪問済みとして記録する | Yes | |
| 基本 3（ドリフトスコア参照） | ドリフト評価→ドリフトスコア格納境界→ドリフトスコア | ドリフトを評価する→格納先の存在を確認する→ドリフトスコアを参照する→スコア値を保持する | Yes | |
| 基本 4（疑わしいノードをマーク） | ドリフト評価→有効ドリフト閾値→走査状態 | 閾値値を取り出す→疑わしいとしてマークする | Yes | |
| 基本 5（結果返却） | 整形→走査状態→探索結果→出力窓口 | 探索結果を整形する→訪問ノードを走査順に取り出す / 疑わしいノードをスコア降順に取り出す / 深さマップを取り出す→探索結果を組み立てる→探索結果を出力する | Yes | |

### Alternative Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 1a（`--drift-threshold` 未指定） | 統括→閾値解決→設定ファイル境界→有効ドリフト閾値 | 閾値を解決する(閾値指定なし)→デフォルトドリフト閾値を取り出す→閾値値を確定する | Yes | 以降は基本フローに合流 |
| 代替 3a（embedding 未生成） | 統括がDB不在を検知→ドリフト評価処理を省略→整形へ | 格納先の存在を確認する(不在)→走査状態(疑わしいノードなし)→探索結果を組み立てる(空コレクション) | Yes | FB-INV-4。suspicious_nodes は空で出力 |

### Exception Flows

| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 例外（起点ノードがグラフ上に不在） | 統括→逆方向走査→有向グラフ（不在）→走査失敗→整形→エラー結果出力 | ノードの存在を確認する(不在)→走査失敗を記録する→エラー結果を出力する | Yes | 終了状態種別（失敗）。終了コード 1 |

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
| Total abstract responsibilities | 16 |
| Preserved | 16 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 16 |
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

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC 基本・代替・例外フローすべて実行可能）。抽象責務集合（UC 錨着）が具体責務集合へ 1:1 で保存されている。RBD-LGX-005 §4 が新規クラス発見なし（RBA 主語と 1:1）を確認済みであり、本検査でも同一を確認した。

<!-- VERDICT:APPROVE -->
