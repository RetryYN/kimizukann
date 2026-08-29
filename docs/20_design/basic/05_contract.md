# BD-05 契約（公開 API・FFI・schema・事前事後条件・semver）

- 版: 0.2（起草 cursor-kimi、2026-08-30。PR #1 レビュー r1 反映: hash v2 の追加フィールド、FFI 表を BD-01 r2 §3 に整合、StreamKind 用途の完全割当）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）、`docs/30_contracts/simulation_contract.md` v0.1
- 本章は `simulation_contract.md` v0.1 を吸収・再編した**契約の正本**である。`simulation_contract.md` は v0.1 として凍結し、以後の変更は本章を RFC 経由で改訂する（移行の扱いは §8）
- 完成条件: `cargo public-api` の出力と一致。各 pub 項目に REQ 参照
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。ドメイン集約・不変条件は BD-03、状態機械は BD-04、ビット幅の証明は BD-06、乱数消費回数は BD-07 を参照

## 1. 状態（契約 §1 吸収）

1 セルは `nutrient: Fixed`、`biomass[L]: Fixed`、`carcass: Fixed`、`waste: Fixed`、`energy[L]: Fixed`、`occupancy_peak: Fixed` の 6 状態のみを持ち、全体状態は `tick: u32` を持つ。`L ≤ 8`。物質の単位は `mass_u`（固定小数点の最小単位）、エネルギーは無次元の系統別予算。参照: REQ-SIM-01, REQ-SIM-08

さらに run 状態として `fixed_streak: u32`（Fixed 継続カウンタ）、`tick0_ranking: [u8; 8]`（tick 0 順位。Reversal 判定用）、`inflow_cursor: u32`（InflowEvent 消化位置）を持つ（BD-01 r2、ADR-0003 改定）。`fixed_streak` と `tick0_ranking` は履歴依存で保存時点から再計算不能であり、`inflow_cursor` は厳密には tick と config から導出可能だが BD-01 r2 の決定に従い状態として保持する。3 者とも hash 正規化に含める（§10、`hash=sha256-v2`）。確定。参照: REQ-END-03, REQ-END-04b, REQ-SIM-07, REQ-DET-05

## 2. tick 順序（契約 §2 吸収）

各 tick は固定順 7 phase を一度だけ実行する: diffuse → intake → maintenance → starvation_and_death → reproduction → emission → occupancy。走査順はセル row-major、系統 ID 昇順、近傍は北・東・南・西の固定順。順序の入替は契約違反。確定。参照: REQ-SIM-04, REQ-DET-04c

inflow の適用位置: `InflowEvent` は tick 先頭（diffuse 直前）に該当プールへ加算する。7 phase には含めない。同 tick 複数件は config 配列の出現順。確定。参照: REQ-SIM-07, REQ-DET-04c

## 3. 物質・エネルギー二重台帳（契約 §3 吸収）

物質台帳は `nutrient + Σbiomass + carcass + waste` を追跡する。流入は `Vec<InflowEvent { tick, pool, amount }>`（閉鎖系は空 Vec）。エネルギー台帳は系統ごとに摂取加算・維持/移動/繁殖への配分・熱散逸を追跡し、熱散逸は物質を減らさない。全変換は `LedgerEntry { from_pool, to_pool, amount, reason }` を通し、負値と未記録残差を禁止する。確定。参照: REQ-SIM-05, REQ-SIM-07, REQ-SCOPE-04

`LedgerEntry` は `tick: u32, cell_index: u32, lineage: u8（系統なしは 255）, from_pool, to_pool, amount, reason` を持つ（BD-01 r3 の台帳ダイジェスト定義に合わせる）。確定。参照: REQ-SIM-05, REQ-EVT-04

エネルギー台帳の補則: intake の energy 加算は 1.0（Fixed では 10^6）で飽和し、溢出分は熱散逸としてエネルギー台帳に記録する。熱散逸は物質を減らさない。確定。参照: REQ-SIM-08

**台帳ダイジェスト**（state hash 外だが三経路 AT で比較。BD-01 r3 §5 と一致）: `SHA-256(台帳エントリを tick, cell_index, lineage, reason, from, to, amount の順に LE 直列化 → スタンプを tick, kind, region_ids の順 → z 窓を pool, 系統, 値の順)`。確定。参照: REQ-DET-02, REQ-EVT-04

