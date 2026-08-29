# [D0-contract-001] 統合判定（Claude）

- 入力: `D0-contract-001_claude.md`（6 件）、`D0-contract-001_kimi.md`（21 件）
- 判定: **changes_requested**。1 commit で以下を反映後、再提出

## A. 最優先（v0.5 確定定数と再現性の根幹）— 必須
| # | 項目 | 決定（仕様オーナー） |
|---|---|---|
| A1 | toxin 倍率と θ_w（kimi 4 / claude 1） | §4 か §7 に `toxin_maintenance_multiplier = 1.4`、`waste_toxic_threshold θ_w` を初期仮説として置く。θ_w の初期値は Codex が提案し「初期仮説」と明記。`Thresholds` に両フィールド追加 |
| A2 | occupancy_peak 更新則（kimi 1 / claude 2） | emission の後に第 7 phase `occupancy` を置く。`biomass_sum ≥ θ_occ → occupancy_peak = 1.0`、それ以外 `×0.995`。空き家条件 `occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ`、θ＝セル栄養の初期中央値の 10%。θ_occ は Codex 提案・初期仮説 |
| A3 | PRNG アルゴリズム（kimi 9） | **SplitMix64 で seed から 4 ストリームの初期状態を導出し、各ストリームは xoshiro256\*\***。バージョン文字列 `prng=xoshiro256ss-v1` を `ModelVersion` に含める |
| A4 | state hash 関数（kimi 12） | **SHA-256**（`sha2` crate）。正規化バイト列の定義は §10 のまま。`hash=sha256-v1` を `ModelVersion` に含める |

## B. 機械判定できる文にする — 必須
| # | 項目 | 決定 |
|---|---|---|
| B1 | 終了判定タイミング（kimi 11） | `Extinct` と `Fixed` は毎 tick 判定して即終了。`Coexist` と `Reversal` は `TimeLimit` 到達時にのみ判定する終了時ラベル。優先順は全滅＞固定＞（上限時）共存＞逆転＞上限 |
| B2 | Coexist 継続条件（kimi 10） | 上限時の瞬間条件でよい（B1 により継続不要）。契約に明記 |
| B3 | Reversal の同率（kimi 11） | tick 0 の順位は生体量降順、同率は系統 ID 昇順 |
| B4 | 残余の戻し先（kimi 7・8 / claude） | 二者択一を廃止。**変換の余りは常にその変換の主出力プール（表の 1 行目）へ戻す。拡散の余りは送り元セルに残す**。§4 の表に「余り戻し先」列を追加 |
| B5 | inflow の形（kimi 6 / claude 3） | `inflow: Vec<InflowEvent { tick: u32, pool: Pool, amount: Fixed }>`。閉鎖系スイートは空 Vec。§9 の期待値＝初期総量＋Σamount |
| B6 | death phase（claude 4） | starvation に統合しない。第 4 phase を `starvation_and_death` とし、energy 不足分の carcass 化に加え、`biomass[L] < mortality_threshold` のセル系統は全量 carcass 化（ReasonCode::Death） |

## C. 段階で確定する項目 — 契約に「初期仮説」または「Dn で確定」と明記すれば可
| # | 項目 | 扱い |
|---|---|---|
| C1 | 拡散係数（kimi 2） | D2 で確定。契約には「環境レコード `diffusion_coefficients[pool]` から取る。初期仮説 0.05/近傍/tick」と書く（値は Codex 提案可） |
| C2 | 摂取量決定則と配分順（kimi 3） | D3 で確定。契約には「系統 ID 昇順で逐次（按分しない）、1 tick 上限は `intake 倍率 × 基準摂取量`」を初期仮説として書く |
| C3 | 繁殖の余剰判定（kimi 5） | D3 で確定。初期仮説「energy > 維持コスト × 2 のとき余剰の一定割合を生体量へ」 |
| C4 | movement 軸（claude 6） | D2 で確定。§2 に open issue として明記（生体量の近傍拡散率案） |

## D. 型と schema — 必須
- D1 `ConversionRule { from, to, coefficient, remainder_to }`、`TerminationRule { label, timing: EveryTick|AtTimeLimit, priority }`、`StateSnapshot`、`InflowEvent` を追加（kimi 13〜16）
- D2 `RoundingMode = enum { TowardZero }`、`NumericError = enum { Negative, OverflowI64, OverflowI128 }`、`ModelVersion { major, minor, scale, rounding, prng, hash }`（kimi 17 / claude 5）。`ScanOrder / RandomStream / VerifySuite` は D1 で埋める旨の TODO 可
- D3 config schema：`lineages[].required` に `mortality_threshold`, `waste_emission`；`trait_vector` 5 キー・`mechanism_tags` 5 キーを明示し `additionalProperties: false`；`seed` は u64 範囲、`grid.width/height` は 1〜65535（kimi 18・19・21）
- D4 `SaveEnvelope` 用 schema を `docs/contracts/schema/save.schema.json` として新設し `prng_state` を含める。result schema は変更しない（kimi 20）
- D5 §1 の状態列挙に `tick: u32` を追加（kimi 注記）

## 合否条件
A〜B〜D がすべて反映され、C が「初期仮説」または「Dn で確定」と明記されていれば approve。再提出は `[D0-contract-001][result] status=pass commit=...`。
