# BD-01 コンテキストマップ（r5: D8-Q1 裁定反映 — 台帳ダイジェストの対象集合を 3 層保持ポリシに明示）

<!-- r2: grok 敵対レビュー 14 件反映 / r4: 台帳二段モデル（発生時 cell_index・digest/Save は region 集約）/ r5: 2026-08-30 D8-Q1 裁定 -->

参照: REQ-GOAL-02, REQ-CON-01, REQ-CON-05, REQ-CON-08, REQ-OUT-04, REQ-OUT-05, REQ-VIS-04, REQ-NFR-04, REQ-DET-02/06/07, REQ-EVT-04/05, REQ-END-03, REQ-UI-03/05

## 1. 境界づけられたコンテキスト
| コンテキスト | 責務 | 実体（crate / dir） | 持たないもの |
|---|---|---|---|
| **SimCore** | 決定論的な世界更新、保存則、終了判定、state hash、WorldSave の**直列化/復元**（バイト列 ⇄ 状態） | `crates/sim-types`, `crates/sim-core` | wall clock、乱数の外部入力、浮動小数点、**ファイル I/O**、描画、文章、トークン |
| **Explain** | 台帳ビュー→ドメインイベント→転換点スコア→理由コード→テンプレート文。**純関数** | `crates/sim-explain` | 状態の変更、SimCore 内部型への依存、生成 AI |
| **FFI（腐敗防止層）** | 7 操作の C ABI、handle 管理、再入禁止、バッファ規約、エラーコード | `crates/sim-ffi` | ロジック（core/explain を呼ぶだけ） |
| **Calibration** | seed バッチ、分布集計、上位互換判定、manifest、チュートリアル seed 候補抽出 | `crates/sim-cli`, `docs/calib/` | 世界更新のロジック、UI |
| **Presentation** | Flutter UI、再生スケジューラ、描画、**トークン生成（表示専用）**、SessionSave、カード保存、ファイル I/O（原子書込）、指標ログ | `app/` | シミュレーション計算、乱数、ネットワーク |
| **Distribution** | APK ビルド・署名・配布メモ（model_version・SHA-256）、テスター運用、検証レポート | `.github/workflows/flutter.yml`, `docs/dist/` | アプリのロジック |

## 2. 依存方向（矢印の向きにのみ依存してよい。F-01/F-02）
```
                sim-types
               ▲    ▲    ▲
               │    │    │
        sim-core  sim-explain (types + DTO のみ。core に依存しない)
               ▲    ▲
               │    │
             sim-ffi ◄────── app (Flutter)   ※ app は ffi にのみ依存。ffi が core と explain を並列に呼ぶ
               ▲
             sim-cli (Calibration。core 直呼びはせず ffi の Rust API を経由 → 経路を一本化)
```
- 逆依存禁止。検査: `crates/deny.toml`（bans。`cargo deny --manifest-path crates/Cargo.toml check bans`）＋ `scripts/check_deps.py`（各 Cargo.toml の dependencies を上の許可表と照合）。CI job `deps` は `.github/workflows/deps.yml`（本 PR）。`check_deps.py` の TRANSITIONAL（sim-cli→sim-core 直依存）は **sim-ffi 新設 PR（ADR-0008 実装）で空にする**ことを期限とする
- `sim-explain` は `sim-types` と DTO（`SnapshotView`, `LedgerView`）だけに依存。`&mut` を受ける API を持たない（型で読み取り専用を担保。F-01/F-03）
- `sim-cli` は `sim-ffi` の Rust 側 API（`sim_ffi::api`）を経由し、UI と同じ経路で save バイト列・エラーを扱う（F-02）

