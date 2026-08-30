# DD-D11 詳細設計: FFI 境界と reference_scenarios ゲート

- 版: 0.2（起草 cursor-kimi、2026-08-30。0.2: grok 審査 r1（PR #37）反映 — destroy の panic 挙動・busy フラグの finally クリア・同一スレッド確定・2 段バッファの required_len 不変条件・panic fixture を cfg(test) 限定・trace 誤配線の修正。D11-Q1 裁定（KZ_ERR_INTERNAL=8 採用、2026-08-30）を反映）
- 上位正本: BD-05 §12（FFI 7 操作・C ABI・エラー enum・事前/事後条件。シグネチャの正本）、BD-10 §2（envelope）、BD-11 §4（呼出順・失敗）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-CON-01/08、REQ-DET-09、REQ-ACC-05）、BD-08 §9（AT-D11-01/02）
- スコープ: FFI crate の実装構造（panic 捕捉・handle 管理・バッファプロトコル）、reference_scenarios の形式と model_version ゲート
- 非スコープ: FFI シグネチャ自体（BD-05 §12.3 が確定済み。本 DD は変更しない）、UI（DD-D12）
- 配置: 新規 crate `crates/sim-ffi`（cdylib）

## 1. FFI crate の実装構造（確定）

- `crates/sim-ffi` は `crate-type = ["cdylib", "staticlib"]`。公開シンボルは BD-05 §12.3 の 7 関数のみ（AT-D12-FFI-04: 公開面が 7 操作以外を持たない）
- 各関数は BD-05 §12.5 の事前/事後条件を守る。事前条件違反は対応する `KzError` を返し、UB に落とさない。null ポインタ（handle・バッファ・`out_required_len`）は `KZ_ERR_SCHEMA` でも `KZ_ERR_BUSY` でもなく呼出規約違反として **`KZ_ERR_INTERNAL = 8`** を即 return する（D11-Q1 裁定 2026-08-30 採用。確定）

### 1.1 panic 捕捉（確定。D11-Q1 裁定 2026-08-30 採用）

- 全 7 関数を `catch_unwind` で包む。FFI 境界で panic を unwind させない（UB のため）
- 捕捉した panic の返却コードは **`KZ_ERR_INTERNAL = 8`**（`KzError` への追加は semver minor。BD-05 §12.2「値の追加は minor」に適合）。**BD-05 §12 の KzError 追加は basic 改訂 PR（semver minor）として DD-D11 より先にマージする**（裁定の条件）
- panic 捕捉時は操作中フラグを必ずクリアしてから return する（finally 相当。クリアしないと handle が永久に `KZ_ERR_BUSY` になる）。プロセスは殺さない
- `kz_destroy` は `void` でエラーコードを返せないため、内部で panic を捕捉した場合は操作中フラグと handle を解放して黙って return する（UB に落とさない・プロセスを殺さない、の一文で固定）
- テスト用の意図的 panic 注入は core の製品経路にフラグを残さない。`cfg(test)` 専用フック、または sim-ffi のテスト内への差し込みに限定する（UT-D11-02）

### 1.2 handle 管理（確定）

- `KzSim` は opaque。所有権は呼出側、`kz_destroy` で解放（BD-05 §12.1）。二重 destroy は「無視して成功」（BD-11 §4.2）— void なので 2 回目も正常 return する
- 呼出スレッド: BD-05 の同一スレッド確定に従い、**他スレッド・別 isolate からの呼出は未定義**（FFI 利用側の責任で同一スレッドに集約する）。同一スレッド再入の検出は handle 内の操作中フラグ（`Cell<bool>` で足りる。check-then-set の競合は同一スレッド前提では起きない）とし、操作中の呼出は `KZ_ERR_BUSY`（BD-05 §12.2）
- 同時 handle 数は最大 1（BD-11 §4 の初期仮説をここで確定）。`kz_create` / `kz_load` は既存 handle 生存中は `KZ_ERR_BUSY` を返す

### 1.3 バッファプロトコル（確定。REQ-CON-08）

