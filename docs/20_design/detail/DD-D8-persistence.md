# DD-D8 詳細設計: 保存・中断再開（SaveEnvelope 実装仕様）

- 版: 0.2（起草 cursor-kimi、2026-08-30。0.2: grok 審査 r1（PR #33）反映 — 書込先を「採用しなかったスロット」に修正（唯一正常ファイルの上書き防止）・UT-D8-02 の checksum 再計算・世代選択の純関数化・構造検査の挿入点固定。Claude 一括裁定（D8-Q1 条件付採用 / D8-Q2 採用）を §4/§8 に記録）
- 上位正本: BD-10 0.3（SaveEnvelope・版規則・migration）、BD-01 r4 §5（責務分担・台帳ダイジェスト）、BD-05 §10/§12/§13/§14、BD-03（INV-13/14）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-DET-02/06、REQ-CON-08、REQ-EVT-04、REQ-NFR-02/06、REQ-UI-05）、BD-08 §8（AT-D8-01..05）
- スコープ: envelope の直列化・検証・書込み（2 世代ローテーション）の実装仕様、台帳保持ポリシ、容量実測計画
- 非スコープ: 復帰手順・画面遷移（BD-11 / D12）、自動 save の間隔（D12 の UI スケジューラ。core は冪等な save のみ提供）、SessionSave の内容（D12）
- 前提: BD-10 §2 の 12 フィールド・正規化・検証順は確定済み。本 DD はそれを実装構造に落とす

## 1. 実装構造（確定）

- `crates/sim-core/src/save.rs`: envelope の構築・パース・検証。**core はバイト列（UTF-8 JSON）のみを扱い、ファイル I/O を持たない**（BD-10 §2: WorldSave のパースと生成は core のみ。Presentation は透過）
- ファイル書込み（一時ファイル → fsync → rename、2 世代ローテーション）は Presentation 側の責務だが、手順は本 DD §3 が契約として固定し、D11/D12 が従う
- 直列化は自前の正規化ライタ（BD-10 §2 の正規化: 空白なし・キー順固定・整数 10 進・u64 は 10 進文字列）を使い、serde_json のデフォルト出力を使わない（キー順・数値表現の決定性のため。確定）。再掲: 改行なし・指数表記禁止・checksum 計算時は checksum フィールド自身を入力から除く（BD-10 §2）

## 2. load の検証とエラー（確定。BD-10 §2 の展開）

検証順: checksum → schema_version → model_version → **構造検査** → state・ledger 復元 → state_hash・ledger_hash 突合。

| 失敗 | エラー |
|---|---|
| checksum 不一致 | `KZ_ERR_CHECKSUM` |
| schema_version 不受理（major 相違・minor 超過） | `KZ_ERR_SCHEMA`（migration 経由は §5） |
| model_version 不一致 | `KZ_ERR_MODEL_VERSION`（migration なし。チュートリアル再選定ゲートへ。REQ-DET-06、REQ-DET-09） |
| 構造不整合（cells 件数 ≠ config の width × height 等） | `KZ_ERR_SCHEMA`（state_hash 計算以前に弾く。D8-Q2 裁定済。§8） |
| state_hash / ledger_hash 不一致 | `KZ_ERR_STATE_HASH` |

- 構造検査の挿入点は checksum 後・state_hash 突合前に固定する（順序を入れ替えると SCHEMA / STATE_HASH の区別が反転する）
- ledger_hash は JSON テキストからではなく、復元したレコードから BD-01 r4 §5 の LE 直列化を再構成して再計算する（BD-10 §2 確定どおり）

## 3. 書込み・世代管理（確定。BD-10 §6 の契約化）

1. envelope をメモリで構築し checksum まで確定
2. 書込先は**直近の load で採用しなかったスロット**（欠損・破損スロットを含む）。採用済みの唯一の正常ファイルは決して上書きしない（A gen5 正常・B gen6 破損なら採用は A、書込先は B。古い側へ書く規則では A を壊しえた）。初回起動（両スロット欠損）の書込先は slot A
3. 書込先へ `*.tmp` 書出し → ファイル fsync → 同一ディレクトリ内で rename による原子置換（Windows では既存パスへの rename が原子置換にならないため ReplaceFile 相当の置換 API を使う）→ ディレクトリ fsync（rename 後の dirent 永続化）
4. `generation` は採用した世代 + 1（初回は 1。u64、10 進文字列）
5. 起動時は 2 本を読み、checksum 正常かつ generation 最大を採用。両方破損なら新規開始（BD-11 の S-BOOT へ）。この世代選択は core の純関数（2 本の envelope バイト列 → 採用 / 新規）として実装し、I/O を伴わない（UT-D8-03 が直接検査する）
6. 書込途中 kill は rename により旧世代が残る（AT-D12-ADV-07 がカバー）

## 4. 台帳保持ポリシ（D8-Q1 裁定済。条件付き採用）

