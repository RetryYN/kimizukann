# BD-03 ドメインモデル

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）、`docs/30_contracts/simulation_contract.md` v0.1（契約 §n で参照）
- 完成条件: 各不変条件に property test の雛形（入力生成・assert）が付く
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。確定の根拠は契約節または REQ
- 状態機械は BD-04、公開 API・FFI は BD-05、ビット幅の証明は BD-06、乱数消費回数は BD-07 を参照

## 1. 集約

### 1.1 集約一覧

| 集約 | ルート型 | 構成要素 | 責務 | 参照 |
|---|---|---|---|---|
| World | `WorldState` | `tick: u32`、`GridState`、`Vec<LineageParams>` | 1 run の全状態を所有し、7 phase の固定順適用を保証する | REQ-SIM-01, REQ-SIM-04 |
| Cell | `CellState` | `nutrient / biomass[L] / carcass / waste / energy[L] / occupancy_peak`（L ≤ 8）の 6 状態のみ | セル内の物質・エネルギー量を保持する。6 状態以外を持たない | REQ-SIM-01 |
| Ledger | `MassLedger` / `EnergyLedger` | 追記のみの `Vec<LedgerEntry { tick, cell_index, lineage, from_pool, to_pool, amount, reason }>`（フィールドは BD-05 §3） | 全変換を理由コード付きで記録し、負値・未記録残差を禁止する | REQ-SIM-05, REQ-SCOPE-04 |
| Rng | `PrngState` | `seed: Seed` + 4 ストリーム（movement / reproduction / mutation / interaction）の内部状態 | 用途別乱数供給。SplitMix64 で seed から 4 ストリームを導出し、各ストリームは xoshiro256**（確定） | REQ-DET-04a |
| Termination | `TerminationRule` / `Thresholds` / `TerminationLabel` | 5 ラベル、判定タイミング（EveryTick / AtTimeLimit）、優先度 | 終了条件の判定とラベル確定。判定理由を保存する | REQ-END-01, REQ-END-04c |
| Save | `SaveEnvelope` / `StateSnapshot` / `StateHash` | `schema_version / model_version / config_hash / seed / prng_state / state_hash / state` | 永続化と再開。load 時に checksum・schema_version・model_version を検証する | REQ-DET-06, REQ-CON-08 |

### 1.2 集約間の依存方向

- World → Cell、World → LineageParams を所有する。Cell は他セルを参照しない（拡散は World が 4 近傍を仲介する）。参照: REQ-SIM-10
- World → Ledger へ追記する。Ledger は World を参照しない（逆流なし）。参照: REQ-SIM-05
- Termination → World を読むだけで書かない。参照: REQ-END-02
- Save → World と Rng を読むだけで書かない。参照: REQ-DET-06
- Rng は World と独立に保持し、state hash の正規化バイト列には World と Rng の両方を含める。参照: REQ-DET-05
- UI・描画・転換点検出は World を読むだけで書き戻さない。表示用トークンは計算に作用しない。参照: REQ-CON-05, REQ-OUT-04, REQ-EVT-05

## 2. 値オブジェクト

