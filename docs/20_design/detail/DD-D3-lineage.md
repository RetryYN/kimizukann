# DD-D3 詳細設計: 複数系統の intake / maintenance / reproduction（lineage 系 phase）

- 版: 0.1（起草 cursor-kimi、2026-08-30）。上位正本: BD-04 §3（Lineage-in-cell 状態機械）、BD-05 §2/§3/§4、BD-06 §3（P3〜P9）、BD-07 §2/§3、BD-01 r4 §5（台帳二段モデル）
- スコープ: 系統に作用する 5 phase（intake / maintenance / starvation_and_death / reproduction / emission）の**複数系統意味論**と台帳記録。格子への機械的一般化（走査ループ化）は D2（composer）が済ませている前提で、本 DD は振る舞いの新規定義を行う
- 前提: D2（cursor-grok）マージ済みであること。台帳基盤（LedgerRecord・region 集約・digest）と grid 一般化は D2 が導入する（§11 のファイル分割）

## 1. 複数系統の同時存在（確定）

- セルは最大 8 系統の `biomass[L]` / `energy[L]` を持つ（BD-03 §1）。D1 は先頭系統のみに初期 biomass を置いたが、D3 では config の全系統が初期配置を持つ（配置位置の抽選自体は D4。D3 は config で与えられた配置をそのまま受け取る）。参照: REQ-SCOPE-01, REQ-GEN-01
- 系統に作用する全 phase の処理順は **セル row-major × 系統 ID 昇順の逐次**（按分しない。REQ-SIM-11）。先順系統がプールを消費すると後順系統の取り分は減る（これは仕様であり、公平化しない）
- `Lineage-in-cell` の 4 状態（Absent / Alive / Starving / Dying）と遷移は BD-04 §3.2 の表どおり。Starving からの回復は starvation_and_death のみ（intake で energy が回復しても当 tick は Starving のまま。ADR 候補 3）

## 2. intake（REQ-SIM-11、REQ-SCOPE-05 準備）

- 各 (cell, lineage) は系統 ID 昇順に、`mechanism_tags` が許す基質プール（`use_nutrient` → nutrient / `use_carcass` → carcass / `use_waste` → waste）からこの順に 1 回ずつ摂取する。確定（基質の列挙順は hash に効くため固定）
- 1 回の摂取量: `take = min(基質プール残量, base_intake × intake 倍率)`（i128 中間、P3）。`base_intake` = 0.1（Fixed 100,000）は初期仮説（D3 較正で確定、OPEN-02）。参照: REQ-SIM-11
- 配分: `take` を係数で biomass / waste へ分ける（余りは biomass へ、INV-03）。係数: nutrient 由来は biomass 0.70 + waste 0.30（BD-05 §4、初期仮説 D3 で確定）。carcass / waste 由来の係数は初期仮説として **biomass 0.50 + waste 0.50** を置く（D3 較正で確定。死骸利用系統の採算を決める値なので較正対象であることを明記）。参照: REQ-SIM-05, REQ-SCOPE-05
- energy 加算: `take × energy 係数（1.0）` を `energy[L]` に加え 1.0（Fixed 10^6）で飽和クランプ（P4b）。**溢出分は熱散逸としてエネルギー台帳に記録する**（BD-05 §3 補則。D1 は未記録だったが D3 で台帳導入）。確定。参照: REQ-SIM-08
- 台帳: 基質 → biomass（係数分＋余り）と基質 → waste（係数分）の 2 エントリ（reason = Intake）＋ エネルギー台帳に摂取加算と熱散逸。参照: REQ-SIM-05, REQ-SCOPE-04

## 3. maintenance（REQ-SIM-02/08）

- 維持コスト: `cost = base_maintenance × maintenance_cost 倍率`（P5）。`base_maintenance` = 0.01（Fixed 10,000）は初期仮説（D3 較正で確定、OPEN-02）
- 毒: `waste > θ_w` かつ `toxin_sensitive` なら cost に ×1.4（P6。1.4 は初期仮説、D3 実測で更新しうる。REQ-SIM-02）。θ_w = 0.1（Fixed 100,000）は初期仮説（OPEN-02）
- ガード: `energy ≥ cost` なら `energy −= cost` で Alive 維持。不足なら `energy = 0` とし、**不足分 `cost − energy` を記録する**（BD-04 §3.2。D1 は未記録だったが D3 で台帳に記録）。Starving へ遷移。確定。参照: REQ-SIM-08
- 台帳: エネルギー台帳に Maintenance 消費（不足時は不足分も）。物質台帳は動かない

## 4. starvation_and_death（REQ-SIM-08/09）

- Starving の系統: 不足分（cost − energy）だけ `biomass → carcass`（reason = Starvation）。処理後 `biomass ≥ mortality_threshold` なら Alive に復帰、未満なら Dying へ。確定（BD-04 §3.2）
- Dying: `biomass < mortality_threshold` の系統は全量を carcass へ（reason = Death）。Dying は tick を跨がず同 phase 内で Absent へ。確定。参照: REQ-SIM-09
- 台帳: biomass → carcass の物質エントリ（reason = Starvation / Death）

