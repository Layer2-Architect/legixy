Document ID: ADR-LGX-018

# ADR-LGX-018: TRIAGE §4 RBA 阻害挙動裁定 16 件の一括批准（SPEC 準拠デフォルト）

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（人間裁定、一括批准）
**対象**: TRIAGE-2026-06-13 §4 の RBA 阻害 20 件のうち「要熟慮」4 件（#1/#8/#13/#19）を除く 16 件

## 1. 文脈（Context）

ADR-LGX-014（v3↔SPEC 統治方針 = SPEC 準拠原則 + 個別 ADR）の下、TRIAGE §4 の RBA 阻害挙動裁定のデフォルトが SPEC 寄りに定まった。本 ADR は、単純な v3/SPEC 二択で確定できる 16 件を**推奨 lean のまま一括批准**し、RBA/SEQA 着手の前提を確定する。SPEC 改訂を伴いうる「要熟慮」4 件（compile_context 返却セクション数 / CheckCategory 完全性 / observation 状態モデル / drift exit 非対称）は本 ADR の対象外（別途個別議論）。

## 2. 判断（Decision）— 批准した 16 件

| # | 項目 | 批准した決定 | 区分 |
|---|---|---|---|
| 2 | SUPP-002 S2-08 heading_levels | **h2/h3 固定**（v3 の可変 heading_levels は不採用、将来拡張） | SPEC 準拠 |
| 3 | SUPP-002 S2-09 コードフェンス内 `#` | コードフェンス内 `#` 行は見出し抽出から**除外**（v3 維持、REQ.05 凍結趣旨） | v3 維持（SPEC 沈黙領域） |
| 4 | SUPP-002 S2-18 Levenshtein | refresh-subnodes rename 対応は**閾値なし**（v3 維持、全ペア最小距離貪欲） | v3 維持（SPEC 沈黙） |
| 5 | SUPP-002 S2-23 matrix.md 生成 | Phase1 で matrix.md **生成コマンドを設けない**・不在許容（init からは除去済＝UC-009 C2）。REQ.02 自動生成は将来/申し送り | スコープ |
| 6 | SUPP-003 S2-22 engine.db open/監査ログ | CLI 直接実行時、engine.db が**利用可能な場合のみ** context_log 記録（不在時は記録なしで成功）。STATE-INV-1 / FB-INV-4 整合 | SPEC 準拠解釈 |
| 7 | SUPP-004 D-02 freshness git | Phase1 は **mtime のみ**。`method=git` は将来拡張（指定時の no-op/Warning は DD） | スコープ |
| 9 | SUPP-004 D-07 SubnodeIdCollision データフロー | **parser が縮退を graph 構造体に記録 → validate_graph が finding 化**（check 側で再走査しない）。SPEC-002.REQ.12 分担 | 実装責務 |
| 10 | SUPP-005 investigate --max-depth | **機能化**（SPEC-005.REQ.04 準拠）。v3 の「受理 only・無視」を変更 ★【v3差分・加算的】 | SPEC 準拠（v3挙動変更） |
| 11 | SUPP-005 investigate drift 配線 | engine.db を配線し **suspicious_nodes を実出力**（v3 の恒常空を変更） ★【v3差分・加算的】 | SPEC 準拠（v3挙動変更） |
| 12 | SUPP-006 2.5-e DriftFinding kind | **kind 列挙を新設**（ContentChanged / FileMissing / EmbeddingMissing）し未生成を包含（v3 の無言 skip を変更） ★【v3差分・加算的】 | SPEC 準拠（v3挙動変更） |
| 14 | SUPP-008 2.1 migrate 移行元 | **feedback.db を一次対象**（engine.db user_version=0 も対象）。→ ADR-LGX-015 で確定済 | 既決(ADR-015) |
| 15 | SUPP-008 2.4 `[id.chains]` 変種 | v0.1.0 の `[id.chains]`+`[id.areas]` を**正規変種として抽出受理**（破損扱いしない）。REQ.03 破損判定の限定は申し送り | SPEC 準拠拡張 |
| 16 | SUPP-008 2.6 custom_edges source_glob | migrate で source_glob を**パス→ノード ID 解決**（解決不能は REQ.11 マッピング不可 ID と同扱い）。CTX-INV-2 保全。v3 の転記を変更 ★【v3差分・加算的】 | SPEC 準拠（v3挙動変更） |
| 17 | SUPP-008 2.10 vectors.bin | Phase1 は **Skip+Warning**（embed --all で再生成）。**UC-009 基本フロー Step6「インポートする」との不整合は申し送り**（UC 改訂 or Phase2 解釈） | v3 維持・UC申し送り |
| 18 | SUPP-009 §2.5 get_compile_audit | v3 の省略形整形を**正準化 + SPEC へ明文化**（返却フィールド範囲は v3 形式、MCP-INV-2 解釈） | v3 維持+SPEC明文化 |
| 20 | SUPP-010 D-6 model_version 照合 | `--against` 省略時（embeddings ベースライン）も **model_version 照合を適用**（SCORE-INV-2 忠実）。モデル切替直後・再 embed 前は exit 1 になる旨 SPEC 確認 | SPEC 準拠 |

### ★ v3 観測挙動を変える加算的拡張サブセット（#10/#11/#12/#16）

ADR-LGX-014 が要する「v3 観測挙動を変える採用の記録」を本項で担う。4 件いずれも **SPEC 字義への準拠**（v3 が SPEC 要求を満たしていなかった箇所の実装）であり、新たな SPEC 改訂は不要だが、v3 利用者から見た観測挙動が変わるため:

- 各 SPEC の該当 REQ に **【v3差分】注記**を付す（人間承認時。investigate 出力・DriftFinding・custom_edges 解決）。
- LGX-COMPAT-001 の**引数契約**は不変（出力意味論の SPEC 準拠化であり、サブコマンド/引数/終了コードの凍結対象を侵さない）。
- 後方互換: 既存の引数呼出は壊れない（出力内容が SPEC 準拠に充実する方向の変化）。

将来、いずれかが LGX-COMPAT-001 改訂や SPEC 本文改訂を要すると判明した場合は、その時点で専用 ADR / spec-change を spin off する。

## 3. 結果（Consequences）

- **TRIAGE §4 の 16 件が確定**。RBA/SEQA 着手の前提のうち 16/20 が解決。残るは「要熟慮」4 件 + refs 残課題（CTX-INV-5、VAL-LGX-001）。
- 各決定の具体（JSON スキーマ、終了コード番号、kind 型定義、メッセージ文言、解決順詳細）は DD で凍結（本 ADR の方針下）。
- **申し送り事項**: ①SUPP-008 2.1 の REQ.01 文言（engine.db→feedback.db、ADR-015）②#15 REQ.03 破損判定の限定 ③#17 UC-009 Step6 vectors.bin インポートの UC 改訂要否。いずれも人間承認時に処理。

## 4. 関連

- 統治: ADR-LGX-014（SPEC 準拠原則）
- 連動: ADR-LGX-015（DBパス・#14）, ADR-LGX-016（env）, UC-LGX-009（#5 matrix.md は C2 で除去済、#17 vectors.bin 申し送り）
- トリアージ: TRIAGE-2026-06-13 §4
- 後続（対象外）: 要熟慮 4 件（#1/#8/#13/#19）の個別議論、CTX-INV-5 正準定義（D-09）
