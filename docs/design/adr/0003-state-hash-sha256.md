# ADR-0003 state hash は SHA-256、正規化バイト列に PRNG 状態と model_version を含む

- 状態: 採用（2026-08-30、D1 r2 で PRNG 状態・model_version を追加。BD-01 r2 で継続カウンタ等を追加 → `hash=sha256-v2`）
- 参照: REQ-DET-01/02/03/05

## 文脈
三経路一致・クロス OS 一致・golden の基準となる 1 値が必要。D1 初版は PRNG 状態を含めておらず、D2 で乱数消費が始まると save/load 経路の一致が検出できなくなる欠陥があった（kimi r1 #2）。

## 選択肢
1. FNV/xxhash → 高速だが衝突耐性が弱く、golden の信頼性が落ちる
2. **SHA-256（sha2 crate）** → 採用。速度は 2000 tick に 1 回なら無視できる

## 結果
- 正規化順（v2）: tick, seed, width, height, model_version(UTF-8), PRNG 4 ストリーム state（LE u64×4×4）, **Fixed 継続カウンタ（u32）, tick 0 順位（u8×L）, inflow 消化位置（u32）**, セル row-major で nutrient, biomass[0..8], carcass, waste, energy[0..8], occupancy_peak（各 i64 LE）。三経路 AT は hash に加えて終了ラベルと台帳ダイジェストも比較する（grok F-14）
- 順序変更は model_version の hash 部（`hash=sha256-v1`）を bump
