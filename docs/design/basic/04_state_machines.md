# BD-04 状態機械

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/要件定義書_検証版_v0.2.md`（sign-off 済）、`docs/contracts/simulation_contract.md` v0.1（契約 §n で参照）
- 完成条件: 全状態 × 全イベントが表に埋まり、生成テストが書ける
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。集約・不変条件は BD-03、公開 API・FFI は BD-05 を参照
- 表の記法: `→ 次状態 / アクション`。`—` = その組合せは到達不能（理由を付記）。`Err` = 拒否（エラーを返し状態不変）

## 1. Run 状態機械

### 1.1 状態とイベントの定義

- 状態（参照: REQ-CON-01, REQ-DET-02, REQ-END-01）
  - `Prepared`: create 直後。tick = 0、終了判定未実施
  - `Running`: 1 tick 以上進行し、終了条件未成立
  - `Terminated(label)`: 終了条件成立または上限到達。label ∈ {Extinct, Fixed, Coexist, Reversal, TimeLimit}
  - `Destroyed`: destroy 済み。メモリ解放後
- イベント（FFI 7 操作 + 内部イベント。参照: REQ-CON-01）
  - `create` / `load` / `step(n)` / `snapshot` / `explain` / `save` / `destroy`
  - `termination_detected`: step 内の終了判定が発火する内部イベント。優先順 Extinct > Fixed > Coexist > Reversal > TimeLimit（確定。参照: REQ-END-04c）
- コアは wall clock を読まず、一時停止・再開は UI 側スケジューラの責務であるため Run の状態に Pause を持たない（確定。参照: REQ-CON-05, REQ-UI-03）

### 1.2 Run 状態 × イベント表

| 状態 ＼ イベント | create | step(n) | save | load | snapshot | explain | destroy | termination_detected |
|---|---|---|---|---|---|---|---|---|
| Prepared | —（create は無状態から Prepared を生成する初期イベント） | → Running / 7 phase を n 回適用し tick += n。step 内で終了条件成立時は Terminated へ直接遷移（T1） | → Prepared / SaveEnvelope を生成（状態不変） | —（load は常に新規 Run を生成し、既存 Run には適用しない。ADR 候補 1） | → Prepared / 固定レイアウトバッファへコピーアウト（状態不変。REQ-VIS-04） | → Prepared / 4 段説明を生成（状態不変。REQ-EXP-01） | → Destroyed / メモリ解放 | → Terminated(label) / tick 0 でも全系統生体量 < ε なら Extinct が成立しうる（REQ-END-02） |
| Running | —（二重 create は行わない） | → Running / 同上。終了条件成立時は Terminated へ（T1） | → Running / SaveEnvelope を生成（状態不変。REQ-DET-02 の三経路の中継点） | —（同上） | → Running / 同上 | → Running / 同上 | → Destroyed / メモリ解放 | → Terminated(label) / ラベル確定・判定理由を保存（REQ-END-04c） |
| Terminated(label) | — | Err / 終了済み Run への step は拒否（ADR 候補 2） | → Terminated / 結果の SaveEnvelope を生成（状態不変） | —（同上） | → Terminated / 終了時描画をコピーアウト | → Terminated / 絶滅時を含め原因候補と再実験入口を返す（REQ-EXP-05） | → Destroyed / メモリ解放 | —（終了後に再判定しない。冪等） |
| Destroyed | — | Err / use-after-destroy 禁止 | Err / 同上 | — | Err / 同上 | Err / 同上 | —（二重 destroy は行わない） | —（到達不能） |

- T1（step 内の終了判定ガード。確定）: 各 tick の 7 phase 完了後に、Extinct（毎 tick）→ Fixed（毎 tick）を判定し即終了。上限 tick 到達時は Coexist → Reversal を判定し、いずれも不成立なら TimeLimit。同時成立時は優先順を適用する。参照: REQ-END-02, REQ-END-03, REQ-END-04a, REQ-END-04b, REQ-END-04c
- step のアクション内訳: 7 phase を固定順（diffuse → intake → maintenance → starvation_and_death → reproduction → emission → occupancy）で 1 回ずつ実行する。順序の入替は契約違反。参照: REQ-SIM-04
- save → load で復元された Run は、保存時の tick・状態・PRNG 状態を持ち、経路に応じて Prepared（tick = 0）または Running（tick ≥ 1）として生成される。参照: REQ-DET-02, REQ-DET-06

### 1.3 Run 生成テスト雛形

```rust
// 生成: 任意の config・seed、任意のイベント列（FFI 7 操作のランダム列、長さ 0..=64）
// 操作: イベント列を先頭から適用
// assert: 全プレフィックスで状態が 4 状態のいずれかに収まり、
//         表中の — / Err セルに該当する遷移は必ず Err で状態不変
// assert: Terminated 到達後の step は Err、save / snapshot / explain は Ok で状態不変
// 参照: REQ-CON-01, REQ-DET-02
```

## 2. Cell 占有状態機械

### 2.1 状態とイベントの定義

- 状態（セルごと。occupancy phase で更新。参照: REQ-SIM-03a, REQ-SIM-04）
  - `Occupied`: 直前の occupancy 判定で `biomass_sum ≥ θ_occ`。`occupancy_peak = 1.0` に飽和
  - `Fading`: 直前の occupancy 判定で `biomass_sum < θ_occ`。`occupancy_peak × 0.995` で減衰
- 導出判定（状態を変えない。参照: REQ-SIM-03b）
  - `Vacant`（空き家）: `occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ`。表示・説明器専用の判定で、セル状態への書き戻しは行わない
- イベント
  - `occupancy_tick`: occupancy phase の定期更新（毎 tick 1 回）
  - `biomass_changed`: intake / reproduction / starvation_and_death / death による biomass_sum の変化（次回 occupancy_tick のガード条件を変えるだけで、即座に状態遷移は起こさない）
- 定数: 飽和値 1.0・減衰率 ×0.995・空き家判定線 0.3（確定）、θ_occ（初期仮説、D2 で確定）、θ = セル栄養の初期中央値の 10%（確定）、ε = 1e-4 × 初期総生体量（確定）。参照: REQ-SIM-03a, REQ-SIM-03b, REQ-END-02

### 2.2 Cell 占有 状態 × イベント表

| 状態 ＼ イベント | occupancy_tick（biomass_sum ≥ θ_occ） | occupancy_tick（biomass_sum < θ_occ） | biomass_changed |
|---|---|---|---|
| Occupied | → Occupied / occupancy_peak = 1.0（再飽和） | → Fading / occupancy_peak ×= 0.995 | → 状態不変 / 次回 occupancy_tick のガードに反映 |
| Fading | → Occupied / occupancy_peak = 1.0（再占有） | → Fading / occupancy_peak ×= 0.995（継続減衰。下限 0 で飽和） | → 状態不変 / 同上 |

- Vacant 判定表（導出。状態 × 追加ガード。参照: REQ-SIM-03b）

| 状態 | occupancy_peak > 0.3 | biomass_sum < ε | nutrient > θ | Vacant 判定 |
|---|---|---|---|---|
| Occupied | 常に真（1.0） | 偽（≥ θ_occ > ε） | 任意 | 非 Vacant |
| Fading | 真 | 真 | 真 | **Vacant**（表示・説明器へ通知。状態不変） |
| Fading | 真 | 真 | 偽 | 非 Vacant（栄養が回復していない） |
| Fading | 真 | 偽 | 任意 | 非 Vacant（再占有途中） |
| Fading | 偽 | 任意 | 任意 | 非 Vacant（占有の痕跡が消えた） |

### 2.3 Cell 占有 生成テスト雛形

```rust
// 生成: 任意の biomass_sum 系列（長さ 2,000、各値 ∈ 0..=10^12）、θ_occ ∈ 1..=10^12、
//       初期 occupancy_peak ∈ 0..=scale、nutrient 系列
// 操作: occupancy_tick を系列に沿って 2,000 回適用
// assert: 全 tick で 0 ≤ occupancy_peak ≤ scale（INV-09）
// assert: biomass_sum ≥ θ_occ の tick では occupancy_peak == scale
// assert: Vacant 判定の前後でセル状態がビット一致（判定は状態を変えない）
// 参照: REQ-SIM-03a, REQ-SIM-03b
```

## 3. Lineage-in-cell 状態機械

### 3.1 状態とイベントの定義

- 状態（セル × 系統ごと。参照: REQ-SIM-08, REQ-SIM-09）
  - `Absent`: biomass[L] = 0（未侵入または死滅後）
  - `Alive`: biomass[L] ≥ mortality_threshold かつ energy[L] ≥ 維持コスト
  - `Starving`: biomass[L] ≥ mortality_threshold かつ energy[L] < 維持コスト（maintenance で不足が確定）
  - `Dying`: biomass[L] < mortality_threshold（starvation_and_death で全量 carcass 化される遷移状態。tick を跨がない）
- イベント（7 phase のうち系統に作用するもの。参照: REQ-SIM-04）
  - `intake` / `maintenance` / `starvation_and_death` / `reproduction` / `emission`
  - diffuse は biomass に作用しない（movement 軸による生体量の近傍拡散は初期仮説、D2 で確定。確定後にイベントを追加する）。参照: REQ-SIM-10
- 定数: 維持コスト = base_maintenance × maintenance_cost 倍率（waste > θ_w かつ toxin_sensitive なら ×1.4。1.4 は初期仮説、D3 で実測更新しうる）、繁殖条件 energy > 維持コスト × 2（初期仮説、D3 で確定）、energy→質量係数 1.0（初期仮説、D3 で確定）。参照: REQ-SIM-02, REQ-SIM-12

### 3.2 Lineage-in-cell 状態 × イベント表

| 状態 ＼ イベント | intake | maintenance | starvation_and_death | reproduction | emission |
|---|---|---|---|---|---|
| Absent | → Absent / 摂取しない（biomass = 0 では取込み主体が存在しない） | → Absent / コスト 0 | → Absent / 何もしない | → Absent / energy 余剰なし | → Absent / 排出なし |
| Alive | → Alive / 利用可能プールから取込み、係数で biomass・energy へ配分（余りは biomass へ。REQ-SIM-05） | ガード energy ≥ cost: → Alive / energy −= cost。ガード energy < cost: → Starving / energy = 0、不足分 cost − energy を記録 | ガード biomass ≥ mortality_threshold: → Alive / 変化なし。ガード biomass < mortality_threshold: → Dying / 全量を carcass へ（ReasonCode::Death。REQ-SIM-09） | ガード energy > cost × 2: → Alive / 余剰の一定割合を biomass・energy 双方から質量へ（係数 1.0）。それ以外: → Alive / 変化なし | → Alive / 代謝残差を waste へ（残差を捨てない。REQ-SIM-05）。waste > θ_w かつ toxin_sensitive なら次 tick の maintenance ガードに ×1.4 を反映（REQ-SIM-02） |
| Starving | → Starving / intake は可能（取込みで energy が回復しても当 tick の状態は維持し、次 maintenance で再判定） | → Starving / energy = 0 のまま、不足分を再記録 | → Alive / 不足分（cost − energy）だけ biomass → carcass（ReasonCode::Starvation。REQ-SIM-08）。処理後 biomass ≥ mortality_threshold なら Alive、未満なら → Dying / 残り全量を carcass へ（ReasonCode::Death） | → Starving / energy 余剰なしのため繁殖しない（REQ-SIM-08: 不足時は繁殖 0） | → Starving / 代謝残差を waste へ |
| Dying | —（同一 phase 内で即座に Absent へ遷移するため到達不能） | —（同上） | → Absent / 全量 carcass 化を確定（tick を跨がない） | —（同上） | —（同上） |

- 補足: Starving → Alive の回復は starvation_and_death の処理結果にのみ依存する。intake で energy が回復しても当 tick 内では Starving のままとし、phase 順の決定性を優先する（確定。REQ-SIM-04 の固定順と整合）
- 補足: Absent への侵入経路は movement 軸の確定（D2）後に追加する。D1 時点では初期配置のみが Absent → Alive を起こす。参照: REQ-SIM-10

### 3.3 Lineage-in-cell 生成テスト雛形

```rust
// 生成: 任意の (biomass, energy) ∈ 0..=10^12 × 0..=scale、任意の mortality_threshold ∈ 1..=10^6、
//       任意のイベント列（5 イベントのランダム列、ただし phase 固定順の部分列のみ許可）
// 操作: イベント列を 1 tick 分として適用
// assert: 全プレフィックスで状態が 4 状態のいずれかに収まる
// assert: Dying に入ったプレフィックスは同一 tick 内に Absent で終わる
// assert: Absent では全イベントが無操作（biomass・energy・waste が不変）
// assert: 全遷移で質量保存（biomass 減少分 == carcass 増加分 + 変換係数分。INV-01/03）
// 参照: REQ-SIM-04, REQ-SIM-08, REQ-SIM-09
```

## 4. ADR 候補（REQ に無い設計判断）

- ADR 候補 1: `load` は既存 Run への適用を持たず、常に新規 Run を生成する（ハンドルの再定義を防ぐ）。参照: REQ-CON-01, REQ-DET-06
- ADR 候補 2: Terminated への `step` は Err とする（終了後の状態改変を防ぐ）。参照: REQ-END-01
- ADR 候補 3: Starving は tick を跨ぐ状態とし、回復判定を starvation_and_death に限定する（phase 順の決定性を優先）。参照: REQ-SIM-04
