# BD-06 数値モデル

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/要件定義書_検証版_v0.2.md`（sign-off 済）、BD-05（契約）§5
- 完成条件: 64×64 × 8 系統 × 2,000 tick で i128 に収まる計算が表で示され、上限値が UT の境界値になる
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。丸め・余りの規則は BD-05 §5、乱数は BD-07 を参照

## 1. 表現と scale

- `Fixed = i64`、scale = 1_000_000（10 進 6 桁）。確定。参照: REQ-CON-02
- 乗算は `(a as i128 * b as i128) / scale`、除算はゼロ方向丸め 1 種（`RoundingMode::TowardZero`）。確定。参照: REQ-CON-02
- i64 の絶対値上限は 2^63 − 1 ≈ 9.22×10^18（実数 ≈ 9.22×10^12）。i128 の絶対値上限は 2^127 − 1 ≈ 1.70×10^38

## 2. 設計上限（各量の上限）

設計上限は **config 検証で強制する入力の上限**であり、これを超える config は create 時に拒否する（拒否型。BD-03 §4）。上限そのものは UT の境界値とする。参照: REQ-SIM-13, REQ-SIM-14

| # | 量 | 上限（Fixed 単位） | 確定度 | 強制箇所 | 参照 |
|---|---|---|---|---|---|
| N1 | 格子セル数 | 64 × 64 = 4,096（検証版）。契約上限 65,535² | 確定 | config schema（grid.width/height ≤ 65535）+ 検証版 config 検査 | REQ-SIM-01 |
| N2 | 系統数 L | 8 | 確定 | config schema（lineages maxItems=8） | REQ-SIM-01 |
| N3 | 初期総質量 M₀ | 10^14（実数 10^8） | 初期仮説（D2 で確定） | config 検証（create 時） | REQ-SIM-06, REQ-SIM-14 |
| N4 | 流入総量 Σ inflow.amount | 10^14 | 初期仮説（D5 で確定） | config 検証（create 時） | REQ-SIM-07 |
| N5 | 総質量 M_max = M₀ + Σinflow | 2×10^14 | 初期仮説（N3/N4 に従属） | N3・N4 の検証で導出 | REQ-SIM-06, REQ-SIM-07 |
| N6 | セル単一プール（nutrient / carcass / waste / biomass[L]） | M_max = 2×10^14（全質量の 1 セル集中を許す） | 初期仮説（N5 に従属） | property test（INV-01/02/04） | REQ-SIM-13 |
| N7 | energy[L] | 10^6（= 1.0） | 確定 | phase 適用後の値域検査 | REQ-SIM-08 |
| N8 | occupancy_peak | 10^6（= 1.0） | 確定 | occupancy phase の飽和規則 | REQ-SIM-03a |
| N9 | 質量係数（各 ConversionRule.coefficient） | 10^6（= 1.0） | 確定 | config 検証 | REQ-SIM-05 |
| N10 | 札 5 軸の各倍率 | 4×10^6（= 4.0） | 初期仮説（D4 で確定） | config 検証 | REQ-GEN-01, REQ-GEN-04 |
| N11 | toxin_maintenance_multiplier | 1.4×10^6（= 1.4） | 初期仮説（D3 の実測で更新しうる） | config 検証 | REQ-SIM-02 |
| N12 | 拡散係数（pool ごと・近傍あたり） | 5×10^4（= 0.05）。4 近傍合計 ≤ 2×10^5（= 0.2）< 1.0 | 初期仮説（D2 で確定） | config 検証（合計 < scale を強制） | REQ-SIM-10 |
| N13 | base_intake / base_maintenance | 10^9 | 初期仮説（D3 で確定） | config 検証 | REQ-SIM-11, REQ-SIM-12 |
| N14 | mortality_threshold / waste_emission | M_max = 2×10^14 | 確定（型の上限） | config 検証 | REQ-GEN-01 |
| N15 | tick | 2,000（u32 で表現） | 確定 | 終了判定（TimeLimit） | REQ-SCOPE-06 |
| N16 | 1 tick の inflow 件数 | 4,096（セル数と同数まで） | 初期仮説（D5 で確定） | config 検証 | REQ-SIM-07 |

## 3. phase ごとの最大中間値とビット幅の証明表

中間値は除算（/ scale）**前**の値を記す。必要ビット数 = ⌈log₂(最大中間値 + 1)⌉。全て i128（127 ビット）に収まることを示す。確定（N3/N4/N10/N12/N13 の初期仮説が確定値に更新されても、上限を超えない限り本表は成立する）。参照: REQ-SIM-14

| # | phase | 演算 | 最大入力 | 最大中間値 | 必要 bit | 格納 | 判定 |
|---|---|---|---|---|---|---|---|
| P1 | diffuse | pool × diff_coeff（近傍あたり） | 2×10^14 × 5×10^4 | 1×10^19 | 64 | i128 | i64（≈9.2×10^18）を超えるため **i128 必須**。除算後 ≤ 10^13 で i64 に収まる |
| P2 | diffuse | 4 近傍への送出合計 + 残余 | 4 × 10^13 + 残余 | ≤ 2×10^14 | 48 | i64 | 送り元の減少量 = 送出合計 + 残余（INV-03） |
| P3 | intake | intake 倍率 × base_intake | 4×10^6 × 10^9 | 4×10^15 | 52 | i128 | 除算後 ≤ 4×10^9 |
| P4 | intake | take × coefficient（biomass/waste 配分） | 2×10^14 × 10^6 | 2×10^20 | 68 | i128 | **全 phase 最大**。除算後 ≤ 2×10^14 |
| P5 | maintenance | base_maintenance × maintenance_cost 倍率 | 10^9 × 4×10^6 | 4×10^15 | 52 | i128 | 除算後 ≤ 4×10^9 |
| P6 | maintenance | cost × toxin_multiplier（toxin_sensitive かつ waste > θ_w） | 4×10^9 × 1.4×10^6 | 5.6×10^15 | 53 | i128 | 除算後 ≤ 5.6×10^9 |
| P7 | starvation_and_death | loss = min(biomass, cost − energy) | 2×10^14 | 2×10^14（乗算なし） | 48 | i64 | 減算のみ。cost − energy は cost ≤ 5.6×10^9 で非負を検査 |
| P8 | reproduction | (energy − 2×cost) / 2 × energy→質量係数 | 10^6 × 10^6 | 1×10^12 | 40 | i128 | energy ≤ 1.0 に制約され小さい。除算後 ≤ 10^6 |
| P9 | emission | 排出量 × waste_emission | 2×10^14 × 10^6 | 2×10^20 | 68 | i128 | P4 と同桁。除算後 ≤ 2×10^14 |
| P10 | occupancy | occupancy_peak × 995,000 | 10^6 × 995×10^3 | 9.95×10^11 | 40 | i128 | 除算後 ≤ 10^6 |
| P11 | 終了判定 | Σbiomass（全セル・全系統） | 4,096 × 8 × 2×10^14 | 6.6×10^18 | 63 | i64 | i64 ギリギリ収まる（< 9.2×10^18）。**集計は i128 で行い結果を検査** |
| P12 | 終了判定 | ε = 初期総生体量 × 10^5（1e-4） | 2×10^14 × 10^5 | 2×10^19 | 65 | i128 | 除算後 ≤ 2×10^9 |

### 3.1 証明の骨子

1. 閉鎖系では総質量が保存され（INV-01）、流入系でも N4 により M_max を超えない。したがって任意の単一プール ≤ M_max = 2×10^14 が全 2,000 tick で成立する。参照: REQ-SIM-06, REQ-SIM-07
2. energy・occupancy_peak は値域 [0, 10^6] に制約される（INV-05、INV-09）。参照: REQ-SIM-08, REQ-SIM-03a
3. 乗算の被乗数は N6/N7/N8、乗数は N9〜N13 で抑えられるため、最大中間値は P4/P9 の 2×10^20（68 bit）であり、i128 の 127 bit に対し 59 bit の余裕（≈ 5×10^17 倍）がある
4. 除算後の値は全て ≤ 2×10^14（48 bit）で i64（63 bit）に収まる。集計のみ i128 で行う（P11）
5. tick 数 2,000 は加算の繰り返しによる増大を招かない（保存則により総量不変）。参照: REQ-SIM-14

## 4. 丸めと余り

- 除算はゼロ方向丸めのみ（負値では絶対値が小さい方向）。丸めモードは model_version の一部。確定。参照: REQ-CON-02, REQ-DET-05
- 変換余り = 入力 − Σ出力。余りは主出力プールへ戻し、拡散余りは送り元セルに残す。捨てる経路は存在しない。確定。参照: REQ-SIM-05
- 余りの最大値: 出力 1 系につき < 1（Fixed 最小単位）。1 変換あたり余り < 2、1 tick 全変換でもプール単位で保存則を満たす（INV-03 で検査）。確定。参照: REQ-SIM-05

## 5. NumericError の条件

| 条件 | 検出箇所 | 挙動 | 参照 |
|---|---|---|---|
| `Negative`: 演算結果が負（sub_nonnegative で b > a、または変換後プールが負） | 全 phase の減算・変換 | 現在 tick を中断し `Err(Negative)`。状態は違反前のまま | REQ-SIM-13 |
| `OverflowI64`: 除算後の値が i64 範囲外 | fixed::mul / div / add の出口 | 同上。設計上限内では到達不能（§3）だが防御的に検査 | REQ-SIM-13, REQ-SIM-14 |
| `OverflowI128`: 中間値が i128 範囲外 | fixed::mul の中間 | 同上。設計上限内では到達不能（§3.1-3）だが防御的に検査 | REQ-SIM-13, REQ-SIM-14 |
| 設計上限超過（N1〜N16） | create 時の config 検証 | `Err`（拒否型。NumericError ではなく ValidationError 系。BD-03 §5 ADR 候補 1） | REQ-SIM-14 |

## 6. UT 境界値（上限値のテスト化）

各上限 N3〜N16 について「上限ちょうど = 受理・正常終了」「上限 + 1 = 拒否または NumericError」を UT とする。確定。参照: REQ-SIM-13, REQ-SIM-14

| テスト | 入力 | 期待 | 参照 |
|---|---|---|---|
| UT-N3a | M₀ = 10^14 の config | create 受理、2,000 tick で NumericError なし | REQ-SIM-14 |
| UT-N3b | M₀ = 10^14 + 1 の config | create 拒否 | REQ-SIM-14 |
| UT-N6a | 全質量を 1 セル 1 プールに集中（2×10^14） | 全 phase 正常・保存則成立 | REQ-SIM-06, REQ-SIM-13 |
| UT-P4 | take = 2×10^14、coefficient = 10^6 の配分 | Ok（中間 2×10^20 を i128 で処理） | REQ-SIM-05, REQ-SIM-13 |
| UT-P11 | 4,096 セル × 8 系統に 2×10^14 ずつ配置（合計 6.6×10^18） | 集計が i128 で行われ終了判定が正常 | REQ-END-02, REQ-SIM-14 |
| UT-N7a/b | energy = 10^6 / 10^6 + 1 | 受理 / 拒否（INV-05） | REQ-SIM-08 |
| UT-N12b | 拡散係数 4 近傍合計 = scale（= 1.0） | create 拒否（送出がプールを超えうるため） | REQ-SIM-10, REQ-SIM-13 |

## 7. ADR 候補（REQ に無い設計判断）

- ADR 候補 1: 設計上限 N3/N4（10^14）は REQ に直接の根拠が無く、性能予算（REQ-NFR-01: 1 seed 2,000 tick ≤ 1.0 s）と §3 のビット幅余裕から導出した。D2 の較正で見直す。参照: REQ-NFR-01, REQ-SIM-14
- ADR 候補 2: 集計（P11）のみ i128 を結果型として使うことを許す（状態は i64 のまま）。参照: REQ-SIM-14
