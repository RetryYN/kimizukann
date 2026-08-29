# [D3-lineage-001][brief] 複数系統の intake/maintenance/reproduction（writer=cursor-grok, review=cursor-kimi(契約)+Claude(チェックリスト), gate=Claude）

前提: DD `docs/20_design/detail/DD-D3-lineage.md`（本 PR で追加）を正本とし、BD-04（状態機械）・BD-05（契約）・BD-06（数値）・BD-07（決定性）に矛盾する実装をしない。矛盾を見つけたら実装せず `[D3-lineage-001][question]` を cursor-kimi に直接投げる（NETWORK 規則: grok→kimi=[question]）。
環境: `cargo test --workspace` が通ることが提出条件。lint はリポジトリルートの `clippy.toml` + crate 先頭の `#![deny(clippy::float_arithmetic)]`（BD-07 §4.2）。
依存: **D2（cursor-grok、grid/diffuse/ledger 基盤）のマージ後に着手すること**。D2 未マージの間は DD の熟読と UT 設計の確認に留める。

## スコープ（300〜500 行）
1. `crates/sim-core/src/lineage_phases.rs` を新設し、系統に作用する 5 phase（intake / maintenance / starvation_and_death / reproduction / emission）の複数系統意味論を DD §1〜§6 どおり実装する（セル row-major × 系統 ID 昇順の逐次。按分しない）
2. intake: 基質一般化（use_nutrient/use_carcass/use_waste）、上限式 `min(pool, base_intake × intake 倍率)`、係数配分（nutrient 0.70/0.30、carcass・waste 0.50/0.50 の初期仮説）、energy 飽和と熱散逸の台帳記録
3. maintenance: toxin ×1.4、不足分 `cost − energy` の記録と Starving 遷移（BD-04 §3.2 の表どおり）
4. starvation_and_death: Starvation（不足分だけ carcass 化）/ Death（閾値未満で全量 carcass 化、同 tick に Absent）
5. reproduction: ガード `energy > 2×cost`、成立 (cell, lineage) ごとに reproduction ストリーム 1 語消費の抽選（p_repro = 1.0 の初期仮説）、質量は nutrient から同量移動（係数 1.0）
6. 台帳: 全変換で LedgerEntry を生成し、D2 の ledger 基盤の集約 API に渡す（LedgerRecord への集約・digest 自体は D2 の責務。**region の機械定義は論点 D3-Q1 で claude 判定中。判定まで集約キーの region_id 部分は実装しない**）
7. **UT/property を先に書く**: DD §9 の 16 件を failing テストとして commit してから実装する（テスト commit が実装 commit より前）
8. model_version を `d3-v1` に bump（繁殖抽選の乱数消費追加のため。BD-05 §14）。golden hash の更新は行わない（Claude 承認・別 PR）

## 触ってよいファイル（one-file-one-writer。D2/D3 は同一実装者だが PR は分ける）
`crates/sim-core/src/lineage_phases.rs`（新規）, `crates/sim-core/src/lib.rs`（`mod lineage_phases;` 追加と tick_once の phase 呼出差替えのみ）, `crates/sim-core/tests/d3_*.rs`（新規）, `Cargo.lock`。
**触らない**: `crates/sim-core/src/grid.rs` / `diffuse.rs` / `ledger.rs`（D2 PR の管轄）, `crates/sim-cli/**`, `docs/**`, `clippy.toml`, golden。

## 審査案件（論点 D3-Q1 / D3-Q2）
- D3-Q1: region の機械定義（何の連結成分か・採番順・16 超過時）が正本にない。DD §7 に提案を記載。claude 判定が出るまで region_id 絡みは実装しない
- D3-Q2: 繁殖抽選の導入（p_repro = 1.0 の初期仮説）。DD §5 に記載。claude 判定済みならそのまま実装

## 提出
`cargo test --workspace` の出力要約を添えて `[D3-lineage-001][result] status=pass commit=<hash> tests=<n>` を post。レビュー観点（kimi）: 決定性（走査順・消費回数表との一致）・保存則・状態機械（BD-04 §3.2）との一致・DD 逸脱のみ。