| 名前 | 型・値域 | 確定度 | 参照 |
|---|---|---|---|
| `Fixed` | i64 固定小数点、scale = 1_000_000（10 進 6 桁）。乗算中間は i128、除算はゼロ方向丸め 1 種 | 確定 | REQ-CON-02 |
| `Pool` | Nutrient / Biomass / Carcass / Waste の 4 種。毒は独立プールを持たず waste 濃度で表す | 確定 | REQ-SIM-02 |
| `ReasonCode` | Intake / Maintenance / Starvation / Death / Reproduction / Emission / Diffusion の 7 種 | 確定 | REQ-SCOPE-04 |
| `TickPhase` | Diffuse → Intake → Maintenance → StarvationAndDeath → Reproduction → Emission → Occupancy の 7 種固定順 | 確定 | REQ-SIM-04 |
| `TraitVector` | movement / intake / conversion / maintenance_cost / reproduction の 5 軸（Fixed 倍率） | 確定 | REQ-GEN-01 |
| `MechanismTags` | use_nutrient / use_carcass / use_waste / toxin_sensitive / density_bonus の 5 ビット | 確定 | REQ-GEN-01 |
| `LineageParams` | `id: u8` + TraitVector + MechanismTags + `mortality_threshold` + `waste_emission` のみ。それ以外の系統固有パラメータを持たない | 確定 | REQ-GEN-01 |
| `MassCoefficients` | intake→biomass 0.70 / intake→waste 0.30 / starvation→carcass 1.00 / death→carcass 1.00（契約 §4） | 初期仮説（D3 で確定） | REQ-SIM-05 |
| `ConversionRule` | `from / to / coefficient / remainder_to`。余りは常に主出力プールへ、拡散余りは送り元セルへ | 確定（規則） | REQ-SIM-05 |
| `InflowEvent` | `{ tick: u32, pool: Pool, amount: Fixed }`。閉鎖系は空 Vec | 確定 | REQ-SIM-07 |
| `Thresholds.epsilon` | 1e-4 × 初期総生体量 | 確定 | REQ-END-02 |
| `Thresholds.fixed_share / fixed_ticks` | 総生体量の 70% 以上を 200 tick 継続 | 確定 | REQ-END-03 |
| `Thresholds.coexist_share` | 各 15% 以上 | 確定 | REQ-END-04a |
| `Thresholds.max_ticks` | 2,000 tick（= 100 世代） | 確定 | REQ-SCOPE-06 |
| `Thresholds.waste_toxic_threshold`（θ_w） | 値は未定。超過時に toxin_sensitive 系統の維持コストへ倍率適用 | 初期仮説（D3 で確定） | REQ-SIM-02 |
| `Thresholds.toxin_maintenance_multiplier` | ×1.4 | 初期仮説（契約 §8 で確定済み、D3 の実測で更新しうる） | REQ-SIM-02 |
| `Thresholds.occupancy_threshold`（θ_occ） | 値は未定 | 初期仮説（D2 で確定） | REQ-SIM-03a |
| `Thresholds.vacant_nutrient_threshold`（θ） | セル栄養の初期中央値の 10% | 確定 | REQ-SIM-03b |
| occupancy 減衰率 | 毎 tick ×0.995、飽和値 1.0、空き家判定線 0.3 | 確定 | REQ-SIM-03a, REQ-SIM-03b |
| 拡散係数 | 各プール 0.05 / 近傍 / tick | 初期仮説（D2 で確定） | REQ-SIM-10 |
| 摂取 1 tick 上限 | intake 倍率 × 基準摂取量（`base_intake`） | 初期仮説（D3 で確定） | REQ-SIM-11 |
| 繁殖条件 | energy > 維持コスト × 2 のとき余剰の一定割合を生体量へ | 初期仮説（D3 で確定） | REQ-SIM-12 |
| energy→質量係数 | 1.0（D1） | 初期仮説（D3 で確定） | REQ-SIM-12 |
| 遺伝的ばらつき上限 | ±0.05（5 軸各値） | 初期仮説（D4 で確定） | REQ-GEN-08 |
| 格子 | 検証版 64×64（確定）、契約上限 65535²（確定） | 確定 | REQ-SIM-01 |
| `TerminationLabel` | Extinct / Fixed / Coexist / Reversal / TimeLimit の 5 種のみ | 確定 | REQ-END-01 |
| `ModelVersion` | `{ major, minor, scale, rounding, prng = "xoshiro256ss-v1", hash = "sha256-v1" }` | 確定 | REQ-DET-05, REQ-NFR-06 |
| `StateHash` | SHA-256 の 32 バイト | 確定 | REQ-DET-05 |
| `NumericError` | Negative / OverflowI64 / OverflowI128 の 3 種 | 確定 | REQ-SIM-13 |

## 3. 不変条件

違反時の挙動は 3 型に分類する（分類の定義は §4）。各不変条件に property test 雛形を付す。雛形は proptest 相当の「生成 → 操作 → assert」で記述し、詳細設計（DD）で具体化する。

### INV-01 質量保存（閉鎖系）

- 式: inflow = ∅ のとき、全 tick t で `Σ_cells (nutrient + Σ_L biomass[L] + carcass + waste)(t) = M₀`（Fixed 厳密一致）
- 違反時: 検査型。`InvariantReport.mass_ok = false` とし、`verify --suite` が非 0 終了する
- 参照: REQ-SIM-06, REQ-OPS-01

```rust
// 生成: 任意の 1 セル閉鎖 config（nutrient/carcass/waste ∈ 0..=10^12、
//       biomass ∈ 0..=10^12、tags 任意、mortality_threshold ∈ 1..=10^6、inflow = []）
// 操作: SimCore::try_one_cell → tick_once × 2,000
// assert: 各 tick 後に nutrient + Σbiomass + carcass + waste == 初期総量（Fixed 厳密）
```

### INV-02 質量保存（流入系）

- 式: 全 tick t で `総質量(t) = M₀ + Σ_{e ∈ inflow, e.tick ≤ t} e.amount`（Fixed 厳密一致）
- 違反時: 検査型。INV-01 に同じ
- 参照: REQ-SIM-07

```rust
// 生成: 任意の 1 セル config + 任意の inflow: Vec<InflowEvent>（tick ∈ 0..2,000、
//       pool 任意、amount ∈ 0..=10^9、tick 昇順）
// 操作: tick_once × 2,000（inflow を該当 tick で適用）
// assert: 各 tick 後に 総質量 == 初期総量 + Σ 適用済み inflow.amount（Fixed 厳密）
```

