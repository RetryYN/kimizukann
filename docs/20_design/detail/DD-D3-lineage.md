# DD-D3 詳細設計: 複数系統の intake / maintenance / reproduction（lineage 系 phase）

- 版: 0.3（起草 cursor-kimi、2026-08-30。r2: PR #26 契約審査を反映し §3/§4/§5/§6 を明確化、§9 を実数仕様化。r3: grok 敵対審査 r1（PR #27）と claude 裁定（D3-Q3/Q4、2026-08-30）を反映 — 丸めを TowardZero に統一、§9.1 の適用単位を固定、エネルギー台帳の account 対を本文化、§5 余剰式の二重半減を解消、UT-D3-13 を 64×64 同一タイルに修正、UT-D3-17 追加）。上位正本: BD-04 §3（Lineage-in-cell 状態機械）、BD-05 §2/§3/§4、BD-06 §3（P3〜P9）、BD-07 §2/§3、BD-01 r4 §5（台帳二段モデル）
- スコープ: 系統に作用する 5 phase（intake / maintenance / starvation_and_death / reproduction / emission）の**複数系統意味論**と台帳記録。格子への機械的一般化（走査ループ化）は D2（cursor-grok）が済ませている前提で、本 DD は振る舞いの新規定義を行う
- 前提: D2（cursor-grok）マージ済みであること。grid 一般化は D2 が導入した。**台帳基盤（LedgerRecord・region 集約・digest）は D2 では導入されなかった**（fold フックのみ）。LedgerRecord のエントリ生成は D3 が行い、region 集約（amount = 和）は `crates/sim-core/src/ledger.rs`（D3-A で新設。D3-Q3 = claude 裁定済み、§7）

## 1. 複数系統の同時存在（確定）

- セルは最大 8 系統の `biomass[L]` / `energy[L]` を持つ（BD-03 §1）。D1 は先頭系統のみに初期 biomass を置いたが、D3 では config の全系統が初期配置を持つ（配置位置の抽選自体は D4。D3 は config で与えられた配置をそのまま受け取る）。参照: REQ-SCOPE-01, REQ-GEN-01
- 系統に作用する全 phase の処理順は **セル row-major × 系統 ID 昇順の逐次**（按分しない。REQ-SIM-11）。先順系統がプールを消費すると後順系統の取り分は減る（これは仕様であり、公平化しない）
- `Lineage-in-cell` の 4 状態（Absent / Alive / Starving / Dying）と遷移は BD-04 §3.2 の表どおり。Starving からの回復は starvation_and_death のみ（intake で energy が回復しても当 tick は Starving のまま。ADR 候補 3）

## 2. intake（REQ-SIM-11、REQ-SCOPE-05 準備）

- 各 (cell, lineage) は系統 ID 昇順に、`mechanism_tags` が許す基質プール（`use_nutrient` → nutrient / `use_carcass` → carcass / `use_waste` → waste）からこの順に 1 回ずつ摂取する。確定（基質の列挙順は hash に効くため固定）
- 1 回の摂取量: `take = min(基質プール残量, base_intake × intake 倍率)`（i128 中間、P3）。`base_intake` = 0.1（Fixed 100,000）は初期仮説（D3 較正で確定、OPEN-02）。参照: REQ-SIM-11
- 配分: `take` を係数で biomass / waste へ分ける（余りは biomass へ、INV-03）。係数: nutrient 由来は biomass 0.70 + waste 0.30（BD-05 §4、初期仮説 D3 で確定）。carcass / waste 由来の係数は初期仮説として **biomass 0.50 + waste 0.50** を置く（D3 較正で確定。死骸利用系統の採算を決める値なので較正対象であることを明記）。参照: REQ-SIM-05, REQ-SCOPE-05
- energy 加算: `take × energy 係数（1.0）` を `energy[L]` に加え 1.0（Fixed 10^6）で飽和クランプ（P4b）。**溢出分は熱散逸としてエネルギー台帳に記録する**（BD-05 §3 補則。D1 は未記録だったが D3 で台帳導入）。確定。参照: REQ-SIM-08
- 台帳（reason = Intake）: 物質台帳は基質 → Biomass（係数分＋余り）と基質 → Waste（係数分）の 2 エントリ。エネルギー台帳は保持分 `<基質> → Biomass`（amount = take − heat）と熱散逸 `<基質> → Waste`（amount = heat = 1.0 クランプの溢出分）の最大 2 エントリ。**from 側の合計は take に一致**させ、基質アカウントの流出と帳尻を合わせる。熱のシンクは Waste アカウントで表す（系外散逸の代表。Pool 列挙に熱専用値は追加しない）。参照: REQ-SIM-05, REQ-SCOPE-04

