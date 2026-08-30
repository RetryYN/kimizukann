# DD-D9 詳細設計: 説明器（転換点検出・理由コード・4 段出力）

- 版: 0.2（起草 cursor-kimi、2026-08-30。0.2: grok 審査 r1（PR #34）反映 — x_latest の窓含有・非 z 種別の対象系列表・event_id 採番・理由コード入力源・除算 0・trace 用 REQ 個別言及。D9-Q1 裁定（採用）を §8 に記録）
- 上位正本: BD-12 0.3（イベント 3 層・7 種別・スコア式・理由コード・レバー写像）、BD-01 r4 §4/§5、BD-03 §1.2（Explain は World を読むだけ）、BD-07 §4.1（解析 crate の f64 例外）、BD-08 §9、BD-10（LedgerSave）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-EVT-01..05、REQ-EXP-01..06、REQ-OUT-05）
- スコープ: BD-12 が「詳細設計で固定する」とした集計の実装詳細（窓・閾値の初期値・計算手順）、Explain 関数の入出力、禁止語リントの適用点
- 非スコープ: 文言本文・禁止語リスト（BD-02 §10.2 が正本）、説明画面の UI（D12）、台帳保持ポリシ（D8-Q1）
- 配置: 新規 crate `crates/sim-analysis`（f64 許可の例外 crate。BD-07 §4.1）。core 状態へ逆流しない

## 1. Explain の契約（確定。BD-12 §1）

- 純関数 `explain(snapshot: &SnapshotView, ledger: &LedgerView, query: &ExplainQuery) -> Vec<u8>`（JSON 出力）。入力を変えず、イベント・PRNG を消費しない
- クエリ種別: `Current`（今どうなってる？）/ `Stamp { event_id }`（スタンプ詳細）
- SnapshotView = 読取専用の World 参照、LedgerView = region 集約レコード・スタンプ列・z 窓（BD-10 §1 LedgerSave と同じ構成）
- 検出の on/off・Explain の呼出は state hash に影響しない（AT-D8-04 で検査）

## 2. 転換点検出器（確定。BD-12 §2/§3 の実装固定）

### 2.1 時系列と z スコア

- 系列: `biomass[L]`（系統別セル合計）・`nutrient`・`carcass`・`waste` を 10 tick ごとに平均し、窓 = 直近 20 サンプル（= 200 tick）
- 窓の 20 サンプルには `x_latest`（最新サンプル）を**含める**（除外すると「平坦 19 + 末尾スパイク」が std = 0 → z = 0 で検出不能になる）。mean / std は x_latest を含む 20 点で計算する
- ウォームアップ: サンプル数 < 20（tick < 200）の間は検出しない（短い N では計算しない）
- `z = (x_latest − mean) / std`（**母標準偏差**: `std = sqrt( Σ(x − mean)² / 20 )`、N = 20 で除算（N−1 ではない）。f64）。`std = 0` のとき z = 0（ゼロ除算回避。確定）
- 急増/急減は z の符号で分岐（BD-12 §2 どおり）

### 2.1.1 種別ごとの対象系列（z・継続率・影響生体量比の入力。確定）

トリガは BD-12 §2 の表どおりだが、スコアの z 項・継続率（窓内で |z| ≥ 1 のサンプル割合）・影響生体量比は以下の対象系列から計算する。これにより非 z トリガ種別でもスコアが検出閾値 1.2 に到達可能となる（z = 0 固定では上限 0.65 < 1.2 で数学的に検出不能となるため）:

| 種別 | 対象系列 |
|---|---|
| 急増 / 急減 | 系統 × pool の 10 tick 平均系列（BD-12 §2 どおり） |
| 資源枯渇 | 当該 region の nutrient 系列 |
| 初死骸利用 | 当該系統の carcass プール系列（全体合計） |
| 逆転 | 窓終点で 1 位の系統の総生体量系列 |
| 絶滅 | 当該系統の総生体量系列 |
| 固定候補 | 当該系統の総生体量系列（share ではなく biomass。窓内変化を見るため） |

### 2.2 スコア項の計算順（確定）

1. トリガ判定（BD-12 §2 の表どおり）
2. 発火した種別についてスコア = `0.5·min(|z|,3) + 0.2·継続率 + 0.2·影響生体量比 + 0.1·新規性 + 種別ボーナス`（各項は BD-12 §3 の定義どおり。z・継続率・影響生体量比の対象系列は §2.1.1）
3. `score ≥ 1.2` で検出確定。確定したイベントへ **event_id を採番**してから後処理する: event_id は検出確定順の連番（0 始まり）。同 tick に複数確定した場合は種別 enum 順 → 対象系列 id 昇順の順に採番する（決定性のため）
4. 近接同種は 30 tick 半径で高スコア側に集約。スコア同率は event_id 昇順で残す
5. 上限: 皿上 3（スコア降順）／タイムライン 12／保存 32。超過時はスコア降順で切捨て（同率は event_id 昇順。採番済みのため決定的）

### 2.3 region_ids の派生（確定。BD-12 §1 の二層）

スタンプの region_ids はイベント tick の占有マスク（Σbiomass > 0）の 4 連結成分を row-major 初出順に採番（最大 16、超過は 15 に併合）。派生 ID は LedgerRecord に書かない

## 3. 理由コードの実装（確定。BD-12 §4 の窓・閾値を固定）

