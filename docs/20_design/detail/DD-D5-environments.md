# DD-D5 詳細設計: 4 環境プリセット（環境レコード・分布マップ・流入）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 上位正本: `docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-SCOPE-01、REQ-ENV-01..04、REQ-SIM-07/10）、`docs/00_product/第2回_統合案_v0.5.md` §1.1・§7.3、BD-03（InflowEvent・ConversionRule）、BD-05 §10/§13、BD-06、BD-08 §5（AT-D5-01..04）、DD-D2（拡散係数の適用）
- スコープ: 環境レコードの schema、4 環境プリセット JSON の内容、分布マップの表現と展開規則、初期物質総量の整合
- 非スコープ: 系統プリセット・初期配置レバーの機械定義（DD-D4）、較正と代表史の選定（D7）、地形札の UI 表現（D12）
- 依存: 初期配置の既定値は DD-D4 §3（PR #29）を参照する

## 1. 環境レコード（確定。REQ-ENV-01 の 7 フィールド）

| フィールド | 型 | 内容 |
|---|---|---|
| `environment_id` | enum 文字列 | `center_rich` / `edge_sparse` / `local_waste` / `carcass_pulse`（REQ-ENV-02） |
| `geometry_id` | 文字列 | 空間配置の識別子。検証版の 4 環境は全て `square_64x64`（障害物・無効セルなし）。地形の差は分布マップで表現する（v0.5 §7.3 の geometry/regime 分離は、regime を後続 4 フィールドへ分解することで実現する。`regime_id` フィールドは持たない。D5-Q2） |
| `initial_pool_totals` | `[Fixed; 4]` | Nutrient / Biomass / Carcass / Waste の初期総量。Biomass は常に 0（系統の初期生体量は `initial_state` 側。REQ-ENV-04） |
| `pool_distribution_maps` | patch リスト（§3） | 各セルへの初期配分。JSON は compact 表現、ローダが 64×64 へ決定的に展開する |
| `diffusion_coefficients` | `{ nutrient, carcass, waste }: Fixed` | プール別・近傍あたりの拡散係数。DD-D2 の一様係数モデルに渡す。4 環境とも 50_000（0.05）で統一（D5-Q1） |
| `inflow_tick_mask` | `InflowEvent[]` | 4 環境とも空（閉鎖系。D5-Q3）。流入ありの保存則検査は AT-D5-02 の別 fixture で行う |
| `expected_niche_tags` | 文字列配列 | その環境で有利と想定される機構タグ（REQ-GEN-06 の検査材料） |

## 2. 4 環境プリセット（初期仮説。総量は D7 較正で更新しうる）

全環境で初期物質総量（4 プール + 初期生体量を除く環境プールの合計）を **40_960_000_000（= 4,096 セル × 10.0 mass_u）** に揃える（REQ-ENV-03）。配分は環境ごとに異なる。

### 2.1 center_rich（中央の島）

- 思想: 中央に高密度の栄養。アオシキ（高摂取・低移動）のニッチ
- nutrient: 中央 16×16（rows/cols 24..39、256 セル）に 100.0 ずつ（計 25_600_000_000）、外縁 3,840 セルに 4.0 ずつ（計 15_360_000_000）
- carcass / waste: 0
- expected_niche_tags: `["use_nutrient"]`

### 2.2 edge_sparse（縁の輪）

- 思想: 外周 2 セル幅の輪に資源が偏り、内部は希薄。シロナミ（高移動）のニッチ
- nutrient: 輪（row < 2 または row ≥ 62 または col < 2 または col ≥ 62 の 496 セル）に計 37_360_000_000、内部 3,600 セルに 1.0 ずつ（計 3_600_000_000）
- carcass / waste: 0
- expected_niche_tags: `[]`（移動軸由来のニッチであり機構タグに対応しない）

### 2.3 local_waste（二つの池）

- 思想: 老廃物が 2 区画に滞留。use_waste（クロシデ）のニッチ。toxin_sensitive（アオシキ）へのペナルティ場
- nutrient: 全 4,096 セルに 9.5 ずつ（計 38_912_000_000）
- waste: 池 1（rows/cols 16..31、256 セル）に 4.0 ずつ、池 2（rows/cols 32..47、256 セル）に 4.0 ずつ（計 2_048_000_000）
- carcass: 0
- expected_niche_tags: `["use_waste"]`
- v0.4「片側の waste 拡散率を低く」は空間可変拡散を要求し DD-D2 の一様係数モデルと衝突するため D5-Q1（§9）

### 2.4 carcass_pulse（死骸の回廊）

- 思想: 初期物質の一部が死骸の帯として配置。use_carcass（クロシデ）のニッチ
- nutrient: 全 4,096 セルに 9.0 ずつ（計 36_864_000_000）
- carcass: 回廊（rows 30..33 の 4 行 × 64 列 = 256 セル）に 16.0 ずつ（計 4_096_000_000）
- waste: 0
- expected_niche_tags: `["use_carcass"]`

### 2.5 initial_state（REQ-ENV-04）