## 3. maintenance（REQ-SIM-02/08）

- 維持コスト: `cost = max(1, mul(mul(base_maintenance, maintenance_cost 倍率), toxin 倍率))`（P5/P6）の 1 本の式で定義する。toxin 倍率は `waste > θ_w` かつ `toxin_sensitive` のとき 1.4、それ以外は 1.0（乗算自体は省略してよい）。**適用順は逐次**: 基礎 × 系統倍率（TowardZero 丸め）→ 毒条件なら ×1.4（TowardZero 丸め）→ 最後に下限ガード `max(1)`。`max(1)` の 1 は **Fixed 量子 1（= 1e-6）**であり表示単位の 1.0 ではない（積が 0 に丸まる縮退 config で無料維持を防ぐ）。`base_maintenance` = 0.01（Fixed 10,000）、θ_w = 0.1（Fixed 100,000）、1.4 は初期仮説（D3 較正で確定、OPEN-02。REQ-SIM-02）
- ガード: `energy ≥ cost` なら `energy −= cost` で Alive 維持。不足なら支払える分（= energy）だけ支払って `energy = 0` とし、**不足分 `cost − energy` を deficit として記録する**（BD-04 §3.2。D1 は未記録だったが D3 で保持）。Starving へ遷移。確定。参照: REQ-SIM-08
- 台帳: エネルギー台帳に Maintenance `Biomass → Waste`（amount = 実際に支払った量 = `min(cost, 支払前 energy)`）のみ。**不足分の台帳行は出さない**（deficit は実在するフローではなく状態。架空の変換行を置くと INV-11 の帳尻検査と台帳の意味論を壊す）。物質台帳は動かない

## 4. starvation_and_death（REQ-SIM-08/09）

- Starving の系統: 不足分（cost − energy）だけ `biomass → carcass`（reason = Starvation）。処理後 `biomass ≥ mortality_threshold` なら Alive に復帰、未満なら Dying へ。確定（BD-04 §3.2）
- **本 phase は Starving だけでなく Alive を含む全非 Absent 系統に適用する**（BD-04 §3.2 Alive 行）。Alive かつ `biomass < mortality_threshold` の系統は Starvation 処理を経ず直接 Dying へ（emission 等で Alive のまま閾値割れしうるため到達可能。PR #26 審査で明確化）
- Dying: `biomass < mortality_threshold` の系統は全量を carcass へ（reason = Death）。Dying は tick を跨がず同 phase 内で Absent へ。確定。参照: REQ-SIM-09
- 台帳: biomass → carcass の物質エントリ（reason = Starvation / Death）

## 5. reproduction（REQ-SIM-12）＋ 抽選導入（審査案件 D3-Q2）

