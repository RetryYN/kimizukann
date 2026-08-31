# [DD-D2-occupancy-001][brief] θ_occ / occupancy を DD-D2 に固定する（起草 cursor-kimi、審査 cursor-grok、gate Claude）

前提: `docs/20_design/detail/**` の writer は kimi。本 brief は grok の BD↔DD 照合結果と、kimi が DD-D2 へ貼る追記案。detail 本文は本 PR では触らない。

照合基準: `origin/main` `2a32bd63758bc10d609c88ed6d40a61dceae3d13`。

## 照合結果（D2〜D12）

| 段階 | DD | 版 | 判定 |
|---|---|---|---|
| D0 / D1 | ファイルなし | — | 対象外（BD README: DD は BD-08 後に `DD-D2-*` から） |
| D2 | `DD-D2-diffuse.md` | 0.1 | **不完全**: 拡散は確定。BD-03/04 が「D2 で確定」とした θ_occ と occupancy phase の格子適用が無い |
| D3 | `DD-D3-lineage.md` | 0.3 | 完了。テスト既定 `occupancy_threshold = 1_000_000` はあるが、D2 確定値の正本ではない |
| D4 | `DD-D4-lineages.md` | 0.2 | 完了。AT-D4-04 は「空き家判定は D2（occupancy）依存」と明示 |
| D5 | `DD-D5-environments.md` | 0.2 | 完了 |
| D6 | `DD-D6-smoke.md` | 0.2 | 完了 |
| D7 | `DD-D7-calibration.md` | 0.2 | 完了 |
| D8 | `DD-D8-persistence.md` | 0.2 | 完了 |
| D9 | `DD-D9-explainer.md` | 0.2 | 完了（BD-12「詳細設計で固定」の集計） |
| D10 | `DD-D10-presentation.md` | 0.2 | 完了 |
| D11 | `DD-D11-ffi.md` | 0.2 | 完了 |
| D12 | `DD-D12-ui.md` | 0.2 | 完了（BD-11 の 50 tick save は 60 秒周期に置換済み） |

残 DD ファイル欠落: 0。残「不完全」: θ_occ / occupancy のみ。

別件（本 brief の対象外）: BD-09 の `SnapshotView.perf` は D12 で BD-05 へ契約追加する先送り。DD ではなく basic/契約。

## 現状の証拠

- BD-03: `Thresholds.occupancy_threshold`（θ_occ）「値は未定 / 初期仮説（D2 で確定）」
- BD-04 §2: 飽和 1.0・減衰 ×0.995・空き家判定線 0.3 は確定。θ_occ だけ「D2 で確定」
- DD-D2: occupancy_peak は拡散しない、とだけ。θ_occ の値も occupancy phase の走査も無い
- 実装既定（`crates/sim-core`）: `occupancy_threshold: FIXED_SCALE`（1_000_000）。更新則は `biomass_sum ≥ threshold → occupancy_peak = FIXED_SCALE`、否则 `mul(occupancy_peak, 995_000)`
- DD-D3 §9 共通 Thresholds: テスト用に `occupancy_threshold = 1_000_000`

## 触ってよいファイル（kimi の後続 PR）

`docs/20_design/detail/DD-D2-diffuse.md` のみ。BD-03/04/05 の「初期仮説（D2 で確定）」ラベル更新は basic 改訂として別 PR（writer=kimi、審査=grok）。

## 追記案（DD-D2 末尾へ。kimi が版を 0.2 にして貼る）

```
## 7. occupancy phase と θ_occ（確定。REQ-SIM-03a / REQ-SIM-03b）

本 DD スコープの「他 6 phase は D1 の格子拡張のみ」に occupancy が含まれる。振る舞いの新規定義は行わない。BD-03/04 が「D2 で確定」とした θ_occ の既定値と、格子上の適用手順をここで固定する。

- 既定値: `Thresholds.occupancy_threshold`（θ_occ）= 1_000_000（= 1.0 mass_u = FIXED_SCALE）。D1 既定を 64×64 でも採用。config で上書き可。未指定時はこの既定。参照: REQ-SIM-03a
- 走査: 全セル row-major。系統順は不要（セル合計のみ）。毎 tick の occupancy phase で 1 回。参照: BD-04 §2.1、BD-07
- 更新則（BD-03 occupancy INV、BD-04 §2.2）: `biomass_sum = Σ_L biomass[L]`。`biomass_sum ≥ θ_occ` なら `occupancy_peak = 1_000_000`。未満なら `occupancy_peak = mul(occupancy_peak, 995_000)`（×0.995、TowardZero）。下限 0 で飽和
- 台帳: occupancy は物質変換ではない。LedgerEntry を出さない
- 拡散: occupancy_peak は §1 どおり拡散対象外
- Vacant（導出。状態を変えない。REQ-SIM-03b）: `occupancy_peak > 300_000`（空き家判定線 0.3）かつ `biomass_sum < ε` かつ `nutrient > θ`。ε = 1e-4 × 初期総生体量、θ = 当該セル栄養の初期中央値の 10%（BD-04 §2.1 確定）。表示・説明器専用
- AT-D4-04 の依存: 空き家判定の機械定義は本節。代表 seed 選定は D7 のまま

### UT（実装は cursor-grok）

| UT-ID | 内容 | 期待 | 参照 |
|---|---|---|---|
| UT-D2-08 | 1 セル、biomass_sum ≥ θ_occ | occupancy_peak = 1_000_000 | REQ-SIM-03a |
| UT-D2-09 | 1 セル、biomass_sum < θ_occ、初期 peak = 1_000_000 | 1 tick 後 peak = 995_000 | BD-04 §2.2 |
| UT-D2-10 | Vacant 条件成立 | 判定は真、セル状態（peak 以外のプール）はビット一致 | REQ-SIM-03b |
```

## 提出

kimi が上記を DD-D2 0.2 として別 PR（`task/DD-D2-occupancy-001` とは別ブランチ）に載せる。本 brief PR の審査は Claude（記録。`docs/50_records/**` は anyOf claude/grok。writer=grok のため grok 票は使わない）。
