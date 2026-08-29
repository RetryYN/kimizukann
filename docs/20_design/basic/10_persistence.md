# BD-10 永続化（SaveEnvelope・版規則・migration）

- 版: 0.3（起草 cursor-kimi、2026-08-30。r1: grok 審査反映 — envelope に config・ledger・ledger_hash・generation を同梱、prng_state を 4 語×10 進文字列化、正規化を一文で固定、state_hash 不一致に専用エラー。r2: BD-01 r4（#13 マージ、台帳二段モデル）に追従 — ledger_hash は r4 の LE ダイジェスト、schema の ledger 部を LedgerRecord フィールドに固定）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）。責務分担の正本は BD-01 r4 §5（WorldSave / LedgerSave / SessionSave 表、台帳ダイジェスト定義）、FFI 境界の正本は BD-05 §12、semver の正本は BD-05 §14。本章はそれらを保存設計に展開する
- 完成条件: 旧 save 読込テストの設計がある（§6）。各項目に REQ 参照
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する

## 1. save の責務と同梱構造（BD-01 r4 §5 を正本として確定）

| save | 内容 | 書く側 | state hash に含むか | 参照 |
|---|---|---|---|---|
| **WorldSave** | state（tick・全セル・fixed_streak・tick0_ranking・inflow_cursor）＋ config 本体 | SimCore が直列化、Presentation がファイルへ | ◯（再計算不能なカウンタ類も正規化に含める） | REQ-DET-06, BD-05 §10 |
| **LedgerSave** | フロー台帳（region 単位に集約した `LedgerRecord`）、スタンプ列、z 窓（10 tick 平均 × 20、値は i64 Fixed の平均であり f64 の z-score は保存しない） | SimCore（台帳）／Explain（窓・スタンプ） | ✕（ただし台帳ダイジェストを三経路 AT で比較。ダイジェスト定義は BD-01 r4 §5 が正本） | REQ-EVT-04, BD-05 §3 |
| **SessionSave** | 速度、due tick、画面、ひとこと仮説 3 値、経過秒 | Presentation | ✕ | REQ-UI-03, REQ-UI-05, REQ-UI-06 |

- **WorldSave と LedgerSave は 1 つの SaveEnvelope に同梱する**（§2）。「同一トランザクション」を構造で保証し、片側だけ新しい世代が残る故障を排除する。LedgerSave は復帰後に**再計算しない**（イベント列一致のため）。SessionSave は別ファイルで、無くても envelope から再開可能（UX が落ちるだけ）。確定。参照: REQ-UI-05, REQ-DET-07
- FFI の `save` / `load` がやり取りするバイト列はこの envelope 全体（BD-05 §12.1 を本 PR で更新）。参照: REQ-CON-01, REQ-DET-06

## 2. SaveEnvelope フォーマット

- 直列化は **JSON（UTF-8）**。確定。理由: (a) save.schema.json による機械検査（AT-D8-02）がそのまま使える、(b) 整数を浮動小数点を介さず格納できる、(c) §5 の容量見積で 5 MB 予算内。**WorldSave のパースと生成は core（Rust）のみが行い、Presentation（Dart）はバイト列を透過的に扱う**（Dart の `jsonDecode` は 2^53 超の整数で壊れるため）。確定。参照: REQ-DET-06, REQ-EVT-04
- **正規化（確定）**: 空白・改行なし、キー順は schema の required 定義順、整数は 10 進表記のみ（指数表記禁止）、u64 は 10 進文字列（下記）、`checksum` フィールド自身を除いたバイト列を checksum 入力とする
- u64（`seed`、`prng_state` の各語、`generation`）は **10 進文字列**で格納する（JSON number はパーサ依存のため）。schema の正規表現は 20 桁まで許すため 2^64−1 超を弾けない。範囲検査はパース時に実装側で行う。確定。参照: REQ-DET-06
- フィールド（必須 12。BD-05 §13・AT-D8-02 を本 PR で更新）: `schema_version` / `model_version` / `config_hash` / `config` / `seed` / `prng_state` / `state_hash` / `ledger_hash` / `generation` / `state` / `ledger` / `checksum`
  - `config`: config JSON 本体（正規化形式）。config_hash はこの正規化バイト列の SHA-256。4 レバー変更後の再開・複製（REQ-UI-08）に必要。確定
  - `prng_state`: 4 ストリーム × 各 `[u64; 4]`（xoshiro256**、BD-07 §2）を 10 進文字列 4 要素の配列で。確定。参照: REQ-DET-04a
  - `state_hash`: state の state hash（BD-05 §10、sha256-v2）。load 後に再計算して照合し、不一致は **`KZ_ERR_STATE_HASH`**（BD-05 §12.2 に追加する新値。checksum 破損・版不一致の 3 経路と区別する）。確定。参照: REQ-DET-06
  - `ledger_hash`: 台帳ダイジェスト（BD-01 r4 §5 が正本）= `SHA-256(region 集約後の LedgerRecord を tick→region_id→lineage→reason→from→to の順にソートし tick, region_id, lineage, reason, from, to, amount の順に LE 直列化 → スタンプを tick, kind, region_ids の順 → z 窓を pool, 系統, 値の順)`。load 時は ledger 部から **r4 の LE 直列化を再構成して**ダイジェストを再計算し突合する（envelope の JSON テキストを再出力するのではない。JSON は表現形式、hash 入力は r4 の LE バイト列）。確定。参照: REQ-DET-02, REQ-EVT-04
  - `generation`: 単調増加 u64（10 進文字列）。2 世代ローテーションの最新判定に使う（§6）。確定。参照: REQ-UI-05
  - `state`: `tick: u32`、`cells`（row-major。件数 = config の grid 寸法 width × height で、load 時に検証する）、`fixed_streak`、`tick0_ranking: [u8; 8]`、`inflow_cursor`。確定。参照: REQ-SIM-01, BD-05 §1/§11
  - `ledger`: region 集約レコード（`LedgerRecord { tick: u32, region_id: u8 (0..=15), lineage: u8, reason, from_pool, to_pool, amount: 和 }`、BD-01 r4 §5）・スタンプ列・z 窓（§1 の LedgerSave 行どおり）。各配列の要素 schema は save.schema.json で r4 のフィールドに固定する
  - `checksum`: 上記全フィールド（checksum 自身を除く）の正規化バイト列の SHA-256（hex 64）