- ガード: `energy > cost × 2`（初期仮説、D3 で確定）。**cost は §3 の維持コストと同一の値**（toxin 条項・下限ガード max(1) 込み。毒環境で維持不能な系統が繁殖判定だけ通ることを防ぐ。PR #26 審査で明確化）。不成立なら繁殖 0・**乱数消費 0**。確定（BD-04 §3.2）
- 増量: 余剰 `surplus = (energy − 2×cost) / 2`（TowardZero）、増分 `gain = min(surplus, nutrient)`。`energy −= gain`、`nutrient −= gain`、`biomass += gain`（energy→質量係数 1.0、初期仮説 D3 で確定。D1 と同じ構造。半減は surplus の定義にのみ含まれ、gain で再度割らない）。P8。参照: REQ-SIM-12
- **ガードに D1 の `traits.reproduction` 倍率は使わない**（BD-04 §3.2 の `energy > cost × 2` のみに合わせる。PT-D3-02 が traits を [0.5, 1.5] で振るため、ガードが traits に依存すると実装解釈が分かれる。D1 との意図的差分として記録）
- **抽選の導入（BD-07 §2/§3 の「D3 で確定」への回答。D3-Q2 = claude 判定で採用）**: ガード成立 (cell, lineage) ごとに reproduction ストリームから 1 語を消費し、`u / 2^64 < p_repro` なら繁殖成立。`p_repro` = 1.0 を初期仮説とする（D1 と同じ振る舞いを保ちつつ消費パターンだけ先に確定する。D3 較正で < 1.0 にするか判断）。**p_repro = 1.0 でも消費をスキップしてはならない**（消費回数は状態のみの関数。BD-07 §3）。消費回数表（BD-07 §3）は本 PR で更新済み
- 抽選導入は PRNG 消費が変わる＝振る舞い変更のため **model_version を bump**（`d3-v1`。BD-05 §14）。**D3-Q4 = claude 裁定済み（2026-08-30、(a)(1) 条件付き採用）**: H2 ゲート稼働後は model_version bump・`one_tick_reference` 更新・golden 再生成・照合文字列更新を**同一 PR**で行う（「別 PR」は verify 赤の中間状態を生むため構造的に不可）。条件: 新 oracle 期待値は writer（grok）が PR 本文に手計算の導出を書き、kimi が契約審査で独立に検算して approve に「検算一致」と明記する（golden の Claude 承認は kimi の検算一致票をもって代える）
- **D3-Q5 = claude 裁定（2026-08-30、(a)）**: version 文字列は phase 集合の完成時点で確定。中間 PR は同一文字列のまま golden を更新してよい（未リリースに限る）。`d3-v1` は D3 完成形（A+B）の識別子。
- 台帳: 物質台帳 `Nutrient → Biomass`（amount = gain）＋ エネルギー台帳 `Biomass → Waste`（amount = gain。繁殖仕事は熱として散逸）（reason = Reproduction）

## 6. emission（REQ-SIM-05）

- `amount = min(biomass[L], waste_emission)` を biomass → waste へ（reason = Emission）。D1 と同じ規則を複数系統に適用。P9。確定。台帳: 物質台帳 `Biomass → Waste`（amount）のみ。エネルギー台帳は動かない
- **排出後 `biomass[L] = 0` となった系統は同 phase 内で Absent へ遷移する**（BD-04 §3.2 の状態定義 Absent ⟺ biomass = 0 と整合させ、save/load 後の再構築（biomass からの life 復元）と分岐しないため。PR #26 審査で明確化。参照: REQ-DET-02, REQ-SIM-09）。なお BD-04 §3.2 表側の Alive × emission 行（→ Alive）の追従は D4 以降の BD 改訂で扱う（本 DD の範囲外）
- 代謝残差を捨てない（INV-03）。waste > θ_w かつ toxin_sensitive なら次 tick の maintenance で ×1.4（§3）

## 7. 台帳と region 集約（論点 D3-Q1 = claude 判定 r2 で確定）

- 全変換は LedgerEntry を生成し、tick 終了時に region 集約の LedgerRecord（キー tick→region_id→lineage→reason→from→to、amount = 和）へ畳む（BD-01 r4 §5・D2-Q1 確定どおり）。**D3-Q3 = claude 裁定済み（2026-08-30、(c)）**: 集約実装は `crates/sim-core/src/ledger.rs` を D3-A で新設する（writer = grok）。D2 の fold フック（`fold_diffuse_region_aggregates`）はこれを呼ぶ形に寄せる
- 台帳の account 対の総括（§2/§3/§5/§6 で定義。物質台帳は INV-11 どおり from 減量 = to 増量。エネルギー台帳は系外散逸を Waste アカウントで表す）:

