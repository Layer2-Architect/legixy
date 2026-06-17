Document ID: GAP-LGX-081

# GAP-LGX-081: custom エッジの「順方向 / 逆方向」のセマンティクスが未定義

**親 TP**: TP-LGX-005
**観点出典**: TP-LGX-005 §2.11 観点 27
**ステータス**: closed (2026-06-10)
**重要度**: minor (verification: low-value, 人間判断で drop 可)
**起票日**: 2026-06-08

> **敵対的精査メモ（2026-06-09）**: SPEC-LGX-002 REQ.04 は 3 種別すべてを有向エッジ（`from`/`to`）としてモデル化しており、有向エッジを「順方向に辿る」は `from`→`to` 追跡を一意に指す。REQ.08（親→サブノード = 順方向）が既にこの一般則を具体化済みで、custom も同じ有向エッジである以上、別扱いの根拠は存在しない。CTX-INV-3（custom 独立性）は走査到達性ではなく**意味的制約**であり、所有は SPEC-LGX-003 REQ.05（compile_context）。残るのは「from→to を順方向とする」を SPEC-005 本文に一文として明記するか否かのみで、決定論的挙動を変えない low-value の文章整備。人間判断で drop 可。

## 1. 観点

REQ.01/REQ.02 は chain / custom / parent_child の 3 種別を「順方向 / 逆方向に辿る」と規定する。chain は成果物連鎖の向き（上流→下流）で順方向が自明、parent_child は REQ.08 で「親→サブノードが順方向」と明示される。しかし `custom` エッジ（人間が graph.toml に明示する任意の参照関係、SPEC-LGX-002 REQ.04）については、何をもって「順方向」とするか（`from`→`to` を順方向とみなすのか）が SPEC-LGX-005 に明文化されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-005 REQ.01/REQ.02 で **「chain / custom / parent_child エッジを順方向/逆方向に辿り」** に触れているが、**custom エッジにおける順方向 = `from`→`to` か、逆方向 = `to`→`from` か** という方向の確定が未定義。

SPEC-LGX-002 REQ.04 は custom を「任意の参照関係」と定義するのみで方向の意味づけはしておらず、CTX-INV-3（カスタムエッジ独立性）は「chain 上流に影響しない」という意味的制約にとどまる。`impact`（順方向）/`investigate`（逆方向）が custom エッジをどちら向きに辿るかは走査の責務であり SPEC-LGX-005 が確定すべき。

## 3. 期待される情報

SPEC-LGX-005 に追加されるべき記述:

- 全エッジ種別で「順方向 = エッジの `from`→`to`、逆方向 = `to`→`from`」と統一する旨の明記（REQ.08 の parent_child 規則をこの一般則の特例として位置づける）
- もし custom のみ別扱い（例: 双方向に辿る等）とするなら、その規則と根拠
- 確定した方向規則が CTX-INV-3（custom が chain 上流に影響しない）と矛盾しないことの確認（`investigate` で custom 逆方向辿りが chain 由来の追跡を汚染しないか）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-005（逆方向探索）/ UC-LGX-006（順方向探索）: custom エッジを含むグラフで `impact`/`investigate` の到達集合が実装者により解釈が分かれる
- 下流の RBA/SEQA/RBD/SEQD/DD: 走査アルゴリズムの隣接展開関数で「どの種別をどちら向きに辿るか」が一意に決まらない
- 他の TP / GAP との依存関係: GAP-LGX-082（種別混在時の整列）と隣接展開規則を共有する

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-005 v0.4.0（人間裁定 fix・承認 2026-06-10）: REQ.01 に方向の一般則を明記 — 全エッジ種別で順方向 = from→to / 逆方向 = to→from（REQ.08 は具体化）、custom の双方向特例なし、CTX-INV-3（意味的制約、所有 SPEC-LGX-003.REQ.05）との無矛盾を注記。

## 6. 関連 ADR

custom エッジ方向の意味づけが CTX-INV-3 と干渉する場合は ADR 起票候補。
