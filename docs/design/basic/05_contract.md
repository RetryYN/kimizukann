# BD-05 契約（公開 API・FFI・schema・事前事後条件・semver）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/要件定義書_検証版_v0.2.md`（sign-off 済）、`docs/contracts/simulation_contract.md` v0.1
- 本章は `simulation_contract.md` v0.1 を吸収・再編した**契約の正本**である。`simulation_contract.md` は v0.1 として凍結し、以後の変更は本章を RFC 経由で改訂する（移行の扱いは §8）
- 完成条件: `cargo public-api` の出力と一致。各 pub 項目に REQ 参照
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。ドメイン集約・不変条件は BD-03、状態機械は BD-04、ビット幅の証明は BD-06、乱数消費回数は BD-07 を参照

## 1. 状態（契約 §1 吸収）

1 セルは `nutrient: Fixed`、`biomass[L]: Fixed`、`carcass: Fixed`、`waste: Fixed`、`energy[L]: Fixed`、`occupancy_peak: Fixed` の 6 状態のみを持ち、全体状態は `tick: u32` を持つ。`L ≤ 8`。物質の単位は `mass_u`（固定小数点の最小単位）、エネルギーは無次元の系統別予算。参照: REQ-SIM-01, REQ-SIM-08

## 2. tick 順序（契約 §2 吸収）

各 tick は固定順 7 phase を一度だけ実行する: diffuse → intake → maintenance → starvation_and_death → reproduction → emission → occupancy。走査順はセル row-major、系統 ID 昇順、近傍は北・東・南・西の固定順。順序の入替は契約違反。確定。参照: REQ-SIM-04, REQ-DET-04c

## 3. 物質・エネルギー二重台帳（契約 §3 吸収）

物質台帳は `nutrient + Σbiomass + carcass + waste` を追跡する。流入は `Vec<InflowEvent { tick, pool, amount }>`（閉鎖系は空 Vec）。エネルギー台帳は系統ごとに摂取加算・維持/移動/繁殖への配分・熱散逸を追跡し、熱散逸は物質を減らさない。全変換は `LedgerEntry { from_pool, to_pool, amount, reason }` を通し、負値と未記録残差を禁止する。確定。参照: REQ-SIM-05, REQ-SIM-07, REQ-SCOPE-04

## 4. 質量係数表（契約 §4 吸収）

係数は入力質量に対する出力質量の割合で、合計 1.0（固定小数点 1_000_000）。余りは常に主出力プールへ戻し、拡散の余りは送り元セルに残す。捨てる経路は存在しない。参照: REQ-SIM-05

| 変換 | 出力 | 係数 | 余り戻し先 | 確定度 |
|---|---|---:|---|---|
| 摂取 | biomass | 0.70 | biomass（主出力） | 初期仮説（D3 で確定） |
| 摂取 | waste | 0.30 | biomass（主出力） | 初期仮説（D3 で確定） |
| biomass 維持不足 | carcass | 1.00 | carcass（主出力） | 確定 |
| biomass 死亡 | carcass | 1.00 | carcass（主出力） | 確定 |

係数は config の hash に含める。エネルギー係数は物質係数と別の無次元値。参照: REQ-SIM-05, REQ-DET-05

## 5. 固定小数点・丸め規則（契約 §5 吸収）

コア状態と係数は i64 固定小数点 scale = 1_000_000、乗算中間は i128、除算はゼロ方向丸め 1 種。変換余りは主出力プールへ、拡散余りは送り元セルへ。負値・i64 範囲外・i128 中間範囲外は `NumericError`。解析用 z-score のみ浮動小数点を許すがコア状態へ戻さない。丸めモードと scale は model_version の一部。確定。上限とビット幅の証明は BD-06。参照: REQ-CON-02, REQ-SIM-13

## 6. PRNG・用途別ストリーム（契約 §6 吸収）

`SplitMix64` で seed から 4 ストリーム（movement / reproduction / mutation / interaction）の初期状態を導出し、各ストリームは xoshiro256**。バージョン文字列 `prng=xoshiro256ss-v1`。表示用サンプリングはコア乱数を消費しない。seed 内は単一スレッド、seed 間バッチのみ並列化。確定。phase 割当と消費回数表は BD-07。参照: REQ-DET-04a, REQ-DET-04b

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