| phase | 物質台帳（reason） | エネルギー台帳（reason） |
|---|---|---|
| intake | 基質→Biomass / 基質→Waste（Intake） | 保持: 基質→Biomass = take−heat、熱: 基質→Waste = heat（Intake） |
| maintenance | なし | Biomass→Waste = min(cost, energy)（Maintenance）。不足分は行を出さない |
| starvation / death | Biomass→Carcass（Starvation / Death） | なし |
| reproduction | Nutrient→Biomass = gain（Reproduction） | Biomass→Waste = gain（Reproduction） |
| emission | Biomass→Waste（Emission） | なし |
- **region は二層（確定）**:
  - **(A) 台帳 LedgerRecord.region_id = 静的 4×4 タイル**。64×64 を 16×16 セルのタイル 16 枚に分割し、`ID = (row/16)*4 + (col/16)`（row-major、0..=15）。tick をまたいで安定し、digest も窓集計も安定。空タイルも ID を持つ（nutrient のみの区画の資源枯渇を置ける）
  - **(B) スタンプの region_ids（REQ-EVT-04）= 動的 4 連結成分**。イベント tick の占有マスク（Σbiomass > 0）の 4 連結成分を row-major 初出順に採番（最大 16、超過は 15 に併合）。説明器側で派生計算し、保存は stamp 内のみ。**派生 ID を LedgerRecord に書かない**
  - 動的採番を台帳に使わない理由（grok 指摘を採用）: row-major 採番は新成分の出現で ID がずれ、digest/Save の region_id が tick 間で意味を失う。16 超の併合でも無関係な塊が同 ID になる

## 8. 境界値・エラー

- プール残量 < take は `min` でクランプされるため発生しない。負値・i64/i128 範囲外は `NumericError`（状態不変）。参照: REQ-SIM-13
- 全系統 Absent のセルは全 phase で無操作（BD-04 §3.2 Absent 行）
- ビット幅は BD-06 §3 の P3〜P9 の証明どおり（最大中間値 2×10^20、i128）。複数系統化で新たな乗算経路は増えない（逐次処理のため合算は発生しない）

## 9. UT / property 設計（BD-08: D3 は AT なし。UT・property で担保）

| UT-ID | 内容 | 期待 | 参照 |
|---|---|---|---|
| UT-D3-01 | 2 系統（ID 0, 1）が同一セルの nutrient を摂取 | ID 0 が先に上限まで取り、残りを ID 1 が取る（按分なし） | REQ-SIM-11 |
| UT-D3-02 | pool 十分時の摂取量 | take = base_intake × intake 倍率（上限式どおり） | REQ-SIM-11 |
| UT-D3-03 | use_carcass 系統（use_nutrient=false） | nutrient を取らず carcass から摂取 | REQ-SCOPE-05 |
| UT-D3-04 | energy 飽和 | 1.0 でクランプ、溢出分が熱散逸エントリに一致（P4b） | REQ-SIM-08, BD-05 §3 |
| UT-D3-05 | toxin: waste > θ_w | toxin_sensitive は cost ×1.4、非 sensitive は据置 | REQ-SIM-02 |
| UT-D3-06 | energy < cost | energy = 0・不足分記録・Starving 遷移 | REQ-SIM-08, BD-04 §3.2 |
| UT-D3-07 | Starving で不足分 < biomass | 不足分だけ carcass 化（Starvation）、biomass ≥ 閾値なら Alive 復帰 | REQ-SIM-08 |
| UT-D3-08 | biomass < mortality_threshold | 全量 carcass（Death）、同 tick に Absent | REQ-SIM-09 |
| UT-D3-09 | energy ≤ 2×cost | 繁殖 0・乱数消費 0 | REQ-SIM-12 |
| UT-D3-10 | ガード成立時の抽選 | reproduction ストリームをちょうど 1 語消費（消費カウンタ差分。p_repro=1.0 で必ず成立） | REQ-DET-04b, BD-07 §3 |
| UT-D3-11 | 繁殖の質量保存 | nutrient 減少量 = biomass 増加量（係数 1.0）、energy も同量減 | REQ-SIM-12, REQ-SIM-05 |
| UT-D3-12 | emission | waste_emission どおり biomass → waste | REQ-SIM-05 |
| UT-D3-13 | 台帳集約 | tick 終了時の LedgerRecord がキー順ソート済み・amount は和 | BD-01 r4 §5 |
| UT-D3-14 | PRNG 消費回数表 | 代表 config で step(1) 前後のカウンタ差分が BD-07 §3 更新版と一致 | REQ-DET-04b |
| UT-D3-15 | Alive かつ biomass < mortality_threshold | Starvation を経ず Death・同 phase に Absent（§4） | REQ-SIM-09, BD-04 §3.2 |
| UT-D3-16 | emission で biomass = 0 | 同 phase に Absent へ遷移し、次 tick の intake は無操作（§6） | REQ-SIM-09, REQ-DET-02 |
| UT-D3-17 | dual-tag 系統の複数プール摂取 | nutrient → carcass → waste の順に 1 回ずつ摂取（§2） | REQ-SIM-11, REQ-SCOPE-05 |
| PT-D3-01 | property: BD-04 §3.3 雛形を 8 系統に拡張 | 4 状態収束・Dying は同 tick Absent・Absent 無操作・質量保存 | REQ-SIM-04/08/09 |
| PT-D3-02 | property: ランダム 8 系統 config で 2,000 tick | INV-01/03/04（保存則・非負）・0 ≤ energy ≤ 1（INV-05） | REQ-SIM-06, REQ-SIM-08 |

