# BD-09 性能予算

- 版: 0.3（起草 claude、2026-08-30。r2: 床を 2021 年以降に更新。r3: grok 審査 7 major 反映 — スレッド模型固定、PB-07 を REQ どおり 32 MB、一巡 10 分、iOS 仮置き、トレース修正）。上位正本: 要件定義書 v0.2 REQ-NFR-01/02/03、REQ-UI-04a/b、OPEN-03
- OPEN-03 の決定（オーナー 2026-08-30）: **「スマホ全部。ただし 2020 年以前の機種は不要」**。本章はこれを「2021 年以降の実売エントリー機を床に固定し、予算は床から導出する」と解釈する（§1）。床の機種スペックは Claude の提案値であり、オーナーが変更すれば数値だけ再導出する

## 1. 端末の定義（OPEN-03 クローズ）

| 名称 | 定義 | 代表スペック | 用途 |
|---|---|---|---|
| **最低対象端末（床）** | **2021 年以降**の実売エントリー機。Android 11 (API 30) 以上。**検証版は Android のみ**（REQ-OPS-07 / OPEN-07）。iOS 15 以上（iPhone 13 / SE3 以降）は**仮置き**で、実機試験まで配布文に書かない | CPU: Snapdragon 480 / 680、Helio G85 相当（Cortex-A76 級 ×2 + A55 ×6）、RAM 3 GB（アプリ利用可能 ≤ 1 GB）、GPU あり（Impeller） | REQ-UI-04b、REQ-NFR-02 の下限。**全予算の導出元** |
| **基準端末（ミドル）** | 2022 年以降のミドルレンジ | Snapdragon 7xx / Tensor G2 相当（Cortex-A78 級 ×1–4 + A55）、RAM 6 GB | REQ-UI-04a、REQ-NFR-02 の p95 計測、テスター 1 台以上（REQ-USER-02） |
| **基準 PC** | GitHub Actions `ubuntu-latest` ランナー（4 vCPU）。CI しきい値の正本 | 変動を吸収するため予算は 3 回中央値。オーナー PC は参考値 | REQ-NFR-01/03 |

2020 年以前の機種（API 29 以下、RAM 2 GB 以下、iOS 14 以下）は**非対象**と明記し、ストア/配布ページに最低要件として記載する（REQ-NFR-08 の対外文リント対象）。

## 2. 予算（確定値。CI・実機計測のしきい値）

| ID | 項目 | 床（最低対象） | 基準端末 | 基準 PC | 導出 |
|---|---|---|---|---|---|
| PB-01 | 1 tick（64×64、4 系統、全 7 phase）p95。**sim isolate 上で計測** | ≤ 6 ms | ≤ 4 ms | ≤ 0.5 ms | REQ-NFR-02 の 4 ms を基準端末に置き、床（A76 級）は単コア性能比 ≈ 2/3 で 1.5 倍。PC は 2,000 tick ≤ 1.0 s から |
| PB-02 | 高速 16 tick/s の 2,000 tick 実再生時間 | ≤ 130 s | ≤ 128 s | — | REQ-UI-04b。§2.1 の模型により条件は **PB-01 ≤ 62.5 ms かつ PB-04 ≤ 16.7 ms を各々独立に**満たすこと（足し算しない） |
| PB-03 | 標準速度 4 tick/s の 2,000 tick 再生 | — | ≤ 500 s（8 分 20 秒） | — | REQ-UI-04a。2000/4 = 500 s ちょうど。tick 落ちが 0 であること（余裕は PB-03b 側で持つ） |
| PB-03b | 標準一巡（開始前操作 + PB-03 + 終了表示） | — | ≤ 600 s（10 分） | — | REQ-UI-04a。開始前操作 + 終了表示に 100 s 以内 |
| PB-04 | フレーム p95（**UI スレッドのみ**: snapshot コピー + 描画。tick を含まない） | ≤ 16.7 ms（60 fps） | ≤ 16.7 ms | — | REQ-NFR-02。床でも 60 fps |
| PB-05 | 拡散のみ 2,000 tick | — | — | ≤ 200 ms | REQ-NFR-01（D2 で criterion） |
| PB-06 | headless 1 seed 2,000 tick | ≤ 12 s | ≤ 8 s | ≤ 1.0 s | REQ-NFR-01 |
| PB-07 | 常駐メモリ（アプリ全体、REQ-NFR-02 の定義どおり） | ≤ 32 MB | ≤ 32 MB | — | **REQ-NFR-02 の値をそのまま採用**（本章で書き換えない）。内訳の初期仮説: SimCore ≤ 8 MB（state 64×64×4 系統 ≈ 1 MB、台帳 region 集約 = BD-01 r4 §5）、Flutter/描画 ≤ 24 MB。D12 実測で Flutter 分が超えるなら **RFC で REQ-NFR-02 を改定**する（黙って予算側を広げない） |
| PB-08 | 保存サイズ（SaveEnvelope） | ≤ 5 MB | 同左 | — | REQ-NFR-02、BD-10 |
| PB-09 | 起動〜開始画面 | ≤ 3 s | ≤ 2 s | — | 導出（テスター離脱防止） |
| PB-10 | 100 seed バッチ（D7） | — | — | ≤ 10 min | REQ-NFR-03 = 400 ラン × PB-06 × 1.5 余裕 |