### INV-03 変換保存・余り不消失

- 式: 全変換で `Σ 出力 + 余り = 入力`。余りは主出力プールへ戻し、拡散の余りは送り元セルに残す。捨てる経路は存在しない
- 違反時: 検査型。Ledger の全 LedgerEntry が保存則を満たすことを検査する
- 参照: REQ-SIM-05

```rust
// 生成: 任意の input: Fixed ∈ 0..=10^15、任意の ConversionRule
//       （coefficient ∈ 0..=scale、remainder_to ∈ {主出力, 副出力}）、waste_coefficient ∈ 0..=scale − coefficient
// 操作: split_output_with_rule(input, rule, waste_coefficient)
// assert: primary + secondary + remainder == input かつ remainder が rule.remainder_to のプールに加算されている
// assert: 拡散では 送出量合計 + 残余 == 送り元の減少量（余りは送り元に残る）
```

### INV-04 非負プール

- 式: 全 tick・全セル・全プール・全系統で `値 ≥ 0`
- 違反時: 停止型。負値が生じる変換は `NumericError::Negative` で即停止し、状態は違反前のままとする
- 参照: REQ-SIM-13

```rust
// 生成: 任意の WorldState（各プール ∈ 0..=10^12、L ∈ 1..=4、格子 1×1..=8×8）
// 操作: tick_once × 100
// assert: 各 tick 後に全セル全プール ≥ 0（Err(Negative) なく完了するか、
//         Err なら直前状態が保持されている）
```

### INV-05 energy 値域

- 式: 全セル・全系統で `0 ≤ energy[L] ≤ 1`（無次元予算）
- 違反時: 停止型。範囲外は `NumericError` で即停止する
- 参照: REQ-SIM-08

```rust
// 生成: 任意の CellState（energy ∈ 0..=scale、biomass ∈ 0..=10^12）と任意の TraitVector
// 操作: intake / maintenance / reproduction 各 phase を単独で 1 回ずつ適用
// assert: 適用後も 0 ≤ energy ≤ scale（= 1.0）
// assert: energy < 維持コストのとき移動・繁殖は 0、不足分（cost − energy）だけ biomass → carcass
```

### INV-06 数値範囲と停止

- 式: 全状態は i64 範囲内、全乗算中間値は i128 範囲内。範囲外になりうる演算は実行前に検査する
- 違反時: 停止型。`NumericError::OverflowI64 / OverflowI128` で即停止し、状態を壊さない。上限設計の証明表は BD-06 に置く
- 参照: REQ-SIM-13, REQ-SIM-14

```rust
// 生成: 境界値 config（各プール = 上限、上限 + 1、上限 × 係数で i128 を超える入力）
// 操作: 該当変換を 1 回適用
// assert: 上限ちょうど → Ok、上限 + 1 → Err(NumericError) かつ状態不変
```

### INV-07 系統数不変

- 式: 全 tick t で `lineages.len(t) = lineages.len(0)`。分岐・新系統発生は存在しない
- 違反時: 検査型。AT（全検証 seed で系統数不変）で検出する
- 参照: REQ-OUT-01

```rust
// 生成: 任意の config（L ∈ 1..=4）と任意の seed
// 操作: step(2,000)
// assert: 終了時の lineages.len() == 初期値（途中の WorldState でも不変）
```

### INV-08 系統 ID 一意・昇順処理

- 式: `lineages` の id は一意で、intake 等の逐次処理は id 昇順に行う（按分しない）
- 違反時: 拒否型。`try_one_cell` / run 生成時に重複 id はエラーとし、run を作らない
- 参照: REQ-SIM-11, REQ-DET-04c

```rust
// 生成: 任意の lineages（id に重複を含む / 含まない、順序シャッフル）
// 操作: run 生成 → step(100)
// assert: 重複 id → Err。受理時は処理順が id 昇順（LedgerEntry の順序で検証）
// assert: 同じ集合を別順で渡した 2 run の state hash が一致
```

### INV-09 occupancy_peak 値域

- 式: 全セルで `0 ≤ occupancy_peak ≤ 1`。更新は `biomass_sum ≥ θ_occ → 1.0`、それ以外 → `× 0.995`
- 違反時: 停止型。範囲外は `NumericError` で即停止する
- 参照: REQ-SIM-03a

```rust
// 生成: 任意の occupancy_peak ∈ 0..=scale、biomass_sum ∈ 0..=10^12、θ_occ ∈ 1..=10^12
// 操作: occupancy phase を 1 回適用
// assert: 0 ≤ occupancy_peak' ≤ scale かつ（biomass_sum ≥ θ_occ → occupancy_peak' == scale）
//         かつ（biomass_sum < θ_occ → occupancy_peak' == occupancy_peak × 0.995 のゼロ方向丸め）
```