同一 model_version・config・seed で `step(2000)` ≡ `step(1) × 2000` ≡ 任意 tick で save → load → 残り、が同一の正規化 state hash と終了ラベルを返す。hash は SHA-256（`hash=sha256-v1`）。正規化はセル row-major・系統昇順・i64 バイト列・PRNG 4 ストリーム状態・model_version を含み、描画用トークン・UI・ログ時刻を除く。速度変更・描画間引き・トークン有無・seed 間並列数は hash を変えない。確定。検証手順は BD-07 §5。参照: REQ-DET-02, REQ-DET-05, REQ-OUT-04, REQ-DET-07

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
| `WorldState { tick: u32, grid, lineages }` | struct | REQ-SIM-01 |
| `Seed(pub u64)` | struct | REQ-DET-04a |
| `PrngState { seed, movement, reproduction, mutation, interaction }` | struct | REQ-DET-04a, REQ-DET-05 |
| `StateHash(pub [u8; 32])` | struct | REQ-DET-05 |
| `Thresholds { base_intake, base_maintenance, epsilon, fixed_share, fixed_ticks, coexist_share, max_ticks, waste_toxic_threshold, toxin_maintenance_multiplier, occupancy_threshold, vacant_nutrient_threshold }` | struct | REQ-END-02, REQ-END-03, REQ-END-04a, REQ-SIM-02, REQ-SIM-03 |
| `SaveEnvelope { schema_version, model_version, config_hash, seed, prng, state_hash, state }` | struct | REQ-DET-06 |
| `MassCoefficients { intake_to_biomass, intake_to_waste, starvation_to_carcass, death_to_carcass }` | struct | REQ-SIM-05 |
| `LedgerEntry { from_pool, to_pool, amount, reason }` | struct | REQ-SIM-05 |
| `MassLedger { entries }` / `EnergyLedger { entries }` | struct | REQ-SIM-05, REQ-SCOPE-04 |
| `InvariantReport { mass_ok, energy_ok, non_negative, message }` | struct | REQ-SIM-06, REQ-OPS-01 |
| `RoundingMode { TowardZero }` | enum | REQ-CON-02 |
| `NumericError { Negative, OverflowI64, OverflowI128 }` | enum | REQ-SIM-13 |
| `VerifySuite` | struct | REQ-OPS-01 |
| `ModelVersion { major, minor, scale, rounding, prng, hash }` | struct | REQ-DET-05, REQ-NFR-06 |
| `ScanOrder` | struct | REQ-DET-04c |
| `RandomStream` | struct | REQ-DET-04a |
| `StreamKind { Movement, Reproduction, Mutation, Interaction }` | enum | REQ-DET-04a |
| `Substrate { Nutrient, Carcass, Waste }` | enum | REQ-SIM-02, REQ-SCOPE-05 |
| `InflowEvent { tick, pool, amount }` | struct | REQ-SIM-07 |
| `ConversionRule { from, to, coefficient, remainder_to }` | struct | REQ-SIM-05 |
| `TerminationTiming { EveryTick, AtTimeLimit }` | enum | REQ-END-04c |
| `TerminationRule { label, timing, priority }` | struct | REQ-END-01, REQ-END-04c |
| `StateSnapshot { state, prng }` | struct | REQ-DET-02, REQ-VIS-04 |

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

### 12.1 共通規約

- ハンドルは opaque 型 `KzSim` へのポインタ `*mut KzSim`。所有権は呼出側が `kz_destroy` で解放する。参照: REQ-CON-01
- エラーは `KzError`（i32）で返す: `KZ_OK=0 / KZ_ERR_INVALID_ARG=1 / KZ_ERR_NUMERIC=2 / KZ_ERR_CHECKSUM=3 / KZ_ERR_SCHEMA_VERSION=4 / KZ_ERR_MODEL_VERSION=5 / KZ_ERR_BUFFER_TOO_SMALL=6 / KZ_ERR_TERMINATED=7 / KZ_ERR_DESTROYED=8`。確定（値の追加は semver minor、既存値の変更は major）。参照: REQ-SIM-13, REQ-DET-06
- 可変長出力は呼出側がバッファと容量を渡す。容量不足時は `KZ_ERR_BUFFER_TOO_SMALL` を返し、`out_required_len` に必要バイト数を書き込む。参照: REQ-CON-08
- 全関数はスレッドセーフでなくてよい（1 run = 1 スレッド。seed 間並列は呼出側がハンドルを分ける）。参照: REQ-DET-04a

### 12.2 操作一覧（C ABI シグネチャ）

