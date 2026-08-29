# [D0-contract-001][review] r2 reviewer=claude — **approve**

- 対象: commit 0b92499
- 統合判定 A〜D の確認結果

| 項目 | 状態 | 根拠 |
|---|---|---|
| A1 toxin ×1.4 / θ_w | ✓ | §8 に `toxin_maintenance_multiplier`、`theta_w` は D3 確定の初期仮説と明記。`Thresholds` にフィールド追加 |
| A2 occupancy 更新則 | ✓ | 第 7 phase `occupancy`、飽和／0.995 減衰／空き家条件、`theta_occ` は D2 確定 |
| A3 PRNG | ✓ | SplitMix64 → xoshiro256** 4 ストリーム、ModelVersion に prng 名 |
| A4 hash | ✓ | SHA-256、ModelVersion に hash 名 |
| B1〜B3 終了判定 | ✓ | Extinct/Fixed 毎 tick、Coexist/Reversal は上限時、同率は ID 昇順、`TerminationRule { timing }` |
| B4 余り戻し先 | ✓ | §4 表に列追加、拡散余りは送り元 |
| B5 inflow | ✓ | `InflowEvent { tick, pool, amount }` |
| B6 death | ✓ | 第 4 phase `starvation_and_death`、`biomass < mortality_threshold` で全量 carcass |
| C1〜C4 | ✓ | 拡散係数 D2、摂取則・繁殖余剰 D3、movement D2 と明記 |
| D1〜D2 型 | ✓ | ConversionRule / TerminationRule / StateSnapshot / InflowEvent / RoundingMode enum / NumericError enum / ModelVersion struct |
| D3 schema | ✓ | lineage required 5、trait 5 キー、tags additionalProperties=false、seed u64、grid ≤ 65535 |
| D4 save schema | ✓ | `save.schema.json` に prng_state を含む required 7 |
| D5 tick | 要確認 | §1 に `tick: u32` の明記は kimi r2 で確認 |

合否: kimi r2 が approve なら D0 完了。次段階 D1 は cargo 導入後に brief を発行。
