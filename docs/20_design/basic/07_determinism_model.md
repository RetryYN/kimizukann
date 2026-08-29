# BD-07 決定性モデル

- 版: 0.2（起草 cursor-kimi、2026-08-30。PR #1 レビュー r1 反映: mutation を初期配置ばらつき専用・interaction を初期配置位置抽選に割当、「予備」禁止を明記）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）、BD-05（契約）§6・§10
- 完成条件: 消費回数表が UT で検査できる。lint 設定が CI にある
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する。数値の上限は BD-06、hash 正規化は BD-05 §10 を参照

## 1. 走査順

全ての反復は以下の固定順とする。確定。参照: REQ-DET-04c

| # | 対象 | 順序 | 参照 |
|---|---|---|---|
| S1 | セル | row-major（`index = y × width + x`、index 昇順） | REQ-DET-04c |
| S2 | 系統 | ID 昇順（按分しない逐次処理） | REQ-SIM-11, REQ-DET-04c |
| S3 | 近傍 | 北・東・南・西の固定順 | REQ-SIM-10 |
| S4 | inflow 適用 | tick 先頭（diffuse 直前）に適用し、7 phase には含めない。tick 昇順・同 tick は config 配列の出現順 | REQ-SIM-07, REQ-DET-04c |
| S5 | 終了判定 | 優先順 Extinct > Fixed > Coexist > Reversal > TimeLimit | REQ-END-04c |
| S6 | 並列化 | seed 内は単一スレッド。seed 間バッチのみ並列。並列化しても同一結果になる決定的リダクションのみ許可 | REQ-DET-04c |

## 2. PRNG と 4 ストリームの用途割当

- PRNG: `SplitMix64` で seed から 4 ストリームの初期状態を導出し、各ストリームは xoshiro256**。バージョン文字列 `prng=xoshiro256ss-v1`。確定。参照: REQ-DET-04a
- ストリーム導出: `SplitMix64(seed)` の連続出力から各ストリームの `[u64; 4]` 状態を movement → reproduction → mutation → interaction の順に 4 語ずつ充填する。確定（導出順を変えると hash が変わるため model_version の一部）。参照: REQ-DET-04a, REQ-DET-05

| ストリーム | 用途 | 使用 phase / 処理 | 確定度 | 参照 |
|---|---|---|---|---|
| movement | 生体量の近傍拡散の tie-break・移動先抽選 | diffuse（D2 で導入） | 初期仮説（D2 で確定） | REQ-SIM-10 |
| reproduction | 繁殖の成立抽選（ガード成立 (cell, lineage) ごとに 1 語、`u / 2^64 < p_repro` で成立判定。p_repro = 1.0 は初期仮説） | reproduction（D3 で導入） | 確定（DD-D3 §5。構造は確定、p_repro の値は D3 較正で確定） | REQ-SIM-12 |
| mutation | 初期配置の離散アレル分布（ばらつきレバー）専用。検証版で変異の計算侵入はなく、tick 中の消費は常に 0 | create 時の初期配置のみ | 初期仮説（D4 で確定） | REQ-GEN-08, REQ-OUT-01 |
| interaction | 初期配置の位置抽選（配置レバーのランダム配置）専用。tick 中の消費は常に 0 | create 時の初期配置のみ | 初期仮説（D4 で確定） | REQ-SCOPE-03 |

- 4 ストリーム全てに用途が割り当てられており「予備」は存在しない。UI・Explain がストリームを借りることは禁止（BD-01 r2 F-04）。確定。参照: REQ-DET-04b, REQ-OUT-04

- 表示用サンプリング・UI・転換点検出はコア乱数を消費しない（別途 UI 側の乱数を使うか、決定的な間引き）。確定。参照: REQ-VIS-03, REQ-EVT-05, REQ-DET-05

## 3. 1 tick の乱数消費回数表

消費回数は**状態のみの関数**とし、乱数値に依存する分岐で消費回数が変わってはならない（変わると三経路一致が壊れる）。各セルは「消費 0」を含め全 phase が埋まっている。確定（D1 時点）。参照: REQ-DET-04b