```c
// create: config JSON（UTF-8）から run を生成。参照: REQ-CON-01, REQ-SCOPE-01
int32_t kz_create(const uint8_t* config_json, uintptr_t config_len, KzSim** out_handle);

// load: SaveEnvelope バイト列から run を復元。checksum・schema_version・model_version を検証し
// 不一致は KZ_ERR_CHECKSUM / KZ_ERR_SCHEMA_VERSION / KZ_ERR_MODEL_VERSION。参照: REQ-DET-06, REQ-CON-08
int32_t kz_load(const uint8_t* save_bytes, uintptr_t save_len, KzSim** out_handle);

// step: n tick 進める。終了済みなら KZ_ERR_TERMINATED。参照: REQ-SIM-04, REQ-CON-05
int32_t kz_step(KzSim* handle, uint32_t n);

// snapshot: 描画用スナップショットを固定レイアウトバッファへコピーアウト。状態を変えない。参照: REQ-VIS-04
int32_t kz_snapshot(const KzSim* handle, uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// explain: 説明器出力（JSON）をバッファへ。状態を変えない。参照: REQ-EXP-01, REQ-EXP-03
int32_t kz_explain(const KzSim* handle, uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// save: SaveEnvelope をバッファへ。状態を変えない。参照: REQ-DET-06
int32_t kz_save(const KzSim* handle, uint8_t* buf, uintptr_t cap, uintptr_t* out_required_len);

// destroy: ハンドルを解放。以後の使用は KZ_ERR_DESTROYED 相当の未定義動作とし、呼出側が触れない。参照: REQ-CON-01
void kz_destroy(KzSim* handle);
```

### 12.3 事前 / 事後条件

| 操作 | 事前条件 | 事後条件 | 参照 |
|---|---|---|---|
| kz_create | config_json は config.schema.json 適合の UTF-8 JSON。out_handle は非 null | KZ_OK なら out_handle に Prepared の run（tick = 0）。エラー時は out_handle 不変 | REQ-SCOPE-01, REQ-CON-01 |
| kz_load | save_bytes は save.schema.json 適合 | KZ_OK なら保存時の tick・状態・PRNG を持つ run。不一致は対応エラーで run を作らない | REQ-DET-06, REQ-CON-08 |
| kz_step | handle は Prepared / Running。n ≥ 1 | KZ_OK なら tick が n 増加。終了条件成立時は Terminated(label)。NumericError 時は状態不変 | REQ-SIM-04, REQ-SIM-13, REQ-END-01 |
| kz_snapshot | handle は Destroyed 以外 | 状態・PRNG・hash 不変。cap 不足時は out_required_len に必要量 | REQ-VIS-04, REQ-CON-08 |
| kz_explain | handle は Destroyed 以外 | 状態不変。出力は 4 段構造（事実→解釈→不明→次の一手）の JSON | REQ-EXP-01, REQ-EXP-05 |
| kz_save | handle は Destroyed 以外 | 状態不変。出力は save.schema.json 適合 | REQ-DET-06 |
| kz_destroy | handle は非 null・未 destroy | メモリ解放。handle は再利用不可 | REQ-CON-01 |

## 13. schema

3 schema を `docs/contracts/schema/` に置く。`additionalProperties: false` を全オブジェクトに付す。確定。参照: REQ-GEN-01, REQ-ENV-01, REQ-DET-06

| schema | ファイル | 内容 | 参照 |
|---|---|---|---|
| config | `config.schema.json` | schema_version / model_version / seed / grid / lineages / inflow。lineages[].id は 0..=7 の整数 | REQ-SCOPE-01, REQ-GEN-01, REQ-SIM-07 |
| result | `result.schema.json` | schema_version / model_version / config_hash / seed / ticks / termination_label / state_hash / events / invariants | REQ-END-01, REQ-OPS-01 |
| save | `save.schema.json` | schema_version / model_version / config_hash / seed / prng_state / state_hash / state | REQ-DET-06 |

## 14. semver 規則

- `schema_version`: 3 schema それぞれに付す。フィールド追加は minor、削除・型変更は major。major 変更は migration と旧 save 読込テストを伴う。参照: REQ-NFR-06
- `model_version`: シミュレーションの振る舞い（係数・丸め・PRNG・hash・phase 規則）を変えたら bump。scale・rounding・prng・hash の文字列を含む。model_version が変わったらチュートリアル seed の再選定を必須ゲートとする。参照: REQ-DET-05, REQ-DET-09, REQ-NFR-06
- 互換を壊す変更は RFC 経由で、migration とその試験を同じ PR に含める。参照: REQ-NFR-06
- golden（`docs/contracts/golden/`）の更新は Claude のみが承認できる。参照: REQ-OPS-01

## 15. 移行の扱い（simulation_contract.md の吸収）

- 本章が契約の正本となり、`docs/contracts/simulation_contract.md` v0.1 は凍結（変更禁止・参照専用）とする。本章と v0.1 の間に差分は無い（§1〜§10 を §1〜§10 としてそのまま移し、確定度と REQ 参照を付記したのみ）
- 契約の変更は本章に対して RFC で行い、版を上げる。参照: REQ-NFR-06
