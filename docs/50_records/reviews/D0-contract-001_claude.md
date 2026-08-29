# [D0-contract-001][review] reviewer=claude（企画整合・合否）

- 対象: commit 44e1bb4（simulation_contract.md v0.1, sim-types/src/lib.rs, schema/*.json）
- 判定: **changes_requested**（軽微。構造は正しい。v0.5 7.1 の定数と、状態更新の未定義が残る）

## findings

1. **toxin_sensitive の倍率と閾値が未記載**（契約 2-3, v0.5 7.1）
   節 2 の maintenance に「毒性閾値超過時は倍率を適用」とあるが、値が無い。v0.5 7.1 の初期仮説 `waste > θ_w → 維持コスト ×1.4` を節 4 か節 7 に定数として置き、`Thresholds` に `waste_toxic_threshold` と `toxin_maintenance_multiplier` を追加すること。
2. **occupancy_peak の更新規則が無い**（契約 1, v0.5 1.1）
   状態には入っているが、飽和条件（生体量合計が閾値超で 1 に飽和）・減衰 0.995/tick・空き家条件 `occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ`（θ＝セル栄養の初期中央値の 10%）が契約に無い。どの tick phase で更新するかも明記（emission の後を推奨）。
3. **inflow_tick_mask に対応する型が無い**（契約 3・9）
   節 3・9 で参照しているが sim-types に無い。`InflowMask` 相当を追加し、閉鎖系スイートでは空であることを型で表す。
4. **死亡（mortality_threshold）の phase が無い**（契約 2・4・7）
   節 4 に「biomass 死亡→carcass 1.00」、節 7 に `mortality_threshold`、ReasonCode に `Death` があるが、TickPhase に死亡判定が無い。starvation と別に `death`（energy または生体量が閾値未満のセル系統を carcass へ）を置くか、starvation に統合するかを決めて明記。
5. **RoundingMode / NumericError / ScanOrder / RandomStream / ModelVersion / VerifySuite が空の unit struct**
   D0 の完了判定「未定義の数値型が無い」に対し、少なくとも `RoundingMode` は enum（`TowardZero` のみで可）、`NumericError` は enum（`Negative / OverflowI64 / OverflowI128`）、`ModelVersion` は `{ major, minor, scale, rounding }` を持つ struct にすること。他は D1 で埋めてよい旨を TODO に書く。
6. **movement 軸の意味が未定義**（契約 2・7、v0.5 1.1 も同様）
   TraitVector に `movement` があるが、生体量を動かす phase が無い（diffuse は栄養・死骸・老廃物のみ）。これは v0.5 側の穴でもあるので **D0 のブロッカーにしない**。契約に「movement は D2 で定義（生体量の近傍拡散率として扱う案）」と open issue を明記すること。

## 企画整合（問題なし）
- 4 プール・閉鎖系・tick 順序・二重台帳・i64/i128・用途別 PRNG・5 ビットタグ＋系統定数 2・終了ラベル 5 種と優先順・三経路一致は v0.4/v0.5 と一致
- 11 章「やらないこと」に抵触する項目なし。生成 AI・分岐・コドンは入っていない

## 合否
1〜5 を反映した commit で **approve** 予定。6 は open issue として記録すれば可。kimi の試験可能性レビューが別途届くので、同じ修正 commit にまとめてよい。
