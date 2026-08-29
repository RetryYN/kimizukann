# 基本設計書 BD-02: 用語集（ユビキタス言語）

- 版: 0.1（起草 cursor-gemini、2026-08-30）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`、`docs/30_contracts/simulation_contract.md`、`crates/sim-types/src/lib.rs`、`docs/00_product/第2回_統合案_v0.5.md`
- 目的: 要件定義書・契約・Rust コア実装・Flutter UI・説明器の間で用語・識別子・型・単位を厳密に一致させ、表記揺れや意味の曖昧さを排除する。

---

## 1. 物質プール・エネルギー・空間状態

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 栄養 | `Nutrient` / `nutrient` | 環境中に存在し、系統が摂取して生体量やエネルギーへ変換する基礎資源プール | `Pool::Nutrient` / `Fixed` | mass_u (0 ≤ x ≤ i64::MAX) | REQ-SIM-01, REQ-SIM-06 |
| 生体量 | `Biomass` / `biomass` | 系統を構成する物質質量。セルごとに最大8系統分保持される | `Pool::Biomass` / `[Fixed; 8]` | mass_u (0 ≤ x ≤ i64::MAX) | REQ-SIM-01, REQ-SIM-06 |
| 死骸 | `Carcass` / `carcass` | 系統の飢餓・死亡によって生体量から転換された未分解有機物プール | `Pool::Carcass` / `Fixed` | mass_u (0 ≤ x ≤ i64::MAX) | REQ-SIM-01, REQ-SCOPE-05 |
| 老廃物 | `Waste` / `waste` | 物質変換や代謝によって生じる排出物プール。蓄積濃度が毒性判定に用いられる | `Pool::Waste` / `Fixed` | mass_u (0 ≤ x ≤ i64::MAX) | REQ-SIM-01, REQ-SIM-02 |
| エネルギー | `energy` | 各系統が1tick内での活動（維持・移動・繁殖）に使用する無次元の系統別予算 | `[Fixed; 8]` | 無次元 (0 ≤ x ≤ 1_000_000、即ち 0.0〜1.0) | REQ-SIM-01, REQ-SIM-08 |
| 最大占有度（占有ピーク） | `occupancy_peak` | セルの過去の繁栄履歴を記録する減衰スカラー。生体量合計が閾値以上で1.0飽和、毎tick 0.995減衰 | `Fixed` | 無次元減衰値 (0 ≤ x ≤ 1_000_000、即ち 0.0〜1.0) | REQ-SIM-01, REQ-SIM-03a |
| セル状態 | `CellState` | 格子上の1セルが保持する全状態（栄養、系統別生体量、死骸、老廃物、系統別エネルギー、占有ピーク） | `struct CellState` | 複合構造体 | REQ-SIM-01 |
| 格子状態 | `GridState` | 2次元格子空間全体のセル配列と寸法（幅・高さ）を保持する構造 | `struct GridState` | 64×64（契約上限 65535²） | REQ-SIM-01 |
| 世界状態 | `WorldState` | シミュレーションの現在tick、格子状態、系統パラメータ一覧を含む全体スナップショット | `struct WorldState` | 複合構造体 | REQ-SIM-01 |
| 空き家 | `vacant_home` / `is_vacant` | 過去に繁栄したが現在は絶滅し、栄養が残っている未利用セル（表示・説明器用判定） | `bool` | 条件: `occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ` | REQ-SIM-03b |

---

## 2. 系統・適応ベクトル・機構タグ・系統定数

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 系統 | `lineage` / `LineageParams` | シミュレーション内で活動する個体群種族の定義パラメータ群（最大8系統） | `struct LineageParams` | 複合構造体 | REQ-GEN-01 |
| 系統識別子 | `lineage_id` / `id` | 各系統を一意に識別するインデックス番号 | `u8` | 0..=7 | REQ-GEN-01 |
| 適応ベクトル | `trait_vector` / `TraitVector` | プレイヤーが札として選択する5軸の形質倍率パラメータ群 | `struct TraitVector` | 複合構造体 | REQ-GEN-01, REQ-GEN-02 |
| 移動 | `movement` | 系統の生体量が近傍セルへ拡散・移動する度合いの倍率 | `Fixed` | 倍率 (scale=1e6, 基準 1.0 = 1_000_000) | REQ-GEN-01, REQ-GEN-03 |
| 摂取 | `intake` | 系統が基質（栄養・死骸・老廃物）を取り込む能力の倍率 | `Fixed` | 倍率 (scale=1e6, 基準 1.0 = 1_000_000) | REQ-GEN-01, REQ-GEN-03 |
| 変換 | `conversion` | 摂取した物質を生体量やエネルギーへ転換する効率倍率 | `Fixed` | 倍率 (scale=1e6, 基準 1.0 = 1_000_000) | REQ-GEN-01, REQ-GEN-03 |
| 維持コスト | `maintenance_cost` | 系統が生体量を維持するために毎tick消費する基礎エネルギーコストの倍率（小さいほど有利） | `Fixed` | 倍率 (scale=1e6, 基準 1.0 = 1_000_000) | REQ-GEN-01, REQ-GEN-03 |
| 繁殖 | `reproduction` | 余剰エネルギーを生体量増加へ割り当てる効率倍率 | `Fixed` | 倍率 (scale=1e6, 基準 1.0 = 1_000_000) | REQ-GEN-01, REQ-GEN-03 |
| 機構タグ | `mechanism_tags` / `MechanismTags` | 系統の代謝経路や環境感受性を表す5つの真偽値フラグ | `struct MechanismTags` | 5-bit ブール構造体 | REQ-GEN-01 |
| 栄養利用タグ | `use_nutrient` | 栄養プールを取り込み基質として利用できるかを示すフラグ | `bool` | true / false | REQ-GEN-01 |
| 死骸利用タグ | `use_carcass` | 死骸プールを取り込み基質として利用できるかを示すフラグ | `bool` | true / false | REQ-GEN-01, REQ-SCOPE-05 |
| 老廃物利用タグ | `use_waste` | 老廃物プールを取り込み基質として利用できるかを示すフラグ | `bool` | true / false | REQ-GEN-01 |
| 毒性感受性タグ | `toxin_sensitive` | 老廃物濃度が毒性閾値を超えた場合に維持コスト増大ペナルティを受けるフラグ | `bool` | true / false | REQ-GEN-01, REQ-SIM-02 |
| 密度効果タグ | `density_bonus` | 同一セル内の高密度集積によって有利な補正を受けるフラグ | `bool` | true / false | REQ-GEN-01 |
| 死亡閾値 | `mortality_threshold` | セル内生体量がこの値を下回ると即座に全量死骸化する系統定数 | `Fixed` | mass_u (scale=1e6) | REQ-GEN-01, REQ-SIM-09 |
| 老廃物排出量 | `waste_emission` | 代謝活動に伴い固定的に発生する老廃物量の系統定数 | `Fixed` | mass_u (scale=1e6) | REQ-GEN-01 |
| 基質 | `Substrate` | 系統が摂取対象として消費可能な物質プール種別（栄養・死骸・老廃物） | `enum Substrate` | Nutrient / Carcass / Waste | REQ-GEN-01 |

---

## 3. 系統プリセット（4系統）

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| アオシキ | `aoshiki` | 安定型系統。高摂取・標準維持のバランス型（移動0.70 / 摂取1.05 / 変換0.95 / 維持1.00 / 繁殖0.85） | `LineageParams` (preset 0) | 固定パラメータ | REQ-SCOPE-02, REQ-GEN-03 |
| シロナミ | `shironami` | 高移動・分散型系統。移動力に優れ外周展開を得意とする（移動1.20 / 摂取0.80 / 変換0.90 / 維持1.10 / 繁殖0.85） | `LineageParams` (preset 1) | 固定パラメータ | REQ-SCOPE-02, REQ-GEN-03 |
| アカバエ | `akabae` | 突出型系統。高摂取・超高繁殖だが高維持コストで崩壊リスクを持つ（移動1.00 / 摂取1.15 / 変換0.85 / 維持1.30 / 繁殖1.60） | `LineageParams` (preset 2) | 固定パラメータ | REQ-SCOPE-02, REQ-GEN-03 |
| クロシデ | `kuroshide` | 死骸利用型（意図的劣位型）。死骸を利用し省維持で粘る（移動0.80 / 摂取0.45 / 変換0.95 / 維持0.90 / 繁殖0.65、`use_carcass=true`） | `LineageParams` (preset 3) | 固定パラメータ | REQ-SCOPE-02, REQ-GEN-03, REQ-SCOPE-05 |

---

## 4. シミュレーション実行・フェーズ・台帳・数値規則

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 固定小数点数 | `Fixed` | コア計算で確定性と保存則を保証するための64ビット符号付き固定小数点数型 | `type Fixed = i64` | scale = 1_000_000 (10進6桁) | REQ-CON-02 |
| 固定スケール | `FIXED_SCALE` | 固定小数点の基数倍率定数（1.0 = 1_000_000） | `pub const FIXED_SCALE: i64` | 1_000_000 | REQ-CON-02 |
| 更新フェーズ | `TickPhase` | 1tick内で厳密に定められた順序で実行される7つの状態更新段階 | `enum TickPhase` | 7値列挙型 | REQ-SIM-04 |
| 拡散フェーズ | `Diffuse` / `diffuse` | 栄養・死骸・老廃物を4近傍セルへ分配する第1フェーズ | `TickPhase::Diffuse` | フェーズ 1/7 | REQ-SIM-04, REQ-SIM-10 |
| 摂取フェーズ | `Intake` / `intake` | 系統が利用可能プールを摂取し生体量とエネルギーへ変換する第2フェーズ | `TickPhase::Intake` | フェーズ 2/7 | REQ-SIM-04, REQ-SIM-11 |
| 維持課金フェーズ | `Maintenance` / `maintenance` | 各系統が生体量に応じた維持コストをエネルギー予算から支払う第3フェーズ | `TickPhase::Maintenance` | フェーズ 3/7 | REQ-SIM-04, REQ-SIM-08 |
| 飢餓・死亡フェーズ | `StarvationAndDeath` / `starvation_and_death` | エネルギー不足分を生体量から死骸へ移し、閾値未満系統を全滅死骸化する第4フェーズ | `TickPhase::StarvationAndDeath` | フェーズ 4/7 | REQ-SIM-04, REQ-SIM-08, REQ-SIM-09 |
| 繁殖フェーズ | `Reproduction` / `reproduction` | 余剰エネルギーを用いて生体量を増殖させる第5フェーズ | `TickPhase::Reproduction` | フェーズ 5/7 | REQ-SIM-04, REQ-SIM-12 |
| 排出フェーズ | `Emission` / `emission` | 代謝残差や排出物を老廃物プールへ格納する第6フェーズ | `TickPhase::Emission` | フェーズ 6/7 | REQ-SIM-04, REQ-SIM-05 |
| 占有度更新フェーズ | `Occupancy` / `occupancy` | セルの生体量合計に基づいて `occupancy_peak` を更新・減衰する第7フェーズ | `TickPhase::Occupancy` | フェーズ 7/7 | REQ-SIM-04, REQ-SIM-03a |
| 台帳理由コード | `ReasonCode` | 物質・エネルギー台帳エントリに記録される変換理由識別子 | `enum ReasonCode` | 7値列挙型 | REQ-SCOPE-04 |
| 摂取理由 | `Intake` | 摂取による物質移動・エネルギー獲得を示す理由コード | `ReasonCode::Intake` | 理由コード | REQ-SCOPE-04 |
| 維持理由 | `Maintenance` | 生命維持コストの消費を示す理由コード | `ReasonCode::Maintenance` | 理由コード | REQ-SCOPE-04 |
| 飢餓理由 | `Starvation` | エネルギー不足による生体量の死骸転換を示す理由コード | `ReasonCode::Starvation` | 理由コード | REQ-SCOPE-04 |
| 死亡理由 | `Death` | 死亡閾値未満による全量死骸化を示す理由コード | `ReasonCode::Death` | 理由コード | REQ-SCOPE-04, REQ-SIM-09 |
| 繁殖理由 | `Reproduction` | エネルギー余剰からの生体量増殖を示す理由コード | `ReasonCode::Reproduction` | 理由コード | REQ-SCOPE-04 |
| 排出理由 | `Emission` | 代謝に伴う老廃物発生を示す理由コード | `ReasonCode::Emission` | 理由コード | REQ-SCOPE-04 |
| 拡散理由 | `Diffusion` | 空間的な近傍拡散移動を示す理由コード | `ReasonCode::Diffusion` | 理由コード | REQ-SCOPE-04 |
| 物質台帳 | `MassLedger` | 1tick内の全プール間物質移動・変換を追跡し保存則を保証する台帳 | `struct MassLedger` | エントリ配列 | REQ-SIM-06 |
| エネルギー台帳 | `EnergyLedger` | 各系統のエネルギー獲得・消費・散逸を追跡する台帳 | `struct EnergyLedger` | エントリ配列 | REQ-SIM-08 |
| 台帳エントリ | `LedgerEntry` | 移動元プール、移動先プール、移動量、理由コードを保持する単一記録単位 | `struct LedgerEntry` | 複合構造体 | REQ-SIM-05 |
| 質量係数 | `MassCoefficients` | 摂取・飢餓・死亡時の物質分配比率（合計 1.0 = 1_000_000） | `struct MassCoefficients` | 固定小数点係数群 | REQ-SIM-05 |
| 変換規則 | `ConversionRule` | 変換元プール、主出力先、係数、余り戻し先プールを定義する規則 | `struct ConversionRule` | 複合構造体 | REQ-SIM-05 |
| 流入イベント | `InflowEvent` | 指定tickに指定プールへ外部から投入される物質量イベント | `struct InflowEvent` | `tick: u32, pool: Pool, amount: Fixed` | REQ-SIM-07 |
| 不変条件レポート | `InvariantReport` | 物質保存則、エネルギー非負、非負プール制約の検証成否を格納するレポート | `struct InvariantReport` | ブール判定＋メッセージ | REQ-SIM-06, REQ-SIM-13 |
| 丸めモード | `RoundingMode` | 固定小数点除算時の丸め方式（検証版はゼロ方向への丸め一択） | `enum RoundingMode` | TowardZero | REQ-CON-02 |
| 数値エラー | `NumericError` | 負値発生やi64/i128オーバーフロー時に即時安全停止するためのエラー型 | `enum NumericError` | Negative / OverflowI64 / OverflowI128 | REQ-SIM-13 |
| 走査順序 | `ScanOrder` | セル走査（Row-Major）、系統走査（ID昇順）、近傍走査（北・東・南・西）の確定規約 | `struct ScanOrder` | 確定性規約 | REQ-DET-04c |

---

## 5. 終了判定・閾値

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 終了ラベル | `TerminationLabel` | 培養シミュレーションの結末を表す5種類の公式ラベル | `enum TerminationLabel` | 5値列挙型 | REQ-END-01 |
| 全滅 | `Extinct` | 全系統の生体量合計が極小値 `ε` 未満となった結末（毎tick即時終了） | `TerminationLabel::Extinct` | ラベル 1/5 | REQ-END-02 |
| 固定 | `Fixed` | 1系統が総生体量の70%以上を200tick連続して維持した結末（毎tick即時終了） | `TerminationLabel::Fixed` | ラベル 2/5 | REQ-END-03 |
| 共存 | `Coexist` | 上限tick到達時に2系統以上が各15%以上の生体量シェアを維持している結末 | `TerminationLabel::Coexist` | ラベル 3/5 | REQ-END-04a |
| 逆転 | `Reversal` | 上限tick到達時の1位系統が、tick 0時点の順位で3位以下であった結末 | `TerminationLabel::Reversal` | ラベル 4/5 | REQ-END-04b |
| 時間制限（上限到達） | `TimeLimit` | 他の条件を満たさずに最大tick（2,000 tick）に到達した結末 | `TerminationLabel::TimeLimit` | ラベル 5/5 | REQ-END-01 |
| 終了規則 | `TerminationRule` | 各終了ラベルの判定タイミング（毎tick／上限時）と優先順位（全滅 > 固定 > 共存 > 逆転 > 上限） | `struct TerminationRule` | 複合構造体 | REQ-END-04c |
| 終了判定タイミング | `TerminationTiming` | 判定を実行する契機（毎tick判定 または 上限tick到達時判定） | `enum TerminationTiming` | EveryTick / AtTimeLimit | REQ-END-01, REQ-END-04a |
| 閾値群 | `Thresholds` | 判定や制御に使用される各種定数閾値の集約構造体 | `struct Thresholds` | 各種 Fixed / u32 | REQ-END-02, REQ-SIM-02, REQ-SIM-03a |

---

## 6. 環境・地形・札

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 環境レコード | `environment` / `EnvironmentRecord` | 培養皿の空間形状、初期資源分布、拡散係数、流入設定を統合した設定構造 | `struct EnvironmentRecord` / JSON schema | 設定オブジェクト | REQ-ENV-01 |
| 環境識別子 | `environment_id` | 4つの標準環境を一意に識別する文字列識別子 | `String` | center_rich / edge_sparse / local_waste / carcass_pulse | REQ-ENV-01, REQ-SCOPE-01 |
| 空間幾何識別子 | `geometry_id` | 空間のセル形状・島配置の構造識別子 | `String` | 文字列 | REQ-ENV-01, REQ-ENV-02 |
| 初期プール総量 | `initial_pool_totals` | シミュレーション開始時の4プールそれぞれの空間総量（全環境で統一） | `[Fixed; 4]` | mass_u | REQ-ENV-01, REQ-ENV-03 |
| プール分布マップ | `pool_distribution_maps` | 各セルへの初期物質配分を定めた2次元マップ配列 | `Vec<Vec<Fixed>>` | mass_u | REQ-ENV-01 |
| 拡散係数 | `diffusion_coefficients` | 4プールそれぞれの近傍セルへの1tick拡散率（初期仮説 0.05/近傍/tick） | `[Fixed; 4]` | 割合 (scale=1e6) | REQ-ENV-01, REQ-SIM-10 |
| 流入マスク（流入イベント列） | `inflow_tick_mask` | 各tickに注入される外部物質流入イベントの時系列リスト | `Vec<InflowEvent>` | イベント配列 | REQ-ENV-01, REQ-SIM-07 |
| 期待ニッチタグ | `expected_niche_tags` | その環境で有利となることが想定される機構タグのリスト | `Vec<String>` | 文字列配列 | REQ-ENV-01, REQ-GEN-06 |
| 中央の島 | `center_rich` | 中央部に高密度な栄養が集約された基本環境（地形札名: 中央の島） | `environment_id` | 4環境プリセット | REQ-ENV-02 |
| 縁の輪 | `edge_sparse` | 外周部に希薄な資源が環状に配置された環境（地形札名: 縁の輪） | `environment_id` | 4環境プリセット | REQ-ENV-02 |
| 二つの池 | `local_waste` | 局所的に老廃物が滞留・偏在しやすい二区画の環境（地形札名: 二つの池） | `environment_id` | 4環境プリセット | REQ-ENV-02 |
| 死骸の回廊 | `carcass_pulse` | 初期資源の一部が死骸として細い通路状に配置された環境（地形札名: 死骸の回廊） | `environment_id` | 4環境プリセット | REQ-ENV-02 |
| 札（適応方針札） | `trait_card` / `policy_card` | プレイヤーが各系統に割り当てる5軸適応方針ベクトルのカード選択肢 | オブジェクト | 5軸倍率ベクトル | REQ-GEN-02, REQ-SCOPE-03 |
| 地形札（培養皿札） | `geometry_card` / `dish_card` | プレイヤーが培養環境として選択する地形・資源レジームのカード選択肢 | オブジェクト | 4環境プリセット | REQ-SCOPE-01, REQ-ENV-02 |

---

## 7. 決定性・再現性・永続化

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 乱数シード | `seed` / `Seed` | 決定論的擬似乱数を初期化するための64ビット整数 | `struct Seed(pub u64)` | 0..=u64::MAX | REQ-DET-04a |
| 擬似乱数生成器状態 | `prng_state` / `PrngState` | SplitMix64から派生した用途別4ストリーム（xoshiro256**）の内部状態 | `struct PrngState` | 複合構造体 | REQ-DET-04a |
| 乱数ストリーム | `stream` / `RandomStream` | 用途別に分離された乱数系列ハンドル（表示サンプリング等の混入を防止） | `struct RandomStream` | ハンドル型 | REQ-DET-04a, REQ-DET-04b |
| 乱数ストリーム種別 | `StreamKind` | 4つの用途別乱数系列（移動、繁殖、変異、相互作用）の列挙 | `enum StreamKind` | Movement / Reproduction / Mutation / Interaction | REQ-DET-04b |
| 状態ハッシュ | `state_hash` / `StateHash` | 世界状態、PRNG状態、model_version等の正規化バイト列から算出したSHA-256値 | `struct StateHash(pub [u8; 32])` | 32バイト / 64桁16進文字列 | REQ-DET-05 |
| モデルバージョン | `model_version` / `ModelVersion` | 決定論的シミュレーション規則・丸め・PRNGアルゴリズムの互換性を表す識別文字列 | `struct ModelVersion` / `String` | 例: "xoshiro256ss-v1/sha256-v1" | REQ-DET-06, REQ-DET-09 |
| スキーマバージョン | `schema_version` | セーブデータや設定JSONの構造バージョンを表すセマンティックバージョニング文字列 | `String` | 例: "0.1.0" | REQ-DET-06, REQ-NFR-06 |
| コンフィグハッシュ | `config_hash` | 初期設定JSONから算出したSHA-256ハッシュ文字列 | `String` | 64桁16進文字列 | REQ-DET-06 |
| 三経路（三経路一致） | `three_execution_paths` | `step(2000)`、`step(1)×2000`、任意tickでのセーブ・ロード再開の3経路でstate_hashがビット一致する性質 | 決定性プロトコル | ビット完全一致 | REQ-DET-02 |
| 保存包 | `SaveEnvelope` | 中断セーブおよび再現実行に必要な状態・乱数・ハッシュ・設定を完全封入した永続化構造 | `struct SaveEnvelope` | JSON schema準拠 | REQ-DET-06, REQ-CON-08 |
| 状態スナップショット | `StateSnapshot` | 任意tickにおける世界状態とPRNG状態のメモリ上複製コピー | `struct StateSnapshot` | 複合構造体 | REQ-DET-02, REQ-VIS-04 |
| 検証スイート | `VerifySuite` | CIおよびCLI上で保存則・決定性・不変条件を網羅検証するテストスイート | `struct VerifySuite` | 検査コマンド・スイート | REQ-OPS-01 |

---

## 8. 転換点・スタンプ・説明器・UI・プレイヤー操作

| 語（日本語） | 識別子（英語、コードと同一） | 定義 | 型 | 単位・値域 | 出典 REQ |
|---|---|---|---|---|---|
| 転換点 | `turning_point` / `event` | 時系列z-scoreやドメイン事象から自動検出される生態系の重大変化（急増・急減・枯渇・逆転・絶滅等） | `TransitionEvent` / `event` | イベントオブジェクト | REQ-EVT-01, REQ-EVT-03 |
| スタンプ | `stamp` / `evidence_refs` | 転換点の発生時刻、スコア、影響領域ID（region_ids）、観測事実をまとめた証拠参照オブジェクト | `Stamp` / `evidence_refs` | スコア ≥ 1.2 | REQ-EVT-02, REQ-ACC-05 |
| 領域識別子 | `region_ids` | 二層（D3-Q1 判定 r2）: (A) 台帳 LedgerRecord の region_id = 静的 4×4 タイル（16×16 セル、row-major 0..=15、tick をまたぎ安定）。(B) スタンプの region_ids = イベント tick の占有マスク（Σbiomass>0）の 4 連結成分を row-major 初出順に採番（最大16、超過は15に併合）。説明器が派生計算し保存は stamp 内のみ | `Vec<u8>` / `u8` | 0..=15 | REQ-EVT-04, BD-01 r4 §5, BD-12 §2 |
| 理由コード（説明器） | `reason_code` / `contributor_reason` | 台帳やイベント履歴から導出される生態系変化の直接的要因識別コード | 列挙 / コード文字列 | 資源偏在/繁殖過多/ニッチ不適合/分散不足 等 | REQ-EXP-03, REQ-EXP-04 |
| もしもレバー | `what_if_lever` | 終了時に最も影響を与えた理由コードに基づき、次回再実験で変更を推薦する1つの操作レバー | `enum WhatIfLever` | 配置 / 個体数 / 適応方針 / ばらつき | REQ-EXP-04, REQ-UI-01 |
| 生命史カード | `life_history_card` | 一巡の培養結果（4系統推移、終了ラベル、主要転換点、有力原因、次回変更案）を記録するカード | `struct LifeHistoryCard` | 記録オブジェクト | REQ-UI-07, REQ-SCOPE-07 |
| ひとこと仮説 | `pre_run_hypothesis` / `hypothesis` | 培養開始前にプレイヤーが選択する3問の仮説（最初に増える系統、最後まで残る系統、心配なこと） | `struct Hypothesis` | 選択肢構造体 (終了時: 起きた/起きなかった/未判定) | REQ-SCOPE-08, REQ-UI-06 |
| 遺伝的ばらつきレバー | `genetic_variation_lever` | 初期配置時のセル内アレル分布のばらつき幅を調整する操作レバー | `Fixed` | ±0.05 上限 (scale=1e6) | REQ-SCOPE-03, REQ-GEN-08 |
| 初期配置レバー | `initial_placement_lever` | 系統ごとの初期配置セル位置を指定・調整する操作レバー | `PlacementConfig` | 格子座標配置 | REQ-SCOPE-03 |
| 初期個体数レバー | `initial_population_lever` | 系統ごとの初期投入生体量を調整する操作レバー | `[Fixed; 8]` | mass_u | REQ-SCOPE-03 |

---

## 9. スコープ外用語（REQ-OUT-01/03 参照、初期版以降）

※以下の用語は製品企画・相談会で言及されたが、検証版（v0.2）では意図的にスコープ外（REQ-OUT-01〜03）とされたか、初期ストア版以降の仕様策定待ち（初期版以降）となっている概念である。

| 語（日本語） | 識別子（英語） | 定義 | 型 | 単位・値域 | 出典 REQ・状態 |
|---|---|---|---|---|---|
| 培養カプセル | `culture_capsule` | 初期条件・モデル版・seedをQR/コード化して端末間で培養を再生する共有形式 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.4（初期版候補） |
| 一セル顕微鏡 | `cell_microscope` | 特定セルの物質受け渡しアニメーションを局所リプレイ再生するUI機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.2（初期版候補） |
| 観察の約束 | `observation_promise` | 開始時に指定した観察対象領域が転換点に近づいた際に減速・強調する機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.2（初期版候補） |
| 観察者の付箋 | `observer_note` | 生命史カードにプレイヤー自身の感想や仮説メモを添付する機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.4（初期版候補） |
| 三秒の転換標本 | `three_second_specimen` | 最大転換点の前後3秒間をループ再生して閲覧する標本カード機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.4（初期版候補） |
| 名付け | `naming` | 新系統が発生した際にプレイヤーが固有名称を命名する機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-01, 統合案 v0.5 2.3（初期版候補） |
| コドン編集・遺伝子編集 | `codon_editing` | 系統の遺伝子配列やコドンを詳細に手動編集する機能 | スコープ外（検証版対象外） | スコープ外（検証版対象外） | REQ-OUT-01（検証版対象外） |
| 捕食 | `predation` | ある系統が生きた別系統の生体量を直接摂取・捕食する相互作用 | スコープ外（検証版対象外） | スコープ外（検証版対象外） | REQ-OUT-01, REQ-SCOPE-05（検証版は死骸利用のみ） |
| 分岐（種分化） | `speciation` | シミュレーション進行中に既存系統から新たな独立系統が発生・分岐する現象 | スコープ外（検証版対象外） | スコープ外（検証版対象外） | REQ-OUT-01（検証版対象外） |
| 突然変異（計算侵入） | `mutation_active` | 計算ループ内で系統形質値が動的に変異・改変される機構 | スコープ外（検証版対象外） | スコープ外（検証版対象外） | REQ-OUT-01（検証版対象外） |
| 裏返し培養 | `reverse_culture` | 系統名や形質札を伏せた状態で結果から推理するゲームモード | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.5（後続候補） |
| 生命史の音 | `life_history_sound` | 生態系の状態変化や転換点を音響効果でフィードバックする機能 | スコープ外（初期版以降） | スコープ外（初期版以降） | REQ-OUT-03, 統合案 v0.5 2.5（後続候補） |

---

## 10. 集計

- **登録総語数（terms）**: 116 語
- **検証版スコープ内用語数（in-scope）**: 104 語
- **スコープ外用語数（out-of-scope, REQ-OUT-01/03 参照、初期版以降）**: 12 語