- `load` の検証順: checksum → schema_version → model_version →（state・ledger 復元後）state_hash・ledger_hash。エラーはそれぞれ `KZ_ERR_CHECKSUM` / `KZ_ERR_SCHEMA` / `KZ_ERR_MODEL_VERSION` / `KZ_ERR_STATE_HASH`（ledger_hash 不一致も `KZ_ERR_STATE_HASH`）。確定。参照: REQ-CON-08, REQ-DET-06

## 3. 版（bump）規則

BD-05 §14 を保存面で具体化する。確定。参照: REQ-NFR-06

| 版 | 対象 | bump 条件 | load の受理 |
|---|---|---|---|
| schema_version | 3 schema それぞれ（semver 文字列。3 schema の `schema_version` に semver パターンを本 PR で追加） | フィールド追加 = minor、削除・型変更 = major | 同 major かつ save の minor ≤ 実装の minor。major 相違は migration（§4）経由 |
| model_version | シミュレーションの振る舞い（係数・丸め・PRNG・hash・phase 規則） | 振る舞いが変わったら bump | **完全一致のみ受理**。不一致に migration は存在しない（振る舞い差は save 変換で埋められない）。`KZ_ERR_MODEL_VERSION` で拒否し、チュートリアル seed 再選定ゲート（REQ-DET-09、AT-D11-01）へ |

- config_hash は受理条件に含めない（同一 model で別 config の save は正当。config 本体を同梱するので復元に外部 config を必要としない）。確定。参照: REQ-DET-06

## 4. migration 方針

- schema major 変更にのみ migration を定義する。migration は `save_vN → save_vN+1` の純関数を chain し、各関数はフィールドの改名・追加既定値・削除のみを行う（**値の再計算は禁止**。再計算が必要な変更は model_version の bump であり migration 不可）。確定。参照: REQ-NFR-06
- migration 適用後の checksum・state_hash・ledger_hash は新 schema で再計算して封入する。確定
- migration とその試験は schema 変更と同じ PR に含める（BD-05 §14）。確定。参照: REQ-NFR-06

## 5. 容量予算（REQ-EVT-04 / REQ-NFR-02 の 5 MB 内訳。初期仮説、D8 で実測確定）

| ファイル | 見積（最悪） | 根拠 |
|---|---|---|
| SaveEnvelope（WorldSave + LedgerSave の JSON） | ≤ 2.6 MB | 4,096 セル ×（20 個の i64 ≈ 20 B/個 + キー名 ≈ 15 B）≈ 2.3 MB + ledger ≤ 0.3 MB（region 16・保存 32 件上限・z 窓 20×4×8） |
| SessionSave | ≤ 2 KB | 速度・due tick・画面 ID・仮説 3 値・経過秒 |
| 生命史カード | ≤ 0.1 MB/件 | REQ-UI-07 の必須項目のみ |
| 合計 | 現行世代 + カードで ≤ 2.8 MB。2 世代ローテーションで ≤ 5.4 MB | **予算 5 MB を約 8% 超過しうる**。D8 の MEAS で実測し、超過なら (a) cells を平坦配列化してキー名を削る（schema minor）、(b) バイナリ化（schema major + migration、RFC）の順で検討する。初期仮説 |

## 6. テスト設計（旧 save 読込・世代管理を含む）

- **旧 save 読込テスト**: schema major を上げる PR は、旧版の save fixture を `docs/30_contracts/golden/saves/` に版名付きで残し、AT-D8-05 が fixture → migration → load の成功と state_hash 照合を機械判定する。fixture は golden なので更新は Claude 承認（BD-05 §14）。確定。参照: REQ-NFR-06, REQ-DET-06
- 破損・版不一致の否定系は AT-D8-02（checksum 破損 / schema 不一致 / model 不一致 / state_hash 不一致の 4 経路）で判定。確定。参照: REQ-DET-06
- **2 世代ローテーション（確定）**: envelope ファイルを `save.a.json` / `save.b.json` の 2 本とし、書込は古い側へ一時ファイル → fsync → rename（原子）。起動時は 2 本を読み、checksum 正常かつ `generation` 最大のものを採用、両方破損なら新規開始を提示。world/ledger の世代不整合は envelope 同梱により構造的に存在しない。参照: REQ-UI-05, BD-01 r4 §5
- 中断復帰の hash 連続は AT-D12-09 / AT-D12-RES-*、三経路は AT-D1-04 / AT-D8-01 が担保。本章は save 内容の正しさに責任を持ち、復帰手順は BD-11 に委譲する。参照: REQ-UI-05, REQ-DET-02
- 書込途中 kill は AT-D12-ADV-07 がカバー。確定。参照: REQ-UI-05