## 5. reproduction（REQ-SIM-12）＋ 抽選導入（審査案件 D3-Q2）

- ガード: `energy > cost × 2`（初期仮説、D3 で確定）。不成立なら繁殖 0・**乱数消費 0**。確定（BD-04 §3.2）
- 増量: 余剰 `(energy − 2×cost) / 2` を上限に、質量は同量を nutrient から引いて biomass へ（energy→質量係数 1.0、初期仮説 D3 で確定）。質量保存のため増分は `min(余剰/2, nutrient)` でクランプ（D1 と同じ構造）。P8。参照: REQ-SIM-12
- **抽選の導入（BD-07 §2/§3 の「D3 で確定」への回答。D3-Q2 = claude 判定で採用）**: ガード成立 (cell, lineage) ごとに reproduction ストリームから 1 語を消費し、`u / 2^64 < p_repro` なら繁殖成立。`p_repro` = 1.0 を初期仮説とする（D1 と同じ振る舞いを保ちつつ消費パターンだけ先に確定する。D3 較正で < 1.0 にするか判断）。**p_repro = 1.0 でも消費をスキップしてはならない**（消費回数は状態のみの関数。BD-07 §3）。消費回数表（BD-07 §3）は本 PR で更新済み
- 抽選導入は PRNG 消費が変わる＝振る舞い変更のため **model_version を bump**（`d3-v1`。BD-05 §14）。golden hash の更新は Claude 承認・別 PR
- 台帳: nutrient → biomass（reason = Reproduction）＋ エネルギー台帳に Reproduction 消費

## 6. emission（REQ-SIM-05）

- `amount = min(biomass[L], waste_emission)` を biomass → waste へ（reason = Emission）。D1 と同じ規則を複数系統に適用。P9。確定
- 代謝残差を捨てない（INV-03）。waste > θ_w かつ toxin_sensitive なら次 tick の maintenance で ×1.4（§3）

## 7. 台帳と region 集約（論点 D3-Q1 = claude 判定 r2 で確定）

- 全変換は LedgerEntry を生成し、tick 終了時に region 集約の LedgerRecord（キー tick→region_id→lineage→reason→from→to、amount = 和）へ畳む（BD-01 r4 §5・D2-Q1 確定どおり）。集約実装は `crates/sim-core/src/ledger.rs`（D3-A。D2 は fold hook のみ）。D3-A は intake/maintenance のエントリ生成を追加する
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
| PT-D3-01 | property: BD-04 §3.3 雛形を 8 系統に拡張 | 4 状態収束・Dying は同 tick Absent・Absent 無操作・質量保存 | REQ-SIM-04/08/09 |
| PT-D3-02 | property: ランダム 8 系統 config で 2,000 tick | INV-01/03/04（保存則・非負）・0 ≤ energy ≤ 1（INV-05） | REQ-SIM-06, REQ-SIM-08 |

- **テスト先行**: UT/PT を failing で commit してから実装する（テスト commit が実装 commit より前）

## 10. 性能

- 全 7 phase 込みの予算は PB-01（床 6 ms / PC 0.5 ms、BD-09）。D3 時点では PC で PB-06（headless 2,000 tick ≤ 1.0 s）を維持すること。criterion ベンチは D2 のものを流用し、悪化時は cause を特定して報告

## 11. ファイル分割（TEAM-2core: 実装は cursor-grok が D2/D3 ともに担当。確定）

| ファイル | 担当 PR | 内容 |
|---|---|---|
| `crates/sim-core/src/grid.rs`, `diffuse.rs` | D2（cursor-grok） | grid 一般化・diffuse。`ledger.rs` の集約は D3-A で新設 |
| `crates/sim-core/src/lib.rs` | D2（cursor-grok） | SimCore 本体・tick_once。D3 は `mod lineage_phases;` 追加と phase 呼出の差替えのみ（D2 マージ後の姿に追従） |
| `crates/sim-core/src/lineage_phases.rs` | **D3（cursor-grok）** | intake / maintenance / starvation_and_death / reproduction / emission の複数系統意味論（本 DD） |
| `crates/sim-core/tests/d3_*.rs` | **D3（cursor-grok）** | §9 の UT/PT |
| `crates/sim-core/tests/d2_*.rs`, `benches/` | D2（cursor-grok） | D2 のテスト・ベンチ |
| `docs/**`, `clippy.toml`, golden | 触らない | golden 更新は Claude 承認・別 PR |

- 依存順序: D3 実装は D2 マージ後に着手（ledger 基盤と grid 一般化に依存）。同一実装者（grok）が D2 → D3 の順で担当するためファイル衝突は発生しない。不明点は `[D3-lineage-001][question]` を cursor-kimi へ（NETWORK 規則: grok→kimi=[question]）