- 可変長出力（explain / save）は呼出側バッファ。不足時は `KZ_ERR_BUFFER` + `out_required_len` に必要バイト数。呼出側は再確保して 1 回だけ再呼出（AT-D12-FFI-01 の 2 段）
- `out_required_len` は 1 回目と 2 回目の呼出で増えない（explain / save は純関数で、同一状態からは常に同一長）。`out_required_len == null` は呼出規約違反（`KZ_ERR_INTERNAL`。§1）
- 固定長の snapshot も cap 不足は `KZ_ERR_BUFFER`（必要長は §12.4 の `16 + width × height × 96` で一意。DD-D10 §1）
- ポインタを Dart へ渡さない（BD-11 §4）

## 2. reference_scenarios（確定。REQ-DET-09、REQ-ACC-05）

- 形式: `docs/30_contracts/reference_scenarios.json`。各シナリオ = `{ pattern_id, model_version, environment_id, seed, initial_state, expected: { end_label 分布・主要スタンプの種別 } }`
- 代表史は生命史パターン ID で管理し、seed 番号ではなくパターンに紐付く（v0.5 §7.6）。配置を変えた試験は別 ID
- `initial_state` は DD-D4 §3 の PlacementConfig + 初期生体量。`expected` の「end_label 分布・主要スタンプの種別」は枠のみ定義し、要素 schema は初回登録時（D7 較正後）に確定する

### 2.1 model_version ゲート（AT-D11-01。確定）

- CI で `reference_scenarios.json` の全シナリオの `model_version` を現在のビルドと照合。不一致が 1 件でもゲート失敗（終了コード 1）し、チュートリアル seed 再選定（REQ-DET-09）を強制する
- 再選定の手順: model_version を bump する PR は、同 PR 内で reference_scenarios を新 version で再実行・更新する（golden 更新のため Claude 承認。BD-05 §14）

### 2.2 チュートリアル seed 受入（AT-D11-02。確定）

- チュートリアル seed の一巡が: 主要スタンプ ≥ 1 件・一巡 10 分以内の予算（BD-11）・`evidence_refs` 非空・最小描画で転換点を視認可能（INSP 補助）
- 判定は `sim-cli` のシナリオ実行結果 JSON + INSP で行う

## 3. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D11-01 | 全 7 関数に null handle / null バッファ / null `out_required_len` | UB なく `KZ_ERR_INTERNAL = 8` 返却（呼出規約違反。D11-Q1 採用値で固定） |
| UT-D11-02 | 意図的 panic 注入（`cfg(test)` 専用フックまたは sim-ffi テスト内差し込み。製品経路にフラグを残さない） | unwind せず `KZ_ERR_INTERNAL = 8`・操作中フラグはクリア済み |
| UT-D11-03 | handle 生存中に kz_create | `KZ_ERR_BUSY`（同時 1 確定） |
| UT-D11-04 | 操作中（模擬）に同一 handle へ kz_step | `KZ_ERR_BUSY` |
| UT-D11-05 | explain に cap = 0 / snapshot に cap = 0 | ともに `KZ_ERR_BUFFER` + required_len > 0（snapshot は `16 + width × height × 96`）、再呼出で成功。required_len は再呼出で増えない |
| UT-D11-06 | reference_scenarios の model_version を 1 件改変 | ゲート失敗・終了コード 1（AT-D11-01 の UT 版） |
| UT-D11-07 | kz_destroy を 2 回呼ぶ | 2 回目も正常 return（無視して成功。BD-11 §4.2） |

## 4. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-ffi/src/lib.rs` | 7 関数・catch_unwind・handle 管理（§1） |
| `docs/30_contracts/reference_scenarios.json` | §2（初回は D7 較正後に登録。枠のみ） |
| `scripts/check_reference_scenarios.py` | §2.1 のゲート |
| `crates/sim-ffi/tests/d11_ffi.rs` | §3 UT |

## 5. 裁定記録（claude 2026-08-30）

- **D11-Q1（採用）**: FFI 境界で捕捉した panic および呼出規約違反（null ポインタ等）の返却コードは `KZ_ERR_INTERNAL = 8` を semver minor で `KzError` に追加する。条件: BD-05 §12 の KzError 追加は basic 改訂 PR（semver minor）として DD-D11 より先にマージする。§1/§1.1/§3 に確定値として反映済み
