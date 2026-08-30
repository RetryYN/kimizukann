# DD-D4 詳細設計: 4 系統プリセット・初期配置・終了判定

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 上位正本: `docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-SCOPE-02/03、REQ-GEN-01..08、REQ-END-01..05）、`docs/00_product/第2回_統合案_v0.5.md` §1.9（案 A）・§7.6、BD-03（集約・値オブジェクト）、BD-04 §1（Run 状態機械 T1）、BD-06（P11/P12）、BD-07 §2/§3（PRNG 割当）、BD-08 §4（AT-D4-01..08）
- スコープ: 4 系統プリセットの確定値、初期配置・遺伝的ばらつきレバーの機械的定義、終了判定（5 ラベル）の詳細手順
- 非スコープ: 4 環境 JSON（D5）、煙試験（D6）、較正と分布帯の判定（D7）、札・レバーの UI（D12）、台帳の保存形式（D8）
- 前提: 数値表現は Fixed（scale = 1_000_000、ゼロ方向丸め、乗算中間 i128）。phase 順・台帳仕様は DD-D3 0.3、格子・拡散は DD-D2 に従う

## 1. 4 系統プリセット（案 A の確定）

### 1.1 特性ベクトル（確定。REQ-GEN-03、v0.5 §1.9 案 A）

| id | 通称 | movement | intake | conversion | maintenance_cost | reproduction |
|---|---|---:|---:|---:|---:|---:|
| 0 | アオシキ | 700_000 (0.70) | 1_050_000 (1.05) | 950_000 (0.95) | 1_000_000 (1.00) | 850_000 (0.85) |
| 1 | シロナミ | 1_200_000 (1.20) | 800_000 (0.80) | 900_000 (0.90) | 1_100_000 (1.10) | 850_000 (0.85) |
| 2 | アカバエ | 1_000_000 (1.00) | 1_150_000 (1.15) | 850_000 (0.85) | 1_300_000 (1.30) | 1_600_000 (1.60) |
| 3 | クロシデ | 800_000 (0.80) | 450_000 (0.45) | 950_000 (0.95) | 900_000 (0.90) | 650_000 (0.65) |

- id は 0..3 に固定し、INV-08 の昇順処理・Reversal の同率タイブレークの基準とする
- REQ-GEN-04（単純上位互換の禁止）検査: アオシキは intake > 1 に対し movement < 1、シロナミは movement > 1 に対し intake < 1、アカバエは intake・reproduction > 1 に対し conversion < 1。クロシデは全軸 ≤ 1.0 で規則の適用外（意図的劣位。v0.4 付録 C）
- 変更は D7 の規則（軽い戻しのみ・勝者補正禁止。REQ-GEN-03、v0.5 §7.6）に従う

### 1.2 機構タグ（確定）

| id | 通称 | use_nutrient | use_carcass | use_waste | toxin_sensitive | density_bonus |
|---|---|---|---|---|---|---|
| 0 | アオシキ | 1 | 0 | 0 | 1 | 0 |
| 1 | シロナミ | 1 | 0 | 0 | 0 | 0 |
| 2 | アカバエ | 1 | 0 | 0 | 0 | 0 |
| 3 | クロシデ | 1 | 1 | 1 | 0 | 0 |

- 根拠（v0.4 付録 C の機構タグ欄）: アオシキ「通常資源のみ・毒耐性低」、クロシデ「死骸・老廃物利用・通常資源弱」（通常資源の「弱」は intake 0.45 で表現し、タグは立てる）
- AT-D4-03（use_carcass ≥ 1 系統。REQ-SCOPE-05）はクロシデで充足
- density_bonus は全系統 0。効果の機械的定義は未決事項 D4-Q2（§10）

### 1.3 系統定数（初期仮説。D7 較正で更新しうる）

| id | 通称 | mortality_threshold | waste_emission |
|---|---|---:|---:|
| 0 | アオシキ | 5_000 | 1_000 |
| 1 | シロナミ | 5_000 | 1_000 |
| 2 | アカバエ | 10_000（D4-Q1） | 2_000 |
| 3 | クロシデ | 5_000 | 1_000 |

- アカバエの waste_emission 2 倍は v0.4 付録 C「老廃物排出多」の直訳
- アカバエの mortality_threshold は v0.4「死亡閾値低」の解釈が機械定義（BD-04 §3.2: `biomass < mortality_threshold → Dying`）と衝突するため D4-Q1（§10）。表は推奨値（高く = 死亡しやすい、代償として機能）

## 2. 終了判定（確定）

### 2.1 判定手順

各 tick の 7 phase 完了後（BD-04 §1.2 T1）に以下を順に評価し、最初に成立したラベルで即終了する。判定は World を読むだけで書かない（BD-03 §1.2）。

1. **Extinct**（毎 tick。REQ-END-02）: 全系統の総生体量 `Σbiomass < ε`。ε = `initial_total_biomass × 100 / 1_000_000`（1e-4 の Fixed 表現は 100。乗算は i128、除算はゼロ方向丸め。BD-06 P12）。`initial_total_biomass` は create 時点の Σbiomass で確定し以後不変
2. **Fixed**（毎 tick。REQ-END-03）: ある系統 L について `share(L) ≥ 700_000`（70%）が 200 tick 連続。`share(L) = lineage_total(L) × 1_000_000 / grand_total`（集計は防御的に i128。BD-06 P11）。`grand_total = 0` のとき share は未定義とし streak をリセット（通常は Extinct が先に発火するため防御）
3. 上限 tick（`max_ticks = 2_000`。REQ-SCOPE-06）到達時のみ、順に:
   - **Coexist**（REQ-END-04a）: 2 系統以上が各 `share ≥ 150_000`（15%）
   - **Reversal**: 終了時の 1 位系統の tick 0 順位が 3 位以下（4 系統では順位 3 または 4）
   - いずれも不成立なら **TimeLimit**

同時成立時の優先順は Extinct > Fixed > Coexist > Reversal > TimeLimit（REQ-END-04c）。判定理由（成立ラベル・成立 tick・各系統 share）を保存する（BD-03 §1.1 Termination）

### 2.2 終了判定の状態

- `fixed_streak: u32`（現在の連続達成 tick 数。share ≥ 70% の系統が同じ id で継続する限り加算、それ以外で 0 にリセット）
- `tick0_ranking: Vec<u8>`（tick 0 の順位表。総生体量降順、同率は id 昇順。create 時に確定し不変）
- 両者は run 状態として保持し、state hash（sha256-v2）の正規化バイト列に含める（golden `d1-week1.json` の known_gap と整合）。Save への同梱は D8 のスコープ

### 2.3 集計の数値要件

- 系統別・総生体量の集計は i128 で行い、結果が i64 に収まることを検査してから使用（BD-06 P11）
- share の比較は除算を避け `lineage_total × 1_000_000 ≥ grand_total × threshold` の乗算形で行い、丸め誤差を排除する（初期仮説）

## 3. 初期配置（PlacementConfig）

### 3.1 配置モード

| mode | 内容 | interaction 消費 |
|---|---|---|
| `default` | 系統 id 順に象限中心 (16,16) / (48,16) / (16,48) / (48,48) へ 1 セルずつ（初期仮説。64×64 前提） | 0 |
| `explicit` | config が系統ごとのセル列を指定。範囲外・重複は拒否（ValidationError。BD-03 ADR 候補 1） | 0 |
| `random` | 系統ごとに k セルを抽選 | 配置セル × 系統ごとに 1 語（BD-07 §3 create 行） |

### 3.2 random モードの抽選（消費回数を 1 語に固定する方式）

1. interaction ストリームから 1 語 u を取得
2. 候補 = `(u % 64, (u >> 8) % 64)`（初期仮説の写像）
3. 既出セルとの衝突時は PRNG を再消費せず、`index + 1 (mod 4096)` の row-major 順で最初の空きセルへ決定的にずらす

消費回数が config のみの決定的関数（= 配置セル数 × 系統数）となり、BD-07 の「予備なし・回数固定」を満たす。配置可能セルが環境で制限される場合の扱いは D5 の環境定義に従う（D4 では全セル有効を仮定）

### 3.3 初期生体量（初期個体数レバー）

- `initial_biomass: [Fixed; L]`、既定値は全系統 1_000_000（1.0 mass_u）、配置セルごとにこの量を設定（初期仮説）
- 既定配置・既定生体量での初期総生体量は 4_000_000（4.0）、ε = 400
- 全系統 0 は拒否（ValidationError。ε = 0 となり Extinct が永遠に不成立となる退化を防ぐ。初期仮説）
- tick 0（Prepared）でも Extinct は成立しうる（BD-04 §1.2）

## 4. 遺伝的ばらつき（離散アレル。REQ-GEN-08）

- レバー値 `variation: Fixed ∈ [0, 1_000_000]`（0..1.0）
- アレル: 各系統の各軸に `a ∈ {−50_000, 0, +50_000}`（±0.05 = REQ-GEN-08 上限ちょうど）の離散オフセット。適用後の軸値 = `base + a`（加算のみ、丸めなし）
- 抽選: mutation ストリーム 1 語を 5 軸に分割し、各 12 bit の `mod 3` で {−1, 0, +1} を選び 50_000 を乗じる。`variation = 0` のとき結果を捨て全軸 0 とするが、消費回数は変えない（seed 同一性のため消費は常に系統数 × 1 語。初期仮説）
- 適用単位は系統ごと 1 セット（D4-Q3。BD-07 §3 の「配置セル × 系統ごとに 1 回」との差異は §10 で裁定依頼）
- ばらつき適用後の値は REQ-GEN-04 検査の対象外（検査はプリセット base 値に適用。初期仮説）
- 分岐・新系統の発生はない（INV-07）。tick 中の mutation 消費は常に 0（BD-07）

## 5. config schema 要件

- `lineages`: LineageParams の配列。id 一意・重複は拒否（INV-08）。`additionalProperties = false`（REQ-GEN-01、AT-D4-01）
- `placement`: 系統ごとに `{ mode, cells?, k? }`（§3.1）
- `initial_biomass`: `[Fixed; L]`、値域 `0 < x ≤ 2×10^14`（BD-06 上限）
- `variation`: Fixed、値域 `[0, 1_000_000]`。範囲外は拒否（REQ-GEN-08 の UT「上限・上限+1」）
- 終了ラベルは enum `TerminationLabel` の 5 種のみ（AT-D4-05。schema enum 検査）

## 6. UT 設計（実数仕様）

config 特記なき限り 1 セル・inflow なし・対象 phase のみ適用。終了判定 UT は判定器を直接呼ぶ（7 phase は介さない）

| ID | config / 入力 | 期待 |
|---|---|---|
| UT-D4-01 | §1.1 プリセット 4 系統 | 各系統で「1.0 超の軸あり ⇒ 1.0 未満の軸あり」。クロシデは適用外（REQ-GEN-04） |
| UT-D4-02 | 初期総生体量 4_000_000 | ε = 400（i128 中間・ゼロ方向丸め） |
| UT-D4-03 | Σbiomass = 399 / 400（ε = 400） | 399 → Extinct / 400 → 非終了（境界） |
| UT-D4-04 | 系統 0 の share = 700_000 を 199 tick / 200 tick 継続 | 199 → 非終了 / 200 → Fixed（REQ-END-03）。699_999 では streak 不発 |
| UT-D4-05 | streak 中に別系統が 70% を奪う / 70% 割れ | streak リセット（連続性の検査） |
| UT-D4-06 | tick 2_000、share = [150_000, 150_000, 700_000, 0] | Coexist（15% ちょうどは ≥ で成立） |
| UT-D4-07 | tick 0 生体量が同率（例: 全系統 1.0） | tick0_ranking は id 昇順（REQ-END-04b 同率ケース） |
| UT-D4-08 | tick 2_000、1 位系統の tick 0 順位 = 3 位 / 2 位 | 3 位 → Reversal / 2 位 → TimeLimit |
| UT-D4-09 | Σbiomass < ε かつ 1 系統が 70% 超 | Extinct（優先順 REQ-END-04c） |
| UT-D4-10 | variation = 1_000_050（+0.05 超の軸を生成） | 拒否（REQ-GEN-08 上限+1） |
| UT-D4-11 | random 配置、同 seed 2 run | 配置が一致し interaction 消費 = 配置セル数 × 系統数（各 1 語） |
| UT-D4-12 | random 配置で候補衝突を強制（k = 4096） | 全セルが重複なく埋まり、消費は 1 語/セルのまま（決定的ずらし） |

## 7. AT 対応（BD-08 §4）

| AT | 対応 |
|---|---|
| AT-D4-01 | §1 プリセット + §5 schema 検査（本 DD の §1 表が正本） |
| AT-D4-02 | ReasonCode 網羅 fixture。代表 seed の選定は D7、到達不能なら REQ-SCOPE-04 の RFC（持ち越し） |
| AT-D4-03 | §1.2 タグ表の schema 検査 |
| AT-D4-04 | 空き家判定は D2（occupancy）依存。代表 seed は D7（持ち越し） |
| AT-D4-05 | §5 enum schema 検査 |
| AT-D4-06 | UT-D4-03 の config を引用（BD-08 の「D4 の境界値 UT で構成」に対応） |
| AT-D4-07 | UT-D4-04/05 の config を引用（同上） |
| AT-D4-08 | UT-D4-06 の config を引用（同上） |

## 8. 性能

- 終了判定は全セル × 全系統の生体量集計が O(4096 × 4) / tick。i64 加算のみで拡散（DD-D2 §8）より軽く、NFR-01 予算（200 ms / 2,000 tick）への影響は無視できる（初期仮説）
- 集計は 1 pass で系統別・総量を同時に求める

## 9. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-core/src/termination.rs` | 判定器（§2）。World を読むだけ |
| `crates/sim-core/src/placement.rs` | 初期配置（§3）・アレル抽選（§4） |
| `docs/30_contracts/presets/lineages_v1.json` | §1 プリセット（schema 検査の fixture） |
| `crates/sim-core/tests/d4_lineages.rs` | §6 UT |

## 10. 未決事項（claude 裁定依頼）

- **D4-Q1**: アカバエの mortality_threshold。v0.4 付録 C は代償欄に「死亡閾値低」とあるが、機械定義（`biomass < threshold → Dying`）では閾値が低いほど死亡しにくく、代償として機能しない。推奨: 代償の意図どおり高く（10_000、他系統の 2 倍）設定し「急減」のドラマと整合させる
- **D4-Q2**: density_bonus の所持系統と機械的効果。用語集は「高密度集積で有利」と定義するが、v0.4 付録 C 上どの系統の特徴にも対応せず（シロナミは低密度に強い＝逆）、効果の式も未定義。推奨: 検証版では全系統 0・効果なしの予約ビットとし、ニッチは 5 軸と他タグで表現する（REQ-GEN-06 のニッチ条件は D7 で検証）
- **D4-Q3**: 離散アレルの適用単位。BD-07 §3 は mutation 消費を「配置セル × 系統ごとに 1 回」とするが、LineageParams は系統集約（BD-03）でセル別の特性を保持できず、REQ-GEN-01「それ以外の系統固有パラメータを持たない」と衝突する。推奨: 系統ごと 1 セット（消費 = 系統数 × 1 語）とし、BD-07 §3 create 行を改訂する
