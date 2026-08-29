# キミ図鑑 検証版シミュレーション契約 v0.1

この契約は、検証版D0の状態・数値・更新順・再現性を定める。物質の単位は `mass_u`（整数、1 u は固定小数点の最小単位）で表し、エネルギーは無次元の系統別予算である。

## 1. 状態

1セルは `nutrient: Fixed`、`biomass[L]: Fixed`、`carcass: Fixed`、`waste: Fixed`、`energy[L]: Fixed`、`occupancy_peak: Fixed` を持つ。環境は `GridState`、系統設定は `LineageParams`、全体は `WorldState` が所有する。`L` は最大8系統。

対応するRust型名: `CellState`, `GridState`, `WorldState`, `LineageParams`, `Fixed`

## 2. tick順序

各tickは次の順序を一度だけ実行する。走査順はセルrow-major、系統ID昇順、近傍は北・東・南・西の固定順とする。

1. `diffuse`: 栄養、死骸、老廃物を4近傍へ移す
2. `intake`: 系統が利用可能なプールを取り、生体量とenergyへ変換する
3. `maintenance`: 維持コストをenergyから課金する。毒性閾値超過時は倍率を適用する
4. `starvation`: energy不足分を生体量から死骸へ移す
5. `reproduction`: energy余剰と繁殖係数に基づき生体量を増やす
6. `emission`: 代謝残差を老廃物へ置く。全変換残差を捨てない

対応するRust型名: `SimCore::step`, `TickPhase`

## 3. 物質・エネルギー二重台帳

物質台帳は `nutrient + biomass_sum + carcass + waste` を追跡する。閉鎖系では外部流入・流出を0とし、プリセット流入は `inflow_tick_mask` の期待値として明示する。エネルギー台帳は系統ごとに、摂取加算、維持・移動・繁殖への配分、熱散逸を追跡する。熱散逸は物質を減らさない。

全変換は `LedgerEntry { from_pool, to_pool, amount, reason }` を通し、負値と未記録の残差を禁止する。

対応するRust型名: `MassLedger`, `EnergyLedger`, `LedgerEntry`, `Pool`, `ReasonCode`

## 4. 質量係数表（検証版の初期仮説）

係数は入力質量に対する出力質量の割合で、合計1.0（固定小数点では1_000_000）とする。余りは同一変換の指定先へ戻し、捨てない。

| 変換 | 出力 | 係数 |
|---|---|---:|
| 摂取 | biomass | 0.70 |
| 摂取 | waste | 0.30 |
| biomass維持不足 | carcass | 1.00 |
| biomass死亡 | carcass | 1.00 |

係数はD3までに実測で更新し、configのhashへ含める。エネルギー係数は物質係数と別の無次元値である。

対応するRust型名: `MassCoefficients`, `ConversionRule`

## 5. 固定小数点・丸め規則

コア状態と係数はi64固定小数点、scale=1_000_000（10進6桁）とする。乗算の中間値はi128へ拡張する。除算はゼロ方向へ丸め、余りは固定された近傍順またはセル内残余へ戻す。負値、i64範囲外、中間i128範囲外はエラーとする。

解析用z-scoreだけは浮動小数点を許すが、終了ラベルやコア状態へ戻さない。丸めモードとscaleはmodel_versionの一部とする。

対応するRust型名: `Fixed`, `RoundingMode`, `NumericError`

## 6. PRNG・用途別ストリーム・走査順

PRNGアルゴリズムとバージョンを固定し、seedから用途別ストリーム（`movement`, `reproduction`, `mutation`, `interaction`）を導出する。表示用サンプリングはコア乱数を消費しない。seed内の計算は単一スレッド、seed間のバッチだけ並列化する。HashMapの反復順へ依存しない。

対応するRust型名: `Seed`, `PrngState`, `RandomStream`, `ScanOrder`

## 7. 機構タグ・系統定数・適応ベクトル

機構タグは5ビット `use_nutrient / use_carcass / use_waste / toxin_sensitive / density_bonus` とする。札の5軸は `movement / intake / conversion / maintenance_cost / reproduction` の固定倍率で、維持コストは小さいほど有利。系統定数は `mortality_threshold` と `waste_emission` の2つだけを許す。タグと定数はプリセット固定で、プレイヤーは札のベクトルだけ選ぶ。

対応するRust型名: `MechanismTags`, `TraitVector`, `LineageParams`, `Substrate`

## 8. 終了ラベルと閾値

検証版の初期仮説は次のとおり。優先順は全滅、固定、共存、逆転、上限とする。

- `Extinct`: 全系統の生体量が `epsilon` 未満
- `Fixed`: 1系統が総生体量の70%以上を200 tick継続
- `Coexist`: 2系統以上が各15%以上
- `Reversal`: 終了時1位がtick 0の順位で3位以下
- `TimeLimit`: 2,000 tick到達

`epsilon = 1e-4 × 初期総生体量`。同時成立時は優先順を適用し、判定理由を保存する。

対応するRust型名: `TerminationLabel`, `TerminationRule`, `Thresholds`

## 9. 保存則テスト定義

閉鎖系では各tickおよび最終状態で、物質4プールの総量が初期総量と一致することを要求する。流入プリセットでは、初期総量＋期待流入量と一致することを要求する。全プールは非負であること。energyは系統別予算の負値を禁止し、熱散逸を含むエネルギー予算の差分を台帳化する。

必須テスト: 1セル2,000 tick、64×64拡散2,000 tick、一様場不変、境界流出なし、係数合計、残余非消失、starvationのcarcass移動。

対応するRust型名: `InvariantReport`, `MassLedger`, `EnergyLedger`, `VerifySuite`

## 10. 三経路一致・state hash

同じmodel_version、config、seedについて、`step(2000)`、`step(1)`を2,000回、tick 1,000でsave/loadして再開、の三経路は同じ正規化state hashと終了ラベルを返す。正規化はセルrow-major、系統昇順、固定小数点のi64バイト列、PRNG状態、model_versionを含み、描画用トークン・UI・ログ時刻を除く。

速度変更、描画間引き、トークンの有無、seed間の並列数はhashを変えてはならない。保存には`schema_version / model_version / config_hash / seed / prng_state / state_hash`を含める。

対応するRust型名: `StateSnapshot`, `StateHash`, `SaveEnvelope`, `ModelVersion`

## Rust型と契約節の対応

| 契約節 | Rust型 |
|---|---|
| 1 状態 | `CellState`, `GridState`, `WorldState`, `LineageParams`, `Fixed` |
| 2 tick順序 | `SimCore::step`, `TickPhase` |
| 3 二重台帳 | `MassLedger`, `EnergyLedger`, `LedgerEntry`, `Pool`, `ReasonCode` |
| 4 質量係数 | `MassCoefficients`, `ConversionRule` |
| 5 固定小数点 | `Fixed`, `RoundingMode`, `NumericError` |
| 6 PRNG | `Seed`, `PrngState`, `RandomStream`, `ScanOrder` |
| 7 機構タグ | `MechanismTags`, `TraitVector`, `LineageParams`, `Substrate` |
| 8 終了 | `TerminationLabel`, `TerminationRule`, `Thresholds` |
| 9 保存則 | `InvariantReport`, `VerifySuite` |
| 10 再現 | `StateSnapshot`, `StateHash`, `SaveEnvelope`, `ModelVersion` |
