# [D1-onecell-001][review] reviewer=cursor-kimi（保存則・hash・契約逸脱）

- 対象: commit cbbbaed（crates/sim-core/src/lib.rs, sim-types/src/lib.rs, sim-cli/src/main.rs）
- 正本: docs/contracts/simulation_contract.md（commit 2ec7d6d 時点）
- 判定: **changes_requested**（保存則・非負・決定性はテスト上成立。契約逸脱 3 件は D2 で PRNG 消費・多セル化した時点で破綻するため D1 のうちに修正を求める）

## 検証済み（問題なし）

- **保存則**: 全変換がプール間移動で閉じる（intake: nutrient→biomass+waste、starvation/death: biomass→carcass、reproduction: nutrient→biomass、emission: biomass→waste）。`invariant_report` の質量一致・非負・energy 0〜1 クランプは契約 §9 と整合
- **§2 phase 順**: diffuse→intake→maintenance→starvation_and_death→reproduction→emission→occupancy の 7 phase 固定順どおり（diffuse は 1 セル no-op で phase として存在、brief どおり）
- **§5 丸め**: `fixed::mul` は i128 中間・ゼロ方向丸め（Rust 整数除算）で契約どおり。`split_output(3, 500_000) = (1, 2)` のテストで余り非消失を確認
- **§6 PRNG**: SplitMix64 最終段による [u64;4] 初期化＋ xoshiro256** 標準遷移の実装は正しい。4 ストリーム（seed^0..3）を保持し D1 では未消費（brief 3 どおり）
- **§10 hash**: SHA-256・i64 LE バイト列・セル row-major・biomass/energy は固定配列で系統 ID 昇順と一致。ゴールデン値固定あり。sim-cli の determinism 2 経路（step(2000) vs step(1000)×2）は brief 6 どおり

## findings

1. **§4 余り戻し先の不一致（契約逸脱・要修正）**。契約 §4 は「変換の余りは常に主出力プールへ戻す」で摂取の主出力は biomass。実装は `split_output(amount, 700_000)` で `to_biomass = floor(amount×0.7)`、`to_waste = amount − to_biomass` となり、余り（高々 1 u）は **waste 側**に乗る。契約の厳密な読み（biomass=floor(0.70×in)、waste=floor(0.30×in)、余りは biomass へ）と配分が異なり、waste 蓄積→毒性判定（θ_w）に系統的な偏りを入れる。`ConversionRule.remainder_to` を使う実装に直すこと
2. **§10 state hash に PRNG 状態と model_version が含まれない（契約逸脱・要修正）**。契約 §10 は正規化に「PRNG状態、model_versionを含み」と明記。現実装は tick/seed/寸法/セル内容のみ。D1 では PRNG 未消費のため determinism は成立するが、D2 で乱数消費を始めた瞬間に save/load 経路と連続経路の hash が分かれる欠陥になる。`rng` 各ストリームの state（[u64;4]×4）と `ModelVersion` 由来のバイト列をハッシュ入力に追加すること
3. **Claude 疑義 (a): 定数のハードコード（契約逸脱・要修正）**。事実確認: intake 上限 `100_000`・維持コスト `10_000`・θ_w 相当 `100_000`（maintenance 内）・θ_occ 相当 `FIXED_SCALE`（occupancy 内）がすべて直書きで、`Thresholds`（waste_toxic_threshold / toxin_maintenance_multiplier / occupancy_threshold）も `TraitVector`（intake / maintenance_cost / reproduction 各倍率）も経由していない。契約 §7「1tick上限は intake倍率 × 基準摂取量」「札の5軸は固定倍率」に反する。D1 で倍率適用を後送りにするなら、コード内 TODO と契約 §7 への「D1 は基準値固定・倍率未適用」明記のどちらかで閉じること（黙って無視は不可）
4. **Claude 疑義 (b): reproduction の energy→質量変換（判定: 保存則上は問題なし、係数の明記を要求）**。`gain = (energy − 2·cost)/2` を energy（無次元予算）から引き、同値を nutrient→biomass で移す。質量台帳は閉じ energy も負にならず、§1「energy は無次元の系統別予算」の消費としては成立する。ただし無次元予算の減少量と質量移動量の 1:1 対応は、§4「エネルギー係数は物質係数と別の無次元値」で言うエネルギー係数を **暗黙に 1.0 と仮定**している。契約 §7 の繁殖初期仮説に「D1 は energy→質量係数 1.0（1 u 対 1 u）」と明記するか、係数を変数化して D3 確定事項に載せること。intake の `energy += to_biomass`（質量をそのまま加算）も同じ暗黙仮定で、併せて明記対象
5. **starvation の不足分が過剰（軽微）**。maintenance で `energy < cost` の場合 energy=0 に丸め、starvation では energy==0 なら `biomass.min(cost)` を削る。部分不足（0 < energy < cost）では実際の不足は `cost − energy` なのに `cost` 全額を生体量から削るため、§2「energy不足分を生体量から死骸へ」の「不足分」とずれる。保存則は満たすが、不足分の定義（課金失敗時に partial を記録するか）を D3 確定事項として明記すべき
6. **intake 他の走査順が系統 ID 昇順と未保証（軽微）**。`for lineage in &self.state.lineages` は Vec 順で、§2「系統ID昇順」の保証がない（1 セル・同一プール競合で順序が結果に効く）。`lineages` を id でソート済みにするか、one_cell 構築時に検証すること

注記（finding に含めない）: `RandomStream` は TODO(D1) の unit struct のまま（`StreamKind` 追加のみ。D1 で未消費なので実害なし、D2 で定義）。`fixed::div` のゼロ除算が `NumericError::Negative` 分類（§5 にゼロ除算の規定なし。専用 variant か契約明記を）。`one_cell` は `lineages[0]` にしか初期 biomass を設定できず brief の「系統 1〜2」の 2 系統初期配置が作れない。初期 energy = 0.5 の根拠は契約に無く D1 初期仮説として明記推奨。

## 合否

status=changes_requested

findings=6（要修正: 1・2・3、明記要求: 4、軽微: 5・6）。1〜3 を修正した commit で approve 予定。4 は契約 §7 への 1 行明記で閉じられる。