- **テスト先行**: UT/PT を failing で commit してから実装する（テスト commit が実装 commit より前）

### 9.1 テスト仕様（実数。r2 で確定）

共通: Thresholds は既定値（`base_intake` = 100,000 / `base_maintenance` = 10,000 / `waste_toxic_threshold` = 100,000 / `toxin_maintenance_multiplier` = 1,400,000 / `occupancy_threshold` = 1,000,000）。テスト用系統は `traits` 全項 1.0（movement = 0）、`mortality_threshold` = 5,000、`waste_emission` = 1,000 とする。丸めは全て **TowardZero**（BD-06 §1/§4: i128 中間、商は切り捨て）。期待値は同じ丸めで事前計算した以下の実数に一致させること（テスト内で丸め関数を再実装しない）。

**適用単位（固定）**: 明示のない行は**当該 phase のみを 1 回適用**する（`apply_phase`。`tick_once` / `step` ではない）。`step` を使う行は config に明記する。`deficit` / `life` は WorldState の保存フィールドではなく導出量とする（実装が SimCore に保持するのは自由だが、state hash / save の対象外。§9.1 の config 欄に書く deficit / life は前期 phase の結果として導出される値をテストが設定するものとする）。UT-D3-06/07/08 の deficit は `energy` と `cost`（toxin 込み）から導出し、life は BD-04 §3.1 の導出どおりに検証する。

