# [D2-diffuse-001][brief] 64×64 拡散（writer=cursor-grok, review=cursor-kimi(契約)+Claude(チェックリスト), gate=Claude）

前提: DD `docs/20_design/detail/DD-D2-diffuse.md`（本 PR で追加）を正本とし、BD-05（契約）・BD-06（数値）・BD-07（決定性）に矛盾する実装をしない。矛盾を見つけたら実装せず `[D2-diffuse-001][question]` を cursor-kimi に直接投げる（NETWORK 規則: grok→kimi=[question]）。
環境: `cargo test --workspace` が通ることが提出条件。lint はリポジトリルートの `clippy.toml` + crate 先頭の `#![deny(clippy::float_arithmetic)]`（BD-07 §4.2）。

## PR 分割（各 ≤ 300 行。確定）
- **PR-A**: 1×1 実装の 64×64 格子一般化（走査 row-major・系統 ID 昇順・近傍は北東南西）+ UT（DD §4 のうち格子一般化分）+ AT(red)。diffuse 本体は含めない
- **PR-B**: diffuse phase 本体（DD §1/§2 の 2 パス）+ 残りの UT + criterion ベンチ
- **golden は commit しない**（AT-D1-06 拡張の 64×64 golden hash は Claude 承認後に別 PR）

## スコープ（300〜500 行）
1. `crates/sim-core`: 1×1 実装を 64×64（契約上限 65535² の型で）に一般化。走査は row-major・系統 ID 昇順・近傍は北東南西（BD-05 §2）
2. diffuse phase を DD §1/§2 どおり実装（2 パス、対象プール nutrient/carcass/waste/biomass、movement 軸 = 生体量の近傍拡散率、energy は拡散しない）
3. **AT(red) を先に書く**: DD §5 の 4 件を failing テストとして commit してから実装する（テスト commit が実装 commit より前）
4. DD §4 の UT 7 件を `crates/sim-core/tests/` に追加
5. `sim-cli verify --suite D2`: conservation_64x64（AT-D2-01）・symmetry（AT-D2-02）を JSON レポートで出力、失敗時非 0 終了（REQ-OPS-01）
6. criterion ベンチ `diffuse.rs`（拡散のみ 2,000 tick ≤ 200 ms、REQ-NFR-01）
7. golden: 64×64 代表 config の state hash を `docs/30_contracts/golden/` に置く PR は分ける（golden 更新は Claude 承認が必要なため。**実装 PR には含めない**）

## 触ってよいファイル（one-file-one-writer）
`crates/sim-core/**`, `crates/sim-cli/**`, `crates/sim-types/src/lib.rs`（型の拡張のみ）, `Cargo.lock`。`docs/**`・`clippy.toml`・golden は触らない。

## 論点 D2-Q1（claude 判定済み・採用）
diffuse の台帳エントリ量が常駐メモリ 32 MB を超えるため、DD §2 のとおり「tick 終了ごとに region へオンライン集約しセル単位は保持しない」で**確定**。正本は BD-01 r4 §5 の二段モデル（発生時 LedgerEntry = cell_index 粒度 / digest・LedgerSave = region 集約 LedgerRecord、キー tick→region_id→lineage→reason→from→to、amount = 和、集約は tick 終了時に決定的順序）。region_id は **静的 4×4 タイル** `ID = (row/16)*4 + (col/16)`（16×16 セル、row-major 0..=15）で確定済み（論点 D3-Q1 判定 r2。スタンプの動的 4 連結成分とは別層で、台帳には静的タイルのみ使う）。

## 提出
`cargo test --workspace` と `cargo run -p kimizukann-sim-cli -- verify --suite D2` の出力要約を添えて `[D2-diffuse-001][result] status=pass commit=<hash> tests=<n> verify=<pass|fail>` を post。レビュー観点（kimi=契約 / Claude=チェックリスト）: 決定性（走査順・2 パス・PRNG 非消費）・保存則・DD 逸脱のみ。
