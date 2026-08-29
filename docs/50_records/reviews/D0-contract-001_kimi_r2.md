# [D0-contract-001][review] r2 reviewer=cursor-kimi（試験可能性・未定義定数）

- 対象: commit 0b92499（simulation_contract.md, config.schema.json, save.schema.json（新設）, sim-types/src/lib.rs）
- 基準: docs/50_records/reviews/D0-contract-001_統合判定.md（A1〜A4 / B1〜B6 / C1〜C4 / D1〜D5）＋ r1 の 21 件
- 判定: **changes_requested**（21 件中 19 件解消・1 件部分・1 件未解消。未解消は B カテゴリ（必須）の §5 二者択一残存。新規の軽微矛盾 1 件）

## 1. r1 21 件の解消状況

| # | finding（r1） | 判定 | 確認結果 |
|---|---|---|---|
| 1 | §1 occupancy_peak 更新則なし | **解消** | §2 に第 7 phase `occupancy`（`biomass_sum >= theta_occ → 1.0`、他 `×0.995`）、空き家条件と θ＝初期栄養中央値の 10% を明記。`Thresholds` に `occupancy_threshold` / `vacant_nutrient_threshold` 追加 |
| 2 | §2 拡散係数なし | **解消** | §7 に「環境レコードから取得、初期仮説 0.05/近傍/tick」明記（C1） |
| 3 | §2 摂取量決定則なし | **解消** | §7 に「系統 ID 昇順で逐次、上限 = intake 倍率 × 基準摂取量（D3 で確定）」明記（C2） |
| 4 | §2 θ_w・×1.4 なし | **解消** | §8 に `toxin_maintenance_multiplier = 1.4`、θ_w は「D3 で確定する初期仮説」と明記。`Thresholds` に両フィールド追加（値の設定自体は D3） |
| 5 | §2 繁殖余剰閾値なし | **解消** | §7 に「energy > 維持コスト × 2」初期仮説を明記（C3） |
| 6 | §3 流入量なし | **解消** | `InflowEvent { tick, pool, amount }` 型追加、§3「閉鎖系は空 Vec」、§9「初期総量＋Σ inflow.amount」、config schema も `inflow` 配列へ更新 |
| 7 | §4 余り戻し先なし | **解消** | §4 表に「余り戻し先」列追加（主出力プールへ戻す） |
| 8 | §5 残余戻し先の二者択一 | **未解消** | §5「余りは固定された近傍順**または**セル内残余へ戻す」が **変更されず残存**。統合判定 B4「二者択一を廃止」「拡散の余りは送り元セルに残す」が契約本文に反映されていない（§4 表は変換のみで、拡散・除算の余りの規定が無い） |
| 9 | §6 PRNG 未指定 | **解消** | SplitMix64 → xoshiro256**、`prng=xoshiro256ss-v1` 明記（A3）。`ModelVersion.prng` に保持 |
| 10 | §8 Coexist 継続条件なし | **解消** | 「上限時の瞬間条件でよい」明記（B2） |
| 11 | §8 判定タイミング・同率順位 | **解消** | Extinct/Fixed＝毎 tick 即終了、Coexist/Reversal＝上限時のみ（B1）。同率は生体量降順・系統 ID 昇順（B3） |
| 12 | §10 ハッシュ関数未指定 | **解消** | SHA-256、`hash=sha256-v1` 明記（A4）。`ModelVersion.hash` に保持 |
| 13 | ConversionRule なし | **解消** | `ConversionRule { from, to, coefficient, remainder_to }` 追加（D1） |
| 14 | TerminationRule なし | **解消** | `TerminationRule { label, timing, priority }`＋`TerminationTiming { EveryTick, AtTimeLimit }` 追加（D1） |
| 15 | StateSnapshot なし | **解消** | `StateSnapshot { state, prng }` 追加（D1） |
| 16 | inflow 型なし | **解消** | `InflowEvent` 追加（D1） |
| 17 | unit struct 6 件 | **部分** | `RoundingMode{ TowardZero }`・`NumericError{ Negative, OverflowI64, OverflowI128 }`・`ModelVersion{ major, minor, scale, rounding, prng, hash }` は中身あり（D2）。`ScanOrder / RandomStream / VerifySuite` は unit のまま。D2 は「D1 で埋める旨の TODO 可」だが **TODO コメントの明記が無い** |
| 18 | 系統定数が required 外 | **解消** | `mortality_threshold` / `waste_emission` を required 化、`additionalProperties: false`（D3） |
| 19 | trait_vector 等が無検証 | **解消** | 5 軸・5 ビットのキー明示、`additionalProperties: false`（D3） |
| 20 | prng_state なし | **解消** | save.schema.json 新設、`prng_state` 必須（D4）。result schema 不変 |
| 21 | seed・grid 上限なし | **解消** | seed ≤ u64 max、grid 1〜65535（D3） |

