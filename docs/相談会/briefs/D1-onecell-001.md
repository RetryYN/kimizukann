# [D1-onecell-001][brief] 1 セル閉鎖系（writer=Codex, review=cursor-kimi, gate=Claude）

前提: D0 契約 `docs/contracts/simulation_contract.md`（commit 2ec7d6d）を正本とし、契約を変えない。契約に矛盾があれば実装せず `[D1-onecell-001][question]` で Claude に投げる。
環境: `rustup` 導入済み（rustc 1.98.0）。`cargo test --workspace` が通ることが提出条件。

## スコープ（300〜500 行）
1. `crates/sim-core`: 1×1 グリッド・inflow なし・系統 1〜2 で、契約 §2 の 7 phase を固定順に実装（diffuse は 1 セルでは no-op だが phase として存在させる）。
2. 固定小数点演算ヘルパ（i64/scale 1e6、i128 中間、ゼロ方向丸め、余りは主出力プールへ）を `sim-core::fixed` に置き、`NumericError` を返す。
3. PRNG: SplitMix64 → xoshiro256** 4 ストリーム（契約 §6）。D1 では消費しなくてよいが `RandomStream` 型を D0 の TODO どおり定義する。
4. `invariant_report()`: 質量保存（nutrient+Σbiomass+carcass+waste = 初期総量）、energy 0〜1、非負を実装。
5. state hash（SHA-256、契約 §10 正規化順）を `SimCore::state_hash()` で返す。
6. `sim-cli verify --suite week1`: 以下を JSON レポートで出力し、失敗時は非 0 終了。
   - conservation_1cell: 2000 tick 後に質量差 = 0（固定小数点で厳密 0）
   - determinism: 同一 config/seed の 2 経路（連続 step(2000) vs step(1000)×2）で state_hash 一致
   - nonneg: 全 tick で全プール ≥ 0
7. `crates/sim-core/tests/` にユニットテスト（fixed 丸め・余り、hash の安定値 1 件をゴールデンとして固定）。

## 触ってよいファイル（one-file-one-writer）
`crates/sim-core/**`, `crates/sim-cli/**`, `crates/sim-types/src/lib.rs`（D0 TODO の型を埋めるのみ）、`Cargo.lock`。`docs/**` は触らない。

## 提出
1 commit（複数可だが squash 不要）。`cargo test --workspace` と `cargo run -p sim-cli -- verify --suite week1` の出力要約を添えて
`[D1-onecell-001][result] status=pass commit=<hash> tests=<n> verify=<pass|fail>` を post。
レビュー観点（kimi）: 保存則・hash・契約逸脱のみ。見た目は対象外。