保存（LedgerSave）には region 単位に集約したレコード・スタンプ列・z 窓のみを含め、セル単位の全 tick 履歴は保存しない。これにより転換点の region_ids（REQ-EVT-04）と保存 ≤ 5 MB（REQ-NFR-02）を両立する。確定。参照: REQ-EVT-04, REQ-NFR-02

## 4. 質量係数表（契約 §4 吸収）

係数は入力質量に対する出力質量の割合で、合計 1.0（固定小数点 1_000_000）。余りは常に主出力プールへ戻し、拡散の余りは送り元セルに残す。捨てる経路は存在しない。参照: REQ-SIM-05

| 変換 | 出力（係数） | 余り戻し先 | 確定度 |
|---|---|---|---|
| 摂取 | biomass 0.70 ＋ waste 0.30 | biomass（主出力。余りは 1 変換単位で `入力 − 0.70・入力 − 0.30・入力` として計算し biomass に戻す） | 初期仮説（D3 で確定） |
| biomass 維持不足 | carcass 1.00 | carcass（主出力） | 確定 |
| biomass 死亡 | carcass 1.00 | carcass（主出力） | 確定 |

係数は config の hash に含める。エネルギー係数は物質係数と別の無次元値。参照: REQ-SIM-05, REQ-DET-05

## 5. 固定小数点・丸め規則（契約 §5 吸収）

コア状態と係数は i64 固定小数点 scale = 1_000_000、乗算中間は i128、除算はゼロ方向丸め 1 種。変換余りは主出力プールへ、拡散余りは送り元セルへ。負値・i64 範囲外・i128 中間範囲外は `NumericError`。解析用 z-score のみ浮動小数点を許すがコア状態へ戻さない。丸めモードと scale は model_version の一部。確定。上限とビット幅の証明は BD-06。参照: REQ-CON-02, REQ-SIM-13

## 6. PRNG・用途別ストリーム（契約 §6 吸収）

`SplitMix64` で seed から 4 ストリーム（movement / reproduction / mutation / interaction）の初期状態を導出し、各ストリームは xoshiro256**。バージョン文字列 `prng=xoshiro256ss-v1`。表示用サンプリングはコア乱数を消費しない。seed 内は単一スレッド、seed 間バッチのみ並列化。確定。phase 割当と消費回数表は BD-07。参照: REQ-DET-04a, REQ-DET-04b

4 ストリーム全てに用途を割り当て、「予備」は存在しない。`mutation` は検証版で変異の計算侵入がないため（REQ-OUT-01）**初期配置のばらつき抽選専用**で、tick 中の消費は常に 0。`interaction` は**初期配置の位置抽選**に割り当て、同じく tick 中の消費は常に 0（割当の詳細は BD-07 §2/§3）。確定。参照: REQ-OUT-01, REQ-GEN-08, REQ-SCOPE-03, REQ-DET-04b

## 7. 機構タグ・系統定数・適応ベクトル（契約 §7 吸収）

機構タグは 5 ビット（use_nutrient / use_carcass / use_waste / toxin_sensitive / density_bonus）、札の 5 軸は movement / intake / conversion / maintenance_cost / reproduction の固定倍率、系統定数は mortality_threshold と waste_emission の 2 つのみ。タグと定数はプリセット固定でプレイヤーは札のみ選ぶ。確定。参照: REQ-GEN-01, REQ-GEN-02

- 拡散係数: 各プール 0.05 / 近傍 / tick（初期仮説、D2 で確定）。参照: REQ-SIM-10
- 摂取 1 tick 上限: intake 倍率 × 基準摂取量（初期仮説、D3 で確定）。参照: REQ-SIM-11
- 繁殖条件: energy > 維持コスト × 2（初期仮説、D3 で確定）。参照: REQ-SIM-12
- energy→質量係数: 1.0（初期仮説、D3 で確定）。参照: REQ-SIM-12

## 8. 終了ラベルと閾値（契約 §8 吸収）