| UT-ID | config（1 セル、seed = 7） | 期待（実数） |
|---|---|---|
| UT-D3-01 | nutrient = 150,000。L0, L1（ともに use_nutrient）、biomass = 各 1,000,000、energy = 各 500,000 | Intake 後: nutrient = 0、biomass = [1,070,000 / 1,035,000]、waste = 45,000、energy = [600,000 / 550,000]。mass 台帳 4 件（L0: →Biomass 70,000 / →Waste 30,000、L1: →Biomass 35,000 / →Waste 15,000） |
| UT-D3-02 | nutrient = 1,000,000、L0、biomass = 1,000,000、energy = 500,000 | take = 100,000。nutrient = 900,000、biomass = 1,070,000、waste = 30,000、energy = 600,000 |
| UT-D3-03 | nutrient = 1,000,000、carcass = 1,000,000。L0 は use_carcass のみ | nutrient 不変。carcass = 900,000、biomass = 1,050,000、waste = 50,000（係数 0.50/0.50） |
| UT-D3-04 | nutrient = 1,000,000、L0、biomass = 1,000,000、energy = 999,990 | energy = 1,000,000（clip）。energy 台帳: Intake Nutrient→Biomass = 10（保持分）、Intake Nutrient→Waste = 99,990（熱散逸）のちょうど 2 件（§2 の account 対どおり from は基質プール） |
| UT-D3-05 | waste = 200,000（> θ_w）、energy = 1,000,000、biomass = 1,000,000。toxin_sensitive 系統と非 sensitive 系統を別々に実行 | sensitive: cost = 14,000 → energy = 986,000。非 sensitive: cost = 10,000 → energy = 990,000 |
| UT-D3-06 | energy = 3,000、biomass = 1,000,000 | energy = 0、deficit = 7,000（= cost 10,000 − 支払前 energy 3,000 の導出）、life = Starving。energy 台帳: Maintenance Biomass→Waste = 3,000（支払済分）のみ。**不足分の行は出ない**（§3） |
| UT-D3-07 | 導出状態: life = Starving、deficit = 10,000（maintenance を energy = 0・cost = 10,000 で通して導出）、biomass = 80,000 | biomass = 70,000、carcass = 10,000、life = Alive、deficit = 0。mass 台帳: Starvation Biomass→Carcass = 10,000 |
| UT-D3-08 | 導出状態: life = Starving、deficit = 10,000（UT-D3-07 と同じ導出）、biomass = 3,000 | biomass = 0、carcass = 3,000、life = Absent。Starvation エントリ = 3,000（Death エントリは残量 0 のため出ない） |
| UT-D3-09 | energy = 15,000（≤ 2×cost = 20,000）、nutrient = 1,000,000、biomass = 1,000,000 | Reproduction 後: rng[1] 不変・全状態不変 |
| UT-D3-10 | energy = 100,000（Reproduction 直入力。> 2×cost = 20,000）、nutrient = 1,000,000、biomass = 1,000,000 | rng[1] が**ちょうど 1 語**進む: phase 前の rng[1] を clone して `next_u64()` を 1 回呼んだ参照と `words()` が一致することで検証（`assert_ne` のみでは不可） |
| UT-D3-11 | UT-D3-10 と同じ（Reproduction 直入力 energy = 100,000） | surplus = (100,000 − 20,000) / 2 = 40,000、gain = min(40,000, nutrient) = 40,000。nutrient = 960,000、biomass = 1,040,000、energy = 60,000。mass 台帳: Reproduction Nutrient→Biomass = 40,000。energy 台帳: Reproduction Biomass→Waste = 40,000 |
| UT-D3-12 | biomass = 50,000 | Emission 後: biomass = 49,000、waste = 1,000。mass 台帳: Emission Biomass→Waste = 1,000 |
| UT-D3-13 | **64×64 grid** の同一タイル内 2 セル（row 0, col 0）と（row 0, col 1）（§7 (A) の式でともに region 0。小格子への外挿は使わない）。各セルに L0・nutrient 十分で Intake を実行し fold | キー (tick = 0, region = 0, lineage = 0, Intake, Nutrient, Biomass) のレコードが **1 件に和約**され amount = 140,000（= 70,000 × 2）。全レコードがキー順ソート済み。和約は `ledger.rs`（§7、D3-Q3 裁定済） |
| UT-D3-14 | energy = 100,000、nutrient = 1,000,000、biomass = 1,000,000 で **step(1)**（step 前 energy = 100,000。Intake +100,000 → Maintenance −10,000 で Reproduction 直前 energy = 190,000 > 20,000） | rng[1]（reproduction）がちょうど 1 語進む（UT-D3-10 と同じ参照比較）。rng[0]/[2]/[3] は新規インスタンスと同一（消費 0）。BD-07 §3 表: diffuse 0 / reproduction 1 / 他 0 |
| UT-D3-15 | life = Alive、biomass = 4,000（< 5,000）、energy = 1,000,000 | StarvationAndDeath 後: biomass = 0、carcass = 4,000、life = Absent。mass 台帳: Death Biomass→Carcass = 4,000 のみ（Starvation エントリなし） |
| UT-D3-16 | life = Alive、biomass = 5,000、waste_emission = 10,000 の系統 | Emission 後: biomass = 0、waste = 5,000、life = Absent。続けて nutrient 十分で Intake を適用しても nutrient 不変（Absent 無操作） |
| UT-D3-17 | dual-tag: L0 は use_nutrient + use_waste。nutrient = 100,000、waste = 100,000、biomass = 1,000,000、energy = 0 | Intake 後: nutrient から take = 100,000（biomass +70,000 / waste +30,000）、続けて waste から take = 100,000（biomass +50,000 / waste +50,000）。nutrient = 0、waste = 80,000、biomass = 1,120,000、energy = 200,000。mass 台帳 4 件（基質別に 2 件ずつ）、energy 台帳 Nutrient→Biomass = 100,000 / Waste→Biomass = 100,000（heat なし） |
| PT-D3-01 | 8 系統（id 0..7、use_nutrient）、nutrient = 10,000,000、biomass = 各 20,000、energy = 各 500,000 | step(8): **各 tick 後に** total_mass 不変・非負。全 (cell, lineage) の life ∈ {Absent, Alive, Starving} |
| PT-D3-02 | 8 系統: 固定シードの SplitMix64 で traits ∈ [0.5, 1.5]・mortality_threshold ∈ [1, 10,000]・waste_emission ∈ [0, 5,000] を決定的に生成（生成手順をテスト内に固定記述）。nutrient = 8,000,000、初期 biomass = 先頭系統 1,000,000・他 100,000 | step(2,000): 各 tick 後に INV-01/03/04（保存則・非負）・全 energy ∈ [0, 1,000,000]（INV-05） |