### INV-10 tick 単調増加・wall clock 非依存

- 式: `step(n)` 1 回の呼出で tick はちょうど n 増える。コアは wall clock を読まず、再生速度・一時停止は PRNG・hash に影響しない
- 違反時: 検査型。3 速度での hash 一致 AT で検出する
- 参照: REQ-CON-05, REQ-DET-07

```rust
// 生成: 任意の config・seed、任意の分割 n₁ + n₂ = 2,000
// 操作: step(n₁) → step(n₂) と step(2,000) を並行実行
// assert: 両者の tick == 2,000 かつ state hash が一致
```

### INV-11 台帳完全性

- 式: 全変換が `LedgerEntry { tick, cell_index, lineage, from_pool, to_pool, amount, reason }` を 1 件以上生成し、amount > 0、reason ∈ ReasonCode。未記録の残差・負値エントリは存在しない
- 違反時: 検査型。Ledger 走査で `from 側減量合計 = to 側増量合計 + 余り戻し` を検証する
- 参照: REQ-SIM-05, REQ-SCOPE-04

```rust
// 生成: 任意の代表 seed config（4 系統・1 環境）
// 操作: step(2,000)
// assert: ReasonCode 全 7 種が Ledger に 1 回以上出現する
// assert: 全 LedgerEntry で amount > 0 かつ from ≠ to（同一プール循環エントリなし）
```

### INV-12 hash 正規化の非描画依存

- 式: 正規化バイト列 = tick・seed・寸法・全セル（row-major・系統昇順・i64 バイト列）・PRNG 4 ストリーム状態・model_version。描画用トークン・UI・ログ時刻・転換点検出の有無を含まない
- 違反時: 検査型。トークン on/off・検出 on/off で hash 一致を AT で検査する
- 参照: REQ-DET-05, REQ-OUT-04, REQ-EVT-05

```rust
// 生成: 任意の config・seed、任意の表示用トークン列（0..=8 個）
// 操作: トークン有り / 無しの 2 run を step(2,000)
// assert: 両者の state hash・終了ラベルがビット一致
```

### INV-13 SaveEnvelope 整合

- 式: `load(save(run))` は checksum・schema_version・model_version が全て一致するときのみ成功し、成功時は `state` と `prng_state` を復元する
- 違反時: 拒否型。不一致はエラーを返し、run を作らない
- 参照: REQ-DET-06, REQ-CON-08

```rust
// 生成: 任意の run の save → バイト列に任意の 1 バイト改変 / schema_version・model_version を ±1
// 操作: load
// assert: 改変あり → Err（種別: checksum / schema_version / model_version）
// assert: 無改変 → Ok かつ load 後の state hash == save 時の state_hash
```

### INV-14 三経路一致

- 式: 同一 model_version・config・seed で `step(2000)` ≡ `step(1) × 2000` ≡ 任意 tick で save → load → 残り、の 3 経路が同一の state hash・終了ラベルを返す
- 違反時: 検査型。AT（代表 tick + ランダム tick 数点）で検出する
- 参照: REQ-DET-02

```rust
// 生成: 任意の config・seed、任意の分割点 t ∈ 1..2,000
// 操作: 経路 A = step(2,000)、経路 B = step(t) → save → load → step(2,000 − t)
// assert: A.state_hash == B.state_hash かつ A.終了ラベル == B.終了ラベル
```

## 4. 違反時の挙動の分類

| 分類 | 意味 | 該当不変条件 | 参照 |
|---|---|---|---|
| 停止型 | `NumericError` で現在 tick を中断し、状態は違反前のまま保持する | INV-04, INV-05, INV-06, INV-09 | REQ-SIM-13 |
| 検査型 | コアは停止せず、`InvariantReport` / `verify --suite` / AT が検出して非 0 終了する（違反 = 実装バグ） | INV-01, INV-02, INV-03, INV-07, INV-10, INV-11, INV-12, INV-14 | REQ-OPS-01 |
| 拒否型 | 入力検証でエラーを返し、run・状態を作らない | INV-08, INV-13 | REQ-DET-06, REQ-CON-08 |

## 5. ADR 候補（REQ に無い設計判断）

本章の記述のうち REQ に直接の根拠が無い判断は以下のとおり。ADR 化は BD-05 以降で行う（本章では編集対象外のため列挙に留める）。

- ADR 候補 1: 拒否型のエラー型は `NumericError` とは別の `ValidationError` 系とする（重複 id・version 不一致を数値異常と区別する）。参照: REQ-SIM-13, REQ-DET-06
- ADR 候補 2: Ledger は run 中に全件保持し、保存には含めない（保存 ≤ 5 MB の REQ-EVT-04 制約と両立させる）。参照: REQ-EVT-04, REQ-NFR-02