集計: 解消 19 / 部分 1（#17）/ 未解消 1（#8）

## 2. 統合判定 A〜D の反映確認

- **A1 toxin**: multiplier=1.4 明記・`Thresholds` 両フィールド追加 ✓。θ_w の初期値は未設定だが「D3 で確定する初期仮説」の明記あり（値の提案は Codex 作業として残る）
- **A2 occupancy**: 第 7 phase・飽和/減衰・空き家条件・θ の定義 ✓。θ_occ の初期値も同様に D2 送りの明記
- **A3 PRNG**: ✓ 完全反映
- **A4 hash**: ✓ 完全反映
- **B1 判定タイミング / B2 Coexist / B3 同率**: ✓ 完全反映
- **B4 残余戻し先**: **部分**。変換の余り（§4 表）は反映済みだが、「拡散の余りは送り元セルに残す」が契約に無く、§5 の二者択一（「近傍順またはセル内残余」）が残存（＝ r1 #8 の未解消）
- **B5 inflow / B6 starvation_and_death**: ✓ 完全反映（`TickPhase::StarvationAndDeath`、ReasonCode::Death 経路明記）
- **C1〜C4**: ✓ いずれも「初期仮説」または「Dn で確定」の明記あり（§7 に集約配置。拡散・摂取・繁殖は §2 関連事項なので節の配置はやや不自然だが内容は充足）
- **D1〜D5**: ✓ 反映（D2 の unit struct 3 件は TODO 明記なし＝ r1 #17 部分）

## 3. 新たな矛盾・注記

- **N1（要修正・軽微）**: config schema の `lineages[].id` が `{"type": "string"}` のままで、sim-types の `LineageParams.id: u8` と不整合。§2・§8 の「系統 ID 昇順」は数値順を前提に読めるため、schema は `{"type": "integer", "minimum": 0, "maximum": 255}` に揃えるべき（r1 の時点で既存だが、D3 で schema を厳密化した今回の commit で顕在化）
- N2（注記）: save schema の `prng_state` は 4 ストリームのみで `seed` を含まない（sim-types の `PrngState` は `seed: Seed` を保持）。トップレベルの `seed` と冗長なので意図的なら契約か schema に一言あると親切
- N3（注記）: state_hash の hex 長が save schema（`{64}`）と result schema（`+`）で不揃い。D4 は result 不変の方針だが、SHA-256 確定に伴い result 側も `{64}` に揃えることを推奨

## 4. 合否

status=changes_requested

findings:
- 未解消 1 件: r1 #8（§5 の残余二者択一が残存し、B4「拡散の余りは送り元セルに残す」が契約本文に未反映）— B は必須カテゴリのため修正を要求
- 部分 1 件: r1 #17（`ScanOrder / RandomStream / VerifySuite` が unit のまま。D2 条件の「D1 で埋める旨の TODO」明記を求める）
- 新規 1 件: N1（config schema `lineages[].id` の string/u8 不整合）
- 注記 2 件: N2（save schema の prng_state に seed 無し）、N3（state_hash hex 長の不揃い）

修正は §5 の 1 文置換（二者択一廃止・拡散の余りは送り元に残す旨）＋ unit struct 3 件への TODO 明記＋ schema `id` の integer 化で完了する見込み。次回 commit で approve 予定。