- 集計窓: 直近 20 サンプル（= 200 tick。検出窓と同一。D9-Q1 裁定済。§8）
- 閾値の初期仮説（D8 較正で確定。OPEN-04）: `θ_skew = 0.5`、`θ_repro = 0.5`、`θ_niche = 0.1`、`θ_disp = 4`
- 計算は LedgerView の region 集約レコードから行い、セル粒度の生台帳を要求しない（D8-Q1 の累計 + リング構成と両立するよう、直近窓分はリングバッファ、それより古い累計は累計レコードから読む。確定）
- 入力源の固定（LedgerView に無いもの）: `NICHE_MISMATCH` の region プール量は **SnapshotView** から読む（LedgerRecord からは出せない）。`LOW_DISPERSION` の tick 0 占有セル数は create 時に `tick0_occupancy: [u16; L]` として run 状態に保存し（tick0_ranking と同様に create 時確定・不変）、Explain 入力に含める
- 除算の分母 0: 全 region の Intake = 0（RESOURCE_SKEW の分母）や正の biomass 増分 = 0（REPRO_DRIVEN の分母）のとき、当該統計量は stat = 0 として不発（発火しない）とする
- 出力は `top_contributors`（理由コード + 統計量の実測値、最大 3 件、統計量降順。同率は理由コードの enum 順）

## 4. 出力構造（確定。REQ-EXP-01/02/05/06）

```json
{
  "query": { "kind": "Current" },
  "facts": ["<観測事実テンプレートに実測値を束縛>"],
  "interpretation": { "top_contributors": [{ "reason": "REPRO_DRIVEN", "stat": 0.62 }] },
  "unknowns": ["<データ不足の明示テンプレート>"],
  "next_step": { "lever": "個体数", "reason": "REPRO_DRIVEN" }
}
```

- 4 段（facts / interpretation / unknowns / next_step）固定。事実・推論・未知はフィールドで区別（表示側の区画分けの根拠。REQ-EXP-02）
- レバー写像は BD-12 §4 の 5 行表どおり（NONE → 配置）
- 絶滅時も top_contributors と next_step を必ず出力（REQ-EXP-05）
- テンプレートは理由コード × 4 段の固定文面表（文言本文は BD-02 §10.2 が正本）。生成 AI・外部 solver は使わない（REQ-EXP-03、REQ-OUT-05）
- schema は `docs/30_contracts/explanation.schema.json`（実装 PR で新設）

## 5. 禁止語リントの適用点（確定。REQ-EXP-06）

- テンプレート表・出力 JSON の双方に対し、CI の文言リント（BD-02 §10.2 の禁止語リスト）を適用する。適用点は (a) テンプレート表のソース（INSP）、(b) Explain 出力のスナップショットテスト（UT-D9-06）

## 6. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D9-01 | 固定系列（mean = 100, std = 10, x = 130） | z = 3.0 → clip3 で 3.0、スコア項 0.5×3 = 1.5 ≥ 1.2 で検出 |
| UT-D9-02 | std = 0 の系列 | z = 0、検出なし（ゼロ除算なし） |
| UT-D9-03 | 同種別が 30 tick 内に 2 件（score 1.3 / 1.5） | 1.5 に集約され 1 件 |
| UT-D9-04 | 検出 33 件 | 保存 32 件に切捨て（スコア降順・同率 event_id 昇順） |
| UT-D9-05 | RESOURCE_SKEW 統計: region Intake [50, 25, 15, 10]（合計 100）/ [51, 25, 15, 9]（合計 100） | 上位シェア 0.50 ≤ θ_skew で不発 / 0.51 > θ_skew で発火（境界。θ 超は厳密不等号） |
| UT-D9-06 | 全テンプレート × 代表束縛で Explain 出力 | 禁止語リスト非含有・4 フィールド存在・schema 通過 |
| UT-D9-07 | 絶滅 run で explain(Current) | top_contributors と next_step が非空（REQ-EXP-05） |
| UT-D9-08 | 検出 on/off の同一 seed 2 run | state hash 一致（AT-D8-04 の UT 版） |
| UT-D9-09 | REPRO_DRIVEN 統計: 増分系列 [+5, −3, +7, 0] / 正の増分なし | 分母は正の増分のみ（5+7=12）で計算 / 分母 0 → stat = 0 で不発（BD-12 r2） |

## 7. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-analysis/src/detect.rs` | 時系列・z・スコア・集約（§2） |
| `crates/sim-analysis/src/reasons.rs` | 理由コード統計（§3） |
| `crates/sim-analysis/src/explain.rs` | 純関数 explain・テンプレート束縛（§4） |
| `docs/30_contracts/explanation.schema.json` | §4 schema |
| `crates/sim-analysis/tests/d9_explainer.rs` | §6 UT |

## 7.5 REQ 対応（trace 用に個別列挙）

| REQ | 対応 |
|---|---|
| REQ-EVT-01 | §2.2 スコア式の計算順（UT-D9-01/02） |
| REQ-EVT-02 | §2.2 の閾値 1.2・半径 30・上限 3/12/32（UT-D9-03/04） |
| REQ-EVT-03 | §2.1.1 の 7 種別と対象系列 |
| REQ-EVT-04 | §2.3 の region_ids 派生（4 連結成分・最大 16） |
| REQ-EVT-05 | §1（Explain は PRNG・状態を消費しない。UT-D9-08） |
| REQ-EXP-01 | §4 の 4 段固定（UT-D9-06） |
| REQ-EXP-02 | §4 の facts / interpretation / unknowns 区別 |
| REQ-EXP-03 | §4 テンプレート束縛（生成 AI 不使用） |
| REQ-EXP-04 | §4 のレバー写像（BD-12 §4 の 5 行表どおり） |
| REQ-EXP-05 | UT-D9-07（絶滅時の出力保証） |
| REQ-EXP-06 | §5 の禁止語リント適用点 |
| REQ-OUT-05 | §4（外部 solver 不使用） |

## 8. 裁定結果（Claude 一括裁定 2026-08-30）

- **D9-Q1（採用）**: 理由コードの集計窓は検出窓と同一の 20 サンプル（200 tick）。§3 に確定済み