## 3. FFI 7 操作（許可された入出力の完全な一覧。これ以外は通らない。F-05/F-06/F-12）
| 操作 | 入力 | 出力 | バッファ | PRNG 消費 | hash への影響 |
|---|---|---|---|---|---|
| `create(config_json, seed)` | config UTF-8、u64 | handle または err | — | 初期化（SplitMix64）のみ | 初期状態を定める |
| `load(save_bytes)` | WorldSave | handle または err（checksum / schema_version / model_version 不一致） | 呼出側バッファ | 0（state を復元） | 復元した状態 |
| `step(handle, n)` | u32 | tick 後の終了ラベル（Option）または err | — | 各 phase の割当分のみ（BD-07） | 唯一 hash を進める操作 |
| `snapshot(handle, out)` | — | 固定レイアウト（BD-05 §Snapshot） | **固定長**、呼出側確保 | **0** | なし |
| `explain(handle, query, out)` | query JSON（event_id 等） | JSON | 可変長: 呼出側バッファ、不足時 `required_len` | **0** | なし（純関数、台帳は読み取りビュー） |
| `save(handle, out)` | — | WorldSave バイト列 | 可変長: 同上 | 0 | なし |
| `destroy(handle)` | — | ok | — | 0 | — |
- **再入禁止**: handle は同時 1 つ、全操作は同一スレッド、操作中の呼出は `ERR_BUSY`（F-11）。Presentation のスケジューラは説明表示中・save 中に `step` を発行しない
- エラーコードは enum（`ERR_SCHEMA / ERR_MODEL_VERSION / ERR_CHECKSUM / ERR_BUFFER(required_len) / ERR_BUSY / ERR_NUMERIC`）。数値・文字列の自由形式は返さない
- 「Presentation → core の入力」= この表の入力列がすべて。第 8 操作・未知フィールドは FFI が拒否

## 4. 決定性の責務（F-03/F-04/F-10/F-13/F-14）
| コンテキスト | 保証 | 検証（BD-08 の AT） |
|---|---|---|
| SimCore | 同一 config/seed → 同一 **(state hash, 終了ラベル, 台帳)**（三経路・クロス OS）。hash 正規化には継続カウンタ・inflow 消化位置を含める（ADR-0003 改定） | AT-D1/D2/D8: hash と終了ラベルと台帳ダイジェストの 3 つを比較 |
| Explain | 純関数 `(SnapshotView, LedgerView, query) -> bytes`。同一入力→同一出力、入力を変えない、イベントを消費しない | AT-D9: 2 回呼んで同一出力、呼出前後で hash・台帳ダイジェスト一致 |
| FFI | `snapshot / explain / save / destroy` は PRNG 消費 0、hash 不変 | AT-D12-FFI: 任意順序で挟んでも `step` 列だけで決まる hash に一致 |
| Presentation | 速度・一時停止・間引き・中断復帰・トークンは hash・終了ラベル・イベント列に影響しない。トークンは Presentation が snapshot から生成（core に無い） | AT-D12-SCH/RES（BD-11） |
| Calibration | 同一 seed 集合 → 同一 manifest | AT-D7 |
- 浮動小数点の逆流禁止: Explain の z-score（f64）は理由コードと文にだけ使い、SimCore の入力（閾値・終了・分岐）には一切使わない。`sim-core` と `sim-ffi` で f32/f64 を clippy disallow（F-10）
- 4 ストリームの用途は BD-07 で全部割り当て、「予備」を UI/Explain が借りることを禁止（F-04）

