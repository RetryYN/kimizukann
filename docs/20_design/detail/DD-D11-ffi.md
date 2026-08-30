# DD-D11 詳細設計: FFI 境界と reference_scenarios ゲート

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 上位正本: BD-05 §12（FFI 7 操作・C ABI・エラー enum・事前/事後条件。シグネチャの正本）、BD-10 §2（envelope）、BD-11 §4（呼出順・失敗）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-CON-01/05/08、REQ-DET-06/09、REQ-ACC-05、REQ-NFR-06）、BD-08 §9（AT-D11-01/02）
- スコープ: FFI crate の実装構造（panic 捕捉・handle 管理・バッバッファプロトコル）、reference_scenarios の形式と model_version ゲート
- 非スコープ: FFI シグネチャ自体（BD-05 §12.3 が確定済み。本 DD は変更しない）、UI（DD-D12）
- 配置: 新規 crate `crates/sim-ffi`（cdylib）

## 1. FFI crate の実装構造（確定）

- `crates/sim-ffi` は `crate-type = ["cdylib", "staticlib"]`。公開シンボルは BD-05 §12.3 の 7 関数のみ（AT-D12-FFI-04: 公開面が 7 操作以外を持たない）
- 各関数は BD-05 §12.5 の事前/事後条件を守る。事前条件違反は対応する `KzError` を返し、UB に落とさない（null ポインタは `KZ_ERR_SCHEMA` ではなく呼出規約違反として即 return。初期仮説: `KZ_ERR_BUSY` とは区別し `KZ_ERR_INTERNAL`（D11-Q1））

### 1.1 panic 捕捉（D11-Q1）

- 全 7 関数を `catch_unwind` で包む。FFI 境界で panic を unwind させない（UB のため）
- 捕捉した panic の返却コードは `KzError` に無い → D11-Q1（推奨: `KZ_ERR_INTERNAL = 8` を semver minor で追加。BD-05 §12.2「値の追加は minor」に適合）

### 1.2 handle 管理（確定）

- `KzSim` は opaque。所有権は呼出側、`kz_destroy` で解放（BD-05 §12.1）
- 再入検出: handle 内の `AtomicBool`（操作中フラグ）。操作中の呼出は `KZ_ERR_BUSY`（BD-05 §12.2）
- 同時 handle 数は最大 1（BD-11 §4 の初期仮説をここで確定）。`kz_create` / `kz_load` は既存 handle 生存中は `KZ_ERR_BUSY` を返す

### 1.3 バッファプロトコル（確定。REQ-CON-08）

- 可変長出力（explain / save）は呼出側バッファ。不足時は `KZ_ERR_BUFFER` + `out_required_len` に必要バイト数。呼出側は再確保して 1 回だけ再呼出（AT-D12-FFI-01 の 2 段）
- ポインタを Dart へ渡さない（BD-11 §4）

## 2. reference_scenarios（確定。REQ-DET-09、REQ-ACC-05）

- 形式: `docs/30_contracts/reference_scenarios.json`。各シナリオ = `{ pattern_id, model_version, environment_id, seed, initial_state, expected: { end_label 分布・主要スタンプの種別 } }`
- 代表史は生命史パターン ID で管理し、seed 番号ではなくパターンに紐付く（v0.5 §7.6）。配置を変えた試験は別 ID
- `initial_state` は DD-D4 §3 の PlacementConfig + 初期生体量（REQ-ENV-04）

### 2.1 model_version ゲート（AT-D11-01。確定）

- CI で `reference_scenarios.json` の全シナリオの `model_version` を現在のビルドと照合。不一致が 1 件でもゲート失敗（終了コード 1）し、チュートリアル seed 再選定（REQ-DET-09）を強制する
- 再選定の手順: model_version を bump する PR は、同 PR 内で reference_scenarios を新 version で再実行・更新する（golden 更新のため Claude 承認。BD-05 §14）

### 2.2 チュートリアル seed 受入（AT-D11-02。確定）

- チュートリアル seed の一巡が: 主要スタンプ ≥ 1 件・10 分以内（REQ-UI-04a の一巡予算）・`evidence_refs` 非空・最小描画で転換点を視認可能（INSP 補助）
- 判定は `sim-cli` のシナリオ実行結果 JSON + INSP で行う

## 3. UT 設計（実数仕様）

| ID | 入力 | 期待 |
|---|---|---|
| UT-D11-01 | 全 7 関数に null handle / null バッファ | UB なくエラーコード返却 |
| UT-D11-02 | 意図的 panic fixture（core に panic 注入フック） | unwind せずエラーコード（D11-Q1 の値） |
| UT-D11-03 | handle 生存中に kz_create | `KZ_ERR_BUSY`（同時 1 確定） |
| UT-D11-04 | 操作中（模擬）に同一 handle へ kz_step | `KZ_ERR_BUSY` |
| UT-D11-05 | explain に cap = 0 | `KZ_ERR_BUFFER` + required_len > 0、再呼出で成功 |
| UT-D11-06 | reference_scenarios の model_version を 1 件改変 | ゲート失敗・終了コード 1（AT-D11-01 の UT 版） |

## 4. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `crates/sim-ffi/src/lib.rs` | 7 関数・catch_unwind・handle 管理（§1） |
| `docs/30_contracts/reference_scenarios.json` | §2（初回は D7 較正後に登録。枠のみ） |
| `scripts/check_reference_scenarios.py` | §2.1 のゲート |
| `crates/sim-ffi/tests/d11_ffi.rs` | §3 UT |

## 5. 未決事項（claude 裁定依頼）

- **D11-Q1**: FFI 境界で捕捉した panic の返却コード。推奨: `KZ_ERR_INTERNAL = 8` を semver minor で `KzError` に追加（BD-05 §12.2 の「値の追加は minor」に適合。事前条件違反の null ポインタも同コードに寄せる）