| phase | movement | reproduction | mutation | interaction | 備考 |
|---|---:|---:|---:|---:|---|
| diffuse | 0 | 0 | 0 | 0 | D1 は決定的拡散。movement 導入後は「移動判定が発生した (cell, lineage) ごとに 1 回」（D2 で確定） |
| intake | 0 | 0 | 0 | 0 | 逐次処理・按分なし（REQ-SIM-11） |
| maintenance | 0 | 0 | 0 | 0 | 決定的課金 |
| starvation_and_death | 0 | 0 | 0 | 0 | 決定的移行（REQ-SIM-09） |
| reproduction | 0 | ガード成立 (cell, lineage) ごとに 1 回 | 0 | 0 | D3 で抽選導入（DD-D3 §5）。ガード `energy > 2×cost` は状態のみの関数で、乱数値に依らず消費回数が決まる。ガード不成立なら消費 0 |
| emission | 0 | 0 | 0 | 0 | 決定的排出 |
| occupancy | 0 | 0 | 0 | 0 | 決定的更新（REQ-SIM-03a） |
| create（初期配置） | 0 | 0 | 配置セル × 系統ごとに 1 回（アレル抽選） | 配置セル × 系統ごとに 1 回（位置抽選） | ばらつきレバー・ランダム配置（D4 で確定）。tick 進行中は全ストリーム 0 |
| **1 tick 合計（D1）** | **0** | **0** | **0** | **0** | D1 の hash は PRNG 初期状態のまま進行する |

- 消費回数の UT: 各ストリームに消費カウンタを付け、代表 config で `step(1)` 前後のカウンタ差分が本表と一致することを検査する。表の更新（D2/D3/D4 での確定）は UT の期待値更新を伴う。確定。参照: REQ-DET-04b
- 表に無い乱数消費は禁止。新たな消費は本表の RFC 改訂を伴う。確定。参照: REQ-DET-04b

FFI 操作ごとの消費回数（BD-05 §12.1 と一致。確定）:

| FFI 操作 | movement | reproduction | mutation | interaction | 備考 |
|---|---:|---:|---:|---:|---|
| create | 0 | 0 | 初期配置のアレル抽選分（§3 create 行） | 初期配置の位置抽選分（§3 create 行） | SplitMix64 による 4 ストリーム初期化は消費に含めない |
| load | 0 | 0 | 0 | 0 | PRNG 状態を復元するのみ |
| step(n) | phase 割当分 × n | 同左 | 0 | 0 | 唯一ストリームを進める操作 |
| snapshot | 0 | 0 | 0 | 0 | 読み取りのみ |
| explain | 0 | 0 | 0 | 0 | 純関数 |
| save | 0 | 0 | 0 | 0 | 読み取りのみ |
| destroy | 0 | 0 | 0 | 0 | — |

- `step(n)` が途中で終了（Terminated 遷移）した場合、残り tick 分の乱数は消費しない（消費は実際に実行した tick のみ）。確定。参照: REQ-DET-04b

## 4. 禁止構造と lint 設定

状態更新経路で反復順序が不定な構造と浮動小数点を使わない。確定。参照: REQ-DET-04d, REQ-CON-02

### 4.1 禁止対象

| 禁止対象 | 理由 | 代替 | 参照 |
|---|---|---|---|
| `std::collections::HashMap` / `HashSet`（`hashbrown` / `rustc_hash::FxHashMap` 等の亜種を含む） | 反復順序が実行ごとに不定 | `BTreeMap` / `BTreeSet` または `Vec` | REQ-DET-04d |
| `f32` / `f64`（sim-types / sim-core の状態更新経路） | 丸めがプラットフォーム依存になりうる | `Fixed`（i64 固定小数点） | REQ-CON-02 |
| wall clock・`SystemTime` / `Instant` | 時刻依存は再現性を壊す | `step(n)` のみ | REQ-CON-05 |
| スレッド乱数・`rand::thread_rng` | seed 非依存の乱数 | `PrngState` の 4 ストリーム | REQ-DET-04a |
| 浮動小数点の例外: 解析（z-score 等）は許可するがコア状態へ戻さない | — | 解析 crate に隔離 | REQ-CON-02, REQ-EVT-01 |

### 4.2 lint 設定（CI の `lint` job で強制）

`clippy.toml`（リポジトリルートに本 PR で追加。依存経由の穿過も塞ぐ）:

