Document ID: GAP-LGX-126

# GAP-LGX-126: engine.db 破損時の Admin コマンド挙動が未定義（DB 不在とは別ケース）

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.2 観点 E5
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

engine.db が「存在するが破損している」（ファイルは読めるがスキーマ不整合・SQLite ファイル破損）場合の feedback / analyze / proposals / approve / reject の挙動が定義されていない。FB-INV-4 が扱うのは DB **不在**であり、**破損**は別ケース。

## 2. 現状の SPEC / UC

SPEC-LGX-007 §4 は FB-INV-4 として「DB 不在時は機能が無効化される」ことのみ扱い、主導を SPEC-LGX-003 に委譲。破損ケースは SPEC-LGX-007 にも SPEC-LGX-003 の FB-INV-4 記述にも現れない。NFR-LGX-001 REL.01 は engine.db 破損耐性（WAL + PRAGMA で電源断耐性）に触れるが、これは「破損防止」であり「破損検出後の各コマンド挙動」ではない。

## 3. 期待される情報

SPEC-LGX-007（または SPEC-LGX-003 へ委譲を明記）に以下を追加すべき:

- 破損検出時に各 Admin/Agent コマンドが exit 1 で明示的に失敗するか、engine.db を再生成（STATE-INV-1: 再生成可能なキャッシュ）してフォールバックするか
- 破損が observation/proposal データの喪失を伴う場合の扱い（再生成不能なユーザ生成データの保護）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: 各コマンドの例外フロー（DB 破損）が具体化できない
- 下流の DD / TS: 破損シミュレーションテスト（NFR REL.01 検証方法）の期待挙動が決まらない
- STATE-INV-1（再生成可能性）と observation/proposal（ユーザ生成データ、再生成不能）の整合が未解決

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.2（人間承認 2026-06-10）: REQ.09 に DB 破損時（不在と区別）「自動再生成せず exit 1」を確定。observation/proposal を再生成不能データとして STATE-INV-1 の例外として保護。§4 FB-INV-4 行更新。ADR-LGX-005。

## 6. 関連 ADR

engine.db を「純粋な再生成可能キャッシュ」とみなすか「observation/proposal はユーザ生成データとして保護対象」とみなすかは architectural 判断のため ADR 起票を推奨。
