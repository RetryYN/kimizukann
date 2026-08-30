# DD-D8 詳細設計: 保存・中断再開（SaveEnvelope 実装仕様）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 上位正本: BD-10 0.3（SaveEnvelope・版規則・migration）、BD-01 r4 §5（責務分担・台帳ダイジェスト）、BD-05 §10/§12/§13/§14、BD-03（INV-13/14）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-DET-02/06、REQ-CON-08、REQ-EVT-04、REQ-NFR-02/06、REQ-UI-05）、BD-08 §8（AT-D8-01..05）
- スコープ: envelope の直列化・検証・書込み（2 世代ローテーション）の実装仕様、台帳保持ポリシ、容量実測計画
- 非スコープ: 復帰手順・画面遷移（BD-11 / D12）、自動 save の間隔（D12 の UI スケジューラ。core は冪等な save のみ提供）、SessionSave の内容（D12）
- 前提: BD-10 §2 の 12 フィールド・正規化・検証順は確定済み。本 DD はそれを実装構造に落とす

## 1. 実装構造（確定）

- `crates/sim-core/src/save.rs`: envelope の構築・パース・検証。**core はバイト列（UTF-8 JSON）のみを扱い、ファイル I/O を持たない**（BD-10 §2: WorldSave のパースと生成は core のみ。Presentation は透過）
- ファイル書込み（一時ファイル → fsync → rename、2 世代ローテーション）は Presentation 側の責務だが、手順は本 DD §3 が契約として固定し、D11/D12 が従う
- 直列化は自前の正規化ライタ（BD-10 §2 の正規化: 空白なし・キー順固定・整数 10 進・u64 は 10 進文字列）を使い、serde_json のデフォルト出力を使わない（キー順・数値表現の決定性のため。確定）

## 2. load の検証とエラー（確定。BD-10 §2 の展開）

検証順: checksum → schema_version → model_version → state・ledger 復元 → state_hash・ledger_hash 突合。

| 失敗 | エラー |
|---|---|
| checksum 不一致 | `KZ_ERR_CHECKSUM` |
| schema_version 不受理（major 相違・minor 超過） | `KZ_ERR_SCHEMA`（migration 経由は §5） |
| model_version 不一致 | `KZ_ERR_MODEL_VERSION`（migration なし。チュートリアル再選定ゲートへ。REQ-DET-09） |
| state_hash / ledger_hash 不一致 | `KZ_ERR_STATE_HASH` |

- ledger_hash は JSON テキストからではなく、復元したレコードから BD-01 r4 §5 の LE 直列化を再構成して再計算する（BD-10 §2 確定どおり）
- cells 件数 ≠ config の width × height は `KZ_ERR_STATE_HASH` ではなく `KZ_ERR_SCHEMA`（構造不整合として先に弾く。初期仮説）

## 3. 書込み・世代管理（確定。BD-10 §6 の契約化）

1. envelope をメモリで構築し checksum まで確定
2. 古い側の世代ファイルへ `*.tmp` 書出し → fsync → rename（原子）
3. `generation` は前回 + 1（u64、10 進文字列）
4. 起動時は 2 本を読み、checksum 正常かつ generation 最大を採用。両方破損なら新規開始（BD-11 の S-BOOT へ）
5. 書込途中 kill は rename により旧世代が残る（AT-D12-ADV-07 がカバー）

## 4. 台帳保持ポリシ（D8-Q1。§8）

- 現状の問題: region 集約 LedgerRecord は tick をキーに含むため run 中に tick 線形で増大し、最悪 2,000 tick × 16 region × 4 系統 × 複数 reason で BD-10 §5 の見積（ledger ≤ 0.3 MB）を超えうる
- 推奨案: (a) 累計レコード（tick を落とし region × lineage × reason × from × to で全期間集約）+ (b) 直近 200 tick の per-tick リングバッファ + (c) スタンプ前後窓のスナップショット（v0.5 §7.4）。説明器は (a) を原因集計に、(b)(c) を転換点前後の再生に使う
- 裁定までは BD-10 §5 の見積どおり全 tick 保持を前提とし、実装 PR 初手の MEAS（UT-D8-08）で実測してから確定する

## 5. migration（確定。BD-10 §4 の運用）

- schema major bump PR は旧版 fixture を `docs/30_contracts/golden/saves/` に版名付きで残し、AT-D8-05 が fixture → migration → load → state_hash 照合を機械判定
- migration 関数は改名・追加既定値・削除のみ（値の再計算禁止）。適用後に checksum・state_hash・ledger_hash を新 schema で再計算して封入
- fixture は golden なので更新は Claude 承認（BD-05 §14）

## 6. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D8-01 | 任意の run（1 セル / 64×64）で save → load | state_hash が往復で一致（INV-13/14） |
| UT-D8-02 | checksum 1 バイト改変 / schema major+1 / model_version 改変 / state 1 バイト改変 | それぞれ §2 の専用エラーで拒否（AT-D8-02 の 4 経路） |
| UT-D8-03 | 2 世代ファイル（gen 5 正常・gen 6 破損 / 両方破損） | 正常最大 gen を採用 / 両方破損は新規開始相当の Err |
| UT-D8-04 | u64 最大値 18_446_744_073_709_551_615 の seed・prng_state | 10 進文字列で往復一致（2^53 超） |
| UT-D8-05 | 同一 run の save を 2 回 | 正規化バイト列が一致（generation を除く） |
| UT-D8-06 | ledger 部の 1 レコード改変 | ledger_hash 不一致 → `KZ_ERR_STATE_HASH` |
| UT-D8-07 | 三経路: step(2000) vs step(t) → save → load → step(2000−t)、t ∈ {1, 999, 1999} | 最終 state_hash・終了ラベル一致（AT-D8-01 の UT 版） |
| UT-D8-08 | 最悪 config（64×64・4 系統・2,000 tick・ledger 全保持） | envelope サイズを実測し JSON に記録（BD-10 §5 の 2.6 MB 見積の検証。超過は失敗ではなく D8-Q1 の入力） |

## 7. AT 対応（BD-08 §8）

| AT | 対応 |
|---|---|
| AT-D8-01 | UT-D8-07 を CI の決定的 tick 選択で実行 |
| AT-D8-02 | UT-D8-02/06 + save.schema.json 検査 |
| AT-D8-03 | 転換点検出の分布。D8 実装後の較正で確定（OPEN-04）。本 DD は検出器の仕様を持たない（BD-12 / D9） |
| AT-D8-04 | 検出 on/off の hash 一致。検出器が World を書かないことの検査（BD-03 §1.2） |
| AT-D8-05 | §5 の fixture 方式 |

## 8. 未決事項（claude 裁定依頼）

- **D8-Q1**: 台帳保持ポリシ（§4）。推奨: 累計 + リングバッファ 200 tick + スタンプ窓。BD-03 ADR 候補 2（セル粒度は保存しない）と REQ-EVT-04（5 MB）を両立させる
- **D8-Q2**: cells 件数と config 寸法の不整合を `KZ_ERR_SCHEMA` に振る初期仮説（§2）の承認。state_hash 経路と分ける理由は、構造不正はハッシュ計算以前に弾くべきため

## 9. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-core/src/save.rs` | envelope 構築・パース・検証（§1/§2） |
| `docs/30_contracts/save.schema.json` | 12 フィールド・ledger 部の schema 更新（BD-10 §2 どおり） |
| `crates/sim-core/tests/d8_save.rs` | §6 UT |