```toml
# 反復順序が不定な構造の禁止。hashbrown 経由の穿過も塞ぐ。参照: REQ-DET-04d
disallowed-types = [
  "std::collections::HashMap",
  "std::collections::HashSet",
  "std::collections::hash_map::RandomState",
  "hashbrown::HashMap",
  "hashbrown::HashSet",
  "rustc_hash::FxHashMap",
  "rustc_hash::FxHashSet",
  "rand::rngs::OsRng",
]
# wall clock・スレッド乱数・OS 乱数の禁止。参照: REQ-CON-05, REQ-DET-04a
disallowed-methods = [
  { path = "std::time::SystemTime::now" },
  { path = "std::time::Instant::now" },
  { path = "std::time::Instant::elapsed" },
  { path = "rand::thread_rng" },
  { path = "rand::rngs::OsRng::fill_bytes" },
  { path = "getrandom::getrandom" },
]
```

crate レベル（`crates/sim-types/src/lib.rs`、`crates/sim-core/src/lib.rs` の先頭）:

```rust
#![deny(clippy::float_arithmetic)] // 状態更新経路の浮動小数点演算を禁止。参照: REQ-CON-02
```

- `clippy.toml` はワークスペースルートに置くため全 crate に自動適用される。将来新設する `sim-ffi` / `sim-cli` / `sim-explain` にも同じく適用され、各 crate の `lib.rs` / `main.rs` 先頭にも同じ `#![deny(clippy::float_arithmetic)]` を付ける（crate 作成 PR のレビュー条件）。確定。参照: REQ-DET-04d, REQ-CON-02

- CI の `lint` job は `cargo fmt --check` + `cargo clippy --workspace -- -D warnings` + 上記設定を実行する。確定。参照: REQ-DET-04d, REQ-CON-02
- 解析 crate（転換点スコア等）は `float_arithmetic` の対象外とし、コアへの逆流が無いことを INSP で検査する。確定。参照: REQ-CON-02, REQ-EVT-01

## 5. 三経路一致・クロス OS の検証手順

### 5.1 三経路一致（確定。参照: REQ-DET-02）

同一 model_version・config・seed について以下の 3 経路が同一の state hash・終了ラベルを返すことを AT で検査する:

1. 経路 A: `step(2000)` 一括
2. 経路 B: `step(1)` × 2,000 回
3. 経路 C: 任意 tick t で `save` → `load` → 残り `step(2000 − t)`。t は代表 tick = 1,000 と、ランダム tick 数点（検証側で固定した seed による抽選）を使う

手順: `sim-cli verify --suite <Dn>` が 3 経路を実行し、hash・終了ラベルの一致を JSON レポートに出力、不一致で非 0 終了。参照: REQ-DET-02, REQ-OPS-01

### 5.2 同一 seed 反復（確定。参照: REQ-DET-01）

- 同一 config・seed で 100 回実行し、state hash が 100 回一致することを AT で検査する

### 5.3 クロス OS（確定。参照: REQ-DET-03）

- CI の `determinism-xos` job: ubuntu と windows で同一入力の state hash を比較し一致を必須チェックとする
- Android arm64 は実機 MEAS で同一入力の hash を CI 産物と照合する（D12）
- 不一致時の調査順: (1) 浮動小数点の混入（§4 の lint 回避）、(2) HashMap 等の反復順、(3) 未初期化メモリ・依存クレートの OS 差、(4) エンディアン（正規化バイト列はリトルエンディアン固定とする。確定。参照: REQ-DET-05）

### 5.4 非干渉の検査（確定）

- 再生速度（1 / 4 / 16 tick/s）・一時停止・描画間引き・表示用トークンの有無・seed 間並列数を変えた実行で、最終 hash・終了ラベル・イベント列が一致することを AT で検査する。参照: REQ-DET-07, REQ-OUT-04, REQ-VIS-04, REQ-EVT-05

## 6. ADR 候補（REQ に無い設計判断）

- ADR 候補 1: 正規化バイト列のエンディアンをリトルエンディアン固定とする（REQ-DET-05 はバイト列構成のみ規定し、エンディアンを明示しない）。参照: REQ-DET-05, REQ-DET-03
- ADR 候補 2: ストリーム導出の充填順（movement → reproduction → mutation → interaction）は契約 §6 の列挙順を採用した。参照: REQ-DET-04a