5 ラベルのみ: Extinct / Fixed / Coexist / Reversal / TimeLimit。Extinct・Fixed は毎 tick 判定し即終了、Coexist・Reversal は上限到達時のみ判定。優先順 Extinct > Fixed > Coexist > Reversal > TimeLimit。同時成立時は優先順を適用し判定理由を保存する。確定。参照: REQ-END-01, REQ-END-04c

| ラベル | 条件 | 確定度 | 参照 |
|---|---|---|---|
| Extinct | 全系統生体量 < ε（ε = 1e-4 × 初期総生体量） | 確定 | REQ-END-02 |
| Fixed | 1 系統が総生体量 70% 以上を 200 tick 継続 | 確定 | REQ-END-03 |
| Coexist | 上限 tick 時点で 2 系統以上が各 15% 以上 | 確定 | REQ-END-04a |
| Reversal | 終了時 1 位が tick 0 順位で 3 位以下（生体量降順・同率は ID 昇順） | 確定 | REQ-END-04b |
| TimeLimit | 2,000 tick 到達 | 確定 | REQ-SCOPE-06 |

閾値: θ_w（初期仮説、D3）、θ_occ（初期仮説、D2）、toxin_maintenance_multiplier = 1.4（初期仮説、D3 の実測で更新しうる）。参照: REQ-SIM-02, REQ-SIM-03a

## 9. 保存則テスト定義（契約 §9 吸収）

閉鎖系では各 tick および最終状態で物質 4 プール総量が初期総量と Fixed 厳密一致。流入プリセットでは初期総量 + Σ inflow.amount と一致。全プール非負。energy は負値禁止、熱散逸を含む差分を台帳化。必須テスト: 1 セル 2,000 tick、64×64 拡散 2,000 tick、一様場不変、境界流出なし、係数合計、残余非消失、starvation の carcass 移動。確定。参照: REQ-SIM-06, REQ-SIM-07, REQ-SIM-13

## 10. 三経路一致・state hash（契約 §10 吸収）

同一 model_version・config・seed で `step(2000)` ≡ `step(1) × 2000` ≡ 任意 tick で save → load → 残り、が同一の正規化 state hash と終了ラベルを返す。hash は SHA-256（`hash=sha256-v2`。ADR-0003 改定）。正規化は tick・seed・寸法・全セル（row-major・系統昇順・i64 バイト列）・PRNG 4 ストリーム状態・**Fixed 継続カウンタ（u32）・tick 0 順位（u8×L）・inflow 消化位置（u32）**・model_version を含み、描画用トークン・UI・ログ時刻を除く。速度変更・描画間引き・トークン有無・seed 間並列数は hash を変えない。確定。検証手順は BD-07 §5。参照: REQ-DET-02, REQ-DET-05, REQ-OUT-04, REQ-DET-07

## 11. 公開 API（Rust シグネチャ）

`cargo public-api` の出力と一致させる。全 pub 項目と REQ 参照の対応を以下に固定する。pub 項目の追加・削除・シグネチャ変更は本章の RFC を伴う（REQ-NFR-06）。

### 11.1 crate `kimizukann-sim-types`

