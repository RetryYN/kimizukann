# DD-D6 詳細設計: 煙試験（4 環境 × 20 seed バッチ）

- 版: 0.2（起草 cursor-kimi、2026-08-30。0.2: grok 審査 r1（PR #31）反映 — blocker（UT-D6-03 と elapsed_ms の両立不能）・major 5 件を修正）
- 上位正本: `docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-OPS-02a、REQ-OUT-01、REQ-SIM-14、REQ-DET-03）、v0.5 §7.2、BD-03（INV-07）、BD-05 §10、BD-06 §3、BD-08 §6（AT-D6-01/02）
- スコープ: 煙試験バッチの実行仕様・seed 導出・分布 JSON 形式・失敗時の終了コード
- 非スコープ: 較正ゲート本体（D7 = 100 seed + 上位互換判定）、分布帯の合否判定（REQ-END-05 は D7）、CI ワークフロー定義（H0 済み。本 DD はバッチコマンドの契約のみ）
- 依存: D3（系統 phase）・D4（プリセット・終了判定）・D5（環境プリセット）の実装マージ後に実装可能

## 1. 目的と完了判定（確定。REQ-OPS-02a、v0.5 §7.2）

4 環境 × 20 seed、各 2,000 tick を実行し、**panic・NumericError 0 件で分布 JSON を返す**こと。分布の中身の合否は問わない（D7 の較正ゲートが担当）。系統数不変（REQ-OUT-01、INV-07）を全 run で検査する。

## 2. バッチ実行仕様

### 2.1 コマンド

`sim-cli batch --suite smoke`（初期仮説。`verify --suite all` とは別コマンドとし、verify は従来どおり単一 run の検査群を担当する）

- 入力: 環境プリセット 4 件（DD-D5）、DD-D4 既定プリセット・既定配置、`max_ticks = 2_000`
- 出力: 分布 JSON（§3）を stdout または `--out <path>` へ。終了コード §4

### 2.2 seed 導出（確定）

- `seed(i) = batch_base + i`（i = 0..19、batch_base = 42 の初期仮説）。seed 列は config のみの決定的関数とし、PRNG ストリームの消費とは無関係（各 run は独立した SplitMix64 導出を行う。BD-05 §6）
- seed 列を JSON に記録し、再実行時の同一性を担保する

### 2.3 並列化（確定。契約 §6）

- seed 間バッチのみ並列化（run 内は単一スレッド）。ワーカー数は `--jobs`（既定 = 論理コア数）
- 並列実行は結果に影響しない（run 間で状態を共有しない）。出力 JSON の run レコード順は `environment_id` の ASCII 辞書順（`carcass_pulse` < `center_rich` < `edge_sparse` < `local_waste`）→ `seed` 昇順にソートし、ワーカー完了順に依存しない

### 2.4 時間予算（初期仮説）

- 1 run = 2,000 tick。NFR-01（headless 全体 ≤ 1.0 s / 2,000 tick）から 1 run ≤ 1.0 s を目標とし、80 run を 4 並列で ≤ 30 s。超過時は失敗ではなく JSON の `elapsed_ms` に記録して D7 の較正材料とする

## 3. 分布 JSON 形式（確定）

```json
{
  "suite": "smoke",
  "model_version": "<実装時の World.model_version を書く>",
  "batch_base": 42,
  "max_ticks": 2000,
  "runs": [
    {
      "environment_id": "carcass_pulse",
      "seed": 42,
      "status": "ok",
      "end_label": "TimeLimit",
      "ticks_run": 2000,
      "final_shares": [250000, 250000, 250000, 250000],
      "lineage_count": 4,
      "state_hash": "<64hex>",
      "elapsed_ms": 1234
    }
  ],
  "summary": {
    "per_environment": {
      "center_rich": { "Extinct": 0, "Fixed": 3, "Coexist": 12, "Reversal": 1, "TimeLimit": 4 }
    },
    "panic_count": 0,
    "numeric_error_count": 0,
    "lineage_count_violations": 0
  }
}
```

- `final_shares` は系統 id 昇順の Fixed（百万分率）。`lineage_count` は系統配列の**スロット数**（INV-07: Absent/Dead で縮めない。分岐もないため常に初期配列長と等しいはず）と定義する。違反条件は `lineage_count != 初期配列長`（INV-07 の等号条件を採用。REQ-OUT-01「初期値を超えない」を含意する stronger 条件）
- `status` は `ok | panic | numeric_error | lineage_violation` の 4 値。失敗時の必須フィールド:

| status | end_label | final_shares | state_hash | ticks_run |
|---|---|---|---|---|
| ok | 必須 | 必須 | 必須 | 必須（= 終了 tick） |
| panic / numeric_error | 省略 | 省略 | 省略 | 必須（異常発生までの tick 数） |
| lineage_violation | 必須 | 必須 | 必須 | 必須 |

- `state_hash` を全正常 run に記録し、同一 seed の再実行一致（REQ-DET-03 の CI xos 検査）に使う
- schema は `docs/30_contracts/batch_result.schema.json`（実装 PR で新設）

## 4. 失敗時の挙動（確定）

| 事象 | 挙動 |
|---|---|
| いずれかの run が panic | バッチは残りを最後まで実行し、JSON に記録。終了コード 1 |
| いずれかの run が NumericError | 同上（REQ-SIM-14: 煙試験で panic 0 が合格線。NumericError も 0 が合格線） |
| `lineage_count != 初期配列長` | 同上（INV-07 違反として記録。終了コード 1） |
| 環境プリセットの load 失敗（存在しない environment_id・設計上限超過 config を含む） | 即時終了コード 2（バッチ不成立。run は未実行） |

panic の捕捉はワーカー側で `catch_unwind` 相当を用い、プロセスを殺さず記録する（初期仮説）

## 5. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D6-01 | batch_base = 42 | seed 列 = 42..61（20 件、重複なし） |
| UT-D6-02 | 同一 config で batch 2 回 | 全 run の state_hash が一致（REQ-DET-03） |
| UT-D6-03 | jobs = 1 と jobs = 4 | 出力 JSON から `elapsed_ms` を除いた canonical JSON がバイト一致（レコード順が決定的。`elapsed_ms` は壁時計由来のため比較対象外） |
| UT-D6-04 | テスト hook で run 中に NumericError を注入 | 終了コード 1、`status: "numeric_error"` で JSON に記録、他 run は完走（上限超過 config は create 時 ValidationError であり NumericError ではない。BD-06 §5。上限超過は load 失敗 = 終了コード 2 側） |
| UT-D6-05 | 存在しない environment_id | 終了コード 2、run 0 件 |
| UT-D6-06 | テスト hook で run 中に panic を注入 | 終了コード 1、`status: "panic"` で JSON に記録、他 run は完走 |

## 6. AT 対応（BD-08 §6）

| AT | 対応 |
|---|---|
| AT-D6-01 | 全 80 run の `lineage_count` を集計し初期配列長と異なるものを検出（§3 summary.lineage_count_violations） |
| AT-D6-02 | CI の**独立 job** として `sim-cli batch --suite smoke` を直接実行し、終了コード 0 + JSON schema 通過を合否とする。`verify --suite all` / `verify --suite <Dn>` には含めない（verify は単一 run の検査群のまま。80 run を verify に入れる解釈は CI を壊す） |

## 7. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-cli/src/batch.rs` | バッチランナ（§2〜§4） |
| `crates/sim-cli/src/main.rs` | `batch` サブコマンド追加のみ |
| `docs/30_contracts/batch_result.schema.json` | §3 schema |
| `crates/sim-cli/tests/d6_smoke.rs` | §5 UT |

## 8. 未決事項

なし（D6-Q なし。batch_base = 42・時間予算は初期仮説として本 DD で閉じ、D7 の較正で見直す）
