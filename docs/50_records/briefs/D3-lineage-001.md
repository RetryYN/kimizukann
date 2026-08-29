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
6. 台帳: 全変換で LedgerEntry を生成し、D2 の ledger 基盤の集約 API に渡す（LedgerRecord への集約・digest 自体は D2 の責務。region_id は **静的 4×4 タイル** `ID = (row/16)*4 + (col/16)` で確定済み = 論点 D3-Q1 判定 r2）
7. **UT/property を先に書く**: DD §9 の 16 件を failing テストとして commit してから実装する（テスト commit が実装 commit より前）
8. model_version を `d3-v1` に bump（繁殖抽選の乱数消費追加のため。BD-05 §14）。golden hash の更新は行わない（Claude 承認・別 PR）

## 触ってよいファイル（one-file-one-writer。D2/D3 は同一実装者だが PR は分ける）
`crates/sim-core/src/lineage_phases.rs`（新規）, `crates/sim-core/src/lib.rs`（`mod lineage_phases;` 追加と tick_once の phase 呼出差替えのみ）, `crates/sim-core/tests/d3_*.rs`（新規）, `Cargo.lock`。
**触らない**: `crates/sim-core/src/grid.rs` / `diffuse.rs` / `ledger.rs`（D2 PR の管轄）, `crates/sim-cli/**`, `docs/**`, `clippy.toml`, golden。

## 論点（claude 判定済み）
- D3-Q1（判定 r2）: region は二層。台帳 LedgerRecord.region_id = 静的 4×4 タイル（row-major 0..=15、tick をまたぎ安定）。スタンプの region_ids = イベント tick の占有マスクの 4 連結成分（説明器が派生計算、保存は stamp 内のみ、LedgerRecord には書かない）。DD §7
- D3-Q2（採用）: 繁殖抽選 p_repro = 1.0 の初期仮説。ガード成立ごとに必ず 1 語消費（スキップ禁止）。model_version d3-v1、golden は別 PR（Claude 承認）。DD §5

## 提出
`cargo test --workspace` の出力要約を添えて `[D3-lineage-001][result] status=pass commit=<hash> tests=<n>` を post。レビュー観点（kimi）: 決定性（走査順・消費回数表との一致）・保存則・状態機械（BD-04 §3.2）との一致・DD 逸脱のみ。