各環境 JSON は 4 系統の初期配置を `initial_state` として保持する。既定は DD-D4 §3.1 の default 配置（象限中心 (16,16) / (48,16) / (16,48) / (48,48)、id 順）・初期生体量 全系統 1.0。代表史の配置は生命史パターン ID で別管理（D7）

## 3. 分布マップの compact 表現と展開（確定）

JSON では patch のリストで記述し、ローダが 64×64 の 2 次元配列（row-major）へ展開する。4,096 セル × 4 プールの全要素列挙を避け、レビュー可能なサイズに保つための表現上の圧縮であり、メモリ上の `EnvironmentRecord` は展開済み配列を持つ（用語集の「2 次元マップ配列」と同物）。

- patch = `{ pool, rect: { r0, c0, r1, c1 }（両端含む）, total: Fixed }`
- 展開規則（決定的）:
  1. 全セル 0 で初期化
  2. patch を配列順に適用。各 patch は rect 内 n セルへ `q = total / n`（ゼロ方向丸め）を配り、先頭 `total − q×n` セル（row-major 順）に +1。**上書きではなく加算**とする
  3. 全 patch 適用後、プールごとの合計が `initial_pool_totals` と厳密一致することを検証し、不一致は拒否（ValidationError）
- rect の範囲外・`r0 > r1` 等の不正は拒否（ValidationError）

## 4. schema 要件

- `docs/30_contracts/environment.schema.json`（新設）: 7 フィールド必須、`additionalProperties = false`、Fixed は整数（scale 1e6 の生値）、enum・値域（係数 ∈ [0, 250_000] = 4 近傍合計 ≤ 1.0。BD-06 UT-N12b）を検査
- プリセット JSON は `docs/30_contracts/environments/{environment_id}.json` に 4 件
- チュートリアル config は `environment_id` で参照し、dangling 参照は拒否（REQ-SCOPE-01、AT-D5-01）

## 5. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D5-01 | 4 環境プリセット | 環境プール総量が全て 40_960_000_000 に厳密一致（REQ-ENV-03） |
| UT-D5-02 | patch `{ total: 10, n: 3 }` | 展開 = [4, 3, 3]（row-major で先頭から +1） |
| UT-D5-03 | 重複 rect の 2 patch（total 5 + 7） | 重複セル = 12（加算規則） |
| UT-D5-04 | patch 合計 ≠ initial_pool_totals | 拒否（ValidationError） |
| UT-D5-05 | rect 範囲外（r1 = 64） | 拒否（ValidationError） |
| UT-D5-06 | 4 環境で load → save → load | state hash 一致（AT-D5-04 の UT 版。REQ-ENV-02） |
| UT-D5-07 | inflow fixture（tick 10 に nutrient +1_000_000） | 総質量 = 初期 + Σ inflow に厳密一致（AT-D5-02 引用。REQ-SIM-07、INV-02） |
| UT-D5-08 | 拡散係数 250_001 | 拒否（4 近傍合計 > 1.0。BD-06 UT-N12b） |

## 6. AT 対応（BD-08 §5）

| AT | 対応 |
|---|---|
| AT-D5-01 | §4 schema + 参照検査（UT-D5-05 系の拒否テストを引用） |
| AT-D5-02 | UT-D5-07 の inflow fixture を引用 |
| AT-D5-03 | §1 の 7 フィールド schema 検査 |
| AT-D5-04 | UT-D5-06 を 4 環境で実行 |

## 7. 性能

- patch 展開は create 時に 1 回、O(4096)。tick 中のコストなし
- 環境 JSON は 4 件とも patch 数 ≤ 4 で、1 件あたり 1 KB 未満（REQ-NFR-02 の保存予算に影響なし）

## 8. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-core/src/environment.rs` | レコード型・patch 展開・検証（§3） |
| `docs/30_contracts/environment.schema.json` | §4 schema |
| `docs/30_contracts/environments/*.json` | §2 の 4 プリセット |
| `crates/sim-core/tests/d5_environments.rs` | §5 UT |

## 9. 未決事項（claude 裁定依頼）

- **D5-Q1**: local_waste の「片側の waste 拡散率を低く」（v0.4）。DD-D2 は環境内一様係数であり、空間可変拡散は拡散ループの契約変更・性能再検証を伴う。推奨: 検証版では初期 waste 偏在のみで表現し（§2.3）、拡散係数は 4 環境とも 0.05 で統一。空間可変拡散はストア版へ送る
- **D5-Q2**: `regime_id` フィールドの有無。v0.5 §7.3 は geometry/regime 分離を求めるが、REQ-ENV-01（sign-off 済正本）の 7 フィールドに regime_id はなく、regime は pool_distribution_maps・diffusion_coefficients・inflow_tick_mask に分解済み。推奨: フィールドを追加しない（REQ どおり）
- **D5-Q3**: inflow を持つプリセットの有無。v0.5 §1.1 は流入プリセットの存在を示唆するが、4 環境の定義（§7.3）に流入の記述はない。推奨: 4 環境は全て閉鎖系とし、AT-D5-02 は合成 fixture で検査（流入系プリセットはストア版の論点）