| pub 項目 | 種別 | 参照 |
|---|---|---|
| `FIXED_SCALE: i64 = 1_000_000` | const | REQ-CON-02 |
| `Fixed = i64` | type | REQ-CON-02 |
| `Pool { Nutrient, Biomass, Carcass, Waste }` | enum | REQ-SIM-02 |
| `ReasonCode { Intake, Maintenance, Starvation, Death, Reproduction, Emission, Diffusion }` | enum | REQ-SCOPE-04 |
| `TickPhase { Diffuse, Intake, Maintenance, StarvationAndDeath, Reproduction, Emission, Occupancy }` | enum | REQ-SIM-04 |
| `TerminationLabel { Extinct, Fixed, Coexist, Reversal, TimeLimit }` | enum | REQ-END-01 |
| `MechanismTags { use_nutrient, use_carcass, use_waste, toxin_sensitive, density_bonus }` | struct | REQ-GEN-01 |
| `TraitVector { movement, intake, conversion, maintenance_cost, reproduction }` | struct | REQ-GEN-01 |
| `LineageParams { id: u8, traits, tags, mortality_threshold, waste_emission }` | struct | REQ-GEN-01 |
| `CellState { nutrient, biomass: [Fixed; 8], carcass, waste, energy: [Fixed; 8], occupancy_peak }` | struct | REQ-SIM-01 |
| `GridState { width: u16, height: u16, cells }` | struct | REQ-SIM-01 |
| `WorldState { tick: u32, grid, lineages, fixed_streak: u32, tick0_ranking: [u8; 8], inflow_cursor: u32 }` | struct（fixed_streak / tick0_ranking / inflow_cursor は hash v2 で追加。BD-01 r2。sim-types への反映は D2 実装タスク） | REQ-SIM-01, REQ-END-03, REQ-END-04b, REQ-DET-05 |
| `Seed(pub u64)` | struct | REQ-DET-04a |
| `PrngState { seed, movement, reproduction, mutation, interaction }` | struct | REQ-DET-04a, REQ-DET-05 |
| `StateHash(pub [u8; 32])` | struct | REQ-DET-05 |
| `Thresholds`（フィールドは下表に分離） | struct | REQ-END-02, REQ-END-03, REQ-END-04a, REQ-SIM-02, REQ-SIM-03 |
| `SaveEnvelope { schema_version, model_version, config_hash, seed, prng, state_hash, state }` | struct | REQ-DET-06 |
| `MassCoefficients { intake_to_biomass, intake_to_waste, starvation_to_carcass, death_to_carcass }` | struct | REQ-SIM-05 |
| `LedgerEntry { tick: u32, cell_index: u32, lineage: u8, from_pool, to_pool, amount, reason }` | struct（tick/cell_index/lineage は台帳ダイジェスト用。BD-01 r3。sim-types への反映は D2 実装タスク） | REQ-SIM-05, REQ-EVT-04 |
| `MassLedger { entries }` / `EnergyLedger { entries }` | struct | REQ-SIM-05, REQ-SCOPE-04 |
| `InvariantReport { mass_ok, energy_ok, non_negative, message }` | struct | REQ-SIM-06, REQ-OPS-01 |
| `RoundingMode { TowardZero }` | enum | REQ-CON-02 |
| `NumericError { Negative, OverflowI64, OverflowI128 }` | enum | REQ-SIM-13 |
| `VerifySuite` | struct | REQ-OPS-01 |
| `ModelVersion { major, minor, scale, rounding, prng = "xoshiro256ss-v1", hash = "sha256-v2" }` | struct | REQ-DET-05, REQ-NFR-06 |
| `ScanOrder` | struct | REQ-DET-04c |
| `RandomStream` | struct | REQ-DET-04a |
| `StreamKind { Movement, Reproduction, Mutation, Interaction }` | enum | REQ-DET-04a |
| `Substrate { Nutrient, Carcass, Waste }` | enum | REQ-SIM-02, REQ-SCOPE-05 |
| `InflowEvent { tick, pool, amount }` | struct | REQ-SIM-07 |
| `ConversionRule { from, to, coefficient, remainder_to }` | struct | REQ-SIM-05 |
| `TerminationTiming { EveryTick, AtTimeLimit }` | enum | REQ-END-04c |
| `TerminationRule { label, timing, priority }` | struct | REQ-END-01, REQ-END-04c |
| `StateSnapshot { state, prng }` | struct | REQ-DET-02, REQ-VIS-04 |

`Thresholds` のフィールド（1 フィールド 1 行）:

| フィールド | 型 | 意味 | 確定度 | 参照 |
|---|---|---|---|---|
| `base_intake` | Fixed | 基準摂取量（1 tick 上限 = intake 倍率 × 本値） | 初期仮説（D3 で確定） | REQ-SIM-11 |
| `base_maintenance` | Fixed | 基準維持コスト | 初期仮説（D3 で確定） | REQ-SIM-12 |
| `epsilon` | Fixed | 1e-4 × 初期総生体量 | 確定 | REQ-END-02 |
| `fixed_share` | Fixed | 0.70（総生体量比） | 確定 | REQ-END-03 |
| `fixed_ticks` | u32 | 200 tick 継続 | 確定 | REQ-END-03 |
| `coexist_share` | Fixed | 0.15（各系統） | 確定 | REQ-END-04a |
| `max_ticks` | u32 | 2,000 | 確定 | REQ-SCOPE-06 |
| `waste_toxic_threshold` | Fixed | θ_w | 初期仮説（D3 で確定） | REQ-SIM-02 |
| `toxin_maintenance_multiplier` | Fixed | 1.4 | 初期仮説（D3 の実測で更新しうる） | REQ-SIM-02 |
| `occupancy_threshold` | Fixed | θ_occ | 初期仮説（D2 で確定） | REQ-SIM-03a |
| `vacant_nutrient_threshold` | Fixed | セル栄養の初期中央値の 10% | 確定 | REQ-SIM-03b |

