# BD-10 永続化（SaveEnvelope・版規則・migration）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）。責務分担の正本は BD-01 r3 §5（WorldSave / LedgerSave / SessionSave 表）、FFI 境界の正本は BD-05 §12、semver の正本は BD-05 §14。本章はそれらを保存設計に展開する
- 完成条件: 旧 save 読込テストの設計がある（§6）。各項目に REQ 参照
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する

## 1. 3 つの save の責務（BD-01 r3 §5 を正本として確定）

| save | 内容 | 書く側 | state hash に含むか | 参照 |
|---|---|---|---|---|
| **WorldSave** | schema_version / model_version / config_hash / seed / prng_state×4 / state_hash / state（tick・全セル・fixed_streak・tick0_ranking・inflow_cursor） | SimCore が直列化、Presentation がファイルへ | ◯（再計算不能なカウンタ類も正規化に含める） | REQ-DET-06, BD-05 §10 |
| **LedgerSave** | フロー台帳（region 単位）、スタンプ列、z 窓（10 tick 平均 × 20） | SimCore（台帳）／Explain（窓・スタンプ） | ✕（ただし台帳ダイジェストを三経路 AT で比較。ダイジェスト定義は BD-01 r3 §5 が正本: SHA-256、台帳エントリを tick, cell_index, lineage, reason, from, to, amount の順に LE 直列化 → スタンプを tick, kind, region_ids の順 → z 窓を pool, 系統, 値の順） | REQ-EVT-04, BD-05 §3 |
| **SessionSave** | 速度、due tick、画面、ひとこと仮説 3 値、経過秒 | Presentation | ✕ | REQ-UI-03, REQ-UI-05, REQ-UI-06 |

- WorldSave と LedgerSave は**同一トランザクション**で保存する（どちらかだけ古い状態を禁止）。LedgerSave は復帰後に**再計算しない**（イベント列一致のため）。SessionSave は別ファイルで、無くても World/Ledger から再開可能（UX が落ちるだけ）。確定。参照: REQ-UI-05, REQ-DET-07

## 2. SaveEnvelope フォーマット

- 直列化は **JSON（UTF-8）**。確定。理由: (a) save.schema.json による機械検査（AT-D8-02）がそのまま使える、(b) 整数は JSON で厳密に表せる（Fixed の生 i64 値をそのまま格納し浮動小数点を介さない）、(c) §5 の容量見積で 5 MB 予算内。参照: REQ-DET-06, REQ-EVT-04
- フィールド順は schema 定義順に固定（正規化。人が diff を読めるようにする）。確定
- `checksum` = 他の全フィールドを含む正規化 JSON バイト列の SHA-256（hex 64 文字）。`load` は checksum → schema_version → model_version の順に検証し、それぞれ `KZ_ERR_CHECKSUM` / `KZ_ERR_SCHEMA` / `KZ_ERR_MODEL_VERSION`。確定。参照: REQ-DET-06, REQ-CON-08, BD-05 §12.2
- `state`（WorldSave 本体）の構造: `tick: u32`、`cells`（row-major 4,096 件、各 `{nutrient, biomass[8], carcass, waste, energy[8], occupancy_peak}` の i64）、`fixed_streak: u32`、`tick0_ranking: [u8; 8]`、`inflow_cursor: u32`。確定。参照: REQ-SIM-01, BD-05 §1/§11
- save.schema.json に `checksum`（必須・hex64）と `state` の構造を追加する（本 PR で更新。現行は `state` が無定義のまま）。参照: REQ-DET-06

## 3. 版（bump）規則

BD-05 §14 を保存面で具体化する。確定。参照: REQ-NFR-06

| 版 | 対象 | bump 条件 | load の受理 |
|---|---|---|---|
| schema_version | 3 schema それぞれ | フィールド追加 = minor、削除・型変更 = major | 同 major かつ save の minor ≤ 実装の minor。major 相違は migration（§4）経由 |
| model_version | シミュレーションの振る舞い（係数・丸め・PRNG・hash・phase 規則） | 振る舞いが変わったら bump | **完全一致のみ受理**。不一致に migration は存在しない（振る舞い差は save 変換で埋められない）。`KZ_ERR_MODEL_VERSION` で拒否し、チュートリアル seed 再選定ゲート（REQ-DET-09、AT-D11-01）へ |

- config_hash は受理条件に含めない（同一 model で別 config の save は正当）。state_hash は load 後の再計算と照合し、不一致は `KZ_ERR_CHECKSUM`（破損検出の二重化）。確定。参照: REQ-DET-06

## 4. migration 方針

- schema major 変更にのみ migration を定義する。migration は `save_vN → save_vN+1` の純関数を chain し、各関数はフィールドの改名・追加既定値・削除のみを行う（**値の再計算は禁止**。再計算が必要な変更は model_version の bump であり migration 不可）。確定。参照: REQ-NFR-06
- migration 適用後の checksum・state_hash は新 schema で再計算して封入する。確定
- migration とその試験は schema 変更と同じ PR に含める（BD-05 §14）。確定。参照: REQ-NFR-06

## 5. 容量予算（REQ-EVT-04 / REQ-NFR-02 の 5 MB 内訳。初期仮説、D8 で実測確定）

| ファイル | 見積 | 根拠 |
|---|---|---|
| WorldSave（JSON） | ≤ 1.7 MB | 4,096 セル × 20 個の i64（最大 15 桁 + 区切り ≈ 20 B）≈ 1.6 MB + ヘッダ |
| LedgerSave | ≤ 0.3 MB | 台帳は region 単位（最大 16 region）・保存 32 件上限（REQ-EVT-02/04）+ z 窓 20 サンプル × 4 pool × 8 系統 |
| SessionSave | ≤ 2 KB | 速度・due tick・画面 ID・仮説 3 値・経過秒 |
| 生命史カード | ≤ 0.1 MB/件 | REQ-UI-07 の必須項目のみ |
| 合計 | ≤ 2.2 MB（2 世代ローテーションでも ≤ 4.4 MB） | 5 MB 予算内。確定は D8 の MEAS |

## 6. テスト設計（旧 save 読込を含む）

- **旧 save 読込テスト**: schema major を上げる PR は、旧版の save fixture を `docs/30_contracts/golden/saves/` に版名付きで残し、AT-D8-05 が fixture → migration → load の成功と state_hash 照合を機械判定する。fixture は golden なので更新は Claude 承認（BD-05 §14）。確定。参照: REQ-NFR-06, REQ-DET-06
- 破損・版不一致の否定系は AT-D8-02（checksum 破損 / schema 不一致 / model 不一致の 3 経路）で判定。確定。参照: REQ-DET-06
- 中断復帰の hash 連続は AT-D12-09 / AT-D12-RES-*、三経路は AT-D1-04 / AT-D8-01 が担保。本章は save 内容の正しさに責任を持ち、復帰手順は BD-11 に委譲する。参照: REQ-UI-05, REQ-DET-02
- 原子書込（一時ファイル → fsync → rename）と破損時に直前の正常 save へ戻る 2 世代ローテーションは Presentation の責務。書込途中 kill は AT-D12-ADV-07 がカバー。確定。参照: REQ-UI-05, BD-01 r3 §5
