# [D0-contract-001][review] reviewer=cursor-kimi（試験可能性・未定義定数）

- 対象: commit 44e1bb4（docs/contracts/simulation_contract.md v0.1、docs/contracts/schema/*.json、crates/sim-types/src/lib.rs）
- 正本: docs/相談会/第2回_統合案_v0.5.md 1.1〜1.3, 1.6, 7.1〜7.3
- 観点: 各節が機械判定できる文か／v0.5 確定定数が契約に入っているか／契約状態と sim-types の対応／schema と契約の整合。数値の妥当性（数値契約）には踏み込まない
- 判定: **changes_requested**（骨格・型の網羅性は良好。v0.5 7.1 の確定定数 3 件が契約に未反映で、§10 の再現性テストが現状では定義不能）

## (1) 機械判定可能性（曖昧な閾値・単位・順序の列挙）

1. **§1 `occupancy_peak` の更新規則が存在しない**。減衰率（v0.5 では 0.995/tick）、飽和条件（生体量合計が閾値超で 1）、空き家条件（`occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ`）、更新する tick phase のいずれも契約に無く、この状態の不変条件テストが書けない（v0.5 1.1 / 7.1 の確定事項が未反映）
2. **§2 `diffuse` の拡散係数が未定義**。「4近傍へ移す」は移行割合（係数・単位）が無く判定不能。v0.5 7.3 の環境レコード `diffusion_coefficients` との対応も契約本文に無い
3. **§2 `intake` の摂取量決定則が未定義**。「利用可能なプールを取り」は、1 tick あたりの摂取上限・`TraitVector.intake` の入り方・複数系統が同一プールを取る際の配分順（系統 ID 昇順で枯渇するか按分か）が無く機械判定できない
4. **§2 `maintenance` の毒性閾値 θ_w と倍率が未定義**。「毒性閾値超過時は倍率を適用」は判定文にならない。v0.5 7.1 の確定値 `waste > θ_w → 維持コスト ×1.4` を契約に置くこと
5. **§2 `reproduction` の「energy 余剰」が未定義**。余剰判定の閾値（0 超か、維持コストの何倍か）が無く繁殖フェーズの期待値を計算できない
6. **§3 `inflow_tick_mask` が時刻マスクのみで流入量（mass_u）を持たない**。§9「初期総量＋期待流入量と一致」の期待値が config から機械計算できない。マスクの各要素が tick 番号なのか (tick, amount) なのかも不明
7. **§4 質量係数表に余りの戻し先（指定先プール）の列が無い**。「同一変換の指定先へ戻し」の指定先が表から読み取れず、残余非消失テスト（§9）の期待配置が定まらない
8. **§5「固定された近傍順またはセル内残余へ戻す」の二者択一の条件が未定義**。どちらへ戻すかが状況依存なら、その判定規則自体を固定しないと再現性テスト（§10）が不安定になる
9. **§6 PRNG のアルゴリズム名とバージョンが未指定**。「固定し」という要求だけで具体名（例: ChaCha8 等）が無く、§10 の三経路一致・seed 間バッチの期待 hash が定義できない
10. **§8 `Coexist` に継続条件・判定時点が無い**。`Fixed` は「200 tick 継続」とあるのに対し `Coexist` は「各 15% 以上」の瞬間条件のみで、1 tick でも満たせば終了か、継続が必要か読み取れない
11. **§8 終了判定の実施タイミングがラベル横断で未定義**。毎 tick 判定か TimeLimit 到達時にまとめて判定かで `Fixed`/`Coexist`/`Reversal` の結果が変わる。`Reversal` の tick 0 順位で初期生体量が同率の場合の順位付け規則も未定義
12. **§10 state hash のハッシュ関数が未指定**。正規化内容（row-major・系統昇順・i64 バイト列等）は定義済みだが、32 バイト出力の関数名が無く、実装間で hash が一致するか検証できない

## (2) v0.5 7.1 定数の含有確認

| 定数 | 契約への含有 | 備考 |
|---|---|---|
| 質量係数 4 本 | ✓ §4 | 0.70 / 0.30 / 1.00 / 1.00、合計 1.0 規定あり |
| ε = 1e-4 × 初期総生体量 | ✓ §8 | `Thresholds.epsilon` にも対応 |
| energy[L]（系統別） | ✓ §1 | sim-types も `energy: [Fixed; 8]` で一致 |
| toxin ×1.4（θ_w 含む） | **未含有** | finding 4。`Thresholds` にも該当フィールド無し |
| 空き家 θ（セル栄養の初期中央値の 10%） | **未含有** | finding 1。契約に θ の文字自体が無い |

## (3) 契約状態 ↔ sim-types 対応

セル 6 状態（`nutrient / biomass[L] / carcass / waste / energy[L] / occupancy_peak`）はすべて `CellState` の 1 フィールドに対応し、配列長 8 も §1「最大 8 系統」と一致。対応漏れは以下：

13. **`ConversionRule` が sim-types に存在しない**（§4・型対応表に登場）。`MassCoefficients` のみで変換規則（余りの戻し先を含む）を表す型が無い
14. **`TerminationRule` が sim-types に存在しない**（§8・型対応表に登場）。`Thresholds` と `TerminationLabel` はあるが判定規則（優先順・判定タイミング）を載せる型が無い
15. **`StateSnapshot` が sim-types に存在しない**（§10・型対応表に登場）。`SaveEnvelope` のみ。三経路一致の save/load 経路で使うスナップショット型が無い
16. **`inflow_tick_mask` に対応する型が sim-types に存在しない**。契約 §3・§9 と config schema（required）の三方から参照されるのに型が無く、閉鎖系＝空マスクを型で表せない
17. **`RoundingMode / NumericError / VerifySuite / ModelVersion / ScanOrder / RandomStream` が中身の無い unit struct**。§5「丸めモードと scale は model_version の一部」に対し `ModelVersion` が値を保持できず、契約条項を型で検証する D0 の完了判定に抵触する

注記（軽微）: `WorldState.tick: u32` は契約 §1 の状態列挙に明記が無い（§10 の `step(2000)` で暗黙）。`SimCore::step` は sim-core 側に存在することを確認（対応表は crate 横断と解釈）。

## (4) schema ↔ 契約の食い違い

18. **config schema の `lineages[].required` に `mortality_threshold` / `waste_emission` が無い**。契約 §7「系統定数は 2 つだけを許す」「プリセット固定」に対し schema では任意扱い（`additionalProperties: true` で黙認）で、必須 2 定数を欠いた config が検証を通る
19. **`trait_vector` / `mechanism_tags` が `{"type": "object"}` のみ**で 5 軸・5 ビットのキーと値域を検証できない。契約 §7 の固定集合に対し schema が無防備
20. **result schema に `prng_state` が無い**。契約 §10「保存には `schema_version / model_version / config_hash / seed / prng_state / state_hash` を含める」との対応が不明（SaveEnvelope 用の別 schema を作るのか、result に含めるのか要明確化）
21. **`seed`・`grid.width/height` に上限が無い**。sim-types の `Seed(u64)` / `u16` に対し schema は `minimum` のみで、範囲外値が config 検証を通りコアでエラーになる経路がある

整合確認（問題なし）: `termination_label` enum 5 種 ✓、`lineages` maxItems 8 ✓、`state_hash` hex pattern ✓、`inflow_tick_mask` の存在 ✓、`model_version` / `config_hash` 必須 ✓

## 合否

status=changes_requested

findings（21 件）:
- (1) 機械判定不能な文: §1 occupancy_peak 更新則欠落 / §2 拡散係数なし / §2 摂取量決定則なし / §2 θ_w・×1.4 なし / §2 繁殖余剰閾値なし / §3 流入量なし / §4 余り戻し先なし / §5 残余戻し先の選択条件なし / §6 PRNG アルゴリズム未指定 / §8 Coexist 継続条件なし / §8 終了判定タイミング・同率順位なし / §10 ハッシュ関数未指定
- (2) v0.5 7.1 定数の未含有: toxin ×1.4・θ_w・空き家 θ（質量係数 4 本・ε・energy[L] は含有済み）
- (3) 型の対応漏れ: ConversionRule / TerminationRule / StateSnapshot / inflow 型 / unit struct 6 件
- (4) schema 食い違い: 系統定数 2 件が required 外 / trait_vector・mechanism_tags 無検証 / prng_state なし / seed・grid 上限なし

修正優先度の提案: finding 1・4・9・12（v0.5 確定定数の未反映と再現性テストの根幹）を最優先、次に 13〜17 の型補完、schema 側（18〜21）は config 受入テスト導入前までに。