### 11.2 crate `kimizukann-sim-core`

| pub 項目 | シグネチャ | 参照 |
|---|---|---|
| `fixed::add` | `fn add(a: Fixed, b: Fixed) -> Result<Fixed, NumericError>` | REQ-CON-02, REQ-SIM-13 |
| `fixed::sub_nonnegative` | `fn sub_nonnegative(a: Fixed, b: Fixed) -> Result<Fixed, NumericError>` | REQ-SIM-13 |
| `fixed::mul` | `fn mul(a: Fixed, b: Fixed) -> Result<Fixed, NumericError>`（i128 中間） | REQ-CON-02 |
| `fixed::div` | `fn div(a: Fixed, b: Fixed) -> Result<Fixed, NumericError>`（ゼロ方向） | REQ-CON-02 |
| `fixed::split_output` | `fn split_output(input: Fixed, coefficient: Fixed) -> Result<(Fixed, Fixed), NumericError>` | REQ-SIM-05 |
| `fixed::split_output_with_rule` | `fn split_output_with_rule(input: Fixed, rule: &ConversionRule, waste_coefficient: Fixed) -> Result<(Fixed, Fixed), NumericError>` | REQ-SIM-05 |
| `Xoshiro256StarStar::from_seed` | `fn from_seed(seed: u64) -> Self` | REQ-DET-04a |
| `Xoshiro256StarStar::next_u64` | `fn next_u64(&mut self) -> u64` | REQ-DET-04a |
| `Xoshiro256StarStar::words` | `fn words(&self) -> [u64; 4]`（hash 用内部状態公開） | REQ-DET-05 |
| `SimCore::one_cell` / `try_one_cell` | `fn try_one_cell(...) -> Result<SimCore, ...>`（重複 id 拒否） | REQ-SIM-06, REQ-DET-04c |
| `SimCore::step` | `fn step(&mut self, ticks: u32) -> Result<(), String>` | REQ-SIM-04, REQ-CON-05 |
| `SimCore::invariant_report` | `fn invariant_report(&self) -> InvariantReport` | REQ-SIM-06, REQ-OPS-01 |
| `SimCore::state_hash` | `fn state_hash(&self) -> StateHash` | REQ-DET-05 |

## 12. FFI（C ABI・7 操作・バッファ規約）

FFI は 7 操作（create / load / step / snapshot / explain / save / destroy）に限定する。確定。参照: REQ-CON-01

### 12.1 操作一覧（BD-01 r2 §3 と一致。これ以外は通らない）

| 操作 | 入力 | 出力 | バッファ | PRNG 消費 | hash への影響 |
|---|---|---|---|---|---|
| `create(config_json, seed)` | config UTF-8、u64 | handle または err | — | 初期化（SplitMix64）のみ | 初期状態を定める |
| `load(save_bytes)` | WorldSave | handle または err（checksum / schema_version / model_version 不一致） | 呼出側バッファ | 0（状態を復元） | 復元した状態 |
| `step(handle, n)` | u32 | tick 後の終了ラベル（Option）または err | — | 各 phase の割当分のみ（BD-07 §3） | 唯一 hash を進める操作 |
| `snapshot(handle, out)` | — | 固定レイアウト（§12.4） | 固定長・呼出側確保 | 0 | なし |
| `explain(handle, query, out)` | query JSON（event_id 等） | JSON | 可変長: 呼出側バッファ、不足時 `required_len` | 0 | なし（純関数、台帳は読み取りビュー） |
| `save(handle, out)` | — | WorldSave バイト列 | 可変長: 同上 | 0 | なし |
| `destroy(handle)` | — | ok | — | 0 | — |

