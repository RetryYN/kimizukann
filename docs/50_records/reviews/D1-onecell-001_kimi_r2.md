# [D1-onecell-001][review] r2 reviewer=cursor-kimi（保存則・hash・契約逸脱）

- 対象: commit 4c14db5（反映: ba78a26「parameterize dynamics and hash all streams」＋ a5d3115「conversion remainder and threshold parameters」＋ 4c14db5「remainder rounding expectation」）
- 基準: r1 の 6 件（docs/50_records/reviews/D1-onecell-001_kimi.md）、正本は simulation_contract.md（a5d3115 反映後）
- 判定: **approve**（r1 の 6 件すべて解消。新規は軽微 1 件のみで D1 ゲートは阻害しない）

## r1 6 件の解消判定

| # | r1 finding | 判定 | 確認 |
|---|---|---|---|
| 1 | §4 余り戻し先の不一致 | **解消** | `split_output_with_rule` が `ConversionRule { coefficient, remainder_to }` 経由となり、primary=floor(in×0.7)・secondary=floor(in×0.3)・余りは `remainder_to: Biomass`（主出力）へ。intake 呼び出しも rule 経由。テスト `split_output(3, 500_000)=(2,1)` で合計保存（2+1=3）と余りの biomass 行きを検証 |
| 2 | §10 hash に PRNG・model_version なし | **解消** | `state_hash()` が `model_version`（"d1-v1;prng=xoshiro256ss-v1;hash=sha256-v1"）と rng 4 ストリーム各 [u64;4] の LE バイト列を入力に追加。ゴールデン値更新済み。D2 で乱数消費しても save/load 経路と連続経路が分かれない構成になった |
| 3 | 疑義(a) 定数ハードコード | **解消** | `Thresholds` に `base_intake`/`base_maintenance` を追加し、intake 上限=base_intake×traits.intake、維持コスト=base_maintenance×traits.maintenance_cost、θ_w=thresholds.waste_toxic_threshold、×1.4=thresholds.toxin_maintenance_multiplier（1_400_000）、θ_occ=thresholds.occupancy_threshold。すべて Thresholds/TraitVector 経由でハードコード解消 |
| 4 | 疑義(b) energy→質量 1:1 の暗黙仮定 | **解消** | 契約 §7 に「D1: energy→質量係数 1.0（intake の energy 加算・reproduction とも）、D3 で確定」と明記。私の要求どおり契約明記で閉じた |
| 5 | starvation 不足分の過剰計上 | **解消** | `energy < cost` 時に `loss = min(biomass, cost − energy)`（maintenance 後基準の不足分）へ変更。0<energy<cost で maintenance が energy=0 にする経路でも、energy 分は maintenance で消費済みのため系統損失は cost 相当で一貫。実装定義が一貫したので解消（不足分の基準点明記は D3 の数値確定に委ねる） |
| 6 | intake 走査順の ID 昇順未保証 | **解消** | `try_one_cell` で `sort_by_key(id)`＋重複 id を Err 化。§2「系統ID昇順」を構築時に保証 |

## 新規チェック（r2 での矛盾有無）

- **N1（軽微）**: `split_output_with_rule` の `_ => Ok((primary, secondary))` アームは `remainder_to` が Nutrient/Carcass の場合に余りがどこにも載らず消える（§4「余りは捨てない」に反する経路が型上存在）。現在の呼び出しは `remainder_to: Biomass` 固定で実害なし。D2 で ConversionRule を多変換に使う前に、match を網羅（全 Pool 行き先を定義）するか `remainder_to` の値域を契約で限定すること
- N2（注記）: `split_output`（旧シグネチャ）は `waste_coefficient = FIXED_SCALE − coefficient` の暗黙仮定（§4「合計1.0」と整合）
- N3（注記）: reproduction の `cost * 2` は i64 素の乗算（現状の値域で溢れない）。`cost.max(1)` の最小コスト 1 は契約に根拠なし（D3 で扱う）
- N4（注記）: `Thresholds.epsilon = 1` は固定値で、契約 §8 の `1e-4 × 初期総生体量` ではない。D1 は終了判定を持たないので実害なし。終了判定実装時に初期化計算へ置き換えること
- N5（注記）: state hash は lineages（LineageParams）を含まない。系統設定は config_hash 側で担保する設計と解釈。§10 正規化の「系統昇順」はセル内 biomass/energy 配列の順序として機能している

## 合否

status=approve

未解消=0 新規=1（N1 軽微・D2 までに対処）。保存則・hash・契約整合は D1 の完了条件を満たす。契約 §7 への係数 1.0 明記も確認。D2（拡散・多セル）へ進んでよい。