## 3. 段階ごとの適用

| 段階 | 有効になる予算 | ゲート |
|---|---|---|
| D2 | PB-05, PB-06（PC） | criterion ベンチ、CI で中央値 > しきい値なら fail |
| D4 | PB-01（PC）, PB-07 SimCore 分 | `verify --suite all` の report.json に `tick_p95_us`, `rss_peak_bytes` を追加（Codex） |
| D7 | PB-10 | バッチジョブ実測 |
| D12 | PB-01..04（PB-03b 含む）, PB-07..09（実機） | 実機 MEAS。床 1 台 + 基準端末 1 台で計測し docs/50_records/ に記録。床実機が無い場合、Android エミュレータは参考値であり合否に使えない → **REQ-UI-04b はブロック**（オーナーに床実機 1 台を要請） | |

## 2.1 スレッド模型（確定）

- SimCore の `step(n)` は **sim isolate** で実行し、UI isolate は毎フレーム `SnapshotView` のコピーを受け取って描画するだけ。tick と描画は互いにブロックしない（BD-11 §2 の固定時間刻みと整合）
- したがって PB-01（tick）と PB-04（フレーム）は**独立に**判定し、合算しない。高速 16 tick/s の成立条件は PB-01 ≤ 62.5 ms（tick 落ちなし）

## 4. 計測方法

- PC: `criterion`（`crates/sim-core/benches/`）。warm-up 3 s、sample 20、報告値は中央値。しきい値の**正本は本章の表**。`crates/sim-core/benches/budget.toml` は実装時に kimi（crates 管轄）が本章から転記し、値の変更は本章の PR が先
- 実機: Flutter `Timeline` + `SchedulerBinding.addTimingsCallback` で frame、Rust 側は tick ごとの経過 µs を ring buffer（256）に保持し p95 を snapshot に載せる（`SnapshotView.perf` は BD-05 に未定義。D12 で BD-05 §5 に追加する契約変更として先送り）
- メモリ: Android `Debug.getPss()`、iOS `task_info` 相当。SimCore 単体は `verify` が report.json に自己申告

## 5. 予算超過時の縮退（順序固定）

1. 描画: 格子の描画間引き（毎 tick → 2 tick に 1 回）。ログ表示に明記
2. 高速: 16 tick/s → 4 tick/s に一段下げ（製品速度は 1/4/16 のみ。BD-11 §2）。REQ-UI-04b を満たせないので、これはリリース前に直す欠陥として扱う
3. シミュレーション本体の縮退はしない（決定論・科学性を守る。REQ-NFR-07）

## 6. トレース

| REQ | PB | 検証 |
|---|---|---|
| REQ-NFR-01 | PB-05, PB-06, PB-10 | MEAS（criterion、CI しきい値。BD-08 に AT なし） |
| REQ-NFR-02 | PB-01, PB-04, PB-07, PB-08 | MEAS（実機、D12） |
| REQ-NFR-03 | PB-10 | MEAS（D7 バッチ） |
| REQ-UI-04a | PB-03, PB-03b | MEAS（実機、基準端末） |
| REQ-UI-04b | PB-02 | MEAS（実機、床） |

AT-D12-03/04 は権限・wall-clock lint であり本章の対象外。
