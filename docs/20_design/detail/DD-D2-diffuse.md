# DD-D2 詳細設計: 64×64 拡散（diffuse phase）

- 版: 0.1（起草 cursor-kimi、2026-08-30）。上位正本: BD-05 §2/§4/§5、BD-06 §3、BD-07 §1/§3、契約 §7（movement 軸の D2 確定）
- スコープ: 64×64 格子への一般化と diffuse phase の実装。他 6 phase は D1 の 1 セル実装を格子対応に拡張するだけで、振る舞いの新規定義は行わない

## 1. diffuse の意味論（確定）

- 対象プール: `nutrient` / `carcass` / `waste`（環境プール）と `biomass[L]`。`energy` と `occupancy_peak` は拡散しない。参照: 契約 §2-1, REQ-SIM-10
- **biomass の移動**: movement 軸の値を「生体量の近傍拡散率」として用いる（契約 §7 の D0 open issue をここで確定）。系統 L のセルから近傍 i への送出係数 = `movement_L`（Fixed）。移動した biomass は系統・energy を伴わない（energy はセル内予算のまま。移動先での energy 配分は D3 の intake/maintenance で調整される）。参照: REQ-SIM-08
- 環境プールの係数: 環境レコード `diffusion_coefficients[pool]`（初期仮説 0.05/近傍/tick、D2 実測で確定）。参照: REQ-SIM-10

## 2. アルゴリズム（2 パス。確定）

1. **計算パス**: 全セルを row-major に走査し、セル・プール・近傍（北・東・南・西の固定順、境界では存在する近傍のみ）ごとに送出量 `out = floor(pool × coeff / scale)` を i128 中間・ゼロ方向丸めで計算し、送出バッファに蓄積する。読み取りは tick 開始時の状態から行う（in-place 更新禁止。in-place だと後続セルの送出量が先行セルの移転で変わり、物理的に非対称になる）
2. **適用パス**: 全送出を加算し、送り元からは `Σ送出` を減じる。残余（`pool − Σ送出`）は送り元に残る（INV-03、余りを捨てない）
3. ビット幅: 最大中間値は P1（2×10^14 × 5×10^4 = 10^19、64 bit、i128）と P2（送出合計 + 残余 ≤ 2×10^14、48 bit）。BD-06 §3 の証明表どおり。参照: REQ-SIM-13, REQ-SIM-14
- 台帳: Diffusion の LedgerEntry は `from_cell → to_cell` ごとに生成する（reason = Diffusion）。**審査案件（論点 D2-Q1）**: 生エントリは最大 4,096 セル × 4 近傍 × 4 pool × 2,000 tick ≈ 1.3 億件で常駐メモリ 32 MB（REQ-NFR-02）を超える。提案: コア内で tick 終了ごとに region 単位（BD-12 §1）へオンライン集約し、セル単位エントリは保持しない。台帳ダイジェスト（BD-01 r3 §5 が正本）の入力が集約レコードになるため、BD-03/BD-05 の「全変換が LedgerEntry を生成」との整合をレビューで確認すること。矛盾と判定されたら RFC で正本を直してから実装する

## 3. 境界値・エラー

- 境界セル: 近傍数 2（角）/ 3（辺）。係数は存在する近傍にのみ適用し、不存在近傍分は送出しない（境界流出なし。契約 §9 必須テスト）
- 一様場: 全セル同値なら送出が相殺され状態不変（契約 §9 必須テスト）
- `pool < Σ送出` は丸め方向（ゼロ方向）により発生しない。発生したら `NumericError`（状態不変）。参照: REQ-SIM-13
- coeff > scale（1.0 超の拡散率）は config 検証で拒否（config schema の range。DD-D5 で確定）

## 4. UT 設計（DD 列。実装は composer）

| UT-ID | 内容 | 期待 | 参照 |
|---|---|---|---|
| UT-D2-01 | 2 セル、nutrient のみ、係数 0.05 | 送出 = floor(pool×0.05)、残余は送り元 | REQ-SIM-10 |
| UT-D2-02 | 角セル（近傍 2）・辺セル（近傍 3） | 不存在近傍へ送出なし、総量保存 | 契約 §9 |
| UT-D2-03 | 一様場 64×64、2,000 tick | 状態不変（hash 不変） | 契約 §9 |
| UT-D2-04 | 係数 0 のプール | 当該プール不変 | REQ-SIM-10 |
| UT-D2-05 | biomass 移動（movement_L 差のある 2 系統） | 移動量が movement 軸どおり、energy は不変 | 契約 §7 |
| UT-D2-06 | 上限: pool = M_max = 2×10^14 で送出計算 | NumericError なし（i128 中間） | BD-06 P1/P2 |
| UT-D2-07 | 余り非消失: 奇数値・小係数で残余が送り元に残る | Σ = 入力（厳密） | REQ-SIM-05 |

## 5. AT(red)（先に failing で書く。BD-08 の ID と一致）

| AT-ID | 内容 | red 条件 |
|---|---|---|
| AT-D2-01 | 64×64 閉鎖系 2,000 tick の総質量厳密一致 | `verify --suite D2` に conservation_64x64 が無いと fail |
| AT-D2-02 | 左右反転 config で鏡像一致 | 同上（symmetry ケース） |
| AT-D2-03 | CI ubuntu / windows で hash 一致 | xos ジョブ（Codex と連携） |
| AT-D1-06 拡張 | 64×64 代表 config の golden hash を新規 commit（Claude 承認） | golden 未存在で fail |

## 6. 性能

- 拡散のみ 2,000 tick ≤ 200 ms（REQ-NFR-01、基準 PC）。criterion ベンチを `crates/sim-core/benches/diffuse.rs` に追加。確定