## 5. 保存の境界（何が save に必須で、何が再計算可能か。F-07/F-08）
| 保存物 | 中身 | 所有 | hash 対象 | 書込 |
|---|---|---|---|---|
| **WorldSave** | 契約 §10 の項目（schema_version, model_version, config_hash, seed, prng_state×4, state_hash, 全セル）＋ **再計算不能なコア状態**: tick、Fixed の継続カウンタ、tick 0 順位（Reversal 用）、InflowEvent の消化位置 | SimCore が直列化、Presentation がファイルへ | ◯（カウンタ類も正規化に含める） | Presentation が一時ファイル→fsync→rename の原子書込。破損時は直前の正常 save に戻す |
| **LedgerSave** | フロー台帳（region 単位）、スタンプ列、z 窓（10 tick 平均×20） | SimCore（台帳）／Explain（窓・スタンプ） | ✕（hash 外だが **台帳ダイジェスト** = SHA-256(集約後 LedgerRecord を tick, region_id, lineage, reason, from, to, amount の順に LE 直列化（発生時 `LedgerEntry` は cell_index 粒度。ダイジェスト/LedgerSave は tick 終了時に region へ集約した `LedgerRecord { tick: u32, region_id: u8 (0..=15。**region = 静的 4×4 タイル**（64×64 を 16×16 セルのタイル 16 枚、row-major で ID = (row/16)*4 + (col/16)。tick をまたいで安定。D3-Q1 判定 r2。スタンプの region_ids＝動的 4 連結成分とは幅のみ同じで意味は別。BD-12 §2）, lineage: u8, reason, from_pool, to_pool, amount: 和 }` を tick→region_id→lineage→reason→from→to の順にソートしたもの） → スタンプを tick, kind, region_ids の順 → z 窓を pool, 系統, 値の順) を三経路 AT で比較） | WorldSave と同じトランザクションで保存。**再計算しない**（復帰後のイベント列一致のため） |
| **SessionSave** | 速度、due tick、画面、ひとこと仮説 3 値、経過秒 | Presentation | ✕ | 別ファイル。無くても World/Ledger から再開可能（UX が落ちるだけ） |
| カード・指標ログ | 生命史カード、ローカル指標 | Presentation | ✕ | 別ファイル |
- 「save 完了を確認してから kill しても復帰できる」は AT-D12-RES で検証（BD-11 RES-01..04）
- **台帳ダイジェストの対象集合（D8-Q1 裁定 2026-08-30。確定）**: LedgerRecord の保持ポリシは 3 層 — (a) 累計レコード（tick を落とし region × lineage × reason × from × to で全期間集約）+ (b) 直近 200 tick の per-tick リングバッファ（常に直近 200 tick 分を保持し、201 tick 目の記録時に最古から FIFO で破棄）+ (c) スタンプ窓（転換点スタンプの検出確定 tick 時点の (b) リング全内容 = 確定 tick までの直近 200 tick 分の per-tick レコードをコピーして保存。保存数はスタンプ上限 32 件（BD-10 §5）に従う）—— とし、ダイジェスト入力はこの保持集合全体とする。**(b) と (c) の重複は除重しない**: (c) は複写元レコードが (b) から FIFO 退避した後も残すための独立した保存物であり、ダイジェストは保持集合の物理的内容をそのまま写す（複写元が (b) に残存する間は同一レコードが (b)(c) 双方に入力として現れる）。直列化順: (a) は tick フィールドを持たない累計レコードとして region_id→lineage→reason→from→to の順にソートして LE 直列化 → (b) は tick→region_id→lineage→reason→from→to の順にソート → (c) は窓の確定 tick 昇順・窓内は (b) と同順 → スタンプ（tick, kind, region_ids の順）→ z 窓（pool, 系統, 値の順）。参照: REQ-DET-02, REQ-EVT-04

## 6. 各境界を越えるデータ（契約の所在）
| 境界 | データ | 契約 |
|---|---|---|
| Presentation → sim-ffi | §3 の入力列 | BD-05 §FFI、`config.schema.json`、`save.schema.json` |
| sim-ffi → Presentation | §3 の出力列、エラー enum | BD-05 §FFI、`explanation_contract.md` |
| sim-core → sim-explain | `SnapshotView`, `LedgerView`（コピーアウト DTO、`sim-types`） | BD-05、BD-12 |
| Calibration → docs | manifest | BD-08（D6/D7 AT）、`result.schema.json` |
| Presentation → Distribution | APK、`model_version`、SHA-256 | REQ-OPS-05（配布メモは CI が生成） |

## 7. ADR 化する判断
- ADR-0007: sim-explain は独立 crate、types+DTO のみに依存（本 r2 で決定。「内部モジュール可」は撤回）
- ADR-0008: sim-ffi は独立 crate、sim-cli も ffi の Rust API を経由（本 r2 で決定）
- ADR-0003 改定: hash 正規化に Fixed 継続カウンタ・tick 0 順位・inflow 消化位置を追加（model_version の hash 部を `sha256-v2` へ）