- 「Presentation → core の入力」は上表の入力列が全て。第 8 操作・未知フィールドは FFI が拒否する。確定。参照: REQ-CON-01, REQ-OUT-05
- ハンドルは opaque 型 `KzSim` へのポインタ `*mut KzSim`。所有権は呼出側が `kz_destroy` で解放する。確定。参照: REQ-CON-01

### 12.2 エラー enum と再入禁止

- エラーコードは enum のみ（数値・文字列の自由形式は返さない）。確定。参照: REQ-CON-08, REQ-SIM-13

```c
typedef enum KzError {
  KZ_OK = 0,
  KZ_ERR_SCHEMA = 1,         // schema_version 不一致・config/save が schema 不適合
  KZ_ERR_MODEL_VERSION = 2,  // model_version 不一致
  KZ_ERR_CHECKSUM = 3,       // checksum 不一致
  KZ_ERR_BUFFER = 4,         // 容量不足。out_required_len に必要バイト数を返す
  KZ_ERR_BUSY = 5,           // 再入（操作中の同一 handle 呼出）
  KZ_ERR_NUMERIC = 6,        // NumericError（負値・i64/i128 範囲外）
} KzError;
```

- 再入禁止: handle は同時 1 操作、全操作は同一スレッド。操作中の呼出は `KZ_ERR_BUSY`。Presentation のスケジューラは説明表示中・save 中に step を発行しない。確定。参照: REQ-CON-01, REQ-UI-03
- 値の追加は semver minor、既存値の変更・削除は major。参照: REQ-NFR-06
- 可変長出力は呼出側がバッファと容量を渡す。容量不足時は `KZ_ERR_BUFFER` を返し、`out_required_len` に必要バイト数を書き込む。確定。参照: REQ-CON-08

### 12.3 C ABI シグネチャ

```c
// create: config JSON（UTF-8）と seed から run を生成（Prepared）。参照: REQ-CON-01, REQ-SCOPE-01
int32_t kz_create(const uint8_t* config_json, uintptr_t config_len, uint64_t seed, KzSim** out_handle);

// load: WorldSave バイト列から run を復元。checksum・schema_version・model_version を検証し、
// 不一致は KZ_ERR_CHECKSUM / KZ_ERR_SCHEMA / KZ_ERR_MODEL_VERSION。参照: REQ-DET-06, REQ-CON-08
int32_t kz_load(const uint8_t* save_bytes, uintptr_t save_len, KzSim** out_handle);

// step: n tick 進める。終了していれば out_label にラベルを返す（無ければ *out_has_label = 0）。
// Terminated 済みなら状態を変えず既存ラベルを返す（冪等）。参照: REQ-SIM-04, REQ-CON-05, REQ-END-01
int32_t kz_step(KzSim* handle, uint32_t n, uint8_t* out_has_label, uint8_t* out_label);

// snapshot: 固定レイアウト（§12.4）へコピーアウト。cap 不足は KZ_ERR_BUFFER。状態不変。参照: REQ-VIS-04
int32_t kz_snapshot(const KzSim* handle, uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// explain: query JSON を受け取り説明 JSON を返す。純関数・状態不変。参照: REQ-EXP-01, REQ-EXP-03
int32_t kz_explain(const KzSim* handle, const uint8_t* query_json, uintptr_t query_len,
                   uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// save: WorldSave をバッファへ。状態不変。参照: REQ-DET-06
int32_t kz_save(const KzSim* handle, uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// destroy: ハンドルを解放。解放後に呼出側が触れないこと（use-after-destroy は未定義）。参照: REQ-CON-01
void kz_destroy(KzSim* handle);
```

### 12.4 snapshot 固定レイアウト

- ヘッダ（16 バイト、リトルエンディアン固定。BD-07 §6 ADR 候補 1）: `width: u16, height: u16, n_lineages: u8, tick: u32`、残余 7 バイトは 0
- セルレコード（row-major 固定順、96 バイト）: `nutrient: i64, biomass: [i64; 8], carcass: i64, waste: i64, occupancy_peak: i64`。energy は描画に不要なため含めない
- サイズは handle ごとに固定（64×64 なら 16 + 4,096 × 96 = 393,232 バイト）。描画間引きはリプレイ経路に載らない。確定。参照: REQ-VIS-04, REQ-VIS-01