## 10. 性能

- 全 7 phase 込みの予算は PB-01（床 6 ms / PC 0.5 ms、BD-09）。D3 時点では PC で PB-06（headless 2,000 tick ≤ 1.0 s）を維持すること。criterion ベンチは D2 のものを流用し、悪化時は cause を特定して報告

## 11. ファイル分割（TEAM-2core: 実装は cursor-grok が D2/D3 ともに担当。確定）

| ファイル | 担当 PR | 内容 |
|---|---|---|
| `crates/sim-core/src/grid.rs`, `diffuse.rs` | D2（cursor-grok） | grid 一般化・diffuse（実績: lib.rs 内に実装） |
| `crates/sim-core/src/ledger.rs` | **D3-A（cursor-grok）** | LedgerRecord・region 集約 `fold_region_records`（D3-Q3 裁定 (c) により D3-A で新設） |
| `crates/sim-core/src/lib.rs` | D2（cursor-grok） | SimCore 本体・tick_once。D3 は `mod lineage_phases;` 追加と phase 呼出の差替えのみ（D2 マージ後の姿に追従） |
| `crates/sim-core/src/lineage_phases.rs` | **D3（cursor-grok）** | intake / maintenance / starvation_and_death / reproduction / emission の複数系統意味論（本 DD） |
| `crates/sim-core/tests/d3_*.rs` | **D3（cursor-grok）** | §9 の UT/PT |
| `crates/sim-core/tests/d2_*.rs`, `benches/` | D2（cursor-grok） | D2 のテスト・ベンチ |
| `docs/**`, `clippy.toml`, golden | 触らない | golden 更新は D3-Q4 裁定 (a)(1) どおり**同一 PR + 検算票**（§5。「別 PR」は H2 稼働後は構造的に不可） |

- 未決事項: なし。D3-Q3（台帳和約の所在 → §7）・D3-Q4（model_version bump と golden 更新の手順 → §5）は 2026-08-30 の claude 裁定で確定

- 依存順序: D3 実装は D2 マージ後に着手（grid 一般化に依存。ledger 集約は D3-A で新設）。同一実装者（grok）が D2 → D3 の順で担当するためファイル衝突は発生しない。不明点は `[D3-lineage-001][question]` を cursor-kimi へ（NETWORK 規則: grok→kimi=[question]）
