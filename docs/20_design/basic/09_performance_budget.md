# BD-09 性能予算

- 版: 0.2（起草 claude、2026-08-30。r2: オーナー指示で床を 2021 年以降に更新）。上位正本: 要件定義書 v0.2 REQ-NFR-01/02/03、REQ-UI-04a/b、OPEN-03
- OPEN-03 の決定（オーナー 2026-08-30）: **「スマホ全部。ただし 2020 年以前の機種は不要」**。本章はこれを「2021 年以降の実売エントリー機を床に固定し、予算は床から導出する」と解釈する（§1）。床の機種スペックは Claude の提案値であり、オーナーが変更すれば数値だけ再導出する

## 1. 端末の定義（OPEN-03 クローズ）

| 名称 | 定義 | 代表スペック | 用途 |
|---|---|---|---|
| **最低対象端末（床）** | **2021 年以降**の実売エントリー機。Android 11 (API 30) 以上、iOS 15 以上（iPhone 13 / SE 第 3 世代以降） | CPU: Snapdragon 480 / 680、Helio G85 相当（Cortex-A76 級 ×2 + A55 ×6）、RAM 3 GB（アプリ利用可能 ≤ 1 GB）、GPU あり（Impeller） | REQ-UI-04b、REQ-NFR-02 の下限。**全予算の導出元** |
| **基準端末（ミドル）** | 2022 年以降のミドルレンジ | Snapdragon 7xx / Tensor G2 相当（Cortex-A78 級 ×1–4 + A55）、RAM 6 GB | REQ-UI-04a、REQ-NFR-02 の p95 計測、テスター 1 台以上（REQ-USER-02） |
| **基準 PC** | GitHub Actions `ubuntu-latest` ランナー（4 vCPU）。CI しきい値の正本 | 変動を吸収するため予算は 3 回中央値。オーナー PC は参考値 | REQ-NFR-01/03 |

2020 年以前の機種（API 29 以下、RAM 2 GB 以下、iOS 14 以下）は**非対象**と明記し、ストア/配布ページに最低要件として記載する（REQ-NFR-08 の対外文リント対象）。

## 2. 予算（確定値。CI・実機計測のしきい値）

| ID | 項目 | 床（最低対象） | 基準端末 | 基準 PC | 導出 |
|---|---|---|---|---|---|
| PB-01 | 1 tick（64×64、4 系統、全 7 phase）p95 | ≤ 6 ms | ≤ 4 ms | ≤ 0.5 ms | REQ-NFR-02 の 4 ms を基準端末に置き、床（A76 級）は単コア性能比 ≈ 2/3 で 1.5 倍。PC は 2,000 tick ≤ 1.0 s から |
| PB-02 | 高速 16 tick/s の 2,000 tick 実再生時間 | ≤ 130 s | ≤ 128 s | — | REQ-UI-04b。tick 62.5 ms 枠に PB-01 + 描画 PB-04 が収まることが条件 |
| PB-03 | 標準速度 4 tick/s の 2,000 tick | — | ≤ 500 s（8 分 20 秒） | — | REQ-UI-04a。基準端末・OS 中断なし |
| PB-04 | フレーム p95（描画 + tick 実行を含む UI スレッド） | ≤ 16.7 ms（60 fps） | ≤ 16.7 ms | — | 床でも 60 fps。tick は UI スレッド外（isolate）で実行し、フレームは snapshot コピーのみ |
| PB-05 | 拡散のみ 2,000 tick | — | — | ≤ 200 ms | REQ-NFR-01（D2 で criterion） |
| PB-06 | headless 1 seed 2,000 tick | ≤ 12 s | ≤ 8 s | ≤ 1.0 s | REQ-NFR-01 |
| PB-07 | 常駐メモリ（アプリ全体、Flutter 込み） | ≤ 128 MB | ≤ 128 MB | — | 床のアプリ利用可能 1 GB に対し余裕。うち **SimCore ≤ 32 MB**（REQ-NFR-02、台帳は region 集約が前提 = BD-01 r4 §5） |
| PB-08 | 保存サイズ（SaveEnvelope） | ≤ 5 MB | 同左 | — | REQ-NFR-02、BD-10 |
| PB-09 | 起動〜開始画面 | ≤ 3 s | ≤ 2 s | — | 導出（テスター離脱防止） |
| PB-10 | 100 seed バッチ（D7） | — | — | ≤ 10 min | REQ-NFR-03 = 400 ラン × PB-06 × 1.5 余裕 |

## 3. 段階ごとの適用

| 段階 | 有効になる予算 | ゲート |
|---|---|---|
| D2 | PB-05, PB-06（PC） | criterion ベンチ、CI で中央値 > しきい値なら fail |
| D4 | PB-01（PC）, PB-07 SimCore 分 | `verify --suite all` の report.json に `tick_p95_us`, `rss_peak_bytes` を追加（Codex） |
| D7 | PB-10 | バッチジョブ実測 |
| D12 | PB-01..04, PB-07..09（実機） | 実機 MEAS。床 1 台 + 基準端末 1 台で計測し docs/50_records/ に記録。床が無ければ Android エミュレータ（cores=4, RAM 3 GB, x86 → 参考値扱い、合否には使わない） |

## 4. 計測方法

- PC: `criterion`（`crates/sim-core/benches/`）。warm-up 3 s、sample 20、報告値は中央値。CI しきい値は `crates/sim-core/benches/budget.toml` に PB-ID で持ち、ベンチが読む（1 ファイル 1 writer: Claude）
- 実機: Flutter `Timeline` + `SchedulerBinding.addTimingsCallback` で frame、Rust 側は tick ごとの経過 µs を ring buffer（256）に保持し p95 を snapshot に載せる（BD-05 §5 の `SnapshotView.perf` を D12 で追加）
- メモリ: Android `Debug.getPss()`、iOS `task_info` 相当。SimCore 単体は `verify` が report.json に自己申告

## 5. 予算超過時の縮退（順序固定）

1. 描画: 格子の描画間引き（毎 tick → 2 tick に 1 回）。ログ表示に明記
2. 高速: 16 tick/s → 8 tick/s に落とす（REQ-UI-04b は「床で 130 s」を満たせなくなるので、これはリリース前に直す欠陥として扱う）
3. シミュレーション本体の縮退はしない（決定論・科学性を守る。REQ-NFR-07）

## 6. トレース

REQ-NFR-01 → PB-05/06/10、REQ-NFR-02 → PB-01/04/07/08、REQ-NFR-03 → PB-10、REQ-UI-04a → PB-03、REQ-UI-04b → PB-02。AT: AT-D2-04（PB-05）、AT-D12-03/04（PB-02/03、BD-08）