### 12.5 事前 / 事後条件

| 操作 | 事前条件 | 事後条件 | 参照 |
|---|---|---|---|
| kz_create | config_json は config.schema.json 適合の UTF-8 JSON。out_handle は非 null | KZ_OK なら out_handle に Prepared の run（tick = 0）。エラー時は out_handle 不変 | REQ-SCOPE-01, REQ-CON-01 |
| kz_load | save_bytes は save.schema.json 適合 | KZ_OK なら保存時の tick・状態・PRNG・カウンタ類（fixed_streak / tick0_ranking / inflow_cursor）を持つ run。不一致は KZ_ERR_CHECKSUM / KZ_ERR_SCHEMA / KZ_ERR_MODEL_VERSION で run を作らない | REQ-DET-06, REQ-CON-08 |
| kz_step | handle は Prepared / Running / Terminated。n ≥ 1。同一 handle の操作中でない | KZ_OK なら tick が n 増加（Terminated では不変・既存ラベルを返す）。終了条件成立時は out_label にラベル。KZ_ERR_NUMERIC 時は状態不変 | REQ-SIM-04, REQ-SIM-13, REQ-END-01 |
| kz_snapshot | handle は Destroyed 以外・操作中でない | 状態・PRNG・hash 不変。cap 不足時は KZ_ERR_BUFFER と out_required_len | REQ-VIS-04, REQ-CON-08 |
| kz_explain | handle は Destroyed 以外・操作中でない。query は JSON | 状態不変。出力は 4 段構造（事実→解釈→不明→次の一手）の JSON | REQ-EXP-01, REQ-EXP-05 |
| kz_save | handle は Destroyed 以外・操作中でない | 状態不変。出力は save.schema.json 適合の WorldSave | REQ-DET-06 |
| kz_destroy | handle は非 null・未 destroy・操作中でない | メモリ解放。handle は再利用不可 | REQ-CON-01 |

## 13. schema

3 schema を `docs/30_contracts/schema/` に置く。`additionalProperties: false` を全オブジェクトに付す。確定。参照: REQ-GEN-01, REQ-ENV-01, REQ-DET-06

| schema | ファイル | 内容 | 参照 |
|---|---|---|---|
| config | `config.schema.json` | schema_version / model_version / seed / grid / lineages / inflow。lineages[].id は 0..=7 の整数 | REQ-SCOPE-01, REQ-GEN-01, REQ-SIM-07 |
| result | `result.schema.json` | schema_version / model_version / config_hash / seed / ticks / termination_label / state_hash / events / invariants | REQ-END-01, REQ-OPS-01 |
| save | `save.schema.json` | schema_version / model_version / config_hash / seed / prng_state / state_hash / state（state は WorldSave: tick・全セル・fixed_streak・tick0_ranking・inflow_cursor を含む。BD-01 r2 §5） | REQ-DET-06 |

## 14. semver 規則

- `schema_version`: 3 schema それぞれに付す。フィールド追加は minor、削除・型変更は major。major 変更は migration と旧 save 読込テストを伴う。参照: REQ-NFR-06
- `model_version`: シミュレーションの振る舞い（係数・丸め・PRNG・hash・phase 規則）を変えたら bump。scale・rounding・prng・hash の文字列を含む。model_version が変わったらチュートリアル seed の再選定を必須ゲートとする。参照: REQ-DET-05, REQ-DET-09, REQ-NFR-06
- 互換を壊す変更は RFC 経由で、migration とその試験を同じ PR に含める。参照: REQ-NFR-06
- golden（`docs/30_contracts/golden/`）の更新は Claude のみが承認できる。参照: REQ-OPS-01

## 15. 移行の扱い（simulation_contract.md の吸収）

- 本章が契約の正本となり、`docs/30_contracts/simulation_contract.md` v0.1 は凍結（変更禁止・参照専用）とする。本章と v0.1 の間に差分は無い（§1〜§10 を §1〜§10 としてそのまま移し、確定度と REQ 参照を付記したのみ）
- 契約の変更は本章に対して RFC で行い、版を上げる。参照: REQ-NFR-06