- 採用: **(a) 累計レコード**（tick を落とし region × lineage × reason × from × to で全期間集約）+ **(b) 直近 200 tick の per-tick リングバッファ** + **(c) スタンプ前後窓のスナップショット**（v0.5 §7.4）。説明器は (a) を原因集計に、(b)(c) を転換点前後の再生に使う
- (b) の破棄順序: FIFO（最古 tick から破棄）。リングは常に直近 200 tick 分を保持し、201 tick 目の記録時に最古を捨てる
- (c) の窓確定 tick: 転換点スタンプの検出が確定した tick（DD-D9 の検出窓 200 tick がそろった時点）に、その時点の (b) リング内容（= 検出 tick までの直近 200 tick）をコピーして保存する。保存数は BD-10 §5 のスタンプ上限（32 件）に従う
- 条件（Claude 裁定）: ledger_hash（BD-01 r4 §5 / BD-10）の対象集合を (a)+(b)+(c) と明示する BD 改訂 PR を DD-D8 より先にマージする（別 PR。kimi 起草）
- UT-D8-08 の実測は実装 PR の初手で行い、envelope サイズ（REQ-NFR-06・REQ-NFR-02 の容量予算との照合）を記録する

## 5. migration（確定。BD-10 §4 の運用）

- schema major bump PR は旧版 fixture を `docs/30_contracts/golden/saves/` に版名付きで残し、AT-D8-05 が fixture → migration → load → state_hash 照合を機械判定
- migration 関数は改名・追加既定値・削除のみ（値の再計算禁止）。適用後に checksum・state_hash・ledger_hash を新 schema で再計算して封入
- fixture は golden なので更新は Claude 承認（BD-05 §14）

## 6. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D8-01 | 任意の run（1 セル / 64×64）で save → load | state_hash が往復で一致（INV-13/14） |
| UT-D8-02 | schema major+1 / model_version 改変 / state 1 バイト改変。いずれも**改変後に checksum を正規化再計算して封入**（先に checksum で落ちて対象経路に届かないことを防ぐ）。checksum 経路自体は別途 1 バイト改変・再計算なしで検査 | それぞれ §2 の専用エラーで拒否（AT-D8-02 の 4 経路） |
| UT-D8-03 | 世代選択の純関数に 2 本の envelope バイト列を入力（gen 5 正常・gen 6 破損 / 両方破損 / 両方正常で gen 5・7） | 正常最大 gen を採用（5・7 → 7）/ 両方破損は新規開始相当の Err。core の純関数として検査し、ファイル I/O は介さない（§3-5） |
| UT-D8-04 | u64 最大値 18_446_744_073_709_551_615 の seed・prng_state | 10 進文字列で往復一致（2^53 超） |
| UT-D8-05 | 同一 run の save を 2 回 | 正規化バイト列が一致（generation と checksum を除く。checksum は generation 由来で変わる） |
| UT-D8-06 | ledger 部の 1 レコード改変 | ledger_hash 不一致 → `KZ_ERR_STATE_HASH` |
| UT-D8-07 | 三経路: step(2000) vs step(t) → save → load → step(2000−t)、t ∈ {1, 999, 1999} | 最終 state_hash・終了ラベル一致（AT-D8-01 の UT 版） |
| UT-D8-08 | 最悪 config（64×64・4 系統・2,000 tick・ledger 全保持） | envelope サイズを実測し JSON に記録（BD-10 §5 の 2.6 MB 見積の検証。超過は失敗ではなく D8-Q1 の入力） |

## 7. AT 対応（BD-08 §8）

| AT | 対応 |
|---|---|
| AT-D8-01 | UT-D8-07 の固定 3 点（t ∈ {1, 999, 1999}）を CI で実行（ランダム tick ではなく UT と同一の決定的選択） |
| AT-D8-02 | UT-D8-02/06 + save.schema.json 検査 |
| AT-D8-03 | 転換点検出の分布。D8 実装後の較正で確定（OPEN-04）。本 DD は検出器の仕様を持たない（BD-12 / D9） |
| AT-D8-04 | 検出 on/off の hash 一致。検出器が World を書かないことの検査（BD-03 §1.2） |
| AT-D8-05 | §5 の fixture 方式 |

## 8. 裁定結果（Claude 一括裁定 2026-08-30）

- **D8-Q1（条件付き採用）**: 台帳保持ポリシは (a) 累計 + (b) 直近 200 tick リング + (c) スタンプ前後窓（§4）。条件: ledger_hash の対象集合を (a)+(b)+(c) と明示する BD 改訂 PR を DD-D8 より先にマージする。(b) の破棄順序（FIFO）と (c) の窓確定 tick（スタンプ検出確定時に (b) をコピー）は §4 に固定済み。UT-D8-08 の実測は実装初手
- **D8-Q2（採用）**: cells 件数と config 寸法の不整合は `KZ_ERR_SCHEMA`（構造検査として state_hash 突合前に弾く。§2）

## 9. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-core/src/save.rs` | envelope 構築・パース・検証（§1/§2） |
| `docs/30_contracts/save.schema.json` | 12 フィールド・ledger 部の schema 更新（BD-10 §2 どおり） |
| `crates/sim-core/tests/d8_save.rs` | §6 UT |
